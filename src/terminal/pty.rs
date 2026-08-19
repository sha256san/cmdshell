use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use crate::shell::{EnvironmentBuilder, ShellResolver};

pub struct PtyBackend {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    is_alive: Arc<AtomicBool>,
}

impl PtyBackend {
    pub fn spawn(
        shell: Option<&str>,
        cwd: Option<PathBuf>,
        cols: u16,
        rows: u16,
        output_tx: crossbeam_channel::Sender<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // 1. Resolve candidate shells with active health check
        let candidate_results = ShellResolver::resolve_with_health_check(shell);
        let normalized_env = EnvironmentBuilder::build_shell_environment(None);

        let mut last_error: Option<String> = None;
        let mut spawned = false;

        // Try launching verified healthy shells in priority order
        for (candidate, health) in &candidate_results {
            if !health.is_healthy() {
                continue;
            }

            let mut cmd = CommandBuilder::new(&candidate.path);

            // Apply shell-specific arguments (-NoLogo, -NoProfile, etc.)
            for arg in &candidate.default_args {
                cmd.arg(arg);
            }

            if let Some(ref dir) = cwd {
                cmd.cwd(dir);
            }

            // Apply normalized environment
            for (k, v) in &normalized_env {
                cmd.env(k, v);
            }

            match pair.slave.spawn_command(cmd) {
                Ok(_) => {
                    spawned = true;
                    break;
                }
                Err(e) => {
                    last_error = Some(format!("{}: {}", candidate.name, e));
                }
            }
        }

        // 2. If no healthy shell spawned, try best available shell
        if !spawned {
            let fallback_shell = ShellResolver::get_best_shell(shell);
            let mut cmd = CommandBuilder::new(&fallback_shell.path);

            for arg in &fallback_shell.default_args {
                cmd.arg(arg);
            }

            if let Some(ref dir) = cwd {
                cmd.cwd(dir);
            }

            for (k, v) in &normalized_env {
                cmd.env(k, v);
            }

            if let Err(e) = pair.slave.spawn_command(cmd) {
                let err_msg = format!(
                    "Failed to spawn terminal shell (fallback {}): {}. Previous errors: {:?}",
                    fallback_shell.name, e, last_error
                );
                return Err(err_msg.into());
            }
        }

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let is_alive = Arc::new(AtomicBool::new(true));
        let is_alive_clone = Arc::clone(&is_alive);

        // Spawn background reader thread
        thread::Builder::new()
            .name("pty-reader".to_string())
            .spawn(move || {
                let mut buffer = [0u8; 4096];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => {
                            is_alive_clone.store(false, Ordering::SeqCst);
                            break;
                        }
                        Ok(n) => {
                            if output_tx.send(buffer[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(_) => {
                            is_alive_clone.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                }
            })?;

        Ok(Self {
            master: Arc::new(Mutex::new(pair.master)),
            writer: Arc::new(Mutex::new(writer)),
            is_alive,
        })
    }

    pub fn write_bytes(&self, data: &[u8]) -> std::io::Result<()> {
        let mut writer = self.writer.lock();
        writer.write_all(data)?;
        writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let master = self.master.lock();
        master.resize(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst)
    }
}
