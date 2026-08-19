use std::sync::Arc;
use parking_lot::Mutex;
use crate::database::history::HistoryDb;
use crate::predictor::candidate::{Candidate, CandidateSource};
use crate::predictor::context::PredictionContext;
use crate::providers::CandidateProvider;

pub struct HistoryProvider {
    db: Arc<Mutex<HistoryDb>>,
}

impl HistoryProvider {
    pub fn new(db: Arc<Mutex<HistoryDb>>) -> Self {
        Self { db }
    }
}

impl CandidateProvider for HistoryProvider {
    fn name(&self) -> &'static str {
        "History"
    }

    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate> {
        let input = context.input_up_to_cursor().trim();
        if input.is_empty() {
            // Return recent commands when line is blank
            let db = self.db.lock();
            if let Ok(entries) = db.get_recent(10) {
                return entries
                    .into_iter()
                    .map(|entry| {
                        Candidate::new(entry.command.clone(), CandidateSource::History, 30.0)
                            .with_description(format!("Run {} time(s)", entry.execution_count))
                            .with_prefix_len(0)
                    })
                    .collect();
            }
            return Vec::new();
        }

        let db = self.db.lock();
        if let Ok(entries) = db.search_prefix(input, 15) {
            entries
                .into_iter()
                .map(|entry| {
                    let count_bonus = (entry.execution_count as f32).min(20.0);
                    Candidate::new(entry.command.clone(), CandidateSource::History, 60.0 + count_bonus)
                        .with_description(format!("Run {} time(s)", entry.execution_count))
                        .with_prefix_len(input.len())
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
