//! 3D Visualization - Surface rendering with Phong lighting
//!
//! Provides 3D rendering for mathematical surfaces z = f(x, y).

use std::f32::consts::PI;
use std::num::NonZero;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, Buffer, BufferDescriptor, BufferUsages, CommandEncoder,
    DepthBiasState, DepthStencilState, Device, FragmentState, PipelineLayoutDescriptor,
    PrimitiveState, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, StencilState, TextureFormat, VertexBufferLayout, VertexState,
};

use crate::Error;

/// 3D Vector
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn up() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            *self
        }
    }

    pub fn cross(&self, other: &Vec3) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn add(&self, other: &Vec3) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: &Vec3) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scale(&self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

/// 4x4 Matrix for 3D transformations
#[derive(Copy, Clone, Debug, Default)]
pub struct Mat4(pub [[f32; 4]; 4]);

impl Mat4 {
    pub fn identity() -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_translation(x: f32, y: f32, z: f32) -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ])
    }

    pub fn from_scale(x: f32, y: f32, z: f32) -> Self {
        Self([
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self([
            [c, 0.0, s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_rotation_x(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, -s, 0.0],
            [0.0, s, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_rotation_z(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self([
            [c, -s, 0.0, 0.0],
            [s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn multiply(&self, other: &Mat4) -> Self {
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * other.0[0][j]
                    + self.0[i][1] * other.0[1][j]
                    + self.0[i][2] * other.0[2][j]
                    + self.0[i][3] * other.0[3][j];
            }
        }
        Self(result)
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.0.as_ptr() as *const f32
    }

    pub fn transpose(&self) -> Self {
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[j][i];
            }
        }
        Self(result)
    }

    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let tan_half_fov = (fov_y / 2.0).tan();
        let f = 1.0 / tan_half_fov;

        Self([
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) / (near - far), -1.0],
            [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
        ])
    }

    pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Self {
        let f = center.sub(eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(&f);

        Self([
            [s.x, u.x, -f.x, 0.0],
            [s.y, u.y, -f.y, 0.0],
            [s.z, u.z, -f.z, 0.0],
            [-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
        ])
    }
}

/// 3D Camera with orbit controls
#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub orbit_theta: f32,
    pub orbit_phi: f32,
    pub distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::zero(),
            up: Vec3::up(),
            fov: 45.0 * PI / 180.0,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            orbit_theta: 0.0,
            orbit_phi: PI / 4.0,
            distance: 10.0,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_orbit(&mut self) {
        self.position.x =
            self.target.x + self.distance * self.orbit_phi.cos() * self.orbit_theta.sin();
        self.position.y = self.target.y + self.distance * self.orbit_phi.sin();
        self.position.z =
            self.target.z + self.distance * self.orbit_phi.cos() * self.orbit_theta.cos();
    }

    pub fn rotate(&mut self, delta_theta: f32, delta_phi: f32) {
        self.orbit_theta += delta_theta;
        self.orbit_phi = (self.orbit_phi + delta_phi).clamp(0.01, PI - 0.01);
        self.update_orbit();
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(1.0, 100.0);
        self.update_orbit();
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let right = self.target.sub(&self.position).cross(&self.up).normalize();
        let up = self.up.normalize();
        let offset = right.scale(-delta_x).add(&up.scale(delta_y));
        self.target = self.target.add(&offset);
        self.position = self.position.add(&offset);
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(&self.position, &self.target, &self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov, self.aspect, self.near, self.far)
    }

    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix().multiply(&self.view_matrix())
    }

    pub fn set_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height.max(1.0);
    }
}

/// 3D Vertex format
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

/// Uniform data for 3D rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms3D {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
    pub light_pos: [f32; 3],
    pub _pad0: f32,
    pub camera_pos: [f32; 3],
    pub _pad1: f32,
    pub ambient: [f32; 3],
    pub _pad2: f32,
    pub diffuse: [f32; 3],
    pub _pad3: f32,
    pub specular: [f32; 3],
    pub shininess: f32,
    pub time: f32,
    pub _pad4: [f32; 3],
}

/// Material properties for Phong lighting
#[derive(Clone, Debug)]
pub struct Material {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambient: [0.1, 0.1, 0.1],
            diffuse: [0.7, 0.7, 0.7],
            specular: [1.0, 1.0, 1.0],
            shininess: 32.0,
        }
    }
}

