// Dendrite Graph Shader
// Grid background and instanced node rendering

struct Uniforms {
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    aspect: f32,
    time: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct NodeVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) flags: u32,
}

struct EdgeVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) flags: u32,
}

// Full-screen triangle for background grid
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index) - 1);
    let y = f32(i32(vertex_index & 1u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

// Background grid pattern
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pan = vec2<f32>(uniforms.pan_x, uniforms.pan_y);
    let uv = (in.uv * vec2<f32>(uniforms.aspect, 1.0) - pan) / uniforms.zoom;

    let grid_scale = 10.0;
    let grid_uv = fract(uv * grid_scale);
    let grid_width = 0.02;
    let grid_x = step(grid_width, grid_uv.x);
    let grid_y = step(grid_width, grid_uv.y);
    let grid = grid_x * grid_y;

    let base_color = vec3<f32>(0.1, 0.1, 0.15);
    let grid_color = vec3<f32>(0.2, 0.2, 0.25);
    let color = mix(grid_color, base_color, grid);

    return vec4<f32>(color, 1.0);
}

// Node rendering with instancing
@vertex
fn vs_node(
    @location(0) vertex_pos: vec2<f32>,
    @location(1) instance_position: vec2<f32>,
    @location(2) instance_size: vec2<f32>,
    @location(3) instance_color: vec4<f32>,
    @location(4) instance_flags: u32,
) -> NodeVertexOutput {
    var out: NodeVertexOutput;

    // Transform vertex by instance size and position (world space)
    let world_pos = vertex_pos * instance_size + instance_position;

    // Apply pan and zoom
    let pan = vec2<f32>(uniforms.pan_x, uniforms.pan_y);
    let view_pos = (world_pos - pan) * uniforms.zoom;

    // Convert to NDC (normalized device coordinates)
    let ndc_x = view_pos.x / uniforms.aspect;
    let ndc_y = view_pos.y;

    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local_pos = vertex_pos;
    out.size = instance_size;
    out.color = instance_color;
    out.flags = instance_flags;

    return out;
}

// SDF for rounded rectangle
fn sdf_rounded_rect(p: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

// Node fragment shader with effects
@fragment
fn fs_node(in: NodeVertexOutput) -> @location(0) vec4<f32> {
    let selected = (in.flags & 0x1u) != 0u;
    let hovered = (in.flags & 0x2u) != 0u;
    let in_cycle = (in.flags & 0x4u) != 0u;

    // SDF rounded rectangle
    let corner_radius = 4.0;
    let d = sdf_rounded_rect(in.local_pos * in.size, in.size * 0.5, corner_radius);

    // Anti-aliased edge
    let edge_smoothness = 1.0;
    let alpha = 1.0 - smoothstep(-edge_smoothness, edge_smoothness, d);

    var color = in.color;

    // Apply selection effect (brighter)
    if (selected) {
        color = vec4<f32>(color.rgb * 1.3, color.a);
    }

    // Apply hover effect (slight brightness boost)
    if (hovered) {
        color = vec4<f32>(color.rgb * 1.15, color.a);
    }

    // Apply cycle pulsing effect
    if (in_cycle) {
        let pulse = (sin(uniforms.time * 3.0) * 0.5 + 0.5) * 0.3;
        color = vec4<f32>(color.rgb * (1.0 + pulse), color.a);
    }

    // Border for selected nodes
    if (selected) {
        let border_width = 2.0;
        let border_d = abs(d) - border_width;
        let border_alpha = 1.0 - smoothstep(-edge_smoothness, edge_smoothness, border_d);
        let border_color = vec4<f32>(1.0, 1.0, 1.0, border_alpha);

        // Mix border with fill
        if (d > -border_width) {
            color = mix(color, border_color, border_alpha);
        }
    }

    return vec4<f32>(color.rgb, color.a * alpha);
}

// Edge rendering with line expansion to quads
@vertex
fn vs_edge(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) start: vec2<f32>,
    @location(1) end: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) width: f32,
    @location(4) flags: u32,
) -> EdgeVertexOutput {
    var out: EdgeVertexOutput;

    // Compute direction and perpendicular
    let dir = normalize(end - start);
    let perp = vec2<f32>(-dir.y, dir.x);

    // Generate quad corners (4 vertices per instance)
    // 0: start + perp, 1: start - perp, 2: end + perp, 3: end - perp
    let corner = vertex_index % 4u;
    let is_end = corner >= 2u;
    let is_right = (corner % 2u) == 0u;

    let along = select(start, end, is_end);
    let offset = perp * width * 0.5 * select(-1.0, 1.0, is_right);
    let world_pos = along + offset;

    // Apply pan and zoom
    let pan = vec2<f32>(uniforms.pan_x, uniforms.pan_y);
    let view_pos = (world_pos - pan) * uniforms.zoom;

    // Convert to NDC
    let ndc_x = view_pos.x / uniforms.aspect;
    let ndc_y = view_pos.y;

    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);

    // Local position for anti-aliasing (-1 to 1 across width)
    out.local_pos = vec2<f32>(select(-1.0, 1.0, is_right), select(0.0, 1.0, is_end));
    out.color = color;
    out.flags = flags;

    return out;
}

// Edge fragment shader with anti-aliasing
@fragment
fn fs_edge(in: EdgeVertexOutput) -> @location(0) vec4<f32> {
    let in_cycle = (in.flags & 0x1u) != 0u;

    var color = in.color;

    // Apply cycle pulsing effect
    if (in_cycle) {
        let pulse = (sin(uniforms.time * 3.0) * 0.5 + 0.5) * 0.4;
        color = vec4<f32>(color.rgb * (1.0 + pulse), color.a);
    }

    // Anti-aliased edges using distance from center line
    let edge_dist = abs(in.local_pos.x);
    let edge_smoothness = 0.1;
    let alpha = 1.0 - smoothstep(1.0 - edge_smoothness, 1.0, edge_dist);

    return vec4<f32>(color.rgb, color.a * alpha);
}
