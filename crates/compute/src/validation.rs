//! Input validation utilities for MathCore
//!
//! This module provides validation functions to ensure input safety
//! and prevent potential security issues like injection attacks or DoS.

use std::fmt;

/// Maximum allowed expression length (1MB)
pub const MAX_EXPRESSION_LENGTH: usize = 1_048_576;

/// Maximum allowed variable name length
pub const MAX_VARIABLE_NAME_LENGTH: usize = 256;

/// Maximum allowed number of variables
pub const MAX_VARIABLES_COUNT: usize = 1000;

/// Valid characters for mathematical expressions
/// Allows: alphanumeric, operators, parentheses, whitespace, and common math symbols
pub const VALID_EXPRESSION_CHARS: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+-*/^()[]{}.,= _\t\n\rπθαβγδεζηικλμνξρστυφχψω∞∫∂∇√∑∏";

/// Valid characters for variable names
/// Allows: alphanumeric and underscore
pub const VALID_VARIABLE_NAME_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789";

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Input exceeds maximum length
    TooLong {
        max: usize,
        actual: usize,
        context: String,
    },
    /// Input contains invalid characters
    InvalidCharacters {
        position: usize,
        char: char,
        context: String,
    },
    /// Input is empty
    EmptyInput {
        context: String,
    },
    /// Too many variables
    TooManyVariables {
        max: usize,
        actual: usize,
    },
    /// Invalid variable name
    InvalidVariableName {
        name: String,
        reason: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong { max, actual, context } => {
                write!(f, "{}: input too long (max {}, actual {})", context, max, actual)
            }
            Self::InvalidCharacters { position, char, context } => {
                write!(f, "{}: invalid character '{}' at position {}", context, char, position)
            }
            Self::EmptyInput { context } => {
                write!(f, "{}: input cannot be empty", context)
            }
            Self::TooManyVariables { max, actual } => {
                write!(f, "Too many variables: max {}, actual {}", max, actual)
            }
            Self::InvalidVariableName { name, reason } => {
                write!(f, "Invalid variable name '{}': {}", name, reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate expression input
pub fn validate_expression(expression: &str) -> Result<(), ValidationError> {
    if expression.is_empty() {
        return Err(ValidationError::EmptyInput {
            context: "Expression".to_string(),
        });
    }

    if expression.len() > MAX_EXPRESSION_LENGTH {
        return Err(ValidationError::TooLong {
            max: MAX_EXPRESSION_LENGTH,
            actual: expression.len(),
            context: "Expression".to_string(),
        });
    }

    for (i, ch) in expression.char_indices() {
        if !VALID_EXPRESSION_CHARS.contains(ch) {
            return Err(ValidationError::InvalidCharacters {
                position: i,
                char: ch,
                context: "Expression".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate variable name
pub fn validate_variable_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::InvalidVariableName {
            name: name.to_string(),
            reason: "name cannot be empty".to_string(),
        });
    }

    if name.len() > MAX_VARIABLE_NAME_LENGTH {
        return Err(ValidationError::InvalidVariableName {
            name: name.to_string(),
            reason: format!("name too long (max {})", MAX_VARIABLE_NAME_LENGTH),
        });
    }

    if let Some(first_char) = name.chars().next() {
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(ValidationError::InvalidVariableName {
                name: name.to_string(),
                reason: "name must start with a letter or underscore".to_string(),
            });
        }
    } else {
        return Err(ValidationError::InvalidVariableName {
            name: name.to_string(),
            reason: "name cannot be empty".to_string(),
        });
    }

    for (i, ch) in name.char_indices() {
        if !VALID_VARIABLE_NAME_CHARS.contains(ch) {
            return Err(ValidationError::InvalidVariableName {
                name: name.to_string(),
                reason: format!("invalid character '{}' at position {}", ch, i),
            });
        }
    }

    Ok(())
}

/// Validate variables map
pub fn validate_variables(variables: &std::collections::HashMap<String, f64>) -> Result<(), ValidationError> {
    if variables.len() > MAX_VARIABLES_COUNT {
        return Err(ValidationError::TooManyVariables {
            max: MAX_VARIABLES_COUNT,
            actual: variables.len(),
        });
    }

    for name in variables.keys() {
        validate_variable_name(name)?;
    }

    for (name, value) in variables {
        if !value.is_finite() {
            return Err(ValidationError::InvalidVariableName {
                name: name.clone(),
                reason: format!("variable value {} is not finite", value),
            });
        }
    }

    Ok(())
}

/// Validate numeric range
pub fn validate_numeric_range(value: f64, min: f64, max: f64, name: &str) -> Result<(), ValidationError> {
    if !value.is_finite() {
        return Err(ValidationError::InvalidVariableName {
            name: name.to_string(),
            reason: "value is not finite".to_string(),
        });
    }

    if value < min || value > max {
        return Err(ValidationError::InvalidVariableName {
            name: name.to_string(),
            reason: format!("value {} out of range [{}, {}]", value, min, max),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_expression() {
        assert!(validate_expression("x + y").is_ok());
        assert!(validate_expression("x^2 + 2*x + 1").is_ok());
        assert!(validate_expression("").is_err());
        assert!(validate_expression(&"x".repeat(MAX_EXPRESSION_LENGTH + 1)).is_err());
        assert!(validate_expression("x + \x00").is_err());
        assert!(validate_expression("sin(x) + cos(y)").is_ok());
        assert!(validate_expression("x * (y + z)").is_ok());
        assert!(validate_expression("sqrt(x^2 + y^2)").is_ok());
    }

    #[test]
    fn test_validate_expression_edge_cases() {
        assert!(validate_expression("0").is_ok());
        assert!(validate_expression("0.0").is_ok());
        assert!(validate_expression(".5").is_ok());
        assert!(validate_expression("1e10").is_ok());
        assert!(validate_expression("   ").is_ok());
        assert!(validate_expression("\t\n\r").is_ok());
    }

    #[test]
    fn test_validate_variable_name() {
        assert!(validate_variable_name("x").is_ok());
        assert!(validate_variable_name("_var").is_ok());
        assert!(validate_variable_name("var123").is_ok());
        assert!(validate_variable_name("").is_err());
        assert!(validate_variable_name("123var").is_err());
        assert!(validate_variable_name("var-name").is_err());
        assert!(validate_variable_name("var name").is_err());
    }

    #[test]
    fn test_validate_variable_name_edge_cases() {
        assert!(validate_variable_name("a").is_ok());
        assert!(validate_variable_name("Z").is_ok());
        assert!(validate_variable_name("_").is_ok());
        assert!(validate_variable_name("_1").is_ok());
        assert!(validate_variable_name("a1b2c3").is_ok());
        assert!(validate_variable_name(&"a".repeat(MAX_VARIABLE_NAME_LENGTH)).is_ok());
        assert!(validate_variable_name(&"a".repeat(MAX_VARIABLE_NAME_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_validate_variables() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("x".to_string(), 1.0);
        vars.insert("y".to_string(), 2.0);
        assert!(validate_variables(&vars).is_ok());

        vars.insert("bad-var".to_string(), 3.0);
        assert!(validate_variables(&vars).is_err());
    }

    #[test]
    fn test_validate_variables_edge_cases() {
        let mut vars = std::collections::HashMap::new();
        
        vars.insert("x".to_string(), 0.0);
        assert!(validate_variables(&vars).is_ok());
        
        vars.insert("y".to_string(), -1.0);
        assert!(validate_variables(&vars).is_ok());
        
        vars.insert("z".to_string(), f64::NAN);
        assert!(validate_variables(&vars).is_err());
        
        let mut vars2 = std::collections::HashMap::new();
        for i in 0..MAX_VARIABLES_COUNT {
            vars2.insert(format!("var{}", i), 1.0);
        }
        assert!(validate_variables(&vars2).is_ok());
        
        vars2.insert("extra".to_string(), 1.0);
        assert!(validate_variables(&vars2).is_err());
    }

    #[test]
    fn test_validate_numeric_range() {
        assert!(validate_numeric_range(5.0, 0.0, 10.0, "test").is_ok());
        assert!(validate_numeric_range(-1.0, 0.0, 10.0, "test").is_err());
        assert!(validate_numeric_range(f64::NAN, 0.0, 10.0, "test").is_err());
        assert!(validate_numeric_range(f64::INFINITY, 0.0, 10.0, "test").is_err());
    }

    #[test]
    fn test_validate_numeric_range_edge_cases() {
        assert!(validate_numeric_range(0.0, 0.0, 10.0, "test").is_ok());
        assert!(validate_numeric_range(10.0, 0.0, 10.0, "test").is_ok());
        assert!(validate_numeric_range(0.001, 0.0, 1.0, "test").is_ok());
        assert!(validate_numeric_range(-0.001, -1.0, 0.0, "test").is_ok());
        assert!(validate_numeric_range(1.001, 0.0, 1.0, "test").is_err());
        assert!(validate_numeric_range(f64::NEG_INFINITY, 0.0, 10.0, "test").is_err());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::TooLong {
            max: 100,
            actual: 200,
            context: "Test".to_string(),
        };
        assert!(err.to_string().contains("200"));
        
        let err = ValidationError::EmptyInput {
            context: "Expression".to_string(),
        };
        assert!(err.to_string().contains("empty"));
        
        let err = ValidationError::InvalidVariableName {
            name: "test".to_string(),
            reason: "invalid character".to_string(),
        };
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_validation_error_debug() {
        let err = ValidationError::TooLong {
            max: 100,
            actual: 200,
            context: "Test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("TooLong"));
    }
}