impl Material {
    pub fn new(ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3], shininess: f32) -> Self {
        Self {
            ambient,
            diffuse,
            specular,
            shininess,
        }
    }

    pub fn copper() -> Self {
        Self {
            ambient: [0.1913, 0.0737, 0.0225],
            diffuse: [0.7038, 0.2705, 0.0828],
            specular: [0.2568, 0.1373, 0.0860],
            shininess: 12.8,
        }
    }

    pub fn gold() -> Self {
        Self {
            ambient: [0.2473, 0.1995, 0.0745],
            diffuse: [0.7516, 0.6065, 0.2265],
            specular: [0.6283, 0.5558, 0.3661],
            shininess: 51.2,
        }
    }

    pub fn cyan() -> Self {
        Self {
            ambient: [0.0, 0.1, 0.1],
            diffuse: [0.0, 0.6, 0.6],
            specular: [1.0, 1.0, 1.0],
            shininess: 64.0,
        }
    }
}

/// Surface mesh generated from function f(x, y)
#[derive(Clone, Debug)]
pub struct SurfaceMesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl SurfaceMesh {
    pub fn from_function<F>(
        func: F,
        x_range: (f32, f32),
        y_range: (f32, f32),
        resolution: u32,
    ) -> Self
    where
        F: Fn(f32, f32) -> f32 + Send + Sync,
    {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let step_x = (x_range.1 - x_range.0) / (resolution - 1) as f32;
        let step_y = (y_range.1 - y_range.0) / (resolution - 1) as f32;

        for j in 0..resolution {
            for i in 0..resolution {
                let x = x_range.0 + (i as f32) * step_x;
                let y = y_range.0 + (j as f32) * step_y;
                let z = func(x, y);

                let h = 0.01;
                let dzdx = (func(x + h, y) - func(x - h, y)) / (2.0 * h);
                let dzdy = (func(x, y + h) - func(x, y - h)) / (2.0 * h);
                let normal = Vec3::new(-dzdx, -dzdy, 1.0).normalize();

                let normalized_z = ((z + 2.0) / 4.0).clamp(0.0, 1.0);
                let color = [
                    0.2 + normalized_z * 0.8,
                    0.4 * (1.0 - normalized_z),
                    0.6 + normalized_z * 0.4,
                ];

                vertices.push(Vertex3D {
                    position: [x, z, y],
                    normal: [normal.x, normal.z, normal.y],
                    color,
                });
            }
        }

        for j in 0..(resolution - 1) {
            for i in 0..(resolution - 1) {
                let top_left = j * resolution + i;
                let top_right = top_left + 1;
                let bottom_left = (j + 1) * resolution + i;
                let bottom_right = bottom_left + 1;

                indices.push(top_left as u32);
                indices.push(bottom_left as u32);
                indices.push(top_right as u32);

                indices.push(top_right as u32);
                indices.push(bottom_left as u32);
                indices.push(bottom_right as u32);
            }
        }

        Self { vertices, indices }
    }

    pub fn sine_wave(resolution: u32) -> Self {
        Self::from_function(
            |x, y| (x * 0.5).sin() * (y * 0.5).sin() * 2.0,
            (-PI, PI),
            (-PI, PI),
            resolution,
        )
    }

    pub fn saddle(resolution: u32) -> Self {
        Self::from_function(
            |x, y| (x * x - y * y) * 0.1,
            (-3.0, 3.0),
            (-3.0, 3.0),
            resolution,
        )
    }

    pub fn ripple(resolution: u32) -> Self {
        Self::from_function(
            |x, y| {
                let r = (x * x + y * y).sqrt();
                (r * 3.0).sin() * 0.5 / (r + 0.5)
            },
            (-5.0, 5.0),
            (-5.0, 5.0),
            resolution,
        )
    }
}

/// 3D render pipeline
pub struct Pipeline3D {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
    index_count: usize,
}

impl Pipeline3D {
    pub async fn new(device: &Device, format: TextureFormat) -> Result<Self, Error> {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("3D Surface Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/surface.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("3D Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZero::new(std::mem::size_of::<Uniforms3D>() as u64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("3D Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex3D>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            primitive: PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 1,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: 1,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms3D>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("3D Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
            }],
        });

        Ok(Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            bind_group_layout,
            index_count: 0,
        })
    }

    pub fn upload_mesh(&mut self, device: &Device, mesh: &SurfaceMesh) {
        let vertex_size = mesh.vertices.len() * std::mem::size_of::<Vertex3D>();
        let index_size = mesh.indices.len() * std::mem::size_of::<u32>();

        self.vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: vertex_size as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: index_size as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.index_count = mesh.indices.len();
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &Uniforms3D) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn begin_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        view: &'a wgpu::TextureView,
        depth_view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("3D Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.15,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }
}

