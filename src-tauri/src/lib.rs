pub mod db;
pub mod engine;
pub mod files;
pub mod jobs;
pub mod menu;
pub mod metadata;
pub mod model;
pub mod planner;
pub mod preview;
pub mod service;

use model::*;
use serde_json::{json, Value};
use service::Service;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{Emitter, Manager};

type AppState = Arc<Service>;
static STARTED: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
#[tauri::command]
fn frontend_ready() -> u128 {
    let ms = STARTED
        .get()
        .map(|i| i.elapsed().as_millis())
        .unwrap_or_default();
    eprintln!(
        "TAGRYN_READY {}",
        json!({"startupMs":ms,"pid":std::process::id(),"version":env!("CARGO_PKG_VERSION")})
    );
    ms
}
#[tauri::command]
fn frontend_diagnostic(message: String) {
    eprintln!(
        "Tagryn UI: {}",
        message.chars().take(2000).collect::<String>()
    );
}
#[tauri::command]
async fn open_paths(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
    recursive: bool,
) -> Result<String, String> {
    state.inner().scan(Some(app), paths, recursive)
}
#[tauri::command]
async fn search_metadata(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Value, String> {
    let service = state.inner().clone();
    tokio::task::spawn_blocking(move || {
        let paths: std::collections::HashSet<_> = service.database.search(&query)?.into_iter().collect();
        let indexed: std::collections::HashSet<_> = service.database.search("")?.into_iter().collect();
        let files = service.files.lock().map_err(|e| e.to_string())?;
        let ids: Vec<_> = files.values().filter(|file| paths.contains(&file.path)).map(|file| file.id.clone()).collect();
        Ok(json!({"ids":ids,"indexed":files.values().filter(|file| indexed.contains(&file.path)).count()}))
    }).await.map_err(|e| e.to_string())?
}
#[tauri::command]
async fn read_metadata(
    state: tauri::State<'_, AppState>,
    file_id: String,
    refresh: bool,
) -> Result<Document, String> {
    state.document(&file_id, refresh).await
}
#[tauri::command]
async fn read_preview(
    state: tauri::State<'_, AppState>,
    file_id: String,
) -> Result<Option<String>, String> {
    preview::preview(state.inner().clone(), &file_id).await
}
#[tauri::command]
async fn plan_changes(
    state: tauri::State<'_, AppState>,
    requests: Vec<ChangeRequest>,
) -> Result<Plan, String> {
    state.plan(requests).await
}
#[tauri::command]
async fn execute_plan(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    plan_id: String,
) -> Result<String, String> {
    state.inner().start_job(Some(app), plan_id)
}
#[tauri::command]
fn cancel_job(state: tauri::State<'_, AppState>, job_id: String) -> Result<(), String> {
    state.cancel(&job_id)
}
#[tauri::command]
fn job_history(state: tauri::State<'_, AppState>) -> Result<Vec<Job>, String> {
    state.history()
}
#[tauri::command]
fn get_setting(state: tauri::State<'_, AppState>, key: String) -> Result<Option<Value>, String> {
    state.settings(&key)
}
#[tauri::command]
fn set_setting(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    key: String,
    value: Value,
) -> Result<(), String> {
    let old_language = state
        .database
        .get(&key)?
        .and_then(|v| v.get("language").cloned());
    state.database.set(&key, &value)?;
    if key == "preferences" && old_language.as_ref() != value.get("language") {
        menu::install(
            &app,
            value.get("language").and_then(Value::as_str) != Some("en"),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
fn initial_paths(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(std::mem::take(
        &mut *state.os_paths.lock().map_err(|e| e.to_string())?,
    ))
}
#[tauri::command]
async fn engine_status(state: tauri::State<'_, AppState>) -> Result<Value, String> {
    let result = state.engine.execute(&["-ver".into()]).await?;
    Ok(
        json!({"version":result.stdout.trim(),"offline":true,"appVersion":env!("CARGO_PKG_VERSION")}),
    )
}
#[tauri::command]
async fn restore_backup(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    job_id: String,
    file_id: String,
) -> Result<String, String> {
    let _guard = state
        .write_slot
        .acquire()
        .await
        .map_err(|e| e.to_string())?;
    let mut job = state
        .history()?
        .into_iter()
        .find(|j| j.id == job_id)
        .ok_or("Job not found")?;
    let result = job
        .results
        .iter()
        .find(|r| r.file_id == file_id)
        .cloned()
        .ok_or("File result not found")?;
    if result.status == "restored" {
        return Err("This result has already been restored".into());
    }
    let copy = result.clone();
    let message = tokio::task::spawn_blocking(move || jobs::restore(&copy))
        .await
        .map_err(|e| e.to_string())??;
    if let Some(saved) = job.results.iter_mut().find(|r| r.file_id == file_id) {
        saved.status = "restored".into();
        saved.message = message.clone();
    }
    state.update_job(Some(&app), &job)?;
    if let Some(original) = result.original_path {
        if Path::new(&original).exists()
            && state
                .file(&file_id)
                .is_ok_and(|file| file.path == result.path)
        {
            let file = state.relocate(&file_id, Path::new(&original))?;
            app.emit("tagryn:files", vec![file])
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(message)
}
#[tauri::command]
async fn export_metadata(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<String>,
    path: String,
    format: String,
) -> Result<usize, String> {
    if file_ids.is_empty() || file_ids.len() > 10000 {
        return Err("Select between 1 and 10,000 files".into());
    }
    if !["json", "csv"].contains(&format.as_str()) {
        return Err("Unsupported export format".into());
    }
    let destination = PathBuf::from(path);
    if destination.exists() {
        return Err("Choose a new export filename; existing files are never overwritten".into());
    }
    let temporary =
        destination.with_file_name(format!(".tagryn-export-{}.tmp", uuid::Uuid::new_v4()));
    struct ExportGuard(PathBuf);
    impl Drop for ExportGuard {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _guard = ExportGuard(temporary.clone());
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let mut output = std::io::BufWriter::new(file);
    let count = file_ids.len();
    if format == "json" {
        output
            .write_all(b"{\"schema\":\"tagryn.metadata/v1\",\"files\":[")
            .map_err(|e| e.to_string())?;
        for (index, id) in file_ids.iter().enumerate() {
            if index > 0 {
                output.write_all(b",").map_err(|e| e.to_string())?;
            }
            let document = state.document(id, false).await?;
            serde_json::to_writer(&mut output, &document).map_err(|e| e.to_string())?;
        }
        output.write_all(b"]}").map_err(|e| e.to_string())?;
    } else {
        let mut writer = csv::Writer::from_writer(&mut output);
        writer
            .write_record([
                "file",
                "source",
                "group",
                "location",
                "tag",
                "identity",
                "raw",
                "formatted",
            ])
            .map_err(|e| e.to_string())?;
        for id in &file_ids {
            let doc = state.document(id, false).await?;
            for tag in &doc.tags {
                writer
                    .write_record([
                        &csv_safe(&doc.file.path),
                        &csv_safe(&tag.source),
                        &csv_safe(&tag.group),
                        &csv_safe(&tag.location),
                        &csv_safe(&tag.name),
                        &csv_safe(&tag.key),
                        &csv_safe(&value_text(&tag.raw)),
                        &csv_safe(&value_text(&tag.formatted)),
                    ])
                    .map_err(|e| e.to_string())?;
            }
        }
        writer.flush().map_err(|e| e.to_string())?;
    }
    output.flush().map_err(|e| e.to_string())?;
    output.get_ref().sync_all().map_err(|e| e.to_string())?;
    drop(output);
    fs::hard_link(&temporary, &destination).map_err(|e|format!("Export was not installed; choose a new filename on a filesystem supporting hard links: {e}"))?;
    Ok(count)
}
fn csv_safe(s: &str) -> String {
    if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s.into()
    }
}
#[tauri::command]
async fn read_import(path: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&path);
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
            return Err("Import exceeds 16 MiB limit".into());
        }
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        if files::extension(path) == "json" {
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())
        } else if files::extension(path) == "csv" {
            let mut reader = csv::Reader::from_reader(bytes.as_slice());
            let headers = reader.headers().map_err(|e| e.to_string())?.clone();
            let mut rows = Vec::new();
            for row in reader.records() {
                let row = row.map_err(|e| e.to_string())?;
                let mut object = serde_json::Map::new();
                for (key, value) in headers.iter().zip(row.iter()) {
                    object.insert(key.into(), json!(value));
                }
                rows.push(Value::Object(object));
            }
            Ok(json!(rows))
        } else {
            Err("Choose a CSV or JSON file".into())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn run() {
    let _ = STARTED.set(std::time::Instant::now());
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let runtime = if cfg!(debug_assertions) {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/runtime")
            } else {
                app.path().resource_dir()?.join("runtime")
            };
            let data = match std::env::var_os("TAGRYN_PROFILE_DIR") {
                Some(path) => {
                    let path = PathBuf::from(path);
                    if !path.is_absolute() {
                        return Err("TAGRYN_PROFILE_DIR must be absolute".into());
                    }
                    path
                }
                None => app.path().app_data_dir()?,
            };
            let service = Arc::new(Service::new(runtime, data).map_err(std::io::Error::other)?);
            let german = service
                .settings("preferences")
                .ok()
                .flatten()
                .and_then(|v| v.get("language").cloned())
                .as_ref()
                .and_then(Value::as_str)
                != Some("en");
            menu::install(app.handle(), german)?;
            if let Ok(mut paths) = service.os_paths.lock() {
                *paths = std::env::args()
                    .skip(1)
                    .filter(|arg| Path::new(arg).exists())
                    .collect();
            }
            app.manage(service);
            Ok(())
        })
        .on_menu_event(|app, event| {
            let _ = app.emit("tagryn:menu", event.id().as_ref());
        })
        .invoke_handler(tauri::generate_handler![
            search_metadata,
            frontend_diagnostic,
            frontend_ready,
            open_paths,
            read_metadata,
            read_preview,
            plan_changes,
            execute_plan,
            cancel_job,
            job_history,
            get_setting,
            set_setting,
            initial_paths,
            engine_status,
            restore_backup,
            export_metadata,
            read_import
        ])
        .build(tauri::generate_context!())
        .expect("Tagryn could not start");
    app.run(|handle, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Opened { urls } = &event {
            let paths: Vec<String> = urls
                .iter()
                .filter_map(|u| u.to_file_path().ok())
                .filter_map(|p| p.to_str().map(str::to_owned))
                .collect();
            if let Some(state) = handle.try_state::<AppState>() {
                if let Ok(mut pending) = state.os_paths.lock() {
                    pending.extend(paths.clone());
                }
            }
            let _ = handle.emit("tagryn:open", paths);
        }
        if let tauri::RunEvent::Exit = event {
            if let Some(state) = handle.try_state::<AppState>() {
                tauri::async_runtime::block_on(state.engine.shutdown());
            }
        }
    });
}
