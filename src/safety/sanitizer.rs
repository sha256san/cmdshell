use regex::Regex;

pub struct SecretSanitizer {
    token_patterns: Vec<Regex>,
    assignment_patterns: Vec<Regex>,
}

impl Default for SecretSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretSanitizer {
    pub fn new() -> Self {
        let token_patterns = vec![
            // GitHub personal access tokens
            Regex::new(r"ghp_[A-Za-z0-9_]{36,}").unwrap(),
            // GitLab personal access tokens
            Regex::new(r"glpat-[A-Za-z0-9_\-]{20,}").unwrap(),
            // Slack tokens
            Regex::new(r"xox[baprs]-[0-9a-zA-Z]{10,48}").unwrap(),
            // OpenAI / Anthropic / Generic API keys
            Regex::new(r"sk-[A-Za-z0-9_\-]{20,}").unwrap(),
            // AWS Access Key IDs
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            // Generic Bearer token
            Regex::new(r"(?i)bearer\s+[A-Za-z0-9\-_.~+/]+=*").unwrap(),
        ];

        let assignment_patterns = vec![
            Regex::new(r#"(?i)(password|passwd|pwd|secret|token|api_key|apikey|auth_token|access_token|private_key)\s*=\s*(?:'[^']*'|"[^"]*"|[^\s\n]+)"#).unwrap(),
            Regex::new(r#"(?i)(--password|--token|--secret|--api-key)\s+(?:'[^']*'|"[^"]*"|[^\s\n]+)"#).unwrap(),
        ];

        Self {
            token_patterns,
            assignment_patterns,
        }
    }

    pub fn sanitize(&self, command: &str) -> String {
        let mut sanitized = command.to_string();

        // Mask well-known tokens
        for pattern in &self.token_patterns {
            sanitized = pattern.replace_all(&sanitized, "[REDACTED_SECRET]").to_string();
        }

        // Mask assignments (like API_KEY=xyz)
        for pattern in &self.assignment_patterns {
            sanitized = pattern.replace_all(&sanitized, "$1=[REDACTED_SECRET]").to_string();
        }

        sanitized
    }

    pub fn contains_secret(&self, command: &str) -> bool {
        for pattern in &self.token_patterns {
            if pattern.is_match(command) {
                return true;
            }
        }
        for pattern in &self.assignment_patterns {
            if pattern.is_match(command) {
                return true;
            }
        }
        false
    }
}
