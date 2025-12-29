//! Alignment example
//!
//! Demonstrates h_align and v_align working together for different layout directions.
//!
//! Controls:
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, FullOutput, HorizontalAlign,
    Layout, Node, Rect, Shape, Size, Spacing, Style, TextContent, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::{EventDispatcher, InputState, InteractiveStateManager, Renderer};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

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
                .with_width(Size::px(60.0))
                .with_height(Size::px(40.0))
                .with_shape(Shape::rect())
                .with_style(Style {
                    fill_color: Some(color),
                    corner_shape: Some(CornerShape::Round(8.0)),
                    ..Default::default()
                })
                .with_children(vec![Node::new().with_content(Content::Text(TextContent {
                    text: text.to_string(),
                    font_size: 14.0,
                    color: mocha::BASE,
                    h_align: HorizontalAlign::Center,
                    v_align: VerticalAlign::Center,
                }))])
                .with_h_align(HorizontalAlign::Center)
                .with_v_align(VerticalAlign::Center)
        };

        // Helper to create a container with alignment settings
        let create_container =
            |h_align: HorizontalAlign, v_align: VerticalAlign, layout: Layout| {
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
                let layout_label = match layout {
                    Layout::Horizontal => "Horizontal",
                    Layout::Vertical => "Vertical",
                    Layout::Stack => "Stack",
                };

                Node::new()
                    .with_width(Size::px(200.0))
                    .with_height(Size::px(200.0))
                    .with_layout_direction(Layout::Vertical)
                    .with_gap(4.0)
                    .with_children(vec![
                        // Label
                        Node::new()
                            .with_height(Size::px(40.0))
                            .with_content(Content::Text(TextContent {
                                text: format!("{} {}\n{}", h_label, v_label, layout_label),
                                font_size: 12.0,
                                color: mocha::SUBTEXT0,
                                h_align: HorizontalAlign::Center,
                                v_align: VerticalAlign::Center,
                            })),
                        // Container with alignment
                        Node::new()
                            .with_layout_direction(layout)
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_shape(Shape::rect())
                            .with_style(Style {
                                fill_color: Some(mocha::SURFACE0),
                                corner_shape: Some(CornerShape::Round(4.0)),
                                stroke_color: Some(mocha::SURFACE2),
                                stroke_width: Some(1.0),
                                ..Default::default()
                            })
                            .with_padding(Spacing::all(8.0))
                            .with_gap(8.0)
                            .with_h_align(h_align)
                            .with_v_align(v_align)
                            .with_children(vec![
                                create_box(mocha::RED, "1"),
                                create_box(mocha::GREEN, "2"),
                                create_box(mocha::BLUE, "3"),
                            ]),
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
            .with_padding(Spacing::all(20.0))
            .with_layout_direction(Layout::Vertical)
            .with_gap(20.0)
            .with_children(vec![
                // Title
                Node::new()
                    .with_height(Size::px(40.0))
                    .with_content(Content::Text(TextContent {
                        text: "Alignment Examples".to_string(),
                        font_size: 24.0,
                        color: mocha::TEXT,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    })),
                // Horizontal Layout Examples
                Node::new()
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(20.0)
                    .with_children(vec![
                        create_container(
                            HorizontalAlign::Left,
                            VerticalAlign::Top,
                            Layout::Horizontal,
                        ),
                        create_container(
                            HorizontalAlign::Center,
                            VerticalAlign::Center,
                            Layout::Horizontal,
                        ),
                        create_container(
                            HorizontalAlign::Right,
                            VerticalAlign::Bottom,
                            Layout::Horizontal,
                        ),
                    ]),
                // Vertical Layout Examples
                Node::new()
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(20.0)
                    .with_children(vec![
                        create_container(
                            HorizontalAlign::Left,
                            VerticalAlign::Top,
                            Layout::Vertical,
                        ),
                        create_container(
                            HorizontalAlign::Center,
                            VerticalAlign::Center,
                            Layout::Vertical,
                        ),
                        create_container(
                            HorizontalAlign::Right,
                            VerticalAlign::Bottom,
                            Layout::Vertical,
                        ),
                    ]),
                // Stack Layout Examples
                Node::new()
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(20.0)
                    .with_children(vec![
                        create_container(HorizontalAlign::Left, VerticalAlign::Top, Layout::Stack),
                        create_container(
                            HorizontalAlign::Center,
                            VerticalAlign::Center,
                            Layout::Stack,
                        ),
                        create_container(
                            HorizontalAlign::Right,
                            VerticalAlign::Bottom,
                            Layout::Stack,
                        ),
                    ]),
            ])
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Alignment Example")
                .with_inner_size(winit::dpi::LogicalSize::new(700, 900));

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
