use crate::config::{Confidence, VerificationLevel, VerificationStatus};
use serde::{Deserialize, Serialize};

/// Verification certificate containing proof of verification results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCertificate {
    /// Unique identifier for this verification run
    pub id: String,
    /// The expression that was verified
    pub expression: String,
    /// Level at which verification was performed
    pub level: VerificationLevel,
    /// Verification result
    pub status: VerificationStatus,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Confidence level enum
    pub confidence_level: Confidence,
    /// Number of samples/tests performed
    pub samples: usize,
    /// Maximum error observed
    pub max_error: f64,
    /// Mean error across all samples
    pub mean_error: f64,
    /// Standard deviation of errors
    pub std_error: f64,
    /// Timestamp of verification (Unix epoch)
    pub timestamp: i64,
    /// Time taken for verification in milliseconds
    pub duration_ms: u64,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl VerificationCertificate {
    pub fn new(
        expression: String,
        level: VerificationLevel,
        samples: usize,
        max_error: f64,
        mean_error: f64,
        std_error: f64,
        duration_ms: u64,
    ) -> Self {
        let confidence = if samples > 0 {
            let base_confidence = 1.0 - mean_error.min(1.0);
            let sample_bonus = (samples as f64 / 100.0).min(0.1);
            (base_confidence + sample_bonus).min(1.0)
        } else {
            0.0
        };

        let confidence_level = Confidence::from_score(confidence);

        Self {
            id: uuid_v4(),
            expression,
            level,
            status: if confidence >= 0.85 {
                VerificationStatus::Passed
            } else if confidence >= 0.5 {
                VerificationStatus::Uncertain
            } else {
                VerificationStatus::Failed
            },
            confidence,
            confidence_level,
            samples,
            max_error,
            mean_error,
            std_error,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            duration_ms,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_status(mut self, status: VerificationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn is_verified(&self) -> bool {
        matches!(self.status, VerificationStatus::Passed)
    }

    pub fn summary(&self) -> String {
        format!(
            "VerificationCertificate {{\n  id: {},\n  expression: {},\n  level: {},\n  status: {:?},\n  confidence: {:.2}%,\n  samples: {},\n  max_error: {:.2e},\n  mean_error: {:.2e},\n  duration: {}ms\n}}",
            &self.id[..8],
            truncate(&self.expression, 40),
            self.level,
            self.status,
            self.confidence * 100.0,
            self.samples,
            self.max_error,
            self.mean_error,
            self.duration_ms
        )
    }
}

fn uuid_v4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        (bytes[6] & 0x0f) | 0x40, bytes[7],
        (bytes[8] & 0x3f) | 0x80, bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_creation() {
        let cert = VerificationCertificate::new(
            "x^2 + 1".to_string(),
            VerificationLevel::Level1,
            100,
            1e-10,
            1e-12,
            1e-13,
            5,
        );

        assert_eq!(cert.samples, 100);
        assert!(cert.is_verified());
        assert!(matches!(cert.confidence_level, Confidence::VeryHigh));
    }

    #[test]
    fn test_confidence_from_score() {
        assert_eq!(Confidence::from_score(0.96), Confidence::VeryHigh);
        assert_eq!(Confidence::from_score(0.90), Confidence::High);
        assert_eq!(Confidence::from_score(0.75), Confidence::Medium);
        assert_eq!(Confidence::from_score(0.50), Confidence::Low);
    }
}
