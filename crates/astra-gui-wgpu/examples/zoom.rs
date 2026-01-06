//! Demonstrates zoom functionality - both browser-style zoom and pan.
//!
//! Controls:
//! - Mouse wheel: Browser-style zoom (scales everything with layout reflow)
//! - Arrow keys: Pan camera
//! - R: Reset zoom and pan
//! - M/P/B/C/G/D: Debug visualizations (Margins/Padding/Borders/Content/Gaps/All)
//! - S: Toggle render mode (SDF/Mesh)
//! - ESC: Exit

mod shared;

use astra_gui::{
    catppuccin::mocha, CornerShape, DebugOptions, FullOutput, HorizontalAlign, Layout, Node, Rect,
    Size, Spacing, Stroke, TextContent, UiContext, VerticalAlign, ZIndex,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::WinitInputExt;
use shared::debug_controls::DEBUG_HELP_TEXT;
use shared::gpu_state::GpuState;
use std::sync::Arc;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct Zoom {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    zoom_level: f32, // 1.0 = 100%, 2.0 = 200%, etc.
    pan_offset: (f32, f32),
}

impl Zoom {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            zoom_level: 1.0,
            pan_offset: (0.0, 0.0),
        }
    }

    fn window_title() -> &'static str {
        "Zoom Example - Mouse Wheel: Zoom | Arrows: Pan | R: Reset"
    }

    fn window_size() -> (u32, u32) {
        (800, 700)
    }

    fn build_ui(&mut self, _ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        // Debug info panel (not affected by content zoom, positioned absolutely)
        let debug_panel = Node::new()
            .with_style(astra_gui::Style {
                fill_color: Some(mocha::SURFACE0),
                stroke: Some(Stroke::new(Size::lpx(2.0), mocha::OVERLAY0)),
                corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                ..Default::default()
            })
            .with_padding(Spacing::all(Size::lpx(15.0)))
            .with_z_index(ZIndex::OVERLAY) // Ensure panel renders on top of grid
            .with_content(astra_gui::Content::Text(
                TextContent::new(format!(
                    "Zoom: {:.0}%\nPan: ({:.0}, {:.0})\n\nWheel: Zoom\nArrows: Pan\nR: Reset",
                    self.zoom_level * 100.0,
                    self.pan_offset.0,
                    self.pan_offset.1
                ))
                .with_font_size(Size::lpx(14.0))
                .with_color(mocha::TEXT)
                .with_h_align(HorizontalAlign::Left)
                .with_v_align(VerticalAlign::Center),
            ));

        // Create a colorful grid of boxes to demonstrate zoom
        let mut grid_rows = Vec::new();

        for row in 0..3 {
            let mut row_children = Vec::new();
            for col in 0..3 {
                let colors = [
                    mocha::RED,
                    mocha::GREEN,
                    mocha::BLUE,
                    mocha::YELLOW,
                    mocha::PEACH,
                    mocha::MAUVE,
                    mocha::SKY,
                    mocha::TEAL,
                    mocha::PINK,
                ];
                let color = colors[row * 3 + col];

                row_children.push(
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::Fill)
                        .with_style(astra_gui::Style {
                            fill_color: Some(mocha::CRUST),
                            stroke: Some(Stroke::new(Size::lpx(2.0), color)),
                            corner_shape: Some(CornerShape::Round(Size::ppx(12.0))),
                            ..Default::default()
                        })
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_content(astra_gui::Content::Text(
                            TextContent::new(format!("{},{}", row + 1, col + 1))
                                .with_font_size(Size::lpx(24.0))
                                .with_color(mocha::TEXT)
                                .with_h_align(HorizontalAlign::Center)
                                .with_v_align(VerticalAlign::Center),
                        )),
                );
            }

            grid_rows.push(
                Node::new()
                    .with_layout_direction(Layout::Horizontal)
                    .with_height(Size::Fill)
                    .with_width(Size::Fill)
                    .with_gap(Size::ppx(20.0))
                    .with_children(row_children),
            );
        }

        let content_grid = Node::new()
            .with_height(Size::Fill)
            .with_width(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::ppx(20.0))
            .with_padding(Spacing::all(Size::ppx(50.0)))
            .with_children(grid_rows);

        // Root: Stack layout with debug panel on top
        Node::new()
            .with_height(Size::Fill)
            .with_width(Size::Fill)
            .with_layout_direction(Layout::Stack)
            .with_padding(Spacing::top(Size::lpx(1.0)) + Spacing::bottom(Size::lpx(1.0)))
            .with_children(vec![content_grid, debug_panel])
    }

    /// Custom input handling for zoom and pan controls
    pub fn handle_custom_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if key_event.state == ElementState::Pressed => match key_event.physical_key {
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyR) => {
                    self.zoom_level = 1.0;
                    self.pan_offset = (0.0, 0.0);
                    println!("Reset zoom and pan");
                    true
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp) => {
                    self.pan_offset.1 -= 20.0;
                    println!("Pan: ({:.1}, {:.1})", self.pan_offset.0, self.pan_offset.1);
                    true
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown) => {
                    self.pan_offset.1 += 20.0;
                    println!("Pan: ({:.1}, {:.1})", self.pan_offset.0, self.pan_offset.1);
                    true
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft) => {
                    self.pan_offset.0 -= 20.0;
                    println!("Pan: ({:.1}, {:.1})", self.pan_offset.0, self.pan_offset.1);
                    true
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight) => {
                    self.pan_offset.0 += 20.0;
                    println!("Pan: ({:.1}, {:.1})", self.pan_offset.0, self.pan_offset.1);
                    true
                }
                _ => false,
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 20.0) as f32,
                };

                // Browser-style zoom with mouse wheel
                self.zoom_level *= 1.0 + scroll_delta * 0.1;
                self.zoom_level = self.zoom_level.clamp(0.25, 10.0);
                println!("Zoom: {:.0}%", self.zoom_level * 100.0);
                true
            }
            _ => false,
        }
    }
}

