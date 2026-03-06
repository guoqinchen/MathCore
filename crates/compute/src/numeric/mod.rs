//! Numeric computation engine

use std::collections::HashMap;
use std::f64::consts::{E, PI};

/// Numeric error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Computation error: {0}")]
    ComputationFailed(String),

    #[error("Overflow: {0}")]
    Overflow(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Convergence failed: {0}")]
    ConvergenceFailed(String),
}

/// Token types for expression parsing
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

/// Simple lexer for numeric expressions
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
        let mut has_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_numeric() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
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

/// Simple recursive descent parser for numeric expressions
struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    vars: &'a HashMap<String, f64>,
}

impl<'a> Parser<'a> {
    fn new(input: &str, vars: &'a HashMap<String, f64>) -> Self {
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
        Self { tokens, pos: 0, vars }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        self.pos += 1;
        tok
    }

    fn expression(&mut self) -> Result<f64, Error> {
        let mut left = self.term()?;

        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.term()?;
                    left += right;
                }
                Token::Minus => {
                    self.advance();
                    let right = self.term()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<f64, Error> {
        let mut left = self.unary()?;

        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.unary()?;
                    left *= right;
                }
                Token::Slash => {
                    self.advance();
                    let right = self.unary()?;
                    if right == 0.0 {
                        return Err(Error::ComputationFailed("Division by zero".to_string()));
                    }
                    left /= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<f64, Error> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.power()?;
                Ok(-expr)
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<f64, Error> {
        let base = self.primary()?;

        match self.peek() {
            Token::Caret => {
                self.advance();
                let exp = self.power()?;
                Ok(base.powf(exp))
            }
            _ => Ok(base),
        }
    }

    fn primary(&mut self) -> Result<f64, Error> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(n)
            }
            Token::Identifier(name) => {
                self.advance();
                match self.peek() {
                    Token::LParen => {
                        self.advance();
                        let arg = self.expression()?;
                        self.advance(); // consume RParen
                        match name.as_str() {
                            "sin" => Ok(arg.sin()),
                            "cos" => Ok(arg.cos()),
                            "tan" => Ok(arg.tan()),
                            "asin" => Ok(arg.asin()),
                            "acos" => Ok(arg.acos()),
                            "atan" => Ok(arg.atan()),
                            "sqrt" => {
                                if arg < 0.0 {
                                    Err(Error::ComputationFailed("sqrt of negative".to_string()))
                                } else {
                                    Ok(arg.sqrt())
                                }
                            }
                            "cbrt" => Ok(arg.cbrt()),
                            "abs" => Ok(arg.abs()),
                            "ln" => {
                                if arg <= 0.0 {
                                    Err(Error::ComputationFailed("ln of non-positive".to_string()))
                                } else {
                                    Ok(arg.ln())
                                }
                            }
                            "log" | "log10" => {
                                if arg <= 0.0 {
                                    Err(Error::ComputationFailed("log of non-positive".to_string()))
                                } else {
                                    Ok(arg.log10())
                                }
                            }
                            "log2" => {
                                if arg <= 0.0 {
                                    Err(Error::ComputationFailed(
                                        "log2 of non-positive".to_string(),
                                    ))
                                } else {
                                    Ok(arg.log2())
                                }
                            }
                            "exp" => Ok(arg.exp()),
                            "pow" => {
                                if let Token::Comma = self.peek() {
                                    self.advance();
                                    let exp = self.expression()?;
                                    self.advance();
                                    Ok(arg.powf(exp))
                                } else {
                                    Err(Error::Parse("Expected comma for pow function".to_string()))
                                }
                            }
                            _ => Err(Error::Parse(format!("Unknown function: {}", name))),
                        }
                    }
                    _ => {
                        // First check if it's a variable in the vars map
                        if let Some(&value) = self.vars.get(&name) {
                            Ok(value)
                        } else {
                            // Handle constants like pi and e
                            match name.as_str() {
                                "pi" => Ok(PI),
                                "e" => Ok(E),
                                _ => Err(Error::Parse(format!("Unknown identifier: {}", name))),
                            }
                        }
                    }
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.expression()?;
                self.advance(); // consume RParen
                Ok(expr)
            }
            _ => Err(Error::Parse(format!("Unexpected token: {:?}", self.peek()))),
        }
    }
}

/// Parse and evaluate an expression string with variable bindings
pub fn eval(input: &str, vars: &HashMap<String, f64>) -> Result<f64, Error> {
    let mut parser = Parser::new(input, vars);
    let result = parser.expression()?;

    if !matches!(parser.peek(), Token::Eof) {
        return Err(Error::Parse("Unexpected tokens".to_string()));
    }

    Ok(result)
}

