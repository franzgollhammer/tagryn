use std::{path::PathBuf, sync::Arc};
use tagryn_lib::{files, model::*, service::Service};
pub struct Fixture {
    pub service: Arc<Service>,
    pub directory: tempfile::TempDir,
}
impl Fixture {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary directory");
        let service = Arc::new(
            Service::new(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/runtime"),
                directory.path().join("state"),
            )
            .expect("service"),
        );
        Self { service, directory }
    }
    pub fn file(&self, name: &str) -> FileEntry {
        let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../artifacts/fixtures/Alpine Licht_001 – Österreich.jpg");
        let target = self.directory.path().join(name);
        files::copy_new(&input, &target).expect("Run npm run fixtures before testing");
        self.service.register(&target).expect("register")
    }
    pub async fn request(&self, file: &FileEntry, edits: Vec<Edit>) -> ChangeRequest {
        let doc = self
            .service
            .document(&file.id, true)
            .await
            .expect("read metadata");
        ChangeRequest {
            file_id: file.id.clone(),
            expected: doc.fingerprint,
            sidecar_expected: doc.sidecar_fingerprint,
            edits,
            privacy: None,
            sync_legacy: false,
            force_sidecar: false,
            rename: None,
        }
    }
}
pub fn run(future: impl std::future::Future<Output = ()>) {
    tokio::runtime::Runtime::new()
        .expect("runtime")
        .block_on(future);
}
