use std::sync::Arc;
use parking_lot::Mutex;
use crate::config::settings::Config;
use crate::database::history::HistoryDb;
use crate::predictor::candidate::Candidate;
use crate::predictor::context::PredictionContext;
use crate::predictor::engine::{PredictionEngine, PredictionResult};
use crate::safety::dangerous::{DangerousDetector, DangerousVerdict};
use crate::safety::sanitizer::SecretSanitizer;
use crate::terminal::session::TerminalSession;

pub struct AppState {
    pub config: Config,
    pub sessions: Vec<TerminalSession>,
    pub active_session_index: usize,
    pub history_db: Arc<Mutex<HistoryDb>>,
    pub prediction_engine: Arc<PredictionEngine>,
    pub dangerous_detector: DangerousDetector,
    pub secret_sanitizer: SecretSanitizer,
    pub active_prediction: Option<PredictionResult>,
    pub pending_dangerous_command: Option<(String, DangerousVerdict)>,
}

impl AppState {
    pub fn new(config: Config, history_db: HistoryDb) -> Self {
        let history_db = Arc::new(Mutex::new(history_db));
        let prediction_engine = Arc::new(PredictionEngine::new(Arc::clone(&history_db)));
        let dangerous_detector = DangerousDetector::new(&config.safety.custom_dangerous_patterns);
        let secret_sanitizer = SecretSanitizer::new();

        Self {
            config,
            sessions: Vec::new(),
            active_session_index: 0,
            history_db,
            prediction_engine,
            dangerous_detector,
            secret_sanitizer,
            active_prediction: None,
            pending_dangerous_command: None,
        }
    }

    pub fn active_session(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active_session_index)
    }

    pub fn active_session_mut(&mut self) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(self.active_session_index)
    }

    pub fn add_session(&mut self, session: TerminalSession) {
        self.sessions.push(session);
        self.active_session_index = self.sessions.len().saturating_sub(1);
    }

    pub fn close_tab(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.sessions.remove(index);
            if self.active_session_index >= self.sessions.len() {
                self.active_session_index = self.sessions.len().saturating_sub(1);
            }
        }
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.active_session_index = index;
            self.active_prediction = None;
        }
    }

    pub fn on_input_changed(&mut self, text: String, cursor_pos: usize) {
        let (cwd, enabled, max_suggestions, shell) = if let Some(session) = self.sessions.get_mut(self.active_session_index) {
            session.input_state.text = text.clone();
            session.input_state.cursor_index = cursor_pos;
            (
                session.cwd.clone(),
                self.config.prediction.enabled,
                self.config.prediction.max_suggestions,
                self.config.terminal.shell.clone().unwrap_or_else(|| "bash".to_string()),
            )
        } else {
            return;
        };

        if enabled {
            let context = PredictionContext::new(text, cursor_pos, cwd, shell);
            let result = self.prediction_engine.predict(&context, max_suggestions);
            self.active_prediction = Some(result);
        } else {
            self.active_prediction = None;
        }
    }

    pub fn select_next_candidate(&mut self) {
        if let Some(pred) = &mut self.active_prediction {
            if !pred.candidates.is_empty() {
                let curr = pred.selected_index.unwrap_or(0);
                let next = (curr + 1) % pred.candidates.len();
                pred.selected_index = Some(next);
            }
        }
    }

    pub fn select_prev_candidate(&mut self) {
        if let Some(pred) = &mut self.active_prediction {
            if !pred.candidates.is_empty() {
                let curr = pred.selected_index.unwrap_or(0);
                let prev = if curr == 0 { pred.candidates.len() - 1 } else { curr - 1 };
                pred.selected_index = Some(prev);
            }
        }
    }

    pub fn accept_selected_candidate(&mut self) -> Option<String> {
        let (cand, text_before, cursor_before) = {
            let pred = self.active_prediction.as_ref()?;
            let idx = pred.selected_index?;
            let cand = pred.candidates.get(idx)?.clone();
            let sess = self.active_session()?;
            (cand, sess.input_state.text.clone(), sess.input_state.cursor_index)
        };

        let new_text = Self::apply_candidate_to_input(&text_before, cursor_before, &cand);
        if let Some(sess) = self.active_session_mut() {
            sess.input_state.text = new_text.clone();
            sess.input_state.cursor_index = new_text.len();
        }
        self.active_prediction = None;
        Some(new_text)
    }

    pub fn accept_ghost_text(&mut self) -> Option<String> {
        let ghost = self.active_prediction.as_ref()?.ghost_text.clone()?;
        let sess = self.active_session_mut()?;
        sess.input_state.text.push_str(&ghost);
        sess.input_state.cursor_index = sess.input_state.text.len();
        let new_text = sess.input_state.text.clone();
        self.active_prediction = None;
        Some(new_text)
    }

    pub fn apply_candidate_to_input(input: &str, cursor_pos: usize, candidate: &Candidate) -> String {
        let prefix = &input[..cursor_pos.min(input.len())];
        let suffix = &input[cursor_pos.min(input.len())..];

        if candidate.prefix_len > 0 && prefix.len() >= candidate.prefix_len {
            let retained = &prefix[..prefix.len() - candidate.prefix_len];
            format!("{}{}{}", retained, candidate.text, suffix)
        } else if prefix.ends_with(' ') || prefix.is_empty() {
            format!("{}{}{}", prefix, candidate.text, suffix)
        } else if let Some(last_space) = prefix.rfind(' ') {
            format!("{} {}{}", &prefix[..last_space], candidate.text, suffix)
        } else {
            format!("{}{}", candidate.text, suffix)
        }
    }

    pub fn execute_command(&mut self, command: String) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            if let Some(sess) = self.active_session_mut() {
                sess.write_to_pty(b"\n")?;
                sess.input_state.clear();
            }
            return Ok(true);
        }

        // Check safety
        if self.config.safety.enable_dangerous_confirmation {
            let verdict = self.dangerous_detector.inspect(trimmed);
            if verdict.is_dangerous {
                self.pending_dangerous_command = Some((trimmed.to_string(), verdict));
                return Ok(false); // Intercepted for confirmation
            }
        }

        self.execute_confirmed_command(trimmed.to_string())
    }

    pub fn execute_confirmed_command(&mut self, command: String) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = command.trim().to_string();
        self.pending_dangerous_command = None;

        // Mask secrets before storing in SQLite
        let sanitized = if self.config.safety.mask_secrets_in_history {
            self.secret_sanitizer.sanitize(&trimmed)
        } else {
            trimmed.clone()
        };

        // Record to DB
        let cwd_str = self.active_session().map(|s| s.cwd.to_string_lossy().to_string());
        {
            let mut db = self.history_db.lock();
            let _ = db.record_command(&sanitized, cwd_str.as_deref(), Some(0));
        }

        // Send to PTY
        if let Some(sess) = self.active_session_mut() {
            let mut bytes = trimmed.as_bytes().to_vec();
            bytes.push(b'\n');
            sess.write_to_pty(&bytes)?;
            sess.input_state.clear();
        }

        self.active_prediction = None;
        Ok(true)
    }

    pub fn cancel_pending_command(&mut self) {
        self.pending_dangerous_command = None;
        if let Some(sess) = self.active_session_mut() {
            sess.input_state.clear();
        }
    }
}
