// Disposable camera-original check; excluded from product bundles.
use serde_json::json;
use std::{path::PathBuf, sync::Arc, time::Instant};
use tagryn_lib::{files, jobs, metadata, model::*, service::Service};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().ok_or("Missing root")?.to_path_buf();
        let temporary = std::env::temp_dir().join(format!("tagryn-camera-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temporary).map_err(|e| e.to_string())?;
        let service = Arc::new(Service::new(root.join("src-tauri/resources/runtime"), temporary.join("state"))?);
        let mut measurements = Vec::new();
        for name in ["sony-a7v.ARW", "nikon-d2x.NEF"] {
            let input = root.join("artifacts/samples").join(name);
            let working = temporary.join(name);
            files::copy_new(&input, &working)?;
            let file = service.register(&working)?;
            let start = Instant::now();
            let document = service.document(&file.id, true).await?;
            let read_ms = start.elapsed().as_millis();
            if document.capabilities.write != "sidecar" { return Err("Camera RAW did not choose sidecar".into()); }
            let start = Instant::now();
            let preview = tagryn_lib::preview::preview(service.clone(), &file.id).await?;
            let preview_ms = start.elapsed().as_millis();
            let request = ChangeRequest { file_id:file.id.clone(), expected:document.fingerprint.clone(), sidecar_expected:None, edits:vec![Edit { field:"copyright".into(), value:json!("Tagryn disposable acceptance check"), mode:"replace".into() }], privacy:None, sync_legacy:false, force_sidecar:false, rename:None };
            let plan = service.plan(vec![request]).await?;
            let saved = jobs::execute_file(&service.engine, &plan.files[0], "camera", |_| Ok(())).await?;
            let reread = service.document(&file.id, true).await?;
            if metadata::values(&reread.tags, "XMP-dc:Rights", "sidecar") != json!("Tagryn disposable acceptance check") { return Err("Sidecar verification failed".into()); }
            if files::fingerprint(&working)?.sha256 != document.fingerprint.sha256 { return Err("RAW original bytes changed".into()); }
            jobs::restore(&saved)?;
            if files::existing_sidecar(&working).is_some() { return Err("New sidecar was not restored to absence".into()); }
            let start = Instant::now();
            for _ in 0..10 { service.document(&file.id, true).await?; }
            measurements.push(json!({"file":name,"bytes":file.size,"tags":document.tags.len(),"firstReadMs":read_ms,"previewMs":preview_ms,"previewAvailable":preview.is_some(),"tenFreshReadsMs":start.elapsed().as_millis(),"originalSha256":document.fingerprint.sha256,"sidecarSaveAndRestore":"passed; original bytes unchanged"}));
        }
        let result = json!({"profile":if cfg!(debug_assertions){"debug"}else{"release"},"source":"https://raw.pixls.us/ (CC0)","samples":measurements,"result":"passed"});
        std::fs::write(root.join("artifacts/camera-check.json"), serde_json::to_vec_pretty(&result).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        println!("{result}"); service.engine.shutdown().await; Ok::<_,String>(())
    }).map_err(Into::into)
}