/// Evaluate a simple expression without variables
pub fn eval_simple(input: &str) -> Result<f64, Error> {
    eval(input, &HashMap::new())
}

/// Numeric differentiation using central difference
pub fn differentiate<F>(f: F, x: f64, h: Option<f64>) -> Result<f64, Error>
where
    F: Fn(f64) -> Result<f64, Error>,
{
    let h = h.unwrap_or(1e-8);
    if h <= 0.0 {
        return Err(Error::InvalidInput(
            "Step size must be positive".to_string(),
        ));
    }

    let f_plus = f(x + h)?;
    let f_minus = f(x - h)?;

    Ok((f_plus - f_minus) / (2.0 * h))
}

/// Numerical differentiation for expressions with variables
pub fn differentiate_expr(
    expr: &str,
    var: &str,
    vars: &HashMap<String, f64>,
    h: Option<f64>,
) -> Result<f64, Error> {
    let h = h.unwrap_or(1e-8);

    let x = vars
        .get(var)
        .ok_or_else(|| Error::InvalidInput(format!("Variable '{}' not found", var)))?;

    let mut vars_plus = vars.clone();
    vars_plus.insert(var.to_string(), x + h);

    let mut vars_minus = vars.clone();
    vars_minus.insert(var.to_string(), x - h);

    let f_plus = eval(expr, &vars_plus)?;
    let f_minus = eval(expr, &vars_minus)?;

    Ok((f_plus - f_minus) / (2.0 * h))
}

/// Numerical integration using Simpson's rule
pub fn integrate_simpson<F>(f: F, a: f64, b: f64, n: Option<usize>) -> Result<f64, Error>
where
    F: Fn(f64) -> Result<f64, Error>,
{
    let n = n.unwrap_or(1000);
    let n = if n % 2 == 0 { n } else { n + 1 };

    if a >= b {
        return Err(Error::InvalidInput(
            "Lower bound must be less than upper bound".to_string(),
        ));
    }

    let h = (b - a) / n as f64;

    let mut sum = f(a)? + f(b)?;

    for i in 1..n {
        let x = a + i as f64 * h;
        let fx = f(x)?;

        if i % 2 == 0 {
            sum += 2.0 * fx;
        } else {
            sum += 4.0 * fx;
        }
    }

    Ok(sum * h / 3.0)
}

/// Numerical integration for expressions
pub fn integrate_expr_simpson(
    expr: &str,
    var: &str,
    a: f64,
    b: f64,
    n: Option<usize>,
) -> Result<f64, Error> {
    let expr_owned = expr.to_string();
    let var_owned = var.to_string();

    let f = move |x: f64| -> Result<f64, Error> {
        let mut vars = HashMap::new();
        vars.insert(var_owned.clone(), x);
        eval(&expr_owned, &vars)
    };

    integrate_simpson(f, a, b, n)
}

/// Bisection method for root finding
pub fn solve_bisection<F>(
    f: F,
    a: f64,
    b: f64,
    tol: Option<f64>,
    max_iter: Option<usize>,
) -> Result<f64, Error>
where
    F: Fn(f64) -> Result<f64, Error>,
{
    let tol = tol.unwrap_or(1e-10);
    let max_iter = max_iter.unwrap_or(100);

    if a >= b {
        return Err(Error::InvalidInput(
            "Lower bound must be less than upper bound".to_string(),
        ));
    }

    let fa = f(a)?;
    let fb = f(b)?;

    if fa * fb > 0.0 {
        return Err(Error::InvalidInput(
            "Function values at bounds must have opposite signs".to_string(),
        ));
    }

    if fa.abs() < tol {
        return Ok(a);
    }
    if fb.abs() < tol {
        return Ok(b);
    }

    let mut low = a;
    let mut high = b;

    for _ in 0..max_iter {
        let mid = (low + high) / 2.0;
        let fmid = f(mid)?;

        if fmid.abs() < tol || (high - low) / 2.0 < tol {
            return Ok(mid);
        }

        if fmid * f(low)? > 0.0 {
            low = mid;
        } else {
            high = mid;
        }
    }

    Err(Error::ConvergenceFailed(
        "Bisection did not converge".to_string(),
    ))
}

/// Solve equation using bisection for expressions
pub fn solve_bisection_expr(
    expr: &str,
    var: &str,
    a: f64,
    b: f64,
    tol: Option<f64>,
    max_iter: Option<usize>,
) -> Result<f64, Error> {
    let expr_owned = expr.to_string();
    let var_owned = var.to_string();

    let f = move |x: f64| -> Result<f64, Error> {
        let mut vars = HashMap::new();
        vars.insert(var_owned.clone(), x);
        eval(&expr_owned, &vars)
    };

    solve_bisection(f, a, b, tol, max_iter)
}

