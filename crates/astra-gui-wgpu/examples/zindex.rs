//! Demonstrates z-index layering control.
//!
//! Shows how z-index can override tree order to control which elements render on top.
//! - Red box (z-index: -10) - Forced to bottom despite being last in tree
//! - Green box (z-index: 0 default) - Middle layer, first in tree
//! - Blue box (z-index: 0 default) - Middle layer, second in tree
//! - Yellow box (z-index: 100) - Forced to top despite being first in tree
//!
//! Controls:
//! - Mouse wheel: Zoom
//! - Arrow keys: Pan
//! - R: Reset zoom and pan
//! - M/P/B/C/G/D: Debug visualizations
//! - ESC: Exit

#![allow(unused_imports, unused_variables, dead_code)]

use astra_gui::{
    catppuccin::mocha, CornerShape, DebugOptions, FullOutput, HorizontalAlign, Layout, Node, Size,
    Spacing, Stroke, TextContent, Translation, VerticalAlign, ZIndex,
};
use astra_gui_wgpu::Renderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const DEBUG_HELP_TEXT: &str =
    "M:Margins | P:Padding | B:Borders | C:Content | G:Gaps | T:Text | D:All | S:RenderMode";

fn handle_debug_keybinds(event: &WindowEvent, debug_options: &mut DebugOptions) -> bool {
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
        _ => false,
    }
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    renderer: Renderer,
}

struct App {
    gpu_state: Option<GpuState>,
    zoom_level: f32,
    pan_offset: (f32, f32),
    debug_options: DebugOptions,
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

