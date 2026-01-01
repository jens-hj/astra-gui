//! Overflow::Scroll example
//!
//! Demonstrates scrollable containers with mouse wheel support.
//!
//! Controls:
//! - Mouse wheel to scroll
//! - ESC: quit

use astra_gui::{
    catppuccin::mocha, Content, CornerShape, DebugOptions, FullOutput, HorizontalAlign, Layout,
    Node, NodeId, Overflow, Shape, Size, Spacing, Style, TextContent, VerticalAlign,
};
use astra_gui_wgpu::{EventDispatcher, InputState, InteractiveStateManager, RenderMode, Renderer};
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
    debug_options: DebugOptions,
    last_frame_time: Option<std::time::Instant>,
    item_heights: Vec<f32>, // Random heights for vertical scroll items
    item_widths: Vec<f32>,  // Random widths for horizontal scroll items
    frame_count: u64,       // For performance logging
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
        ui_output: &FullOutput,
    ) -> Result<(std::time::Duration, std::time::Duration), wgpu::SurfaceError> {
        let gpu_work_start = std::time::Instant::now();

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
                multiview_mask: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));

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

        let gpu_work_time = gpu_work_start.elapsed();

        let present_start = std::time::Instant::now();
        surface_texture.present();
        let present_time = present_start.elapsed();

        Ok((gpu_work_time, present_time))
    }
}

