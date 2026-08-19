use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use crate::database::history::HistoryDb;
use crate::predictor::candidate::Candidate;
use crate::predictor::context::PredictionContext;
use crate::predictor::ranking::RankingEngine;
use crate::providers::ai::AiProvider;
use crate::providers::command::CommandProvider;
use crate::providers::filesystem::FilesystemProvider;
use crate::providers::git::GitProvider;
use crate::providers::history::HistoryProvider;
use crate::providers::option::OptionProvider;
use crate::providers::project::ProjectProvider;
use crate::providers::CandidateProvider;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionResult {
    pub candidates: Vec<Candidate>,
    pub ghost_text: Option<String>,
    pub selected_index: Option<usize>,
}

pub struct PredictionEngine {
    providers: Vec<Box<dyn CandidateProvider>>,
    ranking_engine: RankingEngine,
}

impl PredictionEngine {
    pub fn new(history_db: Arc<Mutex<HistoryDb>>) -> Self {
        let providers: Vec<Box<dyn CandidateProvider>> = vec![
            Box::new(HistoryProvider::new(history_db)),
            Box::new(GitProvider::new()),
            Box::new(ProjectProvider::new()),
            Box::new(OptionProvider::new()),
            Box::new(FilesystemProvider::new()),
            Box::new(CommandProvider::new()),
            Box::new(AiProvider::default()),
        ];

        Self {
            providers,
            ranking_engine: RankingEngine::new(),
        }
    }

    pub fn predict(&self, context: &PredictionContext, max_suggestions: usize) -> PredictionResult {
        let mut raw_candidates = Vec::new();

        for provider in &self.providers {
            let mut results = provider.suggest(context);
            raw_candidates.append(&mut results);
        }

        let full_query = context.input_up_to_cursor();
        let token_query = context.current_token();

        let ranked = self.ranking_engine.rank_and_deduplicate(full_query, token_query, raw_candidates, max_suggestions);
        let ghost_text = self.compute_ghost_text(context, ranked.first());

        let selected_index = if ranked.is_empty() { None } else { Some(0) };

        PredictionResult {
            candidates: ranked,
            ghost_text,
            selected_index,
        }
    }

    pub fn compute_ghost_text(&self, context: &PredictionContext, top_candidate: Option<&Candidate>) -> Option<String> {
        let candidate = top_candidate?;
        let input = context.input_up_to_cursor();

        if input.is_empty() {
            return None;
        }

        // Check if candidate matches full input
        if candidate.text.starts_with(input) && candidate.text.len() > input.len() {
            return Some(candidate.text[input.len()..].to_string());
        }

        // Check if candidate matches current token
        let token = context.current_token();
        if !token.is_empty() && candidate.text.starts_with(token) && candidate.text.len() > token.len() {
            return Some(candidate.text[token.len()..].to_string());
        }

        None
    }
}
