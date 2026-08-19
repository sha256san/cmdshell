use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousVerdict {
    pub is_dangerous: bool,
    pub risk_level: RiskLevel,
    pub explanation: String,
}

impl DangerousVerdict {
    pub fn safe() -> Self {
        Self {
            is_dangerous: false,
            risk_level: RiskLevel::Low,
            explanation: String::new(),
        }
    }

    pub fn dangerous(risk_level: RiskLevel, explanation: impl Into<String>) -> Self {
        Self {
            is_dangerous: true,
            risk_level,
            explanation: explanation.into(),
        }
    }
}

pub struct DangerousDetector {
    custom_patterns: Vec<Regex>,
}

impl Default for DangerousDetector {
    fn default() -> Self {
        Self::new(&[])
    }
}

impl DangerousDetector {
    pub fn new(custom_pattern_strings: &[String]) -> Self {
        let mut custom_patterns = Vec::new();
        for pat in custom_pattern_strings {
            if let Ok(re) = Regex::new(pat) {
                custom_patterns.push(re);
            }
        }
        Self { custom_patterns }
    }

    pub fn inspect(&self, command: &str) -> DangerousVerdict {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return DangerousVerdict::safe();
        }

        // 1. Fork bomb
        if trimmed.contains(":(){ :|:& };:") || trimmed.contains(":(){:|:&};:") {
            return DangerousVerdict::dangerous(
                RiskLevel::Critical,
                "Fork bomb detected! This will freeze your operating system.",
            );
        }

        // 2. Destructive rm
        if trimmed.starts_with("rm ") || trimmed.contains(" rm ") {
            if trimmed.contains("-rf /") || trimmed.contains("-fr /") || trimmed.contains("-r -f /") {
                return DangerousVerdict::dangerous(
                    RiskLevel::Critical,
                    "Command attempts to recursively delete the root filesystem (`/`).",
                );
            }
            if trimmed.contains("-rf ~") || trimmed.contains("-rf $HOME") || trimmed.contains("-rf *") {
                return DangerousVerdict::dangerous(
                    RiskLevel::High,
                    "Recursive forceful deletion of entire home directory or wildcard files.",
                );
            }
            if trimmed.contains("-rf") || trimmed.contains("-fr") || (trimmed.contains("-r") && trimmed.contains("-f")) {
                return DangerousVerdict::dangerous(
                    RiskLevel::Medium,
                    "Recursive forceful deletion (`rm -rf`). Files cannot be restored from trash.",
                );
            }
        }

        // 3. Disk formatting and raw block writing
        if trimmed.starts_with("mkfs") || trimmed.contains(" mkfs") {
            return DangerousVerdict::dangerous(
                RiskLevel::Critical,
                "Filesystem formatting command (`mkfs`) will permanently erase target partition.",
            );
        }
        if (trimmed.starts_with("dd ") || trimmed.contains(" dd ")) && trimmed.contains("of=/dev/") {
            return DangerousVerdict::dangerous(
                RiskLevel::Critical,
                "Raw disk write (`dd of=/dev/...`) will overwrite partition tables or device data.",
            );
        }

        // 4. Git destructive actions
        if trimmed.starts_with("git reset --hard") || trimmed.contains("git reset --hard") {
            return DangerousVerdict::dangerous(
                RiskLevel::High,
                "Hard git reset will discard all uncommitted and unstaged working tree changes.",
            );
        }
        if trimmed.starts_with("git clean -fd") || trimmed.contains("git clean -f") {
            return DangerousVerdict::dangerous(
                RiskLevel::Medium,
                "Git clean will permanently delete all untracked files and directories.",
            );
        }
        if (trimmed.starts_with("git push ") || trimmed.contains("git push ")) && (trimmed.contains("--force") || trimmed.contains(" -f ")) {
            return DangerousVerdict::dangerous(
                RiskLevel::High,
                "Force push can overwrite remote branch history and erase commits.",
            );
        }

        // 5. System shutdown/reboot
        if trimmed == "shutdown" || trimmed.starts_with("shutdown ") || trimmed == "reboot" || trimmed == "poweroff" || trimmed == "init 0" || trimmed == "init 6" {
            return DangerousVerdict::dangerous(
                RiskLevel::High,
                "System shutdown or reboot command.",
            );
        }

        // 6. Dangerous permissions
        if (trimmed.starts_with("chmod -R 777 /") || trimmed.contains(" chmod -R 777 /")) && !trimmed.contains("/tmp") {
            return DangerousVerdict::dangerous(
                RiskLevel::Critical,
                "Changing root permissions to 777 compromises system security.",
            );
        }

        // 7. Custom regex patterns
        for custom in &self.custom_patterns {
            if custom.is_match(trimmed) {
                return DangerousVerdict::dangerous(
                    RiskLevel::Medium,
                    format!("Matches custom security rule: `{}`", custom.as_str()),
                );
            }
        }

        DangerousVerdict::safe()
    }
}
