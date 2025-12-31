//! Slider with value widget example
//!
//! Demonstrates the combined slider + drag value component.
//!
//! Controls:
//! - Drag slider or value field to adjust
//! - Hold Shift while dragging value for precise control (0.1x speed)
//! - Hold Ctrl while dragging value for fast control (10x speed)
//! - Click on value to enter text input mode
//! - Press Enter to confirm or Escape to cancel text input
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, FullOutput, HorizontalAlign, Layout, Node, Rect,
    Shape, Size, Spacing, StyledRect, TextContent, VerticalAlign,
};
use astra_gui_interactive::{
    slider_with_value, slider_with_value_update, DragValueStyle, SliderStyle,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::{EventDispatcher, InputState, InteractiveStateManager, RenderMode, Renderer};
use std::ops::RangeInclusive;

const DEBUG_HELP_TEXT: &str = "Debug controls:
  M - Toggle margins (red overlay)
  P - Toggle padding (blue overlay)
  B - Toggle borders (green outline)
  C - Toggle content area (yellow outline)
  R - Toggle clip rects (red outline)
  G - Toggle gaps (purple overlay)
  D - Toggle all debug visualizations
  S - Toggle render mode (SDF/Mesh)
  ESC - Exit";

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

use std::sync::Arc;

use wgpu::Trace;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct SliderWithValueState {
    value: f32,
    text_buffer: String,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    focused: bool,
    drag_accumulator: f32,
}

impl SliderWithValueState {
    fn new(value: f32) -> Self {
        Self {
            value,
            text_buffer: String::new(),
            cursor_pos: 0,
            selection: None,
            focused: false,
            drag_accumulator: value,
        }
    }
}

struct App {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    text_engine: TextEngine,

    // Input & interaction
    input_state: InputState,
    event_dispatcher: EventDispatcher,
    interactive_state_manager: InteractiveStateManager,

    // Application state
    basic_slider: SliderWithValueState,
    clamped_slider: SliderWithValueState,
    stepped_slider: SliderWithValueState,
    disabled_slider: SliderWithValueState,

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
            basic_slider: SliderWithValueState::new(42.5),
            clamped_slider: SliderWithValueState::new(50.0),
            stepped_slider: SliderWithValueState::new(10.0),
            disabled_slider: SliderWithValueState::new(99.9),
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

        // Get window size
        let size = match &self.window {
            Some(window) => window.inner_size(),
            None => return,
        };

        // Build UI
        let mut ui = self.build_ui();

        let window_rect = Rect::from_min_size([0.0, 0.0], [size.width as f32, size.height as f32]);
        ui.compute_layout_with_measurer(window_rect, &mut self.text_engine);

        // Generate events and interaction states from input
        let (events, interaction_states) =
            self.event_dispatcher.dispatch(&self.input_state, &mut ui);

        // Apply interactive styles with transitions
        self.interactive_state_manager
            .apply_styles(&mut ui, &interaction_states);

        // Handle slider with value updates
        if slider_with_value_update(
            "basic_slider",
            "basic_value",
            &mut self.basic_slider.value,
            &mut self.basic_slider.text_buffer,
            &mut self.basic_slider.cursor_pos,
            &mut self.basic_slider.selection,
            &mut self.basic_slider.focused,
            &mut self.basic_slider.drag_accumulator,
            &events,
            &self.input_state,
            &mut self.event_dispatcher,
            0.0..=100.0,
            0.1, // speed
            None,
        ) {
            println!("Basic value: {:.2}", self.basic_slider.value);
        }

        if slider_with_value_update(
            "clamped_slider",
            "clamped_value",
            &mut self.clamped_slider.value,
            &mut self.clamped_slider.text_buffer,
            &mut self.clamped_slider.cursor_pos,
            &mut self.clamped_slider.selection,
            &mut self.clamped_slider.focused,
            &mut self.clamped_slider.drag_accumulator,
            &events,
            &self.input_state,
            &mut self.event_dispatcher,
            0.0..=100.0,
            0.1, // speed
            None,
        ) {
            println!("Clamped value: {:.2}", self.clamped_slider.value);
        }

        if slider_with_value_update(
            "stepped_slider",
            "stepped_value",
            &mut self.stepped_slider.value,
            &mut self.stepped_slider.text_buffer,
            &mut self.stepped_slider.cursor_pos,
            &mut self.stepped_slider.selection,
            &mut self.stepped_slider.focused,
            &mut self.stepped_slider.drag_accumulator,
            &events,
            &self.input_state,
            &mut self.event_dispatcher,
            0.0..=100.0,
            0.1,       // speed
            Some(5.0), // step
        ) {
            println!("Stepped value: {:.1}", self.stepped_slider.value);
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
        // Extract values to avoid borrow checker issues
        let basic_val = self.basic_slider.value;
        let basic_focused = self.basic_slider.focused;
        let basic_text = self.basic_slider.text_buffer.clone();
        let basic_cursor = self.basic_slider.cursor_pos;
        let basic_selection = self.basic_slider.selection;

        let clamped_val = self.clamped_slider.value;
        let clamped_focused = self.clamped_slider.focused;
        let clamped_text = self.clamped_slider.text_buffer.clone();
        let clamped_cursor = self.clamped_slider.cursor_pos;
        let clamped_selection = self.clamped_slider.selection;

        let stepped_val = self.stepped_slider.value;
        let stepped_focused = self.stepped_slider.focused;
        let stepped_text = self.stepped_slider.text_buffer.clone();
        let stepped_cursor = self.stepped_slider.cursor_pos;
        let stepped_selection = self.stepped_slider.selection;

        let disabled_val = self.disabled_slider.value;
        let disabled_focused = self.disabled_slider.focused;
        let disabled_text = self.disabled_slider.text_buffer.clone();
        let disabled_cursor = self.disabled_slider.cursor_pos;
        let disabled_selection = self.disabled_slider.selection;

        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::lpx(20.0))
            .with_children(vec![
                // Spacer
                Node::new().with_height(Size::Fill),
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(TextContent {
                        text: "Slider with Value Widget Example".to_string(),
                        font_size: Size::lpx(32.0),
                        color: mocha::TEXT,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    })),
                // Instructions
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(TextContent {
                        text:
                            "Drag slider or value • Click value to type • Shift=precise, Ctrl=fast"
                                .to_string(),
                        font_size: Size::lpx(16.0),
                        color: mocha::SUBTEXT0,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    })),
                Node::new().with_height(Size::lpx(20.0)),
                // Basic slider
                self.create_slider_row(
                    "Basic (0-100):",
                    "basic_slider",
                    "basic_value",
                    basic_val,
                    0.0..=100.0,
                    basic_focused,
                    &basic_text,
                    basic_cursor,
                    basic_selection,
                    false,
                ),
                // Clamped slider
                self.create_slider_row(
                    "Clamped (0-100):",
                    "clamped_slider",
                    "clamped_value",
                    clamped_val,
                    0.0..=100.0,
                    clamped_focused,
                    &clamped_text,
                    clamped_cursor,
                    clamped_selection,
                    false,
                ),
                // Stepped slider
                self.create_slider_row(
                    "Stepped (5.0 steps):",
                    "stepped_slider",
                    "stepped_value",
                    stepped_val,
                    0.0..=100.0,
                    stepped_focused,
                    &stepped_text,
                    stepped_cursor,
                    stepped_selection,
                    false,
                ),
                // Disabled slider
                self.create_slider_row(
                    "Disabled:",
                    "disabled_slider",
                    "disabled_value",
                    disabled_val,
                    0.0..=100.0,
                    disabled_focused,
                    &disabled_text,
                    disabled_cursor,
                    disabled_selection,
                    true,
                ),
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

    fn create_slider_row(
        &mut self,
        label: &str,
        slider_id: &str,
        value_id: &str,
        value: f32,
        range: RangeInclusive<f32>,
        focused: bool,
        text_buffer: &str,
        cursor_pos: usize,
        selection: Option<(usize, usize)>,
        disabled: bool,
    ) -> Node {
        Node::new()
            .with_width(Size::Fill)
            .with_layout_direction(Layout::Horizontal)
            .with_gap(Size::lpx(16.0))
            .with_children(vec![
                // Spacer
                Node::new().with_width(Size::Fill),
                // Label
                Node::new()
                    .with_width(Size::lpx(200.0))
                    .with_height(Size::Fill)
                    .with_content(Content::Text(TextContent {
                        text: label.to_string(),
                        font_size: Size::lpx(20.0),
                        color: mocha::TEXT,
                        h_align: HorizontalAlign::Right,
                        v_align: VerticalAlign::Center,
                    })),
                // Slider with value widget
                slider_with_value(
                    slider_id,
                    value_id,
                    value,
                    range,
                    focused,
                    disabled,
                    &SliderStyle::default(),
                    &DragValueStyle::default().with_precision(1),
                    text_buffer,
                    cursor_pos,
                    selection,
                    &mut self.text_engine,
                    &mut self.event_dispatcher,
                ),
                // Spacer
                Node::new().with_width(Size::Fill),
            ])
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Slider with Value Widget - Astra GUI")
            .with_inner_size(winit::dpi::LogicalSize::new(900, 600));

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
                // Only exit on ESC if nothing is focused
                if self.event_dispatcher.focused_node().is_none() {
                    event_loop.exit();
                } else {
                    // Pass to keyboard handler to unfocus
                    self.input_state.handle_event(&event);
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if !matches!(
                key_event.physical_key,
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
            ) =>
            {
                // First, pass keyboard events to input state
                self.input_state.handle_event(&event);

                // Only handle debug shortcuts if nothing is focused
                // When a field is focused, all keyboard input should go to that field
                let has_focus = self.event_dispatcher.focused_node().is_some();

                if !has_focus {
                    let renderer = self.gpu_state.as_mut().map(|s| &mut s.renderer);
                    let _handled = handle_debug_keybinds(&event, &mut self.debug_options, renderer);
                }

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
    println!("\nSlider with Value Controls:");
    println!("  Drag slider - Adjust value");
    println!("  Drag value field - Adjust value");
    println!("  Shift + Drag value - Precise control (0.1x speed)");
    println!("  Ctrl + Drag value - Fast control (10x speed)");
    println!("  Click value - Enter text input mode");
    println!("  Enter - Confirm text input");
    println!("  Escape - Cancel text input / Exit");

    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}
