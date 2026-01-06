//! Widget memory for storing internal widget state
//!
//! This module provides the `WidgetMemory` struct which stores per-widget
//! internal state that persists across frames. This enables widgets like
//! text inputs, sliders, and drag values to maintain their state without
//! requiring the user to manually manage it.

use std::any::Any;
use std::collections::HashMap;

/// Unique identifier for widget state storage
///
/// This is typically derived from the widget's ID and type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WidgetStateId(String);

impl WidgetStateId {
    /// Create a new widget state ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create a widget state ID with a type suffix
    ///
    /// This is useful when a single widget needs to store multiple types of state.
    pub fn with_suffix(id: impl Into<String>, suffix: &str) -> Self {
        Self(format!("{}_{}", id.into(), suffix))
    }
}

impl<S: Into<String>> From<S> for WidgetStateId {
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

/// State for a text input widget
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    /// The text content
    pub text: String,
    /// Cursor position (byte offset)
    pub cursor_pos: usize,
    /// Selection range (start, end) in byte offsets, if any
    pub selection: Option<(usize, usize)>,
    /// Whether the widget is focused
    pub focused: bool,
}

impl TextInputState {
    /// Create new text input state with initial text
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor_pos = text.len();
        Self {
            text,
            cursor_pos,
            selection: None,
            focused: false,
        }
    }

    /// Get the selected text, if any
    pub fn selected_text(&self) -> Option<&str> {
        self.selection.map(|(start, end)| &self.text[start..end])
    }

    /// Clear the selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Set cursor position, clamping to valid range
    pub fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor_pos = pos.min(self.text.len());
    }
}

/// State for a drag value widget
#[derive(Debug, Clone, Default)]
pub struct DragValueState {
    /// Text input state (for when in text input mode)
    pub text_input: TextInputState,
    /// Continuous accumulator for drag movements
    pub drag_accumulator: f32,
    /// Whether currently in text input mode
    pub text_mode: bool,
}

impl DragValueState {
    /// Create new drag value state with initial value
    pub fn new(value: f32) -> Self {
        Self {
            text_input: TextInputState::default(),
            drag_accumulator: value,
            text_mode: false,
        }
    }

    /// Enter text input mode with the current value
    pub fn enter_text_mode(&mut self, value: f32, precision: usize) {
        self.text_mode = true;
        self.text_input.text = format_value(value, precision);
        self.text_input.cursor_pos = self.text_input.text.len();
        self.text_input.selection = None;
        self.text_input.focused = true;
    }

    /// Exit text input mode
    pub fn exit_text_mode(&mut self) {
        self.text_mode = false;
        self.text_input.focused = false;
    }
}

/// Format a float value with the given precision
fn format_value(value: f32, precision: usize) -> String {
    if precision == 0 {
        format!("{:.0}", value)
    } else {
        let formatted = format!("{:.prec$}", value, prec = precision);
        // Strip trailing zeros after decimal point
        if formatted.contains('.') {
            formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            formatted
        }
    }
}

/// State for a slider widget
#[derive(Debug, Clone, Default)]
pub struct SliderState {
    /// Whether the slider is currently being dragged
    pub dragging: bool,
}

/// State for a collapsible widget
#[derive(Debug, Clone)]
pub struct CollapsibleState {
    /// Whether the collapsible is expanded
    pub expanded: bool,
}

impl Default for CollapsibleState {
    fn default() -> Self {
        Self { expanded: true }
    }
}

impl CollapsibleState {
    /// Create new collapsible state
    pub fn new(expanded: bool) -> Self {
        Self { expanded }
    }

    /// Toggle the expanded state
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

/// State for a toggle/checkbox widget
#[derive(Debug, Clone, Default)]
pub struct ToggleState {
    /// Whether the toggle is on/checked
    pub checked: bool,
}

impl ToggleState {
    /// Create new toggle state
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }

    /// Toggle the checked state
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

/// Widget memory - stores internal state for all widgets
///
/// This is a type-erased storage that allows widgets to store arbitrary
/// state that persists across frames. Each widget type should use a
/// consistent state type (e.g., `TextInputState` for text inputs).
pub struct WidgetMemory {
    /// Type-erased storage for widget states
    states: HashMap<WidgetStateId, Box<dyn Any>>,
}

impl WidgetMemory {
    /// Create a new empty widget memory
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Get or create state for a widget
    ///
    /// If state doesn't exist for this ID, creates it using the provided default.
    pub fn get_or_insert<T: 'static>(
        &mut self,
        id: impl Into<WidgetStateId>,
        default: T,
    ) -> &mut T {
        let id = id.into();
        self.states
            .entry(id)
            .or_insert_with(|| Box::new(default))
            .downcast_mut::<T>()
            .expect("Widget state type mismatch")
    }

