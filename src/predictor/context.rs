use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Cpp,
    CMake,
    Ruby,
    Java,
}

impl ProjectType {
    pub fn name(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust (Cargo)",
            ProjectType::Node => "Node.js (npm/yarn/pnpm)",
            ProjectType::Python => "Python (pip/poetry/uv)",
            ProjectType::Go => "Go",
            ProjectType::Cpp => "C/C++",
            ProjectType::CMake => "CMake",
            ProjectType::Ruby => "Ruby",
            ProjectType::Java => "Java (Maven/Gradle)",
        }
    }

    pub fn detect(dir: &Path) -> Option<Self> {
        if dir.join("Cargo.toml").exists() {
            Some(ProjectType::Rust)
        } else if dir.join("package.json").exists() {
            Some(ProjectType::Node)
        } else if dir.join("pyproject.toml").exists()
            || dir.join("requirements.txt").exists()
            || dir.join("setup.py").exists()
        {
            Some(ProjectType::Python)
        } else if dir.join("go.mod").exists() {
            Some(ProjectType::Go)
        } else if dir.join("CMakeLists.txt").exists() {
            Some(ProjectType::CMake)
        } else if dir.join("Makefile").exists() {
            Some(ProjectType::Cpp)
        } else if dir.join("Gemfile").exists() {
            Some(ProjectType::Ruby)
        } else if dir.join("pom.xml").exists() || dir.join("build.gradle").exists() {
            Some(ProjectType::Java)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitContext {
    pub branch: String,
    pub is_dirty: bool,
    pub modified_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub branches: Vec<String>,
    pub remotes: Vec<String>,
}

impl GitContext {
    pub fn detect(dir: &Path) -> Option<Self> {
        let dot_git = dir.join(".git");
        if !dot_git.exists() {
            return None;
        }

        // Try reading branch directly from .git/HEAD
        let head_file = dot_git.join("HEAD");
        let branch = if let Ok(head_content) = std::fs::read_to_string(head_file) {
            let trimmed = head_content.trim();
            if let Some(ref_path) = trimmed.strip_prefix("ref: refs/heads/") {
                ref_path.to_string()
            } else {
                trimmed.chars().take(8).collect()
            }
        } else {
            "main".to_string()
        };

        // Enumerate local branches in .git/refs/heads
        let mut branches = Vec::new();
        let refs_heads = dot_git.join("refs").join("heads");
        if let Ok(entries) = std::fs::read_dir(refs_heads) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    branches.push(name);
                }
            }
        }
        if branches.is_empty() {
            branches.push(branch.clone());
        }

        Some(Self {
            branch,
            is_dirty: false,
            modified_files: Vec::new(),
            staged_files: Vec::new(),
            branches,
            remotes: vec!["origin".to_string()],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionContext {
    pub input: String,
    pub cursor_pos: usize,
    pub cwd: PathBuf,
    pub shell: String,
    pub project_type: Option<ProjectType>,
    pub git: Option<GitContext>,
}

impl PredictionContext {
    pub fn new(input: impl Into<String>, cursor_pos: usize, cwd: PathBuf, shell: impl Into<String>) -> Self {
        let input = input.into();
        let project_type = ProjectType::detect(&cwd);
        let git = GitContext::detect(&cwd);

        Self {
            input,
            cursor_pos,
            cwd,
            shell: shell.into(),
            project_type,
            git,
        }
    }

    pub fn input_up_to_cursor(&self) -> &str {
        let pos = self.cursor_pos.min(self.input.len());
        &self.input[..pos]
    }

    pub fn tokens(&self) -> Vec<&str> {
        self.input.split_whitespace().collect()
    }

    pub fn tokens_up_to_cursor(&self) -> Vec<&str> {
        self.input_up_to_cursor().split_whitespace().collect()
    }

    pub fn current_token(&self) -> &str {
        let prefix = self.input_up_to_cursor();
        if prefix.ends_with(' ') {
            ""
        } else {
            prefix.split_whitespace().last().unwrap_or("")
        }
    }

    pub fn command_name(&self) -> Option<&str> {
        self.tokens_up_to_cursor().first().copied()
    }

    pub fn is_at_command_position(&self) -> bool {
        let prefix = self.input_up_to_cursor();
        let tokens: Vec<&str> = prefix.split_whitespace().collect();
        tokens.len() <= 1 && !prefix.ends_with(' ')
    }
}