/// Custom runner for zoom example that handles zoom/pan input
struct ZoomRunner {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    app: Zoom,
    ctx: UiContext,
}

impl ApplicationHandler for ZoomRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let (width, height) = Zoom::window_size();
        let window_attributes = Window::default_attributes()
            .with_title(Zoom::window_title())
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.window = Some(window.clone());
        self.gpu_state = Some(pollster::block_on(GpuState::new(window)));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Handle input for UiContext
        self.ctx.input_mut().handle_winit_event(&event);

        // Handle custom zoom/pan input BEFORE other processing
        let custom_handled = self.app.handle_custom_input(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

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

            WindowEvent::Resized(physical_size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.resize(physical_size);
                }
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }

            _ => {
                // Debug keybinds
                if !custom_handled {
                    let debug_opts = &mut self.app.debug_options;
                    let _handled =
                        shared::debug_controls::handle_debug_keybinds(&event, debug_opts);
                }
            }
        }

        // Always request redraw
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ZoomRunner {
    fn render(&mut self) {
        // Get window size
        let size = match &self.window {
            Some(window) => window.inner_size(),
            None => return,
        };

        // Begin frame
        let input = self.ctx.input().clone();
        self.ctx.begin_frame(&input);

        // Build UI
        let mut ui = self
            .app
            .build_ui(&mut self.ctx, size.width as f32, size.height as f32);

        // Apply zoom
        let zoom = self.app.zoom_level;
        if zoom != 1.0 {
            ui = ui.with_zoom(zoom);
        }

        // Apply pan offset using translation
        if self.app.pan_offset != (0.0, 0.0) {
            ui = ui.with_pan_offset(astra_gui::Translation::new(
                Size::Logical(self.app.pan_offset.0),
                Size::Logical(self.app.pan_offset.1),
            ));
        }

        // Inject dimension overrides
        self.ctx.inject_dimension_overrides(&mut ui);

        // Compute layout
        let window_rect = Rect::from_min_size([0.0, 0.0], [size.width as f32, size.height as f32]);

        ui.compute_layout_with_measurer(window_rect, &mut self.app.text_engine);

        // End frame - dispatches events and updates transitions
        self.ctx.end_frame(&mut ui);

        // Generate output
        let debug_options = Some(self.app.debug_options);
        let output = FullOutput::from_laid_out_node(
            ui,
            (size.width as f32, size.height as f32),
            debug_options,
        );

        // Render
        let Some(gpu_state) = &mut self.gpu_state else {
            return;
        };

        match gpu_state.render(&output) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => {
                if let Some(window) = &self.window {
                    gpu_state.resize(window.inner_size())
                }
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("Out of memory");
                std::process::exit(1);
            }
            Err(e) => eprintln!("Render error: {:?}", e),
        }

        // Clear input state for next frame
        self.ctx.input_mut().begin_frame();
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let app = Zoom::new();
    let mut runner = ZoomRunner {
        window: None,
        gpu_state: None,
        app,
        ctx: UiContext::new(),
    };

    println!("\nZoom Example");
    println!("Controls:");
    println!("  Mouse Wheel - Browser-style zoom");
    println!("  Arrow Keys  - Pan camera");
    println!("  R           - Reset zoom and pan");
    println!("  {}", DEBUG_HELP_TEXT);
    println!("  ESC         - Exit\n");

    event_loop.run_app(&mut runner).unwrap();
}
