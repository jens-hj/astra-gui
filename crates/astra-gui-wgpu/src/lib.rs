//! # astra-gui-wgpu
//!
//! WGPU rendering backend for astra-gui.

mod events;
mod input;
mod instance;
mod interactive_state;

#[cfg(feature = "text-cosmic")]
mod text;

mod vertex;

pub use events::*;
pub use input::*;
pub use interactive_state::*;

// Re-export keyboard and mouse types for use in interactive components
pub use winit::event::MouseButton;
pub use winit::keyboard::{Key, NamedKey};

use astra_gui::{
    ClippedShape, Color, CornerShape, FullOutput, HorizontalAlign, Rect, Shape, Size, Stroke,
    StyledRect, Tessellator, Transform2D, VerticalAlign, ZIndex,
};
use instance::RectInstance;
use vertex::WgpuVertex;

/// Rendering mode for rectangles
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    /// Use SDF (Signed Distance Field) rendering for analytical anti-aliasing.
    /// Best quality, especially for strokes and rounded corners.
    Sdf,
    /// Use mesh tessellation for rendering.
    /// More compatible but lower quality anti-aliasing.
    Mesh,
    /// Automatically choose based on shape complexity (currently defaults to SDF).
    Auto,
}

#[cfg(feature = "text-cosmic")]
use astra_gui_text as gui_text;
#[cfg(feature = "text-cosmic")]
use gui_text::TextEngine;

/// A draw call with scissor rect for clipped rendering.
#[derive(Clone, Copy, Debug)]
struct ClippedDraw {
    scissor: (u32, u32, u32, u32),
    index_start: u32,
    index_end: u32,
}

/// A draw call for SDF instances with scissor rect.
#[derive(Clone, Copy, Debug)]
struct SdfDraw {
    scissor: (u32, u32, u32, u32),
    instance_start: u32,
    instance_count: u32,
}

/// A rendering layer containing shapes at a specific z-index with rendering ranges.
#[derive(Debug)]
struct RenderLayer<'a> {
    #[allow(dead_code)]
    z_index: astra_gui::ZIndex,
    shapes: Vec<&'a astra_gui::ClippedShape>,
}

const INITIAL_VERTEX_CAPACITY: usize = 1024;
const INITIAL_INDEX_CAPACITY: usize = 2048;

#[cfg(feature = "text-cosmic")]
const INITIAL_TEXT_VERTEX_CAPACITY: usize = 4096;
#[cfg(feature = "text-cosmic")]
const INITIAL_TEXT_INDEX_CAPACITY: usize = 8192;

#[cfg(feature = "text-cosmic")]
const ATLAS_SIZE_PX: u32 = 4096;
#[cfg(feature = "text-cosmic")]
const ATLAS_PADDING_PX: u32 = 1;

/// WGPU renderer for astra-gui
pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    tessellator: Tessellator,
    vertex_capacity: usize,
    index_capacity: usize,
    wgpu_vertices: Vec<WgpuVertex>,

    // Performance optimization: track previous frame sizes to pre-allocate buffers
    last_frame_vertex_count: usize,
    last_frame_index_count: usize,

    // Persistent buffers reused across frames
    frame_indices: Vec<u32>,
    frame_geometry_draws: Vec<ClippedDraw>,

    // Rendering mode configuration
    render_mode: RenderMode,

    // SDF rendering pipeline (analytic anti-aliasing)
    sdf_pipeline: wgpu::RenderPipeline,
    sdf_instance_buffer: wgpu::Buffer,
    sdf_instance_capacity: usize,
    sdf_instances: Vec<RectInstance>,
    sdf_draws: Vec<SdfDraw>, // Track clip rects for SDF instances
    sdf_quad_vertex_buffer: wgpu::Buffer,
    sdf_quad_index_buffer: wgpu::Buffer,
    last_frame_sdf_instance_count: usize,

    #[cfg(feature = "text-cosmic")]
    text_pipeline: wgpu::RenderPipeline,
    #[cfg(feature = "text-cosmic")]
    text_vertex_buffer: wgpu::Buffer,
    #[cfg(feature = "text-cosmic")]
    text_index_buffer: wgpu::Buffer,
    #[cfg(feature = "text-cosmic")]
    text_vertex_capacity: usize,
    #[cfg(feature = "text-cosmic")]
    text_index_capacity: usize,
    #[cfg(feature = "text-cosmic")]
    text_vertices: Vec<text::vertex::TextVertex>,
    #[cfg(feature = "text-cosmic")]
    text_indices: Vec<u32>,
    #[cfg(feature = "text-cosmic")]
    last_frame_text_vertex_count: usize,
    #[cfg(feature = "text-cosmic")]
    last_frame_text_index_count: usize,
    #[cfg(feature = "text-cosmic")]
    last_frame_text_draw_count: usize,

    // Glyph atlas (R8 alpha mask)
    #[cfg(feature = "text-cosmic")]
    atlas_texture: wgpu::Texture,
    #[cfg(feature = "text-cosmic")]
    atlas_bind_group: wgpu::BindGroup,
    #[cfg(feature = "text-cosmic")]
    atlas_bind_group_layout: wgpu::BindGroupLayout,
    #[cfg(feature = "text-cosmic")]
    atlas_sampler: wgpu::Sampler,
    #[cfg(feature = "text-cosmic")]
    atlas: text::atlas::GlyphAtlas,

    // Backend-agnostic text shaping/raster engine (Inter via astra-gui-fonts).
    #[cfg(feature = "text-cosmic")]
    text_engine: gui_text::Engine,

    // Text shaping cache - stores pre-shaped text to avoid expensive reshaping every frame
    // Key: (text, font_size, width, height, wrap, line_height * 100)
    // NOTE: Only caches ShapedText, NOT LinePlacement (which contains absolute positions)
    #[cfg(feature = "text-cosmic")]
    shape_cache: std::collections::HashMap<
        (String, u32, u32, u32, astra_gui::Wrap, u32),
        gui_text::ShapedText,
    >,

    // Glyph metrics cache - stores bearing, size, AND atlas placement to avoid lookups
    // Key: GlyphKey (font_id, glyph_id, px_size, subpixel)
    #[cfg(feature = "text-cosmic")]
    glyph_metrics_cache: std::collections::HashMap<
        text::atlas::GlyphKey,
        ([i32; 2], [u32; 2], text::atlas::PlacedGlyph), // (bearing_px, size_px, placement)
    >,

    // Atlas resize tracking
    #[cfg(feature = "text-cosmic")]
    atlas_needs_resize: bool,

    // For proactive estimation of atlas space needs
    #[cfg(feature = "text-cosmic")]
    avg_glyph_size_estimate_px: u32,

    // GPU texture size limit
    #[cfg(feature = "text-cosmic")]
    max_texture_dimension_2d: u32,

    // Track if we've hit the GPU limit to avoid spamming warnings
    #[cfg(feature = "text-cosmic")]
    atlas_at_gpu_limit: bool,
}

