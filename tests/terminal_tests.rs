use predictterm::terminal::cell::{CellColor, CellFlags};
use predictterm::terminal::grid::{Position, Selection, TerminalGrid};

#[test]
fn test_grid_creation_and_initial_state() {
    let grid = TerminalGrid::new(80, 24, 1000);
    assert_eq!(grid.cols, 80);
    assert_eq!(grid.rows, 24);
    assert_eq!(grid.cursor_x, 0);
    assert_eq!(grid.cursor_y, 0);
    assert!(grid.cursor_visible);
    assert_eq!(grid.scrollback.len(), 0);
}

#[test]
fn test_grid_write_char_and_advance() {
    let mut grid = TerminalGrid::new(10, 5, 100);
    grid.write_char('A', CellColor::Default, CellColor::Default, CellFlags::empty());
    grid.write_char('B', CellColor::Default, CellColor::Default, CellFlags::empty());

    assert_eq!(grid.cursor_x, 2);
    assert_eq!(grid.cursor_y, 0);
    assert_eq!(grid.lines[0][0].c, 'A');
    assert_eq!(grid.lines[0][1].c, 'B');
    assert_eq!(grid.lines[0][2].c, ' ');
}

#[test]
fn test_grid_line_feed_and_scrollback() {
    let mut grid = TerminalGrid::new(10, 3, 5);
    
    // Write line 0
    grid.write_char('1', CellColor::Default, CellColor::Default, CellFlags::empty());
    grid.carriage_return();
    grid.line_feed();
    // Write line 1
    grid.write_char('2', CellColor::Default, CellColor::Default, CellFlags::empty());
    grid.carriage_return();
    grid.line_feed();
    // Write line 2 (bottom)
    grid.write_char('3', CellColor::Default, CellColor::Default, CellFlags::empty());
    grid.carriage_return();
    grid.line_feed(); // triggers scroll

    assert_eq!(grid.scrollback.len(), 1);
    assert_eq!(grid.scrollback[0][0].c, '1');
    assert_eq!(grid.lines[0][0].c, '2');
    assert_eq!(grid.lines[1][0].c, '3');
    assert_eq!(grid.lines[2][0].c, ' ');
}

#[test]
fn test_grid_carriage_return_and_backspace() {
    let mut grid = TerminalGrid::new(10, 5, 100);
    grid.write_char('H', CellColor::Default, CellColor::Default, CellFlags::empty());
    grid.write_char('i', CellColor::Default, CellColor::Default, CellFlags::empty());
    assert_eq!(grid.cursor_x, 2);

    grid.backspace();
    assert_eq!(grid.cursor_x, 1);

    grid.carriage_return();
    assert_eq!(grid.cursor_x, 0);
}

#[test]
fn test_grid_erase_in_line() {
    let mut grid = TerminalGrid::new(10, 5, 100);
    for c in "HelloWorld".chars() {
        grid.write_char(c, CellColor::Default, CellColor::Default, CellFlags::empty());
    }
    assert_eq!(grid.get_line_text(0), "HelloWorld");

    grid.move_cursor_to(5, 0);
    grid.erase_in_line(0); // erase cursor to end
    assert_eq!(grid.get_line_text(0), "Hello");
}

#[test]
fn test_grid_selection_and_copy() {
    let mut grid = TerminalGrid::new(10, 5, 100);
    for c in "ABCDE".chars() {
        grid.write_char(c, CellColor::Default, CellColor::Default, CellFlags::empty());
    }

    grid.selection = Some(Selection {
        start: Position { col: 1, row: 0 },
        end: Position { col: 3, row: 0 },
    });

    let selected = grid.get_selected_text().unwrap();
    assert_eq!(selected, "BCD");
}

#[test]
fn test_grid_resize() {
    let mut grid = TerminalGrid::new(10, 5, 100);
    grid.write_char('X', CellColor::Default, CellColor::Default, CellFlags::empty());
    grid.resize(20, 10);

    assert_eq!(grid.cols, 20);
    assert_eq!(grid.rows, 10);
    assert_eq!(grid.lines[0][0].c, 'X');
}
