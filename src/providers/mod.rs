pub mod ai;
pub mod command;
pub mod filesystem;
pub mod git;
pub mod history;
pub mod option;
pub mod project;

use crate::predictor::candidate::Candidate;
use crate::predictor::context::PredictionContext;

pub trait CandidateProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate>;
}
