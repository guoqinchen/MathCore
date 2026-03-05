//! Symbolic computation engine

use std::collections::HashMap;
use std::fmt;

/// Symbolic error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Simplification failed: {0}")]
    SimplificationFailed(String),

    #[error("Differentiation failed: {0}")]
    DifferentiationFailed(String),

    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Represents a symbolic expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric constant
    Const(f64),
    /// A variable (symbol)
    Var(String),
    /// Addition
    Add(Box<Expr>, Box<Expr>),
    /// Subtraction
    Sub(Box<Expr>, Box<Expr>),
    /// Multiplication
    Mul(Box<Expr>, Box<Expr>),
    /// Division
    Div(Box<Expr>, Box<Expr>),
    /// Power (exponentiation)
    Pow(Box<Expr>, Box<Expr>),
    /// Negation (unary minus)
    Neg(Box<Expr>),
    /// Square root
    Sqrt(Box<Expr>),
    /// Sine function
    Sin(Box<Expr>),
    /// Cosine function
    Cos(Box<Expr>),
    /// Tangent function
    Tan(Box<Expr>),
    /// Logarithm (base 10)
    Log(Box<Expr>),
    /// Natural logarithm
    Ln(Box<Expr>),
    /// Exponential
    Exp(Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Const(n) => write!(f, "{}", n),
            Expr::Var(v) => write!(f, "{}", v),
            Expr::Add(l, r) => write!(f, "({} + {})", l, r),
            Expr::Sub(l, r) => write!(f, "({} - {})", l, r),
            Expr::Mul(l, r) => write!(f, "({} * {})", l, r),
            Expr::Div(l, r) => write!(f, "({} / {})", l, r),
            Expr::Pow(b, e) => write!(f, "({})^{}", b, e),
            Expr::Neg(e) => write!(f, "(-{})", e),
            Expr::Sqrt(e) => write!(f, "sqrt({})", e),
            Expr::Sin(e) => write!(f, "sin({})", e),
            Expr::Cos(e) => write!(f, "cos({})", e),
            Expr::Tan(e) => write!(f, "tan({})", e),
            Expr::Log(e) => write!(f, "log({})", e),
            Expr::Ln(e) => write!(f, "ln({})", e),
            Expr::Exp(e) => write!(f, "exp({})", e),
        }
    }
}

/// Token for parsing
#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
    Eof,
}

/// Tokenizer
struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if !ch.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    fn read_number(&mut self) -> f64 {
        let mut num_str = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_numeric() || ch == '.' {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        num_str.parse().unwrap_or(0.0)
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.advance() {
            None => Token::Eof,
            Some(ch) => match ch {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '^' => Token::Caret,
                '(' => Token::LParen,
                ')' => Token::RParen,
                ',' => Token::Comma,
                c if c.is_numeric() => {
                    self.pos -= 1;
                    Token::Number(self.read_number())
                }
                c if c.is_alphabetic() || c == '_' => {
                    self.pos -= 1;
                    let ident = self.read_identifier();
                    Token::Identifier(ident)
                }
                _ => self.next_token(),
            },
        }
    }
}

