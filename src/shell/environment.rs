use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(windows)]
use std::{collections::HashSet, path::Path};

pub struct EnvironmentBuilder;

impl EnvironmentBuilder {
    /// Builds a normalized, sanitized environment variable map for spawning shells.
    pub fn build_shell_environment(custom_env: Option<&HashMap<String, String>>) -> HashMap<String, String> {
        let mut envs: HashMap<String, String> = HashMap::new();

        // 1. Inherit existing process environment variables
        for (k, v) in std::env::vars() {
            envs.insert(k, v);
        }

        // 2. Windows-specific environment normalization
        #[cfg(windows)]
        {
            Self::normalize_windows_environment(&mut envs);
        }

        // 3. Unix-specific environment normalization
        #[cfg(not(windows))]
        {
            Self::normalize_unix_environment(&mut envs);
        }

        // 4. Merge custom user-provided environment variables
        if let Some(custom) = custom_env {
            for (k, v) in custom {
                envs.insert(k.clone(), v.clone());
            }
        }

        envs
    }

    #[cfg(windows)]
    pub fn normalize_windows_environment(envs: &mut HashMap<String, String>) {
        let system_root = crate::shell::windows::get_system_root();
        let sys_root_str = system_root.to_string_lossy().to_string();

        // Ensure SystemRoot and WINDIR
        envs.insert("SystemRoot".to_string(), sys_root_str.clone());
        envs.insert("WINDIR".to_string(), sys_root_str.clone());

        // Ensure SystemDrive
        if !envs.contains_key("SystemDrive") {
            let drive = system_root
                .components()
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .unwrap_or("C:")
                .to_string();
            envs.insert("SystemDrive".to_string(), drive);
        }

        // Ensure ComSpec
        if !envs.contains_key("ComSpec") {
            let comspec = system_root.join("System32").join("cmd.exe");
            envs.insert("ComSpec".to_string(), comspec.to_string_lossy().to_string());
        }

        // Ensure TEMP & TMP
        if !envs.contains_key("TEMP") {
            let temp = system_root.join("Temp");
            envs.insert("TEMP".to_string(), temp.to_string_lossy().to_string());
        }
        if !envs.contains_key("TMP") {
            if let Some(temp) = envs.get("TEMP") {
                envs.insert("TMP".to_string(), temp.clone());
            }
        }

        // Normalize and prioritize PATH
        let normalized_path = Self::build_normalized_windows_path(&system_root, envs.get("PATH").map(|s| s.as_str()));
        envs.insert("PATH".to_string(), normalized_path);
    }

    #[cfg(windows)]
    pub fn build_normalized_windows_path(system_root: &Path, existing_path: Option<&str>) -> String {
        let mut ordered_paths: Vec<PathBuf> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        let mut add_path = |p: PathBuf| {
            let key = p.to_string_lossy().to_lowercase();
            if !seen.contains(&key) {
                seen.insert(key);
                ordered_paths.push(p);
            }
        };

        // 1. Mandatory Windows core paths at front
        add_path(system_root.join("System32"));
        add_path(system_root.join("System32").join("Wbem"));
        add_path(system_root.join("System32").join("WindowsPowerShell").join("v1.0"));
        add_path(system_root.to_path_buf());

        // 2. Program Files PowerShell 7 if exists
        let program_files = crate::shell::windows::get_program_files();
        let pwsh7 = program_files.join("PowerShell").join("7");
        if pwsh7.exists() {
            add_path(pwsh7);
        }

        // 3. Existing PATH entries (deduplicated)
        if let Some(path_str) = existing_path {
            for entry in std::env::split_paths(path_str) {
                if entry.exists() || !entry.as_os_str().is_empty() {
                    add_path(entry);
                }
            }
        }

        std::env::join_paths(ordered_paths)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| format!("{}\\System32;{}\\System32\\WindowsPowerShell\\v1.0", system_root.display(), system_root.display()))
    }

    #[cfg(not(windows))]
    pub fn normalize_unix_environment(envs: &mut HashMap<String, String>) {
        if !envs.contains_key("TERM") {
            envs.insert("TERM".to_string(), "xterm-256color".to_string());
        }
        if !envs.contains_key("COLORTERM") {
            envs.insert("COLORTERM".to_string(), "truecolor".to_string());
        }

        // Ensure fundamental PATH entries
        let current_path = envs.get("PATH").map(|s| s.as_str()).unwrap_or("/usr/local/bin:/usr/bin:/bin");
        let mut paths: Vec<PathBuf> = std::env::split_paths(current_path).collect();
        let essential = [PathBuf::from("/usr/local/bin"), PathBuf::from("/usr/bin"), PathBuf::from("/bin")];

        for p in &essential {
            if !paths.contains(p) && p.exists() {
                paths.push(p.clone());
            }
        }

        if let Ok(joined) = std::env::join_paths(paths) {
            envs.insert("PATH".to_string(), joined.to_string_lossy().to_string());
        }
    }
}
