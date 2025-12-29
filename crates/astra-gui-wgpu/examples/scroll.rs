//! Overflow::Scroll example
//!
//! Demonstrates scrollable containers with mouse wheel support.
//!
//! Controls:
//! - Mouse wheel to scroll
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Content, CornerShape, DebugOptions, FullOutput, HorizontalAlign, Layout,
    Node, NodeId, Overflow, ScrollDirection, Shape, Size, Spacing, Style, TextContent,
    VerticalAlign,
};
use astra_gui_wgpu::{
    EventDispatcher, InputState, InteractionEvent, InteractiveStateManager, RenderMode, Renderer,
};
use std::collections::HashMap;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
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

struct App {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    scroll_offsets: HashMap<String, (f32, f32)>, // Current scroll positions
    scroll_targets: HashMap<String, (f32, f32)>, // Target scroll positions for smooth scrolling
    debug_options: DebugOptions,
    last_frame_time: Option<std::time::Instant>,
    item_heights: Vec<f32>, // Random heights for each item, generated once
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
        debug_options: &DebugOptions,
        item_heights: &[f32],
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
            item_heights,
        );

        // Dispatch events
        let (_events, interaction_states) = self.event_dispatcher.dispatch(&self.input, &mut ui);

        // Apply interactive styles
        self.interactive_state_manager.begin_frame();
        self.interactive_state_manager
            .apply_styles(&mut ui, &interaction_states);

        // Render UI
        let ui_output = FullOutput::from_node_with_debug(
            ui,
            (self.config.width as f32, self.config.height as f32),
            if debug_options.is_enabled() {
                Some(*debug_options)
            } else {
                None
            },
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

        surface_texture.present();

        Ok(())
    }
}

/// Helper function to find a node by its ID
fn find_node_by_id<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
    if node.id().map(|node_id| node_id.as_str()) == Some(id) {
        return Some(node);
    }

    for child in node.children() {
        if let Some(found) = find_node_by_id(child, id) {
            return Some(found);
        }
    }

    None
}

/// Calculate maximum scroll offset for a container
fn calculate_max_scroll(container: &Node) -> (f32, f32) {
    let Some(layout) = container.computed_layout() else {
        return (0.0, 0.0);
    };

    // Get container dimensions (after padding)
    let padding = container.padding();
    let container_width = layout.rect.max[0] - layout.rect.min[0] - padding.left - padding.right;
    let container_height = layout.rect.max[1] - layout.rect.min[1] - padding.top - padding.bottom;

    // Calculate total content size
    let gap = container.gap();
    let children = container.children();

    if children.is_empty() {
        return (0.0, 0.0);
    }

    let mut content_width = 0.0f32;
    let mut content_height = 0.0f32;

    for (i, child) in children.iter().enumerate() {
        if let Some(child_layout) = child.computed_layout() {
            let child_width = child_layout.rect.max[0] - child_layout.rect.min[0];
            let child_height = child_layout.rect.max[1] - child_layout.rect.min[1];

            content_width = content_width.max(child_width);
            content_height += child_height;

            // Add gap between items (but not after the last one)
            if i < children.len() - 1 {
                content_height += gap;
            }
        }
    }

    // Max scroll is the amount content exceeds container size
    let max_scroll_x = (content_width - container_width).max(0.0);
    let max_scroll_y = (content_height - container_height).max(0.0);

    (max_scroll_x, max_scroll_y)
}

