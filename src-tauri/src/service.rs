use crate::{db::Database, engine::Engine, files, jobs, metadata, model::*, planner};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;

pub struct Service {
    pub engine: Engine,
    pub database: Database,
    pub files: Mutex<HashMap<String, FileEntry>>,
    pub plans: Mutex<HashMap<String, Plan>>,
    pub paths: Mutex<HashMap<String, String>>,
    pub jobs: Mutex<HashMap<String, Job>>,
    pub cancellation: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub previews: Mutex<HashMap<String, String>>,
    pub preview_slots: Semaphore,
    pub write_slot: Semaphore,
    pub os_paths: Mutex<Vec<String>>,
}
impl Service {
    pub fn new(runtime: PathBuf, data: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&data).map_err(|e| e.to_string())?;
        let database = Database::open(&data.join("tagryn.sqlite"))?;
        let history = database
            .jobs()?
            .into_iter()
            .map(|j| (j.id.clone(), j))
            .collect();
        Ok(Self {
            engine: Engine::new(runtime),
            database,
            files: Mutex::new(HashMap::new()),
            paths: Mutex::new(HashMap::new()),
            plans: Mutex::new(HashMap::new()),
            jobs: Mutex::new(history),
            cancellation: Mutex::new(HashMap::new()),
            previews: Mutex::new(HashMap::new()),
            preview_slots: Semaphore::new(1),
            write_slot: Semaphore::new(1),
            os_paths: Mutex::new(vec![]),
        })
    }
    pub fn file(&self, id: &str) -> Result<FileEntry, String> {
        self.files
            .lock()
            .map_err(|e| e.to_string())?
            .get(id)
            .cloned()
            .ok_or("File has not been opened in this workspace".into())
    }
    pub fn register(&self, path: &Path) -> Result<FileEntry, String> {
        let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
        let string = files::path_string(&path)?;
        let mut registry = self.files.lock().map_err(|e| e.to_string())?;
        let mut paths = self.paths.lock().map_err(|e| e.to_string())?;
        let id = paths
            .get(&string)
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let file = files::entry(&path, id.clone())?;
        registry.insert(id, file.clone());
        paths.insert(string, file.id.clone());
        Ok(file)
    }
    pub fn relocate(&self, id: &str, path: &Path) -> Result<FileEntry, String> {
        let updated = files::entry(path, id.into())?;
        let mut registry = self.files.lock().map_err(|e| e.to_string())?;
        let mut paths = self.paths.lock().map_err(|e| e.to_string())?;
        if let Some(previous) = registry.insert(id.into(), updated.clone()) {
            paths.remove(&previous.path);
        }
        paths.insert(updated.path.clone(), id.into());
        Ok(updated)
    }
    pub async fn document(&self, id: &str, refresh: bool) -> Result<Document, String> {
        let registered = self.file(id)?;
        let file = files::entry(Path::new(&registered.path), id.into())?;
        let path = file.path.clone();
        let key = tokio::task::spawn_blocking(move || {
            let path = Path::new(&path);
            let embedded = files::fingerprint(path)?;
            let sidecar = files::existing_sidecar(path)
                .map(|p| files::fingerprint(&p))
                .transpose()?;
            Ok::<_, String>(format!(
                "{}:{}:{}",
                embedded.sha256,
                embedded.modified_ns,
                serde_json::to_string(&sidecar).map_err(|e| e.to_string())?
            ))
        })
        .await
        .map_err(|e| e.to_string())??;
        if !refresh {
            if let Some(mut cached) = self.database.cache_get(&file.path, &key) {
                cached.file = file;
                self.database.cache_put(&cached, &key)?;
                return Ok(cached);
            }
        }
        let document = metadata::read_document(&self.engine, file).await?;
        self.database.cache_put(&document, &key)?;
        Ok(document)
    }
    pub fn update_job(&self, app: Option<&AppHandle>, job: &Job) -> Result<(), String> {
        self.database.save_job(job)?;
        self.jobs
            .lock()
            .map_err(|e| e.to_string())?
            .insert(job.id.clone(), job.clone());
        if let Some(app) = app {
            app.emit("tagryn:job", job).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn scan(
        self: &Arc<Self>,
        app: Option<AppHandle>,
        paths: Vec<String>,
        recursive: bool,
    ) -> Result<String, String> {
        if paths.len() > 10000 {
            return Err("Open at most 10,000 roots per request".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancellation
            .lock()
            .map_err(|e| e.to_string())?
            .insert(id.clone(), cancel.clone());
        let mut job = Job {
            id: id.clone(),
            kind: "scan".into(),
            status: "running".into(),
            total: 0,
            completed: 0,
            errors: 0,
            results: vec![],
            started: now(),
        };
        self.update_job(app.as_ref(), &job)?;
        let service = self.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let mut pending: Vec<(PathBuf, bool)> = paths
                .into_iter()
                .map(|p| (PathBuf::from(p), true))
                .collect();
            let mut visited = HashSet::new();
            let mut batch = Vec::new();
            while let Some((path, root)) = pending.pop() {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                if job.completed >= 100000 {
                    job.errors += 1;
                    job.results
                        .push(scan_error(&path, "Workspace scan limited to 100,000 files"));
                    break;
                }
                let outcome = (|| -> Result<(), String> {
                    let md = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
                    if md.file_type().is_symlink() && !root {
                        return Ok(());
                    }
                    let canonical = fs::canonicalize(&path).map_err(|e| e.to_string())?;
                    if !visited.insert(canonical.clone()) {
                        return Ok(());
                    }
                    if canonical.is_dir() {
                        if root || recursive {
                            for entry in fs::read_dir(&canonical).map_err(|e| e.to_string())? {
                                let entry = entry.map_err(|e| e.to_string())?;
                                if entry.file_name().to_string_lossy().starts_with('.') {
                                    continue;
                                }
                                pending.push((entry.path(), false));
                            }
                        }
                    } else if files::supported(&canonical) {
                        batch.push(service.register(&canonical)?);
                        job.completed += 1;
                        job.total = job.completed;
                    }
                    Ok(())
                })();
                if let Err(error) = outcome {
                    job.errors += 1;
                    job.results.push(scan_error(&path, &error));
                }
                if batch.len() >= 128 {
                    if let Some(app) = &app {
                        let _ = app.emit("tagryn:files", &batch);
                    }
                    batch.clear();
                    let _ = service.update_job(app.as_ref(), &job);
                }
            }
            if !batch.is_empty() {
                if let Some(app) = &app {
                    let _ = app.emit("tagryn:files", &batch);
                }
            }
            job.status = if cancel.load(Ordering::Relaxed) {
                "cancelled"
            } else if job.errors > 0 {
                "partial"
            } else {
                "completed"
            }
            .into();
            let _ = service.update_job(app.as_ref(), &job);
            if let Ok(mut map) = service.cancellation.lock() {
                map.remove(&job.id);
            }
        });
        Ok(id)
    }
    pub async fn plan(&self, requests: Vec<ChangeRequest>) -> Result<Plan, String> {
        if requests.is_empty() || requests.len() > 10000 {
            return Err("Plan must contain between 1 and 10,000 files".into());
        }
        let mut plan = Plan {
            id: uuid::Uuid::new_v4().to_string(),
            files: vec![],
            created: now(),
        };
        let mut seen = HashSet::new();
        let mut targets = HashSet::new();
        for request in requests {
            if !seen.insert(request.file_id.clone()) {
                return Err("Duplicate source file in batch".into());
            }
            let file = self.file(&request.file_id)?;
            let result = match self.document(&request.file_id, true).await {
                Ok(document) => planner::build_file_plan(&document, &request),
                Err(e) => Err(e),
            };
            let mut planned = result.unwrap_or_else(|error| PlannedFile {
                file: file.clone(),
                expected: request.expected,
                sidecar_expected: request.sidecar_expected,
                target: file.path.clone(),
                target_expected: None,
                operations: vec![],
                differences: vec![],
                warnings: vec![],
                error: Some(error),
                rename_to: None,
                privacy: request.privacy,
            });
            let target = planned
                .rename_to
                .as_ref()
                .unwrap_or(&planned.target)
                .to_lowercase();
            if !targets.insert(target) {
                planned.error =
                    Some("Two files in this batch would use the same destination".into());
            }
            plan.files.push(planned);
        }
        let mut plans = self.plans.lock().map_err(|e| e.to_string())?;
        if plans.len() >= 16 {
            plans.clear();
        }
        plans.insert(plan.id.clone(), plan.clone());
        Ok(plan)
    }
    pub fn start_job(
        self: &Arc<Self>,
        app: Option<AppHandle>,
        plan_id: String,
    ) -> Result<String, String> {
        let plan = self
            .plans
            .lock()
            .map_err(|e| e.to_string())?
            .remove(&plan_id)
            .ok_or("Plan expired or was already executed. Review changes again.")?;
        let cancel = Arc::new(AtomicBool::new(false));
        let mut job = Job {
            id: uuid::Uuid::new_v4().to_string(),
            kind: "write".into(),
            status: "queued".into(),
            total: plan.files.len(),
            completed: 0,
            errors: 0,
            results: vec![],
            started: now(),
        };
        let id = job.id.clone();
        self.cancellation
            .lock()
            .map_err(|e| e.to_string())?
            .insert(id.clone(), cancel.clone());
        self.update_job(app.as_ref(), &job)?;
        let service = self.clone();
        tauri::async_runtime::spawn(async move {
            let _write_guard = service.write_slot.acquire().await;
            job.status = "running".into();
            for file in plan.files {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let checkpoint = |result: &FileResult| {
                    let mut snapshot = job.clone();
                    snapshot.results.push(result.clone());
                    service.update_job(app.as_ref(), &snapshot)
                };
                let outcome = jobs::execute_file(&service.engine, &file, &job.id, checkpoint).await;
                let result = match outcome {
                    Ok(result) => result,
                    Err(error) => {
                        job.errors += 1;
                        let saved = service.jobs.lock().ok().and_then(|map| {
                            map.get(&job.id).and_then(|j| {
                                j.results
                                    .iter()
                                    .find(|r| r.file_id == file.file.id)
                                    .cloned()
                            })
                        });
                        let mut result = saved.unwrap_or(FileResult {
                            file_id: file.file.id.clone(),
                            path: file.target.clone(),
                            status: "error".into(),
                            message: String::new(),
                            backup: None,
                            backup_fingerprint: None,
                            after: None,
                            original_path: None,
                            warnings: file.warnings,
                            differences: file.differences,
                        });
                        result.status = "error".into();
                        result.message = error;
                        result
                    }
                };
                if ["success", "warning"].contains(&result.status.as_str())
                    && file.rename_to.is_some()
                {
                    if let Ok(updated) = service.relocate(&file.file.id, Path::new(&result.path)) {
                        if let Some(app) = &app {
                            let _ = app.emit("tagryn:files", vec![updated]);
                        }
                    }
                }
                job.results.push(result);
                job.completed += 1;
                if service.update_job(app.as_ref(), &job).is_err() {
                    job.status = "interrupted".into();
                    break;
                }
            }
            job.status = if job.status == "interrupted" {
                "interrupted"
            } else if cancel.load(Ordering::Relaxed) {
                "cancelled"
            } else if job.errors > 0 {
                "partial"
            } else {
                "completed"
            }
            .into();
            let _ = service.update_job(app.as_ref(), &job);
            if let Ok(mut map) = service.cancellation.lock() {
                map.remove(&job.id);
            }
        });
        Ok(id)
    }
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let map = self.cancellation.lock().map_err(|e| e.to_string())?;
        let flag = map.get(id).ok_or("Job is no longer running")?;
        flag.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub fn history(&self) -> Result<Vec<Job>, String> {
        self.database.jobs()
    }
    pub fn settings(&self, key: &str) -> Result<Option<Value>, String> {
        self.database.get(key)
    }
}
fn scan_error(path: &Path, error: &str) -> FileResult {
    FileResult {
        file_id: String::new(),
        path: path.to_string_lossy().into(),
        status: "error".into(),
        message: error.into(),
        backup: None,
        backup_fingerprint: None,
        after: None,
        original_path: None,
        warnings: vec![],
        differences: vec![],
    }
}
