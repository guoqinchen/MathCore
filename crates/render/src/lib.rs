//! MathCore Render - GPU rendering capabilities
//!
//! This crate provides GPU-based rendering using wgpu.

pub mod data;
pub mod engine;
pub mod pipeline;
pub mod protocol;
pub mod shm;
pub mod stream;
pub mod visualization;
pub mod wgpu;
pub mod window;

#[cfg(test)]
mod tests;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for render engines
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("wgpu error: {0}")]
    Wgpu(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Data error: {0}")]
    Data(String),

    #[error("IO error: {0}")]
    Io(String),
}
