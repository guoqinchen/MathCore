// NaN and Infinity handling (IEEE 754 special values)
use super::types::*;
use super::types::*;

pub struct SpecialValueChecker;

impl SpecialValueChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn is_valid_number(&self, val: f64) -> bool {
        !val.is_nan() && !val.is_infinite()
    }

    pub fn check_result(&self, val: f64, operation: &str) -> ValidationResult<f64> {
        if val.is_nan() {
            return Err(ValidationError::new(
                ErrorCode::O001,
                format!("{} 结果为 NaN (无效数字)", operation),
            ));
        }
        if val.is_infinite() {
            return Err(ValidationError::new(
                ErrorCode::O001,
                format!("{} 结果为无穷大 (溢出)", operation),
            ));
        }
        Ok(val)
    }

    pub fn validate_operation(&self, left: f64, op: &str, right: f64) -> ValidationResult<f64> {
        let result = match op {
            "+" => left + right,
            "-" => left - right,
            "*" => left * right,
            "/" => left / right,
            "%" => left % right,
            "^" => left.powf(right),
            _ => return Err(ValidationError::new(ErrorCode::E002, "未知操作符")),
        };

        self.check_result(result, &format!("{} {} {}", left, op, right))
    }

    pub fn validate_function(&self, func: &str, arg: f64) -> ValidationResult<f64> {
        let result = match func {
            "sin" => arg.sin(),
            "cos" => arg.cos(),
            "tan" => arg.tan(),
            "asin" => arg.asin(),
            "acos" => arg.acos(),
            "atan" => arg.atan(),
            "sinh" => arg.sinh(),
            "cosh" => arg.cosh(),
            "tanh" => arg.tanh(),
            "exp" => arg.exp(),
            "ln" => arg.ln(),
            "log" => arg.log10(),
            "log2" => arg.log2(),
            "sqrt" => arg.sqrt(),
            "abs" => arg.abs(),
            _ => return Err(ValidationError::new(ErrorCode::E002, "未知函数")),
        };

        self.check_result(result, &format!("{}({})", func, arg))
    }

    pub fn check_expression(&self, expr: &str) -> ValidationResult<()> {
        // Check if expression would produce NaN or Infinity
        // This is a simplified check - real implementation would need parsing

        // Check for known problematic patterns
        let problematic = [
            ("0/0", "0/0 会产生 NaN"),
            ("inf-inf", "无穷大减无穷大会产生 NaN"),
            ("inf*0", "无穷大乘以零会产生 NaN"),
            ("inf/inf", "无穷大除以无穷大"),
            ("nan", "包含 NaN"),
        ];

        let lower = expr.to_lowercase();
        for (pattern, msg) in &problematic {
            if lower.contains(pattern) {
                return Err(ValidationError::new(ErrorCode::O001, *msg));
            }
        }

        Ok(())
    }
}

impl Default for SpecialValueChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_number() {
        let svc = SpecialValueChecker::new();
        assert!(svc.is_valid_number(1.0));
        assert!(svc.is_valid_number(-100.0));
        assert!(!svc.is_valid_number(f64::NAN));
        assert!(!svc.is_valid_number(f64::INFINITY));
        assert!(!svc.is_valid_number(f64::NEG_INFINITY));
    }

    #[test]
    fn test_check_result() {
        let svc = SpecialValueChecker::new();
        assert!(svc.check_result(1.0, "test").is_ok());
        assert!(svc.check_result(f64::NAN, "test").is_err());
        assert!(svc.check_result(f64::INFINITY, "test").is_err());
    }

    #[test]
    fn test_validate_operation() {
        let svc = SpecialValueChecker::new();
        assert!(svc.validate_operation(1.0, "+", 2.0).is_ok());
        assert!(svc.validate_operation(1.0, "/", 0.0).is_err());
    }

    #[test]
    fn test_validate_function() {
        let svc = SpecialValueChecker::new();
        assert!(svc.validate_function("sin", 0.0).is_ok());
        assert!(svc.validate_function("sqrt", -1.0).is_err());
        assert!(svc.validate_function("ln", -1.0).is_err());
    }
}
