use crate::{engine::Engine, files, metadata, model::*, planner};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

struct StagingFile(PathBuf);
impl Drop for StagingFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub async fn execute_file(
    engine: &Engine,
    plan: &PlannedFile,
    job_id: &str,
    checkpoint: impl Fn(&FileResult) -> Result<(), String>,
) -> Result<FileResult, String> {
    if let Some(error) = &plan.error {
        return Err(error.clone());
    }
    if plan.operations.is_empty() && plan.rename_to.is_none() {
        return Ok(FileResult {
            file_id: plan.file.id.clone(),
            path: plan.target.clone(),
            status: "skipped".into(),
            message: "No changes needed".into(),
            backup: None,
            backup_fingerprint: None,
            after: None,
            original_path: None,
            warnings: plan.warnings.clone(),
            differences: vec![],
        });
    }
    check_external(plan)?;
    let target = Path::new(&plan.target);
    files::ensure_writable(target)?;
    let parent = target.parent().ok_or("Target has no parent directory")?;
    let temporary = parent.join(format!(
        ".tagryn-{}.{}",
        uuid::Uuid::new_v4(),
        files::extension(target)
    ));
    let _guard = StagingFile(temporary.clone());
    if target.exists() {
        files::copy_new(target, &temporary)?;
    } else {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        file.write_all(b"<?xpacket begin='' id='W5M0MpCehiHzreSzNTczkc9d'?><x:xmpmeta xmlns:x='adobe:ns:meta/'><rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'><rdf:Description rdf:about=''/></rdf:RDF></x:xmpmeta><?xpacket end='w'?>").map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
    }
    let mut warnings = plan.warnings.clone();
    if !plan.operations.is_empty() {
        let mut args = vec![
            "-overwrite_original".into(),
            "-charset".into(),
            "filename=UTF8".into(),
            "-charset".into(),
            "IPTC=UTF8".into(),
        ];
        for operation in &plan.operations {
            append_arguments(&mut args, operation)?;
        }
        args.push(files::path_string(&temporary)?);
        let output = engine.execute(&args).await?;
        if !output.warnings.is_empty() {
            warnings.push(output.warnings);
        }
    }
    let (tags, engine_warnings) = metadata::read_tags(engine, &temporary, "embedded").await?;
    warnings.extend(engine_warnings);
    for operation in &plan.operations {
        verify_operation(&tags, operation)?;
    }
    if plan.privacy.is_some() {
        let residuals = planner::sensitive_residuals(&tags);
        if !residuals.is_empty() {
            warnings.push(format!(
                "Sensitive or preview tags remain: {}",
                residuals.join(", ")
            ));
        }
        warnings.push(
            "Known metadata was checked again. This is not a guarantee of complete anonymization."
                .into(),
        );
    }
    // Verify the display-critical metadata has survived all cleanup operations.
    if plan.privacy.is_some() {
        let (before, _) = metadata::read_tags(engine, target, "embedded").await?;
        for tag in before
            .iter()
            .filter(|t| t.name == "Orientation" || t.group == "ICC_Profile")
        {
            if !tags.iter().any(|t| t.key == tag.key && t.raw == tag.raw) {
                return Err(format!(
                    "Display-critical metadata changed: {}. Original left untouched.",
                    tag.key
                ));
            }
        }
    }
    let final_hash = files::fingerprint(&temporary)?;
    let backup_dir = parent.join(".tagryn-backups").join(job_id);
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let backup = if target.exists() {
        let backup = backup_dir.join(target.file_name().ok_or("Invalid filename")?);
        files::copy_new(target, &backup)?;
        let expected = plan
            .target_expected
            .as_ref()
            .ok_or("Missing expected target fingerprint")?;
        if files::fingerprint(&backup)?.sha256 != expected.sha256 {
            return Err("Backup verification failed. Original left untouched.".into());
        }
        Some(files::path_string(&backup)?)
    } else {
        None
    };
    check_external(plan)?;
    let destination = plan.rename_to.as_deref().unwrap_or(&plan.target);
    let mut result = FileResult {
        file_id: plan.file.id.clone(),
        path: destination.into(),
        status: "writing".into(),
        message: "Verified candidate and backup; committing file".into(),
        backup,
        backup_fingerprint: plan.target_expected.clone(),
        after: Some(final_hash),
        original_path: Some(plan.target.clone()),
        warnings,
        differences: plan.differences.clone(),
    };
    // Journal backup and intended output hash durably BEFORE changing the source.
    checkpoint(&result)?;
    if destination != plan.target || !target.exists() {
        // hard_link fails if the destination already exists, including a racing rename.
        fs::hard_link(&temporary, destination)
            .map_err(|e| format!("Could not create destination without overwriting: {e}"))?;
        if destination != plan.target {
            if let Err(error) = fs::remove_file(target) {
                result.warnings.push(format!(
                    "Renamed copy exists, but source could not be removed: {error}"
                ));
            }
        }
    } else {
        #[cfg(not(windows))]
        fs::rename(&temporary, target)
            .map_err(|e| format!("Replacement failed; backup retained: {e}"))?;
        #[cfg(windows)]
        {
            // The Windows rename API cannot replace an existing file. Keep the verified backup and journal before replacement.
            let retired = backup_dir.join(format!(
                "retired-{}",
                target
                    .file_name()
                    .ok_or("Invalid filename")?
                    .to_string_lossy()
            ));
            fs::rename(target, &retired).map_err(|e| e.to_string())?;
            if let Err(e) = fs::rename(&temporary, target) {
                let _ = fs::rename(&retired, target);
                return Err(format!("Replacement failed: {e}; backup retained"));
            }
            let _ = fs::remove_file(retired);
        }
    }
    let after = files::fingerprint(Path::new(destination))?;
    if after.sha256 != result.after.as_ref().ok_or("Missing output hash")?.sha256 {
        return Err("Post-write integrity check failed. Recover from the journaled backup.".into());
    }
    // Re-read the actual committed file, not just the temporary candidate.
    let (reread, final_warnings) =
        metadata::read_tags(engine, Path::new(destination), "embedded").await?;
    for operation in &plan.operations {
        verify_operation(&reread, operation)?;
    }
    result.warnings.extend(final_warnings);
    result.after = Some(after);
    result.status = if result.warnings.is_empty() {
        "success"
    } else {
        "warning"
    }
    .into();
    result.message = "Saved, reread and verified".into();
    Ok(result)
}

