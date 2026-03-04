// 2D Quad Shader (WGSL)

struct Uniforms {
    transform: mat4x4<f32>,
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Convert from pixel coordinates to normalized device coordinates
    let ndc_position = (input.position / uniforms.resolution) * 2.0 - 1.0;
    // Flip Y axis (wgpu uses Y-up, but screen coords are Y-down)
    let flipped_position = vec2<f32>(ndc_position.x, -ndc_position.y);
    
    output.position = vec4<f32>(flipped_position, 0.0, 1.0);
    output.color = input.color;
    
    return output;
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
