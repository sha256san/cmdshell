use crate::predictor::candidate::Candidate;
use crate::predictor::engine::PredictionResult;
use crate::safety::dangerous::DangerousVerdict;

#[derive(Debug, Clone)]
pub enum AppEvent {
    TerminalOutput { session_id: String, bytes: Vec<u8> },
    InputChanged { session_id: String, text: String, cursor: usize },
    PredictionUpdated { session_id: String, result: PredictionResult },
    ExecuteCommand { session_id: String, command: String },
    DangerousCommandDetected { session_id: String, command: String, verdict: DangerousVerdict },
    ConfirmExecution { session_id: String, command: String },
    CancelExecution { session_id: String },
    SelectCandidate { session_id: String, index: usize },
    AcceptCandidate { session_id: String, candidate: Candidate },
    NewTab,
    CloseTab { index: usize },
    SwitchTab { index: usize },
    Resize { cols: usize, rows: usize },
}
