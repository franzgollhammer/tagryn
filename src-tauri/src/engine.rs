use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

const LIMIT: usize = 32 * 1024 * 1024;
pub struct Engine {
    runtime: PathBuf,
    workers: [Mutex<Option<Worker>>; 2],
    next: AtomicUsize,
    sequence: AtomicU64,
}
struct Worker {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    errors: BufReader<ChildStderr>,
}
pub struct Output {
    pub stdout: String,
    pub warnings: String,
}

impl Engine {
    pub fn new(runtime: PathBuf) -> Self {
        Self {
            runtime,
            workers: [Mutex::new(None), Mutex::new(None)],
            next: AtomicUsize::new(0),
            sequence: AtomicU64::new(1),
        }
    }
    pub fn command(&self) -> Command {
        runtime_command(&self.runtime)
    }
    async fn spawn(&self) -> Result<Worker, String> {
        let mut cmd = self.command();
        let mut child = cmd
            .args(["-config", "", "-stay_open", "True", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Bundled ExifTool could not start: {e}. Run npm run runtime."))?;
        Ok(Worker {
            input: child.stdin.take().ok_or("ExifTool stdin unavailable")?,
            output: BufReader::new(child.stdout.take().ok_or("ExifTool stdout unavailable")?),
            errors: BufReader::new(child.stderr.take().ok_or("ExifTool stderr unavailable")?),
            child,
        })
    }
    pub async fn execute(&self, args: &[String]) -> Result<Output, String> {
        if args.iter().any(|a| a.contains('\0')) {
            return Err("NUL is not a valid process argument".into());
        }
        // The stay-open protocol is line based. Multiline values and filenames use an argv-only isolated request.
        if args
            .iter()
            .any(|a| a.contains(['\n', '\r']) || a.starts_with('#'))
        {
            let out = self.one_shot(args).await?;
            return Ok(Output {
                stdout: String::from_utf8_lossy(&out.0).into(),
                warnings: out.1,
            });
        }
        let index = self.next.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        let mut slot = self.workers[index].lock().await;
        if slot.is_none() {
            *slot = Some(self.spawn().await?);
        }
        let worker = slot.as_mut().ok_or("Engine unavailable")?;
        let id = self.sequence.fetch_add(1, Ordering::Relaxed);
        let marker = format!("TAGRYN-{}-", uuid::Uuid::new_v4());
        let mut request = args.join("\n");
        request.push_str(&format!("\n-echo4\n{marker}${{status}}\n-execute{id}\n"));
        let outcome = tokio::time::timeout(Duration::from_secs(45), async {
            worker
                .input
                .write_all(request.as_bytes())
                .await
                .map_err(|e| e.to_string())?;
            worker.input.flush().await.map_err(|e| e.to_string())?;
            let ready = format!("{{ready{id}}}");
            let (stdout, stderr) = futures_util::future::try_join(
                read_until(&mut worker.output, &ready, false),
                read_until(&mut worker.errors, &marker, true),
            )
            .await?;
            let (warnings, status) = stderr
                .rsplit_once(&marker)
                .ok_or("ExifTool status missing")?;
            if status.trim() != "0" {
                return Err(format!(
                    "ExifTool failed ({}): {}",
                    status.trim(),
                    warnings.trim()
                ));
            }
            Ok(Output {
                stdout,
                warnings: warnings.trim().to_owned(),
            })
        })
        .await;
        match outcome {
            Ok(Ok(output)) => Ok(output),
            other => {
                let _ = worker.child.kill().await;
                *slot = None;
                match other {
                    Ok(Err(e)) => Err(e),
                    _ => Err(
                        "ExifTool timed out. Worker stopped; next request starts a new worker."
                            .into(),
                    ),
                }
            }
        }
    }
    pub async fn one_shot(&self, args: &[String]) -> Result<(Vec<u8>, String), String> {
        let mut cmd = self.command();
        if args.iter().any(|arg| arg.contains('\0')) {
            return Err("NUL is not a valid process argument".into());
        }
        let mut child = cmd
            .args(["-config", ""])
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| e.to_string())?;
        let stdout = child.stdout.take().ok_or("ExifTool stdout unavailable")?;
        let stderr = child.stderr.take().ok_or("ExifTool stderr unavailable")?;
        let outcome = tokio::time::timeout(Duration::from_secs(45), async {
            let (output, errors) =
                futures_util::future::try_join(read_bounded(stdout), read_bounded(stderr)).await?;
            let status = child.wait().await.map_err(|e| e.to_string())?;
            Ok::<_, String>((output, errors, status))
        })
        .await;
        let (output, errors, status) = match outcome {
            Ok(Ok(result)) => result,
            failure => {
                let _ = child.kill().await;
                return Err(match failure {
                    Ok(Err(error)) => error,
                    _ => "ExifTool request timed out".into(),
                });
            }
        };
        let errors = String::from_utf8_lossy(&errors).trim().to_owned();
        if !status.success() {
            return Err(format!("ExifTool: {errors}"));
        }
        Ok((output, errors))
    }
    pub async fn shutdown(&self) {
        for slot in &self.workers {
            if let Some(mut worker) = slot.lock().await.take() {
                let _ = worker.input.write_all(b"-stay_open\nFalse\n").await;
                if tokio::time::timeout(Duration::from_secs(2), worker.child.wait())
                    .await
                    .is_err()
                {
                    let _ = worker.child.kill().await;
                }
            }
        }
    }
}

pub fn runtime_command(runtime: &Path) -> Command {
    #[cfg(windows)]
    let mut command = Command::new(runtime.join("exiftool/bin/exiftool.exe"));
    #[cfg(not(windows))]
    let mut command = {
        let mut c = Command::new(runtime.join("perl/bin/perl"));
        c.arg(runtime.join("exiftool/bin/exiftool"));
        c
    };
    for name in [
        "PERL5OPT",
        "PERL5LIB",
        "PERLLIB",
        "PERL_UNICODE",
        "PERLIO",
        "EXIFTOOL_HOME",
    ] {
        command.env_remove(name);
    }
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    command
}

async fn read_until<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    marker: &str,
    keep_marker: bool,
) -> Result<String, String> {
    let mut data = Vec::new();
    loop {
        let mut line = Vec::new();
        loop {
            let available = reader.fill_buf().await.map_err(|e| e.to_string())?;
            if available.is_empty() {
                return Err("ExifTool exited before completing request".into());
            }
            let end = available.iter().position(|byte| *byte == b'\n');
            let count = end.map_or(available.len(), |index| index + 1);
            if data.len() + line.len() + count > LIMIT {
                return Err("Metadata exceeds 32 MiB output limit".into());
            }
            line.extend_from_slice(&available[..count]);
            reader.consume(count);
            if end.is_some() {
                break;
            }
        }
        let text = String::from_utf8_lossy(&line);
        if (!keep_marker && text.trim_end() == marker) || (keep_marker && text.starts_with(marker))
        {
            if keep_marker {
                data.extend_from_slice(&line);
            }
            return String::from_utf8(data).map_err(|e| e.to_string());
        }
        data.extend_from_slice(&line);
    }
}

async fn read_bounded<R: tokio::io::AsyncRead + Unpin>(mut reader: R) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if count == 0 {
            return Ok(data);
        }
        if data.len() + count > LIMIT {
            return Err("ExifTool output exceeds 32 MiB limit".into());
        }
        data.extend_from_slice(&buffer[..count]);
    }
}
