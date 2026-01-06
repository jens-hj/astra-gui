use astra_gui::{DebugOptions, Node, UiContext};
use astra_gui_text::Engine as TextEngine;
use winit::window::Window;

/// Core trait that all examples must implement.
/// Only requires UI building logic - all boilerplate is handled automatically.
pub trait ExampleApp: Sized {
    /// Create a new instance
    fn new() -> Self;

    /// Optional: Provide a text engine for text measurement
    /// Override this to return your text engine if your UI uses text
    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        None
    }

    /// Build the UI tree for this frame
    ///
    /// The UiContext provides:
    /// - Event checking: `ctx.was_clicked("id")`, `ctx.is_hovered("id")`
    /// - Focus management: `ctx.is_focused("id")`, `ctx.set_focus(Some("id"))`
    /// - Widget memory: `ctx.memory()` for internal state
    /// - ID generation: `ctx.generate_id("label")`
    /// - Text measurement: `ctx.measure_text(...)`
    fn build_ui(&mut self, ctx: &mut UiContext, width: f32, height: f32) -> Node;

    /// Optional: Window title
    fn window_title() -> &'static str {
        "Astra GUI Example"
    }

    /// Optional: Window size
    fn window_size() -> (u32, u32) {
        (800, 600)
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
    fn handle_escape(&mut self, ctx: &UiContext) -> bool {
        // Default: prevent exit if something is focused
        ctx.focused_widget().is_some()
    }
}