    /// Get or create state for a widget using Default
    pub fn get_or_default<T: Default + 'static>(&mut self, id: impl Into<WidgetStateId>) -> &mut T {
        self.get_or_insert(id, T::default())
    }

    /// Get state for a widget, if it exists
    pub fn get<T: 'static>(&self, id: impl Into<WidgetStateId>) -> Option<&T> {
        let id = id.into();
        self.states.get(&id).and_then(|s| s.downcast_ref::<T>())
    }

    /// Get mutable state for a widget, if it exists
    pub fn get_mut<T: 'static>(&mut self, id: impl Into<WidgetStateId>) -> Option<&mut T> {
        let id = id.into();
        self.states.get_mut(&id).and_then(|s| s.downcast_mut::<T>())
    }

    /// Check if state exists for a widget
    pub fn contains(&self, id: impl Into<WidgetStateId>) -> bool {
        self.states.contains_key(&id.into())
    }

    /// Remove state for a widget
    pub fn remove(&mut self, id: impl Into<WidgetStateId>) -> bool {
        self.states.remove(&id.into()).is_some()
    }

    /// Clear all widget state
    pub fn clear(&mut self) {
        self.states.clear();
    }

    /// Get the number of stored states
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Check if the memory is empty
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    // Convenience methods for common widget types

    /// Get or create text input state
    pub fn text_input(&mut self, id: impl Into<WidgetStateId>) -> &mut TextInputState {
        self.get_or_default(id)
    }

    /// Get or create text input state with initial text
    pub fn text_input_with_text(
        &mut self,
        id: impl Into<WidgetStateId>,
        initial_text: impl Into<String>,
    ) -> &mut TextInputState {
        self.get_or_insert(id, TextInputState::new(initial_text))
    }

    /// Get or create drag value state
    pub fn drag_value(
        &mut self,
        id: impl Into<WidgetStateId>,
        initial_value: f32,
    ) -> &mut DragValueState {
        self.get_or_insert(id, DragValueState::new(initial_value))
    }

    /// Get or create slider state
    pub fn slider(&mut self, id: impl Into<WidgetStateId>) -> &mut SliderState {
        self.get_or_default(id)
    }

    /// Get or create collapsible state
    pub fn collapsible(
        &mut self,
        id: impl Into<WidgetStateId>,
        initial_expanded: bool,
    ) -> &mut CollapsibleState {
        self.get_or_insert(id, CollapsibleState::new(initial_expanded))
    }

    /// Get or create toggle state
    pub fn toggle(
        &mut self,
        id: impl Into<WidgetStateId>,
        initial_checked: bool,
    ) -> &mut ToggleState {
        self.get_or_insert(id, ToggleState::new(initial_checked))
    }
}

impl Default for WidgetMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for WidgetMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetMemory")
            .field("num_states", &self.states.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_state() {
        let mut state = TextInputState::new("hello");
        assert_eq!(state.text, "hello");
        assert_eq!(state.cursor_pos, 5);
        assert!(state.selection.is_none());

        state.selection = Some((0, 3));
        assert_eq!(state.selected_text(), Some("hel"));

        state.clear_selection();
        assert!(state.selection.is_none());
    }

    #[test]
    fn test_drag_value_state() {
        let mut state = DragValueState::new(42.0);
        assert_eq!(state.drag_accumulator, 42.0);
        assert!(!state.text_mode);

        state.enter_text_mode(42.5, 2);
        assert!(state.text_mode);
        assert_eq!(state.text_input.text, "42.5");

        state.exit_text_mode();
        assert!(!state.text_mode);
    }

    #[test]
    fn test_widget_memory_basic() {
        let mut memory = WidgetMemory::new();
        assert!(memory.is_empty());

        let state = memory.text_input("my_input");
        state.text = "test".to_string();

        assert_eq!(memory.len(), 1);
        assert!(memory.contains("my_input"));

        let state = memory.get::<TextInputState>("my_input").unwrap();
        assert_eq!(state.text, "test");
    }

    #[test]
    fn test_widget_memory_type_safety() {
        let mut memory = WidgetMemory::new();

        memory.text_input("widget1");
        memory.drag_value("widget2", 10.0);

        // Different types for different IDs work fine
        assert!(memory.get::<TextInputState>("widget1").is_some());
        assert!(memory.get::<DragValueState>("widget2").is_some());

        // Wrong type returns None
        assert!(memory.get::<DragValueState>("widget1").is_none());
        assert!(memory.get::<TextInputState>("widget2").is_none());
    }

    #[test]
    fn test_collapsible_state() {
        let mut state = CollapsibleState::new(false);
        assert!(!state.expanded);

        state.toggle();
        assert!(state.expanded);

        state.toggle();
        assert!(!state.expanded);
    }

    #[test]
    fn test_toggle_state() {
        let mut state = ToggleState::new(false);
        assert!(!state.checked);

        state.toggle();
        assert!(state.checked);
    }
}
