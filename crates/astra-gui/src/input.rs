//! Backend-agnostic input state tracking for mouse and keyboard events
//!
//! This module provides structures to track input state across frames,
//! independent of any specific windowing library (winit, SDL, etc.).

use crate::Point;
use std::collections::HashSet;

/// Backend-agnostic mouse button representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button (scroll wheel click)
    Middle,
    /// Additional mouse buttons (back, forward, etc.)
    Other(u8),
}

/// Backend-agnostic named key representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NamedKey {
    /// Enter/Return key
    Enter,
    /// Escape key
    Escape,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Tab key
    Tab,
    /// Space key
    Space,
    /// Left arrow key
    ArrowLeft,
    /// Right arrow key
    ArrowRight,
    /// Up arrow key
    ArrowUp,
    /// Down arrow key
    ArrowDown,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
    /// Shift key (left or right)
    Shift,
    /// Control key (left or right)
    Control,
    /// Alt key (left or right)
    Alt,
    /// Super/Meta/Windows/Command key
    Super,
    /// Caps Lock key
    CapsLock,
    /// Function keys F1-F12
    F(u8),
    /// Insert key
    Insert,
    /// Print Screen key
    PrintScreen,
    /// Scroll Lock key
    ScrollLock,
    /// Pause/Break key
    Pause,
    /// Num Lock key
    NumLock,
    /// Context menu key
    ContextMenu,
}

/// Backend-agnostic key representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    /// A named key (Enter, Escape, arrows, etc.)
    Named(NamedKey),
    /// A character key (letters, numbers, symbols)
    Character(String),
    /// Unknown/unhandled key
    Unknown,
}

/// Tracks the current state of mouse and keyboard input
///
/// This structure maintains both the current state and frame-specific events
/// (just pressed/just released) to enable easy input handling in the UI.
///
/// This is backend-agnostic - windowing libraries should convert their
/// events to update this structure.
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current cursor position in window coordinates, if known
    pub cursor_position: Option<Point>,

    /// Set of mouse buttons currently held down
    pub buttons_pressed: HashSet<MouseButton>,

    /// Set of mouse buttons that were pressed this frame
    pub buttons_just_pressed: HashSet<MouseButton>,

    /// Set of mouse buttons that were released this frame
    pub buttons_just_released: HashSet<MouseButton>,

    /// Characters typed this frame (for text input)
    pub characters_typed: Vec<char>,

    /// Keys pressed this frame
    pub keys_just_pressed: Vec<Key>,

    /// Keys released this frame
    pub keys_just_released: Vec<Key>,

    /// Whether Shift is currently held down
    pub shift_held: bool,

    /// Whether Ctrl (or Cmd on macOS) is currently held down
    pub ctrl_held: bool,

    /// Whether Alt is currently held down
    pub alt_held: bool,

    /// Whether Super/Meta/Windows/Command is currently held down
    pub super_held: bool,

    /// Scroll delta this frame (horizontal, vertical) in pixels
    pub scroll_delta: (f32, f32),
}

impl InputState {
    /// Create a new input state with no active input
    pub fn new() -> Self {
        Self {
            cursor_position: None,
            buttons_pressed: HashSet::new(),
            buttons_just_pressed: HashSet::new(),
            buttons_just_released: HashSet::new(),
            characters_typed: Vec::new(),
            keys_just_pressed: Vec::new(),
            keys_just_released: Vec::new(),
            shift_held: false,
            ctrl_held: false,
            alt_held: false,
            super_held: false,
            scroll_delta: (0.0, 0.0),
        }
    }

    /// Call at the start of each frame to clear frame-specific state
    ///
    /// This clears the "just pressed" and "just released" sets so they only
    /// contain events from the current frame.
    pub fn begin_frame(&mut self) {
        self.buttons_just_pressed.clear();
        self.buttons_just_released.clear();
        self.characters_typed.clear();
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.scroll_delta = (0.0, 0.0);
    }

    /// Record a mouse button press
    pub fn press_button(&mut self, button: MouseButton) {
        self.buttons_pressed.insert(button);
        self.buttons_just_pressed.insert(button);
    }

    /// Record a mouse button release
    pub fn release_button(&mut self, button: MouseButton) {
        self.buttons_pressed.remove(&button);
        self.buttons_just_released.insert(button);
    }

