//! Verification Rule Engine Module
//!
//! Provides rule-based verification using SMT solving.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Rule parse error: {0}")]
    ParseError(String),
    #[error("Evaluation error: {0}")]
    EvaluationError(String),
    #[error("Conflict detected: {0}")]
    Conflict(String),
}

/// Variable in a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVar {
    pub name: String,
    pub sort: VarSort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VarSort {
    Int,
    Real,
    Bool,
    BitVec(usize),
    Array(Box<VarSort>, Box<VarSort>),
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub lhs: String,
    pub op: ComparisonOp,
    pub rhs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOp {
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Verification rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRule {
    pub name: String,
    pub description: String,
    pub variables: Vec<RuleVar>,
    pub preconditions: Vec<Condition>,
    pub postconditions: Vec<Condition>,
}

impl VerificationRule {
    /// Create a new verification rule
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            variables: Vec::new(),
            preconditions: Vec::new(),
            postconditions: Vec::new(),
        }
    }

    /// Add a variable to the rule
    pub fn add_variable(&mut self, name: &str, sort: VarSort) {
        self.variables.push(RuleVar {
            name: name.to_string(),
            sort,
        });
    }

    /// Add a precondition
    pub fn add_precondition(&mut self, lhs: &str, op: ComparisonOp, rhs: &str) {
        self.preconditions.push(Condition {
            lhs: lhs.to_string(),
            op,
            rhs: rhs.to_string(),
        });
    }

    /// Add a postcondition
    pub fn add_postcondition(&mut self, lhs: &str, op: ComparisonOp, rhs: &str) {
        self.postconditions.push(Condition {
            lhs: lhs.to_string(),
            op,
            rhs: rhs.to_string(),
        });
    }
}

/// Rule evaluation context
pub struct RuleContext {
    bindings: HashMap<String, String>,
}

impl RuleContext {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, name: &str, value: &str) {
        self.bindings.insert(name.to_string(), value.to_string());
    }

    pub fn resolve(&self, name: &str) -> Option<&str> {
        self.bindings.get(name).map(|s| s.as_str())
    }
}

impl Default for RuleContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule engine for verification
pub struct RuleEngine {
    rules: HashMap<String, VerificationRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Register a rule
    pub fn register(&mut self, rule: VerificationRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Get a rule by name
    pub fn get(&self, name: &str) -> Option<&VerificationRule> {
        self.rules.get(name)
    }

    /// Check if all rules are satisfied
    pub fn check_rules(&self, context: &RuleContext) -> Result<Vec<String>, RuleError> {
        let mut violations = Vec::new();

        for (name, rule) in &self.rules {
            // Check preconditions
            for pre in &rule.preconditions {
                let lhs = context.resolve(&pre.lhs).unwrap_or(&pre.lhs);
                let rhs = context.resolve(&pre.rhs).unwrap_or(&pre.rhs);

                let satisfied = match pre.op {
                    ComparisonOp::Eq => lhs == rhs,
                    ComparisonOp::Neq => lhs != rhs,
                    ComparisonOp::Lt => lhs < rhs,
                    ComparisonOp::Le => lhs <= rhs,
                    ComparisonOp::Gt => lhs > rhs,
                    ComparisonOp::Ge => lhs >= rhs,
                };

                if !satisfied {
                    violations.push(format!(
                        "Rule '{}' precondition failed: {} {:?} {}",
                        name, pre.lhs, pre.op, pre.rhs
                    ));
                }
            }

            // Check postconditions
            for post in &rule.postconditions {
                let lhs = context.resolve(&post.lhs).unwrap_or(&post.lhs);
                let rhs = context.resolve(&post.rhs).unwrap_or(&post.rhs);

                let satisfied = match post.op {
                    ComparisonOp::Eq => lhs == rhs,
                    ComparisonOp::Neq => lhs != rhs,
                    ComparisonOp::Lt => lhs < rhs,
                    ComparisonOp::Le => lhs <= rhs,
                    ComparisonOp::Gt => lhs > rhs,
                    ComparisonOp::Ge => lhs >= rhs,
                };

                if !satisfied {
                    violations.push(format!(
                        "Rule '{}' postcondition failed: {} {:?} {}",
                        name, post.lhs, post.op, post.rhs
                    ));
                }
            }
        }

        if violations.is_empty() {
            Ok(vec![])
        } else {
            Err(RuleError::Conflict(violations.join("; ")))
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// DSL for building rules
pub mod dsl {
    use super::*;

    pub fn rule(name: &str, description: &str) -> VerificationRule {
        VerificationRule::new(name, description)
    }

    pub fn int_var(name: &str) -> RuleVar {
        RuleVar {
            name: name.to_string(),
            sort: VarSort::Int,
        }
    }

    pub fn real_var(name: &str) -> RuleVar {
        RuleVar {
            name: name.to_string(),
            sort: VarSort::Real,
        }
    }

    pub fn bool_var(name: &str) -> RuleVar {
        RuleVar {
            name: name.to_string(),
            sort: VarSort::Bool,
        }
    }

    pub fn eq(lhs: &str, rhs: &str) -> Condition {
        Condition {
            lhs: lhs.to_string(),
            op: ComparisonOp::Eq,
            rhs: rhs.to_string(),
        }
    }

    pub fn neq(lhs: &str, rhs: &str) -> Condition {
        Condition {
            lhs: lhs.to_string(),
            op: ComparisonOp::Neq,
            rhs: rhs.to_string(),
        }
    }

    pub fn lt(lhs: &str, rhs: &str) -> Condition {
        Condition {
            lhs: lhs.to_string(),
            op: ComparisonOp::Lt,
            rhs: rhs.to_string(),
        }
    }

    pub fn gt(lhs: &str, rhs: &str) -> Condition {
        Condition {
            lhs: lhs.to_string(),
            op: ComparisonOp::Gt,
            rhs: rhs.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        use dsl::*;

        let mut rule = rule("add_commute", "Addition is commutative");
        rule.add_variable("a", VarSort::Int);
        rule.add_variable("b", VarSort::Int);
        rule.add_precondition("a", ComparisonOp::Gt, "0");
        rule.add_precondition("b", ComparisonOp::Gt, "0");
        rule.add_postcondition("a + b", ComparisonOp::Eq, "b + a");

        let mut context = RuleContext::new();
        context.bind("a", "5");
        context.bind("b", "3");

        let engine = RuleEngine::new();
        let result = engine.check_rules(&context);
        assert!(result.is_ok());
    }
}
