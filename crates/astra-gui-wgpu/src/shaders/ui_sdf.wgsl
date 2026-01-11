// ============================================================================
// UI SDF Shader - Analytic Anti-Aliasing for GUI Components
// ============================================================================
//
// This shader uses signed distance fields (SDFs) to render GUI rectangles
// with pixel-perfect anti-aliasing at any resolution. Instead of tessellating
// shapes into many vertices, we render a simple quad and compute the distance
// to the shape boundary in the fragment shader.

struct Uniforms {
    screen_size: vec2<f32>,
}

struct VertexInput {
    @location(0) pos: vec2<f32>,  // Unit quad vertices: [-1, 1]
}

struct InstanceInput {
    @location(1) center: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) translation: vec2<f32>,
    @location(4) rotation: f32,
    @location(5) transform_origin: vec2<f32>,
    @location(6) scale: f32,
    @location(7) fill_color: vec4<f32>,
    @location(8) stroke_color: vec4<f32>,
    @location(9) stroke_width: f32,
    @location(10) shape_corner_type: u32,
    @location(11) params12: vec2<f32>,  // param1, param2
    @location(12) params34: vec2<f32>,  // param3, param4
    @location(13) params56: vec2<f32>,  // param5, param6
    @location(14) stroke_offset: f32,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) local_pos: vec2<f32>,      // Position relative to rect center
    @location(2) fill_color: vec4<f32>,
    @location(3) stroke_color: vec4<f32>,
    @location(4) stroke_width: f32,
    @location(5) @interpolate(flat) shape_corner_type: u32,
    @location(6) params12: vec2<f32>,
    @location(7) params34: vec2<f32>,
    @location(8) half_size: vec2<f32>,
    @location(9) scale: f32,
    @location(10) params56: vec2<f32>,
    @location(11) stroke_offset: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Expand unit quad to local-space rectangle
    // For Outset/Centered alignments, stroke can extend beyond shape edge
    // Calculate maximum extent the stroke can reach
    let max_stroke_extent = max(0.0, inst.stroke_width + abs(inst.stroke_offset));
    let padding = select(0.0, max_stroke_extent, inst.stroke_width > 0.0);
    let expanded_size = inst.half_size + vec2<f32>(padding);

    // Apply transforms: Scale → Rotate → Translate (around transform origin)
    // 1. Start with center position + translation
    let translated_center = inst.center + inst.translation;
    let local_pos = translated_center + vert.pos * expanded_size;

    // 2. Translate to transform origin for scale and rotation
    let centered = local_pos - inst.transform_origin;

    // 3. Scale
    let scaled = centered * inst.scale;

    // 4. Rotate (clockwise positive, CSS convention)
    let cos_r = cos(inst.rotation);
    let sin_r = sin(inst.rotation);
    let rotated = vec2<f32>(
        scaled.x * cos_r + scaled.y * sin_r,
        -scaled.x * sin_r + scaled.y * cos_r
    );

    // 5. Translate back from origin
    out.world_pos = rotated + inst.transform_origin;

    // Convert to normalized device coordinates (NDC)
    // Add 0.5 to world_pos to account for pixel centers being at half-integer coordinates
    let ndc = ((out.world_pos + 0.5) / uniforms.screen_size) * 2.0 - 1.0;
    out.clip_pos = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);

    // Pass through instance data to fragment shader
    // local_pos must use expanded_size so interpolation correctly represents
    // fragment position relative to center (SDF uses half_size for boundary)
    out.local_pos = vert.pos * expanded_size;
    out.fill_color = inst.fill_color;
    out.stroke_color = inst.stroke_color;
    out.stroke_width = inst.stroke_width;
    out.shape_corner_type = inst.shape_corner_type;
    out.params12 = inst.params12;
    out.params34 = inst.params34;
    out.half_size = inst.half_size;
    out.scale = inst.scale;
    out.params56 = inst.params56;
    out.stroke_offset = inst.stroke_offset;

    return out;
}

// ============================================================================
// SDF Functions
// ============================================================================

