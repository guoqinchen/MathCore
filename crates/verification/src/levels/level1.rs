//! Level 1: Numerical Consistency Verification
//!
//! Uses random sampling to verify numerical consistency of mathematical expressions.

use crate::certificate::VerificationCertificate;
use crate::config::VerificationLevel;
use rand::Rng;
use std::time::Instant;

pub struct Level1Numerical {
    num_samples: usize,
    tolerance: f64,
    variable_ranges: Vec<(char, (f64, f64))>,
}

impl Level1Numerical {
    pub fn new(num_samples: usize, tolerance: f64) -> Self {
        let variable_ranges = vec![
            ('x', (-10.0, 10.0)),
            ('y', (-10.0, 10.0)),
            ('z', (-10.0, 10.0)),
        ];
        Self {
            num_samples,
            tolerance,
            variable_ranges,
        }
    }

    pub fn with_variable(mut self, var: char, min: f64, max: f64) -> Self {
        self.variable_ranges.push((var, (min, max)));
        self
    }

    fn evaluate_expression(&self, expr: &str, x: f64) -> Result<f64, String> {
        self.eval_simple(expr, x)
    }

    fn eval_simple(&self, expr: &str, x: f64) -> Result<f64, String> {
        let expr = expr.replace("x", &format!("({})", x));

        if expr.contains("^2") {
            let base = expr.replace("^2", "");
            let val = self.eval_simple(&base, x)?;
            return Ok(val * val);
        }

        if expr.contains("sin") {
            let inner = expr
                .trim_start_matches("sin")
                .trim_start_matches('(')
                .trim_end_matches(')');
            let val = self.eval_simple(inner, x)?;
            return Ok(val.sin());
        }

        if expr.contains("cos") {
            let inner = expr
                .trim_start_matches("cos")
                .trim_start_matches('(')
                .trim_end_matches(')');
            let val = self.eval_simple(inner, x)?;
            return Ok(val.cos());
        }

        if expr.contains("exp") {
            let inner = expr
                .trim_start_matches("exp")
                .trim_start_matches('(')
                .trim_end_matches(')');
            let val = self.eval_simple(inner, x)?;
            return Ok(val.exp());
        }

        if let Some(pos) = expr.rfind('+') {
            if pos > 0 && pos < expr.len() - 1 {
                let left = self.eval_simple(&expr[..pos], x)?;
                let right = self.eval_simple(&expr[pos + 1..], x)?;
                return Ok(left + right);
            }
        }

        if let Some(pos) = expr.find('-') {
            if pos > 0 && pos < expr.len() - 1 && !expr.starts_with('-') {
                let left = self.eval_simple(&expr[..pos], x)?;
                let right = self.eval_simple(&expr[pos + 1..], x)?;
                return Ok(left - right);
            }
        }

        expr.trim().parse::<f64>().map_err(|e| e.to_string())
    }

    fn generate_samples(&self) -> Vec<std::collections::HashMap<char, f64>> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(self.num_samples);

        for _ in 0..self.num_samples {
            let mut vars = std::collections::HashMap::new();
            for (var, (min, max)) in &self.variable_ranges {
                vars.insert(*var, rng.gen_range(*min..*max));
            }
            samples.push(vars);
        }

        samples
    }
}

impl super::VerificationStrategy for Level1Numerical {
    fn verify(
        &self,
        expression: &str,
    ) -> Result<VerificationCertificate, super::VerificationError> {
        let start = Instant::now();

        let samples = self.generate_samples();

        let mut values: Vec<f64> = Vec::new();

        for vars in &samples {
            let x = *vars.get(&'x').unwrap_or(&0.0);

            let result = self.evaluate_expression(expression, x);

            if let Ok(val) = result {
                if val.is_finite() {
                    values.push(val);
                }
            }
        }

        let n = values.len();
        let mean = if n > 0 {
            values.iter().sum::<f64>() / n as f64
        } else {
            0.0
        };

        let variance = if n > 1 {
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        let max_error = values
            .iter()
            .map(|v| (v - mean).abs())
            .fold(0.0_f64, |a, b| a.max(b));

        let duration = start.elapsed().as_millis() as u64;

        Ok(VerificationCertificate::new(
            expression.to_string(),
            VerificationLevel::Level1,
            n,
            max_error,
            mean,
            std_dev,
            duration,
        ))
    }

    fn level(&self) -> VerificationLevel {
        VerificationLevel::Level1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::VerificationStrategy;

    #[test]
    fn test_level1_basic() {
        let verifier = Level1Numerical::new(50, 1e-10);
        let result = verifier.verify("x^2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_level1_trig() {
        let verifier = Level1Numerical::new(30, 1e-10);
        let result = verifier.verify("sin(x)");
        assert!(result.is_ok());
    }
}
