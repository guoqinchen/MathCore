use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationLevel {
    Level1,
    Level2,
    Level3,
}

impl std::fmt::Display for VerificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationLevel::Level1 => write!(f, "Level1 (Numerical)"),
            VerificationLevel::Level2 => write!(f, "Level2 (Inverse)"),
            VerificationLevel::Level3 => write!(f, "Level3 (Symbolic)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Confidence {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.95 {
            Confidence::VeryHigh
        } else if score >= 0.85 {
            Confidence::High
        } else if score >= 0.70 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            Confidence::VeryHigh => 0.95,
            Confidence::High => 0.85,
            Confidence::Medium => 0.70,
            Confidence::Low => 0.0,
        }
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::VeryHigh => write!(f, "VeryHigh (≥95%)"),
            Confidence::High => write!(f, "High (≥85%)"),
            Confidence::Medium => write!(f, "Medium (≥70%)"),
            Confidence::Low => write!(f, "Low (<70%)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Passed,
    Failed,
    Uncertain,
    Timeout,
    Skipped,
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("Expression evaluation failed: {0}")]
    EvaluationFailed(String),
    #[error("Inverse operation not supported: {0}")]
    InverseNotSupported(String),
    #[error("Symbolic verification failed: {0}")]
    SymbolicFailed(String),
    #[error("Timeout during verification")]
    Timeout,
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