/// Signed distance to a box (sharp corners)
/// Returns negative inside, positive outside, zero on the boundary
/// Based on Inigo Quilez's formula: https://iquilezles.org/articles/distfunctions2d/
fn sd_box(p: vec2<f32>, size: vec2<f32>) -> f32 {
    let d = abs(p) - size;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

/// Signed distance to a rounded box (circular arc corners)
/// Based on Inigo Quilez's formula: https://iquilezles.org/articles/distfunctions2d/
fn sd_rounded_box(p: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

/// Signed distance to a chamfered box (diagonal cut corners at 45°)
/// Based on Inigo Quilez's formula: https://iquilezles.org/articles/distfunctions2d/
fn sd_chamfer_box(p: vec2<f32>, size: vec2<f32>, chamfer: f32) -> f32 {
    var p_local = abs(p) - size;

    // Swap x/y to always work with the diagonal case
    if p_local.y > p_local.x {
        p_local = vec2<f32>(p_local.y, p_local.x);
    }

    p_local.y = p_local.y + chamfer;
    let k = 1.0 - sqrt(2.0);

    if p_local.y < 0.0 && p_local.y + p_local.x * k < 0.0 {
        return p_local.x;
    }
    if p_local.x < p_local.y {
        return (p_local.x + p_local.y) * sqrt(0.5);
    }
    return length(p_local);
}

/// Signed distance to an inverse rounded box (concave circular corners)
/// This creates a shape like a ticket with punched corners
fn sd_inverse_round_box(p: vec2<f32>, size: vec2<f32>, radius: f32, stroke_width: f32) -> f32 {
    // Inverse rounded corners: circles centered at the corners carve into the rectangle
    // Creating concave (inward-curving) corners like a ticket punch
    let p_abs = abs(p);

    // Distance to corner point
    let to_corner = p_abs - size;
    let to_corner_cut = p_abs - (size + radius + stroke_width / 2.0);

    // Box boundary distance (compute once to avoid discontinuities)
    let box_dist = max(to_corner.x, to_corner.y);

    // In corner region (close enough to corner to be affected by circle)
    let corner_dist = -radius - stroke_width;
    let close_to_corner = to_corner.x > corner_dist && to_corner.y > corner_dist;
    if close_to_corner {
        // Circle centered at corner (size.x, size.y)
        let circle_dist = length(p_abs - size) - radius;

        // The shape is the box MINUS circles at corners
        // max(box, -circle) carves the circle out of the box
        return max(box_dist, -circle_dist);
    }

    // Outside corner influence, just use box
    return box_dist;
}

/// Signed distance to a squircle box (superellipse corners)
/// Uses power distance approximation: |x|^n + |y|^n = r^n where n = 2 + smoothness
/// Note: This is an approximation as exact squircle SDF has no closed-form solution
/// Phase 1.4 REVERTED: pow() version kept for visual accuracy
fn sd_squircle_box(p: vec2<f32>, size: vec2<f32>, radius: f32, smoothness: f32) -> f32 {
    let n = 2.0 + smoothness;
    let corner_offset = size - vec2<f32>(radius);
    let p_corner = abs(p) - corner_offset;

    // If we're not in a corner region, use simple box distance
    if p_corner.x <= 0.0 || p_corner.y <= 0.0 {
        return sd_box(p, size);
    }

    // In corner region, compute superellipse approximation
    let p_abs = abs(p_corner);
    let power_sum = pow(p_abs.x, n) + pow(p_abs.y, n);
    let power_dist = pow(power_sum, 1.0 / n);

    return power_dist - radius;
}

/// Signed distance to a triangle defined by 3 vertices
/// Based on Inigo Quilez's formula: https://iquilezles.org/articles/distfunctions2d/
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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Compute signed distance based on shape type
    // dist = distance to stroke boundary, fill_dist = distance to original shape boundary
    var dist: f32;
    var fill_dist: f32;

    if in.shape_corner_type == 100u {
        // Triangle - vertices stored in params as world-space coordinates
        let tri_v0 = in.params12;
        let tri_v1 = in.params34;
        let tri_v2 = in.params56;
        dist = sd_triangle(in.world_pos, tri_v0, tri_v1, tri_v2);
        fill_dist = dist; // Triangle doesn't support stroke offset yet
    } else {
        // Rectangle - compute distance based on corner type
        let corner_type = in.shape_corner_type;
        let corner_param1 = in.params12.x;
        let corner_param2 = in.params12.y;

        // Calculate stroke boundary with offset for alignment
        // The stroke extends stroke_width on each side of dist=0, so we add stroke_width/2
        // to shift the boundary so the stroke aligns correctly
        let stroke_boundary = in.half_size - in.stroke_offset + in.stroke_width * 0.5;

        // Scale corner size for stroke to maintain parallel contours
        // When stroke is offset outward, corner size increases proportionally
        let corner_offset = -in.stroke_offset + in.stroke_width * 0.5;
        let cut_corner_offset = corner_offset * 0.4023689; // sqrt(2)/2
        let stroke_corner_param1 = max(0.0, corner_param1 + corner_offset);
        let stroke_corner_param2 = corner_param2; // smoothness doesn't scale

        switch corner_type {
            case 0u: {  // None (sharp corners)
                dist = sd_box(in.local_pos, stroke_boundary);
                fill_dist = sd_box(in.local_pos, in.half_size);
            }
            case 1u: {  // Round (circular arcs)
                dist = sd_rounded_box(in.local_pos, stroke_boundary, stroke_corner_param1);
                fill_dist = sd_rounded_box(in.local_pos, in.half_size, corner_param1);
            }
            case 2u: {  // Cut (chamfered at 45°)
                dist = sd_chamfer_box(in.local_pos, stroke_boundary, stroke_corner_param1 - cut_corner_offset);
                fill_dist = sd_chamfer_box(in.local_pos, in.half_size, corner_param1);
            }
            case 3u: {  // InverseRound (concave arcs)
                dist = sd_inverse_round_box(in.local_pos, stroke_boundary, stroke_corner_param1, in.stroke_width);
                fill_dist = sd_inverse_round_box(in.local_pos, in.half_size, corner_param1, in.stroke_width);
            }
            case 4u: {  // Squircle (superellipse)
                dist = sd_squircle_box(in.local_pos, stroke_boundary, stroke_corner_param1, stroke_corner_param2);
                fill_dist = sd_squircle_box(in.local_pos, in.half_size, corner_param1, corner_param2);
            }
            default: {
                // Fallback to sharp corners
                dist = sd_box(in.local_pos, stroke_boundary);
                fill_dist = sd_box(in.local_pos, in.half_size);
            }
        }
    }

    // Compute AA width using screen-space derivatives for pixel-perfect antialiasing
    // This accounts for rotation, non-uniform scaling, and perspective
    // The gradient magnitude tells us how many pixels per unit of distance
    let aa_width = length(vec2<f32>(dpdx(dist), dpdy(dist)));

    // Phase 1.2: Early discard for pixels definitely outside shape (10-15% improvement)
    // When stroke is inset, we need to check against fill boundary, not stroke boundary
    // Otherwise we'd discard fill pixels that should be visible
    let outer_dist = min(dist, fill_dist);
    if outer_dist > aa_width * 1.5 {
        discard;
    }

    // Stroke rendering: stroke is a ring, fill is the interior
    // We need to choose stroke OR fill, not blend them

    var final_color: vec4<f32>;

    let stroke_dist = abs(dist) - in.stroke_width;
    var stroke_condition = stroke_dist < 0.0;
    if in.stroke_offset > 0.0 {
        stroke_condition = stroke_dist < -in.stroke_width/2;
    }

    if stroke_condition {
        // In stroke ring - render stroke with AA
        let alpha = 1.0 - smoothstep(-aa_width, aa_width, stroke_dist);
        final_color = in.stroke_color * alpha;
    } else {
        // Outside stroke ring - render fill (if inside original shape) with AA
        let alpha = 1.0 - smoothstep(-aa_width, aa_width, fill_dist);
        final_color = in.fill_color * alpha;
    }

    return final_color;
}
