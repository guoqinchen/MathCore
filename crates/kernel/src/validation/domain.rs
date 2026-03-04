// Domain and boundary checking
use super::types::*;
use super::types::*;

pub struct DomainChecker;

impl DomainChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn check(&self, expr: &str) -> ValidationResult<()> {
        self.check_sqrt_domain(expr)?;
        self.check_log_domain(expr)?;
        self.check_division(expr)?;
        self.check_factorial_domain(expr)?;
        Ok(())
    }

    fn check_sqrt_domain(&self, expr: &str) -> ValidationResult<()> {
        // Pattern: sqrt(negative) or sqrt(-x)
        if let Some(neg_idx) = expr.find("sqrt(-") {
            let inner = &expr[neg_idx + 5..];
            if let Some(end) = inner.find(')') {
                let arg = &inner[1..end];
                if !arg.is_empty() {
                    return Err(err::domain_error("sqrt定义域: 不能对负数求平方根"));
                }
            }
        }

        // Also check for sqrt(-number) pattern
        let re = regex_lite::Regex::new(r"sqrt\s*\(\s*-\s*[\d.]+\s*\)").ok();
        if let Some(re) = re {
            if re.is_match(expr) {
                return Err(err::domain_error("sqrt定义域: 不能对负数求平方根"));
            }
        }

        Ok(())
    }

    fn check_log_domain(&self, expr: &str) -> ValidationResult<()> {
        // Check log, ln, log10, log2
        let log_funcs = ["log(", "ln(", "log10(", "log2("];

        for func in &log_funcs {
            if let Some(start) = expr.find(*func) {
                let after_func = &expr[start + func.len()..];
                if let Some(end) = after_func.find(')') {
                    let arg = after_func[..end].trim();
                    if let Ok(val) = arg.parse::<f64>() {
                        if val <= 0.0 {
                            return Err(err::domain_error("log定义域: 参数必须为正数"));
                        }
                    }
                    // Check for negative literal
                    if arg.starts_with('-') {
                        return Err(err::domain_error("log定义域: 参数必须为正数"));
                    }
                }
            }
        }

        Ok(())
    }

    fn check_division(&self, expr: &str) -> ValidationResult<()> {
        // Check for /0 or / 0
        let div_re = regex_lite::Regex::new(r"/\s*0(\s|$|\))").ok();
        if let Some(re) = div_re {
            if re.is_match(expr) {
                return Err(err::division_by_zero());
            }
        }

        // Check modulo by zero
        let mod_re = regex_lite::Regex::new(r"%\s*0(\s|$|\))").ok();
        if let Some(re) = mod_re {
            if re.is_match(expr) {
                return Err(err::division_by_zero());
            }
        }

        Ok(())
    }

    fn check_factorial_domain(&self, expr: &str) -> ValidationResult<()> {
        // Check factorial for negative numbers
        let fact_re = regex_lite::Regex::new(r"-\d+!|!\s*-\d+").ok();
        if let Some(re) = fact_re {
            if re.is_match(expr) {
                return Err(err::domain_error("阶乘定义域: 不能对负数求阶乘"));
            }
        }

        // Check for large factorial (would overflow)
        let large_fact = regex_lite::Regex::new(r"[2-9]\d{2,}!|1[789]\d{2,}!").ok();
        if let Some(re) = large_fact {
            if re.is_match(expr) {
                return Err(err::overflow("阶乘结果过大，会导致溢出"));
            }
        }

        Ok(())
    }
}

impl Default for DomainChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt_domain() {
        let dc = DomainChecker::new();
        assert!(dc.check("sqrt(-1)").is_err());
        assert!(dc.check("sqrt(4)").is_ok());
    }

    #[test]
    fn test_log_domain() {
        let dc = DomainChecker::new();
        assert!(dc.check("log(0)").is_err());
        assert!(dc.check("log(-1)").is_err());
        assert!(dc.check("log(1)").is_ok());
    }

    #[test]
    fn test_division_by_zero() {
        let dc = DomainChecker::new();
        assert!(dc.check("1/0").is_err());
        assert!(dc.check("5%0").is_err());
        assert!(dc.check("1/2").is_ok());
    }

    #[test]
    fn test_factorial_domain() {
        let dc = DomainChecker::new();
        assert!(dc.check("-5!").is_err());
        // Large factorial would overflow
        assert!(dc.check("200!").is_err());
    }
}
