use serde::{Deserialize, Serialize};
use crate::app::state::AppState;
use crate::predictor::context::{GitContext, ProjectType};
use crate::ui::dialog::ConfirmDialogView;
use crate::ui::status_bar::StatusBarView;
use crate::ui::tab_bar::{TabBarView, TabItem};
use crate::ui::terminal_view::TerminalViewModel;
use crate::ui::suggestion::SuggestionPopupView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MainWindowModel {
    pub title: String,
    pub tab_bar: TabBarView,
    pub active_terminal: Option<TerminalViewModel>,
    pub status_bar: StatusBarView,
    pub confirm_dialog: Option<ConfirmDialogView>,
}

impl MainWindowModel {
    pub fn from_app_state(state: &AppState) -> Self {
        let tabs: Vec<TabItem> = state
            .sessions
            .iter()
            .enumerate()
            .map(|(i, s)| TabItem {
                id: s.id.clone(),
                title: s.title.clone(),
                is_active: i == state.active_session_index,
            })
            .collect();

        let tab_bar = TabBarView::new(tabs, state.active_session_index);

        let active_terminal = state.active_session().map(|session| {
            let ghost = state.active_prediction.as_ref().and_then(|p| p.ghost_text.clone());
            let suggestions = if let Some(pred) = &state.active_prediction {
                SuggestionPopupView::new(pred.candidates.clone(), pred.selected_index)
            } else {
                SuggestionPopupView::new(Vec::new(), None)
            };
            TerminalViewModel::from_session(session, &state.config.theme, ghost, suggestions)
        });

        let (cwd, project_type, branch) = if let Some(s) = state.active_session() {
            let proj = ProjectType::detect(&s.cwd).map(|p| p.name().to_string());
            let git = GitContext::detect(&s.cwd).map(|g| g.branch);
            (s.cwd.display().to_string(), proj, git)
        } else {
            ("~".to_string(), None, None)
        };

        let status_bar = StatusBarView::new(
            state.config.terminal.shell.as_deref().unwrap_or("bash"),
            cwd,
            project_type,
            branch,
            state.config.prediction.enable_ai,
        );

        let confirm_dialog = state
            .pending_dangerous_command
            .as_ref()
            .map(|(cmd, verdict)| ConfirmDialogView::from_verdict(cmd, verdict));

        Self {
            title: "PredictTerm".to_string(),
            tab_bar,
            active_terminal,
            status_bar,
            confirm_dialog,
        }
    }
}