/// Recursive descent parser
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if matches!(tok, Token::Eof) {
                break;
            }
            tokens.push(tok);
        }
        tokens.push(Token::Eof);
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<Token, Error> {
        let tok = self.advance();
        if std::mem::discriminant(&tok) == std::mem::discriminant(&expected) {
            Ok(tok)
        } else {
            Err(Error::Parse(format!(
                "Expected {:?}, got {:?}",
                expected, tok
            )))
        }
    }

    // expression = term (('+' | '-') term)*
    fn parse_expression(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_term()?;

        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::Add(Box::new(left), Box::new(right));
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // term = unary (('*' | '/') unary)*
    fn parse_term(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_unary()?;

        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::Mul(Box::new(left), Box::new(right));
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::Div(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // unary = ('-')? power
    fn parse_unary(&mut self) -> Result<Expr, Error> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_power()?;
                Ok(Expr::Neg(Box::new(expr)))
            }
            _ => self.parse_power(),
        }
    }

    // power = primary ('^' power)?
    fn parse_power(&mut self) -> Result<Expr, Error> {
        let base = self.parse_primary()?;

        match self.peek() {
            Token::Caret => {
                self.advance();
                let exp = self.parse_power()?; // Right-associative
                Ok(Expr::Pow(Box::new(base), Box::new(exp)))
            }
            _ => Ok(base),
        }
    }

    // primary = number | identifier | function | '(' expression ')'
    fn parse_primary(&mut self) -> Result<Expr, Error> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Const(n))
            }
            Token::Identifier(name) => {
                self.advance();
                match self.peek() {
                    Token::LParen => {
                        // Function call
                        self.advance(); // consume '('
                        let arg = self.parse_expression()?;
                        self.expect(Token::RParen)?;
                        match name.as_str() {
                            "sqrt" => Ok(Expr::Sqrt(Box::new(arg))),
                            "sin" => Ok(Expr::Sin(Box::new(arg))),
                            "cos" => Ok(Expr::Cos(Box::new(arg))),
                            "tan" => Ok(Expr::Tan(Box::new(arg))),
                            "log" => Ok(Expr::Log(Box::new(arg))),
                            "ln" => Ok(Expr::Ln(Box::new(arg))),
                            "exp" => Ok(Expr::Exp(Box::new(arg))),
                            _ => Err(Error::Parse(format!("Unknown function: {}", name))),
                        }
                    }
                    _ => Ok(Expr::Var(name)),
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(Error::Parse(format!("Unexpected token: {:?}", self.peek()))),
        }
    }
}

/// Parse a string into an expression
pub fn parse(input: &str) -> Result<Expr, Error> {
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression()?;

    // Check for remaining tokens
    if !matches!(parser.peek(), Token::Eof) {
        return Err(Error::Parse(
            "Unexpected tokens at end of input".to_string(),
        ));
    }

    Ok(expr)
}

