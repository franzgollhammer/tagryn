use crate::model::{Document, Job};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::{path::Path, sync::Mutex};

pub struct Database(Mutex<Connection>);
impl Database {
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL); CREATE TABLE IF NOT EXISTS cache (path TEXT PRIMARY KEY, fingerprint TEXT NOT NULL, document TEXT NOT NULL, accessed INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS jobs (id TEXT PRIMARY KEY, data TEXT NOT NULL, updated TEXT NOT NULL);").map_err(|e| e.to_string())?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS search_index (path TEXT PRIMARY KEY, content TEXT NOT NULL, accessed INTEGER NOT NULL);").map_err(|e| e.to_string())?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS search_budget (id INTEGER PRIMARY KEY CHECK(id=1), bytes INTEGER NOT NULL); INSERT OR IGNORE INTO search_budget VALUES(1,0); UPDATE search_budget SET bytes=(SELECT COALESCE(SUM(length(CAST(content AS BLOB))),0) FROM search_index) WHERE id=1; CREATE TRIGGER IF NOT EXISTS search_insert AFTER INSERT ON search_index BEGIN UPDATE search_budget SET bytes=bytes+length(CAST(NEW.content AS BLOB)) WHERE id=1; END; CREATE TRIGGER IF NOT EXISTS search_delete AFTER DELETE ON search_index BEGIN UPDATE search_budget SET bytes=bytes-length(CAST(OLD.content AS BLOB)) WHERE id=1; END; CREATE TRIGGER IF NOT EXISTS search_update AFTER UPDATE OF content ON search_index BEGIN UPDATE search_budget SET bytes=bytes-length(CAST(OLD.content AS BLOB))+length(CAST(NEW.content AS BLOB)) WHERE id=1; END; CREATE INDEX IF NOT EXISTS search_accessed ON search_index(accessed);").map_err(|e| e.to_string())?;
        // An interrupted write must never appear as successful after restart.
        connection.execute("UPDATE jobs SET data=json_set(data,'$.status','interrupted') WHERE json_extract(data,'$.status') IN ('running','cancelling','queued')", []).map_err(|e| e.to_string())?;
        Ok(Self(Mutex::new(connection)))
    }
    pub fn get(&self, key: &str) -> Result<Option<Value>, String> {
        let db = self.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT value FROM settings WHERE key=?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([key]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => {
                let s: String = row.get(0).map_err(|e| e.to_string())?;
                Ok(Some(serde_json::from_str(&s).map_err(|e| e.to_string())?))
            }
            None => Ok(None),
        }
    }
    pub fn set(&self, key: &str, value: &Value) -> Result<(), String> {
        if key.len() > 128 || value.to_string().len() > 4 * 1024 * 1024 {
            return Err("Preference exceeds storage limit".into());
        }
        self.0.lock().map_err(|e| e.to_string())?.execute("INSERT INTO settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, value.to_string()]).map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn cache_get(&self, path: &str, fingerprint: &str) -> Option<Document> {
        let db = self.0.lock().ok()?;
        let data: String = db
            .query_row(
                "SELECT document FROM cache WHERE path=?1 AND fingerprint=?2",
                params![path, fingerprint],
                |row| row.get(0),
            )
            .ok()?;
        serde_json::from_str(&data).ok()
    }
    pub fn cache_put(&self, document: &Document, key: &str) -> Result<(), String> {
        let db = self.0.lock().map_err(|e| e.to_string())?;
        let value = serde_json::to_string(document).map_err(|e| e.to_string())?;
        if value.len() > 2 * 1024 * 1024 {
            return Ok(());
        }
        db.execute("INSERT INTO cache VALUES(?1,?2,?3,unixepoch()) ON CONFLICT(path) DO UPDATE SET fingerprint=excluded.fingerprint,document=excluded.document,accessed=excluded.accessed", params![document.file.path,key,value]).map_err(|e| e.to_string())?;
        db.execute("DELETE FROM cache WHERE path NOT IN (SELECT path FROM cache ORDER BY accessed DESC LIMIT 128)", []).map_err(|e| e.to_string())?;
        let content = document
            .tags
            .iter()
            .map(|tag| format!("{} {} {} {}", tag.key, tag.label, tag.raw, tag.formatted))
            .collect::<Vec<_>>()
            .join("\n")
            .to_lowercase();
        db.execute("INSERT INTO search_index VALUES(?1,?2,unixepoch()) ON CONFLICT(path) DO UPDATE SET content=excluded.content,accessed=excluded.accessed", params![document.file.path,content]).map_err(|e| e.to_string())?;
        loop {
            let bytes: i64 = db
                .query_row("SELECT bytes FROM search_budget WHERE id=1", [], |row| {
                    row.get(0)
                })
                .map_err(|e| e.to_string())?;
            if bytes <= 64 * 1024 * 1024 {
                break;
            }
            db.execute("DELETE FROM search_index WHERE path IN (SELECT path FROM search_index ORDER BY accessed LIMIT 128)", []).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn search(&self, query: &str) -> Result<Vec<String>, String> {
        let db = self.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT path FROM search_index WHERE instr(content,?1)>0")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([query.to_lowercase()], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
    pub fn save_job(&self, job: &Job) -> Result<(), String> {
        self.0.lock().map_err(|e| e.to_string())?.execute("INSERT INTO jobs VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET data=excluded.data,updated=excluded.updated", params![job.id, serde_json::to_string(job).map_err(|e| e.to_string())?, crate::model::now()]).map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn jobs(&self) -> Result<Vec<Job>, String> {
        let db = self.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT data FROM jobs ORDER BY updated DESC LIMIT 100")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.map(|r| {
            serde_json::from_str(&r.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
        })
        .collect()
    }
}
