// ============================================================================
// UI Triangle SDF Shader - Analytic Anti-Aliasing for Triangles
// ============================================================================
//
// This shader uses signed distance fields (SDFs) to render GUI triangles
// with pixel-perfect anti-aliasing at any resolution. Uses the Inigo Quilez
// distance function for exact triangle distances.
//
// Reference: https://iquilezles.org/articles/distfunctions2d/

struct Uniforms {
    screen_size: vec2<f32>,
}

struct InstanceInput {
    @location(0) v0: vec2<f32>,
    @location(1) v1: vec2<f32>,
    @location(2) v2: vec2<f32>,
    @location(3) fill_color: vec4<f32>,
    @location(4) stroke_color: vec4<f32>,
    @location(5) stroke_width: f32,
}

struct FragmentInput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) v0: vec2<f32>,
    @location(1) v1: vec2<f32>,
    @location(2) v2: vec2<f32>,
    @location(3) fill_color: vec4<f32>,
    @location(4) stroke_color: vec4<f32>,
    @location(5) stroke_width: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs_main(inst: InstanceInput) -> FragmentInput {
    var out: FragmentInput;

    // No actual vertices - we just pass instance data through
    // The fragment shader will handle all rendering
    // We use a fullscreen pass or bounding box rendering (handled by pipeline setup)

    // For now, set dummy clip position - will be overridden by actual vertex buffer
    out.frag_coord = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Pass through triangle vertices and colors
    out.v0 = inst.v0;
    out.v1 = inst.v1;
    out.v2 = inst.v2;
    out.fill_color = inst.fill_color;
    out.stroke_color = inst.stroke_color;
    out.stroke_width = inst.stroke_width;

    return out;
}

// ============================================================================
// Distance Functions
// ============================================================================

// Signed distance to a triangle (Inigo Quilez formula)
// Returns negative distance inside, positive outside
fn sd_triangle(p: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    let e0 = p1 - p0;
    let e1 = p2 - p1;
    let e2 = p0 - p2;
    let v0 = p - p0;
    let v1 = p - p1;
    let v2 = p - p2;

    let pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0);
    let pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0);
    let pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0);

    let s = sign(e0.x * e2.y - e0.y * e2.x);
    let d0 = vec2<f32>(dot(pq0, pq0), s * (v0.x * e0.y - v0.y * e0.x));
    let d1 = vec2<f32>(dot(pq1, pq1), s * (v1.x * e1.y - v1.y * e1.x));
    let d2 = vec2<f32>(dot(pq2, pq2), s * (v2.x * e2.y - v2.y * e2.x));

    let d = min(min(d0, d1), d2);
    return -sqrt(d.x) * sign(d.y);
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    // Fragment position in screen space (pixel coordinates)
    // Adjust for pixel centers at half-integer coordinates
    let pixel_pos = in.frag_coord.xy - 0.5;

    // Compute signed distance to triangle
    let dist = sd_triangle(pixel_pos, in.v0, in.v1, in.v2);

    // Anti-aliasing width based on screen-space derivatives
    // This gives us approximately 1 pixel of AA
    let aa_width = length(vec2<f32>(dpdx(dist), dpdy(dist)));

    // Decide between stroke and fill
    let stroke_width = in.stroke_width;
    var final_color: vec4<f32>;

    if stroke_width > 0.0 {
        // Render stroke: band around the triangle edge
        let stroke_dist = abs(dist) - stroke_width * 0.5;

        if stroke_dist < 0.0 {
            // We're in the stroke region
            let alpha = 1.0 - smoothstep(-aa_width, aa_width, stroke_dist);
            final_color = in.stroke_color * alpha;
        } else {
            // Check if we're inside the triangle (for fill)
            if dist < 0.0 {
                // We're inside, render fill
                let alpha = 1.0 - smoothstep(-aa_width, aa_width, dist);
                final_color = in.fill_color * alpha;
            } else {
                // Outside both stroke and fill
                discard;
            }
        }
    } else {
        // No stroke, just fill
        if dist < 0.0 {
            let alpha = 1.0 - smoothstep(-aa_width, aa_width, dist);
            final_color = in.fill_color * alpha;
        } else {
            discard;
        }
    }

    return final_color;
}
