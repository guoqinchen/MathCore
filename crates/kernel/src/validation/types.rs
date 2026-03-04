// Validation types for NanoCheck L0
use std::collections::HashMap;
use std::fmt;

/// Error code categories
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Syntax errors (E001-E099)
    E001, // Empty expression
    E002, // Invalid character
    E003, // Unmatched parentheses
    E004, // Consecutive operators
    E005, // Trailing operator

    // Type errors (T001-T099)
    T001, // Type mismatch
    T002, // Type inference failed

    // Domain errors (D001-D099)
    D001, // Negative square root
    D002, // Log of non-positive
    D003, // Division by zero
    D004, // Negative factorial
    D005, // Factorial too large

    // Overflow errors (O001-O099)
    O001, // Integer overflow
    O002, // Result too large
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            ErrorCode::E001 => "E001",
            ErrorCode::E002 => "E002",
            ErrorCode::E003 => "E003",
            ErrorCode::E004 => "E004",
            ErrorCode::E005 => "E005",
            ErrorCode::T001 => "T001",
            ErrorCode::T002 => "T002",
            ErrorCode::D001 => "D001",
            ErrorCode::D002 => "D002",
            ErrorCode::D003 => "D003",
            ErrorCode::D004 => "D004",
            ErrorCode::D005 => "D005",
            ErrorCode::O001 => "O001",
            ErrorCode::O002 => "O002",
        };
        write!(f, "{}", code)
    }
}

/// Validation error with location tracking
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub message: String,
    pub position: Option<(usize, usize)>, // (line, column)
    pub context: HashMap<String, String>,
}

impl ValidationError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            position: None,
            context: HashMap::new(),
        }
    }

    pub fn with_position(mut self, line: usize, col: usize) -> Self {
        self.position = Some((line, col));
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos = match self.position {
            Some((line, col)) => format!("行{}列{}", line, col),
            None => "未知位置".to_string(),
        };
        write!(f, "[{}] {}: {}", self.code, pos, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validation result type
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Quick error helpers
pub mod err {
    use super::*;

    pub fn empty_expr() -> ValidationError {
        ValidationError::new(ErrorCode::E001, "表达式为空")
    }

    pub fn invalid_char(c: char, col: usize) -> ValidationError {
        ValidationError::new(ErrorCode::E002, format!("无效字符: '{}'", c)).with_position(1, col)
    }

    pub fn unmatched_paren(col: usize) -> ValidationError {
        ValidationError::new(ErrorCode::E003, "括号不匹配").with_position(1, col)
    }

    pub fn consecutive_ops(op1: &str, op2: &str) -> ValidationError {
        ValidationError::new(ErrorCode::E004, format!("连续操作符: {} {}", op1, op2))
    }

    pub fn trailing_op(op: &str) -> ValidationError {
        ValidationError::new(ErrorCode::E005, format!("尾随操作符: {}", op))
    }

    pub fn type_mismatch(expected: &str, found: &str) -> ValidationError {
        ValidationError::new(
            ErrorCode::T001,
            format!("类型不匹配: 期望{}, 实际{}", expected, found),
        )
    }

    pub fn domain_error(msg: &str) -> ValidationError {
        ValidationError::new(ErrorCode::D001, msg)
    }

    pub fn division_by_zero() -> ValidationError {
        ValidationError::new(ErrorCode::D003, "除零错误")
    }

    pub fn overflow(msg: &str) -> ValidationError {
        ValidationError::new(ErrorCode::O001, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(format!("{}", ErrorCode::E001), "E001");
        assert_eq!(format!("{}", ErrorCode::T001), "T001");
        assert_eq!(format!("{}", ErrorCode::D001), "D001");
    }

    #[test]
    fn test_validation_error_display() {
        let e = ValidationError::new(ErrorCode::E001, "表达式为空");
        assert_eq!(format!("{}", e), "[E001] 未知位置: 表达式为空");

        let e = e.with_position(1, 5);
        assert_eq!(format!("{}", e), "[E001] 行1列5: 表达式为空");
    }

    #[test]
    fn test_validation_error_context() {
        let e = ValidationError::new(ErrorCode::T001, "类型错误")
            .with_context("expr", "sqrt(-1)")
            .with_context("operation", "sqrt");

        assert_eq!(e.context.get("expr"), Some(&"sqrt(-1)".to_string()));
    }
}
