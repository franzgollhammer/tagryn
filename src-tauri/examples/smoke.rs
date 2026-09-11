// Focused acceptance executable; excluded from product bundles.
use serde_json::json;
use std::{path::PathBuf, sync::Arc, time::Instant};
use tagryn_lib::{files, jobs, metadata, model::*, service::Service};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().ok_or("No project root")?.to_path_buf();
        let input = root.join("artifacts/fixtures/Alpine Licht_001 – Österreich.jpg");
        let temporary = std::env::temp_dir().join(format!("tagryn-smoke-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temporary).map_err(|e| e.to_string())?;
        let working = temporary.join("Änderung – 東京.jpg");
        files::copy_new(&input, &working)?;
        let start = Instant::now();
        let service = Arc::new(Service::new(root.join("src-tauri/resources/runtime"), temporary.join("state"))?);
        let file = service.register(&working)?;
        let doc = service.document(&file.id, true).await?;
        let read_ms = start.elapsed().as_millis();
        let before = doc.fingerprint.clone();
        let request = ChangeRequest { file_id: file.id.clone(), expected: doc.fingerprint, sidecar_expected: doc.sidecar_fingerprint, edits: vec![Edit {field:"title".into(), value:json!("Geprüft – 東京\n$(touch never-execute)"), mode:"replace".into()}], privacy:None, sync_legacy:false, force_sidecar:false, rename:None };
        let plan = service.plan(vec![request]).await?;
        let planned = plan.files.first().ok_or("Missing planned file")?;
        if let Some(error) = &planned.error { return Err(error.clone()); }
        let write_start = Instant::now();
        let result = jobs::execute_file(&service.engine, planned, "smoke", |result| {
            std::fs::write(temporary.join("checkpoint.json"), serde_json::to_vec_pretty(result).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
        }).await?;
        let reread = service.document(&file.id,true).await?;
        let value = metadata::values(&reread.tags, "XMP-dc:Title", "embedded");
        if value != json!("Geprüft – 東京\n$(touch never-execute)") { return Err(format!("Unexpected written title: {value}")); }
        let backup = result.backup.as_ref().ok_or("Backup missing")?;
        if files::fingerprint(std::path::Path::new(backup))?.sha256 != before.sha256 { return Err("Backup does not match original".into()); }
        let write_ms = write_start.elapsed().as_millis();
        let restored = jobs::restore(&result)?;
        if files::fingerprint(&working)?.sha256 != before.sha256 { return Err("Restore hash mismatch".into()); }
        let mut documents = Vec::new();
        for entry in std::fs::read_dir(root.join("artifacts/fixtures")).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if files::extension(&path) == "jpg" && !path.file_name().unwrap_or_default().to_string_lossy().starts_with("Beschädigt") {
                let file = service.register(&path)?; documents.push(service.document(&file.id,true).await?);
            }
        }
        documents.sort_by(|a,b| a.file.name.cmp(&b.file.name));
        std::fs::write(root.join("public/review-session.json"),serde_json::to_vec_pretty(&json!({"files":documents})).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        service.engine.shutdown().await;
        println!("{}",json!({"result":"passed","checks":["real metadata read","group-qualified identities","Unicode path","multiline and shell-like literal value","plan","safe save","reread","byte-identical backup","byte-identical restore"],"firstReadMs":read_ms,"writeAndVerifyMs":write_ms,"fixtureDirectory":temporary,"restore":restored}));
        Ok::<_,String>(())
    }).map_err(Into::into)
}
