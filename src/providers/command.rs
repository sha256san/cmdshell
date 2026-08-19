use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use crate::predictor::candidate::{Candidate, CandidateSource};
use crate::predictor::context::PredictionContext;
use crate::providers::CandidateProvider;

pub struct CommandProvider {
    cached_commands: RwLock<HashSet<String>>,
}

impl Default for CommandProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProvider {
    pub fn new() -> Self {
        let mut commands = HashSet::new();

        // Builtins and common utilities with descriptions
        let common = [
            ("cd", "Change the working directory"),
            ("ls", "List directory contents"),
            ("cat", "Concatenate and display files"),
            ("grep", "Search text using patterns"),
            ("find", "Search for files in a directory hierarchy"),
            ("mkdir", "Create directories"),
            ("rm", "Remove files or directories"),
            ("cp", "Copy files and directories"),
            ("mv", "Move or rename files and directories"),
            ("pwd", "Print name of current working directory"),
            ("touch", "Change file access and modification times"),
            ("chmod", "Change file mode bits (permissions)"),
            ("chown", "Change file owner and group"),
            ("tar", "Archiving utility"),
            ("curl", "Transfer data from or to a server"),
            ("wget", "Non-interactive network downloader"),
            ("ssh", "OpenSSH SSH client (remote login program)"),
            ("git", "Fast scalable distributed revision control system"),
            ("cargo", "Rust package manager and build tool"),
            ("rustc", "The Rust compiler"),
            ("docker", "Docker container engine runtime"),
            ("docker-compose", "Define and run multi-container applications"),
            ("node", "JavaScript runtime environment"),
            ("npm", "Node package manager"),
            ("pnpm", "Fast, disk space efficient package manager"),
            ("yarn", "Fast, reliable, and secure dependency management"),
            ("python", "Python programming language interpreter"),
            ("python3", "Python 3 interpreter"),
            ("pip", "Package installer for Python"),
            ("uv", "Extremely fast Python package installer"),
            ("go", "Go programming language tool"),
            ("make", "GNU make utility to maintain groups of programs"),
            ("cmake", "Cross-platform build system generator"),
            ("htop", "Interactive process viewer"),
            ("top", "Display Linux processes"),
            ("ps", "Report a snapshot of current processes"),
            ("kill", "Send a signal to a process"),
            ("df", "Report file system disk space usage"),
            ("du", "Estimate file space usage"),
            ("clear", "Clear terminal screen"),
            ("exit", "Cause normal process termination"),
            ("echo", "Display a line of text"),
            ("which", "Locate a command"),
            ("man", "Interface to system reference manuals"),
            ("sudo", "Execute a command as another user / root"),
        ];

        for (cmd, _) in common {
            commands.insert(cmd.to_string());
        }

        // Scan PATH
        if let Some(path_var) = env::var_os("PATH") {
            for dir in env::split_paths(&path_var) {
                Self::scan_dir(&dir, &mut commands);
            }
        }

        Self {
            cached_commands: RwLock::new(commands),
        }
    }

    fn scan_dir(dir: &Path, commands: &mut HashSet<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() || file_type.is_symlink() {
                        if let Ok(name) = entry.file_name().into_string() {
                            if !name.starts_with('.') {
                                commands.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_command_description(cmd: &str) -> Option<&'static str> {
        match cmd {
            "cd" => Some("Change directory"),
            "ls" => Some("List directory contents"),
            "cat" => Some("Display file contents"),
            "grep" => Some("Search text with regex"),
            "find" => Some("Search files"),
            "mkdir" => Some("Make directory"),
            "rm" => Some("Remove files"),
            "cp" => Some("Copy files"),
            "mv" => Some("Move / rename files"),
            "git" => Some("Git version control"),
            "cargo" => Some("Rust build & package tool"),
            "docker" => Some("Manage containers"),
            "node" => Some("Run Node.js"),
            "npm" => Some("Node package manager"),
            "python" | "python3" => Some("Python interpreter"),
            "curl" => Some("Transfer data with URLs"),
            "ssh" => Some("SSH client"),
            "htop" => Some("Process monitor"),
            "clear" => Some("Clear screen"),
            _ => None,
        }
    }
}

impl CandidateProvider for CommandProvider {
    fn name(&self) -> &'static str {
        "Command"
    }

    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate> {
        if !context.is_at_command_position() {
            return Vec::new();
        }

        let token = context.current_token();
        let guard = self.cached_commands.read().unwrap();
        let mut candidates = Vec::new();

        for cmd in guard.iter() {
            if token.is_empty() || cmd.starts_with(token) {
                let mut candidate = Candidate::new(cmd.clone(), CandidateSource::Command, 50.0)
                    .with_prefix_len(token.len());
                if let Some(desc) = Self::get_command_description(cmd) {
                    candidate = candidate.with_description(desc);
                }
                candidates.push(candidate);
            }
        }

        candidates
    }
}
