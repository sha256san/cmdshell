use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::config::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub shell: Option<String>,
    pub scrollback_lines: usize,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    pub bell_sound: bool,
    pub copy_on_select: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Beam,
    Underline,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None,
            scrollback_lines: 10_000,
            font_family: "JetBrainsMono Nerd Font".to_string(),
            font_size: 14.0,
            line_height: 1.3,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            bell_sound: false,
            copy_on_select: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    pub enabled: bool,
    pub ghost_text_enabled: bool,
    pub popup_enabled: bool,
    pub debounce_ms: u64,
    pub max_suggestions: usize,
    pub min_prefix_length: usize,
    pub enable_history: bool,
    pub enable_filesystem: bool,
    pub enable_commands: bool,
    pub enable_git: bool,
    pub enable_project: bool,
    pub enable_ai: bool,
    pub ai_api_endpoint: Option<String>,
    pub ai_model: Option<String>,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ghost_text_enabled: true,
            popup_enabled: true,
            debounce_ms: 20,
            max_suggestions: 8,
            min_prefix_length: 1,
            enable_history: true,
            enable_filesystem: true,
            enable_commands: true,
            enable_git: true,
            enable_project: true,
            enable_ai: false,
            ai_api_endpoint: None,
            ai_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub enable_dangerous_confirmation: bool,
    pub mask_secrets_in_history: bool,
    pub custom_dangerous_patterns: Vec<String>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enable_dangerous_confirmation: true,
            mask_secrets_in_history: true,
            custom_dangerous_patterns: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub terminal: TerminalConfig,
    pub prediction: PredictionConfig,
    pub safety: SafetyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            terminal: TerminalConfig::default(),
            prediction: PredictionConfig::default(),
            safety: SafetyConfig::default(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "predictterm", "predictterm") {
            proj_dirs.config_dir().to_path_buf()
        } else {
            PathBuf::from(".config/predictterm")
        }
    }

    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load_or_default() -> Self {
        let path = Self::config_file_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
