pub use crate::config::theme::{Color, Theme};

pub struct UiTokens {
    pub theme: Theme,
}

impl UiTokens {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
}
