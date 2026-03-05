// Quick fail mechanism - incremental validation with early exit
use super::domain::DomainChecker;
use super::parser::ParserValidator;
use super::special::SpecialValueChecker;
use super::type_check::Type;
use super::type_check::TypeChecker;
use super::types::*;

pub struct QuickValidator {
    parser: ParserValidator,
    type_checker: TypeChecker,
    domain_checker: DomainChecker,
    special_checker: SpecialValueChecker,
}

impl QuickValidator {
    pub fn new() -> Self {
        Self {
            parser: ParserValidator::new(),
            type_checker: TypeChecker::new(),
            domain_checker: DomainChecker::new(),
            special_checker: SpecialValueChecker::new(),
        }
    }

    /// Quick validation - stops at first error
    pub fn validate(&self, expr: &str) -> ValidationResult<ValidationReport> {
        let start = std::time::Instant::now();

        // Phase 1: Parser validation (fastest first)
        self.parser.validate(expr)?;

        // Phase 2: Domain validation
        self.domain_checker.check(expr)?;

        // Phase 3: Special values
        self.special_checker.check_expression(expr)?;

        // Phase 4: Type inference (can be slow)
        let expr_type = self.type_checker.check(expr)?;

        let elapsed = start.elapsed();

        Ok(ValidationReport {
            valid: true,
            expr: expr.to_string(),
            expr_type,
            elapsed_ms: elapsed.as_secs_f64() * 1000.0,
            checks_passed: vec![
                "parser".to_string(),
                "domain".to_string(),
                "special".to_string(),
                "type".to_string(),
            ],
            errors: None,
        })
    }

    /// Incremental validation - returns all errors
    pub fn validate_incremental(&self, expr: &str) -> ValidationReport {
        let start = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut checks_passed = Vec::new();

        // Phase 1: Parser
        if let Err(e) = self.parser.validate(expr) {
            errors.push(e);
        } else {
            checks_passed.push("parser".to_string());
        }

        // Phase 2: Domain (only if parser passed)
        if errors.is_empty() {
            if let Err(e) = self.domain_checker.check(expr) {
                errors.push(e);
            } else {
                checks_passed.push("domain".to_string());
            }
        }

        // Phase 3: Special values
        if errors.is_empty() {
            if let Err(e) = self.special_checker.check_expression(expr) {
                errors.push(e);
            } else {
                checks_passed.push("special".to_string());
            }
        }

        // Phase 4: Type check
        let expr_type = if errors.is_empty() {
            match self.type_checker.check(expr) {
                Ok(t) => {
                    checks_passed.push("type".to_string());
                    t
                }
                Err(e) => {
                    errors.push(e);
                    Type::Unknown
                }
            }
        } else {
            Type::Unknown
        };

        let elapsed = start.elapsed();

        ValidationReport {
            valid: errors.is_empty(),
            expr: expr.to_string(),
            expr_type,
            elapsed_ms: elapsed.as_secs_f64() * 1000.0,
            checks_passed,
            errors: if errors.is_empty() {
                None
            } else {
                Some(errors)
            },
        }
    }

    /// Fast check - only parser validation (sub-microsecond)
    pub fn quick_check(&self, expr: &str) -> ValidationResult<()> {
        self.parser.validate(expr)
    }
}

impl Default for QuickValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub valid: bool,
    pub expr: String,
    pub expr_type: Type,
    pub elapsed_ms: f64,
    pub checks_passed: Vec<String>,
    pub errors: Option<Vec<ValidationError>>,
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Validation Report:")?;
        writeln!(f, "  Expression: {}", self.expr)?;
        writeln!(f, "  Valid: {}", self.valid)?;
        writeln!(f, "  Type: {}", self.expr_type)?;
        writeln!(f, "  Time: {:.3}ms", self.elapsed_ms)?;
        writeln!(f, "  Checks: {:?}", self.checks_passed)?;

        if let Some(ref errors) = self.errors {
            writeln!(f, "  Errors: {}", errors.len())?;
            for e in errors {
                writeln!(f, "    - {}", e)?;
            }
        }

        Ok(())
    }
}

/// Main entry point for NanoCheck L0 validation
pub struct NanoChecker {
    validator: QuickValidator,
}

impl NanoChecker {
    pub fn new() -> Self {
        Self {
            validator: QuickValidator::new(),
        }
    }

    /// Full validation with report - target <1ms
    pub fn check(&self, expr: &str) -> ValidationResult<ValidationReport> {
        let report = self.validator.validate_incremental(expr);

        if report.valid {
            Ok(report)
        } else {
            Err(report
                .errors
                .as_ref()
                .unwrap()
                .first()
                .cloned()
                .unwrap_or_else(|| ValidationError::new(ErrorCode::E001, "Unknown error")))
        }
    }

    /// Fast check - parser only
    pub fn quick(&self, expr: &str) -> ValidationResult<()> {
        self.validator.quick_check(expr)
    }

    /// Full validation with all errors (non-failing)
    pub fn check_all(&self, expr: &str) -> ValidationReport {
        self.validator.validate_incremental(expr)
    }
}

impl Default for NanoChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_validator() {
        let v = QuickValidator::new();
        assert!(v.quick_check("1+2").is_ok());
        assert!(v.quick_check("").is_err());
    }

    #[test]
    fn test_full_validation() {
        let v = QuickValidator::new();
        let result = v.validate("1+2*3");
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(
            report.elapsed_ms < 1.0,
            "Should be <1ms, was {}ms",
            report.elapsed_ms
        );
    }

    #[test]
    fn test_incremental_validation() {
        let v = QuickValidator::new();
        let report = v.validate_incremental("sqrt(-1)");
        assert!(!report.valid);
        assert!(report.errors.is_some());
    }

    #[test]
    fn test_nano_checker() {
        let nc = NanoChecker::new();
        let report = nc.check_all("1+2");
        assert!(report.valid);
    }

    #[test]
    fn test_performance() {
        let nc = NanoChecker::new();

        for _ in 0..1000 {
            let report = nc.check_all("1+2*3/4-5+6*7-8+9");
            assert!(report.valid);
        }

        // Should still be fast
        let report = nc.check("1+2").unwrap();
        assert!(
            report.elapsed_ms < 1.0,
            "Validation took {}ms",
            report.elapsed_ms
        );
    }
}
