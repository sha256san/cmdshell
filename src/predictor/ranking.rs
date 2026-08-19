use std::collections::HashMap;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use crate::predictor::candidate::{Candidate, CandidateSource};

pub struct RankingEngine {
    fuzzy_matcher: SkimMatcherV2,
}

impl Default for RankingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RankingEngine {
    pub fn new() -> Self {
        Self {
            fuzzy_matcher: SkimMatcherV2::default(),
        }
    }

    pub fn rank_and_deduplicate(
        &self,
        full_query: &str,
        token_query: &str,
        raw_candidates: Vec<Candidate>,
        max_results: usize,
    ) -> Vec<Candidate> {
        let full_query_trimmed = full_query.trim();
        let mut map: HashMap<String, Candidate> = HashMap::new();

        for mut candidate in raw_candidates {
            let text_lower = candidate.text.to_lowercase();

            // Determine if we should match against token or full query
            let is_full_line_candidate = matches!(
                candidate.source,
                CandidateSource::History | CandidateSource::AI | CandidateSource::Project
            );

            let query = if is_full_line_candidate {
                full_query_trimmed
            } else {
                token_query
            };

            let query_lower = query.to_lowercase();
            let mut final_score = candidate.score;

            if !query_lower.is_empty() {
                if text_lower.starts_with(&query_lower) {
                    final_score += 100.0;
                    if text_lower == query_lower {
                        final_score += 30.0;
                    }
                } else if let Some(fuzzy_score) = self.fuzzy_matcher.fuzzy_match(&candidate.text, &query_lower) {
                    final_score += (fuzzy_score as f32) * 0.5;
                } else if let Some(pos) = text_lower.find(&query_lower) {
                    final_score += (50.0 - (pos as f32)).max(10.0);
                } else {
                    // Non-matching candidates for non-empty query get penalised
                    final_score -= 50.0;
                }
            }

            // Source weighting
            match candidate.source {
                CandidateSource::History => final_score += 30.0,
                CandidateSource::Git => final_score += 25.0,
                CandidateSource::Project => final_score += 20.0,
                CandidateSource::Filesystem => final_score += 15.0,
                CandidateSource::Option => final_score += 15.0,
                CandidateSource::Command => final_score += 10.0,
                CandidateSource::AI => final_score += 5.0,
            }

            // Length penalty
            final_score -= (candidate.text.len() as f32) * 0.05;

            candidate.score = final_score;

            if let Some(existing) = map.get_mut(&candidate.text) {
                if candidate.score > existing.score {
                    *existing = candidate;
                }
            } else {
                map.insert(candidate.text.clone(), candidate);
            }
        }

        let mut results: Vec<Candidate> = map.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);
        results
    }
}
