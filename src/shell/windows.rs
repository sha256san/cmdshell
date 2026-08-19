use std::path::PathBuf;
use crate::shell::ShellInfo;

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
        // 1. Command Prompt (cmd.exe) - Native Win32 C binary, zero .NET CLR dependency, immune to 0xc0000142
        (
            "Command Prompt (cmd.exe)",
            system_root.join("System32").join("cmd.exe"),
        ),
        // 2. PowerShell 7+ (pwsh.exe) - Modern cross-platform .NET runtime
        (
            "PowerShell 7 (pwsh.exe)",
            program_files.join("PowerShell").join("7").join("pwsh.exe"),
        ),
        // 3. Windows PowerShell 5.1 (powershell.exe)
        (
            "Windows PowerShell (powershell.exe)",
            system_root
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe"),
        ),
        // 4. Git Bash
        (
            "Git Bash (bash.exe)",
            program_files.join("Git").join("bin").join("bash.exe"),
        ),
        // 5. WSL
        (
            "WSL (wsl.exe)",
            system_root.join("System32").join("wsl.exe"),
        ),
    ];

    candidates
        .into_iter()
        .map(|(name, path)| {
            let is_available = path.exists();
            ShellInfo {
                name: name.to_string(),
                path,
                is_available,
            }
        })
        .collect()
}

/// Injects critical environment variables required by Windows DLLs to avoid 0xc0000142.
pub fn ensure_essential_windows_env(env_setter: &mut dyn FnMut(&str, &str)) {
    // 1. Inherit all current process environment variables
    for (k, v) in std::env::vars() {
        env_setter(&k, &v);
    }

    let system_root = get_system_root();
    let sys_root_str = system_root.to_string_lossy();

    // 2. Guarantee critical Windows subsystem paths
    env_setter("SystemRoot", &sys_root_str);
    env_setter("WINDIR", &sys_root_str);

    let drive = std::env::var("SystemDrive").unwrap_or_else(|_| {
        system_root
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("C:")
            .to_string()
    });
    env_setter("SystemDrive", &drive);

    let comspec = std::env::var("ComSpec").unwrap_or_else(|_| {
        system_root
            .join("System32")
            .join("cmd.exe")
            .to_string_lossy()
            .to_string()
    });
    env_setter("ComSpec", &comspec);

    let path = std::env::var("PATH").unwrap_or_else(|_| {
        format!(
            "{}\\System32;{}\\System32\\WindowsPowerShell\\v1.0;{}",
            sys_root_str, sys_root_str, sys_root_str
        )
    });
    env_setter("PATH", &path);
}
