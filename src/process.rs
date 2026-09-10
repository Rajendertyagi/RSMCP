use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::Mutex;

const MAX_BUFFER_CHARS: usize = 10 * 1024 * 1024; // 10MB buffer per process

pub struct ProcessSession {
    pub pid: u32,
    pub stdout_buffer: Vec<String>,
    pub child: Mutex<tokio::process::Child>,
    pub is_complete: bool,
}

#[derive(Clone)]
pub struct ProcessManager {
    sessions: Arc<Mutex<HashMap<u32, ProcessSession>>>,
    next_pid: Arc<AtomicU32>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_pid: Arc::new(AtomicU32::new(1000)),
        }
    }

    pub async fn start(
        &self,
        command: &str,
        cwd: Option<&Path>,
        timeout_ms: Option<u64>,
    ) -> Result<StartResult, String> {
        let pid = self.next_pid.fetch_add(1, Ordering::Relaxed);

        let stdout_buffer: Vec<String> = Vec::new();

        let mut cmd = Command::new("cmd.exe");
        cmd.arg("/c").arg(command);

        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.stdin(std::process::Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return Err(format!("Failed to spawn process: {}", e)),
        };

        let session = ProcessSession {
            pid,
            stdout_buffer,
            child: Mutex::new(child),
            is_complete: false,
        };

        let sessions = self.sessions.clone();
        let pid_clone = pid;
        let sessions_for_stdout = sessions.clone();

        // Spawn a task to read stdout/stderr and buffer it
        tokio::spawn(async move {
            let (mut reader, mut writer) = tokio::io::duplex(8192);

            // Read stdout
            let stdout_task = tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                let mut total_chars = 0;
                while total_chars < MAX_BUFFER_CHARS {
                    match reader.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Ok(s) = String::from_utf8(buf[..n].to_vec()) {
                                let lines: Vec<&str> = s.lines().collect();
                                for line in lines {
                                    if total_chars >= MAX_BUFFER_CHARS {
                                        break;
                                    }
                                    if let Some(q) = sessions_for_stdout.lock().await.get_mut(&pid_clone) {
                                        if q.stdout_buffer.len() >= 100_000 {
                                            q.stdout_buffer.remove(0);
                                        }
                                        q.stdout_buffer.push(line.to_string());
                                        total_chars += line.len() + 1;
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            // Read stderr to a string
            let stderr_task = tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                let mut stderr_output = String::new();
                while stderr_output.len() < 100_000 {
                    match writer.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            match String::from_utf8(buf[..n].to_vec()) {
                                Ok(s) => stderr_output.push_str(&s),
                                Err(_) => break,
                            }
                        }
                        Err(_) => break,
                    }
                }
                stderr_output
            });

            let _ = stdout_task.await;
            let stderr_out = stderr_task.await.unwrap_or_default();

            // Mark as complete
            if let Some(session) = sessions.lock().await.get_mut(&pid_clone) {
                session.is_complete = true;
                if !stderr_out.is_empty() {
                    if let Some(q) = sessions.lock().await.get_mut(&pid_clone) {
                        if q.stdout_buffer.len() >= 100_000 {
                            q.stdout_buffer.remove(0);
                        }
                        q.stdout_buffer.push(format!("[stderr] {}", stderr_out.trim()));
                    }
                }
            }
        });

        self.sessions.lock().await.insert(pid, session);

        Ok(StartResult {
            pid,
            message: format!("Process started with PID {}", pid),
        })
    }

    pub async fn read_output(
        &self,
        pid: u32,
        offset: i64,
        length: i64,
    ) -> Result<ReadResult, String> {
        let buffer;
        let is_complete;
        {
            let sessions = self.sessions.lock().await;
            let session = match sessions.get(&pid) {
                Some(s) => s,
                None => return Err(format!("No session found for PID {}", pid)),
            };
            is_complete = session.is_complete;
            buffer = session.stdout_buffer.clone();
        }

        if !is_complete && offset == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let sessions = self.sessions.lock().await;
            match sessions.get(&pid) {
                Some(_) => (),
                None => return Err(format!("No session found for PID {}", pid)),
            }
        }
        let total_lines = buffer.len();

        let start_idx = if offset < 0 {
            std::cmp::max(0, total_lines as i64 + offset) as usize
        } else {
            offset as usize
        };

        let end_idx = std::cmp::min(start_idx + length as usize, total_lines);
        let lines: Vec<String> = buffer[start_idx..end_idx].to_vec();

        Ok(ReadResult {
            output: lines.join("\n"),
            read_count: end_idx - start_idx,
            total_lines,
            is_complete,
            remaining: total_lines - end_idx,
        })
    }

    pub async fn write_input(&self, pid: u32, data: &str) -> Result<(), String> {
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&pid).ok_or_else(|| format!("No session found for PID {}", pid))?;

        let mut child = session.child.lock().await;
        let stdin = child.stdin.as_mut().ok_or("Process has no stdin")?;

        use tokio::io::AsyncWriteExt;
        stdin.write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn kill(&self, pid: u32) -> Result<KillResult, String> {
        let sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(&pid) {
            let mut child = session.child.lock().await;
            let _ = child.kill().await;
            Ok(KillResult { killed: true })
        } else {
            Ok(KillResult { killed: false })
        }
    }

    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().await;
        sessions
            .values()
            .map(|s| SessionInfo {
                pid: s.pid,
                is_complete: s.is_complete,
                buffer_size: s.stdout_buffer.len(),
            })
            .collect()
    }

    pub async fn list_processes(&self) -> Result<Vec<ProcessEntry>, String> {
        #[cfg(windows)]
        {
            let output = Command::new("cmd.exe")
                .arg("/c")
                .arg("tasklist /fo csv /nh")
                .output()
                .await
                .map_err(|e| e.to_string())?;

            let content = String::from_utf8_lossy(&output.stdout);
            let mut processes = Vec::new();

            for line in content.lines() {
                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() >= 2 {
                    let pid_str = fields[1].trim_matches('"');
                    let name = fields[0].trim_matches('"');
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        processes.push(ProcessEntry {
                            pid,
                            name: name.to_string(),
                        });
                    }
                }
            }

            Ok(processes)
        }
        #[cfg(unix)]
        {
            let output = Command::new("ps")
                .arg("-ef")
                .output()
                .await
                .map_err(|e| e.to_string())?;

            let content = String::from_utf8_lossy(&output.stdout);
            let mut processes = Vec::new();

            for line in content.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 2 {
                    if let Ok(pid) = fields[1].parse::<u32>() {
                        let name = if fields.len() > 10 {
                            fields[10].to_string()
                        } else {
                            "unknown".to_string()
                        };
                        processes.push(ProcessEntry { pid, name });
                    }
                }
            }

            Ok(processes)
        }
    }

    pub async fn cleanup_completed(&self) {
        let mut sessions = self.sessions.lock().await;
        sessions.retain(|_, s| !s.is_complete);
    }
}

#[derive(Debug)]
pub struct StartResult {
    pub pid: u32,
    pub message: String,
}

#[derive(Debug)]
pub struct ReadResult {
    pub output: String,
    pub read_count: usize,
    pub total_lines: usize,
    pub is_complete: bool,
    pub remaining: usize,
}

#[derive(Debug)]
pub struct KillResult {
    pub killed: bool,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub pid: u32,
    pub is_complete: bool,
    pub buffer_size: usize,
}

#[derive(Debug, Clone)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
}
