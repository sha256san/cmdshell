use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

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
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell_cmd = shell
            .map(|s| s.to_string())
            .or_else(|| std::env::var("SHELL").ok())
            .unwrap_or_else(|| {
                if cfg!(windows) {
                    "powershell.exe".to_string()
                } else {
                    "/bin/bash".to_string()
                }
            });

        let mut cmd = CommandBuilder::new(&shell_cmd);
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        let _child = pair.slave.spawn_command(cmd)?;

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
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst)
    }
}
