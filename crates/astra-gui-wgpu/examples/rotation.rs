//! Rotation example
//!
//! Demonstrates rotation with nested rotations and interactive elements.
//!
//! Controls:
//! - Click buttons inside rotated containers to verify hit testing
//! - Drag sliders to adjust rotation angles
//! - Toggle switches work even when rotated
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, FullOutput, HorizontalAlign, Layout, Node, Rect,
    Shape, Size, Spacing, Stroke, StyledRect, TextContent, TransformOrigin, VerticalAlign,
};
use astra_gui_interactive::{
    button, button_clicked, slider, slider_drag, toggle, toggle_clicked, ButtonStyle, SliderStyle,
    ToggleStyle,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::{EventDispatcher, InputState, InteractiveStateManager, RenderMode, Renderer};

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

const DEBUG_HELP_TEXT_ONELINE: &str =
    "M:Margins | P:Padding | B:Borders | C:Content | R:ClipRects | G:Gaps | O:Origins | D:All | S:RenderMode | ESC:Exit";

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

use std::sync::Arc;

use wgpu::Trace;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    text_engine: TextEngine,

    // Input & interaction
    input_state: InputState,
    event_dispatcher: EventDispatcher,
    interactive_state_manager: InteractiveStateManager,

    // Application state
    outer_rotation: f32, // Degrees
    inner_rotation: f32, // Degrees
    counter: i32,
    toggle_state: bool,
    debug_options: DebugOptions,
    last_frame_time: std::time::Instant,
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
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
            outer_rotation: 30.0,
            inner_rotation: 0.0,
            counter: 0,
            toggle_state: true,
            debug_options: DebugOptions::none(),
            last_frame_time: std::time::Instant::now(),
        }
    }

    fn render(&mut self) {
        // Calculate delta time
        let now = std::time::Instant::now();
        let _delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        // Update frame time for transitions
        self.interactive_state_manager.begin_frame();

        // Build UI
        let mut ui = self.build_ui();

        // Get window size
        let size = match &self.window {
            Some(window) => window.inner_size(),
            None => return,
        };
        let window_rect = Rect::from_min_size([0.0, 0.0], [size.width as f32, size.height as f32]);
        ui.compute_layout_with_measurer(window_rect, &mut self.text_engine);

        // Generate events and interaction states from input
        let (events, interaction_states) =
            self.event_dispatcher.dispatch(&self.input_state, &mut ui);

        // Apply interactive styles with transitions
        self.interactive_state_manager
            .apply_styles(&mut ui, &interaction_states);

        // Handle button clicks
        if button_clicked("increment_btn", &events) {
            self.counter += 1;
            println!("Increment clicked! Counter: {}", self.counter);
        }

        if button_clicked("decrement_btn", &events) {
            self.counter -= 1;
            println!("Decrement clicked! Counter: {}", self.counter);
        }

        if button_clicked("reset_btn", &events) {
            self.counter = 0;
            self.outer_rotation = 0.0;
            self.inner_rotation = 0.0;
            println!("Reset clicked! Counter: {}", self.counter);
        }

        if toggle_clicked("toggle_switch", &events) {
            self.toggle_state = !self.toggle_state;
            println!("Toggle clicked! State: {}", self.toggle_state);
        }

        // Handle rotation sliders
        if slider_drag(
            "outer_rotation_slider",
            &mut self.outer_rotation,
            &(-180.0..=180.0),
            &events,
            &SliderStyle::default(),
            Some(1.0),
        ) {
            println!("Outer rotation: {:.1}°", self.outer_rotation);
        }

        if slider_drag(
            "inner_rotation_slider",
            &mut self.inner_rotation,
            &(-180.0..=180.0),
            &events,
            &SliderStyle::default(),
            Some(1.0),
        ) {
            println!("Inner rotation: {:.1}°", self.inner_rotation);
        }

        // Render
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

        // Get gpu_state after building UI to avoid borrow checker issues
        let Some(ref mut gpu_state) = self.gpu_state else {
            return;
        };

        let frame = gpu_state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            gpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Clear the screen with a dark background color
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        // Request redraw if there are active transitions
        if self.interactive_state_manager.has_active_transitions() {
            if let Some(ref window) = self.window {
                window.request_redraw();
            }
        }

        // Clear frame-specific input state for next frame
        self.input_state.begin_frame();
    }

    fn build_ui(&mut self) -> Node {
        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::lpx(24.0))
            .with_children(vec![
                // Spacer
                Node::new().with_height(Size::Fill),
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::lpx(60.0))
                    .with_padding(Spacing::vertical(Size::lpx(10.0)))
                    .with_content(Content::Text(TextContent {
                        text: "Transform Rotation Example".to_string(),
                        font_size: Size::lpx(32.0),
                        color: mocha::TEXT,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    })),
                // Instructions
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(TextContent {
                        text: "Adjust sliders to rotate containers. Click buttons to verify hit testing works!"
                            .to_string(),
                        font_size: Size::lpx(16.0),
                        color: mocha::SUBTEXT0,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    })),
                // Main content area with rotated containers
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(40.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Left side - Outer rotation control
                        Node::new()
                            .with_width(Size::lpx(300.0))
                            .with_height(Size::Fill)
                            .with_padding(Spacing::all(Size::lpx(20.0)))
                            .with_layout_direction(Layout::Vertical)
                            .with_gap(Size::lpx(20.0))
                            .with_children(vec![
                                // Outer rotation slider section
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(TextContent {
                                        text: "Outer Container Rotation".to_string(),
                                        font_size: Size::lpx(20.0),
                                        color: mocha::LAVENDER,
                                        h_align: HorizontalAlign::Center,
                                        v_align: VerticalAlign::Center,
                                    })),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::lpx(12.0))
                                    .with_children(vec![
                                        slider(
                                            "outer_rotation_slider",
                                            self.outer_rotation,
                                            -180.0..=180.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_content(Content::Text(TextContent {
                                                text: format!("{:.0}°", self.outer_rotation),
                                                font_size: Size::lpx(18.0),
                                                color: mocha::LAVENDER,
                                                h_align: HorizontalAlign::Right,
                                                v_align: VerticalAlign::Center,
                                            })),
                                    ]),
                                // Inner rotation slider section
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(TextContent {
                                        text: "Inner Container Rotation".to_string(),
                                        font_size: Size::lpx(20.0),
                                        color: mocha::GREEN,
                                        h_align: HorizontalAlign::Center,
                                        v_align: VerticalAlign::Center,
                                    })),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::lpx(12.0))
                                    .with_children(vec![
                                        slider(
                                            "inner_rotation_slider",
                                            self.inner_rotation,
                                            -180.0..=180.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_content(Content::Text(TextContent {
                                                text: format!("{:.0}°", self.inner_rotation),
                                                font_size: Size::lpx(18.0),
                                                color: mocha::LAVENDER,
                                                h_align: HorizontalAlign::Right,
                                                v_align: VerticalAlign::Center,
                                            })),
                                    ]),
                                // Counter display
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_height(Size::Fill)
                                    .with_layout_direction(Layout::Vertical)
                                    .with_gap(Size::lpx(10.0))
                                    .with_children(vec![
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(TextContent {
                                                text: "Counter".to_string(),
                                                font_size: Size::lpx(20.0),
                                                color: mocha::TEXT,
                                                h_align: HorizontalAlign::Center,
                                                v_align: VerticalAlign::Center,
                                            })),
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(TextContent {
                                                text: format!("{}", self.counter),
                                                font_size: Size::lpx(48.0),
                                                color: mocha::PEACH,
                                                h_align: HorizontalAlign::Center,
                                                v_align: VerticalAlign::Center,
                                            })),
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(TextContent {
                                                text: format!("Toggle: {}", if self.toggle_state { "ON" } else { "OFF" }),
                                                font_size: Size::lpx(20.0),
                                                color: if self.toggle_state { mocha::GREEN } else { mocha::RED },
                                                h_align: HorizontalAlign::Center,
                                                v_align: VerticalAlign::Center,
                                            })),
                                    ]),
                            ]),
                        // Right side - Rotated containers with interactive elements
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_children(vec![
                                // Outer rotated container (lavender)
                                Node::new()
                                    .with_width(Size::lpx(400.0))
                                    .with_height(Size::lpx(400.0))
                                    .with_rotation(self.outer_rotation.to_radians())
                                    .with_transform_origin(TransformOrigin::center())
                                    .with_shape(Shape::Rect(
                                        StyledRect::new(Default::default(), mocha::CRUST)
                                            .with_stroke(Stroke::new(Size::lpx(3.0), mocha::LAVENDER))
                                            .with_corner_shape(astra_gui::CornerShape::Round(50.0)),
                                    ))
                                    .with_padding(Spacing::all(Size::lpx(30.0)))
                                    .with_layout_direction(Layout::Vertical)
                                    .with_gap(Size::lpx(20.0))
                                    .with_children(vec![
                                        // Label for outer container
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(TextContent {
                                                text: "Outer Container".to_string(),
                                                font_size: Size::lpx(24.0),
                                                color: mocha::TEXT,
                                                h_align: HorizontalAlign::Center,
                                                v_align: VerticalAlign::Center,
                                            })),
                                        // Counter buttons in outer container
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_layout_direction(Layout::Horizontal)
                                            .with_gap(Size::lpx(12.0))
                                            .with_children(vec![
                                                button(
                                                    "decrement_btn",
                                                    "-",
                                                    false,
                                                    &ButtonStyle::default(),
                                                ),
                                                button(
                                                    "increment_btn",
                                                    "+",
                                                    false,
                                                    &ButtonStyle::default(),
                                                ),
                                                button(
                                                    "reset_btn",
                                                    "Reset",
                                                    false,
                                                    &ButtonStyle::default(),
                                                ),
                                            ]),
                                        // Inner rotated container (green)
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_height(Size::lpx(200.0))
                                            .with_rotation(self.inner_rotation.to_radians())
                                            .with_transform_origin(TransformOrigin::center())
                                            .with_shape(Shape::Rect(
                                                StyledRect::new(Default::default(), mocha::CRUST)
                                                    .with_stroke(Stroke::new(Size::lpx(2.0), mocha::GREEN))
                                                    .with_corner_shape(astra_gui::CornerShape::Cut(20.0)),
                                            ))
                                            .with_padding(Spacing::all(Size::lpx(20.0)))
                                            .with_layout_direction(Layout::Vertical)
                                            .with_gap(Size::lpx(15.0))
                                            .with_children(vec![
                                                // Label for inner container
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_content(Content::Text(TextContent {
                                                        text: "Inner Container".to_string(),
                                                        font_size: Size::lpx(20.0),
                                                        color: mocha::TEXT,
                                                        h_align: HorizontalAlign::Center,
                                                        v_align: VerticalAlign::Center,
                                                    })),
                                                // Toggle in inner container
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_layout_direction(Layout::Horizontal)
                                                    .with_gap(Size::lpx(12.0))
                                                    .with_children(vec![
                                                        Node::new()
                                                            .with_width(Size::Fill)
                                                            .with_content(Content::Text(TextContent {
                                                                text: "Toggle:".to_string(),
                                                                font_size: Size::lpx(18.0),
                                                                color: mocha::TEXT,
                                                                h_align: HorizontalAlign::Right,
                                                                v_align: VerticalAlign::Center,
                                                            })),
                                                        toggle(
                                                            "toggle_switch",
                                                            self.toggle_state,
                                                            false,
                                                            &ToggleStyle::default(),
                                                        ),
                                                    ]),
                                                // Nested rotation info
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_content(Content::Text(TextContent {
                                                        text: format!(
                                                            "Total: {:.0}°",
                                                            self.outer_rotation + self.inner_rotation
                                                        ),
                                                        font_size: Size::lpx(16.0),
                                                        color: mocha::TEXT,
                                                        h_align: HorizontalAlign::Center,
                                                        v_align: VerticalAlign::Center,
                                                    })),
                                            ]),
                                    ]),
                            ]),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Spacer
                Node::new().with_height(Size::Fill),
                // Help bar
                Node::new()
                    .with_width(Size::Fill)
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
                    )),
            ])
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Rotation Example - Astra GUI")
            .with_inner_size(winit::dpi::LogicalSize::new(1200, 800));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        self.window = Some(window.clone());

        let gpu_state = pollster::block_on(GpuState::new(window));
        self.gpu_state = Some(gpu_state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if matches!(
                key_event.physical_key,
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
            ) && key_event.state == ElementState::Pressed =>
            {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if !matches!(
                key_event.physical_key,
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
            ) =>
            {
                self.input_state.handle_event(&event);
                let renderer = self.gpu_state.as_mut().map(|s| &mut s.renderer);
                let _handled = handle_debug_keybinds(&event, &mut self.debug_options, renderer);
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { .. } | WindowEvent::MouseInput { .. } => {
                self.input_state.handle_event(&event);
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(ref mut gpu_state) = self.gpu_state {
                    gpu_state.config.width = new_size.width.max(1);
                    gpu_state.config.height = new_size.height.max(1);
                    gpu_state
                        .surface
                        .configure(&gpu_state.device, &gpu_state.config);
                }
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl GpuState {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: Trace::Off,
            })
            .await
            .expect("Failed to create device");

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
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
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
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    println!("{}", DEBUG_HELP_TEXT);

    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}
