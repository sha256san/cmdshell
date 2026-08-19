pub mod windows;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_available: bool,
}

pub struct ShellResolver;

impl ShellResolver {
    pub fn resolve_shell(configured_shell: Option<&str>) -> Vec<ShellInfo> {
        let mut candidates = Vec::new();

        // 1. Configured shell if provided
        if let Some(cfg) = configured_shell {
            let path = PathBuf::from(cfg);
            let is_available = path.exists();
            candidates.push(ShellInfo {
                name: "Configured Shell".to_string(),
                path,
                is_available,
            });
        }

        // 2. Platform-specific shells
        #[cfg(windows)]
        {
            candidates.extend(windows::discover_windows_shells());
        }

        #[cfg(not(windows))]
        {
            if let Ok(shell_env) = std::env::var("SHELL") {
                let path = PathBuf::from(&shell_env);
                let is_available = path.exists();
                candidates.push(ShellInfo {
                    name: format!("Environment $SHELL ({})", shell_env),
                    path,
                    is_available,
                });
            }

            for fallback in &["/bin/bash", "/bin/zsh", "/bin/sh", "/usr/bin/bash", "/usr/bin/zsh"] {
                let path = PathBuf::from(fallback);
                if path.exists() {
                    candidates.push(ShellInfo {
                        name: fallback.to_string(),
                        path,
                        is_available: true,
                    });
                }
            }
        }

        candidates
    }

    pub fn get_default_shell(configured_shell: Option<&str>) -> (String, PathBuf) {
        let shells = Self::resolve_shell(configured_shell);
        for shell in shells {
            if shell.is_available {
                return (shell.name, shell.path);
            }
        }

        #[cfg(windows)]
        {
            (
                "Command Prompt (Fallback)".to_string(),
                PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            )
        }

        #[cfg(not(windows))]
        {
            ("Fallback sh".to_string(), PathBuf::from("/bin/sh"))
        }
    }
}
