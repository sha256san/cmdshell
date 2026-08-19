use std::path::PathBuf;
use crate::shell::{ShellInfo, ShellKind};

pub fn discover_macos_shells() -> Vec<ShellInfo> {
    let mut candidates = Vec::new();

    // 1. Environment $SHELL
    if let Ok(shell_env) = std::env::var("SHELL") {
        let path = PathBuf::from(&shell_env);
        if path.exists() {
            let kind = if shell_env.ends_with("zsh") {
                ShellKind::UnixZsh
            } else if shell_env.ends_with("bash") {
                ShellKind::UnixBash
            } else {
                ShellKind::UnixSh
            };

            candidates.push(ShellInfo {
                name: format!("Default $SHELL ({})", shell_env),
                path,
                kind,
                is_available: true,
                default_args: vec!["-i".to_string()],
            });
        }
    }

    // 2. macOS standard shells (zsh is default on macOS 10.15+)
    let standard = [
        ("/bin/zsh", "Zsh (/bin/zsh)", ShellKind::UnixZsh),
        ("/bin/bash", "Bash (/bin/bash)", ShellKind::UnixBash),
        ("/opt/homebrew/bin/zsh", "Homebrew Zsh (/opt/homebrew/bin/zsh)", ShellKind::UnixZsh),
        ("/opt/homebrew/bin/bash", "Homebrew Bash (/opt/homebrew/bin/bash)", ShellKind::UnixBash),
        ("/usr/local/bin/zsh", "Intel Homebrew Zsh (/usr/local/bin/zsh)", ShellKind::UnixZsh),
        ("/bin/sh", "POSIX Shell (/bin/sh)", ShellKind::UnixSh),
    ];

    for (p, name, kind) in standard {
        let path = PathBuf::from(p);
        if path.exists() && !candidates.iter().any(|c| c.path == path) {
            candidates.push(ShellInfo {
                name: name.to_string(),
                path,
                kind,
                is_available: true,
                default_args: vec!["-i".to_string()],
            });
        }
    }

    candidates
}
