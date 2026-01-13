use astra_gui::{AntiAliasing, ClippedShape, CornerShape, Shape};

/// Instance data for SDF-based rectangle rendering.
///
/// Each instance represents a single rectangle with all the parameters needed
/// to render it using signed distance fields (SDFs) in the fragment shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectInstance {
    /// Center position in untransformed screen-space pixels
    pub center: [f32; 2],
    /// Half-size (width/2, height/2) in pixels
    pub half_size: [f32; 2],
    /// Translation offset (post-layout transform)
    pub translation: [f32; 2],
    /// Rotation in radians (clockwise positive, CSS convention)
    pub rotation: f32,
    /// Uniform scale factor (1.0 = no scale)
    pub scale: f32,
    /// Transform origin (absolute pixels from rect origin)
    pub transform_origin: [f32; 2],
    /// Fill color (RGBA, normalized to 0-255)
    pub fill_color: [u8; 4],
    /// Stroke color (RGBA, normalized to 0-255)
    pub stroke_color: [u8; 4],
    /// Stroke width in pixels (0 = no stroke)
    pub stroke_width: f32,
    /// Shape/corner type:
    /// For rectangles: 0=None, 1=Round, 2=Cut, 3=InverseRound, 4=Squircle
    /// For triangles: 100 = Triangle
    pub shape_corner_type: u32,
    /// Parameter 1: corner radius for rects, or triangle v0.x for triangles
    pub param1: f32,
    /// Parameter 2: corner smoothness for rects, or triangle v0.y for triangles
    pub param2: f32,
    /// Parameter 3: unused for rects, or triangle v1.x for triangles
    pub param3: f32,
    /// Parameter 4: unused for rects, or triangle v1.y for triangles
    pub param4: f32,
    /// Parameter 5: unused for rects, or triangle v2.x for triangles
    pub param5: f32,
    /// Parameter 6: unused for rects, or triangle v2.y for triangles
    pub param6: f32,
    /// Stroke offset for alignment (0 = centered, positive = outward, negative = inward)
    pub stroke_offset: f32,
    /// Anti-aliasing mode: 0 = None, 1 = Analytical
    pub anti_aliasing: u32,
}

