// Dendrite Graph Shader
// Simple grid renderer to verify pan/zoom functionality

struct Uniforms {
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    aspect: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Full-screen triangle (no vertex buffer needed)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index) - 1);
    let y = f32(i32(vertex_index & 1u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

// Simple grid pattern to verify shader works
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Apply pan and zoom to UV coordinates
    let pan = vec2<f32>(uniforms.pan_x, uniforms.pan_y);
    let uv = (in.uv * vec2<f32>(uniforms.aspect, 1.0) - pan) / uniforms.zoom;

    // Grid pattern to show pan/zoom working
    let grid_scale = 10.0;
    let grid_uv = fract(uv * grid_scale);
    let grid_width = 0.02;
    let grid_x = step(grid_width, grid_uv.x);
    let grid_y = step(grid_width, grid_uv.y);
    let grid = grid_x * grid_y;

    // Base color with grid overlay
    let base_color = vec3<f32>(0.1, 0.1, 0.15);
    let grid_color = vec3<f32>(0.2, 0.2, 0.25);
    let color = mix(grid_color, base_color, grid);

    return vec4<f32>(color, 1.0);
}