impl Renderer {
    /// Create a new renderer with the default render mode (Auto/SDF)
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        Self::with_render_mode(device, surface_format, RenderMode::Auto)
    }

    /// Create a new renderer with a specific render mode
    pub fn with_render_mode(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        render_mode: RenderMode,
    ) -> Self {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Astra UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
        });

        // Create uniform buffer (screen size)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Astra UI Uniform Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout (globals)
        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Astra UI Globals Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create bind group (globals)
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Astra UI Globals Bind Group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout (geometry)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Astra UI Pipeline Layout"),
            bind_group_layouts: &[&globals_bind_group_layout],
            immediate_size: 0,
        });

        // Create render pipeline (geometry)
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Astra UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[WgpuVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Create initial buffers (geometry)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Astra UI Vertex Buffer"),
            size: (INITIAL_VERTEX_CAPACITY * std::mem::size_of::<WgpuVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Astra UI Index Buffer"),
            size: (INITIAL_INDEX_CAPACITY * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create SDF pipeline and buffers
        let sdf_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Astra UI SDF Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui_sdf.wgsl").into()),
        });

        let sdf_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Astra UI SDF Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sdf_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer: unit quad
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    // Instance buffer
                    RectInstance::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sdf_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Unit quad vertices: [-1, -1] to [1, 1]
        let quad_vertices: &[[f32; 2]] = &[
            [-1.0, -1.0], // bottom-left
            [1.0, -1.0],  // bottom-right
            [1.0, 1.0],   // top-right
            [-1.0, 1.0],  // top-left
        ];
        let quad_indices: &[u32] = &[0, 1, 2, 0, 2, 3];

        let sdf_quad_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Astra UI SDF Quad Vertex Buffer"),
            size: (quad_vertices.len() * std::mem::size_of::<[f32; 2]>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        sdf_quad_vertex_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(quad_vertices));
        sdf_quad_vertex_buffer.unmap();

        let sdf_quad_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Astra UI SDF Quad Index Buffer"),
            size: (quad_indices.len() * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        sdf_quad_index_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(quad_indices));
        sdf_quad_index_buffer.unmap();

        const INITIAL_SDF_INSTANCE_CAPACITY: usize = 256;
        let sdf_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Astra UI SDF Instance Buffer"),
            size: (INITIAL_SDF_INSTANCE_CAPACITY * std::mem::size_of::<RectInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        #[cfg(feature = "text-cosmic")]
        let (
            text_pipeline,
            text_vertex_buffer,
            text_index_buffer,
            atlas_texture,
            atlas_bind_group,
            atlas_bind_group_layout,
            atlas_sampler,
            atlas,
        ) = {
            // Load text shader
            let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Astra UI Text Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text.wgsl").into()),
            });

            // Atlas texture (R8)
            let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Astra UI Glyph Atlas"),
                size: wgpu::Extent3d {
                    width: ATLAS_SIZE_PX,
                    height: ATLAS_SIZE_PX,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Astra UI Glyph Atlas Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                // Debug atlas is a nearest-neighbor bitmap; keep sampling nearest to avoid
                // filter smearing and edge artifacts at small sizes.
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            });

            let atlas_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Astra UI Text Atlas Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

            let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Astra UI Text Atlas Bind Group"),
                layout: &atlas_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&atlas_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                    },
                ],
            });

            // Pipeline layout (text): globals + atlas
            let text_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Astra UI Text Pipeline Layout"),
                    bind_group_layouts: &[&globals_bind_group_layout, &atlas_bind_group_layout],
                    immediate_size: 0,
                });

            let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Astra UI Text Pipeline"),
                layout: Some(&text_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &text_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[text::vertex::TextVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &text_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            let text_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Astra UI Text Vertex Buffer"),
                size: (INITIAL_TEXT_VERTEX_CAPACITY
                    * std::mem::size_of::<text::vertex::TextVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let text_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Astra UI Text Index Buffer"),
                size: (INITIAL_TEXT_INDEX_CAPACITY * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let atlas =
                text::atlas::GlyphAtlas::new(ATLAS_SIZE_PX, ATLAS_SIZE_PX, ATLAS_PADDING_PX);

            (
                text_pipeline,
                text_vertex_buffer,
                text_index_buffer,
                atlas_texture,
                atlas_bind_group,
                atlas_bind_group_layout,
                atlas_sampler,
                atlas,
            )
        };

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            tessellator: Tessellator::new(),
            vertex_capacity: INITIAL_VERTEX_CAPACITY,
            index_capacity: INITIAL_INDEX_CAPACITY,
            wgpu_vertices: Vec::new(),
            last_frame_vertex_count: 0,
            last_frame_index_count: 0,

            frame_indices: Vec::new(),
            frame_geometry_draws: Vec::new(),

            render_mode,

            sdf_pipeline,
            sdf_instance_buffer,
            sdf_instance_capacity: INITIAL_SDF_INSTANCE_CAPACITY,
            sdf_instances: Vec::new(),
            sdf_draws: Vec::new(),
            sdf_quad_vertex_buffer,
            sdf_quad_index_buffer,
            last_frame_sdf_instance_count: 0,

            #[cfg(feature = "text-cosmic")]
            text_pipeline,
            #[cfg(feature = "text-cosmic")]
            text_vertex_buffer,
            #[cfg(feature = "text-cosmic")]
            text_index_buffer,
            #[cfg(feature = "text-cosmic")]
            text_vertex_capacity: INITIAL_TEXT_VERTEX_CAPACITY,
            #[cfg(feature = "text-cosmic")]
            text_index_capacity: INITIAL_TEXT_INDEX_CAPACITY,
            #[cfg(feature = "text-cosmic")]
            text_vertices: Vec::new(),
            #[cfg(feature = "text-cosmic")]
            text_indices: Vec::new(),
            #[cfg(feature = "text-cosmic")]
            last_frame_text_vertex_count: 0,
            #[cfg(feature = "text-cosmic")]
            last_frame_text_index_count: 0,
            #[cfg(feature = "text-cosmic")]
            last_frame_text_draw_count: 0,
            #[cfg(feature = "text-cosmic")]
            atlas_texture,
            #[cfg(feature = "text-cosmic")]
            atlas_bind_group,
            #[cfg(feature = "text-cosmic")]
            atlas_bind_group_layout,
            #[cfg(feature = "text-cosmic")]
            atlas_sampler,
            #[cfg(feature = "text-cosmic")]
            atlas,
            #[cfg(feature = "text-cosmic")]
            text_engine: gui_text::Engine::new_default(),
            #[cfg(feature = "text-cosmic")]
            shape_cache: std::collections::HashMap::new(),
            #[cfg(feature = "text-cosmic")]
            glyph_metrics_cache: std::collections::HashMap::new(),
            #[cfg(feature = "text-cosmic")]
            atlas_needs_resize: false,
            #[cfg(feature = "text-cosmic")]
            avg_glyph_size_estimate_px: 32, // Conservative initial estimate
            #[cfg(feature = "text-cosmic")]
            max_texture_dimension_2d: device.limits().max_texture_dimension_2d,
            #[cfg(feature = "text-cosmic")]
            atlas_at_gpu_limit: false,
        }
    }

    /// Get the current render mode
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// Set the render mode
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
    }

    /// Get mutable access to the text engine for measurement
    #[cfg(feature = "text-cosmic")]
    pub fn text_engine_mut(&mut self) -> &mut gui_text::Engine {
        &mut self.text_engine
    }

    #[cfg(feature = "text-cosmic")]
    fn resize_atlas(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Collect all cached glyphs before resize (we need to preserve them)
        let old_glyphs: Vec<(text::atlas::GlyphKey, text::atlas::PlacedGlyph)> = self
            .atlas
            .cached_glyphs()
            .map(|(k, p)| (k.clone(), *p))
            .collect();

        let (old_width, old_height) = self.atlas.dimensions();

        // Exponential growth pattern matching buffer growth in codebase
        let new_size = (old_width.max(old_height) * 2).next_power_of_two();
        let new_size = new_size.min(self.max_texture_dimension_2d);

        // Check if we've hit the GPU limit
        if new_size == old_width && new_size == old_height {
            if !self.atlas_at_gpu_limit {
                eprintln!(
                    "WARNING: Atlas at GPU limit of {}x{}. {} glyphs cached. \
                     Further zoom may cause text to disappear.",
                    new_size,
                    new_size,
                    old_glyphs.len()
                );
                self.atlas_at_gpu_limit = true;
            }
            self.atlas_needs_resize = false;
            return;
        }

        eprintln!(
            "Resizing glyph atlas: {}x{} -> {}x{} ({} cached glyphs, GPU limit: {})",
            old_width,
            old_height,
            new_size,
            new_size,
            old_glyphs.len(),
            self.max_texture_dimension_2d
        );

        // Reset GPU limit flag since we're successfully resizing
        self.atlas_at_gpu_limit = false;

        // Create new larger atlas texture
        self.atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Astra UI Glyph Atlas"),
            size: wgpu::Extent3d {
                width: new_size,
                height: new_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Resize atlas allocator (clears internal state)
        self.atlas.resize_to(new_size, new_size);

        // Re-insert all glyphs (they get new UV coordinates based on new atlas size)
        for (key, old_placed) in &old_glyphs {
            // Get original bitmap dimensions (excluding padding)
            let bitmap_width = ((old_placed.rect_px.width() as i32)
                - (old_placed.padding_px as i32 * 2))
                .max(0) as u32;
            let bitmap_height = ((old_placed.rect_px.height() as i32)
                - (old_placed.padding_px as i32 * 2))
                .max(0) as u32;

            // Re-insert into atlas (gets new placement with corrected UVs)
            match self
                .atlas
                .insert(key.clone(), [bitmap_width, bitmap_height])
            {
                text::atlas::AtlasInsert::Placed(_) => {
                    // Success - will re-rasterize and upload below
                }
                text::atlas::AtlasInsert::AlreadyPresent => {
                    // Shouldn't happen since we cleared, but OK
                }
                text::atlas::AtlasInsert::Full => {
                    eprintln!("ERROR: Glyph still doesn't fit after resize! key={:?}", key);
                    // This is serious - atlas is still too small even after doubling
                    continue;
                }
            }
        }

        // Re-rasterize and upload all glyphs at their new positions
        for (key, _) in &old_glyphs {
            // Get the new placement
            let Some(new_placed) = self.atlas.get(key) else {
                continue;
            };

            // Convert atlas key back to text engine key for rasterization
            let text_key = gui_text::GlyphKey::new(
                gui_text::FontId(key.font_id),
                key.glyph_id,
                key.font_px,
                key.variant as i16,
            );

            // Re-rasterize the glyph
            let Some(bitmap) = self.text_engine.rasterize_glyph(text_key) else {
                continue;
            };

            if bitmap.pixels.is_empty() {
                continue;
            }

            // Upload to new atlas position
            let rect_px = new_placed.rect_px;
            let pad = new_placed.padding_px;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: rect_px.min.x + pad,
                        y: rect_px.min.y + pad,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &bitmap.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bitmap.size_px[0]),
                    rows_per_image: Some(bitmap.size_px[1]),
                },
                wgpu::Extent3d {
                    width: bitmap.size_px[0],
                    height: bitmap.size_px[1],
                    depth_or_array_layers: 1,
                },
            );
        }

        // Update metrics cache with new placements
        // (Keep bearing and size, update placement)
        let mut updated_cache = std::mem::take(&mut self.glyph_metrics_cache);
        for (atlas_key, (_bearing, _size, old_placed)) in updated_cache.iter_mut() {
            if let Some(new_placed) = self.atlas.get(atlas_key) {
                *old_placed = new_placed;
            }
        }
        self.glyph_metrics_cache = updated_cache;

        // Recreate bind group with new texture
        let atlas_view = self
            .atlas_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Astra UI Atlas Bind Group"),
            layout: &self.atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        self.atlas_needs_resize = false;
    }

    /// Group shapes into rendering layers by z-index, with shapes separated by type within each layer.
    fn group_into_layers<'a>(shapes: &'a [astra_gui::ClippedShape]) -> Vec<RenderLayer<'a>> {
        if shapes.is_empty() {
            return Vec::new();
        }

        let mut layers: Vec<RenderLayer> = Vec::new();
        let mut current_z_index = shapes[0].z_index;
        let mut current_shapes = Vec::new();

        for shape in shapes {
            if shape.z_index != current_z_index {
                // Save current layer and start new one
                layers.push(RenderLayer {
                    z_index: current_z_index,
                    shapes: current_shapes,
                });
                current_shapes = Vec::new();
                current_z_index = shape.z_index;
            }

            current_shapes.push(shape);
        }

        // Push final layer
        if !current_shapes.is_empty() {
            layers.push(RenderLayer {
                z_index: current_z_index,
                shapes: current_shapes,
            });
        }

        layers
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        screen_width: f32,
        screen_height: f32,
        output: &FullOutput,
    ) {
        // STAGE 2: Reactive resize from previous frame
        #[cfg(feature = "text-cosmic")]
        if self.atlas_needs_resize {
            self.resize_atlas(device, queue);
        }

        // STAGE 1: Proactive estimation
        #[cfg(feature = "text-cosmic")]
        {
            let text_shape_count = output
                .shapes
                .iter()
                .filter(|s| matches!(s.shape, Shape::Text(_)))
                .count();

            if text_shape_count > 0 {
                // Estimate: assume ~10 unique glyphs per text shape (conservative)
                let estimated_new_glyphs = text_shape_count * 10;
                let estimated_space_px = estimated_new_glyphs as u32
                    * self.avg_glyph_size_estimate_px
                    * self.avg_glyph_size_estimate_px;

                let (atlas_w, atlas_h) = self.atlas.dimensions();
                let total_atlas_space = atlas_w * atlas_h;
                let current_utilization = self.atlas.utilization();

                // If we'd exceed 70% utilization with new glyphs, resize proactively
                let estimated_utilization =
                    current_utilization + (estimated_space_px as f32 / total_atlas_space as f32);

                if estimated_utilization > 0.7 {
                    eprintln!(
                        "Proactive atlas resize: current={:.1}%, estimated={:.1}%",
                        current_utilization * 100.0,
                        estimated_utilization * 100.0
                    );
                    self.resize_atlas(device, queue);
                }
            }
        }

        // Group shapes by z-index into rendering layers
        // This ensures correct z-ordering where text respects z-index
        let layers = Self::group_into_layers(&output.shapes);

        // Separate shapes into SDF-renderable and tessellated.
        // SDF rendering is used for simple shapes (currently: all fills, simple strokes).
        // OPTIMIZATION: Pre-allocate based on previous frame to reduce allocations
        self.sdf_instances.clear();
        self.sdf_instances
            .reserve(self.last_frame_sdf_instance_count);
        self.sdf_draws.clear();

        self.wgpu_vertices.clear();
        self.wgpu_vertices.reserve(self.last_frame_vertex_count);

        self.frame_indices.clear();
        self.frame_indices.reserve(self.last_frame_index_count);

        self.frame_geometry_draws.clear();

        // Text buffers
        self.text_vertices.clear();
        self.text_vertices
            .reserve(self.last_frame_text_vertex_count);

        self.text_indices.clear();
        self.text_indices.reserve(self.last_frame_text_index_count);

        let mut text_draws: Vec<ClippedDraw> = Vec::with_capacity(self.last_frame_text_draw_count);

        // Track draw commands for each layer to enable interleaved rendering
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum DrawCommand {
            Sdf(usize),  // Index into sdf_draws
            Text(usize), // Index into text_draws
        }

        let mut layer_draw_commands: Vec<Vec<DrawCommand>> = Vec::with_capacity(layers.len());

        // Collect debug rectangles for text line bounds
        // (rect, color, stroke, clip_rect, transform)
        let mut debug_text_rects: Vec<(Rect, Color, Stroke, Rect, Transform2D)> = Vec::new();

        // Process shapes layer by layer to respect z-index ordering
        for layer in &layers {
            let mut current_layer_commands = Vec::new();

            for clipped in &layer.shapes {
                match &clipped.shape {
                    Shape::Rect(_rect) => {
                        // Decide whether to use SDF or mesh rendering based on render_mode
                        let use_sdf = match self.render_mode {
                            RenderMode::Sdf => true,
                            RenderMode::Mesh => false,
                            RenderMode::Auto => true, // Default to SDF for best quality
                        };

                        if use_sdf {
                            // Use SDF rendering (analytical anti-aliasing)
                            // Compute scissor rect for this shape
                            let sc_min_x = clipped.clip_rect.min[0].max(0.0).floor() as i32;
                            let sc_min_y = clipped.clip_rect.min[1].max(0.0).floor() as i32;
                            let sc_max_x = clipped.clip_rect.max[0].min(screen_width).ceil() as i32;
                            let sc_max_y =
                                clipped.clip_rect.max[1].min(screen_height).ceil() as i32;

                            let sc_w = (sc_max_x - sc_min_x).max(0) as u32;
                            let sc_h = (sc_max_y - sc_min_y).max(0) as u32;

                            // Skip if fully clipped
                            if sc_w > 0 && sc_h > 0 {
                                let scissor = (sc_min_x as u32, sc_min_y as u32, sc_w, sc_h);
                                let instance_index = self.sdf_instances.len() as u32;

                                self.sdf_instances.push(RectInstance::from(*clipped));

                                // Try to batch with previous draw if same scissor
                                // IMPORTANT: Only batch if the previous command was also SDF and from this layer
                                let can_batch = if let Some(DrawCommand::Sdf(last_idx)) =
                                    current_layer_commands.last()
                                {
                                    *last_idx == self.sdf_draws.len() - 1
                                } else {
                                    false
                                };

                                if can_batch {
                                    if let Some(last_draw) = self.sdf_draws.last_mut() {
                                        if last_draw.scissor == scissor
                                            && last_draw.instance_start + last_draw.instance_count
                                                == instance_index
                                        {
                                            // Extend existing batch
                                            last_draw.instance_count += 1;
                                        } else {
                                            // Start new batch (different scissor or non-consecutive)
                                            self.sdf_draws.push(SdfDraw {
                                                scissor,
                                                instance_start: instance_index,
                                                instance_count: 1,
                                            });
                                            current_layer_commands
                                                .push(DrawCommand::Sdf(self.sdf_draws.len() - 1));
                                        }
                                    }
                                } else {
                                    // First draw in this layer or switched from Text
                                    self.sdf_draws.push(SdfDraw {
                                        scissor,
                                        instance_start: instance_index,
                                        instance_count: 1,
                                    });
                                    current_layer_commands
                                        .push(DrawCommand::Sdf(self.sdf_draws.len() - 1));
                                }
                            }
                        } else {
                            // Use mesh tessellation - collect for batch processing
                            // (Tessellator processes all shapes at once)
                        }
                    }
                    Shape::Triangle(_triangle) => {
                        // Triangles are always rendered using mesh tessellation
                        // They will be processed by the tessellator below
                    }
                    Shape::Text(text_shape) => {
                        #[cfg(feature = "text-cosmic")]
                        {
                            // Use untransformed rect for shaping - transforms will be applied to vertices
                            let rect = text_shape.rect;
                            let text = text_shape.text.as_str();

                            if text.is_empty() {
                                continue;
                            }

                            // Compute the scissor rect for this shape, clamped to framebuffer bounds.
                            let sc_min_x = clipped.clip_rect.min[0].max(0.0).floor() as i32;
                            let sc_min_y = clipped.clip_rect.min[1].max(0.0).floor() as i32;
                            let sc_max_x = clipped.clip_rect.max[0].min(screen_width).ceil() as i32;
                            let sc_max_y =
                                clipped.clip_rect.max[1].min(screen_height).ceil() as i32;

                            let sc_w = (sc_max_x - sc_min_x).max(0) as u32;
                            let sc_h = (sc_max_y - sc_min_y).max(0) as u32;

                            if sc_w == 0 || sc_h == 0 {
                                continue;
                            }

                            let scissor_for_shape = (sc_min_x as u32, sc_min_y as u32, sc_w, sc_h);

                            // Start of this shape's indices in the final index buffer.
                            let index_start = self.text_indices.len() as u32;

                            // Shape + placement (backend-agnostic) with caching
                            // Resolve font size to f32 (should already be in physical pixels)
                            let width = rect.max[0] - rect.min[0];
                            let font_size_px = text_shape
                                .font_size
                                .try_resolve_with_scale(width, 1.0)
                                .unwrap_or(16.0);

                            // Create cache key from text + font size + rect dimensions + wrap + line height
                            let cache_key = (
                                text.to_string(),
                                font_size_px as u32,
                                (rect.max[0] - rect.min[0]) as u32,
                                (rect.max[1] - rect.min[1]) as u32,
                                text_shape.wrap,
                                (text_shape.line_height_multiplier * 100.0) as u32,
                            );

                            let shaped = if let Some(cached) = self.shape_cache.get(&cache_key) {
                                // Cache hit - reuse shaped text
                                cached.clone()
                            } else {
                                // Cache miss - shape the text
                                let (shaped_text, _placement) =
                                    self.text_engine.shape_text(gui_text::ShapeTextRequest {
                                        text,
                                        rect,
                                        font_px: font_size_px,
                                        h_align: text_shape.h_align,
                                        v_align: text_shape.v_align,
                                        family: None,
                                        wrap: text_shape.wrap,
                                        line_height_multiplier: text_shape.line_height_multiplier,
                                    });
                                self.shape_cache.insert(cache_key, shaped_text.clone());
                                shaped_text
                            };

                            // Always recalculate placement for this specific rect position
                            // (placement contains absolute screen positions, so it can't be cached)
                            // v_align applies to entire text block
                            let origin_y = match text_shape.v_align {
                                VerticalAlign::Top => rect.min[1],
                                VerticalAlign::Center => {
                                    rect.min[1]
                                        + ((rect.max[1] - rect.min[1]) - shaped.total_height) * 0.5
                                }
                                VerticalAlign::Bottom => rect.max[1] - shaped.total_height,
                            };

                            // Pre-calculate rotation trig functions outside the glyph loop
                            let rotation = clipped.transform.rotation;
                            let (cos_r, sin_r) = if rotation.abs() > 0.0001 {
                                (rotation.cos(), rotation.sin())
                            } else {
                                (1.0, 0.0) // Identity rotation
                            };
                            let has_rotation = rotation.abs() > 0.0001;

                            // Render all lines
                            let mut current_y = origin_y;
                            for line in &shaped.lines {
                                // h_align applies per-line
                                let line_x = match text_shape.h_align {
                                    HorizontalAlign::Left => rect.min[0],
                                    HorizontalAlign::Center => {
                                        rect.min[0]
                                            + ((rect.max[0] - rect.min[0]) - line.metrics.width_px)
                                                * 0.5
                                    }
                                    HorizontalAlign::Right => rect.max[0] - line.metrics.width_px,
                                };

                                for g in &line.glyphs {
                                    // Map glyph key to atlas key
                                    let atlas_key = text::atlas::GlyphKey::new(
                                        g.key.font_id.0,
                                        g.key.glyph_id,
                                        g.key.px_size,
                                        g.key.subpixel_x_64 as u16,
                                    );

                                    // OPTIMIZATION: Check metrics cache first (includes placement)
                                    let (glyph_bearing, glyph_size, placed) = if let Some(&(
                                        bearing,
                                        size,
                                        placement,
                                    )) =
                                        self.glyph_metrics_cache.get(&atlas_key)
                                    {
                                        // Cache hit - use cached metrics and placement (no atlas lookup!)
                                        (bearing, size, placement)
                                    } else {
                                        // Cache miss - need to rasterize and upload
                                        let Some(bitmap) = self.text_engine.rasterize_glyph(g.key)
                                        else {
                                            continue;
                                        };

                                        // Insert into atlas
                                        let placed = match self
                                            .atlas
                                            .insert(atlas_key.clone(), bitmap.size_px)
                                        {
                                            text::atlas::AtlasInsert::AlreadyPresent => {
                                                // Already in atlas, get placement
                                                self.atlas.get(&atlas_key)
                                            }
                                            text::atlas::AtlasInsert::Placed(p) => {
                                                // Newly placed - upload texture
                                                let rect_px =
                                                    text::atlas::GlyphAtlas::upload_rect_px(p);
                                                let pad = p.padding_px;
                                                queue.write_texture(
                                                    wgpu::TexelCopyTextureInfo {
                                                        texture: &self.atlas_texture,
                                                        mip_level: 0,
                                                        origin: wgpu::Origin3d {
                                                            x: rect_px.min.x + pad,
                                                            y: rect_px.min.y + pad,
                                                            z: 0,
                                                        },
                                                        aspect: wgpu::TextureAspect::All,
                                                    },
                                                    &bitmap.pixels,
                                                    wgpu::TexelCopyBufferLayout {
                                                        offset: 0,
                                                        bytes_per_row: Some(bitmap.size_px[0]),
                                                        rows_per_image: Some(bitmap.size_px[1]),
                                                    },
                                                    wgpu::Extent3d {
                                                        width: bitmap.size_px[0],
                                                        height: bitmap.size_px[1],
                                                        depth_or_array_layers: 1,
                                                    },
                                                );

                                                // Update size estimate for better future predictions (smooth average)
                                                let glyph_area =
                                                    bitmap.size_px[0] * bitmap.size_px[1];
                                                let glyph_size = (glyph_area as f32).sqrt() as u32;
                                                self.avg_glyph_size_estimate_px =
                                                    (self.avg_glyph_size_estimate_px * 7
                                                        + glyph_size)
                                                        / 8;

                                                Some(p)
                                            }
                                            text::atlas::AtlasInsert::Full => {
                                                eprintln!(
                                    "WARNING: Glyph atlas full during render! Will resize next frame. \
                                     (font_id={}, glyph_id={}, size={}px)",
                                    atlas_key.font_id, atlas_key.glyph_id, atlas_key.font_px
                                );

                                                // Mark for resize before next frame
                                                self.atlas_needs_resize = true;

                                                // Update size estimate for better future predictions
                                                let glyph_area =
                                                    bitmap.size_px[0] * bitmap.size_px[1];
                                                let glyph_size = (glyph_area as f32).sqrt() as u32;
                                                self.avg_glyph_size_estimate_px =
                                                    (self.avg_glyph_size_estimate_px + glyph_size)
                                                        / 2;

                                                None
                                            }
                                        };

                                        let Some(p) = placed else {
                                            continue;
                                        };

                                        // Cache metrics AND placement for future frames
                                        let metrics = (bitmap.bearing_px, bitmap.size_px, p);
                                        self.glyph_metrics_cache.insert(atlas_key.clone(), metrics);
                                        (bitmap.bearing_px, bitmap.size_px, p)
                                    };

                                    let x0 = line_x + g.x_px + glyph_bearing[0] as f32;
                                    let y0 = current_y + g.y_px + glyph_bearing[1] as f32;
                                    let x1 = x0 + glyph_size[0] as f32;
                                    let y1 = y0 + glyph_size[1] as f32;

                                    // Apply full transform (translation + rotation) to the glyph quad vertices
                                    let translation = clipped.transform.translation;
                                    let transform_origin = if let Some(abs_origin) =
                                        clipped.transform.absolute_origin
                                    {
                                        abs_origin
                                    } else {
                                        // Fallback: resolve origin relative to the node rect
                                        let node_width =
                                            clipped.node_rect.max[0] - clipped.node_rect.min[0];
                                        let node_height =
                                            clipped.node_rect.max[1] - clipped.node_rect.min[1];
                                        let (origin_x, origin_y) = clipped
                                            .transform
                                            .origin
                                            .resolve(node_width, node_height);
                                        [
                                            clipped.node_rect.min[0] + origin_x,
                                            clipped.node_rect.min[1] + origin_y,
                                        ]
                                    };

                                    // Helper to apply translation first, then rotation around the transform origin
                                    // Uses pre-calculated cos_r and sin_r from outside the loop
                                    let apply_transform = |pos: [f32; 2]| -> [f32; 2] {
                                        // 1. Apply translation first
                                        let mut x = pos[0] + translation.x;
                                        let mut y = pos[1] + translation.y;

                                        // 2. Apply rotation if present (use pre-calculated trig values)
                                        if has_rotation {
                                            // Translate to origin
                                            x -= transform_origin[0];
                                            y -= transform_origin[1];

                                            // Rotate (clockwise positive) - uses pre-calculated cos_r and sin_r
                                            let rx = x * cos_r + y * sin_r;
                                            let ry = -x * sin_r + y * cos_r;

                                            x = rx;
                                            y = ry;

                                            // Translate back from origin
                                            x += transform_origin[0];
                                            y += transform_origin[1];
                                        }

                                        [x, y]
                                    };

                                    let p0 = apply_transform([x0, y0]);
                                    let p1 = apply_transform([x1, y0]);
                                    let p2 = apply_transform([x1, y1]);
                                    let p3 = apply_transform([x0, y1]);

                                    // Apply opacity from ClippedShape to text color
                                    let color = [
                                        text_shape.color.r,
                                        text_shape.color.g,
                                        text_shape.color.b,
                                        text_shape.color.a * clipped.opacity,
                                    ];
                                    let uv = placed.uv;

                                    let base = self.text_vertices.len() as u32;
                                    self.text_vertices.push(text::vertex::TextVertex::new(
                                        p0,
                                        [uv.min[0], uv.min[1]],
                                        color,
                                    ));
                                    self.text_vertices.push(text::vertex::TextVertex::new(
                                        p1,
                                        [uv.max[0], uv.min[1]],
                                        color,
                                    ));
                                    self.text_vertices.push(text::vertex::TextVertex::new(
                                        p2,
                                        [uv.max[0], uv.max[1]],
                                        color,
                                    ));
                                    self.text_vertices.push(text::vertex::TextVertex::new(
                                        p3,
                                        [uv.min[0], uv.max[1]],
                                        color,
                                    ));

                                    self.text_indices.extend_from_slice(&[
                                        base,
                                        base + 1,
                                        base + 2,
                                        base,
                                        base + 2,
                                        base + 3,
                                    ]);
                                }

                                // Debug: Show text line bounds (cyan outline)
                                if let Some(debug_opts) = output.debug_options.as_ref() {
                                    if debug_opts.show_text_bounds {
                                        let line_rect = Rect::new(
                                            [line_x, current_y],
                                            [
                                                line_x + line.metrics.width_px,
                                                current_y + line.metrics.height_px,
                                            ],
                                        );
                                        debug_text_rects.push((
                                            line_rect,
                                            Color::rgba(0.0, 1.0, 1.0, 1.0), // Cyan
                                            Stroke::new(
                                                Size::ppx(1.0),
                                                Color::rgba(0.0, 1.0, 1.0, 1.0),
                                            ),
                                            clipped.clip_rect,
                                            clipped.transform,
                                        ));
                                    }
                                }

                                // Move to next line
                                current_y += line.metrics.height_px;
                            }
                            let index_end = self.text_indices.len() as u32;
                            if index_end > index_start {
                                // Try to batch with previous draw if same scissor
                                // IMPORTANT: Only batch if the previous command was also Text and from this layer
                                let can_batch = if let Some(DrawCommand::Text(last_idx)) =
                                    current_layer_commands.last()
                                {
                                    *last_idx == text_draws.len() - 1
                                } else {
                                    false
                                };

                                if can_batch {
                                    if let Some(last_draw) = text_draws.last_mut() {
                                        if last_draw.scissor == scissor_for_shape
                                            && last_draw.index_end == index_start
                                        {
                                            // Extend existing batch
                                            last_draw.index_end = index_end;
                                        } else {
                                            // Start new batch
                                            text_draws.push(ClippedDraw {
                                                scissor: scissor_for_shape,
                                                index_start,
                                                index_end,
                                            });
                                            current_layer_commands
                                                .push(DrawCommand::Text(text_draws.len() - 1));
                                        }
                                    }
                                } else {
                                    // First draw in this layer or switched from SDF
                                    text_draws.push(ClippedDraw {
                                        scissor: scissor_for_shape,
                                        index_start,
                                        index_end,
                                    });
                                    current_layer_commands
                                        .push(DrawCommand::Text(text_draws.len() - 1));
                                }
                            }
                        }
                    }
                }
            } // End for clipped in layer.shapes

            layer_draw_commands.push(current_layer_commands);
        } // End for layer in layers

        // Add debug text line bounds as SDF rectangles (rendered on top)
        if !debug_text_rects.is_empty() {
            let mut debug_layer_commands = Vec::new();

            for (rect, _fill_color, stroke, clip_rect, transform) in debug_text_rects {
                // Create a StyledRect for the debug rectangle
                let styled_rect = StyledRect {
                    rect,
                    fill: Color::rgba(0.0, 0.0, 0.0, 0.0), // Transparent fill
                    stroke: Some(stroke),
                    corner_shape: CornerShape::None,
                };

                // Create a ClippedShape for this debug rectangle with the text's transform
                let clipped_debug = ClippedShape {
                    shape: Shape::Rect(styled_rect),
                    node_rect: rect,
                    clip_rect,
                    opacity: 1.0,
                    transform,                 // Use the transform from the text shape
                    z_index: ZIndex(i32::MAX), // Render on top
                    tree_index: 0,
                };

                // Compute scissor rect
                let sc_min_x = clip_rect.min[0].max(0.0).floor() as i32;
                let sc_min_y = clip_rect.min[1].max(0.0).floor() as i32;
                let sc_max_x = clip_rect.max[0].min(screen_width).ceil() as i32;
                let sc_max_y = clip_rect.max[1].min(screen_height).ceil() as i32;

                let sc_w = (sc_max_x - sc_min_x).max(0) as u32;
                let sc_h = (sc_max_y - sc_min_y).max(0) as u32;

                if sc_w > 0 && sc_h > 0 {
                    let scissor = (sc_min_x as u32, sc_min_y as u32, sc_w, sc_h);
                    let instance_index = self.sdf_instances.len() as u32;

                    self.sdf_instances.push(RectInstance::from(&clipped_debug));

                    // Try to batch with previous debug draw
                    let can_batch =
                        if let Some(DrawCommand::Sdf(last_idx)) = debug_layer_commands.last() {
                            *last_idx == self.sdf_draws.len() - 1
                        } else {
                            false
                        };

                    if can_batch {
                        if let Some(last_draw) = self.sdf_draws.last_mut() {
                            if last_draw.scissor == scissor
                                && last_draw.instance_start + last_draw.instance_count
                                    == instance_index
                            {
                                last_draw.instance_count += 1;
                            } else {
                                self.sdf_draws.push(SdfDraw {
                                    scissor,
                                    instance_start: instance_index,
                                    instance_count: 1,
                                });
                                debug_layer_commands
                                    .push(DrawCommand::Sdf(self.sdf_draws.len() - 1));
                            }
                        }
                    } else {
                        self.sdf_draws.push(SdfDraw {
                            scissor,
                            instance_start: instance_index,
                            instance_count: 1,
                        });
                        debug_layer_commands.push(DrawCommand::Sdf(self.sdf_draws.len() - 1));
                    }
                }
            }

            if !debug_layer_commands.is_empty() {
                layer_draw_commands.push(debug_layer_commands);
            }
        }

        // Store layer count for later use in render pass
        let layer_count = layer_draw_commands.len();

        // Process mesh shapes if using Mesh render mode
        if self.render_mode == RenderMode::Mesh {
            // Tessellate all shapes using mesh rendering
            let mesh = self.tessellator.tessellate(&output.shapes);

            if !mesh.vertices.is_empty() {
                // Convert mesh vertices to WgpuVertex format
                for vertex in &mesh.vertices {
                    self.wgpu_vertices.push(WgpuVertex {
                        pos: vertex.pos,
                        color: [
                            (vertex.color[0] * 255.0).round().clamp(0.0, 255.0) as u8,
                            (vertex.color[1] * 255.0).round().clamp(0.0, 255.0) as u8,
                            (vertex.color[2] * 255.0).round().clamp(0.0, 255.0) as u8,
                            (vertex.color[3] * 255.0).round().clamp(0.0, 255.0) as u8,
                        ],
                    });
                }

                // Copy indices
                self.frame_indices.extend_from_slice(&mesh.indices);

                // Create draw calls with scissor rects
                for clipped in &output.shapes {
                    let sc_min_x = clipped.clip_rect.min[0].max(0.0).floor() as i32;
                    let sc_min_y = clipped.clip_rect.min[1].max(0.0).floor() as i32;
                    let sc_max_x = clipped.clip_rect.max[0].min(screen_width).ceil() as i32;
                    let sc_max_y = clipped.clip_rect.max[1].min(screen_height).ceil() as i32;

                    let sc_w = (sc_max_x - sc_min_x).max(0) as u32;
                    let sc_h = (sc_max_y - sc_min_y).max(0) as u32;

                    if sc_w > 0 && sc_h > 0 {
                        // Use the entire mesh for now (TODO: track per-shape indices)
                        self.frame_geometry_draws.push(ClippedDraw {
                            scissor: (sc_min_x as u32, sc_min_y as u32, sc_w, sc_h),
                            index_start: 0,
                            index_end: self.frame_indices.len() as u32,
                        });
                    }
                }
            }
        }

        // Resize vertex buffer if needed
        if self.wgpu_vertices.len() > self.vertex_capacity {
            self.vertex_capacity = (self.wgpu_vertices.len() * 2).next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Astra UI Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<WgpuVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Resize index buffer if needed
        if self.frame_indices.len() > self.index_capacity {
            self.index_capacity = (self.frame_indices.len() * 2).next_power_of_two();
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Astra UI Index Buffer"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Upload geometry
        if !self.frame_indices.is_empty() {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.wgpu_vertices),
            );
            queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&self.frame_indices),
            );
        }

        // Update uniforms (used by both passes)
        let uniforms = [screen_width, screen_height];
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&uniforms));

        // Upload SDF instances
        if !self.sdf_instances.is_empty() {
            // Resize instance buffer if needed
            if self.sdf_instances.len() > self.sdf_instance_capacity {
                self.sdf_instance_capacity = (self.sdf_instances.len() * 2).next_power_of_two();
                self.sdf_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Astra UI SDF Instance Buffer"),
                    size: (self.sdf_instance_capacity * std::mem::size_of::<RectInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            queue.write_buffer(
                &self.sdf_instance_buffer,
                0,
                bytemuck::cast_slice(&self.sdf_instances),
            );
        }

        // Upload text buffers before render pass
        if !text_draws.is_empty() {
            if self.text_vertices.len() > self.text_vertex_capacity {
                self.text_vertex_capacity = (self.text_vertices.len() * 2).next_power_of_two();
                self.text_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Astra UI Text Vertex Buffer"),
                    size: (self.text_vertex_capacity
                        * std::mem::size_of::<text::vertex::TextVertex>())
                        as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            if self.text_indices.len() > self.text_index_capacity {
                self.text_index_capacity = (self.text_indices.len() * 2).next_power_of_two();
                self.text_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Astra UI Text Index Buffer"),
                    size: (self.text_index_capacity * std::mem::size_of::<u32>()) as u64,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            queue.write_buffer(
                &self.text_vertex_buffer,
                0,
                bytemuck::cast_slice(&self.text_vertices),
            );
            queue.write_buffer(
                &self.text_index_buffer,
                0,
                bytemuck::cast_slice(&self.text_indices),
            );
        }

        // Update frame tracking for next frame's pre-allocation
        self.last_frame_text_vertex_count = self.text_vertices.len();
        self.last_frame_text_index_count = self.text_indices.len();
        self.last_frame_text_draw_count = text_draws.len();

        // Render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Astra UI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Preserve existing content
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Layer-based rendering: Render each z-index layer completely before moving to the next
        // This ensures text respects z-index and doesn't always render on top

        // Track current pipeline state to avoid redundant switches
        #[derive(PartialEq)]
        enum PipelineState {
            None,
            Sdf,
            Text,
        }
        let mut current_pipeline = PipelineState::None;

        for layer_idx in 0..layer_count {
            let commands = &layer_draw_commands[layer_idx];

            for command in commands {
                match command {
                    DrawCommand::Sdf(idx) => {
                        let draw = &self.sdf_draws[*idx];

                        if current_pipeline != PipelineState::Sdf {
                            render_pass.set_pipeline(&self.sdf_pipeline);
                            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                            render_pass.set_vertex_buffer(0, self.sdf_quad_vertex_buffer.slice(..));
                            render_pass.set_vertex_buffer(1, self.sdf_instance_buffer.slice(..));
                            render_pass.set_index_buffer(
                                self.sdf_quad_index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            current_pipeline = PipelineState::Sdf;
                        }

                        let (x, y, w, h) = draw.scissor;
                        render_pass.set_scissor_rect(x, y, w, h);
                        render_pass.draw_indexed(
                            0..6,
                            0,
                            draw.instance_start..(draw.instance_start + draw.instance_count),
                        );
                    }
                    DrawCommand::Text(idx) => {
                        #[cfg(feature = "text-cosmic")]
                        {
                            let draw = &text_draws[*idx];

                            if current_pipeline != PipelineState::Text {
                                render_pass.set_pipeline(&self.text_pipeline);
                                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                                render_pass.set_bind_group(1, &self.atlas_bind_group, &[]);
                                render_pass.set_vertex_buffer(0, self.text_vertex_buffer.slice(..));
                                render_pass.set_index_buffer(
                                    self.text_index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                current_pipeline = PipelineState::Text;
                            }

                            let (x, y, w, h) = draw.scissor;
                            render_pass.set_scissor_rect(x, y, w, h);
                            render_pass.draw_indexed(draw.index_start..draw.index_end, 0, 0..1);
                        }
                    }
                }
            }
        } // End layer loop

        // Draw geometry with batched scissor clipping
        // OPTIMIZATION: Batch consecutive draws with the same scissor rect to reduce draw calls
        if !self.frame_geometry_draws.is_empty() {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // Batch consecutive draws with the same scissor rect
            let mut current_scissor = self.frame_geometry_draws[0].scissor;
            let mut batch_start = self.frame_geometry_draws[0].index_start;
            let mut batch_end = self.frame_geometry_draws[0].index_end;

            for draw in &self.frame_geometry_draws[1..] {
                if draw.scissor == current_scissor && draw.index_start == batch_end {
                    // Extend current batch (consecutive indices, same scissor)
                    batch_end = draw.index_end;
                } else {
                    // Flush current batch
                    let (x, y, w, h) = current_scissor;
                    render_pass.set_scissor_rect(x, y, w, h);
                    render_pass.draw_indexed(batch_start..batch_end, 0, 0..1);

                    // Start new batch
                    current_scissor = draw.scissor;
                    batch_start = draw.index_start;
                    batch_end = draw.index_end;
                }
            }

            // Flush final batch
            let (x, y, w, h) = current_scissor;
            render_pass.set_scissor_rect(x, y, w, h);
            render_pass.draw_indexed(batch_start..batch_end, 0, 0..1);

            // Reset scissor to full screen
            render_pass.set_scissor_rect(0, 0, screen_width as u32, screen_height as u32);
        }

        // Update frame tracking for geometry buffers
        self.last_frame_vertex_count = self.wgpu_vertices.len();
        self.last_frame_index_count = self.frame_indices.len();
        self.last_frame_sdf_instance_count = self.sdf_instances.len();
    }
}
