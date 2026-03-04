// Expression parser validation
use super::types::*;

pub struct ParserValidator;

impl ParserValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, expr: &str) -> ValidationResult<()> {
        self.validate_empty(expr)?;
        self.validate_chars(expr)?;
        self.validate_parentheses(expr)?;
        self.validate_operators(expr)?;
        self.validate_trailing(expr)?;
        Ok(())
    }

    fn validate_empty(&self, expr: &str) -> ValidationResult<()> {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return Err(err::empty_expr());
        }
        Ok(())
    }

    fn validate_chars(&self, expr: &str) -> ValidationResult<()> {
        for (i, c) in expr.char_indices() {
            if !c.is_whitespace()
                && !c.is_ascii_digit()
                && !Self::is_valid_op(c)
                && c != '('
                && c != ')'
                && c != '.'
                && c != 'i'
                && !c.is_alphabetic()
            {
                return Err(err::invalid_char(c, i + 1));
            }
        }
        Ok(())
    }

    fn is_valid_op(c: char) -> bool {
        matches!(c, '+' | '-' | '*' | '/' | '^' | '%')
    }

    fn validate_parentheses(&self, expr: &str) -> ValidationResult<()> {
        let mut depth = 0;
        for (i, c) in expr.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(err::unmatched_paren(i + 1));
                    }
                }
                _ => {}
            }
        }
        if depth > 0 {
            return Err(err::unmatched_paren(expr.len()));
        }
        Ok(())
    }

    fn validate_operators(&self, expr: &str) -> ValidationResult<()> {
        let chars: Vec<char> = expr.chars().filter(|c| !c.is_whitespace()).collect();
        let ops = ['+', '-', '*', '/', '^', '%'];

        for i in 0..chars.len().saturating_sub(1) {
            let curr = chars[i];
            let next = chars[i + 1];

            if ops.contains(&curr) && ops.contains(&next) {
                if curr != '-' || next != '-' {
                    return Err(err::consecutive_ops(&curr.to_string(), &next.to_string()));
                }
            }
        }
        Ok(())
    }

    fn validate_trailing(&self, expr: &str) -> ValidationResult<()> {
        let trimmed = expr.trim_end();
        if let Some(c) = trimmed.chars().last() {
            if Self::is_valid_op(c) && c != ')' {
                return Err(err::trailing_op(&c.to_string()));
            }
        }
        Ok(())
    }
}

impl Default for ParserValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_expression() {
        let v = ParserValidator::new();
        assert!(v.validate("").is_err());
        assert!(v.validate("   ").is_err());
    }

    #[test]
    fn test_valid_expression() {
        let v = ParserValidator::new();
        assert!(v.validate("1+2*3").is_ok());
        assert!(v.validate("(1+2)*3").is_ok());
        assert!(v.validate("sqrt(4) + sin(0)").is_ok());
    }

    #[test]
    fn test_unmatched_parentheses() {
        let v = ParserValidator::new();
        assert!(v.validate("(1+2").is_err());
        assert!(v.validate("1+2)").is_err());
    }

    #[test]
    fn test_consecutive_operators() {
        let v = ParserValidator::new();
        assert!(v.validate("1++2").is_err());
        assert!(v.validate("1*/2").is_err());
    }

    #[test]
    fn test_trailing_operator() {
        let v = ParserValidator::new();
        assert!(v.validate("1+2+").is_err());
        assert!(v.validate("1+2*").is_err());
    }

    #[test]
    fn test_invalid_characters() {
        let v = ParserValidator::new();
        assert!(v.validate("1@2").is_err());
    }

    #[test]
    fn test_complex_expression() {
        let v = ParserValidator::new();
        assert!(v.validate("((x + y) * (z - w)) / 2").is_ok());
        assert!(v.validate("sin(x)^2 + cos(x)^2").is_ok());
    }
}
