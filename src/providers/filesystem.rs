use std::fs;
use std::path::{Path, PathBuf};
use crate::predictor::candidate::{Candidate, CandidateSource};
use crate::predictor::context::PredictionContext;
use crate::providers::CandidateProvider;

pub struct FilesystemProvider;

impl Default for FilesystemProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemProvider {
    pub fn new() -> Self {
        Self
    }

    fn resolve_search_dir<'a>(token: &'a str, cwd: &Path) -> (PathBuf, &'a str, String) {
        if token.starts_with('~') {
            if let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
                let rest = token.strip_prefix('~').unwrap().trim_start_matches('/');
                if let Some(last_slash) = rest.rfind('/') {
                    let dir_part = &rest[..last_slash];
                    let prefix = &rest[last_slash + 1..];
                    (home.join(dir_part), prefix, format!("~/{}", &rest[..=last_slash]))
                } else {
                    (home, rest, "~/".to_string())
                }
            } else {
                (cwd.to_path_buf(), token, String::new())
            }
        } else if token.starts_with('/') {
            let path = Path::new(token);
            if let Some(parent) = path.parent() {
                let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
                let prefix_dir = parent.to_str().unwrap_or("/").to_string();
                let prefix_dir = if prefix_dir.ends_with('/') { prefix_dir } else { format!("{}/", prefix_dir) };
                (parent.to_path_buf(), file_name, prefix_dir)
            } else {
                (PathBuf::from("/"), token.trim_start_matches('/'), "/".to_string())
            }
        } else if let Some(last_slash) = token.rfind('/') {
            let dir_part = &token[..last_slash];
            let file_prefix = &token[last_slash + 1..];
            (cwd.join(dir_part), file_prefix, format!("{}/", dir_part))
        } else {
            (cwd.to_path_buf(), token, String::new())
        }
    }
}

impl CandidateProvider for FilesystemProvider {
    fn name(&self) -> &'static str {
        "Filesystem"
    }

    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate> {
        let token = context.current_token();

        // If at command position and doesn't start with path indicator (./, ../, /, ~), skip
        if context.is_at_command_position() && !token.starts_with("./") && !token.starts_with("../") && !token.starts_with('/') && !token.starts_with('~') {
            return Vec::new();
        }

        let (search_dir, file_prefix, path_prefix) = Self::resolve_search_dir(token, &context.cwd);
        let mut candidates = Vec::new();

        if let Ok(entries) = fs::read_dir(&search_dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    // Ignore hidden files unless user started with '.'
                    if file_name.starts_with('.') && !file_prefix.starts_with('.') {
                        continue;
                    }

                    if file_prefix.is_empty() || file_name.to_lowercase().starts_with(&file_prefix.to_lowercase()) {
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        let formatted_name = if is_dir {
                            format!("{}{}/", path_prefix, file_name)
                        } else {
                            format!("{}{}", path_prefix, file_name)
                        };

                        let desc = if is_dir {
                            "Directory".to_string()
                        } else if let Ok(meta) = entry.metadata() {
                            let len = meta.len();
                            if len < 1024 {
                                format!("{} B", len)
                            } else if len < 1024 * 1024 {
                                format!("{:.1} KB", len as f64 / 1024.0)
                            } else {
                                format!("{:.1} MB", len as f64 / (1024.0 * 1024.0))
                            }
                        } else {
                            "File".to_string()
                        };

                        let score = if is_dir { 45.0 } else { 40.0 };
                        candidates.push(
                            Candidate::new(formatted_name, CandidateSource::Filesystem, score)
                                .with_description(desc)
                                .with_prefix_len(token.len()),
                        );
                    }
                }
            }
        }

        candidates
    }
}
