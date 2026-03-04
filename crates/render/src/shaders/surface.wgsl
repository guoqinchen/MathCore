// 3D Surface Shader (WGSL) - Phong lighting model

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    light_pos: vec3<f32>,
    camera_pos: vec3<f32>,
    // Material properties
    ambient: vec3<f32>,
    diffuse: vec3<f32>,
    specular: vec3<f32>,
    shininess: f32,
    // Time for animations
    time: f32,
    _pad: vec3<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transform position to world space
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.world_pos = world_pos.xyz;
    
    // Transform normal to world space (using normal matrix)
    output.normal = normalize((uniforms.normal_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    
    // Pass color to fragment shader
    output.color = input.color;
    
    // Final position in clip space
    output.position = uniforms.view_proj * world_pos;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize interpolated normal
    let N = normalize(input.normal);
    
    // Light direction (from light to surface)
    let L = normalize(uniforms.light_pos - input.world_pos);
    
    // View direction (from camera to surface)
    let V = normalize(uniforms.camera_pos - input.world_pos);
    
    // Reflect direction (reflection of -L about N)
    let R = reflect(-L, N);
    
    // Ambient component
    let ambient = uniforms.ambient * input.color;
    
    // Diffuse component (Lambertian)
    let NdotL = max(dot(N, L), 0.0);
    let diffuse = uniforms.diffuse * input.color * NdotL;
    
    // Specular component (Blinn-Phong)
    let NdotH = max(dot(N, normalize(L + V)), 0.0);
    let specular = uniforms.specular * pow(NdotH, uniforms.shininess);
    
    // Final color
    let final_color = ambient + diffuse + specular;
    
    return vec4<f32>(final_color, 1.0);
}

// Grid shader for coordinate axes
struct GridUniforms {
    view_proj: mat4x4<f32>,
    grid_color: vec4<f32>,
    axis_color_x: vec4<f32>,
    axis_color_y: vec4<f32>,
    axis_color_z: vec4<f32>,
    grid_size: f32,
    grid_divisions: f32,
};

@group(0) @binding(1)
var<uniform> grid_uniforms: GridUniforms;

struct GridVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) is_axis: u32,
};

@vertex
fn vs_grid(input: VertexInput) -> GridVertexOutput {
    var output: GridVertexOutput;
    output.position = uniforms.view_proj * uniforms.model * vec4<f32>(input.position, 1.0);
    output.world_pos = (uniforms.model * vec4<f32>(input.position, 1.0)).xyz;
    output.is_axis = 0u;
    return output;
}

@fragment
fn fs_grid(input: GridVertexOutput) -> @location(0) vec4<f32> {
    // Determine grid lines
    let pos = input.world_pos;
    let grid_size = grid_uniforms.grid_size;
    let divisions = grid_uniforms.grid_divisions;
    let step = grid_size / divisions;
    
    let fx = abs(fract(pos.x / step + 0.5) - 0.5);
    let fz = abs(fract(pos.z / step + 0.5) - 0.5);
    
    let grid_line_width = 0.02;
    let is_grid = (fx < grid_line_width) || (fz < grid_line_width);
    
    // Axis lines
    let axis_line_width = 0.05;
    let is_x_axis = abs(pos.z) < axis_line_width && pos.x > 0.0;
    let is_z_axis = abs(pos.x) < axis_line_width && pos.z > 0.0;
    let is_y_axis = abs(pos.x) < axis_line_width && abs(pos.z) < axis_line_width && pos.y > 0.0;
    
    var color = grid_uniforms.grid_color;
    
    if (is_x_axis) {
        color = grid_uniforms.axis_color_x;
    } else if (is_z_axis) {
        color = grid_uniforms.axis_color_z;
    } else if (is_y_axis) {
        color = grid_uniforms.axis_color_y;
    } else if (!is_grid) {
        discard;
    }
    
    return color;
}
