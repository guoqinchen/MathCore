//! Python bindings (PyO3)

/// Python binding error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Binding error: {0}")]
    Binding(String),

    #[error("Import error: {0}")]
    Import(String),
}

/// Python bridge
pub struct PythonBridge;

impl PythonBridge {
    /// Create a new Python bridge
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonBridge {
    fn default() -> Self {
        Self::new()
    }
}
