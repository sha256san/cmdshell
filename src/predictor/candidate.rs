use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CandidateSource {
    History,
    Command,
    Filesystem,
    Git,
    Project,
    Option,
    AI,
}

impl CandidateSource {
    pub fn badge(&self) -> &'static str {
        match self {
            CandidateSource::History => "󰋚 History",
            CandidateSource::Command => "󰆍 Cmd",
            CandidateSource::Filesystem => "󰉋 File",
            CandidateSource::Git => "󰊢 Git",
            CandidateSource::Project => "󰏗 Project",
            CandidateSource::Option => "󰘳 Flag",
            CandidateSource::AI => "󰚩 AI",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub text: String,
    pub display: String,
    pub description: Option<String>,
    pub source: CandidateSource,
    pub score: f32,
    pub prefix_len: usize,
}

impl Candidate {
    pub fn new(text: impl Into<String>, source: CandidateSource, score: f32) -> Self {
        let text = text.into();
        let display = text.clone();
        Self {
            text,
            display,
            description: None,
            source,
            score,
            prefix_len: 0,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_prefix_len(mut self, len: usize) -> Self {
        self.prefix_len = len;
        self
    }
}
