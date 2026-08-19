use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellColor {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct CellFlags: u16 {
        const BOLD = 1 << 0;
        const DIM = 1 << 1;
        const ITALIC = 1 << 2;
        const UNDERLINE = 1 << 3;
        const BLINK = 1 << 4;
        const INVERSE = 1 << 5;
        const HIDDEN = 1 << 6;
        const STRIKETHROUGH = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub c: char,
    pub fg: CellColor,
    pub bg: CellColor,
    pub flags: CellFlags,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: CellColor::Default,
            bg: CellColor::Default,
            flags: CellFlags::empty(),
        }
    }
}

impl Cell {
    pub const fn new(c: char) -> Self {
        Self {
            c,
            fg: CellColor::Default,
            bg: CellColor::Default,
            flags: CellFlags::empty(),
        }
    }

    pub const fn blank() -> Self {
        Self::new(' ')
    }

    pub fn reset(&mut self) {
        *self = Self::blank();
    }

    pub fn is_empty(&self) -> bool {
        self.c == ' ' && self.bg == CellColor::Default && !self.flags.contains(CellFlags::INVERSE)
    }
}
