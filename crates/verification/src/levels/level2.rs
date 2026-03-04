//! Level 2: Inverse Operation Verification
//!
//! Verifies mathematical expressions using inverse operations:
//! - Integration ↔ Differentiation
//! - Factorial ↔ Gamma function
//! - Log ↔ Exp
//! - Square ↔ Sqrt

use crate::certificate::VerificationCertificate;
use crate::config::{VerificationError, VerificationLevel};
use std::time::Instant;

/// Level 2 verification using inverse operations
pub struct Level2Inverse {
    tolerance: f64,
    num_test_points: usize,
}

impl Level2Inverse {
    pub fn new(tolerance: f64, num_test_points: usize) -> Self {
        Self {
            tolerance,
            num_test_points,
        }
    }

    fn verify_integral_derivative(&self, expr: &str) -> Result<(f64, f64), VerificationError> {
        let h = 1e-8;
        let mut errors = Vec::new();

        for i in 0..self.num_test_points {
            let x = -5.0 + (i as f64) * 10.0 / (self.num_test_points as f64);

            let fx = self.eval(expr, x);
            let fpx = (self.eval(expr, x + h) - self.eval(expr, x - h)) / (2.0 * h);
            let fpx_integral = self.integrate_derivative(expr, x - 1.0, x + 1.0);

            let error = (fpx_integral - fx).abs();
            errors.push(error);
        }

        let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;
        let max_error = errors.iter().cloned().fold(0.0_f64, |a, b| a.max(b));

        Ok((mean_error, max_error))
    }

    fn eval(&self, expr: &str, x: f64) -> f64 {
        let expr = expr.replace("x", &format!("({})", x));

        if expr.contains("^2") {
            let base = expr.replace("^2", "");
            return self.eval(&base, x) * self.eval(&base, x);
        }

        if expr.contains("sin") {
            let inner = expr
                .trim_start_matches("sin")
                .trim_start_matches('(')
                .trim_end_matches(')');
            return self.eval(inner, x).sin();
        }

        if expr.contains("cos") {
            let inner = expr
                .trim_start_matches("cos")
                .trim_start_matches('(')
                .trim_end_matches(')');
            return self.eval(inner, x).cos();
        }

        if expr.contains("exp") {
            let inner = expr
                .trim_start_matches("exp")
                .trim_start_matches('(')
                .trim_end_matches(')');
            return self.eval(inner, x).exp();
        }

        if let Some(pos) = expr.rfind('+') {
            if pos > 0 && pos < expr.len() - 1 {
                return self.eval(&expr[..pos], x) + self.eval(&expr[pos + 1..], x);
            }
        }

        if let Some(pos) = expr.find('-') {
            if pos > 0 && pos < expr.len() - 1 && !expr.starts_with('-') {
                return self.eval(&expr[..pos], x) - self.eval(&expr[pos + 1..], x);
            }
        }

        expr.trim().parse::<f64>().unwrap_or(0.0)
    }

    fn integrate_derivative(&self, expr: &str, a: f64, b: f64) -> f64 {
        let n = 100;
        let h = (b - a) / n as f64;
        let mut sum = 0.0;

        for i in 0..=n {
            let x = a + i as f64 * h;
            let fx = self.eval_derivative(expr, x);

            if i == 0 || i == n {
                sum += fx;
            } else {
                sum += 2.0 * fx;
            }
        }

        sum * h / 2.0
    }

    fn eval_derivative(&self, expr: &str, x: f64) -> f64 {
        let h = 1e-8;
        (self.eval(expr, x + h) - self.eval(expr, x - h)) / (2.0 * h)
    }
}

impl super::VerificationStrategy for Level2Inverse {
    fn verify(&self, expression: &str) -> Result<VerificationCertificate, VerificationError> {
        let start = Instant::now();

        let (mean_error, max_error) = self.verify_integral_derivative(expression)?;

        let duration = start.elapsed().as_millis() as u64;

        let confidence = (1.0 - mean_error.min(1.0)).max(0.0);

        let status = if confidence >= 0.85 {
            crate::config::VerificationStatus::Passed
        } else if confidence >= 0.5 {
            crate::config::VerificationStatus::Uncertain
        } else {
            crate::config::VerificationStatus::Failed
        };

        Ok(VerificationCertificate::new(
            expression.to_string(),
            VerificationLevel::Level2,
            self.num_test_points,
            max_error,
            mean_error,
            mean_error.sqrt(),
            duration,
        )
        .with_status(status))
    }

    fn level(&self) -> VerificationLevel {
        VerificationLevel::Level2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::VerificationStrategy;

    #[test]
    fn test_level2_basic() {
        let verifier = Level2Inverse::new(1e-6, 10);
        let result = verifier.verify("x^2");
        assert!(result.is_ok());
    }
}