impl RectInstance {
    /// Create a triangle instance from a ClippedShape containing a triangle
    pub fn from_triangle(clipped: &ClippedShape) -> Self {
        let triangle = match &clipped.shape {
            Shape::Triangle(tri) => tri,
            _ => panic!("from_triangle can only be created from Shape::Triangle"),
        };

        // Get triangle vertices in world space
        let vertices = triangle.vertices();

        // Calculate bounding box for the triangle
        let min_x = vertices[0][0].min(vertices[1][0]).min(vertices[2][0]);
        let max_x = vertices[0][0].max(vertices[1][0]).max(vertices[2][0]);
        let min_y = vertices[0][1].min(vertices[1][1]).min(vertices[2][1]);
        let max_y = vertices[0][1].max(vertices[1][1]).max(vertices[2][1]);

        // Center and half-size of bounding box
        let center = [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5];
        let half_size = [(max_x - min_x) * 0.5, (max_y - min_y) * 0.5];

        // Convert vertices to world space (they already are from triangle.vertices())
        let tri_v0 = vertices[0];
        let tri_v1 = vertices[1];
        let tri_v2 = vertices[2];

        // Apply opacity to colors
        let fill_color = [
            (triangle.fill.r * 255.0).round().clamp(0.0, 255.0) as u8,
            (triangle.fill.g * 255.0).round().clamp(0.0, 255.0) as u8,
            (triangle.fill.b * 255.0).round().clamp(0.0, 255.0) as u8,
            ((triangle.fill.a * clipped.opacity) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8,
        ];

        let (stroke_color, stroke_width, stroke_offset) = if let Some(stroke) = &triangle.stroke {
            let width = max_x - min_x;
            let resolved_width = stroke
                .width
                .try_resolve_with_scale(width, 1.0)
                .unwrap_or(0.0);

            let offset = stroke.alignment.calculate_offset(resolved_width);

            (
                [
                    (stroke.color.r * 255.0).round().clamp(0.0, 255.0) as u8,
                    (stroke.color.g * 255.0).round().clamp(0.0, 255.0) as u8,
                    (stroke.color.b * 255.0).round().clamp(0.0, 255.0) as u8,
                    ((stroke.color.a * clipped.opacity) * 255.0)
                        .round()
                        .clamp(0.0, 255.0) as u8,
                ],
                resolved_width,
                offset,
            )
        } else {
            ([0, 0, 0, 0], 0.0, 0.0)
        };

        // Extract transform data
        let translation = [
            clipped.transform.translation.x,
            clipped.transform.translation.y,
        ];
        let rotation = clipped.transform.rotation;
        let scale = clipped.transform.scale;

        let transform_origin = if let Some(abs_origin) = clipped.transform.absolute_origin {
            abs_origin
        } else {
            let width = max_x - min_x;
            let height = max_y - min_y;
            let (origin_x, origin_y) = clipped.transform.origin.resolve(width, height);
            [min_x + origin_x, min_y + origin_y]
        };

        Self {
            center,
            half_size,
            translation,
            rotation,
            scale,
            transform_origin,
            fill_color,
            stroke_color,
            stroke_width,
            shape_corner_type: 100, // 100 = Triangle
            param1: tri_v0[0],
            param2: tri_v0[1],
            param3: tri_v1[0],
            param4: tri_v1[1],
            param5: tri_v2[0],
            param6: tri_v2[1],
            stroke_offset,
            anti_aliasing: match triangle.anti_aliasing {
                AntiAliasing::None => 0,
                AntiAliasing::Analytical => 1,
            },
        }
    }

    /// Vertex buffer layout for instance attributes
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            // center: vec2<f32> at location 1
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            // half_size: vec2<f32> at location 2
            wgpu::VertexAttribute {
                offset: 8,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            // translation: vec2<f32> at location 3
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x2,
            },
            // rotation: f32 at location 4
            wgpu::VertexAttribute {
                offset: 24,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32,
            },
            // scale: f32 at location 6
            wgpu::VertexAttribute {
                offset: 28,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32,
            },
            // transform_origin: vec2<f32> at location 5
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x2,
            },
            // fill_color: vec4<f32> at location 7 (Unorm8x4)
            wgpu::VertexAttribute {
                offset: 40,
                shader_location: 7,
                format: wgpu::VertexFormat::Unorm8x4,
            },
            // stroke_color: vec4<f32> at location 8 (Unorm8x4)
            wgpu::VertexAttribute {
                offset: 44,
                shader_location: 8,
                format: wgpu::VertexFormat::Unorm8x4,
            },
            // stroke_width: f32 at location 9
            wgpu::VertexAttribute {
                offset: 48,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32,
            },
            // shape_corner_type: u32 at location 10
            wgpu::VertexAttribute {
                offset: 52,
                shader_location: 10,
                format: wgpu::VertexFormat::Uint32,
            },
            // params12: vec2<f32> (param1, param2) at location 11
            wgpu::VertexAttribute {
                offset: 56,
                shader_location: 11,
                format: wgpu::VertexFormat::Float32x2,
            },
            // params34: vec2<f32> (param3, param4) at location 12
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 12,
                format: wgpu::VertexFormat::Float32x2,
            },
            // params56: vec2<f32> (param5, param6) at location 13
            wgpu::VertexAttribute {
                offset: 72,
                shader_location: 13,
                format: wgpu::VertexFormat::Float32x2,
            },
            // stroke_offset: f32 at location 14
            wgpu::VertexAttribute {
                offset: 80,
                shader_location: 14,
                format: wgpu::VertexFormat::Float32,
            },
            // anti_aliasing: u32 at location 15
            wgpu::VertexAttribute {
                offset: 84,
                shader_location: 15,
                format: wgpu::VertexFormat::Uint32,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}

