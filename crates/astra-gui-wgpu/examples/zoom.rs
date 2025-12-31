//! Demonstrates zoom functionality - both browser-style zoom and pan.
//!
//! Controls:
//! - Mouse wheel: Browser-style zoom (scales everything with layout reflow)
//! - Arrow keys: Pan camera
//! - R: Reset zoom and pan
//! - M/P/B/C/G/D: Debug visualizations (Margins/Padding/Borders/Content/Gaps/All)
//! - S: Toggle render mode (SDF/Mesh)
//! - ESC: Exit

use astra_gui::{
    catppuccin::mocha, CornerShape, DebugOptions, FullOutput, HorizontalAlign, Layout, Node, Shape,
    Size, Spacing, Stroke, TextContent, VerticalAlign,
};
use astra_gui_wgpu::{RenderMode, Renderer};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const DEBUG_HELP_TEXT: &str =
    "M:Margins | P:Padding | B:Borders | C:Content | G:Gaps | D:All | S:RenderMode";

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
    gpu_state: Option<GpuState>,
    zoom_level: f32, // 1.0 = 100%, 2.0 = 200%, etc.
    pan_offset: (f32, f32),
    debug_options: DebugOptions,
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    renderer: Renderer,
}

impl App {
    fn new() -> Self {
        Self {
            gpu_state: None,
            zoom_level: 1.0,
            pan_offset: (0.0, 0.0),
            debug_options: DebugOptions::none(),
        }
    }

    fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match *key_code {
                winit::keyboard::KeyCode::KeyR => {
                    self.zoom_level = 1.0;
                    self.pan_offset = (0.0, 0.0);
                    println!("Reset zoom and pan");
                    true
                }
                winit::keyboard::KeyCode::ArrowUp => {
                    self.pan_offset.1 -= 20.0;
                    println!("Pan: ({:.1}, {:.1})", self.pan_offset.0, self.pan_offset.1);
                    true
                }
                winit::keyboard::KeyCode::ArrowDown => {
                    self.pan_offset.1 += 20.0;
                    println!("Pan: ({:.1}, {:.1})", self.pan_offset.0, self.pan_offset.1);
                    true
                }
                winit::keyboard::KeyCode::ArrowLeft => {
                    self.pan_offset.0 -= 20.0;
                    println!("Pan: ({:.1}, {:.1})", self.pan_offset.0, self.pan_offset.1);
                    true
                }
                winit::keyboard::KeyCode::ArrowRight => {
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

    fn render(&mut self) {
        // Get window size first
        let (actual_width, actual_height) = if let Some(gpu_state) = &self.gpu_state {
            (
                gpu_state.config.width as f32,
                gpu_state.config.height as f32,
            )
        } else {
            return;
        };

        // Build UI with zoom_level set on the root node
        // Browser-style zoom: zoom_level converts logical pixels to physical pixels
        // At 1.0x zoom, 10 logical pixels = 10 physical pixels
        // At 2.0x zoom, 10 logical pixels = 20 physical pixels (everything twice as big)
        let ui = self
            .build_ui(actual_width, actual_height)
            .with_zoom(self.zoom_level);

        let output_data = FullOutput::from_node_with_debug_and_scale_factor(
            ui,
            (actual_width, actual_height),
            Some(self.debug_options),
            1.0, // Default scale_factor (will be overridden by root's zoom_level)
        );

        let Some(gpu_state) = &mut self.gpu_state else {
            return;
        };

        let output = gpu_state.surface.get_current_texture().unwrap();
        let view = output
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
            multiview_mask: None,
        });

        // Render with zoom scale applied
        gpu_state.renderer.render(
            &gpu_state.device,
            &gpu_state.queue,
            &mut encoder,
            &view,
            actual_width,
            actual_height,
            &output_data,
        );

        gpu_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn build_ui(&self, _window_width: f32, _window_height: f32) -> Node {
        // Debug info panel (not affected by content zoom, positioned absolutely)
        let debug_panel = Node::new()
            .with_width(Size::lpx(115.0))
            .with_height(Size::lpx(40.0))
            .with_shape(Shape::rect())
            .with_style(astra_gui::Style {
                fill_color: Some(mocha::SURFACE0),
                stroke: Some(Stroke::new(Size::lpx(2.0), mocha::OVERLAY0)),
                corner_shape: Some(CornerShape::Round(8.0)),
                ..Default::default()
            })
            .with_padding(Spacing::all(Size::lpx(15.0)))
            .with_content(astra_gui::Content::Text(TextContent {
                text: format!(
                    "Zoom: {:.0}%\nPan: ({:.0}, {:.0})\n\nWheel: Zoom\nArrows: Pan\nR: Reset",
                    self.zoom_level * 100.0,
                    self.pan_offset.0,
                    self.pan_offset.1
                ),
                font_size: Size::lpx(14.0),
                color: mocha::TEXT,
                h_align: HorizontalAlign::Left,
                v_align: VerticalAlign::Center,
            }));

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
                        .with_shape(Shape::rect())
                        .with_style(astra_gui::Style {
                            fill_color: Some(mocha::CRUST),
                            stroke: Some(Stroke::new(Size::lpx(2.0), color)),
                            corner_shape: Some(CornerShape::Round(12.0)),
                            ..Default::default()
                        })
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_content(astra_gui::Content::Text(TextContent {
                            text: format!("{},{}", row + 1, col + 1),
                            font_size: Size::lpx(24.0),
                            color: mocha::TEXT,
                            h_align: HorizontalAlign::Center,
                            v_align: VerticalAlign::Center,
                        })),
                );
            }

