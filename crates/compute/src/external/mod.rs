//! External engine bridge

/// External engine error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Call failed: {0}")]
    CallFailed(String),
}

/// External engine bridge
pub struct ExternalBridge;

impl ExternalBridge {
    /// Create a new external bridge
    pub fn new() -> Self {
        Self
    }

    /// Call an external engine
    pub fn call(&self, _engine: &str, _expr: &str) -> Result<String, Error> {
        Ok(_expr.to_string())
    }
}

impl Default for ExternalBridge {
    fn default() -> Self {
        Self::new()
    }
}
