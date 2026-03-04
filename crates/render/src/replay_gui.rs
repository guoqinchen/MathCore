//! Computation Replay GUI
//!
//! A simple text-based replay viewer that can be integrated with a GUI framework

use mathcore_compute::{ComputationTrace, TraceReplayer};

/// A GUI widget for displaying computation replay
pub struct ReplayViewer {
    replayer: TraceReplayer,
    show_variables: bool,
}

impl ReplayViewer {
    /// Create a new replay viewer
    pub fn new(trace: ComputationTrace) -> Self {
        Self {
            replayer: TraceReplayer::new(trace),
            show_variables: true,
        }
    }

    /// Get current step info
    pub fn current_step_info(&self) -> String {
        match self.replayer.current_step() {
            Some(step) => {
                let mut info = format!(
                    "Step {}/{}: {}\nExpression: {}\n",
                    step.step + 1,
                    self.replayer.total_steps(),
                    step.description,
                    step.expression
                );

                if step.is_evaluation {
                    if let Some(vars) = &step.variables {
                        if self.show_variables && !vars.is_empty() {
                            info.push_str("Variables:\n");
                            for (k, v) in vars {
                                info.push_str(&format!("  {} = {}\n", k, v));
                            }
                        }
                    }
                    if let Some(result) = step.result {
                        info.push_str(&format!("Result: {}\n", result));
                    }
                }

                if step.is_simplification {
                    info.push_str("[Simplification step]\n");
                }

                info
            }
            None => "No steps available".to_string(),
        }
    }

    /// Move to next step
    pub fn next(&mut self) -> bool {
        if !self.replayer.is_at_end() {
            self.replayer.next();
            true
        } else {
            false
        }
    }

    /// Move to previous step
    pub fn prev(&mut self) -> bool {
        if !self.replayer.is_at_start() {
            self.replayer.prev();
            true
        } else {
            false
        }
    }

    /// Jump to specific step
    pub fn jump_to(&mut self, step: usize) {
        self.replayer.jump_to(step);
    }

    /// Check if at start
    pub fn is_at_start(&self) -> bool {
        self.replayer.is_at_start()
    }

    /// Check if at end
    pub fn is_at_end(&self) -> bool {
        self.replayer.is_at_end()
    }

    /// Get total steps
    pub fn total_steps(&self) -> usize {
        self.replayer.total_steps()
    }

    /// Toggle variable display
    pub fn toggle_variables(&mut self) {
        self.show_variables = !self.show_variables;
    }

    /// Get full trace as JSON
    pub fn trace_json(&self) -> String {
        self.replayer.trace().to_json().unwrap_or_default()
    }

    /// Render as ASCII art tree
    pub fn render_tree(&self) -> String {
        let trace = self.replayer.trace();
        let mut output = String::new();

        output.push_str(&format!("Computation: {}\n", trace.input));
        output.push_str(&format!("Steps: {}\n", trace.steps.len()));
        if let Some(result) = trace.final_result {
            output.push_str(&format!("Final Result: {}\n", result));
        }
        output.push_str("\n");

        for step in &trace.steps {
            let prefix = if step.is_simplification {
                "▼"
            } else if step.is_evaluation {
                "▶"
            } else {
                "●"
            };

            output.push_str(&format!(
                "{}[{}] {}: {}\n",
                prefix,
                step.step + 1,
                step.description,
                step.expression
            ));
        }

        output
    }
}

/// Progress bar for replay
pub struct ReplayProgress {
    current: usize,
    total: usize,
}

impl ReplayProgress {
    pub fn new(total: usize) -> Self {
        Self { current: 0, total }
    }

    pub fn set(&mut self, current: usize) {
        self.current = current.min(self.total);
    }

    pub fn render(&self) -> String {
        let bar_width = 30;
        let progress = if self.total > 0 {
            (self.current as f64 / self.total as f64 * bar_width as f64) as usize
        } else {
            0
        };

        let bar: String = (0..bar_width)
            .map(|i| if i < progress { '█' } else { '░' })
            .collect();

        format!(
            "│{}│ {}/{} ({:.1}%)",
            bar,
            self.current + 1,
            self.total,
            if self.total > 0 {
                (self.current as f64 + 1.0) / self.total as f64 * 100.0
            } else {
                0.0
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_replay_viewer() {
        let trace = ComputationTrace::new("x^2 + 2x + 1");
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 2.0);

        trace.add_step("Parse", "x^2 + 2*x + 1");
        trace.add_simplification(&mathcore_compute::Expr::Var("x".to_string()), "Simplify");
        trace.add_evaluation(
            &mathcore_compute::Expr::Var("x".to_string()),
            &vars,
            9.0,
            "Evaluate",
        );
        trace.set_result(9.0);

        let mut viewer = ReplayViewer::new(trace);

        assert!(viewer.is_at_start());
        assert!(!viewer.is_at_end());

        viewer.next();
        assert!(!viewer.is_at_start());

        viewer.prev();
        assert!(viewer.is_at_start());
    }

    #[test]
    fn test_progress_bar() {
        let mut progress = ReplayProgress::new(10);
        progress.set(5);

        let bar = progress.render();
        assert!(bar.contains("5/10") || bar.contains("6/10"));
    }
}