/// 3D Scene containing surfaces
pub struct Scene3D {
    pub meshes: Vec<SurfaceMesh>,
    pub camera: Camera,
    pub light_pos: Vec3,
    pub material: Material,
}

impl Default for Scene3D {
    fn default() -> Self {
        Self {
            meshes: Vec::new(),
            camera: Camera::new(),
            light_pos: Vec3::new(10.0, 10.0, 10.0),
            material: Material::default(),
        }
    }
}

impl Scene3D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_mesh(&mut self, mesh: SurfaceMesh) {
        self.meshes.push(mesh);
    }

    pub fn with_sine_wave(mut self, resolution: u32) -> Self {
        self.meshes.push(SurfaceMesh::sine_wave(resolution));
        self
    }

    pub fn with_saddle(mut self, resolution: u32) -> Self {
        self.meshes.push(SurfaceMesh::saddle(resolution));
        self
    }

    pub fn with_ripple(mut self, resolution: u32) -> Self {
        self.meshes.push(SurfaceMesh::ripple(resolution));
        self
    }

    pub fn build_uniforms(&self, model: &Mat4) -> Uniforms3D {
        let view_proj = self.camera.view_projection();
        let normal_matrix = model.transpose();

        Uniforms3D {
            view_proj: view_proj.0,
            model: model.0,
            normal_matrix: normal_matrix.0,
            light_pos: [self.light_pos.x, self.light_pos.y, self.light_pos.z],
            _pad0: 0.0,
            camera_pos: [
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            ],
            _pad1: 0.0,
            ambient: self.material.ambient,
            _pad2: 0.0,
            diffuse: self.material.diffuse,
            _pad3: 0.0,
            specular: self.material.specular,
            shininess: self.material.shininess,
            time: 0.0,
            _pad4: [0.0; 3],
        }
    }
}

/// Input state for camera control
#[derive(Default)]
pub struct InputState {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_delta_x: f32,
    pub mouse_delta_y: f32,
    pub wheel_delta: f32,
    pub key_left: bool,
    pub key_right: bool,
    pub key_up: bool,
    pub key_down: bool,
}

impl InputState {
    pub fn process_mouse(&mut self, x: f32, y: f32, _button: u32) {
        self.mouse_delta_x = x - self.mouse_x;
        self.mouse_delta_y = y - self.mouse_y;
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn process_wheel(&mut self, delta: f32) {
        self.wheel_delta = delta;
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        if self.mouse_delta_x != 0.0 || self.mouse_delta_y != 0.0 {
            camera.rotate(-self.mouse_delta_x * 0.01, -self.mouse_delta_y * 0.01);
        }

        if self.wheel_delta != 0.0 {
            camera.zoom(self.wheel_delta * 0.5);
        }

        let pan_speed = 0.1;
        if self.key_left {
            camera.pan(-pan_speed, 0.0);
        }
        if self.key_right {
            camera.pan(pan_speed, 0.0);
        }
        if self.key_up {
            camera.pan(0.0, pan_speed);
        }
        if self.key_down {
            camera.pan(0.0, -pan_speed);
        }

        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;
        self.wheel_delta = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mat4_identity() {
        let m = Mat4::identity();
        assert_eq!(m.0[0][0], 1.0);
        assert_eq!(m.0[1][1], 1.0);
        assert_eq!(m.0[2][2], 1.0);
        assert_eq!(m.0[3][3], 1.0);
    }

    #[test]
    fn test_mat4_multiply() {
        let a = Mat4::identity();
        let b = Mat4::from_translation(1.0, 2.0, 3.0);
        let c = a.multiply(&b);
        assert_eq!(c.0[3][0], 1.0);
        assert_eq!(c.0[3][1], 2.0);
        assert_eq!(c.0[3][2], 3.0);
    }

    #[test]
    fn test_vec3_operations() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.add(&b), Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(a.sub(&b), Vec3::new(-3.0, -3.0, -3.0));
        assert_eq!(a.scale(2.0), Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_surface_mesh() {
        let mesh = SurfaceMesh::sine_wave(10);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn test_camera_orbit() {
        let mut camera = Camera::new();
        camera.orbit_theta = PI / 4.0;
        camera.orbit_phi = PI / 4.0;
        camera.distance = 10.0;
        camera.update_orbit();
        assert!(camera.position.length() > 0.0);
    }
}
