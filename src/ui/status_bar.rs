use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusBarView {
    pub shell: String,
    pub cwd: String,
    pub project_type: Option<String>,
    pub git_branch: Option<String>,
    pub ai_enabled: bool,
}

impl StatusBarView {
    pub fn new(
        shell: impl Into<String>,
        cwd: impl Into<String>,
        project_type: Option<String>,
        git_branch: Option<String>,
        ai_enabled: bool,
    ) -> Self {
        Self {
            shell: shell.into(),
            cwd: cwd.into(),
            project_type,
            git_branch,
            ai_enabled,
        }
    }
}
