//! Computation replay/trace module
//!
//! Provides step-by-step recording and replay of mathematical computations.

use crate::Expr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single step in a computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationStep {
    /// Step number (0-indexed)
    pub step: usize,
    /// Description of what happened
    pub description: String,
    /// The expression at this step
    pub expression: String,
    /// Variables at this step (if any)
    pub variables: Option<HashMap<String, f64>>,
    /// Result of evaluating this step
    pub result: Option<f64>,
    /// Whether this step is a simplification
    pub is_simplification: bool,
    /// Whether this step is an evaluation
    pub is_evaluation: bool,
    /// Parent step (for tree visualization)
    pub parent: Option<usize>,
}

/// A complete computation trace
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputationTrace {
    /// Original input expression
    pub input: String,
    /// All steps in order
    pub steps: Vec<ComputationStep>,
    /// Final result
    pub final_result: Option<f64>,
    /// Total time elapsed (microseconds)
    pub elapsed_us: Option<u64>,
}

impl ComputationTrace {
    /// Create a new empty trace
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            steps: Vec::new(),
            final_result: None,
            elapsed_us: None,
        }
    }

    /// Add a simplification step
    pub fn add_simplification(&mut self, expr: &Expr, description: &str) {
        let step = ComputationStep {
            step: self.steps.len(),
            description: description.to_string(),
            expression: expr.to_string(),
            variables: None,
            result: None,
            is_simplification: true,
            is_evaluation: false,
            parent: self.steps.last().map(|s| s.step),
        };
        self.steps.push(step);
    }

    /// Add an evaluation step
    pub fn add_evaluation(
        &mut self,
        expr: &Expr,
        vars: &HashMap<String, f64>,
        result: f64,
        description: &str,
    ) {
        let step = ComputationStep {
            step: self.steps.len(),
            description: description.to_string(),
            expression: expr.to_string(),
            variables: Some(vars.clone()),
            result: Some(result),
            is_simplification: false,
            is_evaluation: true,
            parent: self.steps.last().map(|s| s.step),
        };
        self.steps.push(step);
    }

    /// Add a general step
    pub fn add_step(&mut self, description: &str, expression: &str) {
        let step = ComputationStep {
            step: self.steps.len(),
            description: description.to_string(),
            expression: expression.to_string(),
            variables: None,
            result: None,
            is_simplification: false,
            is_evaluation: false,
            parent: self.steps.last().map(|s| s.step),
        };
        self.steps.push(step);
    }

    /// Set the final result
    pub fn set_result(&mut self, result: f64) {
        self.final_result = Some(result);
    }

    /// Set elapsed time
    pub fn set_elapsed(&mut self, us: u64) {
        self.elapsed_us = Some(us);
    }

    /// Get step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import from JSON string
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// Builder for creating traces with recording
pub struct TraceBuilder {
    trace: ComputationTrace,
}

impl TraceBuilder {
    /// Create a new trace builder
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            trace: ComputationTrace::new(input),
        }
    }

    /// Record a parsing step
    pub fn record_parse(&mut self, expr: &Expr) -> &mut Self {
        self.trace.add_step("Parsed expression", &expr.to_string());
        self
    }

    /// Record a simplification step
    pub fn record_simplify(&mut self, expr: &Expr) -> &mut Self {
        self.trace.add_simplification(expr, "Simplified expression");
        self
    }

    /// Record an evaluation step
    pub fn record_evaluate(
        &mut self,
        expr: &Expr,
        vars: &HashMap<String, f64>,
        result: f64,
    ) -> &mut Self {
        self.trace
            .add_evaluation(expr, vars, result, "Evaluated with variables");
        self
    }

    /// Record a differentiation step
    pub fn record_differentiate(&mut self, expr: &Expr, var: &str) -> &mut Self {
        self.trace.add_step(
            &format!("Differentiated with respect to {}", var),
            &expr.to_string(),
        );
        self
    }

    /// Build the final trace
    pub fn build(self) -> ComputationTrace {
        self.trace
    }
}

/// Replayer for stepping through computation traces
pub struct TraceReplayer {
    trace: ComputationTrace,
    current_step: usize,
}

impl TraceReplayer {
    /// Create a new replayer from a trace
    pub fn new(trace: ComputationTrace) -> Self {
        Self {
            trace,
            current_step: 0,
        }
    }

    /// Get current step
    pub fn current_step(&self) -> Option<&ComputationStep> {
        self.trace.steps.get(self.current_step)
    }

    /// Move to next step
    pub fn next(&mut self) -> Option<&ComputationStep> {
        if !self.is_at_end() {
            self.current_step += 1;
        }
        self.current_step()
    }

    /// Move to previous step
    pub fn prev(&mut self) -> Option<&ComputationStep> {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
        self.current_step()
    }

    /// Jump to specific step
    pub fn jump_to(&mut self, step: usize) -> Option<&ComputationStep> {
        if step < self.trace.steps.len() {
            self.current_step = step;
        }
        self.current_step()
    }

    /// Get total steps
    pub fn total_steps(&self) -> usize {
        self.trace.steps.len()
    }

    /// Check if at end
    pub fn is_at_end(&self) -> bool {
        self.current_step >= self.trace.steps.len().saturating_sub(1)
    }

    /// Check if at start
    pub fn is_at_start(&self) -> bool {
        self.current_step == 0
    }

    /// Get the full trace
    pub fn trace(&self) -> &ComputationTrace {
        &self.trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_creation() {
        let trace = ComputationTrace::new("x^2 + 2*x + 1");
        assert_eq!(trace.input, "x^2 + 2*x + 1");
        assert_eq!(trace.step_count(), 0);
    }

    #[test]
    fn test_trace_builder() {
        use crate::Expr;

        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3.0);

        let expr = Expr::Var("x".to_string());

        let mut builder = TraceBuilder::new("x^2");
        builder.record_parse(&expr);
        builder.record_evaluate(&expr, &vars, 9.0);
        let trace = builder.build();

        assert_eq!(trace.step_count(), 2);
        assert_eq!(trace.final_result, None);
    }

    #[test]
    fn test_replayer() {
        let mut trace = ComputationTrace::new("test");
        trace.add_step("Step 1", "1 + 1");
        trace.add_step("Step 2", "2");

        let mut replayer = TraceReplayer::new(trace);

        assert!(replayer.is_at_start());
        assert_eq!(replayer.current_step().unwrap().expression, "1 + 1");

        replayer.next();
        assert_eq!(replayer.current_step().unwrap().expression, "2");

        replayer.prev();
        assert_eq!(replayer.current_step().unwrap().expression, "1 + 1");

        assert!(replayer.is_at_start());
    }

    #[test]
    fn test_json_serialization() {
        let mut trace = ComputationTrace::new("x + 1");
        trace.add_step("test", "x + 1");

        let json = trace.to_json().unwrap();
        let restored = ComputationTrace::from_json(&json).unwrap();

        assert_eq!(restored.input, "x + 1");
        assert_eq!(restored.step_count(), 1);
    }

    #[test]
    fn test_empty_trace() {
        let trace = ComputationTrace::new("empty");
        let mut replayer = TraceReplayer::new(trace);

        // Should not panic
        assert!(replayer.current_step().is_none());
        replayer.next();
        assert!(replayer.current_step().is_none());
        replayer.prev();
        assert!(replayer.current_step().is_none());
    }
}
