use super::debug_controls::{handle_debug_keybinds, DEBUG_HELP_TEXT};
use super::example_app::ExampleApp;
use super::gpu_state::GpuState;
use astra_gui::{FullOutput, Rect};
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Frame timing statistics
#[derive(Debug, Clone)]
pub struct FrameStats {
    pub total_frame_time_ms: f32,
    pub build_ui_ms: f32,
    pub layout_ms: f32,
    pub event_dispatch_ms: f32,
    pub output_generation_ms: f32,
    pub render_ms: f32,
    pub fps: f32,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            total_frame_time_ms: 0.0,
            build_ui_ms: 0.0,
            layout_ms: 0.0,
            event_dispatch_ms: 0.0,
            output_generation_ms: 0.0,
            render_ms: 0.0,
            fps: 0.0,
        }
    }
}

/// Generic application wrapper that handles all boilerplate
/// This implements ApplicationHandler for any type that implements ExampleApp
pub struct AppRunner<T: ExampleApp> {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    app: T,
    last_frame_time: Instant,
    frame_stats: FrameStats,
    enable_profiling: bool,
}

impl<T: ExampleApp> AppRunner<T> {
    pub fn new(app: T) -> Self {
        Self {
            window: None,
            gpu_state: None,
            app,
            last_frame_time: Instant::now(),
            frame_stats: FrameStats::default(),
            enable_profiling: std::env::var("PROFILE").is_ok(),
        }
    }

    pub fn frame_stats(&self) -> &FrameStats {
        &self.frame_stats
    }

    pub fn enable_profiling(&mut self, enable: bool) {
        self.enable_profiling = enable;
    }

    fn render(&mut self) {
        let frame_start = Instant::now();

        // Update app state
        if let Some(interactive) = self.app.interactive_state() {
            let _delta_time = interactive.delta_time();
            interactive.begin_frame_transitions();
        }

        // Get window size
        let size = match &self.window {
            Some(window) => window.inner_size(),
            None => return,
        };

        // Build UI
        let build_start = Instant::now();
        let mut ui = self.app.build_ui(size.width as f32, size.height as f32);
        let build_time = build_start.elapsed();

        // Apply zoom if needed
        let zoom = self.app.zoom_level();
        if zoom != 1.0 {
            ui = ui.with_zoom(zoom);
        }

        // Compute layout (needed for event dispatch hit testing)
        let layout_start = Instant::now();
        let window_rect = Rect::from_min_size([0.0, 0.0], [size.width as f32, size.height as f32]);

        if let Some(text_measurer) = self.app.text_measurer() {
            ui.compute_layout_with_measurer(window_rect, text_measurer);
        } else {
            ui.compute_layout(window_rect);
        }
        let layout_time = layout_start.elapsed();

        // Handle interactive events if needed
        let event_start = Instant::now();
        let events = if let Some(interactive) = self.app.interactive_state() {
            let (events, interaction_states) = interactive
                .event_dispatcher
                .dispatch(&interactive.input_state, &mut ui);

            interactive
                .state_manager
                .apply_styles(&mut ui, &interaction_states);

            // Recompute layout after applying style overrides (for transitions)
            if let Some(text_measurer) = self.app.text_measurer() {
                ui.compute_layout_with_measurer(window_rect, text_measurer);
            } else {
                ui.compute_layout(window_rect);
            }

            events
        } else {
            Vec::new()
        };
        let event_time = event_start.elapsed();

        // Let app handle events (after releasing the borrow on interactive_state)
        self.app.handle_events(&events);

        // Generate output (using from_laid_out_node since we already computed layout)
        let output_start = Instant::now();
        let debug_options = self.app.debug_options_mut().copied();
        let output = FullOutput::from_laid_out_node(
            ui,
            (size.width as f32, size.height as f32),
            debug_options,
        );
        let output_time = output_start.elapsed();

        // Render
        let render_start = Instant::now();
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
        let render_time = render_start.elapsed();

        // Clear input state for next frame
        if let Some(interactive) = self.app.interactive_state() {
            interactive.end_frame();
        }

        // Update frame stats
        let total_frame_time = frame_start.elapsed();
        let frame_time_since_last = self.last_frame_time.elapsed();
        self.last_frame_time = Instant::now();

        self.frame_stats = FrameStats {
            total_frame_time_ms: total_frame_time.as_secs_f32() * 1000.0,
            build_ui_ms: build_time.as_secs_f32() * 1000.0,
            layout_ms: layout_time.as_secs_f32() * 1000.0,
            event_dispatch_ms: event_time.as_secs_f32() * 1000.0,
            output_generation_ms: output_time.as_secs_f32() * 1000.0,
            render_ms: render_time.as_secs_f32() * 1000.0,
            fps: if frame_time_since_last.as_secs_f32() > 0.0 {
                1.0 / frame_time_since_last.as_secs_f32()
            } else {
                0.0
            },
        };

        // Print stats if profiling enabled
        if self.enable_profiling {
            let shape_count = if let Some(gpu) = &self.gpu_state {
                gpu.last_shape_count()
            } else {
                0
            };
            println!(
                "Frame: {:.2}ms ({:.1} FPS) | Build: {:.2}ms | Layout: {:.2}ms | Events: {:.2}ms | Output: {:.2}ms | Render: {:.2}ms | Shapes: {}",
                self.frame_stats.total_frame_time_ms,
                self.frame_stats.fps,
                self.frame_stats.build_ui_ms,
                self.frame_stats.layout_ms,
                self.frame_stats.event_dispatch_ms,
                self.frame_stats.output_generation_ms,
                self.frame_stats.render_ms,
                shape_count,
            );
        }
    }
}

impl<T: ExampleApp> ApplicationHandler for AppRunner<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let (width, height) = T::window_size();
        let window_attributes = Window::default_attributes()
            .with_title(T::window_title())
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        // Let app know window was created (for PPI detection, etc.)
        self.app.on_window_created(&window);

        self.window = Some(window.clone());
        self.gpu_state = Some(pollster::block_on(GpuState::new(window)));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Handle input events for interactive examples
        if let Some(interactive) = self.app.interactive_state() {
            interactive.input_state.handle_event(&event);
        }

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
                // Custom ESC handling
                if !self.app.handle_escape() {
                    event_loop.exit();
                }
            }

            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if matches!(
                key_event.physical_key,
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyF)
            ) && key_event.state == ElementState::Pressed =>
            {
                // Toggle profiling
                self.enable_profiling = !self.enable_profiling;
                println!(
                    "Profiling {}",
                    if self.enable_profiling {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
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
                let renderer = self.gpu_state.as_mut().map(|s| &mut s.renderer);
                let debug_options = self.app.debug_options_mut();
                if let Some(debug_opts) = debug_options {
                    let _handled = handle_debug_keybinds(&event, debug_opts, renderer);
                }
            }
        }

        // Always request redraw for Poll mode
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Convenience function to run an example
pub fn run_example<T: ExampleApp + 'static>() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let app = T::new();
    let mut runner = AppRunner::new(app);

    println!("{}", DEBUG_HELP_TEXT);

    event_loop.run_app(&mut runner).unwrap();
}
