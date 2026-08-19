use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Self::rgb(r, g, b))
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Self::rgba(r, g, b, a))
        } else {
            None
        }
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Color,
    pub accent: Color,
    pub ghost_text: Color,
    pub suggestion_bg: Color,
    pub suggestion_selected: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
    pub ansi_colors: [Color; 16],
}

impl Default for Theme {
    fn default() -> Self {
        Self::tokyo_night()
    }
}

impl Theme {
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            background: Color::rgb(0x1a, 0x1b, 0x26),
            foreground: Color::rgb(0xc0, 0xca, 0xf5),
            cursor: Color::rgb(0xc0, 0xca, 0xf5),
            selection: Color::rgba(0x36, 0x4a, 0x82, 0x80),
            accent: Color::rgb(0x7a, 0xa2, 0xf7),
            ghost_text: Color::rgb(0x56, 0x5f, 0x89),
            suggestion_bg: Color::rgb(0x24, 0x28, 0x3b),
            suggestion_selected: Color::rgb(0x41, 0x48, 0x68),
            warning: Color::rgb(0xe0, 0xaf, 0x68),
            error: Color::rgb(0xf7, 0x76, 0x8e),
            success: Color::rgb(0x9e, 0xce, 0x6a),
            ansi_colors: [
                Color::rgb(0x15, 0x16, 0x1e), // Black
                Color::rgb(0xf7, 0x76, 0x8e), // Red
                Color::rgb(0x9e, 0xce, 0x6a), // Green
                Color::rgb(0xe0, 0xaf, 0x68), // Yellow
                Color::rgb(0x7a, 0xa2, 0xf7), // Blue
                Color::rgb(0xbb, 0x9a, 0xf7), // Magenta
                Color::rgb(0x7d, 0xcf, 0xff), // Cyan
                Color::rgb(0xa9, 0xb1, 0xd6), // White
                Color::rgb(0x41, 0x48, 0x68), // Bright Black
                Color::rgb(0xf7, 0x76, 0x8e), // Bright Red
                Color::rgb(0x9e, 0xce, 0x6a), // Bright Green
                Color::rgb(0xe0, 0xaf, 0x68), // Bright Yellow
                Color::rgb(0x7a, 0xa2, 0xf7), // Bright Blue
                Color::rgb(0xbb, 0x9a, 0xf7), // Bright Magenta
                Color::rgb(0x7d, 0xcf, 0xff), // Bright Cyan
                Color::rgb(0xc0, 0xca, 0xf5), // Bright White
            ],
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            background: Color::rgb(0x1e, 0x1e, 0x2e),
            foreground: Color::rgb(0xcd, 0xd6, 0xf4),
            cursor: Color::rgb(0xf5, 0xe0, 0xdc),
            selection: Color::rgba(0x58, 0x5b, 0x70, 0x80),
            accent: Color::rgb(0x89, 0xb4, 0xfa),
            ghost_text: Color::rgb(0x6c, 0x70, 0x86),
            suggestion_bg: Color::rgb(0x31, 0x32, 0x44),
            suggestion_selected: Color::rgb(0x45, 0x47, 0x5a),
            warning: Color::rgb(0xf9, 0xe2, 0xaf),
            error: Color::rgb(0xf3, 0x8b, 0xa8),
            success: Color::rgb(0xa6, 0xe3, 0xa1),
            ansi_colors: [
                Color::rgb(0x45, 0x47, 0x5a),
                Color::rgb(0xf3, 0x8b, 0xa8),
                Color::rgb(0xa6, 0xe3, 0xa1),
                Color::rgb(0xf9, 0xe2, 0xaf),
                Color::rgb(0x89, 0xb4, 0xfa),
                Color::rgb(0xf5, 0xc2, 0xe7),
                Color::rgb(0x94, 0xe2, 0xd5),
                Color::rgb(0xba, 0xc2, 0xde),
                Color::rgb(0x58, 0x5b, 0x70),
                Color::rgb(0xf3, 0x8b, 0xa8),
                Color::rgb(0xa6, 0xe3, 0xa1),
                Color::rgb(0xf9, 0xe2, 0xaf),
                Color::rgb(0x89, 0xb4, 0xfa),
                Color::rgb(0xf5, 0xc2, 0xe7),
                Color::rgb(0x94, 0xe2, 0xd5),
                Color::rgb(0xa6, 0xad, 0xc8),
            ],
        }
    }
}