/// Simplify an expression
pub fn simplify(expr: &Expr) -> Result<Expr, Error> {
    match expr {
        // Constants simplify to themselves
        Expr::Const(_) => Ok(expr.clone()),

        // Variables simplify to themselves
        Expr::Var(_) => Ok(expr.clone()),

        // Simplify negation
        Expr::Neg(e) => {
            let simplified = simplify(e)?;
            match simplified {
                Expr::Const(n) => Ok(Expr::Const(-n)),
                Expr::Neg(inner) => Ok(*inner), // --x = x
                _ => Ok(Expr::Neg(Box::new(simplified))),
            }
        }

        // Simplify addition
        Expr::Add(l, r) => {
            let left = simplify(l)?;
            let right = simplify(r)?;

            // 0 + x = x
            if let Expr::Const(0.0) = &left {
                return Ok(right);
            }
            // x + 0 = x
            if let Expr::Const(0.0) = &right {
                return Ok(left);
            }
            // Const + Const = Const
            if let (Expr::Const(a), Expr::Const(b)) = (&left, &right) {
                return Ok(Expr::Const(a + b));
            }

            Ok(Expr::Add(Box::new(left), Box::new(right)))
        }

        // Simplify subtraction
        Expr::Sub(l, r) => {
            let left = simplify(l)?;
            let right = simplify(r)?;

            // x - 0 = x
            if let Expr::Const(0.0) = &right {
                return Ok(left);
            }
            // x - x = 0
            if left == right {
                return Ok(Expr::Const(0.0));
            }
            // Const - Const = Const
            if let (Expr::Const(a), Expr::Const(b)) = (&left, &right) {
                return Ok(Expr::Const(a - b));
            }

            Ok(Expr::Sub(Box::new(left), Box::new(right)))
        }

        // Simplify multiplication
        Expr::Mul(l, r) => {
            let left = simplify(l)?;
            let right = simplify(r)?;

            // 0 * x = 0
            if let Expr::Const(0.0) = &left {
                return Ok(Expr::Const(0.0));
            }
            // x * 0 = 0
            if let Expr::Const(0.0) = &right {
                return Ok(Expr::Const(0.0));
            }
            // 1 * x = x
            if let Expr::Const(1.0) = &left {
                return Ok(right);
            }
            // x * 1 = x
            if let Expr::Const(1.0) = &right {
                return Ok(left);
            }
            // -1 * x = -x
            if let Expr::Const(-1.0) = &left {
                return Ok(Expr::Neg(Box::new(right)));
            }
            // x * -1 = -x
            if let Expr::Const(-1.0) = &right {
                return Ok(Expr::Neg(Box::new(left)));
            }
            // Const * Const = Const
            if let (Expr::Const(a), Expr::Const(b)) = (&left, &right) {
                return Ok(Expr::Const(a * b));
            }

            // x * x = x^2
            if left == right {
                return Ok(Expr::Pow(Box::new(left), Box::new(Expr::Const(2.0))));
            }

            Ok(Expr::Mul(Box::new(left), Box::new(right)))
        }

        // Simplify division
        Expr::Div(l, r) => {
            let left = simplify(l)?;
            let right = simplify(r)?;

            // x / 1 = x
            if let Expr::Const(1.0) = &right {
                return Ok(left);
            }
            // 0 / x = 0
            if let Expr::Const(0.0) = &left {
                return Ok(Expr::Const(0.0));
            }
            // x / x = 1
            if left == right {
                return Ok(Expr::Const(1.0));
            }
            // Const / Const = Const
            if let (Expr::Const(a), Expr::Const(b)) = (&left, &right) {
                if *b != 0.0 {
                    return Ok(Expr::Const(a / b));
                }
            }

            Ok(Expr::Div(Box::new(left), Box::new(right)))
        }

        // Simplify power
        Expr::Pow(b, e) => {
            let base = simplify(b)?;
            let exp = simplify(e)?;

            // x^0 = 1
            if let Expr::Const(0.0) = &exp {
                return Ok(Expr::Const(1.0));
            }
            // x^1 = x
            if let Expr::Const(1.0) = &exp {
                return Ok(base);
            }
            // 1^x = 1
            if let Expr::Const(1.0) = &base {
                return Ok(Expr::Const(1.0));
            }
            // 0^x = 0 (for x > 0)
            if let Expr::Const(0.0) = &base {
                if let Expr::Const(n) = &exp {
                    if *n > 0.0 {
                        return Ok(Expr::Const(0.0));
                    }
                }
            }
            // Const^Const = Const
            if let (Expr::Const(a), Expr::Const(b)) = (&base, &exp) {
                if *a >= 0.0 || b.fract() == 0.0 {
                    return Ok(Expr::Const(a.powf(*b)));
                }
            }

            Ok(Expr::Pow(Box::new(base), Box::new(exp)))
        }

        // Simplify functions
        Expr::Sqrt(e) => {
            let inner = simplify(e)?;
            if let Expr::Const(n) = &inner {
                if *n >= 0.0 {
                    return Ok(Expr::Const(n.sqrt()));
                }
            }
            Ok(Expr::Sqrt(Box::new(inner)))
        }
        Expr::Sin(e) => {
            let inner = simplify(e)?;
            if let Expr::Const(n) = &inner {
                return Ok(Expr::Const(n.sin()));
            }
            Ok(Expr::Sin(Box::new(inner)))
        }
        Expr::Cos(e) => {
            let inner = simplify(e)?;
            if let Expr::Const(n) = &inner {
                return Ok(Expr::Const(n.cos()));
            }
            Ok(Expr::Cos(Box::new(inner)))
        }
        Expr::Tan(e) => {
            let inner = simplify(e)?;
            if let Expr::Const(n) = &inner {
                return Ok(Expr::Const(n.tan()));
            }
            Ok(Expr::Tan(Box::new(inner)))
        }
        Expr::Log(e) => {
            let inner = simplify(e)?;
            if let Expr::Const(n) = &inner {
                if *n > 0.0 {
                    return Ok(Expr::Const(n.log10()));
                }
            }
            Ok(Expr::Log(Box::new(inner)))
        }
        Expr::Ln(e) => {
            let inner = simplify(e)?;
            if let Expr::Const(n) = &inner {
                if *n > 0.0 {
                    return Ok(Expr::Const(n.ln()));
                }
            }
            Ok(Expr::Ln(Box::new(inner)))
        }
        Expr::Exp(e) => {
            let inner = simplify(e)?;
            if let Expr::Const(n) = &inner {
                return Ok(Expr::Const(n.exp()));
            }
            Ok(Expr::Exp(Box::new(inner)))
        }
    }
}

