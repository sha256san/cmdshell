use serde::{Deserialize, Serialize};
use crate::config::theme::Theme;
use crate::terminal::session::TerminalSession;
use crate::ui::ghost_text::GhostTextView;
use crate::ui::suggestion::SuggestionPopupView;
use crate::ui::terminal_grid::TerminalGridRenderData;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalViewModel {
    pub session_id: String,
    pub title: String,
    pub grid_data: TerminalGridRenderData,
    pub input_text: String,
    pub input_cursor: usize,
    pub ghost_text: GhostTextView,
    pub suggestion_popup: SuggestionPopupView,
}

impl TerminalViewModel {
    pub fn from_session(
        session: &TerminalSession,
        theme: &Theme,
        ghost_text: Option<String>,
        suggestions: SuggestionPopupView,
    ) -> Self {
        let grid_data = TerminalGridRenderData::from_grid(&session.grid, theme);
        Self {
            session_id: session.id.clone(),
            title: session.title.clone(),
            grid_data,
            input_text: session.input_state.text.clone(),
            input_cursor: session.input_state.cursor_index,
            ghost_text: GhostTextView::new(ghost_text),
            suggestion_popup: suggestions,
        }
    }
}
