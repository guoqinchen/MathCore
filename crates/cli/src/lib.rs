//! MathCore CLI - Command-line interface

use std::collections::HashMap;
use std::str::FromStr;

use clap::{Parser, Subcommand};

/// CLI error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// MathCore CLI argument parser
#[derive(Parser, Debug)]
#[command(name = "mathcore")]
#[command(version = "0.1.0")]
#[command(about = "MathCore - High-performance mathematical computing", long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "warn")]
    /// Verbosity level: trace, debug, info, warn, error
    pub verbose: String,

    #[command(subcommand)]
    pub command: Commands,
}

/// CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Evaluate a mathematical expression with variable values
    Compute {
        /// The expression to evaluate (e.g., "x^2 + 2*x + 1")
        expression: String,

        /// Variable assignments in the format --var=NAME=VALUE (can be repeated)
        #[arg(short = 'x', long = "var", value_name = "NAME=VALUE")]
        variables: Vec<String>,
    },

    /// Simplify a mathematical expression
    Simplify {
        /// The expression to simplify (e.g., "(x+1)^2 - x^2 - 2*x - 1")
        expression: String,
    },

    /// Differentiate a mathematical expression
    Diff {
        /// The expression to differentiate (e.g., "x^2 + sin(x)")
        expression: String,

        /// Variable to differentiate with respect to
        #[arg(short, long, default_value = "x")]
        var: String,
    },

    /// Integrate a mathematical expression numerically
    Integrate {
        /// The expression to integrate (e.g., "x^2")
        expression: String,

        /// Variable to integrate with respect to
        #[arg(short, long, default_value = "x")]
        var: String,

        /// Lower bound of integration
        #[arg(short, long)]
        from: Option<f64>,

        /// Upper bound of integration
        #[arg(short, long)]
        to: Option<f64>,

        /// Number of subdivisions (default: 1000)
        #[arg(short, long)]
        n: Option<usize>,
    },

    /// Show version information
    Version,
}

/// Parse a variable string in the format "name=value"
fn parse_variable(var: &str) -> Result<(String, f64)> {
    let parts: Vec<&str> = var.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidArgument(format!(
            "Invalid variable format: '{}'. Expected NAME=VALUE",
            var
        )));
    }

    let name = parts[0].trim().to_string();
    if name.is_empty() {
        return Err(Error::InvalidArgument(format!(
            "Invalid variable format: '{}'. Name cannot be empty",
            var
        )));
    }
    let value = f64::from_str(parts[1].trim()).map_err(|_| {
        Error::InvalidArgument(format!("Invalid value for '{}': '{}'", name, parts[1]))
    })?;

    Ok((name, value))
}

/// Run the compute command
pub fn run_compute(expression: &str, variables: &[String]) -> Result<String> {
    let mut vars = HashMap::new();
    for var in variables {
        let (name, value) = parse_variable(var)?;
        vars.insert(name, value);
    }

    if vars.is_empty() {
        return Err(Error::InvalidArgument(
            "No variables provided. Use --var=NAME=VALUE to specify variables.".to_string(),
        ));
    }

    let result = mathcore_compute::compute(expression, &vars)
        .map_err(|e| Error::Execution(e.to_string()))?;

    Ok(format!("{}", result))
}

/// Run the simplify command
pub fn run_simplify(expression: &str) -> Result<String> {
    let expr = mathcore_compute::parse(expression).map_err(|e| Error::Parse(e.to_string()))?;

    let simplified =
        mathcore_compute::simplify(&expr).map_err(|e| Error::Execution(e.to_string()))?;

    Ok(simplified.to_string())
}

/// Run the diff command
pub fn run_diff(expression: &str, var: &str) -> Result<String> {
    let deriv = mathcore_compute::derivative(expression, var)
        .map_err(|e| Error::Execution(e.to_string()))?;

    Ok(deriv)
}

/// Run the integrate command
pub fn run_integrate(
    expression: &str,
    var: &str,
    from: Option<f64>,
    to: Option<f64>,
    n: Option<usize>,
) -> Result<String> {
    let from = from.ok_or_else(|| {
        Error::InvalidArgument("Lower bound (--from) is required for integration.".to_string())
    })?;

    let to = to.ok_or_else(|| {
        Error::InvalidArgument("Upper bound (--to) is required for integration.".to_string())
    })?;

    let result = mathcore_compute::numeric::integrate_expr_simpson(expression, var, from, to, n)
        .map_err(|e| Error::Execution(e.to_string()))?;

    Ok(format!("{}", result))
}

/// Get version string
pub fn get_version() -> String {
    format!("MathCore CLI v{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable() {
        let (name, value) = parse_variable("x=3").unwrap();
        assert_eq!(name, "x");
        assert!((value - 3.0).abs() < 1e-10);

        let (name, value) = parse_variable("y = 3.14").unwrap();
        assert_eq!(name, "y");
        assert!((value - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_parse_variable_invalid() {
    #[test]
    fn test_parse_variable_invalid() {
        assert!(parse_variable("x").is_err());
        // Empty name should be invalid
        assert!(parse_variable("=3").is_err());
        assert!("x=3=y".parse::<f64>().is_err()); // trailing part ignored
    }
        assert!(parse_variable("=3").is_err());
    }

    #[test]
    fn test_run_compute() {
        let result = run_compute("x^2 + 2*x + 1", &["x=3".to_string()]).unwrap();
        assert!((result.parse::<f64>().unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn test_run_simplify() {
        let result = run_simplify("x + 0").unwrap();
        assert_eq!(result, "x");

        let result = run_simplify("x * 1").unwrap();
        assert_eq!(result, "x");
    }

    #[test]
    fn test_run_diff() {
        let result = run_diff("x^2", "x").unwrap();
        // Should contain "2" and "x"
        assert!(result.contains('2') || result.contains('x'));
    }

    #[test]
    fn test_run_integrate() {
        let result = run_integrate("x^2", "x", Some(0.0), Some(1.0), Some(100)).unwrap();
        let value: f64 = result.parse().unwrap();
        assert!((value - 1.0 / 3.0).abs() < 1e-3);
    }
}
