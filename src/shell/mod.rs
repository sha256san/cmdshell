pub mod environment;
pub mod health;
pub mod linux;
pub mod macos;
pub mod resolver;
pub mod windows;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    PowerShell,
    WindowsPowerShell,
    Cmd,
    GitBash,
    Wsl,
    UnixBash,
    UnixZsh,
    UnixSh,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellInfo {
    pub name: String,
    pub path: PathBuf,
    pub kind: ShellKind,
    pub is_available: bool,
    pub default_args: Vec<String>,
}

pub use environment::EnvironmentBuilder;
pub use health::{HealthStatus, ShellHealthChecker};
pub use resolver::ShellResolver;