fn append_arguments(args: &mut Vec<String>, operation: &WriteOp) -> Result<(), String> {
    if !operation
        .tag
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ":-_".contains(c))
    {
        return Err("Invalid controlled tag selector".into());
    }
    let raw = if operation.value.is_number() { "#" } else { "" };
    if operation.value.is_null() {
        args.push(format!("-{}=", operation.tag));
    } else if let Some(list) = operation.value.as_array() {
        args.push(format!("-{}=", operation.tag));
        for value in list {
            args.push(format!(
                "-{}+={}",
                operation.tag,
                value.as_str().ok_or("Expected list of strings")?
            ));
        }
    } else {
        args.push(format!(
            "-{}{raw}={}",
            operation.tag,
            value_text(&operation.value)
        ));
    }
    Ok(())
}

pub fn verify_operation(tags: &[Tag], operation: &WriteOp) -> Result<(), String> {
    if operation.tag.ends_with(":all") {
        let group = operation.tag.trim_end_matches(":all");
        if tags.iter().any(|t| t.group == group || t.location == group) {
            return Err(format!("Verification failed: {group} still exists"));
        }
        return Ok(());
    }
    let matching: Vec<_> = tags
        .iter()
        .filter(|t| {
            metadata::tag_selector(t) == operation.tag
                || (!operation.tag.contains(':') && t.name == operation.tag)
        })
        .collect();
    if operation.value.is_null() {
        if matching.is_empty() {
            return Ok(());
        }
    } else if !matching.is_empty()
        && matching.iter().all(|t| {
            planner::equivalent(&t.raw, &operation.value)
                || planner::equivalent(&t.formatted, &operation.value)
        })
    {
        return Ok(());
    }
    Err(format!("Verification failed for {}. Expected {}. Original is preserved or a journaled backup is available.", operation.tag, value_text(&operation.value)))
}

