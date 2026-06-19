//! UI Context for managing the immediate mode UI lifecycle
//!
//! The `UiContext` is the central coordinator for the UI system. It holds all
//! the "plumbing" that components need to function:
//! - Input state (mouse, keyboard)
//! - Event dispatcher (generates interaction events from input)
//! - Interactive state manager (handles style transitions)
//! - Widget memory (stores internal widget state like text buffers, cursors)
//! - Content measurer (for text measurement)
//! - ID stack (for generating unique widget IDs)
//!
//! This design is inspired by egui's `Context` and enables a clean API where
//! users only need to provide the data that matters (values, ranges, etc.)
//! while the context handles all the internal complexity.

use crate::{
    ContentMeasurer, EventDispatcher, InputState, InteractionEvent, InteractionState,
    InteractiveStateManager, IntrinsicSize, MeasureTextRequest, MouseButton, Node, NodeId,
    TargetedEvent, WidgetMemory,
};
use std::collections::HashMap;

/// The main UI context that coordinates all UI operations
///
/// This is passed to components when building the UI tree. It provides:
/// - Access to input state and events from the previous frame
/// - Widget state storage (text buffers, cursors, etc.)
/// - Event checking methods (was_clicked, is_hovered, etc.)
/// - ID generation for widgets
///
/// # Example
///
/// ```ignore
/// // In your app's update loop:
/// // Input accumulates via ctx.input_mut().handle_winit_event() between frames
/// ctx.begin_frame();
///
/// // Build UI - components check for events and fire callbacks internally
/// let root = Button::new("Click me")
///     .on_click(|| println!("Clicked!"))
///     .node(&mut ctx);
///
/// // Compute layout and dispatch events for next frame
/// ctx.end_frame(&mut root);
/// ```
pub struct UiContext {
    /// Current input state
    input: InputState,

    /// Events from the last frame (available during UI building)
    events: Vec<TargetedEvent>,

    /// Interaction states for nodes (for style transitions)
    interaction_states: HashMap<NodeId, InteractionState>,

    /// Event dispatcher for generating events from input
    dispatcher: EventDispatcher,

    /// State manager for style transitions
    state_manager: InteractiveStateManager,

    /// Widget memory for storing internal state
    memory: WidgetMemory,

    /// Content measurer for text measurement
    measurer: Option<Box<dyn ContentMeasurer>>,

    /// ID stack for hierarchical ID generation
    id_stack: Vec<String>,

    /// Counter for generating unique IDs within a scope
    id_counter: usize,

    /// Scale factor for the display
    scale_factor: f32,

    /// Timestamp of the previous `end_frame`, used to derive the per-frame
    /// delta time that drives smooth scroll animations.
    last_frame_time: Option<std::time::Instant>,
}

impl UiContext {
    /// Create a new UI context
    pub fn new() -> Self {
        Self {
            input: InputState::new(),
            events: Vec::new(),
            interaction_states: HashMap::new(),
            dispatcher: EventDispatcher::new(),
            state_manager: InteractiveStateManager::new(),
            memory: WidgetMemory::new(),
            measurer: None,
            id_stack: Vec::new(),
            id_counter: 0,
            scale_factor: 1.0,
            last_frame_time: None,
        }
    }

    /// Create a new UI context with a content measurer
    pub fn with_measurer(measurer: impl ContentMeasurer + 'static) -> Self {
        Self {
            measurer: Some(Box::new(measurer)),
            ..Self::new()
        }
    }

    /// Set the content measurer
    pub fn set_measurer(&mut self, measurer: impl ContentMeasurer + 'static) {
        self.measurer = Some(Box::new(measurer));
    }

    /// Set the scale factor for the display
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    /// Get the current scale factor
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
    // ========== Frame Lifecycle ==========

    /// Begin a new frame
    ///
    /// This should be called at the start of each frame before building UI.
    /// It prepares the context for a new frame. Input state is accumulated
    /// via `input_mut().handle_winit_event()` between frames.
    pub fn begin_frame(&mut self) {
        self.state_manager.begin_frame();
        self.id_counter = 0;
    }

