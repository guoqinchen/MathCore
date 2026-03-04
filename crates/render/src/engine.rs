//! wgpu rendering engine
//!
//! Provides GPU-accelerated rendering with automatic backend selection.

use std::sync::Arc;
use wgpu::{Adapter, Backend, Device, Instance, Queue, Surface, SurfaceConfiguration};

use crate::Error;

/// Automatic backend selection based on platform
pub fn default_backend() -> wgpu::Backends {
    #[cfg(target_os = "macos")]
    {
        wgpu::Backends::METAL | wgpu::Backends::VULKAN
    }
    #[cfg(target_os = "linux")]
    {
        wgpu::Backends::VULKAN | wgpu::Backends::GL
    }
    #[cfg(target_os = "windows")]
    {
        wgpu::Backends::DX12 | wgpu::Backends::DX11 | wgpu::Backends::VULKAN
    }
    #[cfg(target_family = "wasm")]
    {
        wgpu::Backends::WEBGPU
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_family = "wasm"
    )))]
    {
        wgpu::Backends::all()
    }
}

/// Render engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Backend bitflags (default: auto-select based on platform)
    pub backends: wgpu::Backends,
    /// Power preference (default: high-performance)
    pub power_preference: wgpu::PowerPreference,
    /// Enable vsync (default: true)
    pub vsync: bool,
    /// Request high DPI (default: true)
    pub high_dpi: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            backends: default_backend(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            vsync: true,
            high_dpi: true,
        }
    }
}

/// GPU render engine
pub struct Engine {
    instance: Instance,
    adapter: Adapter,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Option<Surface<'static>>,
    config: EngineConfig,
}

impl Engine {
    /// Create a new engine with default configuration
    pub async fn new() -> Result<Self, Error> {
        Self::with_config(EngineConfig::default()).await
    }

    /// Create a new engine with custom configuration
    pub async fn with_config(config: EngineConfig) -> Result<Self, Error> {
        // Create instance with specified backends
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: config.backends,
            ..Default::default()
        });

        // Enumerate adapters and pick the first available one
        let adapter = instance
            .enumerate_adapters(config.backends)
            .into_iter()
            .next()
            .ok_or_else(|| Error::Wgpu("No compatible GPU found".to_string()))?;

        // Log adapter info
        tracing::info!(
            "Selected GPU: {:?} (backend: {:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("MathCore Render Device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| Error::Wgpu(format!("Failed to request device: {}", e)))?;

        Ok(Self {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: None,
            config,
        })
    }

    /// Get the GPU device
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Get the command queue
    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    /// Get the adapter info
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    /// Check if a specific backend is available
    pub fn has_backend(&self, backend: Backend) -> bool {
        self.adapter.get_info().backend == backend
    }

    /// Configure surface for window rendering
    pub fn configure_surface(
        &mut self,
        surface: Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<(), Error> {
        let caps = surface.get_capabilities(&self.adapter);

        // Select appropriate format
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        // Select present mode
        let present_mode = if self.config.vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Immediate
        };

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![format],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&self.device, &config);
        self.surface = Some(surface);

        Ok(())
    }

    /// Resize the surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(surface) = &self.surface {
            let format = surface
                .get_capabilities(&self.adapter)
                .formats
                .first()
                .copied()
                .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);

            let config = SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width,
                height,
                present_mode: if self.config.vsync {
                    wgpu::PresentMode::Fifo
                } else {
                    wgpu::PresentMode::Immediate
                },
                alpha_mode: surface.get_capabilities(&self.adapter).alpha_modes[0],
                view_formats: vec![format],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&self.device, &config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = Engine::new().await;
        // May fail if no GPU available (e.g., CI environments)
        if let Ok(e) = engine {
            tracing::info!("GPU: {:?}", e.adapter_info());
        }
    }

    #[test]
    fn test_default_backend() {
        let backend = default_backend();
        assert!(!backend.is_empty());
    }
}
