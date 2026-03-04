//! MathCore Bridge - Protocol bridges and external interfaces
//!
//! This crate provides MCP protocol, Python bindings, and HTTP API support.

pub mod mcp;
pub mod python;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for bridge components
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MCP error: {0}")]
    Mcp(#[from] mcp::Error),

    #[error("Python error: {0}")]
    Python(#[from] python::Error),

    #[error("Network error: {0}")]
    Network(String),
}
