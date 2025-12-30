//! Alignment example
//!
//! Demonstrates h_align and v_align working together for different layout directions.
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/D/S)
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, FullOutput, HorizontalAlign,
    Layout, Node, Rect, Shape, Size, Spacing, Stroke, Style, TextContent, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::{EventDispatcher, InputState, InteractiveStateManager, RenderMode, Renderer};
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

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
}

struct App {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    text_engine: TextEngine,
    input_state: InputState,
    event_dispatcher: EventDispatcher,
    interactive_state_manager: InteractiveStateManager,
    debug_options: DebugOptions,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gpu_state: None,
            text_engine: TextEngine::new_default(),
            input_state: InputState::new(),
            event_dispatcher: EventDispatcher::new(),
            interactive_state_manager: InteractiveStateManager::new(),
            debug_options: DebugOptions::none(),
        }
    }

    fn render(&mut self) {
        self.interactive_state_manager.begin_frame();

        let mut ui = self.build_ui();

        let size = match &self.window {
            Some(window) => window.inner_size(),
            None => return,
        };
        let window_rect = Rect::from_min_size([0.0, 0.0], [size.width as f32, size.height as f32]);
        ui.compute_layout_with_measurer(window_rect, &mut self.text_engine);

        let (_events, interaction_states) =
            self.event_dispatcher.dispatch(&self.input_state, &mut ui);

        self.interactive_state_manager
            .apply_styles(&mut ui, &interaction_states);

        let output = FullOutput::from_node_with_debug_and_measurer(
            ui,
            (size.width as f32, size.height as f32),
            if self.debug_options.is_enabled() {
                Some(self.debug_options)
            } else {
                None
            },
            Some(&mut self.text_engine),
        );

        let Some(gpu_state) = &mut self.gpu_state else {
            return;
        };

        let frame = gpu_state.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            gpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
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
        });

        gpu_state.renderer.render(
            &gpu_state.device,
            &gpu_state.queue,
            &mut encoder,
            &view,
            size.width as f32,
            size.height as f32,
            &output,
        );

        gpu_state.queue.submit(Some(encoder.finish()));
        frame.present();

        self.input_state.begin_frame();
    }

    fn build_ui(&mut self) -> Node {
        // Helper function to create a colored box
        let create_box = |color: Color, text: &str| {
            Node::new()
                .with_width(Size::fraction(1.0 / 3.0))
                .with_height(Size::fraction(1.0 / 3.0))
                .with_style(Style {
                    fill_color: Some(mocha::CRUST),
                    stroke: Some(Stroke::new(2.0, color)),
                    corner_shape: Some(CornerShape::Round(8.0)),
                    ..Default::default()
                })
                .with_h_align(HorizontalAlign::Center)
                .with_v_align(VerticalAlign::Center)
                .with_child(
                    Node::new().with_content(Content::Text(
                        TextContent::new(text)
                            .with_font_size(20.0)
                            .with_color(mocha::TEXT),
                    )),
                )
        };

        // Helper to create a container with alignment settings
        let create_container = |h_align: HorizontalAlign, v_align: VerticalAlign| {
            let h_label = match h_align {
                HorizontalAlign::Left => "Left",
                HorizontalAlign::Center => "Center",
                HorizontalAlign::Right => "Right",
            };
            let v_label = match v_align {
                VerticalAlign::Top => "Top",
                VerticalAlign::Center => "Center",
                VerticalAlign::Bottom => "Bottom",
            };

            Node::new()
                .with_layout_direction(Layout::Vertical)
                .with_children(vec![
                    // Label
                    Node::new()
                        // .with_height(Size::px(50.0))
                        .with_width(Size::Fill)
                        .with_margin(Spacing::bottom(20.0))
                        .with_content(Content::Text(TextContent {
                            text: format!("{} {}", h_label, v_label),
                            font_size: 24.0,
                            color: mocha::SUBTEXT0,
                            h_align: HorizontalAlign::Center,
                            v_align: VerticalAlign::Center,
                        })),
                    // Container with alignment
                    Node::new()
                        .with_width(Size::px(300.0))
                        .with_height(Size::px(300.0))
                        .with_style(Style {
                            fill_color: Some(mocha::CRUST),
                            stroke: Some(Stroke::new(2.0, mocha::SURFACE0)),
                            corner_shape: Some(CornerShape::Round(18.0)),
                            ..Default::default()
                        })
                        .with_padding(Spacing::all(12.0))
                        // .with_gap(8.0)
                        .with_layout_direction(Layout::Horizontal)
                        .with_h_align(h_align)
                        .with_v_align(v_align)
                        .with_child(create_box(mocha::BLUE, "Box")),
                ])
        };

        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_shape(Shape::rect())
            .with_style(Style {
                fill_color: Some(mocha::BASE),
                ..Default::default()
            })
            .with_layout_direction(Layout::Vertical)
            .with_gap(24.0)
            .with_children(vec![
                // Spacer
                Node::new().with_height(Size::Fill),
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::px(60.0))
                    .with_padding(Spacing::vertical(10.0))
                    .with_content(Content::Text(TextContent {
                        text: "Alignment Examples".to_string(),
                        font_size: 32.0,
                        color: mocha::TEXT,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    })),
                // Instructions
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(TextContent {
                        text: "h_align and v_align control child positioning within containers"
                            .to_string(),
                        font_size: 16.0,
                        color: mocha::SUBTEXT0,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    })),
                // Main content area
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(40.0)
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Content container
                        Node::new()
                            .with_layout_direction(Layout::Vertical)
                            .with_gap(36.0)
                            .with_children(vec![
                                // Horizontal Layout Examples
                                Node::new()
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(36.0)
                                    .with_children(vec![
                                        create_container(
                                            HorizontalAlign::Left,
                                            VerticalAlign::Top,
                                        ),
                                        create_container(
                                            HorizontalAlign::Center,
                                            VerticalAlign::Top,
                                        ),
                                        create_container(
                                            HorizontalAlign::Right,
                                            VerticalAlign::Top,
                                        ),
                                    ]),
                                // Vertical Layout Examples
                                Node::new()
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(36.0)
                                    .with_children(vec![
                                        create_container(
                                            HorizontalAlign::Left,
                                            VerticalAlign::Center,
                                        ),
                                        create_container(
                                            HorizontalAlign::Center,
                                            VerticalAlign::Center,
                                        ),
                                        create_container(
                                            HorizontalAlign::Right,
                                            VerticalAlign::Center,
                                        ),
                                    ]),
                                // Stack Layout Examples
                                Node::new()
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(36.0)
                                    .with_children(vec![
                                        create_container(
                                            HorizontalAlign::Left,
                                            VerticalAlign::Bottom,
                                        ),
                                        create_container(
                                            HorizontalAlign::Center,
                                            VerticalAlign::Bottom,
                                        ),
                                        create_container(
                                            HorizontalAlign::Right,
                                            VerticalAlign::Bottom,
                                        ),
                                    ]),
                            ]),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Spacer
                Node::new().with_height(Size::Fill),
                // Help text at bottom
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::px(30.0))
                    .with_padding(Spacing::horizontal(10.0))
                    .with_shape(Shape::rect())
                    .with_style(Style {
                        fill_color: Some(mocha::SURFACE0),
                        ..Default::default()
                    })
                    .with_content(Content::Text(
                        TextContent::new(
                            "M:Margins | P:Padding | B:Borders | C:Content | R:ClipRects | G:Gaps | O:Origins | D:All | S:RenderMode | ESC:Exit",
                        )
                        .with_font_size(16.0)
                        .with_color(mocha::TEXT)
                        .with_h_align(HorizontalAlign::Left)
                        .with_v_align(VerticalAlign::Center),
                    )),
            ])
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Alignment Example")
                .with_inner_size(winit::dpi::LogicalSize::new(1200, 1200));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            });

            let surface = instance.create_surface(window.clone()).unwrap();

            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }))
                .unwrap();

            let (device, queue) =
                pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    experimental_features: Default::default(),
                    trace: Default::default(),
                }))
                .unwrap();

            let size = window.inner_size();
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
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            surface.configure(&device, &config);

            let renderer = Renderer::new(&device, surface_format);

            self.window = Some(window);
            self.gpu_state = Some(GpuState {
                surface,
                device,
                queue,
                config,
                renderer,
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.input_state.handle_event(&event);

        // Handle debug keybinds
        if handle_debug_keybinds(
            &event,
            &mut self.debug_options,
            self.gpu_state.as_mut().map(|s| &mut s.renderer),
        ) {
            // Debug option changed, request redraw
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed()
                    && event.logical_key
                        == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
                {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.config.width = new_size.width.max(1);
                    gpu_state.config.height = new_size.height.max(1);
                    gpu_state
                        .surface
                        .configure(&gpu_state.device, &gpu_state.config);
                }
            }
            _ => {}
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
