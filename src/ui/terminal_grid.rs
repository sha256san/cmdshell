use serde::{Deserialize, Serialize};
use crate::config::theme::{Color, Theme};
use crate::terminal::cell::{Cell, CellColor, CellFlags};
use crate::terminal::grid::{Position, TerminalGrid};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderCell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub is_cursor: bool,
    pub is_selected: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderRow {
    pub cells: Vec<RenderCell>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalGridRenderData {
    pub rows: Vec<RenderRow>,
    pub cursor_visible: bool,
    pub cursor_col: usize,
    pub cursor_row: usize,
}

impl TerminalGridRenderData {
    pub fn from_grid(grid: &TerminalGrid, theme: &Theme) -> Self {
        let mut rows = Vec::with_capacity(grid.rows);

        for r in 0..grid.rows {
            let line = grid.get_line(r);
            let mut cells = Vec::with_capacity(grid.cols);

            for c in 0..grid.cols {
                let cell = if c < line.len() { line[c] } else { Cell::blank() };
                let is_cursor = grid.cursor_visible && grid.cursor_x == c && grid.cursor_y == r;
                let is_selected = grid.selection.as_ref().map_or(false, |s| s.contains(Position { col: c, row: r }));

                let (fg, bg) = Self::resolve_colors(&cell, theme, is_selected);

                cells.push(RenderCell {
                    c: cell.c,
                    fg,
                    bg,
                    is_cursor,
                    is_selected,
                    bold: cell.flags.contains(CellFlags::BOLD),
                    italic: cell.flags.contains(CellFlags::ITALIC),
                    underline: cell.flags.contains(CellFlags::UNDERLINE),
                });
            }

            rows.push(RenderRow { cells });
        }

        Self {
            rows,
            cursor_visible: grid.cursor_visible,
            cursor_col: grid.cursor_x,
            cursor_row: grid.cursor_y,
        }
    }

    fn resolve_colors(cell: &Cell, theme: &Theme, is_selected: bool) -> (Color, Color) {
        if is_selected {
            return (theme.foreground, theme.selection);
        }

        let mut fg = match cell.fg {
            CellColor::Default => theme.foreground,
            CellColor::Indexed(idx) => {
                if (idx as usize) < theme.ansi_colors.len() {
                    theme.ansi_colors[idx as usize]
                } else {
                    theme.foreground
                }
            }
            CellColor::Rgb(r, g, b) => Color::rgb(r, g, b),
        };

        let mut bg = match cell.bg {
            CellColor::Default => theme.background,
            CellColor::Indexed(idx) => {
                if (idx as usize) < theme.ansi_colors.len() {
                    theme.ansi_colors[idx as usize]
                } else {
                    theme.background
                }
            }
            CellColor::Rgb(r, g, b) => Color::rgb(r, g, b),
        };

        if cell.flags.contains(CellFlags::INVERSE) {
            std::mem::swap(&mut fg, &mut bg);
        }

        (fg, bg)
    }
}
