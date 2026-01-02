use super::debug_controls::{handle_debug_keybinds, DEBUG_HELP_TEXT};
use super::example_app::ExampleApp;
use super::gpu_state::GpuState;
use astra_gui::{FullOutput, Rect};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Generic application wrapper that handles all boilerplate
/// This implements ApplicationHandler for any type that implements ExampleApp
pub struct AppRunner<T: ExampleApp> {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    app: T,
}

impl<T: ExampleApp> AppRunner<T> {
    pub fn new(app: T) -> Self {
        Self {
            window: None,
            gpu_state: None,
            app,
        }
    }

    fn render(&mut self) {
        // Update app state
        if let Some(interactive) = self.app.interactive_state() {
            let _delta_time = interactive.delta_time();
            interactive.begin_frame();
        }

        // Get window size
        let size = match &self.window {
            Some(window) => window.inner_size(),
            None => return,
        };

        // Build UI
        let mut ui = self.app.build_ui(size.width as f32, size.height as f32);

        // Apply zoom if needed
        let zoom = self.app.zoom_level();
        if zoom != 1.0 {
            ui = ui.with_zoom(zoom);
        }

        // Compute layout
        let window_rect = Rect::from_min_size([0.0, 0.0], [size.width as f32, size.height as f32]);

        if let Some(text_measurer) = self.app.text_measurer() {
            ui.compute_layout_with_measurer(window_rect, text_measurer);
        } else {
            ui.compute_layout(window_rect);
        }

        // Handle interactive events if needed
        let events = if let Some(interactive) = self.app.interactive_state() {
            let (events, interaction_states) = interactive
                .event_dispatcher
                .dispatch(&interactive.input_state, &mut ui);

            interactive
                .state_manager
                .apply_styles(&mut ui, &interaction_states);

            events
        } else {
            Vec::new()
        };

        // Let app handle events (after releasing the borrow on interactive_state)
        // If state changed, rebuild the UI with the new state
        let state_changed = if !events.is_empty() {
            self.app.handle_events(&events)
        } else {
            false
        };

        // Rebuild UI if state changed
        if state_changed {
            ui = self.app.build_ui(size.width as f32, size.height as f32);

            // Reapply zoom
            let zoom = self.app.zoom_level();
            if zoom != 1.0 {
                ui = ui.with_zoom(zoom);
            }

            // Recompute layout with new UI
            if let Some(text_measurer) = self.app.text_measurer() {
                ui.compute_layout_with_measurer(window_rect, text_measurer);
            } else {
                ui.compute_layout(window_rect);
            }

            // Reapply interactive styles (dispatch again on new UI)
            if let Some(interactive) = self.app.interactive_state() {
                let (_, interaction_states) = interactive
                    .event_dispatcher
                    .dispatch(&interactive.input_state, &mut ui);

                interactive
                    .state_manager
                    .apply_styles(&mut ui, &interaction_states);
            }
        }

        // Generate output
        let debug_options = self.app.debug_options_mut().copied();
        let output = if let Some(text_measurer) = self.app.text_measurer() {
            FullOutput::from_node_with_debug_and_measurer(
                ui,
                (size.width as f32, size.height as f32),
                debug_options,
                Some(text_measurer),
            )
        } else {
            FullOutput::from_node_with_debug(
                ui,
                (size.width as f32, size.height as f32),
                debug_options,
            )
        };

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
