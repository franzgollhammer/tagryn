// Synthetic throughput benchmark; excluded from product bundles.
use serde_json::json;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tagryn_lib::{files, service::Service};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().ok_or("No root")?.to_path_buf();
        let fixtures=root.join("artifacts/fixtures");let directory=root.join("artifacts/bench-fixtures").join(uuid::Uuid::new_v4().to_string());std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
        let seed=fixtures.join("Alpine Licht_001 – Österreich.jpg");
        for i in 0..10000 { let target=directory.join(format!("Scan_{i:05}.jpg")); if !target.exists(){std::fs::hard_link(&seed,&target).map_err(|e|e.to_string())?;} }
        let service=Arc::new(Service::new(root.join("src-tauri/resources/runtime"),root.join(".build/benchmark-state"))?);
        let start=Instant::now();let id=service.scan(None,vec![files::path_string(&directory)?],true)?;
        loop {let done=service.jobs.lock().map_err(|e|e.to_string())?.get(&id).is_some_and(|j|j.status!="running");if done{break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        let scan_ms=start.elapsed().as_millis();let count=service.files.lock().map_err(|e|e.to_string())?.len();
        let ids:Vec<_>=service.files.lock().map_err(|e|e.to_string())?.keys().take(200).cloned().collect();
        let start=Instant::now();let mut tags=0;
        for batch in ids.chunks(2) {let (a,b)=futures_util::future::join(service.document(&batch[0],true),service.document(&batch[1],true)).await;tags+=a?.tags.len()+b?.tags.len();}
        let metadata_ms=start.elapsed().as_millis();
        let mut entries:Vec<_>=service.files.lock().map_err(|e|e.to_string())?.values().cloned().collect(); entries.sort_by(|a,b|a.name.cmp(&b.name));
        let first=service.document(&entries[0].id,true).await?;
        std::fs::write(root.join("public/review-large.json"),serde_json::to_vec(&json!({"entries":entries,"files":[first]})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        let tiff=service.register(&fixtures.join("24MP-Studiotest.tiff"))?;let start=Instant::now();let _=service.document(&tiff.id,true).await?;let tiff_ms=start.elapsed().as_millis();
        let start=Instant::now();let _=tagryn_lib::preview::preview(service.clone(),&tiff.id).await?;let preview_ms=start.elapsed().as_millis();
        let output=json!({"profile":if cfg!(debug_assertions){"debug"}else{"release"},"architecture":std::env::consts::ARCH,"os":std::env::consts::OS,"files":count,"scanMs":scan_ms,"scanFilesPerSecond":count as f64/(scan_ms as f64/1000.0),"metadataFiles":ids.len(),"metadataMs":metadata_ms,"metadataFilesPerSecond":ids.len() as f64/(metadata_ms as f64/1000.0),"tags":tags,"tiffBytes":tiff.size,"tiffReadMs":tiff_ms,"tiffPreviewMs":preview_ms,"dataset":"10,000 actual directory entries hard-linked to a synthetic JPEG; 200 fresh engine reads; generated 24MP TIFF. Not representative of a mixed camera-original library."});
        std::fs::write(root.join("artifacts/benchmark.json"),serde_json::to_vec_pretty(&output).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        println!("{output}");service.engine.shutdown().await;Ok::<_,String>(())
    }).map_err(Into::into)
}