/// Differentiate an expression with respect to a variable
pub fn differentiate(expr: &Expr, var: &str) -> Result<Expr, Error> {
    match expr {
        // d/dx(c) = 0
        Expr::Const(_) => Ok(Expr::Const(0.0)),

        // d/dx(x) = 1
        Expr::Var(v) => {
            if v == var {
                Ok(Expr::Const(1.0))
            } else {
                Ok(Expr::Const(0.0))
            }
        }

        // d/dx(-f) = -f'
        Expr::Neg(e) => {
            let deriv = differentiate(e, var)?;
            Ok(Expr::Neg(Box::new(deriv)))
        }

        // d/dx(f + g) = f' + g'
        Expr::Add(l, r) => {
            let dl = differentiate(l, var)?;
            let dr = differentiate(r, var)?;
            Ok(Expr::Add(Box::new(dl), Box::new(dr)))
        }

        // d/dx(f - g) = f' - g'
        Expr::Sub(l, r) => {
            let dl = differentiate(l, var)?;
            let dr = differentiate(r, var)?;
            Ok(Expr::Sub(Box::new(dl), Box::new(dr)))
        }

        // d/dx(f * g) = f' * g + f * g'
        Expr::Mul(l, r) => {
            let dl = differentiate(l, var)?;
            let dr = differentiate(r, var)?;
            let left = Expr::Mul(Box::new(dl), Box::new((**r).clone()));
            let right = Expr::Mul(Box::new((**l).clone()), Box::new(dr));
            Ok(Expr::Add(Box::new(left), Box::new(right)))
        }

        // d/dx(f / g) = (f' * g - f * g') / g^2
        Expr::Div(l, r) => {
            let dl = differentiate(l, var)?;
            let dr = differentiate(r, var)?;
            let left = Expr::Mul(Box::new(dl), Box::new((**r).clone()));
            let right = Expr::Mul(Box::new((**l).clone()), Box::new(dr));
            let numerator = Expr::Sub(Box::new(left), Box::new(right));
            let denominator = Expr::Pow(Box::new((**r).clone()), Box::new(Expr::Const(2.0)));
            Ok(Expr::Div(Box::new(numerator), Box::new(denominator)))
        }

        // d/dx(f^g) - handle special cases
        Expr::Pow(b, e) => {
            let db = differentiate(b, var)?;
            let de = differentiate(e, var)?;

            // Case: x^n (constant exponent)
            if let Expr::Const(n) = e.as_ref() {
                if !contains_var(b, var) {
                    // f^x where f doesn't contain x: f^x * ln(f) * dx
                    let ln_b = Expr::Ln(Box::new((**b).clone()));
                    let pow = Expr::Pow(Box::new((**b).clone()), Box::new((**e).clone()));
                    let mul1 = Expr::Mul(Box::new(pow), Box::new(ln_b));
                    return Ok(Expr::Mul(Box::new(mul1), Box::new(de)));
                } else {
                    // x^n
                    let new_exp = Expr::Const(n - 1.0);
                    let coeff = Expr::Const(*n);
                    let pow = Expr::Pow(Box::new((**b).clone()), Box::new(new_exp));
                    let mul1 = Expr::Mul(Box::new(coeff), Box::new(pow));
                    return Ok(Expr::Mul(Box::new(mul1), Box::new(db)));
                }
            }

            // General case: f^g * (g' * ln(f) + g * f'/f)
            let ln_b = Expr::Ln(Box::new((**b).clone()));
            let term1 = Expr::Mul(Box::new(de), Box::new(ln_b));
            let div = Expr::Div(Box::new(db), Box::new((**b).clone()));
            let term2 = Expr::Mul(Box::new((**e).clone()), Box::new(div));
            let inner = Expr::Add(Box::new(term1), Box::new(term2));
            let pow = Expr::Pow(Box::new((**b).clone()), Box::new((**e).clone()));
            Ok(Expr::Mul(Box::new(pow), Box::new(inner)))
        }

        // d/dx(sqrt(f)) = f' / (2 * sqrt(f))
        Expr::Sqrt(e) => {
            let de = differentiate(e, var)?;
            let sqrt_e = Expr::Sqrt(Box::new((**e).clone()));
            let denom = Expr::Mul(Box::new(Expr::Const(2.0)), Box::new(sqrt_e));
            Ok(Expr::Div(Box::new(de), Box::new(denom)))
        }

        // d/dx(sin(f)) = cos(f) * f'
        Expr::Sin(e) => {
            let de = differentiate(e, var)?;
            let cos_e = Expr::Cos(Box::new((**e).clone()));
            Ok(Expr::Mul(Box::new(cos_e), Box::new(de)))
        }

        // d/dx(cos(f)) = -sin(f) * f'
        Expr::Cos(e) => {
            let de = differentiate(e, var)?;
            let sin_e = Expr::Sin(Box::new((**e).clone()));
            let neg_sin = Expr::Neg(Box::new(sin_e));
            Ok(Expr::Mul(Box::new(neg_sin), Box::new(de)))
        }

        // d/dx(tan(f)) = sec^2(f) * f'
        Expr::Tan(e) => {
            let de = differentiate(e, var)?;
            let cos_e = Expr::Cos(Box::new((**e).clone()));
            let sec2 = Expr::Pow(Box::new(cos_e), Box::new(Expr::Const(-2.0)));
            Ok(Expr::Mul(Box::new(sec2), Box::new(de)))
        }

        // d/dx(log(f)) = f' / (f * ln(10))
        Expr::Log(e) => {
            let de = differentiate(e, var)?;
            let denom = Expr::Mul(
                Box::new((**e).clone()),
                Box::new(Expr::Const(10.0_f64.ln())),
            );
            Ok(Expr::Div(Box::new(de), Box::new(denom)))
        }

        // d/dx(ln(f)) = f' / f
        Expr::Ln(e) => {
            let de = differentiate(e, var)?;
            Ok(Expr::Div(Box::new(de), Box::new((**e).clone())))
        }

        // d/dx(exp(f)) = exp(f) * f'
        Expr::Exp(e) => {
            let de = differentiate(e, var)?;
            let exp_e = Expr::Exp(Box::new((**e).clone()));
            Ok(Expr::Mul(Box::new(exp_e), Box::new(de)))
        }
    }
}

