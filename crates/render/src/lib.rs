//! MathCore Render - GPU rendering capabilities
//!
//! This crate provides GPU-based rendering capabilities using wgpu:
//! - **2D Visualization**: Function plotting and data visualization
//! - **3D Rendering**: 3D surface and mesh rendering
//! - **Camera System**: Orbit controls and view management
//! - **Lighting**: Multiple light types and material system
//! - **Pipeline**: Custom rendering pipelines
//! - **SIMD**: SIMD-accelerated computations
//! - **Streaming**: Real-time frame streaming
//! - **Shared Memory**: Zero-copy data sharing
//!
//! # Core Components
//!
//! - [`engine`] - Rendering engine
//! - [`camera`] - Camera and view management
//! - [`pipeline`] - Rendering pipelines
//! - [`visualization`] - 2D visualization
//! - [`visualization_3d`] - 3D rendering
//! - [`wgpu`] - GPU backend
//!
//! # Quick Start
//!
//! This crate provides GPU rendering capabilities using wgpu.

pub mod cache;
pub mod camera;
pub mod data;
pub mod engine;
pub mod lighting;
pub mod parallel;
pub mod pipeline;
pub mod protocol;
pub mod shm;
pub mod simd;
pub mod stream;
pub mod visualization;
pub mod visualization_3d;
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