impl From<&ClippedShape> for RectInstance {
    fn from(clipped: &ClippedShape) -> Self {
        // Extract StyledRect from the shape
        let rect = match &clipped.shape {
            Shape::Rect(styled_rect) => styled_rect,
            _ => panic!("RectInstance can only be created from Shape::Rect"),
        };

        // Calculate center and half-size from untransformed rect
        let center = [
            (clipped.node_rect.min[0] + clipped.node_rect.max[0]) * 0.5,
            (clipped.node_rect.min[1] + clipped.node_rect.max[1]) * 0.5,
        ];
        let half_size = [
            (clipped.node_rect.max[0] - clipped.node_rect.min[0]) * 0.5,
            (clipped.node_rect.max[1] - clipped.node_rect.min[1]) * 0.5,
        ];

        // Extract transform data
        let translation = [
            clipped.transform.translation.x,
            clipped.transform.translation.y,
        ];
        let rotation = clipped.transform.rotation;
        let scale = clipped.transform.scale;

        // Resolve transform origin to absolute world-space pixels
        // If absolute_origin is set (from hierarchical rotation), use it
        // Otherwise, resolve the percentage-based origin relative to this rect
        let transform_origin = if let Some(abs_origin) = clipped.transform.absolute_origin {
            abs_origin
        } else {
            let width = clipped.node_rect.max[0] - clipped.node_rect.min[0];
            let height = clipped.node_rect.max[1] - clipped.node_rect.min[1];
            let (origin_x, origin_y) = clipped.transform.origin.resolve(width, height);
            [
                clipped.node_rect.min[0] + origin_x,
                clipped.node_rect.min[1] + origin_y,
            ]
        };

        // Apply opacity from ClippedShape to fill color
        let fill_color = [
            (rect.fill.r * 255.0).round().clamp(0.0, 255.0) as u8,
            (rect.fill.g * 255.0).round().clamp(0.0, 255.0) as u8,
            (rect.fill.b * 255.0).round().clamp(0.0, 255.0) as u8,
            ((rect.fill.a * clipped.opacity) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8,
        ];

        // Convert stroke (if present) and apply opacity
        let (stroke_color, stroke_width, stroke_offset) = if let Some(stroke) = &rect.stroke {
            // Resolve stroke width to f32 (should already be in physical pixels at this point)
            let width = clipped.node_rect.max[0] - clipped.node_rect.min[0];
            let resolved_width = stroke
                .width
                .try_resolve_with_scale(width, 1.0)
                .unwrap_or(0.0);

            let offset = stroke.alignment.calculate_offset(resolved_width);

            (
                [
                    (stroke.color.r * 255.0).round().clamp(0.0, 255.0) as u8,
                    (stroke.color.g * 255.0).round().clamp(0.0, 255.0) as u8,
                    (stroke.color.b * 255.0).round().clamp(0.0, 255.0) as u8,
                    ((stroke.color.a * clipped.opacity) * 255.0)
                        .round()
                        .clamp(0.0, 255.0) as u8,
                ],
                resolved_width,
                offset,
            )
        } else {
            ([0, 0, 0, 0], 0.0, 0.0)
        };

        // Convert corner shape to type + parameters
        let (corner_type, param1, param2) = match rect.corner_shape {
            CornerShape::None => (0, 0.0, 0.0),
            CornerShape::Round(radius) => (1, radius.resolve_physical_or_zero(1.0), 0.0),
            CornerShape::Cut(distance) => (2, distance.resolve_physical_or_zero(1.0), 0.0),
            CornerShape::InverseRound(radius) => (3, radius.resolve_physical_or_zero(1.0), 0.0),
            CornerShape::Squircle { radius, smoothness } => {
                (4, radius.resolve_physical_or_zero(1.0), smoothness)
            }
        };

        Self {
            center,
            half_size,
            translation,
            rotation,
            scale,
            transform_origin,
            fill_color,
            stroke_color,
            stroke_width,
            shape_corner_type: corner_type,
            param1,
            param2,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            stroke_offset,
            anti_aliasing: match rect.anti_aliasing {
                AntiAliasing::None => 0,
                AntiAliasing::Analytical => 1,
            },
        }
    }
}