    /// End the current frame
    ///
    /// This should be called after building UI and computing layout.
    /// It dispatches events which will be available in the next frame.
    pub fn end_frame(&mut self, root: &mut Node) {
        // Assign auto-IDs to nodes that need them
        InteractiveStateManager::assign_auto_ids(root);

        // Restore scroll state from previous frame
        self.dispatcher.restore_scroll_state(root);

        // Dispatch events based on input and hit testing
        let (events, interaction_states) = self.dispatcher.dispatch(&self.input, root);
        self.events = events;
        self.interaction_states = interaction_states;

        // Update style transitions
        self.state_manager
            .update_transitions(root, &self.interaction_states);

        // Advance smooth scroll animations toward their targets. Derive dt from
        // the time since the previous frame so the easing is framerate
        // independent. Without this, scroll_offset never moves toward
        // scroll_target and scrolling has no visible effect.
        let now = std::time::Instant::now();
        let dt = self
            .last_frame_time
            .map(|prev| (now - prev).as_secs_f32())
            .unwrap_or(0.0)
            .clamp(0.0, 0.1);
        self.last_frame_time = Some(now);
        if dt > 0.0 {
            root.update_all_scroll_animations(dt);
        }

        // Sync scroll state for persistence
        self.dispatcher.sync_scroll_state(root);
    }

    /// Inject dimension overrides before layout
    ///
    /// Call this after building the UI tree but before computing layout.
    /// This applies interpolated dimensions from ongoing transitions.
    pub fn inject_dimension_overrides(&self, root: &mut Node) {
        self.state_manager.inject_dimension_overrides(root);
    }

    /// Check if any transitions are currently active
    ///
    /// Use this to determine if continuous redraws are needed.
    pub fn has_active_transitions(&self) -> bool {
        self.state_manager.has_active_transitions()
    }

    // ========== Input State Access ==========

    /// Get the current input state
    pub fn input(&self) -> &InputState {
        &self.input
    }

    /// Get mutable access to the input state
    pub fn input_mut(&mut self) -> &mut InputState {
        &mut self.input
    }

    /// Get the current cursor position, if known
    pub fn cursor_position(&self) -> Option<crate::Point> {
        self.input.cursor_position
    }