/// Newton's method for root finding
pub fn solve_newton<F, G>(
    f: F,
    df: G,
    x0: f64,
    tol: Option<f64>,
    max_iter: Option<usize>,
) -> Result<f64, Error>
where
    F: Fn(f64) -> Result<f64, Error>,
    G: Fn(f64) -> Result<f64, Error>,
{
    let tol = tol.unwrap_or(1e-10);
    let max_iter = max_iter.unwrap_or(100);

    let mut x = x0;

    for _ in 0..max_iter {
        let fx = f(x)?;
        let dfx = df(x)?;

        if dfx.abs() < tol {
            return Err(Error::ComputationFailed("Derivative too small".to_string()));
        }

        let dx = fx / dfx;
        x -= dx;

        if dx.abs() < tol || fx.abs() < tol {
            return Ok(x);
        }
    }

    Err(Error::ConvergenceFailed(
        "Newton's method did not converge".to_string(),
    ))
}

/// Solve equation using Newton for expressions
pub fn solve_newton_expr(
    expr: &str,
    var: &str,
    x0: f64,
    tol: Option<f64>,
    max_iter: Option<usize>,
) -> Result<f64, Error> {
    let expr_f = expr.to_string();
    let var_f = var.to_string();

    let f = move |x: f64| -> Result<f64, Error> {
        let mut vars = HashMap::new();
        vars.insert(var_f.clone(), x);
        eval(&expr_f, &vars)
    };

    let expr_df = expr.to_string();
    let var_df = var.to_string();
    let df = move |x: f64| -> Result<f64, Error> {
        differentiate_expr(
            &expr_df,
            &var_df,
            &HashMap::from([(var_df.clone(), x)]),
            None,
        )
    };

    solve_newton(f, df, x0, tol, max_iter)
}

/// Fixed-point iteration
pub fn solve_fixed_point<G>(
    g: G,
    x0: f64,
    tol: Option<f64>,
    max_iter: Option<usize>,
) -> Result<f64, Error>
where
    G: Fn(f64) -> Result<f64, Error>,
{
    let tol = tol.unwrap_or(1e-10);
    let max_iter = max_iter.unwrap_or(100);

    let mut x = x0;

    for _ in 0..max_iter {
        let x_new = g(x)?;

        if (x_new - x).abs() < tol {
            return Ok(x_new);
        }

        x = x_new;
    }

    Err(Error::ConvergenceFailed(
        "Fixed-point iteration did not converge".to_string(),
    ))
}

/// Numeric computation engine
pub struct NumericEngine;

impl NumericEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn eval(&self, expr: &str, vars: &HashMap<String, f64>) -> Result<f64, Error> {
        eval(expr, vars)
    }

    pub fn eval_simple(&self, expr: &str) -> Result<f64, Error> {
        eval_simple(expr)
    }

    pub fn differentiate<F>(&self, f: F, x: f64, h: Option<f64>) -> Result<f64, Error>
    where
        F: Fn(f64) -> Result<f64, Error>,
    {
        differentiate(f, x, h)
    }

    pub fn integrate<F>(&self, f: F, a: f64, b: f64, n: Option<usize>) -> Result<f64, Error>
    where
        F: Fn(f64) -> Result<f64, Error>,
    {
        integrate_simpson(f, a, b, n)
    }

    pub fn solve_bisection<F>(
        &self,
        f: F,
        a: f64,
        b: f64,
        tol: Option<f64>,
        max_iter: Option<usize>,
    ) -> Result<f64, Error>
    where
        F: Fn(f64) -> Result<f64, Error>,
    {
        solve_bisection(f, a, b, tol, max_iter)
    }

    pub fn solve_newton<F, G>(
        &self,
        f: F,
        df: G,
        x0: f64,
        tol: Option<f64>,
        max_iter: Option<usize>,
    ) -> Result<f64, Error>
    where
        F: Fn(f64) -> Result<f64, Error>,
        G: Fn(f64) -> Result<f64, Error>,
    {
        solve_newton(f, df, x0, tol, max_iter)
    }
}

impl Default for NumericEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_simple() {
        let result = eval_simple("2 + 3").unwrap();
        assert!((result - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_with_vars() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3.0);
        let result = eval("x^2 + 2*x + 1", &vars).unwrap();
        assert!((result - 16.0).abs() < 1e-10);
    }
}
