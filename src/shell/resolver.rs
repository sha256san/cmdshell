use std::path::PathBuf;
use crate::shell::health::{HealthStatus, ShellHealthChecker};
use crate::shell::{ShellInfo, ShellKind};

pub struct ShellResolver;

impl ShellResolver {
    /// Discovers all available shells on the system in priority order.
    pub fn resolve_shells(configured_shell: Option<&str>) -> Vec<ShellInfo> {
        let mut candidates = Vec::new();

        // 1. User-configured shell
        if let Some(cfg) = configured_shell {
            let path = PathBuf::from(cfg);
            let is_available = path.exists();
            let kind = Self::detect_kind_from_path(&path);
            let default_args = match kind {
                ShellKind::PowerShell | ShellKind::WindowsPowerShell => {
                    vec!["-NoLogo".to_string(), "-NoProfile".to_string()]
                }
                ShellKind::Cmd => vec!["/Q".to_string()],
                ShellKind::GitBash | ShellKind::UnixBash | ShellKind::UnixZsh => vec!["-i".to_string()],
                _ => vec![],
            };

            candidates.push(ShellInfo {
                name: format!("Configured Shell ({})", cfg),
                path,
                kind,
                is_available,
                default_args,
            });
        }

        // 2. OS-specific discovery
        #[cfg(target_os = "windows")]
        {
            candidates.extend(crate::shell::windows::discover_windows_shells());
        }

        #[cfg(target_os = "macos")]
        {
            candidates.extend(crate::shell::macos::discover_macos_shells());
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            candidates.extend(crate::shell::linux::discover_linux_shells());
        }

        candidates
    }

    /// Performs health check on all candidate shells and returns them along with their health status.
    pub fn resolve_with_health_check(configured_shell: Option<&str>) -> Vec<(ShellInfo, HealthStatus)> {
        let shells = Self::resolve_shells(configured_shell);
        shells
            .into_iter()
            .map(|sh| {
                let status = if sh.is_available {
                    ShellHealthChecker::check(&sh)
                } else {
                    HealthStatus::Failed {
                        exit_code: None,
                        error_message: "Executable not found".to_string(),
                        is_0xc0000142: false,
                    }
                };
                (sh, status)
            })
            .collect()
    }

    /// Selects the best verified healthy shell for launching PTY sessions.
    pub fn get_best_shell(configured_shell: Option<&str>) -> ShellInfo {
        let results = Self::resolve_with_health_check(configured_shell);

        // First attempt: Find the highest-priority verified healthy shell
        for (shell, status) in &results {
            if status.is_healthy() {
                return shell.clone();
            }
        }

        // Second attempt: If health probe failed on all (e.g. sandbox/container), find first available shell on disk
        for (shell, _) in &results {
            if shell.is_available {
                return shell.clone();
            }
        }

        // Ultimate fallback
        #[cfg(target_os = "windows")]
        {
            let system_root = crate::shell::windows::get_system_root();
            ShellInfo {
                name: "Command Prompt (Emergency Fallback)".to_string(),
                path: system_root.join("System32").join("cmd.exe"),
                kind: ShellKind::Cmd,
                is_available: true,
                default_args: vec!["/Q".to_string()],
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            ShellInfo {
                name: "POSIX Shell (Emergency Fallback)".to_string(),
                path: PathBuf::from("/bin/sh"),
                kind: ShellKind::UnixSh,
                is_available: true,
                default_args: vec!["-i".to_string()],
            }
        }
    }

    fn detect_kind_from_path(path: &std::path::Path) -> ShellKind {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.starts_with("pwsh") {
            ShellKind::PowerShell
        } else if file_name.starts_with("powershell") {
            ShellKind::WindowsPowerShell
        } else if file_name.starts_with("cmd") {
            ShellKind::Cmd
        } else if file_name.starts_with("zsh") {
            ShellKind::UnixZsh
        } else if file_name.starts_with("bash") {
            if cfg!(windows) {
                ShellKind::GitBash
            } else {
                ShellKind::UnixBash
            }
        } else if file_name.starts_with("wsl") {
            ShellKind::Wsl
        } else if file_name.starts_with("sh") {
            ShellKind::UnixSh
        } else {
            ShellKind::Custom
        }
    }
}
