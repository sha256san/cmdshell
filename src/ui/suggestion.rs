use serde::{Deserialize, Serialize};
use crate::predictor::candidate::Candidate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestionPopupView {
    pub candidates: Vec<Candidate>,
    pub selected_index: Option<usize>,
    pub visible: bool,
}

impl SuggestionPopupView {
    pub fn new(candidates: Vec<Candidate>, selected_index: Option<usize>) -> Self {
        let visible = !candidates.is_empty();
        Self {
            candidates,
            selected_index,
            visible,
        }
    }

    pub fn selected_candidate(&self) -> Option<&Candidate> {
        let idx = self.selected_index?;
        self.candidates.get(idx)
    }
}
