use std::process::Command;
use crate::shell::{ShellInfo, ShellKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Failed {
        exit_code: Option<i32>,
        error_message: String,
        is_0xc0000142: bool,
    },
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}

pub struct ShellHealthChecker;

impl ShellHealthChecker {
    /// Fast dry-run health probe to verify if a shell can successfully initialize without 0xc0000142.
    pub fn check(shell: &ShellInfo) -> HealthStatus {
        if !shell.path.exists() {
            return HealthStatus::Failed {
                exit_code: None,
                error_message: format!("Binary not found at path: {}", shell.path.display()),
                is_0xc0000142: false,
            };
        }

        let mut cmd = Command::new(&shell.path);

        // Apply health-check probe arguments
        match shell.kind {
            ShellKind::PowerShell | ShellKind::WindowsPowerShell => {
                cmd.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", "exit 0"]);
            }
            ShellKind::Cmd => {
                cmd.args(["/C", "exit 0"]);
            }
            ShellKind::GitBash | ShellKind::UnixBash | ShellKind::UnixZsh | ShellKind::UnixSh => {
                cmd.args(["-c", "exit 0"]);
            }
            ShellKind::Wsl => {
                cmd.args(["--exec", "true"]);
            }
            ShellKind::Custom => {
                cmd.args(["-c", "exit 0"]);
            }
        }

        // Apply normalized environment for probe
        let envs = crate::shell::environment::EnvironmentBuilder::build_shell_environment(None);
        cmd.envs(envs);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    HealthStatus::Healthy
                } else {
                    let code = output.status.code();
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                    // 0xc0000142 on Windows maps to -1073741502 or 3221225794
                    let is_0xc0000142 = match code {
                        Some(c) => (c as u32) == 0xC0000142 || c == -1073741502 || c == 0x142,
                        None => false,
                    };

                    let error_msg = if is_0xc0000142 {
                        "STATUS_DLL_INIT_FAILED (0xc0000142): DLL initialization failed during startup".to_string()
                    } else if !stderr.is_empty() {
                        stderr
                    } else {
                        format!("Exited with status code {:?}", code)
                    };

                    HealthStatus::Failed {
                        exit_code: code,
                        error_message: error_msg,
                        is_0xc0000142,
                    }
                }
            }
            Err(e) => HealthStatus::Failed {
                exit_code: None,
                error_message: e.to_string(),
                is_0xc0000142: false,
            },
        }
    }
}
