use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use crate::shell::ShellResolver;

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

        let candidate_shells = ShellResolver::resolve_shell(shell);
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        let mut spawned = false;

        // Try spawning candidate shells in priority order
        for candidate in &candidate_shells {
            let mut cmd = CommandBuilder::new(&candidate.path);

            if let Some(ref dir) = cwd {
                cmd.cwd(dir);
            }

            // Inherit full environment variables
            for (k, v) in std::env::vars() {
                cmd.env(k, v);
            }

            // Injects essential Windows environment variables to prevent 0xc0000142 (STATUS_DLL_INIT_FAILED)
            #[cfg(windows)]
            {
                crate::shell::windows::ensure_essential_windows_env(&mut |k, v| {
                    cmd.env(k, v);
                });
            }

            match pair.slave.spawn_command(cmd) {
                Ok(_) => {
                    spawned = true;
                    break;
                }
                Err(e) => {
                    last_error = Some(e.to_string().into());
                }
            }
        }

        if !spawned {
            // Ultimate fallback
            let (fallback_name, fallback_path) = ShellResolver::get_default_shell(shell);
            let mut cmd = CommandBuilder::new(&fallback_path);
            if let Some(ref dir) = cwd {
                cmd.cwd(dir);
            }

            for (k, v) in std::env::vars() {
                cmd.env(k, v);
            }

            #[cfg(windows)]
            {
                crate::shell::windows::ensure_essential_windows_env(&mut |k, v| {
                    cmd.env(k, v);
                });
            }

            if let Err(e) = pair.slave.spawn_command(cmd) {
                let err_msg = format!(
                    "Failed to spawn any terminal shell (including {}): {}",
                    fallback_name,
                    last_error.map(|e| e.to_string()).unwrap_or_else(|| e.to_string())
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
