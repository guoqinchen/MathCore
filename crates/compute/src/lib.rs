//! MathCore Compute - Mathematical computation engines
//!
//! This crate provides symbolic, numeric, and external computation capabilities.

pub mod external;
pub mod numeric;
pub mod symbolic;

use std::collections::HashMap;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for compute engines
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Symbolic error: {0}")]
    Symbolic(#[from] symbolic::Error),

    #[error("Numeric error: {0}")]
    Numeric(#[from] numeric::Error),

    #[error("External engine error: {0}")]
    External(#[from] external::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

// Re-export symbolic functions
pub use symbolic::{differentiate, evaluate, parse, simplify, Expr, SymbolicEngine};

// Re-export numeric functions
pub use numeric::{
    differentiate as numeric_differentiate, differentiate_expr, eval, eval_simple,
    integrate_expr_simpson, integrate_simpson, solve_bisection, solve_bisection_expr,
    solve_fixed_point, solve_newton, solve_newton_expr, NumericEngine,
};

/// Convenience function to parse and evaluate a symbolic expression
pub fn compute(input: &str, vars: &HashMap<String, f64>) -> Result<f64> {
    let expr = symbolic::parse(input)?;
    let simplified = symbolic::simplify(&expr)?;
    symbolic::evaluate(&simplified, vars).map_err(Error::Symbolic)
}

/// Convenience function to get derivative of a symbolic expression
pub fn derivative(input: &str, var: &str) -> Result<String> {
    let expr = symbolic::parse(input)?;
    let deriv = symbolic::differentiate(&expr, var)?;
    let simplified = symbolic::simplify(&deriv)?;
    Ok(simplified.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3.0);

        let result = compute("x^2 + 2*x + 1", &vars).unwrap();
        assert!((result - 16.0).abs() < 1e-10);
    }

    #[test]
    fn test_derivative() {
        let deriv = derivative("x^2 + 2*x + 1", "x").unwrap();
        assert!(deriv.contains("2") && deriv.contains("x"));
    }

    #[test]
    fn test_numeric_eval() {
        let result = numeric::eval_simple("2 + 3 * 4").unwrap();
        assert!((result - 14.0).abs() < 1e-10);
    }

    #[test]
    fn test_numeric_differentiate() {
        let f = |x: f64| Ok(x * x);
        let result = numeric::differentiate(f, 2.0, None).unwrap();
        assert!((result - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_numeric_integrate() {
        let f = |x: f64| Ok(x * x);
        let result = numeric::integrate_simpson(f, 0.0, 1.0, Some(100)).unwrap();
        assert!((result - 1.0 / 3.0).abs() < 1e-5);
    }
}
