//! Render pipeline for 2D batching
//!
//! Provides efficient 2D rendering with batched draw calls.

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device,
    FragmentState, IndexFormat, PipelineLayoutDescriptor, PrimitiveState, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState,
};

use crate::Error;

/// Vertex format for 2D rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

/// Uniforms for 2D transformation  
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms2D {
    pub transform: [[f32; 4]; 4],
    pub resolution: [f32; 2],
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub format: TextureFormat,
    pub sample_count: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            format: TextureFormat::Bgra8Unorm,
            sample_count: 1,
        }
    }
}

/// 2D render pipeline
pub struct Pipeline2D {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Pipeline2D {
    /// Create a new 2D pipeline
    pub async fn new(
        device: &Device,
        queue: &wgpu::Queue,
        config: &PipelineConfig,
    ) -> Result<Self, Error> {
        // Create shader module
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("2D Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/quad.wgsl").into()),
        });

        // Create vertex buffer (staging)
        let max_vertices = 65536;
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (max_vertices * std::mem::size_of::<Vertex2D>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create index buffer (staging)
        let max_indices = max_vertices * 6;
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: (max_indices * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("2D Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("2D Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex2D>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            primitive: PrimitiveState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: config.sample_count,
                ..Default::default()
            },
            multiview: None,
        });

        // Initialize uniforms (using a simple default)
        let uniforms = Uniforms2D {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            resolution: [1920.0, 1080.0],
        };

        // Note: In real implementation, we'd create a uniform buffer and bind group here
        // For now, we embed the resolution in the shader

        Ok(Self {
            pipeline,
            vertex_buffer,
            index_buffer,
        })
    }

    /// Begin a render pass
    pub fn begin_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("2D Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    /// Get the render pipeline
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_size() {
        assert_eq!(std::mem::size_of::<Vertex2D>(), 24);
    }

    #[test]
    fn test_uniforms_size() {
        // 4x4 matrix (64 bytes) + 2 floats (8 bytes) = 72 bytes
        assert_eq!(std::mem::size_of::<Uniforms2D>(), 72);
    }
}
