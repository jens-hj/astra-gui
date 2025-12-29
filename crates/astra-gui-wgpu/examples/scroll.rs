//! Overflow::Scroll example
//!
//! Demonstrates scrollable containers with mouse wheel support.
//!
//! Controls:
//! - Mouse wheel to scroll
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, FullOutput, HorizontalAlign, Layout, Node,
    NodeId, Overflow, Shape, Size, Spacing, Style, TextContent, VerticalAlign,
};
use astra_gui_wgpu::{
    EventDispatcher, InputState, InteractionEvent, InteractiveStateManager, Renderer,
};
use std::collections::HashMap;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    scroll_offsets: HashMap<String, (f32, f32)>,
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    input: InputState,
    event_dispatcher: EventDispatcher,
    interactive_state_manager: InteractiveStateManager,
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
            input: InputState::new(),
            event_dispatcher: EventDispatcher::new(),
            interactive_state_manager: InteractiveStateManager::new(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(
        &mut self,
        scroll_offsets: &HashMap<String, (f32, f32)>,
    ) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
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
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // Build UI
        let mut ui = create_demo_ui(
            self.config.width as f32,
            self.config.height as f32,
            scroll_offsets,
        );

        // Dispatch events
        let (events, interaction_states) = self.event_dispatcher.dispatch(&self.input, &mut ui);

        // Apply interactive styles
        self.interactive_state_manager.begin_frame();
        self.interactive_state_manager
            .apply_styles(&mut ui, &interaction_states);

        // Render UI
        let ui_output =
            FullOutput::from_node(ui, (self.config.width as f32, self.config.height as f32));

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

        surface_texture.present();

        Ok(())
    }
}

fn create_demo_ui(_width: f32, _height: f32, scroll_offsets: &HashMap<String, (f32, f32)>) -> Node {
    // Create a scrollable container with many items
    let mut items = Vec::new();
    for i in 0..30 {
        items.push(
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::px(50.0))
                .with_shape(Shape::rect())
                .with_style(Style {
                    fill_color: Some(if i % 2 == 0 {
                        mocha::SURFACE0
                    } else {
                        mocha::SURFACE1
                    }),
                    corner_shape: Some(CornerShape::Round(8.0)),
                    ..Default::default()
                })
                .with_content(Content::Text(TextContent {
                    text: format!("Item {}", i + 1),
                    font_size: 24.0,
                    color: mocha::TEXT,
                    h_align: HorizontalAlign::Center,
                    v_align: VerticalAlign::Center,
                })),
        );
    }

    // Scrollable container
    let scroll_offset = scroll_offsets
        .get("scroll_container")
        .copied()
        .unwrap_or((0.0, 0.0));
    let mut scroll_container = Node::new()
        .with_id(NodeId::new("scroll_container"))
        .with_width(Size::px(400.0))
        .with_height(Size::px(500.0))
        .with_padding(Spacing::all(10.0))
        .with_gap(10.0)
        .with_layout_direction(Layout::Vertical)
        .with_overflow(Overflow::Scroll)
        .with_shape(Shape::rect())
        .with_style(Style {
            fill_color: Some(mocha::MANTLE),
            corner_shape: Some(CornerShape::Round(12.0)),
            ..Default::default()
        })
        .with_children(items);

    scroll_container.set_scroll_offset(scroll_offset);

    // Root with centered layout
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(40.0))
        .with_child(
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_layout_direction(Layout::Vertical)
                .with_gap(20.0)
                .with_children(vec![
                    // Title
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::px(60.0))
                        .with_content(Content::Text(TextContent {
                            text: "Scroll Example - Use Mouse Wheel".to_string(),
                            font_size: 32.0,
                            color: mocha::TEXT,
                            h_align: HorizontalAlign::Center,
                            v_align: VerticalAlign::Center,
                        })),
                    // Centered scroll container
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::Fill)
                        .with_layout_direction(Layout::Horizontal)
                        .with_child(Node::new().with_width(Size::Fill)) // Left spacer
                        .with_child(scroll_container)
                        .with_child(Node::new().with_width(Size::Fill)), // Right spacer
                ]),
        )
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Astra GUI - Scroll Example")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 700));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());
            self.gpu_state = Some(pollster::block_on(GpuState::new(window)));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),

            WindowEvent::Resized(physical_size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.resize(physical_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    // Handle scroll events before rendering
                    gpu_state.input.begin_frame();

                    // Build UI to get events
                    let mut ui = create_demo_ui(
                        gpu_state.config.width as f32,
                        gpu_state.config.height as f32,
                        &self.scroll_offsets,
                    );

                    let (events, _) = gpu_state
                        .event_dispatcher
                        .dispatch(&gpu_state.input, &mut ui);

                    // Process scroll events
                    for event in &events {
                        if let InteractionEvent::Scroll { delta, .. } = event.event {
                            if event.target.as_str() == "scroll_container" {
                                let current = self
                                    .scroll_offsets
                                    .get("scroll_container")
                                    .copied()
                                    .unwrap_or((0.0, 0.0));
                                // Invert delta: positive scroll should move content down (negative offset)
                                let new_offset = (
                                    current.0 - delta.0,
                                    (current.1 - delta.1).max(0.0), // Clamp to not scroll past top
                                );
                                self.scroll_offsets
                                    .insert("scroll_container".to_string(), new_offset);
                                println!("Scroll offset: {:?}", new_offset);
                            }
                        }
                    }

                    match gpu_state.render(&self.scroll_offsets) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            if let Some(window) = &self.window {
                                gpu_state.resize(window.inner_size())
                            }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }
                }
            }

            _ => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.input.handle_event(&event);
                }
            }
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

    let mut app = App {
        window: None,
        gpu_state: None,
        scroll_offsets: HashMap::new(),
    };

    println!("Use mouse wheel to scroll the container");
    println!("ESC to exit");

    event_loop.run_app(&mut app).unwrap();
}