    /// Record a key press
    ///
    /// # Arguments
    /// * `key` - The key that was pressed
    /// * `is_repeat` - Whether this is a key repeat event
    /// * `allow_repeat` - Whether to record repeat events for this key
    pub fn press_key(&mut self, key: Key, is_repeat: bool, allow_repeat: bool) {
        // Update modifier state
        if let Key::Named(named) = &key {
            match named {
                NamedKey::Shift => self.shift_held = true,
                NamedKey::Control => self.ctrl_held = true,
                NamedKey::Alt => self.alt_held = true,
                NamedKey::Super => self.super_held = true,
                _ => {}
            }
        }

        // Record the key press if it's not a repeat, or if repeats are allowed
        if !is_repeat || allow_repeat {
            self.keys_just_pressed.push(key);
        }
    }

    /// Record a key release
    pub fn release_key(&mut self, key: Key) {
        // Update modifier state
        if let Key::Named(named) = &key {
            match named {
                NamedKey::Shift => self.shift_held = false,
                NamedKey::Control => self.ctrl_held = false,
                NamedKey::Alt => self.alt_held = false,
                NamedKey::Super => self.super_held = false,
                _ => {}
            }
        }

        self.keys_just_released.push(key);
    }

    /// Record a character typed (for text input)
    pub fn type_character(&mut self, ch: char) {
        self.characters_typed.push(ch);
    }

    /// Update cursor position
    pub fn set_cursor_position(&mut self, position: Option<Point>) {
        self.cursor_position = position;
    }

    /// Add scroll delta
    pub fn add_scroll_delta(&mut self, horizontal: f32, vertical: f32) {
        self.scroll_delta.0 += horizontal;
        self.scroll_delta.1 += vertical;
    }

    /// Check if a mouse button is currently held down
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.buttons_pressed.contains(&button)
    }

    /// Check if a mouse button was pressed this frame
    pub fn is_button_just_pressed(&self, button: MouseButton) -> bool {
        self.buttons_just_pressed.contains(&button)
    }

    /// Check if a mouse button was released this frame
    pub fn is_button_just_released(&self, button: MouseButton) -> bool {
        self.buttons_just_released.contains(&button)
    }

    /// Check if a specific key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: &Key) -> bool {
        self.keys_just_pressed.contains(key)
    }

    /// Check if a specific named key was just pressed this frame
    pub fn is_named_key_just_pressed(&self, named: NamedKey) -> bool {
        self.keys_just_pressed
            .iter()
            .any(|k| matches!(k, Key::Named(n) if *n == named))
    }

    /// Check if any modifier key is held (Ctrl, Alt, Super, but not Shift)
    pub fn any_modifier_held(&self) -> bool {
        self.ctrl_held || self.alt_held || self.super_held
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_press_release() {
        let mut input = InputState::new();

        // Press left button
        input.press_button(MouseButton::Left);
        assert!(input.is_button_down(MouseButton::Left));
        assert!(input.is_button_just_pressed(MouseButton::Left));
        assert!(!input.is_button_just_released(MouseButton::Left));

        // New frame
        input.begin_frame();
        assert!(input.is_button_down(MouseButton::Left));
        assert!(!input.is_button_just_pressed(MouseButton::Left));

        // Release
        input.release_button(MouseButton::Left);
        assert!(!input.is_button_down(MouseButton::Left));
        assert!(input.is_button_just_released(MouseButton::Left));
    }

    #[test]
    fn test_modifier_keys() {
        let mut input = InputState::new();

        assert!(!input.shift_held);
        assert!(!input.ctrl_held);

        input.press_key(Key::Named(NamedKey::Shift), false, false);
        assert!(input.shift_held);

        input.press_key(Key::Named(NamedKey::Control), false, false);
        assert!(input.ctrl_held);

        input.release_key(Key::Named(NamedKey::Shift));
        assert!(!input.shift_held);
        assert!(input.ctrl_held);
    }

    #[test]
    fn test_character_input() {
        let mut input = InputState::new();

        input.type_character('a');
        input.type_character('b');
        input.type_character('c');

        assert_eq!(input.characters_typed, vec!['a', 'b', 'c']);

        input.begin_frame();
        assert!(input.characters_typed.is_empty());
    }
}
