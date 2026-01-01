//! Demonstrates the built-in Catppuccin color schemes
use astra_gui::{
    catppuccin::{frappe, latte, macchiato, mocha},
    Color, Content, CornerShape, DebugOptions, FullOutput, HorizontalAlign, Layout, Node, Shape,
    Size, Spacing, StyledRect, TextContent, VerticalAlign,
};
use astra_gui_wgpu::{RenderMode, Renderer};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const DEBUG_HELP_TEXT_ONELINE: &str = "M:Margins | P:Padding | B:Borders | C:Content | R:ClipRects | G:Gaps | D:All | S:RenderMode | ESC:Exit";

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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Clear the screen
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
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

        // Create UI node tree and render (with debug visualization)
        let ui_output = create_demo_ui(
            self.config.width as f32,
            self.config.height as f32,
            debug_options,
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

fn theme_card(
    name: &str,
    crust: Color,
    base: Color,
    text: Color,
    mut colors: Vec<(&'static str, Color)>,
) -> Node {
    while colors.len() % 5 != 0 {
        colors.push(("", Color::transparent()));
    }

    let mut rows = vec![];
    for chunk in colors.chunks(5) {
        let row = Node::new()
            .with_height(Size::fraction(1.0 / 5.0))
            .with_layout_direction(Layout::Horizontal)
            .with_gap(Size::lpx(10.0))
            .with_children(
                chunk
                    .iter()
                    .map(|&(n, c)| color_swatch(n, c, base, text))
                    .collect(),
            );
        rows.push(row);
    }

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(20.0)))
        .with_shape(Shape::Rect(StyledRect::new(Default::default(), crust)))
        .with_layout_direction(Layout::Vertical)
        .with_gap(Size::lpx(15.0))
        .with_children(vec![
            // Title
            Node::new()
                .with_height(Size::lpx(40.0))
                .with_content(Content::Text(
                    TextContent::new(name)
                        .with_font_size(Size::lpx(32.0))
                        .with_color(text)
                        .with_h_align(HorizontalAlign::Center)
                        .with_v_align(VerticalAlign::Center),
                )),
            // Content box (Panel)
            Node::new()
                .with_height(Size::Fill)
                .with_padding(Spacing::all(Size::lpx(17.5)))
                .with_shape(Shape::Rect(
                    StyledRect::new(Default::default(), base)
                        .with_corner_shape(CornerShape::Cut(Size::lpx(40.0))),
                ))
                .with_layout_direction(Layout::Vertical)
                .with_gap(Size::lpx(10.0))
                .with_children(rows),
        ])
}

fn color_swatch(name: &str, color: Color, base_color: Color, text_color: Color) -> Node {
    if name.is_empty() {
        return Node::new().with_width(Size::Fill).with_height(Size::Fill);
    }

    let contrast_base = color.contrast_ratio(&base_color);
    let contrast_text = color.contrast_ratio(&text_color);

    let final_text_color = if contrast_base > contrast_text {
        base_color
    } else {
        text_color
    };

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_shape(Shape::Rect(
            StyledRect::new(Default::default(), color)
                .with_corner_shape(CornerShape::Cut(Size::lpx(30.0))),
        ))
        .with_content(Content::Text(
            TextContent::new(name)
                .with_font_size(Size::lpx(24.0))
                .with_color(final_text_color)
                .with_h_align(HorizontalAlign::Center)
                .with_v_align(VerticalAlign::Center),
        ))
}

fn create_demo_ui(width: f32, height: f32, debug_options: &DebugOptions) -> FullOutput {
    macro_rules! colors {
        ($m:ident) => {
            vec![
                ("Rosewater", $m::ROSEWATER),
                ("Flamingo", $m::FLAMINGO),
                ("Pink", $m::PINK),
                ("Mauve", $m::MAUVE),
                ("Red", $m::RED),
                ("Maroon", $m::MAROON),
                ("Peach", $m::PEACH),
                ("Yellow", $m::YELLOW),
                ("Green", $m::GREEN),
                ("Teal", $m::TEAL),
                ("Sky", $m::SKY),
                ("Sapphire", $m::SAPPHIRE),
                ("Blue", $m::BLUE),
                ("Lavender", $m::LAVENDER),
                ("Text", $m::TEXT),
                ("Subtext1", $m::SUBTEXT1),
                ("Subtext0", $m::SUBTEXT0),
                ("Overlay2", $m::OVERLAY2),
                ("Overlay1", $m::OVERLAY1),
                ("Overlay0", $m::OVERLAY0),
                ("Surface2", $m::SURFACE2),
                ("Surface1", $m::SURFACE1),
                ("Surface0", $m::SURFACE0),
                // ("Base", $m::BASE),
                ("Mantle", $m::MANTLE),
                ("Crust", $m::CRUST),
            ]
        };
    }

    // Root container - 2x2 grid
    let root = Node::new()
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![
            // Top Row
            Node::new()
                .with_height(Size::fraction(0.5))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![
                    theme_card(
                        "Latte",
                        latte::CRUST,
                        latte::BASE,
                        latte::TEXT,
                        colors!(latte),
                    ),
                    theme_card(
                        "Frappe",
                        frappe::CRUST,
                        frappe::BASE,
                        frappe::TEXT,
                        colors!(frappe),
                    ),
                ]),
            // Bottom Row
            Node::new()
                .with_height(Size::fraction(0.5))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![
                    theme_card(
                        "Macchiato",
                        macchiato::CRUST,
                        macchiato::BASE,
                        macchiato::TEXT,
                        colors!(macchiato),
                    ),
                    theme_card(
                        "Mocha",
                        mocha::CRUST,
                        mocha::BASE,
                        mocha::TEXT,
                        colors!(mocha),
                    ),
                ]),
        ]);

    // Create help bar at the bottom
    let help_text = Node::new()
        .with_height(Size::lpx(30.0))
        .with_padding(Spacing::horizontal(Size::lpx(10.0)))
        .with_shape(Shape::Rect(StyledRect::new(
            Default::default(),
            mocha::SURFACE0,
        )))
        .with_content(Content::Text(
            TextContent::new(DEBUG_HELP_TEXT_ONELINE)
                .with_font_size(Size::lpx(16.0))
                .with_color(mocha::TEXT)
                .with_h_align(HorizontalAlign::Left)
                .with_v_align(VerticalAlign::Center),
        ));

    // Overlay help text on top of the grid
    // Actually, let's put it below the grid, but the grid takes full height.
    // We can make the grid take (Fill - 30px) and help text 30px.

    let main_layout = Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![
            root.with_height(Size::Fill), // Grid takes remaining space
            help_text,
        ])
        .with_zoom(1.5);

    FullOutput::from_node_with_debug(
        main_layout,
        (width, height),
        if debug_options.is_enabled() {
            Some(*debug_options)
        } else {
            None
        },
    )
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Astra GUI - Catppuccin Themes")
                        .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 1200.0)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());
        self.gpu_state = Some(pollster::block_on(GpuState::new(window)));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        let renderer = self.gpu_state.as_mut().map(|s| &mut s.renderer);
        if handle_debug_keybinds(&event, &mut self.debug_options, renderer) {
            window.request_redraw();
        }

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
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
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = self.gpu_state.as_mut() {
                    state.resize(physical_size);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = self.gpu_state.as_mut() {
                    match state.render(&self.debug_options) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(window.inner_size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        window: None,
        gpu_state: None,
        debug_options: DebugOptions::default(),
    };

    event_loop.run_app(&mut app).unwrap();
}
