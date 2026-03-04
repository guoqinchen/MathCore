//! Verification Mesh - Three-level verification system
//!
//! Coordinates verification across multiple levels with automatic downgrade on timeout.

use crate::certificate::VerificationCertificate;
use crate::config::{Confidence, VerificationError, VerificationLevel, VerificationStatus};
use crate::levels::{Level1Numerical, Level2Inverse, Level3Symbolic, VerificationStrategy};
use std::time::{Duration, Instant};

/// Verification Mesh Configuration
#[derive(Debug, Clone)]
pub struct MeshConfig {
    /// Maximum time to spend on verification
    pub timeout: Duration,
    /// Minimum confidence threshold
    pub min_confidence: Confidence,
    /// Whether to auto-downgrade on timeout
    pub auto_downgrade: bool,
    /// Samples for Level 1
    pub level1_samples: usize,
    /// Test points for Level 2
    pub level2_test_points: usize,
    /// Timeout for Level 3
    pub level3_timeout_ms: u64,
    /// Tolerance for numerical comparison
    pub tolerance: f64,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(100),
            min_confidence: Confidence::Medium,
            auto_downgrade: true,
            level1_samples: 100,
            level2_test_points: 20,
            level3_timeout_ms: 50,
            tolerance: 1e-10,
        }
    }
}

/// Main verification mesh
pub struct VerificationMesh {
    config: MeshConfig,
    level1: Level1Numerical,
    level2: Level2Inverse,
    level3: Level3Symbolic,
}

impl VerificationMesh {
    pub fn new(config: MeshConfig) -> Self {
        Self {
            level1: Level1Numerical::new(config.level1_samples, config.tolerance),
            level2: Level2Inverse::new(config.tolerance, config.level2_test_points),
            level3: Level3Symbolic::new(config.level3_timeout_ms),
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(MeshConfig::default())
    }

    /// Verify at specific level
    pub fn verify_at(
        &self,
        expression: &str,
        level: VerificationLevel,
    ) -> Result<VerificationCertificate, VerificationError> {
        match level {
            VerificationLevel::Level1 => self.level1.verify(expression),
            VerificationLevel::Level2 => self.level2.verify(expression),
            VerificationLevel::Level3 => self.level3.verify(expression),
        }
    }

    /// Verify with automatic level selection and downgrade
    pub fn verify(&self, expression: &str) -> Result<VerificationCertificate, VerificationError> {
        let start = Instant::now();

        // Try Level 3 first (most rigorous)
        if start.elapsed() < self.config.timeout {
            match self.level3.verify(expression) {
                Ok(cert) if cert.confidence >= self.config.min_confidence.score() => {
                    return Ok(cert);
                }
                Ok(cert) => {
                    // Level 3 didn't meet confidence threshold
                    if !self.config.auto_downgrade {
                        return Ok(cert);
                    }
                    // Fall through to lower levels
                }
                Err(e) => {
                    // Level 3 failed, try lower levels
                    if !self.config.auto_downgrade {
                        return Err(e);
                    }
                }
            }
        }

        // Try Level 2
        if start.elapsed() < self.config.timeout {
            match self.level2.verify(expression) {
                Ok(cert) if cert.confidence >= self.config.min_confidence.score() => {
                    return Ok(cert);
                }
                Ok(cert) => {
                    if !self.config.auto_downgrade {
                        return Ok(cert);
                    }
                }
                Err(_) => {
                    if !self.config.auto_downgrade {
                        return Err(VerificationError::InverseNotSupported(
                            expression.to_string(),
                        ));
                    }
                }
            }
        }

        // Fall back to Level 1
        if start.elapsed() >= self.config.timeout {
            return Err(VerificationError::Timeout);
        }

        let cert = self.level1.verify(expression)?;

        // Add timeout metadata if we ran out of time
        if start.elapsed() >= self.config.timeout {
            return Ok(cert.with_metadata("timeout", "true"));
        }

        Ok(cert)
    }

    /// Verify with specific target level, trying lower levels on timeout
    pub fn verify_with_fallback(
        &self,
        expression: &str,
        target_level: VerificationLevel,
    ) -> Result<VerificationCertificate, VerificationError> {
        let start = Instant::now();

        let levels = match target_level {
            VerificationLevel::Level1 => vec![VerificationLevel::Level1],
            VerificationLevel::Level2 => vec![VerificationLevel::Level2, VerificationLevel::Level1],
            VerificationLevel::Level3 => vec![
                VerificationLevel::Level3,
                VerificationLevel::Level2,
                VerificationLevel::Level1,
            ],
        };

        for level in levels {
            if start.elapsed() >= self.config.timeout {
                return Err(VerificationError::Timeout);
            }

            match self.verify_at(expression, level) {
                Ok(cert) if cert.confidence >= self.config.min_confidence.score() => {
                    return Ok(cert);
                }
                Ok(cert) => {
                    // Continue to lower level
                    continue;
                }
                Err(e) => {
                    // Try lower level
                    continue;
                }
            }
        }

        // All levels failed or didn't meet confidence
        self.level1.verify(expression)
    }

    /// Get mesh statistics
    pub fn config(&self) -> &MeshConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_default() {
        let mesh = VerificationMesh::with_default_config();
        let result = mesh.verify("x^2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mesh_verify_at_level1() {
        let mesh = VerificationMesh::with_default_config();
        let result = mesh.verify_at("x^2", VerificationLevel::Level1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mesh_verify_at_level2() {
        let mesh = VerificationMesh::with_default_config();
        let result = mesh.verify_at("x^2", VerificationLevel::Level2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mesh_verify_at_level3() {
        let mesh = VerificationMesh::with_default_config();
        let result = mesh.verify_at("x + 0", VerificationLevel::Level3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mesh_fallback() {
        let mesh = VerificationMesh::with_default_config();
        let result = mesh.verify_with_fallback("x^2", VerificationLevel::Level3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mesh_config() {
        let config = MeshConfig {
            timeout: Duration::from_millis(50),
            min_confidence: Confidence::High,
            auto_downgrade: true,
            level1_samples: 20,
            level2_test_points: 5,
            level3_timeout_ms: 10,
            tolerance: 1e-5,
        };

        let mesh = VerificationMesh::new(config);
        let result = mesh.verify("x^2");
        assert!(result.is_ok());
    }
}
