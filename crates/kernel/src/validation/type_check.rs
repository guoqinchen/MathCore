// Type checking for expressions
use super::types::*;
use super::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unknown,
    Integer,
    Real,
    Complex,
    Matrix { rows: usize, cols: usize },
    Vector { len: usize },
    Boolean,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unknown => write!(f, "Unknown"),
            Type::Integer => write!(f, "Integer"),
            Type::Real => write!(f, "Real"),
            Type::Complex => write!(f, "Complex"),
            Type::Matrix { rows, cols } => write!(f, "Matrix({}×{})", rows, cols),
            Type::Vector { len } => write!(f, "Vector({})", len),
            Type::Boolean => write!(f, "Boolean"),
        }
    }
}

pub struct TypeChecker;

impl TypeChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn check(&self, expr: &str) -> ValidationResult<Type> {
        self.infer_type(expr.trim())
    }

    fn infer_type(&self, expr: &str) -> ValidationResult<Type> {
        let expr = expr.replace(" ", "");

        if expr.is_empty() {
            return Err(err::empty_expr());
        }

        // Handle numbers
        if Self::is_number(&expr) {
            return Ok(if expr.contains('.') || expr.contains('e') {
                Type::Real
            } else {
                Type::Integer
            });
        }

        // Handle complex numbers (e.g., 3+4i)
        if expr.ends_with('i') && !expr.eq("i") {
            return Ok(Type::Complex);
        }

        // Handle parentheses - infer from inner
        if expr.starts_with('(') && expr.ends_with(')') {
            let inner = &expr[1..expr.len() - 1];
            return self.infer_type(inner);
        }

        // Handle functions
        if let Some(func_end) = expr.find('(') {
            let func = &expr[..func_end];
            return self.infer_function_type(func);
        }

        // Handle binary operations
        if let Some(op_pos) = Self::find_main_op(&expr) {
            let left = &expr[..op_pos];
            let right = &expr[op_pos + 1..];
            let op = &expr[op_pos..op_pos + 1];

            let left_type = self.infer_type(left)?;
            let right_type = self.infer_type(right)?;

            return self.binary_op_type(&left_type, &right_type, op);
        }

        // Unknown symbol
        Ok(Type::Unknown)
    }

    fn is_number(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E')
            && !s.is_empty()
    }

    fn find_main_op(expr: &str) -> Option<usize> {
        let mut depth = 0;
        let ops = ['+', '-', '*', '/', '^', '%'];

        for (i, c) in expr.char_indices().rev() {
            match c {
                ')' => depth += 1,
                '(' => depth -= 1,
                _ if depth == 0 && ops.contains(&c) => {
                    // Don't return first char if it's unary minus
                    if c == '-' && i == 0 {
                        continue;
                    }
                    return Some(i);
                }
                _ => {}
            }
        }
        None
    }

    fn infer_function_type(&self, func: &str) -> ValidationResult<Type> {
        match func.to_lowercase().as_str() {
            "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sinh" | "cosh" | "tanh" | "exp"
            | "log" | "ln" | "log10" | "log2" | "sqrt" | "abs" => Ok(Type::Real),

            "sinv" | "cosv" | "tanv" => Ok(Type::Vector { len: 3 }),

            "matrix" | "mat" => Ok(Type::Matrix { rows: 0, cols: 0 }),

            "vector" | "vec" => Ok(Type::Vector { len: 0 }),

            "len" | "size" => Ok(Type::Integer),

            "det" | "inv" | "transpose" => Ok(Type::Matrix { rows: 0, cols: 0 }),

            "dot" | "cross" => Ok(Type::Real),

            _ => Ok(Type::Unknown),
        }
    }

    fn binary_op_type(&self, left: &Type, right: &Type, op: &str) -> ValidationResult<Type> {
        match op {
            "+" | "-" | "*" | "/" | "%" => {
                // Matrix operations
                if let (Type::Matrix { .. }, Type::Matrix { .. }) = (left, right) {
                    return Ok(Type::Matrix { rows: 0, cols: 0 });
                }
                if let (Type::Vector { .. }, Type::Vector { .. }) = (left, right) {
                    return Ok(Type::Vector { len: 0 });
                }

                // Scalar operations
                if left == right {
                    return Ok(left.clone());
                }

                // Type promotion
                if left == &Type::Integer && right == &Type::Real {
                    return Ok(Type::Real);
                }
                if left == &Type::Real && right == &Type::Integer {
                    return Ok(Type::Real);
                }

                // Complex promotion
                if left == &Type::Complex || right == &Type::Complex {
                    return Ok(Type::Complex);
                }

                Err(err::type_mismatch(&left.to_string(), &right.to_string()))
            }
            "^" => Ok(Type::Real),
            _ => Ok(Type::Unknown),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_types() {
        let tc = TypeChecker::new();
        assert_eq!(tc.check("42").unwrap(), Type::Integer);
        assert_eq!(tc.check("3.14").unwrap(), Type::Real);
        assert_eq!(tc.check("1e10").unwrap(), Type::Real);
    }

    #[test]
    fn test_complex() {
        let tc = TypeChecker::new();
        assert_eq!(tc.check("3+4i").unwrap(), Type::Complex);
    }

    #[test]
    fn test_functions() {
        let tc = TypeChecker::new();
        assert_eq!(tc.check("sin(x)").unwrap(), Type::Real);
        assert_eq!(tc.check("sqrt(x)").unwrap(), Type::Real);
    }

    #[test]
    fn test_operations() {
        let tc = TypeChecker::new();
        assert_eq!(tc.check("1+2").unwrap(), Type::Integer);
        assert_eq!(tc.check("1.0+2").unwrap(), Type::Real);
    }
}
