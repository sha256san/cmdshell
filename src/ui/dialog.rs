use serde::{Deserialize, Serialize};
use crate::safety::dangerous::{DangerousVerdict, RiskLevel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmDialogView {
    pub title: String,
    pub command: String,
    pub explanation: String,
    pub risk_level: RiskLevel,
    pub visible: bool,
}

impl ConfirmDialogView {
    pub fn from_verdict(command: &str, verdict: &DangerousVerdict) -> Self {
        let title = match verdict.risk_level {
            RiskLevel::Critical => "🚨 Critical Destructive Command Intercepted",
            RiskLevel::High => "⚠️ High Risk Command Warning",
            RiskLevel::Medium => "⚡ Destructive Command Confirmation",
            RiskLevel::Low => "Notice",
        };

        Self {
            title: title.to_string(),
            command: command.to_string(),
            explanation: verdict.explanation.clone(),
            risk_level: verdict.risk_level,
            visible: true,
        }
    }
}