fn create_demo_ui(_width: f32, _height: f32, item_heights: &[f32], item_widths: &[f32]) -> Node {
    // Create a scrollable container with many items - STRESS TEST with nested children
    let mut items = Vec::new();
    for (i, &height) in item_heights.iter().enumerate() {
        // Create nested children for each item
        let nested_children = vec![
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::lpx(30.0))
                .with_content(Content::Text(TextContent {
                    text: format!("Item {}", i + 1),
                    font_size: Size::lpx(20.0),
                    color: mocha::TEXT,
                    h_align: HorizontalAlign::Left,
                    v_align: VerticalAlign::Center,
                })),
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_layout_direction(Layout::Horizontal)
                .with_gap(Size::lpx(5.0))
                .with_children(vec![
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::Fill)
                        .with_shape(Shape::rect())
                        .with_style(Style {
                            fill_color: Some(mocha::BLUE),
                            corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                            ..Default::default()
                        }),
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::Fill)
                        .with_shape(Shape::rect())
                        .with_style(Style {
                            fill_color: Some(mocha::GREEN),
                            corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                            ..Default::default()
                        }),
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::Fill)
                        .with_shape(Shape::rect())
                        .with_style(Style {
                            fill_color: Some(mocha::RED),
                            corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                            ..Default::default()
                        }),
                ]),
        ];

        items.push(
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::lpx(height))
                .with_padding(Spacing::all(Size::lpx(8.0)))
                .with_gap(Size::lpx(5.0))
                .with_layout_direction(Layout::Vertical)
                .with_shape(Shape::rect())
                .with_style(Style {
                    fill_color: Some(if i % 2 == 0 {
                        mocha::SURFACE0
                    } else {
                        mocha::SURFACE1
                    }),
                    corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                    ..Default::default()
                })
                .with_children(nested_children),
        );
    }

    // Scrollable container - scroll state is now managed automatically
    let scroll_container = Node::new()
        .with_id(NodeId::new("scroll_container"))
        .with_width(Size::lpx(400.0))
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(10.0)))
        .with_gap(Size::lpx(10.0))
        .with_layout_direction(Layout::Vertical)
        .with_overflow(Overflow::Scroll)
        .with_shape(Shape::rect())
        .with_style(Style {
            fill_color: Some(mocha::MANTLE),
            corner_shape: Some(CornerShape::Round(Size::lpx(12.0))),
            ..Default::default()
        })
        .with_children(items);

    // Create horizontal scrollable container - STRESS TEST with nested children
    let mut horizontal_items = Vec::new();
    for (i, &width) in item_widths.iter().enumerate() {
        horizontal_items.push(
            Node::new()
                .with_width(Size::lpx(width))
                .with_height(Size::Fill)
                .with_padding(Spacing::all(Size::lpx(8.0)))
                .with_gap(Size::lpx(5.0))
                .with_layout_direction(Layout::Vertical)
                .with_shape(Shape::rect())
                .with_style(Style {
                    fill_color: Some(if i % 2 == 0 {
                        mocha::SURFACE0
                    } else {
                        mocha::SURFACE1
                    }),
                    corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                    ..Default::default()
                })
                .with_children(vec![
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::lpx(30.0))
                        .with_content(Content::Text(TextContent {
                            text: format!("H-Item {}", i + 1),
                            font_size: Size::lpx(18.0),
                            color: mocha::TEXT,
                            h_align: HorizontalAlign::Center,
                            v_align: VerticalAlign::Center,
                        })),
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::Fill)
                        .with_layout_direction(Layout::Vertical)
                        .with_gap(Size::lpx(3.0))
                        .with_children(vec![
                            Node::new()
                                .with_width(Size::Fill)
                                .with_height(Size::Fill)
                                .with_shape(Shape::rect())
                                .with_style(Style {
                                    fill_color: Some(mocha::PEACH),
                                    corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                    ..Default::default()
                                }),
                            Node::new()
                                .with_width(Size::Fill)
                                .with_height(Size::Fill)
                                .with_shape(Shape::rect())
                                .with_style(Style {
                                    fill_color: Some(mocha::YELLOW),
                                    corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                    ..Default::default()
                                }),
                            Node::new()
                                .with_width(Size::Fill)
                                .with_height(Size::Fill)
                                .with_shape(Shape::rect())
                                .with_style(Style {
                                    fill_color: Some(mocha::TEAL),
                                    corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                    ..Default::default()
                                }),
                        ]),
                ]),
        );
    }

    let horizontal_scroll_container = Node::new()
        .with_id(NodeId::new("horizontal_scroll_container"))
        .with_width(Size::lpx(800.0))
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(10.0)))
        .with_gap(Size::lpx(10.0))
        .with_layout_direction(Layout::Horizontal)
        .with_overflow(Overflow::Scroll)
        .with_shape(Shape::rect())
        .with_style(Style {
            fill_color: Some(mocha::MANTLE),
            corner_shape: Some(CornerShape::Round(Size::lpx(12.0))),
            ..Default::default()
        })
        .with_children(horizontal_items);

    // Create 2D scrollable container (scrolls both X and Y) - STRESS TEST with nested children
    let mut grid_items = Vec::new();
    for row in 0..50 {
        // 10 -> 50 rows
        let mut row_items = Vec::new();
        for col in 0..50 {
            // 10 -> 50 columns (2500 total grid cells!)
            row_items.push(
                Node::new()
                    .with_width(Size::lpx(150.0))
                    .with_height(Size::lpx(100.0))
                    .with_padding(Spacing::all(Size::lpx(6.0)))
                    .with_gap(Size::lpx(3.0))
                    .with_layout_direction(Layout::Vertical)
                    .with_shape(Shape::rect())
                    .with_style(Style {
                        fill_color: Some(if (row + col) % 2 == 0 {
                            mocha::SURFACE0
                        } else {
                            mocha::SURFACE1
                        }),
                        corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                        ..Default::default()
                    })
                    .with_children(vec![
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::lpx(25.0))
                            .with_content(Content::Text(TextContent {
                                text: format!("R{} C{}", row + 1, col + 1),
                                font_size: Size::lpx(16.0),
                                color: mocha::TEXT,
                                h_align: HorizontalAlign::Center,
                                v_align: VerticalAlign::Center,
                            })),
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_layout_direction(Layout::Horizontal)
                            .with_gap(Size::lpx(2.0))
                            .with_children(vec![
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_height(Size::Fill)
                                    .with_shape(Shape::rect())
                                    .with_style(Style {
                                        fill_color: Some(mocha::MAUVE),
                                        corner_shape: Some(CornerShape::Round(Size::lpx(3.0))),
                                        ..Default::default()
                                    }),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_height(Size::Fill)
                                    .with_shape(Shape::rect())
                                    .with_style(Style {
                                        fill_color: Some(mocha::LAVENDER),
                                        corner_shape: Some(CornerShape::Round(Size::lpx(3.0))),
                                        ..Default::default()
                                    }),
                            ]),
                    ]),
            );
        }

        grid_items.push(
            Node::new()
                .with_width(Size::FitContent)
                .with_height(Size::lpx(100.0))
                .with_layout_direction(Layout::Horizontal)
                .with_gap(Size::lpx(10.0))
                .with_children(row_items),
        );
    }

    let grid_scroll_container = Node::new()
        .with_id(NodeId::new("grid_scroll_container"))
        .with_width(Size::lpx(600.0))
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(10.0)))
        .with_gap(Size::lpx(10.0))
        .with_layout_direction(Layout::Vertical)
        .with_overflow(Overflow::Scroll)
        .with_shape(Shape::rect())
        .with_style(Style {
            fill_color: Some(mocha::MANTLE),
            corner_shape: Some(CornerShape::Round(Size::lpx(12.0))),
            ..Default::default()
        })
        .with_children(grid_items);

    // Root with centered layout
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(40.0)))
        .with_child(
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_layout_direction(Layout::Vertical)
                .with_gap(Size::lpx(20.0))
                .with_children(vec![
                    // Title
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::fraction(0.1))
                        .with_content(Content::Text(TextContent {
                            text: "Scroll Example - Use Mouse Wheel (Shift for horizontal)"
                                .to_string(),
                            font_size: Size::lpx(32.0),
                            color: mocha::TEXT,
                            h_align: HorizontalAlign::Center,
                            v_align: VerticalAlign::Center,
                        })),
                    // Centered vertical scroll container
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::fraction(0.37))
                        // .with_height(Size::px(800.0))
                        .with_layout_direction(Layout::Horizontal)
                        .with_child(Node::new().with_width(Size::Fill)) // Left spacer
                        .with_child(scroll_container)
                        .with_child(Node::new().with_width(Size::Fill)), // Right spacer
                    // 2D grid scroll container
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::fraction(0.33))
                        // .with_height(Size::px(600.0))
                        .with_layout_direction(Layout::Horizontal)
                        .with_child(Node::new().with_width(Size::Fill)) // Left spacer
                        .with_child(grid_scroll_container)
                        .with_child(Node::new().with_width(Size::Fill)), // Right spacer
                    // Horizontal scroll container
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::fraction(0.2))
                        // .with_height(Size::px(400.0))
                        .with_layout_direction(Layout::Horizontal)
                        .with_child(Node::new().with_width(Size::Fill)) // Left spacer
                        .with_child(horizontal_scroll_container)
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

                // Update input state (for shift tracking, etc.)
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.input.handle_event(&event);
                }
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.resize(physical_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    let frame_start = std::time::Instant::now();

                    // Build UI to dispatch events
                    let ui_build_start = std::time::Instant::now();
                    let mut ui = create_demo_ui(
                        gpu_state.config.width as f32,
                        gpu_state.config.height as f32,
                        &self.item_heights,
                        &self.item_widths,
                    );
                    let ui_build_time = ui_build_start.elapsed();

                    // IMPORTANT: Compute layout before dispatching events so hit testing works
                    let window_rect = astra_gui::Rect::new(
                        [0.0, 0.0],
                        [
                            gpu_state.config.width as f32,
                            gpu_state.config.height as f32,
                        ],
                    );
                    let layout_start = std::time::Instant::now();
                    ui.compute_layout(window_rect);
                    let layout_time = layout_start.elapsed();

                    // Restore scroll state from persistent storage (before animations)
                    // This needs to happen before dispatch so the initial state is correct
                    gpu_state.event_dispatcher.restore_scroll_state(&mut ui);

                    // Dispatch events (scroll events are automatically processed internally)
                    let (_events, interaction_states) = gpu_state
                        .event_dispatcher
                        .dispatch(&gpu_state.input, &mut ui);

                    // Update smooth scroll animations
                    let now = std::time::Instant::now();
                    let dt = self
                        .last_frame_time
                        .map(|t| (now - t).as_secs_f32())
                        .unwrap_or(0.016); // Default to ~60fps
                    self.last_frame_time = Some(now);

                    ui.update_all_scroll_animations(dt);

                    // Save updated scroll state back to persistent storage
                    gpu_state.event_dispatcher.sync_scroll_state(&ui);

                    // Apply interactive styles
                    gpu_state.interactive_state_manager.begin_frame();
                    gpu_state
                        .interactive_state_manager
                        .apply_styles(&mut ui, &interaction_states);

                    // Generate output
                    let output_start = std::time::Instant::now();
                    let ui_output = FullOutput::from_node_with_debug(
                        ui,
                        (
                            gpu_state.config.width as f32,
                            gpu_state.config.height as f32,
                        ),
                        if self.debug_options.is_enabled() {
                            Some(self.debug_options)
                        } else {
                            None
                        },
                    );
                    let output_time = output_start.elapsed();

                    // Clear input for next frame
                    gpu_state.input.begin_frame();

                    let (gpu_work_time, present_time) = match gpu_state.render(&ui_output) {
                        Ok(times) => times,
                        Err(wgpu::SurfaceError::Lost) => {
                            if let Some(window) = &self.window {
                                gpu_state.resize(window.inner_size())
                            }
                            (std::time::Duration::ZERO, std::time::Duration::ZERO)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            event_loop.exit();
                            (std::time::Duration::ZERO, std::time::Duration::ZERO)
                        }
                        Err(e) => {
                            eprintln!("Render error: {:?}", e);
                            (std::time::Duration::ZERO, std::time::Duration::ZERO)
                        }
                    };

                    let frame_time = frame_start.elapsed();

                    // Print timing every 60 frames
                    if self.frame_count % 60 == 0 {
                        println!("Frame time: {:.2}ms (UI build: {:.2}ms, Layout: {:.2}ms, Output: {:.2}ms, GPU work: {:.2}ms, Present/VSync: {:.2}ms)",
                            frame_time.as_secs_f64() * 1000.0,
                            ui_build_time.as_secs_f64() * 1000.0,
                            layout_time.as_secs_f64() * 1000.0,
                            output_time.as_secs_f64() * 1000.0,
                            gpu_work_time.as_secs_f64() * 1000.0,
                            present_time.as_secs_f64() * 1000.0,
                        );
                    }
                    self.frame_count += 1;
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

    // STRESS TEST: Generate many more items
    let item_heights: Vec<f32> = (0..200) // 30 -> 200
        .map(|_| rand::random::<f32>() * 100.0 + 50.0)
        .collect();

    // STRESS TEST: Generate many more items
    let item_widths: Vec<f32> = (0..200) // 30 -> 200
        .map(|_| rand::random::<f32>() * 150.0 + 100.0)
        .collect();

    let mut app = App {
        window: None,
        gpu_state: None,
        debug_options: DebugOptions::none(),
        last_frame_time: None,
        frame_count: 0,
        item_heights,
        item_widths,
    };

    println!("{}", DEBUG_HELP_TEXT);
    println!();
    println!("Use mouse wheel to scroll the containers");
    println!("Hold Shift while scrolling over the grid to scroll horizontally");

    event_loop.run_app(&mut app).unwrap();
}
