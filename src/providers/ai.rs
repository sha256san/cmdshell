use crate::predictor::candidate::{Candidate, CandidateSource};
use crate::predictor::context::PredictionContext;
use crate::providers::CandidateProvider;

pub struct AiProvider {
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
}

impl Default for AiProvider {
    fn default() -> Self {
        Self::new(false, None, None)
    }
}

impl AiProvider {
    pub fn new(enabled: bool, endpoint: Option<String>, model: Option<String>) -> Self {
        Self {
            enabled,
            endpoint,
            model,
        }
    }
}

impl CandidateProvider for AiProvider {
    fn name(&self) -> &'static str {
        "AI"
    }

    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate> {
        if !self.enabled {
            return Vec::new();
        }

        let input = context.input_up_to_cursor().trim();
        // Natural language query detection (e.g., "# find large files" or "?? extract tar")
        if let Some(nl_query) = input.strip_prefix('#').or_else(|| input.strip_prefix("??")) {
            let q = nl_query.trim().to_lowercase();
            let mut candidates = Vec::new();

            if q.contains("find") && q.contains("large") {
                candidates.push(
                    Candidate::new("find . -type f -size +100M", CandidateSource::AI, 90.0)
                        .with_description("AI: Find files larger than 100MB")
                        .with_prefix_len(input.len()),
                );
            } else if q.contains("port") || q.contains("listen") {
                candidates.push(
                    Candidate::new("lsof -i -P -n | grep LISTEN", CandidateSource::AI, 90.0)
                        .with_description("AI: List all listening ports and services")
                        .with_prefix_len(input.len()),
                );
            } else if q.contains("disk") || q.contains("space") {
                candidates.push(
                    Candidate::new("df -h", CandidateSource::AI, 90.0)
                        .with_description("AI: Show free disk space in human units")
                        .with_prefix_len(input.len()),
                );
            }

            return candidates;
        }

        Vec::new()
    }
}
