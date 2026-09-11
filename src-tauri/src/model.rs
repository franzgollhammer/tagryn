use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Fingerprint {
    pub size: u64,
    pub modified_ns: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub id: String,
    pub path: String,
    pub name: String,
    pub folder: String,
    pub extension: String,
    pub size: u64,
    pub modified: String,
    pub readonly: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub key: String,
    pub group: String,
    pub location: String,
    pub name: String,
    pub label: String,
    pub raw: Value,
    pub formatted: Value,
    pub source: String,
    pub writable: bool,
    pub derived: bool,
    pub field: Option<String>,
    pub format: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub read: bool,
    pub write: String,
    pub preview: String,
    pub format: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub file: FileEntry,
    pub fingerprint: Fingerprint,
    pub tags: Vec<Tag>,
    pub capabilities: Capabilities,
    pub sidecar: Option<String>,
    pub sidecar_fingerprint: Option<Fingerprint>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub field: String,
    pub value: Value,
    #[serde(default = "replace")]
    pub mode: String,
}
fn replace() -> String {
    "replace".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRequest {
    pub file_id: String,
    pub expected: Fingerprint,
    pub sidecar_expected: Option<Fingerprint>,
    pub edits: Vec<Edit>,
    #[serde(default)]
    pub privacy: Option<String>,
    #[serde(default)]
    pub sync_legacy: bool,
    #[serde(default)]
    pub force_sidecar: bool,
    #[serde(default)]
    pub rename: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Difference {
    pub tag: String,
    pub before: Value,
    pub after: Value,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteOp {
    pub tag: String,
    pub value: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedFile {
    pub file: FileEntry,
    pub expected: Fingerprint,
    pub sidecar_expected: Option<Fingerprint>,
    pub target: String,
    pub target_expected: Option<Fingerprint>,
    pub operations: Vec<WriteOp>,
    pub differences: Vec<Difference>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
    pub rename_to: Option<String>,
    pub privacy: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub files: Vec<PlannedFile>,
    pub created: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    pub file_id: String,
    pub path: String,
    pub status: String,
    pub message: String,
    pub backup: Option<String>,
    pub after: Option<Fingerprint>,
    pub original_path: Option<String>,
    #[serde(default)]
    pub backup_fingerprint: Option<Fingerprint>,
    pub warnings: Vec<String>,
    pub differences: Vec<Difference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub total: usize,
    pub completed: usize,
    pub errors: usize,
    pub results: Vec<FileResult>,
    pub started: String,
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
pub fn value_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        _ => value.to_string(),
    }
}
