use crate::model::{Capabilities, FileEntry, Fingerprint};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

pub const RAW: &[&str] = &[
    "dng", "cr2", "cr3", "nef", "nrw", "arw", "sr2", "srf", "raf", "orf", "rw2", "pef", "rwl",
    "3fr", "iiq", "kdc", "mos", "mrw", "x3f",
];
pub fn is_raw(path: &Path) -> bool {
    RAW.contains(&extension(path).as_str())
}
pub fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_lowercase()
}
pub fn supported(path: &Path) -> bool {
    RAW.contains(&extension(path).as_str())
        || [
            "jpg", "jpeg", "tif", "tiff", "png", "webp", "heic", "heif", "avif", "xmp", "mp4",
            "mov", "m4v", "mkv", "avi", "mp3", "m4a", "wav", "flac", "ogg", "pdf",
        ]
        .contains(&extension(path).as_str())
}
pub fn path_string(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or("Non-UTF-8 paths are not supported by this build".into())
}
pub fn fingerprint(path: &Path) -> Result<Fingerprint, String> {
    let before = fs::metadata(path).map_err(|e| e.to_string())?;
    if !before.is_file() {
        return Err("Expected a regular file".into());
    }
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 131072];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let after = file.metadata().map_err(|e| e.to_string())?;
    if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
        return Err("File changed while reading".into());
    }
    Ok(Fingerprint {
        size: after.len(),
        modified_ns: after
            .modified()
            .map_err(|e| e.to_string())?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
            .to_string(),
        sha256: format!("{:x}", hash.finalize()),
    })
}
pub fn entry(path: &Path, id: String) -> Result<FileEntry, String> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let md = fs::metadata(&path).map_err(|e| e.to_string())?;
    if !md.is_file() {
        return Err("Expected regular file".into());
    }
    Ok(FileEntry {
        id,
        path: path_string(&path)?,
        name: path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid filename encoding")?
            .into(),
        folder: path_string(path.parent().ok_or("Missing parent directory")?)?,
        extension: extension(&path),
        size: md.len(),
        modified: chrono::DateTime::<chrono::Utc>::from(md.modified().map_err(|e| e.to_string())?)
            .to_rfc3339(),
        readonly: md.permissions().readonly(),
    })
}
pub fn capabilities(path: &Path, actual_format: &str) -> Capabilities {
    let ext = extension(path);
    let format = actual_format.to_uppercase();
    let embedded = ["JPEG", "TIFF", "PNG", "WEBP", "HEIC", "HEIF", "AVIF", "XMP"]
        .contains(&format.as_str())
        && !is_raw(path);
    Capabilities {
        read: true,
        write: if is_raw(path) {
            "sidecar"
        } else if embedded {
            "embedded"
        } else {
            "readOnly"
        }
        .into(),
        preview: if ["jpg", "jpeg", "png", "webp", "tiff", "tif"].contains(&ext.as_str()) {
            "native"
        } else if is_raw(path) {
            "embedded"
        } else if cfg!(target_os = "macos") && ["heic", "heif", "avif"].contains(&ext.as_str()) {
            "native"
        } else {
            "unavailable"
        }
        .into(),
        format,
    }
}
pub fn sidecar_path(path: &Path) -> PathBuf {
    path.with_extension("xmp")
}
pub fn existing_sidecar(path: &Path) -> Option<PathBuf> {
    if extension(path) == "xmp" {
        return None;
    }
    [path.with_extension("xmp"), path.with_extension("XMP")]
        .into_iter()
        .find(|p| p.is_file())
}
pub fn ensure_writable(path: &Path) -> Result<(), String> {
    if path.exists()
        && fs::metadata(path)
            .map_err(|e| e.to_string())?
            .permissions()
            .readonly()
    {
        return Err("File is read-only".into());
    }
    let parent = path.parent().ok_or("Missing parent directory")?;
    if fs::metadata(parent)
        .map_err(|e| e.to_string())?
        .permissions()
        .readonly()
    {
        return Err("Directory is read-only".into());
    }
    Ok(())
}
pub fn copy_new(from: &Path, to: &Path) -> Result<(), String> {
    let mut input = File::open(from).map_err(|e| e.to_string())?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(to)
        .map_err(|e| e.to_string())?;
    std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
    output.flush().map_err(|e| e.to_string())?;
    output.sync_all().map_err(|e| e.to_string())?;
    fs::set_permissions(
        to,
        fs::metadata(from).map_err(|e| e.to_string())?.permissions(),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 220
        || name == "."
        || name == ".."
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return Err("Filename contains unsupported characters or is too long".into());
    }
    let stem = name.split('.').next().unwrap_or_default().to_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.ends_with(|c: char| c.is_ascii_digit()))
    {
        return Err("Reserved Windows filename".into());
    }
    Ok(())
}