pub fn check_external(plan: &PlannedFile) -> Result<(), String> {
    let path = Path::new(&plan.file.path);
    if fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("Source became a symbolic link".into());
    }
    if files::fingerprint(path)? != plan.expected {
        return Err("External file modification detected. Reload and review again.".into());
    }
    let current_sidecar = files::existing_sidecar(path)
        .map(|p| files::fingerprint(&p))
        .transpose()?;
    if current_sidecar != plan.sidecar_expected {
        return Err("Sidecar changed externally. Reload and resolve its values.".into());
    }
    let target = Path::new(&plan.target);
    if target.exists()
        && fs::symlink_metadata(target)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err("Writing through a sidecar symbolic link is not supported".into());
    }
    let actual = if target.exists() {
        Some(files::fingerprint(target)?)
    } else {
        None
    };
    if actual != plan.target_expected {
        return Err("Write target changed externally".into());
    }
    if plan
        .rename_to
        .as_ref()
        .is_some_and(|name| Path::new(name).exists())
    {
        return Err("Rename destination already exists".into());
    }
    Ok(())
}

pub fn restore(result: &FileResult) -> Result<String, String> {
    let path = Path::new(&result.path);
    if fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("Restore target became a symbolic link".into());
    }
    let expected = result
        .after
        .as_ref()
        .ok_or("No committed fingerprint is available for this result")?;
    if files::fingerprint(path)?.sha256 != expected.sha256 {
        return Err(
            "File changed after this job. Automatic restore would overwrite newer work.".into(),
        );
    }
    let original = Path::new(
        result
            .original_path
            .as_deref()
            .ok_or("Original path unavailable")?,
    );
    if original != path && original.exists() {
        return Err("Original filename is occupied. Restore aborted.".into());
    }
    files::ensure_writable(path)?;
    if let Some(backup) = &result.backup {
        let backup = Path::new(backup);
        if fs::symlink_metadata(backup)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("Backup became a symbolic link".into());
        }
        let recorded = result.backup_fingerprint.as_ref().ok_or(
            "This old journal entry has no backup fingerprint; automatic restore is unavailable",
        )?;
        if files::fingerprint(backup)?.sha256 != recorded.sha256 {
            return Err("Backup changed since it was created. Restore aborted.".into());
        }
    }
    let recovery = path
        .parent()
        .ok_or("No parent")?
        .join(".tagryn-backups")
        .join(format!("restore-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&recovery).map_err(|e| e.to_string())?;
    let undone = recovery.join(path.file_name().ok_or("No filename")?);
    files::copy_new(path, &undone)?;
    if let Some(backup) = &result.backup {
        let backup = Path::new(backup);
        let temporary = recovery.join("restoring.tmp");
        files::copy_new(backup, &temporary)?;
        if files::fingerprint(&temporary)?.sha256 != files::fingerprint(backup)?.sha256 {
            return Err("Restore copy verification failed".into());
        }
        if files::fingerprint(path)?.sha256 != expected.sha256 {
            return Err("File changed during restore preparation".into());
        }
        if original == path {
            #[cfg(windows)]
            {
                let retired = recovery.join("retired-before-restore");
                fs::rename(path, &retired).map_err(|e| e.to_string())?;
                if let Err(error) = fs::rename(&temporary, original) {
                    let _ = fs::rename(&retired, path);
                    return Err(format!(
                        "Restore replacement failed: {error}; recovery copies retained"
                    ));
                }
            }
            #[cfg(not(windows))]
            fs::rename(&temporary, original).map_err(|e| e.to_string())?;
        } else {
            fs::hard_link(&temporary, original).map_err(|e| e.to_string())?;
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        if files::fingerprint(original)?.sha256 != files::fingerprint(backup)?.sha256 {
            return Err("Restored file does not match backup".into());
        }
    } else {
        // Undo creation of a sidecar, preserving the saved sidecar in the recovery folder.
        if files::fingerprint(path)?.sha256 != expected.sha256 {
            return Err("File changed during restore preparation".into());
        }
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(format!(
        "Restored. The undone version is preserved at {}",
        undone.display()
    ))
}
