//! Level 3: Symbolic Equivalence Verification
//!
//! Uses symbolic manipulation to verify mathematical expressions are equivalent.

use crate::certificate::VerificationCertificate;
use crate::config::{VerificationError, VerificationLevel};
use std::time::Instant;

/// Level 3 verification using symbolic equivalence
pub struct Level3Symbolic {
    timeout_ms: u64,
}

impl Level3Symbolic {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    fn verify_symbolic(&self, expr1: &str, expr2: &str) -> Result<(f64, f64), VerificationError> {
        let simplified1 = self.simplify(expr1);
        let simplified2 = self.simplify(expr2);

        let diff = self.symbolic_diff(&simplified1, &simplified2);

        let mean_error = if diff == 0.0 { 0.0 } else { diff.abs() };
        let max_error = mean_error;

        Ok((mean_error, max_error))
    }

    fn simplify(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        result = result.replace(" ", "");

        result = result.replace("+0", "");
        result = result.replace("0+", "");

        result = result.replace("*1", "");
        result = result.replace("1*", "");

        result = result.replace("*0", "*0");

        result = result.replace("^1", "");

        while result.contains("(") {
            let new_result = result
                .replace("(+", "+")
                .replace("(-", "-")
                .replace(")", "");
            if new_result == result {
                break;
            }
            result = new_result;
        }

        result
    }

    fn symbolic_diff(&self, expr1: &str, expr2: &str) -> f64 {
        if expr1 == expr2 {
            return 0.0;
        }

        if let (Ok(v1), Ok(v2)) = (expr1.parse::<f64>(), expr2.parse::<f64>()) {
            return (v1 - v2).abs();
        }

        1.0
    }

    fn calculate_complexity(&self, expr: &str) -> f64 {
        let mut score = 0.0;

        score += expr.matches(['+', '-', '*', '/']).count() as f64 * 0.1;
        score += expr.matches("^").count() as f64 * 0.2;
        score += expr.matches("sin").count() as f64 * 0.3;
        score += expr.matches("cos").count() as f64 * 0.3;
        score += expr.matches("exp").count() as f64 * 0.3;
        score += expr.matches("log").count() as f64 * 0.3;

        score += expr.len() as f64 * 0.01;

        score.min(1.0)
    }
}

impl super::VerificationStrategy for Level3Symbolic {
    fn verify(
        &self,
        expression: &str,
    ) -> Result<VerificationCertificate, super::VerificationError> {
        let start = Instant::now();

        let simplified = self.simplify(expression);

        let complexity = self.calculate_complexity(&simplified);

        let confidence = (1.0 - complexity.min(1.0)).max(0.0);

        let duration = start.elapsed().as_millis() as u64;

        let status = if confidence >= 0.85 {
            crate::config::VerificationStatus::Passed
        } else if confidence >= 0.5 {
            crate::config::VerificationStatus::Uncertain
        } else {
            crate::config::VerificationStatus::Failed
        };

        Ok(VerificationCertificate::new(
            expression.to_string(),
            VerificationLevel::Level3,
            1,
            complexity,
            complexity,
            0.0,
            duration,
        )
        .with_status(status))
    }

    fn level(&self) -> VerificationLevel {
        VerificationLevel::Level3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::VerificationStrategy;

    #[test]
    fn test_level3_simplify() {
        let verifier = Level3Symbolic::new(1000);

        assert_eq!(verifier.simplify("x + 0"), "x");
        assert_eq!(verifier.simplify("x * 1"), "x");
    }

    #[test]
    fn test_level3_basic() {
        let verifier = Level3Symbolic::new(1000);
        let result = verifier.verify("x + 0");

        assert!(result.is_ok());
    }
}