/// Check if expression contains a variable
fn contains_var(expr: &Expr, var: &str) -> bool {
    match expr {
        Expr::Const(_) => false,
        Expr::Var(v) => v == var,
        Expr::Neg(e)
        | Expr::Sqrt(e)
        | Expr::Sin(e)
        | Expr::Cos(e)
        | Expr::Tan(e)
        | Expr::Log(e)
        | Expr::Ln(e)
        | Expr::Exp(e) => contains_var(e, var),
        Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) | Expr::Pow(l, r) => {
            contains_var(l, var) || contains_var(r, var)
        }
    }
}

/// Evaluate an expression with variable bindings
pub fn evaluate(expr: &Expr, vars: &HashMap<String, f64>) -> Result<f64, Error> {
    match expr {
        Expr::Const(n) => Ok(*n),
        Expr::Var(name) => vars
            .get(name)
            .copied()
            .ok_or_else(|| Error::EvaluationFailed(format!("Unknown variable: {}", name))),
        Expr::Neg(e) => {
            let val = evaluate(e, vars)?;
            Ok(-val)
        }
        Expr::Add(l, r) => {
            let a = evaluate(l, vars)?;
            let b = evaluate(r, vars)?;
            Ok(a + b)
        }
        Expr::Sub(l, r) => {
            let a = evaluate(l, vars)?;
            let b = evaluate(r, vars)?;
            Ok(a - b)
        }
        Expr::Mul(l, r) => {
            let a = evaluate(l, vars)?;
            let b = evaluate(r, vars)?;
            Ok(a * b)
        }
        Expr::Div(l, r) => {
            let a = evaluate(l, vars)?;
            let b = evaluate(r, vars)?;
            if b == 0.0 {
                return Err(Error::EvaluationFailed("Division by zero".to_string()));
            }
            Ok(a / b)
        }
        Expr::Pow(b, e) => {
            let base = evaluate(b, vars)?;
            let exp = evaluate(e, vars)?;
            Ok(base.powf(exp))
        }
        Expr::Sqrt(e) => {
            let val = evaluate(e, vars)?;
            if val < 0.0 {
                return Err(Error::EvaluationFailed(
                    "Square root of negative number".to_string(),
                ));
            }
            Ok(val.sqrt())
        }
        Expr::Sin(e) => {
            let val = evaluate(e, vars)?;
            Ok(val.sin())
        }
        Expr::Cos(e) => {
            let val = evaluate(e, vars)?;
            Ok(val.cos())
        }
        Expr::Tan(e) => {
            let val = evaluate(e, vars)?;
            Ok(val.tan())
        }
        Expr::Log(e) => {
            let val = evaluate(e, vars)?;
            if val <= 0.0 {
                return Err(Error::EvaluationFailed(
                    "Logarithm of non-positive number".to_string(),
                ));
            }
            Ok(val.log10())
        }
        Expr::Ln(e) => {
            let val = evaluate(e, vars)?;
            if val <= 0.0 {
                return Err(Error::EvaluationFailed(
                    "Natural log of non-positive number".to_string(),
                ));
            }
            Ok(val.ln())
        }
        Expr::Exp(e) => {
            let val = evaluate(e, vars)?;
            Ok(val.exp())
        }
    }
}

