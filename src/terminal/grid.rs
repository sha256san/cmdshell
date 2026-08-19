use std::collections::VecDeque;
use crate::terminal::cell::{Cell, CellColor, CellFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
}

impl Selection {
    pub fn normalized(&self) -> (Position, Position) {
        if self.start.row < self.end.row
            || (self.start.row == self.end.row && self.start.col <= self.end.col)
        {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    pub fn contains(&self, pos: Position) -> bool {
        let (start, end) = self.normalized();
        if pos.row < start.row || pos.row > end.row {
            return false;
        }
        if start.row == end.row {
            pos.col >= start.col && pos.col <= end.col
        } else if pos.row == start.row {
            pos.col >= start.col
        } else if pos.row == end.row {
            pos.col <= end.col
        } else {
            true
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerminalGrid {
    pub cols: usize,
    pub rows: usize,
    pub max_scrollback: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub saved_cursor_x: usize,
    pub saved_cursor_y: usize,
    pub cursor_visible: bool,
    pub scroll_offset: usize, // 0 = viewing active screen, > 0 = viewing scrollback
    pub selection: Option<Selection>,
    pub lines: Vec<Vec<Cell>>,
    pub scrollback: VecDeque<Vec<Cell>>,
}

impl TerminalGrid {
    pub fn new(cols: usize, rows: usize, max_scrollback: usize) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        let lines = vec![vec![Cell::blank(); cols]; rows];
        Self {
            cols,
            rows,
            max_scrollback,
            cursor_x: 0,
            cursor_y: 0,
            saved_cursor_x: 0,
            saved_cursor_y: 0,
            cursor_visible: true,
            scroll_offset: 0,
            selection: None,
            lines,
            scrollback: VecDeque::with_capacity(max_scrollback.min(1000)),
        }
    }

    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        let new_cols = new_cols.max(1);
        let new_rows = new_rows.max(1);

        if self.cols == new_cols && self.rows == new_rows {
            return;
        }

        let mut new_lines = vec![vec![Cell::blank(); new_cols]; new_rows];
        for (r, row) in self.lines.iter().enumerate().take(new_rows) {
            for (c, cell) in row.iter().enumerate().take(new_cols) {
                new_lines[r][c] = *cell;
            }
        }

        self.cols = new_cols;
        self.rows = new_rows;
        self.lines = new_lines;
        self.cursor_x = self.cursor_x.min(self.cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(self.rows.saturating_sub(1));
    }

    pub fn write_char(&mut self, c: char, fg: CellColor, bg: CellColor, flags: CellFlags) {
        // Reset user scroll offset on new output
        self.scroll_offset = 0;

        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.line_feed();
        }

        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            self.lines[self.cursor_y][self.cursor_x] = Cell { c, fg, bg, flags };
            self.cursor_x += 1;
        }
    }

    pub fn line_feed(&mut self) {
        if self.cursor_y + 1 < self.rows {
            self.cursor_y += 1;
        } else {
            // Scroll down: move top visible line to scrollback
            let old_line = self.lines.remove(0);
            if self.scrollback.len() >= self.max_scrollback && !self.scrollback.is_empty() {
                self.scrollback.pop_front();
            }
            self.scrollback.push_back(old_line);
            self.lines.push(vec![Cell::blank(); self.cols]);
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor_x = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn tab(&mut self) {
        let next_tab = (self.cursor_x / 8 + 1) * 8;
        self.cursor_x = next_tab.min(self.cols.saturating_sub(1));
    }

    pub fn move_cursor_to(&mut self, col: usize, row: usize) {
        self.cursor_x = col.min(self.cols.saturating_sub(1));
        self.cursor_y = row.min(self.rows.saturating_sub(1));
    }

    pub fn move_cursor_relative(&mut self, dx: isize, dy: isize) {
        let new_x = (self.cursor_x as isize + dx).clamp(0, self.cols.saturating_sub(1) as isize);
        let new_y = (self.cursor_y as isize + dy).clamp(0, self.rows.saturating_sub(1) as isize);
        self.cursor_x = new_x as usize;
        self.cursor_y = new_y as usize;
    }

    pub fn save_cursor(&mut self) {
        self.saved_cursor_x = self.cursor_x;
        self.saved_cursor_y = self.cursor_y;
    }

    pub fn restore_cursor(&mut self) {
        self.cursor_x = self.saved_cursor_x.min(self.cols.saturating_sub(1));
        self.cursor_y = self.saved_cursor_y.min(self.rows.saturating_sub(1));
    }

    pub fn erase_in_line(&mut self, mode: u8) {
        if self.cursor_y >= self.rows {
            return;
        }
        match mode {
            0 => {
                // Erase from cursor to end of line
                for c in self.cursor_x..self.cols {
                    self.lines[self.cursor_y][c] = Cell::blank();
                }
            }
            1 => {
                // Erase from start of line to cursor
                for c in 0..=self.cursor_x.min(self.cols.saturating_sub(1)) {
                    self.lines[self.cursor_y][c] = Cell::blank();
                }
            }
            2 => {
                // Erase entire line
                for c in 0..self.cols {
                    self.lines[self.cursor_y][c] = Cell::blank();
                }
            }
            _ => {}
        }
    }

    pub fn erase_in_display(&mut self, mode: u8) {
        match mode {
            0 => {
                // From cursor to end of display
                self.erase_in_line(0);
                for r in (self.cursor_y + 1)..self.rows {
                    for c in 0..self.cols {
                        self.lines[r][c] = Cell::blank();
                    }
                }
            }
            1 => {
                // From start of display to cursor
                for r in 0..self.cursor_y {
                    for c in 0..self.cols {
                        self.lines[r][c] = Cell::blank();
                    }
                }
                self.erase_in_line(1);
            }
            2 => {
                // Entire display
                for r in 0..self.rows {
                    for c in 0..self.cols {
                        self.lines[r][c] = Cell::blank();
                    }
                }
            }
            3 => {
                // Clear scrollback
                self.scrollback.clear();
                self.scroll_offset = 0;
            }
            _ => {}
        }
    }

    pub fn scroll_viewport_up(&mut self, lines: usize) {
        let max_scroll = self.scrollback.len();
        self.scroll_offset = (self.scroll_offset + lines).min(max_scroll);
    }

    pub fn scroll_viewport_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn total_lines(&self) -> usize {
        self.scrollback.len() + self.rows
    }

    pub fn get_line(&self, view_row: usize) -> &[Cell] {
        if self.scroll_offset == 0 {
            if view_row < self.rows {
                &self.lines[view_row]
            } else {
                &[]
            }
        } else {
            // Viewing history + active
            let history_len = self.scrollback.len();
            let start_idx = history_len.saturating_sub(self.scroll_offset);
            let absolute_idx = start_idx + view_row;

            if absolute_idx < history_len {
                &self.scrollback[absolute_idx]
            } else {
                let screen_idx = absolute_idx - history_len;
                if screen_idx < self.rows {
                    &self.lines[screen_idx]
                } else {
                    &[]
                }
            }
        }
    }

    pub fn get_line_text(&self, view_row: usize) -> String {
        let line = self.get_line(view_row);
        let mut text: String = line.iter().map(|c| c.c).collect();
        while text.ends_with(' ') {
            text.pop();
        }
        text
    }

    pub fn get_screen_text(&self) -> String {
        let mut result = String::new();
        for r in 0..self.rows {
            result.push_str(&self.get_line_text(r));
            if r + 1 < self.rows {
                result.push('\n');
            }
        }
        result
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let sel = self.selection?;
        let (start, end) = sel.normalized();
        let mut result = String::new();

        for r in start.row..=end.row {
            let line = self.get_line(r);
            let start_col = if r == start.row { start.col } else { 0 };
            let end_col = if r == end.row { end.col.min(line.len().saturating_sub(1)) } else { line.len().saturating_sub(1) };

            for c in start_col..=end_col {
                if c < line.len() {
                    result.push(line[c].c);
                }
            }
            if r < end.row {
                result.push('\n');
            }
        }

        Some(result)
    }
}
