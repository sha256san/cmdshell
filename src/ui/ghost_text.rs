use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostTextView {
    pub text: String,
    pub visible: bool,
}

impl GhostTextView {
    pub fn new(text: Option<String>) -> Self {
        match text {
            Some(t) if !t.is_empty() => Self { text: t, visible: true },
            _ => Self { text: String::new(), visible: false },
        }
    }
}
