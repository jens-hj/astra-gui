use astra_gui::{DebugOptions, Node};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::TargetedEvent;
use winit::window::Window;

use super::InteractiveState;

/// Core trait that all examples must implement.
/// Only requires UI building logic - all boilerplate is handled automatically.
pub trait ExampleApp: Sized {
    /// Create a new instance
    fn new() -> Self;

    /// Build the UI tree for this frame
    /// This is the ONLY method most examples need to implement
    fn build_ui(&mut self, width: f32, height: f32) -> Node;

    /// Optional: Window title
    fn window_title() -> &'static str {
        "Astra GUI Example"
    }

    /// Optional: Window size
    fn window_size() -> (u32, u32) {
        (800, 600)
    }

    /// Optional: Provide text measurer for layout computation
    /// Only needed if UI contains Text nodes or FitContent sizing
    fn text_measurer(&mut self) -> Option<&mut TextEngine> {
        None
    }

    /// Optional: Access to interactive state for event handling
    /// Return Some if using interactive components
    fn interactive_state(&mut self) -> Option<&mut InteractiveState> {
        None
    }

    /// Optional: Access to debug options
    /// Return Some to enable debug visualization controls
    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        None
    }

    /// Optional: Custom zoom level
    /// Override for examples that need display PPI detection or custom zoom
    fn zoom_level(&self) -> f32 {
        1.0
    }

    /// Optional: Called after window creation
    /// Useful for examples that need to detect display PPI
    fn on_window_created(&mut self, _window: &Window) {}

    /// Optional: Custom ESC key handling
    /// Return true to prevent default exit behavior
    /// Useful for examples with focus management
    fn handle_escape(&mut self) -> bool {
        // Default: prevent exit if something is focused
        if let Some(interactive) = self.interactive_state() {
            interactive.event_dispatcher.focused_node().is_some()
        } else {
            false
        }
    }

    /// Optional: Handle interactive events
    /// Return true if the UI state changed and needs redraw
    fn handle_events(&mut self, _events: &[TargetedEvent]) -> bool {
        false
    }
}