            grid_rows.push(
                Node::new()
                    .with_layout_direction(Layout::Horizontal)
                    .with_height(Size::Fill)
                    .with_gap(Size::ppx(20.0))
                    .with_children(row_children),
            );
        }

        let content_grid = Node::new()
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::ppx(20.0))
            .with_padding(Spacing::all(Size::ppx(50.0)))
            .with_children(grid_rows);

        // Root: Stack layout with debug panel on top
        Node::new()
            .with_layout_direction(Layout::Stack)
            .with_padding(Spacing::top(Size::lpx(1.0)) + Spacing::bottom(Size::lpx(1.0)))
            .with_children(vec![content_grid, debug_panel])
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu_state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Zoom Example - Mouse Wheel: Zoom | Arrows: Pan | R: Reset")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 700));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, config.format);

        self.gpu_state = Some(GpuState {
            surface,
            device,
            queue,
            config,
            window,
            renderer,
        });

        println!("\nZoom Example");
        println!("Controls:");
        println!("  Mouse Wheel - Browser-style zoom");
        println!("  Arrow Keys  - Pan camera");
        println!("  R           - Reset zoom and pan");
        println!("  {}", DEBUG_HELP_TEXT);
        println!("  ESC         - Exit\n");
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
                        state: ElementState::Pressed,
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.config.width = physical_size.width;
                    gpu_state.config.height = physical_size.height;
                    gpu_state
                        .surface
                        .configure(&gpu_state.device, &gpu_state.config);
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {
                // Handle debug keybinds first
                let renderer = self.gpu_state.as_mut().map(|s| &mut s.renderer);
                let handled = handle_debug_keybinds(&event, &mut self.debug_options, renderer);

                // Then handle app input if not handled by debug
                if !handled && self.handle_input(&event) {
                    if let Some(gpu_state) = &self.gpu_state {
                        gpu_state.window.request_redraw();
                    }
                } else if handled {
                    // Redraw if debug state changed
                    if let Some(gpu_state) = &self.gpu_state {
                        gpu_state.window.request_redraw();
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Don't request redraw here - only redraw when state actually changes
        // This prevents infinite redraw loop that can overwhelm the GPU
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
