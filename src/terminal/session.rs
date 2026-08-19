use std::path::PathBuf;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::terminal::ansi::AnsiParser;
use crate::terminal::grid::TerminalGrid;
use crate::terminal::pty::PtyBackend;

#[derive(Debug, Clone)]
pub struct InputState {
    pub text: String,
    pub cursor_index: usize,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_index: 0,
        }
    }
}

impl InputState {
    pub fn insert_str(&mut self, s: &str) {
        self.text.insert_str(self.cursor_index, s);
        self.cursor_index += s.len();
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor_index, c);
        self.cursor_index += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor_index > 0 {
            let prev = self.text[..self.cursor_index].chars().next_back();
            if let Some(c) = prev {
                let char_len = c.len_utf8();
                self.text.remove(self.cursor_index - char_len);
                self.cursor_index -= char_len;
            }
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_index = 0;
    }
}

pub struct TerminalSession {
    pub id: String,
    pub title: String,
    pub cwd: PathBuf,
    pub grid: TerminalGrid,
    pub parser: AnsiParser,
    pub pty: Option<PtyBackend>,
    pub output_rx: Receiver<Vec<u8>>,
    pub output_tx: Sender<Vec<u8>>,
    pub input_state: InputState,
    pub last_bell: bool,
}

impl TerminalSession {
    pub fn new(
        id: String,
        cols: usize,
        rows: usize,
        scrollback: usize,
        cwd: Option<PathBuf>,
        shell: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (output_tx, output_rx) = unbounded();
        let effective_cwd = cwd.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let pty = PtyBackend::spawn(
            shell,
            Some(effective_cwd.clone()),
            cols as u16,
            rows as u16,
            output_tx.clone(),
        )?;

        Ok(Self {
            id,
            title: "Terminal".to_string(),
            cwd: effective_cwd,
            grid: TerminalGrid::new(cols, rows, scrollback),
            parser: AnsiParser::new(),
            pty: Some(pty),
            output_rx,
            output_tx,
            input_state: InputState::default(),
            last_bell: false,
        })
    }

    pub fn create_headless(id: String, cols: usize, rows: usize, scrollback: usize) -> Self {
        let (output_tx, output_rx) = unbounded();
        Self {
            id,
            title: "Terminal".to_string(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            grid: TerminalGrid::new(cols, rows, scrollback),
            parser: AnsiParser::new(),
            pty: None,
            output_rx,
            output_tx,
            input_state: InputState::default(),
            last_bell: false,
        }
    }

    pub fn process_incoming(&mut self) -> bool {
        let mut updated = false;
        while let Ok(bytes) = self.output_rx.try_recv() {
            let (title, bell) = self.parser.process_bytes(&bytes, &mut self.grid);
            if let Some(t) = title {
                self.title = t;
            }
            if bell {
                self.last_bell = true;
            }
            updated = true;
        }
        updated
    }

    pub fn feed_bytes(&mut self, bytes: &[u8]) {
        let (title, bell) = self.parser.process_bytes(bytes, &mut self.grid);
        if let Some(t) = title {
            self.title = t;
        }
        if bell {
            self.last_bell = true;
        }
    }

    pub fn write_to_pty(&self, data: &[u8]) -> std::io::Result<()> {
        if let Some(pty) = &self.pty {
            pty.write_bytes(data)?;
        }
        Ok(())
    }

    pub fn resize(&mut self, cols: usize, rows: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.grid.resize(cols, rows);
        if let Some(pty) = &self.pty {
            pty.resize(cols as u16, rows as u16)?;
        }
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        self.pty.as_ref().map_or(true, |p| p.is_alive())
    }
}
