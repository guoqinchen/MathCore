//! Shared lexer module for expression parsing
//!
//! This module provides a generic lexer implementation that can be used by both
//! symbolic and numeric expression parsers. It eliminates code duplication by
//! providing a single lexer implementation that handles common tokenization
//! needs for mathematical expressions.

use std::fmt;

/// Token types for mathematical expression parsing
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Numeric literal
    Number(f64),
    /// Identifier (variable or function name)
    Identifier(String),
    /// Addition operator
    Plus,
    /// Subtraction operator
    Minus,
    /// Multiplication operator
    Star,
    /// Division operator
    Slash,
    /// Exponentiation operator
    Caret,
    /// Left parenthesis
    LParen,
    /// Right parenthesis
    RParen,
    /// Comma separator
    Comma,
    /// End of input
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::Plus => write!(f, "Plus"),
            Token::Minus => write!(f, "Minus"),
            Token::Star => write!(f, "Star"),
            Token::Slash => write!(f, "Slash"),
            Token::Caret => write!(f, "Caret"),
            Token::LParen => write!(f, "LParen"),
            Token::RParen => write!(f, "RParen"),
            Token::Comma => write!(f, "Comma"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}

/// Lexer for tokenizing mathematical expressions
#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    /// Create a new lexer from an input string
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Peek at the next character without consuming it
    pub fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Advance to the next character and return it
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    /// Skip all whitespace characters
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if !ch.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    /// Read a numeric literal from the input
    pub fn read_number(&mut self) -> f64 {
        let mut num_str = String::with_capacity(32);
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

    /// Read an identifier (variable or function name) from the input
    pub fn read_identifier(&mut self) -> String {
        let mut ident = String::with_capacity(32);
        
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

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
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
                _ => self.next_token(), // Skip invalid characters and continue
            },
        }
    }

    /// Tokenize the entire input into a vector of tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let estimated_capacity = self.input.len() / 2 + 1;
        let mut tokens = Vec::with_capacity(estimated_capacity);
        loop {
            let tok = self.next_token();
            tokens.push(tok.clone());
            if matches!(tok, Token::Eof) {
                break;
            }
        }
        tokens
    }

    /// Get current position for error reporting
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Set position for error reporting
    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() {
        let mut lexer = Lexer::new("2 + 3 * 4");
        assert_eq!(lexer.next_token(), Token::Number(2.0));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Number(3.0));
        assert_eq!(lexer.next_token(), Token::Star);
        assert_eq!(lexer.next_token(), Token::Number(4.0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = Lexer::new("x + sin(y)");
        assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Identifier("sin".to_string()));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Identifier("y".to_string()));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_lexer_functions() {
        let mut lexer = Lexer::new("sqrt(25) + log10(100)");
        assert_eq!(lexer.next_token(), Token::Identifier("sqrt".to_string()));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Number(25.0));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Identifier("log10".to_string()));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Number(100.0));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_lexer_with_whitespace() {
        let mut lexer = Lexer::new("  123  \t\n  +   456  ");
        assert_eq!(lexer.next_token(), Token::Number(123.0));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Number(456.0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}
