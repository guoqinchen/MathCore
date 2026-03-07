//! MathCore Bridge - Protocol bridges and external interfaces
//!
//! This crate provides bridges and interfaces for external systems:
//! - **MCP Protocol**: Model Context Protocol for AI integration
//! - **Python Bindings**: Native Python extension for MathCore
//! - **HTTP API**: RESTful API for remote access
//! - **Arrow Integration**: Apache Arrow data plane support
//!
//! # Core Components
//!
//! - [`mcp`] - MCP protocol implementation
//! - [`python`] - Python bindings
//!
//! # Quick Start
//!
//! ```no_run
//! use mathcore_bridge::{Error, Result};
//!
//! fn example() -> Result<()> {
//!     // Use Python bindings or MCP protocol
//!     Ok(())
//! }
//! ```

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
