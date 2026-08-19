use std::path::PathBuf;
use crate::shell::{ShellInfo, ShellKind};

pub fn get_system_root() -> PathBuf {
    std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Windows"))
}

pub fn get_program_files() -> PathBuf {
    std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Program Files"))
}

pub fn discover_windows_shells() -> Vec<ShellInfo> {
    let system_root = get_system_root();
    let program_files = get_program_files();

    let candidates = vec![
        // 1. PowerShell 7+ (pwsh.exe) - Modern cross-platform .NET runtime, preferred on Windows
        (
            "PowerShell 7 (pwsh.exe)",
            program_files.join("PowerShell").join("7").join("pwsh.exe"),
            ShellKind::PowerShell,
            vec!["-NoLogo".to_string(), "-NoProfile".to_string()],
        ),
        // 2. Windows PowerShell 5.1 (powershell.exe) - Standard built-in Windows PowerShell with -NoProfile
        (
            "Windows PowerShell (powershell.exe)",
            system_root
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe"),
            ShellKind::WindowsPowerShell,
            vec!["-NoLogo".to_string(), "-NoProfile".to_string()],
        ),
        // 3. Command Prompt (cmd.exe) - Native Win32 C binary, zero .NET CLR dependency, immune to 0xc0000142
        (
            "Command Prompt (cmd.exe)",
            system_root.join("System32").join("cmd.exe"),
            ShellKind::Cmd,
            vec!["/Q".to_string()],
        ),
        // 4. Git Bash
        (
            "Git Bash (bash.exe)",
            program_files.join("Git").join("bin").join("bash.exe"),
            ShellKind::GitBash,
            vec!["--login".to_string(), "-i".to_string()],
        ),
        // 5. WSL
        (
            "WSL (wsl.exe)",
            system_root.join("System32").join("wsl.exe"),
            ShellKind::Wsl,
            vec![],
        ),
    ];

    candidates
        .into_iter()
        .map(|(name, path, kind, default_args)| {
            let is_available = path.exists();
            ShellInfo {
                name: name.to_string(),
                path,
                kind,
                is_available,
                default_args,
            }
        })
        .collect()
}
