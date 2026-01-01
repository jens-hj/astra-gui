//! Demonstrates multi-line text rendering with different wrapping modes.
//!
//! This example shows:
//! - Explicit newlines in text
//! - Word wrapping, glyph wrapping, and mixed wrapping modes
//! - Different line heights
//! - Text alignment with multi-line content
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/O/D/S)
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, FullOutput, HorizontalAlign,
    Layout, Node, Rect, Shape, Size, Spacing, Style, StyledRect, TextContent, VerticalAlign, Wrap,
};
use astra_gui_wgpu::{RenderMode, Renderer};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const DEBUG_HELP_TEXT: &str = "Debug controls:
  M - Toggle margins (red overlay)
  P - Toggle padding (blue overlay)
  B - Toggle borders (green outline)
  C - Toggle content area (yellow outline)
  R - Toggle clip rects (red outline)
  G - Toggle gaps (purple overlay)
  O - Toggle transform origins (orange crosshair)
  T - Toggle text line bounds (cyan outline)
  D - Toggle all debug visualizations
  S - Toggle render mode (SDF/Mesh)
  ESC - Exit";

fn handle_debug_keybinds(
    event: &WindowEvent,
    debug_options: &mut DebugOptions,
    renderer: Option<&mut Renderer>,
) -> bool {
    let WindowEvent::KeyboardInput {
        event:
            KeyEvent {
                physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                state: ElementState::Pressed,
                ..
            },
        ..
    } = event
    else {
        return false;
    };

    match *key_code {
        winit::keyboard::KeyCode::KeyM => {
            debug_options.show_margins = !debug_options.show_margins;
            println!("Margins: {}", debug_options.show_margins);
            true
        }
        winit::keyboard::KeyCode::KeyP => {
            debug_options.show_padding = !debug_options.show_padding;
            println!("Padding: {}", debug_options.show_padding);
            true
        }
        winit::keyboard::KeyCode::KeyB => {
            debug_options.show_borders = !debug_options.show_borders;
            println!("Borders: {}", debug_options.show_borders);
            true
        }
        winit::keyboard::KeyCode::KeyC => {
            debug_options.show_content_area = !debug_options.show_content_area;
            println!("Content area: {}", debug_options.show_content_area);
            true
        }
        winit::keyboard::KeyCode::KeyR => {
            debug_options.show_clip_rects = !debug_options.show_clip_rects;
            println!("Clip rects: {}", debug_options.show_clip_rects);
            true
        }
        winit::keyboard::KeyCode::KeyG => {
            debug_options.show_gaps = !debug_options.show_gaps;
            println!("Gaps: {}", debug_options.show_gaps);
            true
        }
        winit::keyboard::KeyCode::KeyO => {
            debug_options.show_transform_origins = !debug_options.show_transform_origins;
            println!(
                "Transform origins: {}",
                debug_options.show_transform_origins
            );
            true
        }
        winit::keyboard::KeyCode::KeyT => {
            debug_options.show_text_bounds = !debug_options.show_text_bounds;
            println!("Text bounds: {}", debug_options.show_text_bounds);
            true
        }
        winit::keyboard::KeyCode::KeyD => {
            if debug_options.is_enabled() {
                *debug_options = DebugOptions::none();
                println!("Debug: OFF");
            } else {
                *debug_options = DebugOptions::all();
                println!("Debug: ALL ON");
            }
            true
        }
        winit::keyboard::KeyCode::KeyS => {
            if let Some(renderer) = renderer {
                let new_mode = match renderer.render_mode() {
                    RenderMode::Sdf | RenderMode::Auto => RenderMode::Mesh,
                    RenderMode::Mesh => RenderMode::Sdf,
                };
                renderer.set_render_mode(new_mode);
                println!("Render mode: {:?}", new_mode);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

struct App {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    debug_options: DebugOptions,
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
}

impl GpuState {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, surface_format);

        Self {
            surface,
            device,
            queue,
            config,
            renderer,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self, debug_options: &DebugOptions) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Clear pass
        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Clear Encoder"),
                });

            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: mocha::BASE.r as f64,
                                g: mocha::BASE.g as f64,
                                b: mocha::BASE.b as f64,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        // UI pass
        let ui_output = build_ui(
            self.config.width as f32,
            self.config.height as f32,
            debug_options,
            &mut self.renderer,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Astra GUI Encoder"),
            });

        self.renderer.render(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            self.config.width as f32,
            self.config.height as f32,
            &ui_output,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Multi-Line Text Example - Astra GUI")
            .with_inner_size(winit::dpi::LogicalSize::new(1200, 900));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let gpu_state = pollster::block_on(GpuState::new(window.clone()));

        self.window = Some(window);
        self.gpu_state = Some(gpu_state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // Handle debug keybinds
        if handle_debug_keybinds(
            &event,
            &mut self.debug_options,
            self.gpu_state.as_mut().map(|s| &mut s.renderer),
        ) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    match gpu_state.render(&self.debug_options) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            let size = winit::dpi::PhysicalSize::new(
                                gpu_state.config.width,
                                gpu_state.config.height,
                            );
                            gpu_state.resize(size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn build_ui(
    width: f32,
    height: f32,
    debug_options: &DebugOptions,
    renderer: &mut Renderer,
) -> FullOutput {
    let ui = Node::new().with_width(Size::Fill).with_height(Size::Fill).with_children(vec![
        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_padding(Spacing::all(Size::lpx(30.0)))
            .with_gap(Size::lpx(20.0))
            .with_style(Style {
                fill_color: Some(mocha::BASE),
                ..Default::default()
            })
            .with_children(vec![
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::lpx(40.0))
                    .with_content(Content::Text(
                        TextContent::new("Multi-Line Text & Wrapping Examples")
                            .with_font_size(Size::lpx(36.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Center),
                    )),
                // Examples grid
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(20.0))
                    .with_children(vec![
                        // Left column
                        column(vec![
                            example_box(
                                "Explicit Newlines",
                                "Line 1\nLine 2\nLine 3\nLine 4",
                                Wrap::None,
                                Size::FitContent,
                                mocha::BLUE,
                                1.2,
                            ),
                            example_box(
                                "Word Wrap (default)",
                                "This is a longer text that will automatically wrap at word boundaries when the container width is constrained. It's the default behavior.",
                                Wrap::Word,
                                Size::lpx(250.0),
                                mocha::GREEN,
                                1.2,
                            ),
                            example_box(
                                "Line Height 1.5x",
                                "This text has\nincreased line\nheight spacing\nfor better readability",
                                Wrap::None,
                                Size::FitContent,
                                mocha::YELLOW,
                                1.5,
                            ),
                        ]),
                        // Middle column
                        column(vec![
                            example_box(
                                "No Wrap (Overflow)",
                                "This is a very long text that will overflow the container instead of wrapping because wrapping is disabled.",
                                Wrap::None,
                                Size::lpx(250.0),
                                mocha::RED,
                                1.2,
                            ),
                            example_box(
                                "Glyph Wrap",
                                "Verylongwordthatwillwrapatanycharacterboundaryinsteadofwordswhenspaceisunavailable",
                                Wrap::Glyph,
                                Size::lpx(250.0),
                                mocha::MAUVE,
                                1.2,
                            ),
                            example_box(
                                "WordOrGlyph Wrap",
                                "Normal words wrap normally, but verylongwordswithoutspacesbreakanywheretofit",
                                Wrap::WordOrGlyph,
                                Size::lpx(250.0),
                                mocha::PEACH,
                                1.2,
                            ),
                        ]),
                        // Right column
                        column(vec![
                            alignment_example("Left Aligned\nMultiple Lines\nH: Left", HorizontalAlign::Left),
                            alignment_example("Center Aligned\nMultiple Lines\nH: Center", HorizontalAlign::Center),
                            alignment_example("Right Aligned\nMultiple Lines\nH: Right", HorizontalAlign::Right),
                        ]),
                    ]),
            ]),
        // Help text at bottom
        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::lpx(30.0))
            .with_padding(Spacing::horizontal(Size::lpx(10.0)))
            .with_style(Style {
                fill_color: Some(mocha::SURFACE0),
                ..Default::default()
            })
            .with_content(Content::Text(
                TextContent::new(
                    "M:Margins | P:Padding | B:Borders | C:Content | R:ClipRects | G:Gaps | O:Origins | T:Text | D:All | S:RenderMode | ESC:Exit",
                )
                .with_font_size(Size::lpx(16.0))
                .with_color(mocha::TEXT)
                .with_h_align(HorizontalAlign::Left)
                .with_v_align(VerticalAlign::Center),
            )),
    ]);

    FullOutput::from_node_with_debug_and_measurer(
        ui,
        (width, height),
        if debug_options.is_enabled() {
            Some(*debug_options)
        } else {
            None
        },
        Some(renderer.text_engine_mut()),
    )
}

fn column(children: Vec<Node>) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_gap(Size::lpx(15.0))
        .with_children(children)
}

fn example_box(
    title: &str,
    text: &str,
    wrap: Wrap,
    width: Size,
    color: Color,
    line_height: f32,
) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::FitContent)
        .with_layout_direction(Layout::Vertical)
        .with_gap(Size::lpx(8.0))
        .with_padding(Spacing::all(Size::lpx(15.0)))
        .with_style(Style {
            fill_color: Some(mocha::SURFACE0),
            corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
            ..Default::default()
        })
        .with_children(vec![
            // Title
            Node::new()
                .with_width(Size::Fill)
                .with_content(Content::Text(
                    TextContent::new(title)
                        .with_font_size(Size::lpx(24.0))
                        .with_color(color),
                )),
            // Example text
            Node::new()
                .with_width(width)
                .with_padding(Spacing::all(Size::lpx(10.0)))
                .with_style(Style {
                    fill_color: Some(mocha::MANTLE),
                    corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                    ..Default::default()
                })
                .with_content(Content::Text(
                    TextContent::new(text)
                        .with_font_size(Size::lpx(16.0))
                        .with_color(color)
                        .with_wrap(wrap)
                        .with_line_height(line_height),
                )),
        ])
}

fn alignment_example(text: &str, h_align: HorizontalAlign) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(15.0)))
        .with_shape(Shape::Rect(
            StyledRect::new(Rect::default(), mocha::SURFACE0)
                .with_corner_shape(CornerShape::Round(Size::lpx(8.0))),
        ))
        .with_content(Content::Text(
            TextContent::new(text)
                .with_font_size(Size::lpx(18.0))
                .with_color(mocha::TEAL)
                .with_h_align(h_align)
                .with_v_align(VerticalAlign::Center),
        ))
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        gpu_state: None,
        debug_options: DebugOptions::none(),
    };

    event_loop.run_app(&mut app).unwrap();
}
