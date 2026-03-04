//! MCP (Model Context Protocol) server implementation

/// MCP error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

/// MCP server
pub struct McpServer;

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        Self
    }

    /// Handle an MCP request
    pub fn handle(&self, _request: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
