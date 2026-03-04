//! MathCore MCP Bridge
//!
//! Model Context Protocol integration for MathCore

pub mod protocol;
pub mod server;
pub mod tools;

pub use protocol::{McpRequest, McpResponse, McpTool};
pub use server::McpServer;
pub use tools::MathCoreTools;