                self.zoom_level *= 1.0 + scroll_delta * 0.1;
                self.zoom_level = self.zoom_level.clamp(0.25, 10.0);
                println!("Zoom: {:.0}%", self.zoom_level * 100.0);
                true
            }
            _ => false,
        }
    }

    fn render(&mut self) {
        let (actual_width, actual_height) = if let Some(gpu_state) = &self.gpu_state {
            (
                gpu_state.config.width as f32,
                gpu_state.config.height as f32,
            )
        } else {
            return;
        };

        let ui = self
            .build_ui(actual_width, actual_height)
            .with_zoom(self.zoom_level)
            .with_pan_offset(Translation::new(
                Size::Logical(self.pan_offset.0),
                Size::Logical(self.pan_offset.1),
            ));

        let output_data = FullOutput::from_node_with_debug_and_scale_factor(
            ui,
            (actual_width, actual_height),
            Some(self.debug_options),
            1.0,
        );

        let Some(gpu_state) = &mut self.gpu_state else {
            return;
        };

        let output = match gpu_state.surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                let size = gpu_state.window.inner_size();
                if size.width > 0 && size.height > 0 {
                    gpu_state.config.width = size.width;
                    gpu_state.config.height = size.height;
                    gpu_state
                        .surface
                        .configure(&gpu_state.device, &gpu_state.config);
                }
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("Out of memory");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Render error: {:?}", e);
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            gpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

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
        // Create overlapping boxes to demonstrate z-index layering
        // Using Stack layout so boxes overlap at the same position

        // Yellow box: First in tree order, but z-index: 100 forces it to TOP
        let yellow_box = Node::new()
            .with_width(Size::lpx(200.0))
            .with_height(Size::lpx(200.0))
            .with_style(astra_gui::Style {
                fill_color: Some(mocha::CRUST),
                stroke: Some(Stroke::new(Size::lpx(4.0), mocha::YELLOW)),
                corner_shape: Some(CornerShape::Round(Size::lpx(16.0))),
                ..Default::default()
            })
            .with_padding(Spacing::all(Size::lpx(20.0)))
            .with_z_index(ZIndex::OVERLAY) // z-index: 100 - Forced to TOP
            .with_translation(Translation::new(Size::lpx(50.0), Size::lpx(50.0)))
            .with_child(
                Node::new().with_content(astra_gui::Content::Text(
                    TextContent::new(
                        "z-index: 100\nYELLOW\n(OVERLAY)\n\nFirst in tree,\nFORCED TO TOP"
                            .to_string(),
                    )
                    .with_font_size(Size::lpx(16.0))
                    .with_color(mocha::YELLOW)
                    .with_h_align(HorizontalAlign::Center)
                    .with_v_align(VerticalAlign::Center),
                )),
            );

        // Green box: Second in tree order, z-index: 0 (default)
        let green_box = Node::new()
            .with_width(Size::lpx(200.0))
            .with_height(Size::lpx(200.0))
            .with_style(astra_gui::Style {
                fill_color: Some(mocha::CRUST),
                stroke: Some(Stroke::new(Size::lpx(4.0), mocha::GREEN)),
                corner_shape: Some(CornerShape::Round(Size::lpx(16.0))),
                ..Default::default()
            })
            .with_padding(Spacing::all(Size::lpx(20.0)))
            // No z-index set, defaults to 0
            .with_translation(Translation::new(Size::lpx(70.0), Size::lpx(180.0)))
            .with_child(
                Node::new().with_content(astra_gui::Content::Text(
                    TextContent::new(
                        "z-index: 0\nGREEN\n(DEFAULT)\n\nSecond in tree,\nmiddle layer".to_string(),
                    )
                    .with_font_size(Size::lpx(16.0))
                    .with_color(mocha::GREEN)
                    .with_h_align(HorizontalAlign::Center)
                    .with_v_align(VerticalAlign::Center),
                )),
            );

        // Blue box: Third in tree order, z-index: 0 (default)
        // Should render on top of green (later in tree) but below yellow (higher z-index)
        let blue_box = Node::new()
            .with_width(Size::lpx(200.0))
            .with_height(Size::lpx(200.0))
            .with_style(astra_gui::Style {
                fill_color: Some(mocha::CRUST),
                stroke: Some(Stroke::new(Size::lpx(4.0), mocha::BLUE)),
                corner_shape: Some(CornerShape::Round(Size::lpx(16.0))),
                ..Default::default()
            })
            .with_padding(Spacing::all(Size::lpx(20.0)))
            // No z-index set, defaults to 0
            .with_translation(Translation::new(Size::lpx(210.0), Size::lpx(210.0)))
            .with_child(
                Node::new().with_content(astra_gui::Content::Text(
                    TextContent::new(
                        "z-index: 0\nBLUE\n(DEFAULT)\n\nThird in tree,\nmiddle layer".to_string(),
                    )
                    .with_font_size(Size::lpx(16.0))
                    .with_color(mocha::BLUE)
                    .with_h_align(HorizontalAlign::Center)
                    .with_v_align(VerticalAlign::Center),
                )),
            );

        // Red box: Fourth/last in tree order, but z-index: -10 forces it to BOTTOM
        let red_box = Node::new()
            .with_width(Size::lpx(200.0))
            .with_height(Size::lpx(200.0))
            .with_style(astra_gui::Style {
                fill_color: Some(mocha::CRUST),
                stroke: Some(Stroke::new(Size::lpx(4.0), mocha::RED)),
                corner_shape: Some(CornerShape::Round(Size::lpx(16.0))),
                ..Default::default()
            })
            .with_padding(Spacing::all(Size::lpx(20.0)))
            .with_z_index(ZIndex::BACKGROUND) // z-index: -100 - Forced to BOTTOM
            .with_translation(Translation::new(Size::lpx(180.0), Size::lpx(70.0)))
            .with_child(
                Node::new().with_content(astra_gui::Content::Text(
                    TextContent::new(
                        "z-index: -100\nRED\n(BACKGROUND)\n\nLast in tree,\nFORCED TO BOTTOM"
                            .to_string(),
                    )
                    .with_font_size(Size::lpx(16.0))
                    .with_color(mocha::RED)
                    .with_h_align(HorizontalAlign::Center)
                    .with_v_align(VerticalAlign::Center),
                )),
            );

        // Stack layout to make boxes overlap
        Node::new()
            .with_layout_direction(Layout::Stack)
            .with_children(vec![
                yellow_box, // First in tree, but z-index: 100
                green_box,  // Second in tree, z-index: 0
                blue_box,   // Third in tree, z-index: 0
                red_box,    // Last in tree, but z-index: -100
            ])
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu_state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Z-Index Example - Layering Control")
            .with_inner_size(winit::dpi::PhysicalSize::new(900, 600));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let backends = std::env::var("WGPU_BACKEND")
            .ok()
            .map(|s| match s.to_lowercase().as_str() {
                "vulkan" => wgpu::Backends::VULKAN,
                "metal" => wgpu::Backends::METAL,
                "dx12" => wgpu::Backends::DX12,
                "gl" => wgpu::Backends::GL,
                "webgpu" => wgpu::Backends::BROWSER_WEBGPU,
                _ => wgpu::Backends::all(),
            })
            .unwrap_or(wgpu::Backends::all());

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
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

        println!("\nZ-Index Layering Example");
        println!("========================");
        println!("Demonstrates how z-index overrides tree order for rendering.");
        println!("\nExpected order (bottom to top):");
        println!("  1. RED box (z: -100 BACKGROUND) - last in tree, forced to bottom");
        println!("  2. GREEN box (z: 0 DEFAULT) - second in tree");
        println!("  3. BLUE box (z: 0 DEFAULT) - third in tree");
        println!("  4. YELLOW box (z: 100 OVERLAY) - first in tree, forced to top");
        println!("\nControls:");
        println!("  Mouse Wheel - Zoom");
        println!("  Arrow Keys  - Pan");
        println!("  R           - Reset");
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
                let handled = handle_debug_keybinds(&event, &mut self.debug_options);

                if !handled && self.handle_input(&event) {
                    if let Some(gpu_state) = &self.gpu_state {
                        gpu_state.window.request_redraw();
                    }
                } else if handled {
                    if let Some(gpu_state) = &self.gpu_state {
                        gpu_state.window.request_redraw();
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw on every frame for smooth interaction
        if let Some(gpu_state) = &self.gpu_state {
            gpu_state.window.request_redraw();
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
