use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use predictterm::app::state::AppState;
use predictterm::database::history::HistoryDb;
use predictterm::predictor::candidate::{Candidate, CandidateSource};
use predictterm::predictor::context::PredictionContext;
use predictterm::predictor::engine::PredictionEngine;

#[test]
fn test_command_provider_and_prefix_matching() {
    let history_db = Arc::new(Mutex::new(HistoryDb::open_in_memory().unwrap()));
    let engine = PredictionEngine::new(history_db);

    let ctx = PredictionContext::new("ca", 2, PathBuf::from("/tmp"), "bash");
    let result = engine.predict(&ctx, 10);

    assert!(!result.candidates.is_empty());
    let contains_cargo_or_cat = result.candidates.iter().any(|c| c.text == "cargo" || c.text == "cat");
    assert!(contains_cargo_or_cat);
}

#[test]
fn test_git_provider_suggestions() {
    let history_db = Arc::new(Mutex::new(HistoryDb::open_in_memory().unwrap()));
    let engine = PredictionEngine::new(history_db);

    // Typing `git `
    let ctx = PredictionContext::new("git ", 4, PathBuf::from("/tmp"), "bash");
    let result = engine.predict(&ctx, 10);

    let contains_status = result.candidates.iter().any(|c| c.text == "status");
    let contains_commit = result.candidates.iter().any(|c| c.text == "commit");
    assert!(contains_status);
    assert!(contains_commit);
}

#[test]
fn test_history_provider_recording_and_suggestions() {
    let db = HistoryDb::open_in_memory().unwrap();
    let history_db = Arc::new(Mutex::new(db));
    
    {
        let mut guard = history_db.lock();
        guard.record_command("docker ps -a", Some("/tmp"), Some(0)).unwrap();
        guard.record_command("docker ps -a", Some("/tmp"), Some(0)).unwrap();
        guard.record_command("docker build -t test .", Some("/tmp"), Some(0)).unwrap();
    }

    let engine = PredictionEngine::new(Arc::clone(&history_db));
    let ctx = PredictionContext::new("docker", 6, PathBuf::from("/tmp"), "bash");
    let result = engine.predict(&ctx, 5);

    assert!(!result.candidates.is_empty());
    assert_eq!(result.candidates[0].text, "docker ps -a");
    assert_eq!(result.candidates[0].source, CandidateSource::History);
}

#[test]
fn test_ghost_text_generation() {
    let history_db = Arc::new(Mutex::new(HistoryDb::open_in_memory().unwrap()));
    let engine = PredictionEngine::new(history_db);

    let ctx = PredictionContext::new("git st", 6, PathBuf::from("/tmp"), "bash");
    let candidate = Candidate::new("git status", CandidateSource::Git, 90.0);
    let ghost = engine.compute_ghost_text(&ctx, Some(&candidate));

    assert_eq!(ghost, Some("atus".to_string()));
}

#[test]
fn test_apply_candidate_replacement() {
    let cand = Candidate::new("status", CandidateSource::Git, 90.0).with_prefix_len(2);
    let updated = AppState::apply_candidate_to_input("git st", 6, &cand);
    assert_eq!(updated, "git status");
}