    /// Check if a mouse button is currently held
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.input.is_button_down(button)
    }

    /// Check if Shift is held
    pub fn shift_held(&self) -> bool {
        self.input.shift_held
    }

    /// Check if Ctrl (or Cmd on macOS) is held
    pub fn ctrl_held(&self) -> bool {
        self.input.ctrl_held
    }

    // ========== Event Checking ==========

    /// Get all events from the last frame
    pub fn events(&self) -> &[TargetedEvent] {
        &self.events
    }

    /// Check if a widget was clicked in the last frame
    pub fn was_clicked(&self, id: &str) -> bool {
        self.events
            .iter()
            .any(|e| matches!(e.event, InteractionEvent::Click { .. }) && e.target.as_str() == id)
    }

    /// Check if a widget was clicked with a specific button
    pub fn was_clicked_with(&self, id: &str, button: MouseButton) -> bool {
        self.events.iter().any(|e| {
            matches!(&e.event, InteractionEvent::Click { button: b, .. } if *b == button)
                && e.target.as_str() == id
        })
    }

    /// Check if a widget is currently hovered
    pub fn is_hovered(&self, id: &str) -> bool {
        self.events
            .iter()
            .any(|e| matches!(e.event, InteractionEvent::Hover { .. }) && e.target.as_str() == id)
    }

    /// Check if a widget is being dragged
    pub fn is_dragging(&self, id: &str) -> bool {
        self.events.iter().any(|e| {
            matches!(
                e.event,
                InteractionEvent::DragStart { .. }
                    | InteractionEvent::DragMove { .. }
                    | InteractionEvent::DragEnd { .. }
            ) && e.target.as_str() == id
        })
    }

    /// Get drag delta for a widget, if it's being dragged
    pub fn drag_delta(&self, id: &str) -> Option<crate::Point> {
        self.events.iter().find_map(|e| {
            if e.target.as_str() == id {
                if let InteractionEvent::DragMove { delta, .. } = &e.event {
                    return Some(*delta);
                }
            }
            None
        })
    }

    /// Get all events targeting a specific widget
    pub fn events_for<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a TargetedEvent> {
        self.events.iter().filter(move |e| e.target.as_str() == id)
    }

    /// Get the interaction state for a widget
    pub fn interaction_state(&self, id: &str) -> InteractionState {
        let node_id = NodeId::new(id);
        self.interaction_states
            .get(&node_id)
            .copied()
            .unwrap_or(InteractionState::Idle)
    }

    // ========== Focus Management ==========

    /// Get the currently focused widget ID
    pub fn focused_widget(&self) -> Option<&NodeId> {
        self.dispatcher.focused_node()
    }

    /// Check if a widget is focused
    pub fn is_focused(&self, id: &str) -> bool {
        self.dispatcher
            .focused_node()
            .map(|fid| fid.as_str() == id)
            .unwrap_or(false)
    }

    /// Set the focused widget
    pub fn set_focus(&mut self, id: Option<&str>) {
        self.dispatcher.set_focus(id.map(|s| NodeId::new(s)));
    }

    /// Update cursor blink for a focused text widget
    pub fn update_cursor_blink(&mut self, id: &str, blink_rate_ms: u64) -> bool {
        self.dispatcher
            .update_cursor_blink(&NodeId::new(id), blink_rate_ms)
    }

    /// Reset cursor blink to visible (call when text changes)
    pub fn reset_cursor_blink(&mut self, id: &str) {
        self.dispatcher.reset_cursor_blink(&NodeId::new(id));
    }

    /// Check if cursor should be visible for a widget
    pub fn is_cursor_visible(&self, id: &str) -> bool {
        self.dispatcher.is_cursor_visible(&NodeId::new(id))
    }

    // ========== Widget Memory ==========

    /// Get access to widget memory for storing internal state
    pub fn memory(&mut self) -> &mut WidgetMemory {
        &mut self.memory
    }

    /// Get read-only access to widget memory
    pub fn memory_ref(&self) -> &WidgetMemory {
        &self.memory
    }

    // ========== Content Measurement ==========

    /// Get mutable access to the content measurer, if set
    pub fn measurer(&mut self) -> Option<&mut dyn ContentMeasurer> {
        match &mut self.measurer {
            Some(m) => Some(m.as_mut()),
            None => None,
        }
    }

    /// Measure text using the content measurer
    ///
    /// Returns zero size if no measurer is set.
    pub fn measure_text(&mut self, request: MeasureTextRequest<'_>) -> IntrinsicSize {
        if let Some(ref mut measurer) = self.measurer {
            measurer.measure_text(request)
        } else {
            IntrinsicSize::zero()
        }
    }

    // ========== ID Generation ==========

    /// Generate a unique ID for a widget
    ///
    /// IDs are generated based on:
    /// 1. The current ID stack (parent scopes)
    /// 2. The provided label/name
    /// 3. A counter for disambiguation
    ///
    /// This ensures stable IDs across frames as long as the UI structure
    /// remains the same.
    pub fn generate_id(&mut self, label: &str) -> String {
        let id = if self.id_stack.is_empty() {
            format!("{}_{}", label, self.id_counter)
        } else {
            format!("{}/{}_{}", self.id_stack.join("/"), label, self.id_counter)
        };
        self.id_counter += 1;
        id
    }

    /// Generate an ID without incrementing the counter
    ///
    /// Useful when you need to reference an ID before/after creating it.
    pub fn peek_id(&self, label: &str) -> String {
        if self.id_stack.is_empty() {
            format!("{}_{}", label, self.id_counter)
        } else {
            format!("{}/{}_{}", self.id_stack.join("/"), label, self.id_counter)
        }
    }

    /// Push a scope onto the ID stack
    ///
    /// All IDs generated while this scope is active will be prefixed
    /// with this scope name.
    pub fn push_id(&mut self, scope: impl Into<String>) {
        self.id_stack.push(scope.into());
    }

    /// Pop the current scope from the ID stack
    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    /// Execute a closure with a temporary ID scope
    pub fn with_id_scope<R>(
        &mut self,
        scope: impl Into<String>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.push_id(scope);
        let result = f(self);
        self.pop_id();
        result
    }

    // ========== Event Dispatcher Access ==========

    /// Get mutable access to the event dispatcher
    ///
    /// This is provided for advanced use cases. Prefer using the
    /// higher-level methods like `was_clicked()` when possible.
    pub fn dispatcher(&mut self) -> &mut EventDispatcher {
        &mut self.dispatcher
    }

    /// Get read-only access to the event dispatcher
    pub fn dispatcher_ref(&self) -> &EventDispatcher {
        &self.dispatcher
    }

    // ========== State Manager Access ==========

    /// Get mutable access to the interactive state manager
    pub fn state_manager(&mut self) -> &mut InteractiveStateManager {
        &mut self.state_manager
    }

    /// Get read-only access to the interactive state manager
    pub fn state_manager_ref(&self) -> &InteractiveStateManager {
        &self.state_manager
    }
}

