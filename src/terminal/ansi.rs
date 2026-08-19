use vte::{Params, Parser, Perform};
use crate::terminal::cell::{CellColor, CellFlags};
use crate::terminal::grid::TerminalGrid;

pub struct AnsiHandler<'a> {
    pub grid: &'a mut TerminalGrid,
    pub fg: CellColor,
    pub bg: CellColor,
    pub flags: CellFlags,
    pub title: Option<String>,
    pub bell: bool,
}

impl<'a> AnsiHandler<'a> {
    pub fn new(grid: &'a mut TerminalGrid) -> Self {
        Self {
            grid,
            fg: CellColor::Default,
            bg: CellColor::Default,
            flags: CellFlags::empty(),
            title: None,
            bell: false,
        }
    }
}

impl<'a> Perform for AnsiHandler<'a> {
    fn print(&mut self, c: char) {
        self.grid.write_char(c, self.fg, self.bg, self.flags);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x07 => self.bell = true, // BEL
            0x08 => self.grid.backspace(), // BS
            0x09 => self.grid.tab(), // TAB
            0x0A | 0x0B | 0x0C => self.grid.line_feed(), // LF, VT, FF
            0x0D => self.grid.carriage_return(), // CR
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.is_empty() {
            return;
        }
        // OSC 0;title or OSC 2;title
        if (params[0] == b"0" || params[0] == b"2") && params.len() > 1 {
            if let Ok(title_str) = std::str::from_utf8(params[1]) {
                self.title = Some(title_str.to_string());
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        let is_private = intermediates.contains(&b'?');

        match action {
            // SGR - Select Graphic Rendition
            'm' => {
                let mut iter = params.iter().map(|p| p.first().copied().unwrap_or(0));
                if params.is_empty() {
                    self.fg = CellColor::Default;
                    self.bg = CellColor::Default;
                    self.flags = CellFlags::empty();
                    return;
                }

                while let Some(param) = iter.next() {
                    match param {
                        0 => {
                            self.fg = CellColor::Default;
                            self.bg = CellColor::Default;
                            self.flags = CellFlags::empty();
                        }
                        1 => self.flags.insert(CellFlags::BOLD),
                        2 => self.flags.insert(CellFlags::DIM),
                        3 => self.flags.insert(CellFlags::ITALIC),
                        4 => self.flags.insert(CellFlags::UNDERLINE),
                        5 | 6 => self.flags.insert(CellFlags::BLINK),
                        7 => self.flags.insert(CellFlags::INVERSE),
                        8 => self.flags.insert(CellFlags::HIDDEN),
                        9 => self.flags.insert(CellFlags::STRIKETHROUGH),
                        22 => {
                            self.flags.remove(CellFlags::BOLD);
                            self.flags.remove(CellFlags::DIM);
                        }
                        23 => self.flags.remove(CellFlags::ITALIC),
                        24 => self.flags.remove(CellFlags::UNDERLINE),
                        25 => self.flags.remove(CellFlags::BLINK),
                        27 => self.flags.remove(CellFlags::INVERSE),
                        28 => self.flags.remove(CellFlags::HIDDEN),
                        29 => self.flags.remove(CellFlags::STRIKETHROUGH),
                        30..=37 => self.fg = CellColor::Indexed((param - 30) as u8),
                        38 => {
                            // Extended foreground
                            match iter.next() {
                                Some(5) => {
                                    if let Some(idx) = iter.next() {
                                        self.fg = CellColor::Indexed(idx as u8);
                                    }
                                }
                                Some(2) => {
                                    let r = iter.next().unwrap_or(0) as u8;
                                    let g = iter.next().unwrap_or(0) as u8;
                                    let b = iter.next().unwrap_or(0) as u8;
                                    self.fg = CellColor::Rgb(r, g, b);
                                }
                                _ => {}
                            }
                        }
                        39 => self.fg = CellColor::Default,
                        40..=47 => self.bg = CellColor::Indexed((param - 40) as u8),
                        48 => {
                            // Extended background
                            match iter.next() {
                                Some(5) => {
                                    if let Some(idx) = iter.next() {
                                        self.bg = CellColor::Indexed(idx as u8);
                                    }
                                }
                                Some(2) => {
                                    let r = iter.next().unwrap_or(0) as u8;
                                    let g = iter.next().unwrap_or(0) as u8;
                                    let b = iter.next().unwrap_or(0) as u8;
                                    self.bg = CellColor::Rgb(r, g, b);
                                }
                                _ => {}
                            }
                        }
                        49 => self.bg = CellColor::Default,
                        90..=97 => self.fg = CellColor::Indexed((param - 90 + 8) as u8),
                        100..=107 => self.bg = CellColor::Indexed((param - 100 + 8) as u8),
                        _ => {}
                    }
                }
            }

            // Cursor Up
            'A' => {
                let count = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1).max(1);
                self.grid.move_cursor_relative(0, -(count as isize));
            }

            // Cursor Down
            'B' => {
                let count = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1).max(1);
                self.grid.move_cursor_relative(0, count as isize);
            }

            // Cursor Forward
            'C' => {
                let count = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1).max(1);
                self.grid.move_cursor_relative(count as isize, 0);
            }

            // Cursor Back
            'D' => {
                let count = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1).max(1);
                self.grid.move_cursor_relative(-(count as isize), 0);
            }

            // Cursor Position (H or f)
            'H' | 'f' => {
                let mut iter = params.iter().map(|p| p.first().copied().unwrap_or(1));
                let row = iter.next().unwrap_or(1).max(1) - 1;
                let col = iter.next().unwrap_or(1).max(1) - 1;
                self.grid.move_cursor_to(col as usize, row as usize);
            }

            // Erase in Display
            'J' => {
                let mode = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                self.grid.erase_in_display(mode);
            }

            // Erase in Line
            'K' => {
                let mode = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                self.grid.erase_in_line(mode);
            }

            // Save Cursor
            's' => self.grid.save_cursor(),

            // Restore Cursor
            'u' => self.grid.restore_cursor(),

            // Private mode set / reset (e.g., cursor visibility ?25h / ?25l)
            'h' if is_private => {
                for param in params.iter().filter_map(|p| p.first()) {
                    if *param == 25 {
                        self.grid.cursor_visible = true;
                    }
                }
            }
            'l' if is_private => {
                for param in params.iter().filter_map(|p| p.first()) {
                    if *param == 25 {
                        self.grid.cursor_visible = false;
                    }
                }
            }

            _ => {}
        }
    }
}

pub struct AnsiParser {
    parser: Parser,
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub fn process_bytes(&mut self, bytes: &[u8], grid: &mut TerminalGrid) -> (Option<String>, bool) {
        let mut handler = AnsiHandler::new(grid);
        self.parser.advance(&mut handler, bytes);
        (handler.title, handler.bell)
    }
}
