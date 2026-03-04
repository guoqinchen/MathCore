//! wgpu-based GPU renderer

/// wgpu renderer error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Device error: {0}")]
    DeviceError(String),

    #[error("Context error: {0}")]
    ContextError(String),
}

/// wgpu-based renderer
pub struct Renderer;

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self
    }

    /// Draw a frame
    pub fn draw(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