impl Default for UiContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for UiContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiContext")
            .field("events", &self.events.len())
            .field("memory", &self.memory)
            .field("id_stack", &self.id_stack)
            .field("scale_factor", &self.scale_factor)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = UiContext::new();
        assert!(ctx.events().is_empty());
        assert!(ctx.focused_widget().is_none());
    }

    #[test]
    fn test_id_generation() {
        let mut ctx = UiContext::new();

        let id1 = ctx.generate_id("button");
        let id2 = ctx.generate_id("button");
        let id3 = ctx.generate_id("slider");

        assert_eq!(id1, "button_0");
        assert_eq!(id2, "button_1");
        assert_eq!(id3, "slider_2");
    }

    #[test]
    fn test_id_scoping() {
        let mut ctx = UiContext::new();

        ctx.push_id("parent");
        let id1 = ctx.generate_id("child");
        ctx.pop_id();

        let id2 = ctx.generate_id("sibling");

        assert_eq!(id1, "parent/child_0");
        assert_eq!(id2, "sibling_1");
    }

    #[test]
    fn test_with_id_scope() {
        let mut ctx = UiContext::new();

        let id = ctx.with_id_scope("container", |ctx| ctx.generate_id("item"));

        assert_eq!(id, "container/item_0");
        assert!(ctx.id_stack.is_empty());
    }

    #[test]
    fn test_scroll_moves_offset_toward_target() {
        use crate::{Layout, Overflow, Point, Rect, Size};

        // Rebuild a fresh tree every frame (as a real app does), so the test
        // exercises the restore/dispatch/animate/sync persistence pipeline.
        let build = || {
            Node::new()
                .with_id(NodeId::new("scroller"))
                .with_width(Size::lpx(100.0))
                .with_height(Size::lpx(100.0))
                .with_layout_direction(Layout::Vertical)
                .with_overflow(Overflow::Scroll)
                .with_child(
                    // Content taller than the container -> 400px of scroll range.
                    Node::new()
                        .with_width(Size::lpx(100.0))
                        .with_height(Size::lpx(500.0)),
                )
        };

        let window = Rect::from_min_size([0.0, 0.0], [100.0, 100.0]);
        let mut ctx = UiContext::new();

        // Frame 1: wheel scroll down (negative y delta) with the cursor over the
        // container. This should set the scroll target but not yet move offset.
        ctx.begin_frame();
        let mut root = build();
        ctx.input_mut().cursor_position = Some(Point::new(50.0, 50.0));
        // Scroll far past the end to also exercise clamping to max_scroll.
        ctx.input_mut().scroll_delta = (0.0, -1000.0);
        root.compute_layout(window);
        ctx.end_frame(&mut root);
        ctx.input_mut().begin_frame(); // clear per-frame input (scroll_delta)

        // Subsequent frames: no new input, just let the animation advance. The
        // easing is driven by real elapsed time, so emulate ~60fps frame pacing.
        let mut last_offset = 0.0;
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(16));
            ctx.begin_frame();
            let mut root = build();
            ctx.input_mut().cursor_position = Some(Point::new(50.0, 50.0));
            root.compute_layout(window);
            ctx.end_frame(&mut root);
            ctx.input_mut().begin_frame();
            last_offset = root.scroll_offset().1;
        }

        // The offset must have moved downward and settled within the scroll
        // range (content 500 - container 100 = 400 max).
        assert!(
            last_offset > 1.0,
            "scroll offset should advance toward target, got {last_offset}"
        );
        assert!(
            last_offset <= 400.0 + 0.01,
            "scroll offset should be clamped to max_scroll (400), got {last_offset}"
        );
        // With a large delta it should approach (but not exceed) the clamp.
        assert!(
            last_offset > 300.0,
            "scroll offset should approach the clamped maximum, got {last_offset}"
        );
    }

    #[test]
    fn test_scroll_axis_routing() {
        use crate::{Layout, Overflow, Point, Rect, Size};

        let window = Rect::from_min_size([0.0, 0.0], [100.0, 100.0]);

        // Run a single frame with the given content size, shift state, and wheel
        // delta; return the resulting (target_x, target_y).
        let run = |child_w: f32, child_h: f32, shift: bool, delta: (f32, f32)| -> (f32, f32) {
            let mut ctx = UiContext::new();
            ctx.begin_frame();
            let mut root = Node::new()
                .with_id(NodeId::new("s"))
                .with_width(Size::lpx(100.0))
                .with_height(Size::lpx(100.0))
                .with_layout_direction(Layout::Vertical)
                .with_overflow(Overflow::Scroll)
                .with_child(
                    Node::new()
                        .with_width(Size::lpx(child_w))
                        .with_height(Size::lpx(child_h)),
                );
            ctx.input_mut().cursor_position = Some(Point::new(50.0, 50.0));
            ctx.input_mut().scroll_delta = delta;
            ctx.input_mut().shift_held = shift;
            root.compute_layout(window);
            ctx.end_frame(&mut root);
            root.scroll_target()
        };

        // A mouse wheel reports a vertical delta only.
        let wheel = (0.0, -100.0);

        // Overflows both axes, no shift -> scrolls Y.
        let (tx, ty) = run(500.0, 500.0, false, wheel);
        assert!(ty > 0.0 && tx == 0.0, "2D no-shift should scroll Y, got ({tx},{ty})");

        // Overflows both axes, shift held -> scrolls X.
        let (tx, ty) = run(500.0, 500.0, true, wheel);
        assert!(tx > 0.0 && ty == 0.0, "2D shift should scroll X, got ({tx},{ty})");

        // Overflows X only -> vertical wheel routed to X without shift.
        let (tx, ty) = run(500.0, 100.0, false, wheel);
        assert!(
            tx > 0.0 && ty == 0.0,
            "X-only should scroll X without shift, got ({tx},{ty})"
        );
    }

    #[test]
    fn test_scroll_speed_is_configurable() {
        use crate::{Layout, Overflow, Point, Rect, Size};

        let window = Rect::from_min_size([0.0, 0.0], [100.0, 100.0]);

        // One frame with an optional explicit scroll speed; return target Y. The
        // content is tall enough that the result is never clamped.
        let run = |speed: Option<f32>| -> f32 {
            let mut ctx = UiContext::new();
            ctx.begin_frame();
            let mut node = Node::new()
                .with_id(NodeId::new("s"))
                .with_width(Size::lpx(100.0))
                .with_height(Size::lpx(100.0))
                .with_layout_direction(Layout::Vertical)
                .with_overflow(Overflow::Scroll)
                .with_child(
                    Node::new()
                        .with_width(Size::lpx(100.0))
                        .with_height(Size::lpx(5000.0)),
                );
            if let Some(s) = speed {
                node = node.with_scroll_speed(s);
            }
            ctx.input_mut().cursor_position = Some(Point::new(50.0, 50.0));
            ctx.input_mut().scroll_delta = (0.0, -10.0);
            node.compute_layout(window);
            ctx.end_frame(&mut node);
            node.scroll_target().1
        };

        let default_speed = run(None); // default 2.0 -> 20
        let faster = run(Some(8.0)); // 8.0 -> 80
        assert!(default_speed > 0.0, "default scroll should move, got {default_speed}");
        assert!(
            faster > default_speed * 3.0,
            "with_scroll_speed should scroll further: {faster} vs {default_speed}"
        );
    }

    #[test]
    fn test_focus_management() {
        let mut ctx = UiContext::new();

        assert!(!ctx.is_focused("my_input"));

        ctx.set_focus(Some("my_input"));
        assert!(ctx.is_focused("my_input"));

        ctx.set_focus(None);
        assert!(!ctx.is_focused("my_input"));
    }
}