fn create_demo_ui(
    _width: f32,
    _height: f32,
    scroll_offsets: &HashMap<String, (f32, f32)>,
    item_heights: &[f32],
) -> Node {
    // Create a scrollable container with many items
    let mut items = Vec::new();
    for (i, &height) in item_heights.iter().enumerate() {
        items.push(
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::px(height))
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
                    text: format!("Item {}, height {:.2}", i + 1, height),
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
        .with_height(Size::px(800.0))
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

            WindowEvent::KeyboardInput { .. } => {
                // Debug controls
                let renderer = self.gpu_state.as_mut().map(|s| &mut s.renderer);
                let _handled = handle_debug_keybinds(&event, &mut self.debug_options, renderer);
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.resize(physical_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    // Build UI to dispatch events
                    let mut ui = create_demo_ui(
                        gpu_state.config.width as f32,
                        gpu_state.config.height as f32,
                        &self.scroll_offsets,
                        &self.item_heights,
                    );

                    // IMPORTANT: Compute layout before dispatching events so hit testing works
                    let window_rect = astra_gui::Rect::new(
                        [0.0, 0.0],
                        [
                            gpu_state.config.width as f32,
                            gpu_state.config.height as f32,
                        ],
                    );
                    ui.compute_layout(window_rect);

                    let (events, _) = gpu_state
                        .event_dispatcher
                        .dispatch(&gpu_state.input, &mut ui);

                    // Process scroll events and update offsets
                    for event in &events {
                        if let InteractionEvent::Scroll { delta, .. } = event.event {
                            if event.target.as_str() == "scroll_container" {
                                // Find the node to get its scroll properties
                                if let Some(node) = find_node_by_id(&ui, "scroll_container") {
                                    let scroll_speed = node.scroll_speed();
                                    let scroll_direction = node.scroll_direction();

                                    let current = self
                                        .scroll_offsets
                                        .get("scroll_container")
                                        .copied()
                                        .unwrap_or((0.0, 0.0));

                                    // Apply scroll speed and direction
                                    let direction_multiplier = match scroll_direction {
                                        ScrollDirection::Normal => 1.0,
                                        ScrollDirection::Inverted => -1.0,
                                    };

                                    let adjusted_delta = (
                                        delta.0 * scroll_speed * direction_multiplier,
                                        delta.1 * scroll_speed * direction_multiplier,
                                    );

                                    // Calculate max scroll based on content size
                                    let max_scroll = calculate_max_scroll(node);

                                    let new_target = (
                                        current.0 + adjusted_delta.0,
                                        (current.1 + adjusted_delta.1).clamp(0.0, max_scroll.1),
                                    );
                                    self.scroll_targets
                                        .insert("scroll_container".to_string(), new_target);
                                }
                            }
                        }
                    }

                    // Smooth scroll interpolation
                    let now = std::time::Instant::now();
                    let dt = self
                        .last_frame_time
                        .map(|t| (now - t).as_secs_f32())
                        .unwrap_or(0.016); // Default to ~60fps
                    self.last_frame_time = Some(now);

                    const SCROLL_SMOOTHNESS: f32 = 10.0; // Higher = faster, lower = smoother
                    for (id, target) in &self.scroll_targets {
                        let current = self.scroll_offsets.get(id).copied().unwrap_or((0.0, 0.0));
                        let t = 1.0 - (-SCROLL_SMOOTHNESS * dt).exp(); // Exponential ease-out
                        let new_offset = (
                            current.0 + (target.0 - current.0) * t,
                            current.1 + (target.1 - current.1) * t,
                        );
                        self.scroll_offsets.insert(id.clone(), new_offset);
                    }

                    // Clear input for next frame
                    gpu_state.input.begin_frame();

                    match gpu_state.render(
                        &self.scroll_offsets,
                        &self.debug_options,
                        &self.item_heights,
                    ) {
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

            WindowEvent::MouseWheel { .. } => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.input.handle_event(&event);
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

    // Generate random heights for items once at startup
    let item_heights: Vec<f32> = (0..30)
        .map(|_| rand::random::<f32>() * 100.0 + 50.0)
        .collect();

    let mut app = App {
        window: None,
        gpu_state: None,
        scroll_offsets: HashMap::new(),
        scroll_targets: HashMap::new(),
        debug_options: DebugOptions::none(),
        last_frame_time: None,
        item_heights,
    };

    println!("{}", DEBUG_HELP_TEXT);
    println!();
    println!("Use mouse wheel to scroll the container");

    event_loop.run_app(&mut app).unwrap();
}
