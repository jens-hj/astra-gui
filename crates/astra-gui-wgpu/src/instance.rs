use astra_gui::{ClippedShape, CornerShape, Shape, StyledRect};

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
    /// Transform origin (absolute pixels from rect origin)
    pub transform_origin: [f32; 2],
    /// Padding for alignment
    pub _padding1: f32,
    /// Fill color (RGBA, normalized to 0-255)
    pub fill_color: [u8; 4],
    /// Stroke color (RGBA, normalized to 0-255)
    pub stroke_color: [u8; 4],
    /// Stroke width in pixels (0 = no stroke)
    pub stroke_width: f32,
    /// Corner type: 0=None, 1=Round, 2=Cut, 3=InverseRound, 4=Squircle
    pub corner_type: u32,
    /// First corner parameter (radius or extent)
    pub corner_param1: f32,
    /// Second corner parameter (smoothness for squircle, unused for others)
    pub corner_param2: f32,
}

impl RectInstance {
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
            // transform_origin: vec2<f32> at location 5
            wgpu::VertexAttribute {
                offset: 28,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x2,
            },
            // _padding1: f32 (skip, location 6 unused)
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
            // corner_type: u32 at location 10
            wgpu::VertexAttribute {
                offset: 52,
                shader_location: 10,
                format: wgpu::VertexFormat::Uint32,
            },
            // corner_param1: f32 at location 11
            wgpu::VertexAttribute {
                offset: 56,
                shader_location: 11,
                format: wgpu::VertexFormat::Float32,
            },
            // corner_param2: f32 at location 12
            wgpu::VertexAttribute {
                offset: 60,
                shader_location: 12,
                format: wgpu::VertexFormat::Float32,
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

        // Resolve transform origin to absolute pixels
        let width = clipped.node_rect.max[0] - clipped.node_rect.min[0];
        let height = clipped.node_rect.max[1] - clipped.node_rect.min[1];
        let (origin_x, origin_y) = clipped.transform.origin.resolve(width, height);
        let transform_origin = [origin_x, origin_y];

        // Convert fill color
        let fill_color = [
            (rect.fill.r * 255.0).round().clamp(0.0, 255.0) as u8,
            (rect.fill.g * 255.0).round().clamp(0.0, 255.0) as u8,
            (rect.fill.b * 255.0).round().clamp(0.0, 255.0) as u8,
            (rect.fill.a * 255.0).round().clamp(0.0, 255.0) as u8,
        ];

        // Convert stroke (if present)
        let (stroke_color, stroke_width) = if let Some(stroke) = &rect.stroke {
            (
                [
                    (stroke.color.r * 255.0).round().clamp(0.0, 255.0) as u8,
                    (stroke.color.g * 255.0).round().clamp(0.0, 255.0) as u8,
                    (stroke.color.b * 255.0).round().clamp(0.0, 255.0) as u8,
                    (stroke.color.a * 255.0).round().clamp(0.0, 255.0) as u8,
                ],
                stroke.width,
            )
        } else {
            ([0, 0, 0, 0], 0.0)
        };

        // Convert corner shape to type + parameters
        let (corner_type, param1, param2) = match rect.corner_shape {
            CornerShape::None => (0, 0.0, 0.0),
            CornerShape::Round(radius) => (1, radius, 0.0),
            CornerShape::Cut(distance) => (2, distance, 0.0),
            CornerShape::InverseRound(radius) => (3, radius, 0.0),
            CornerShape::Squircle { radius, smoothness } => (4, radius, smoothness),
        };

        Self {
            center,
            half_size,
            translation,
            rotation,
            transform_origin,
            _padding1: 0.0,
            fill_color,
            stroke_color,
            stroke_width,
            corner_type,
            corner_param1: param1,
            corner_param2: param2,
        }
    }
}
