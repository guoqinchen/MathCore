//! SMT-LIB Interface Module
//!
//! Provides SMT-LIB 2.6 compatible interface.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmtLibError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Unsupported command: {0}")]
    UnsupportedCommand(String),
    #[error("Evaluation error: {0}")]
    EvaluationError(String),
}

/// SMT-LIB 2.6 command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmtCommand {
    /// Declare sort
    DeclareSort { symbol: String, arity: usize },
    /// Define sort
    DefineSort {
        symbol: String,
        params: Vec<String>,
        sort: String,
    },
    /// Declare function
    DeclareFun {
        symbol: String,
        params: Vec<String>,
        sort: String,
    },
    /// Define function
    DefineFun {
        symbol: String,
        params: Vec<VarDecl>,
        sort: String,
        body: String,
    },
    /// Assert
    Assert { term: String },
    /// Check sat
    CheckSat,
    /// Get model
    GetModel,
    /// Get value
    GetValue { terms: Vec<String> },
    /// Push
    Push { level: usize },
    /// Pop
    Pop { level: usize },
    /// Exit
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarDecl {
    pub name: String,
    pub sort: String,
}

/// SMT-LIB script representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SmtScript {
    pub commands: Vec<SmtCommand>,
}

impl SmtScript {
    /// Create a new empty script
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a command to the script
    pub fn add_command(&mut self, cmd: SmtCommand) {
        self.commands.push(cmd);
    }

    /// Convert to SMT-LIB string format
    pub fn to_smt_lib_string(&self) -> String {
        let mut output = String::new();
        output.push_str("(set-option :print-success true)\n");
        output.push_str("(set-option :produce-models true)\n");

        for cmd in &self.commands {
            output.push_str(&self.command_to_string(cmd));
            output.push('\n');
        }

        output
    }

    fn command_to_string(&self, cmd: &SmtCommand) -> String {
        match cmd {
            SmtCommand::DeclareSort { symbol, arity } => {
                format!("(declare-sort {} {})", symbol, arity)
            }
            SmtCommand::DefineSort {
                symbol,
                params,
                sort,
            } => {
                if params.is_empty() {
                    format!("(define-sort {} {})", symbol, sort)
                } else {
                    format!("(define-sort ({} {}) {})", symbol, params.join(" "), sort)
                }
            }
            SmtCommand::DeclareFun {
                symbol,
                params,
                sort,
            } => {
                if params.is_empty() {
                    format!("(declare-fun {} ( ) {})", symbol, sort)
                } else {
                    format!("(declare-fun {} ({}) {})", symbol, params.join(" "), sort)
                }
            }
            SmtCommand::DefineFun {
                symbol,
                params,
                sort,
                body,
            } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("({} {})", p.name, p.sort))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("(define-fun {} ({}) {} {})", symbol, params_str, sort, body)
            }
            SmtCommand::Assert { term } => {
                format!("(assert {})", term)
            }
            SmtCommand::CheckSat => "(check-sat)".to_string(),
            SmtCommand::GetModel => "(get-model)".to_string(),
            SmtCommand::GetValue { terms } => {
                format!("(get-value ({}))", terms.join(" "))
            }
            SmtCommand::Push { level } => {
                format!("(push {})", level)
            }
            SmtCommand::Pop { level } => {
                format!("(pop {})", level)
            }
            SmtCommand::Exit => "(exit)".to_string(),
        }
    }

    /// Parse SMT-LIB string to script
    pub fn from_smt_lib_string(input: &str) -> Result<Self, SmtLibError> {
        let mut script = Self::new();

        // Simple parser for demonstration
        // In production, use a proper parser
        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Parse basic commands
            if line.contains("(check-sat)") {
                script.add_command(SmtCommand::CheckSat);
            } else if line.contains("(get-model)") {
                script.add_command(SmtCommand::GetModel);
            } else if line.contains("(exit)") {
                script.add_command(SmtCommand::Exit);
            } else if line.contains("(push") {
                // Parse push level
                if let Some(level) = line.split_whitespace().nth(1) {
                    let level = level.trim_end_matches(')').parse().unwrap_or(1);
                    script.add_command(SmtCommand::Push { level });
                }
            } else if line.contains("(pop") {
                if let Some(level) = line.split_whitespace().nth(1) {
                    let level = level.trim_end_matches(')').parse().unwrap_or(1);
                    script.add_command(SmtCommand::Pop { level });
                }
            }
        }

        Ok(script)
    }
}

/// SMT-LIB result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmtResult {
    Sat,
    Unsat,
    Unknown,
    Model(String),
    Value(String),
    Success,
    Error(String),
}

impl SmtResult {
    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "sat" => SmtResult::Sat,
            "unsat" => SmtResult::Unsat,
            "unknown" => SmtResult::Unknown,
            _ => SmtResult::Value(s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_creation() {
        let mut script = SmtScript::new();
        script.add_command(SmtCommand::DeclareFun {
            symbol: "x".to_string(),
            params: vec![],
            sort: "Int".to_string(),
        });
        script.add_command(SmtCommand::Assert {
            term: "(> x 0)".to_string(),
        });
        script.add_command(SmtCommand::CheckSat);

        let output = script.to_smt_lib_string();
        assert!(output.contains("(declare-fun"));
        assert!(output.contains("(assert"));
        assert!(output.contains("(check-sat)"));
    }

    #[test]
    fn test_script_parsing() {
        let input = r#"
            (push 1)
            (declare-fun x () Int)
            (assert (> x 0))
            (check-sat)
            (pop 1)
        "#;

        let script = SmtScript::from_smt_lib_string(input).unwrap();
        assert_eq!(script.commands.len(), 3);
    }
}
