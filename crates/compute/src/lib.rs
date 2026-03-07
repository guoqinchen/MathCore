//! MathCore Compute - Mathematical computation engines
//!
//! This crate provides symbolic, numeric, and external computation capabilities.
//!
//! # Core Features
//!
//! - **Symbolic Computation**: Parse, simplify, differentiate, and evaluate mathematical expressions
//! - **Numeric Computation**: Numerical integration, differentiation, and equation solving
//! - **Caching**: Multi-level caching for optimized performance
//! - **Validation**: Input validation to prevent errors and security issues
//!
//! # Quick Start
//!
//! This crate provides functions for symbolic and numeric computation.
//! See individual function documentation for usage examples.
//!
//! # Modules
//!
//! - [`cache`] - Multi-level caching system for computation optimization
//! - [`external`] - External computation engine integration
//! - [`lexer`] - Tokenizer for mathematical expressions
//! - [`numeric`] - Numerical computation (integration, differentiation, solving)
//! - [`replay`] - Computation tracing and replay
//! - [`symbolic`] - Symbolic computation engine
//! - [`validation`] - Input validation utilities

pub mod cache;
pub mod external;
pub mod lexer;
pub mod numeric;
pub mod replay;
pub mod symbolic;
pub mod validation;

use std::collections::HashMap;

/// Result type alias for compute operations
///
/// # Example
///
/// ```
/// use mathcore_compute::Result;
///
/// fn example() -> Result<f64> {
///     Ok(42.0)
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for compute engines
///
/// This enum covers all possible errors that can occur during computation:
/// - Symbolic computation errors
/// - Numeric computation errors
/// - External engine errors
/// - Parse errors
/// - Validation errors
///
/// # Example
///
/// ```
/// use mathcore_compute::Error;
///
/// fn handle_error(err: Error) {
///     match err {
///         Error::Symbolic(e) => println!("Symbolic error: {}", e),
///         Error::Numeric(e) => println!("Numeric error: {}", e),
///         Error::Parse(msg) => println!("Parse error: {}", msg),
///         Error::Validation(e) => println!("Validation error: {}", e),
///         Error::External(e) => println!("External error: {}", e),
///     }
/// }
/// ```
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

    #[error("Validation error: {0}")]
    Validation(#[from] validation::ValidationError),
}

// Re-export symbolic functions
pub use symbolic::{differentiate, evaluate, parse, simplify, Expr, SymbolicEngine};

// Re-export replay functions
pub use replay::{ComputationStep, ComputationTrace, TraceBuilder, TraceReplayer};

// Re-export numeric functions
pub use numeric::{
    differentiate as numeric_differentiate, differentiate_expr, eval, eval_simple,
    integrate_expr_simpson, integrate_simpson, solve_bisection, solve_bisection_expr,
    solve_fixed_point, solve_newton, solve_newton_expr, NumericEngine,
};

/// Convenience function to parse and evaluate a symbolic expression
///
/// This is a high-level function that combines parsing, simplification,
/// and evaluation in one step.
///
/// # Arguments
///
/// * `input` - A string containing a mathematical expression
/// * `vars` - A map of variable names to their values
///
/// # Returns
///
/// Returns the evaluated result as a `f64`, or an error if the
/// expression is invalid or evaluation fails.
///
/// # Errors
///
/// Returns an error if:
/// - The expression fails validation
/// - The expression cannot be parsed
/// - The expression cannot be simplified
/// - Evaluation fails (e.g., division by zero)
///
/// # Example
///
/// ```
/// use mathcore_compute::compute;
/// use std::collections::HashMap;
///
/// // Simple expression
/// let result = compute("2 + 3", &HashMap::new()).unwrap();
/// assert!((result - 5.0).abs() < 1e-10);
///
/// // With variables
/// let mut vars = HashMap::new();
/// vars.insert("x".to_string(), 3.0);
/// let result = compute("x^2 + 2*x + 1", &vars).unwrap();
/// assert!((result - 16.0).abs() < 1e-10);
/// ```
pub fn compute(input: &str, vars: &HashMap<String, f64>) -> Result<f64> {
    validation::validate_expression(input)?;
    validation::validate_variables(vars)?;
    
    let expr = symbolic::parse(input)?;
    let simplified = symbolic::simplify(&expr)?;
    symbolic::evaluate(&simplified, vars).map_err(Error::Symbolic)
}

/// Convenience function to get derivative of a symbolic expression
///
/// Computes the symbolic derivative of an expression with respect to
/// a given variable.
///
/// # Arguments
///
/// * `input` - A string containing a mathematical expression
/// * `var` - The variable name to differentiate with respect to
///
/// # Returns
///
/// Returns the derivative as a simplified string, or an error if
/// the expression is invalid.
///
/// # Errors
///
/// Returns an error if:
/// - The expression fails validation
/// - The variable name is invalid
/// - The expression cannot be parsed
/// - Differentiation fails
///
/// # Example
///
/// ```
/// use mathcore_compute::derivative;
///
/// // Basic derivative
/// let deriv = derivative("x^2", "x").unwrap();
/// assert!(deriv.contains("2") && deriv.contains("x"));
/// ```
pub fn derivative(input: &str, var: &str) -> Result<String> {
    validation::validate_expression(input)?;
    validation::validate_variable_name(var)?;
    
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