/// Symbolic computation engine
pub struct SymbolicEngine;

impl SymbolicEngine {
    /// Create a new symbolic engine
    pub fn new() -> Self {
        Self
    }

    /// Parse an expression string
    pub fn parse(&self, input: &str) -> Result<Expr, Error> {
        parse(input)
    }

    /// Simplify an expression
    pub fn simplify(&self, expr: &Expr) -> Result<Expr, Error> {
        simplify(expr)
    }

    /// Differentiate an expression
    pub fn differentiate(&self, expr: &Expr, var: &str) -> Result<Expr, Error> {
        differentiate(expr, var)
    }

    /// Evaluate an expression with variable bindings
    pub fn evaluate(&self, expr: &Expr, vars: &HashMap<String, f64>) -> Result<f64, Error> {
        evaluate(expr, vars)
    }
}

impl Default for SymbolicEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_constant() {
        let expr = parse("42").unwrap();
        assert_eq!(expr, Expr::Const(42.0));
    }

    #[test]
    fn test_parse_variable() {
        let expr = parse("x").unwrap();
        assert_eq!(expr, Expr::Var("x".to_string()));
    }

    #[test]
    fn test_simplify_add_zero() {
        let expr = parse("x + 0").unwrap();
        let simplified = simplify(&expr).unwrap();
        assert_eq!(simplified, Expr::Var("x".to_string()));
    }

    #[test]
    fn test_differentiate_variable() {
        let expr = parse("x").unwrap();
        let deriv = differentiate(&expr, "x").unwrap();
        assert_eq!(deriv, Expr::Const(1.0));
    }

    #[test]
    fn test_evaluate() {
        let expr = parse("x^2 + 2*x + 1").unwrap();
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3.0);
        let result = evaluate(&expr, &vars).unwrap();
        assert!((result - 16.0).abs() < 1e-10);
    }
}
