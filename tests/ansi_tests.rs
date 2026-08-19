use predictterm::terminal::ansi::AnsiParser;
use predictterm::terminal::cell::{CellColor, CellFlags};
use predictterm::terminal::grid::TerminalGrid;

#[test]
fn test_ansi_basic_text() {
    let mut grid = TerminalGrid::new(20, 5, 100);
    let mut parser = AnsiParser::new();

    parser.process_bytes(b"Hello, PredictTerm!\r\n", &mut grid);

    assert_eq!(grid.get_line_text(0), "Hello, PredictTerm!");
    assert_eq!(grid.cursor_x, 0);
    assert_eq!(grid.cursor_y, 1);
}

#[test]
fn test_ansi_colors_and_bold() {
    let mut grid = TerminalGrid::new(20, 5, 100);
    let mut parser = AnsiParser::new();

    // Red text (ESC[31m), Bold (ESC[1m), Reset (ESC[0m)
    parser.process_bytes(b"\x1b[31;1mRedBold\x1b[0mNormal", &mut grid);

    assert_eq!(grid.lines[0][0].c, 'R');
    assert_eq!(grid.lines[0][0].fg, CellColor::Indexed(1)); // ANSI Red
    assert!(grid.lines[0][0].flags.contains(CellFlags::BOLD));

    assert_eq!(grid.lines[0][7].c, 'N');
    assert_eq!(grid.lines[0][7].fg, CellColor::Default);
    assert!(!grid.lines[0][7].flags.contains(CellFlags::BOLD));
}

#[test]
fn test_ansi_truecolor_rgb() {
    let mut grid = TerminalGrid::new(20, 5, 100);
    let mut parser = AnsiParser::new();

    // 24-bit TrueColor RGB: ESC[38;2;120;180;240m
    parser.process_bytes(b"\x1b[38;2;120;180;240mRGB\x1b[0m", &mut grid);

    assert_eq!(grid.lines[0][0].c, 'R');
    assert_eq!(grid.lines[0][0].fg, CellColor::Rgb(120, 180, 240));
}

#[test]
fn test_ansi_cursor_movement() {
    let mut grid = TerminalGrid::new(20, 10, 100);
    let mut parser = AnsiParser::new();

    // Move to row 3, col 5 (1-based -> index row 2, col 4): ESC[3;5H
    parser.process_bytes(b"\x1b[3;5HX", &mut grid);

    assert_eq!(grid.lines[2][4].c, 'X');
    assert_eq!(grid.cursor_x, 5);
    assert_eq!(grid.cursor_y, 2);
}

#[test]
fn test_ansi_osc_window_title() {
    let mut grid = TerminalGrid::new(20, 5, 100);
    let mut parser = AnsiParser::new();

    // Set title via OSC 0;My Title\x07
    let (title, _) = parser.process_bytes(b"\x1b]0;My Title\x07", &mut grid);

    assert_eq!(title, Some("My Title".to_string()));
}
