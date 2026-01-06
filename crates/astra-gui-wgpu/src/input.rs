//! Winit input adapter for astra-gui
//!
//! This module provides conversion from winit events to astra-gui's
//! backend-agnostic input types.

use astra_gui::{InputState, Key, MouseButton, NamedKey, Point};
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::Key as WinitKey;

/// Extension trait for InputState to handle winit events
pub trait WinitInputExt {
    /// Process a winit WindowEvent and update internal state
    ///
    /// This should be called for each WindowEvent received from winit.
    fn handle_winit_event(&mut self, event: &WindowEvent);
}

impl WinitInputExt for InputState {
    fn handle_winit_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.set_cursor_position(Some(Point {
                    x: position.x as f32,
                    y: position.y as f32,
                }));
            }
            WindowEvent::CursorLeft { .. } => {
                self.set_cursor_position(None);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        // Line delta - multiply by pixels per line (typical: 20-40)
                        const PIXELS_PER_LINE: f32 = 20.0;
                        self.add_scroll_delta(x * PIXELS_PER_LINE, y * PIXELS_PER_LINE);
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        // Pixel delta - use directly
                        self.add_scroll_delta(pos.x as f32, pos.y as f32);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let btn = convert_mouse_button(*button);
                match state {
                    ElementState::Pressed => {
                        self.press_button(btn);
                    }
                    ElementState::Released => {
                        self.release_button(btn);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let key = convert_key(&event.logical_key);

                // Allow repeats for navigation and editing keys
                let allow_repeat = matches!(
                    key,
                    Key::Named(NamedKey::Backspace)
                        | Key::Named(NamedKey::Delete)
                        | Key::Named(NamedKey::ArrowLeft)
                        | Key::Named(NamedKey::ArrowRight)
                        | Key::Named(NamedKey::ArrowUp)
                        | Key::Named(NamedKey::ArrowDown)
                );

                match event.state {
                    ElementState::Pressed => {
                        self.press_key(key, event.repeat, allow_repeat);

                        // Handle text input from key events
                        match &event.logical_key {
                            WinitKey::Character(ref text) => {
                                // Only skip if it's a ctrl+key shortcut (ctrl+letter, but not space)
                                let is_shortcut = self.ctrl_held
                                    && text.len() == 1
                                    && text.chars().next().unwrap().is_alphabetic();
                                if !is_shortcut {
                                    for ch in text.chars() {
                                        self.type_character(ch);
                                    }
                                }
                            }
                            WinitKey::Named(winit::keyboard::NamedKey::Space) => {
                                // Always allow space, even with modifiers
                                self.type_character(' ');
                            }
                            _ => {}
                        }
                    }
                    ElementState::Released => {
                        self.release_key(key);
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// Convert winit MouseButton to astra-gui MouseButton
pub fn convert_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Other(3),
        winit::event::MouseButton::Forward => MouseButton::Other(4),
        winit::event::MouseButton::Other(n) => MouseButton::Other(n as u8),
    }
}

/// Convert astra-gui MouseButton to winit MouseButton
/// Convert astra-gui MouseButton to winit MouseButton
#[allow(dead_code)]
pub fn convert_mouse_button_to_winit(button: MouseButton) -> winit::event::MouseButton {
    match button {
        MouseButton::Left => winit::event::MouseButton::Left,
        MouseButton::Right => winit::event::MouseButton::Right,
        MouseButton::Middle => winit::event::MouseButton::Middle,
        MouseButton::Other(3) => winit::event::MouseButton::Back,
        MouseButton::Other(4) => winit::event::MouseButton::Forward,
        MouseButton::Other(n) => winit::event::MouseButton::Other(n as u16),
    }
}

/// Convert winit Key to astra-gui Key
pub fn convert_key(key: &WinitKey) -> Key {
    match key {
        WinitKey::Named(named) => Key::Named(convert_named_key(named)),
        WinitKey::Character(s) => Key::Character(s.to_string()),
        _ => Key::Unknown,
    }
}

/// Convert winit NamedKey to astra-gui NamedKey
pub fn convert_named_key(key: &winit::keyboard::NamedKey) -> NamedKey {
    use winit::keyboard::NamedKey as WN;

    match key {
        WN::Enter => NamedKey::Enter,
        WN::Escape => NamedKey::Escape,
        WN::Backspace => NamedKey::Backspace,
        WN::Delete => NamedKey::Delete,
        WN::Tab => NamedKey::Tab,
        WN::Space => NamedKey::Space,
        WN::ArrowLeft => NamedKey::ArrowLeft,
        WN::ArrowRight => NamedKey::ArrowRight,
        WN::ArrowUp => NamedKey::ArrowUp,
        WN::ArrowDown => NamedKey::ArrowDown,
        WN::Home => NamedKey::Home,
        WN::End => NamedKey::End,
        WN::PageUp => NamedKey::PageUp,
        WN::PageDown => NamedKey::PageDown,
        WN::Shift => NamedKey::Shift,
        WN::Control => NamedKey::Control,
        WN::Alt => NamedKey::Alt,
        WN::Super => NamedKey::Super,
        WN::CapsLock => NamedKey::CapsLock,
        WN::F1 => NamedKey::F(1),
        WN::F2 => NamedKey::F(2),
        WN::F3 => NamedKey::F(3),
        WN::F4 => NamedKey::F(4),
        WN::F5 => NamedKey::F(5),
        WN::F6 => NamedKey::F(6),
        WN::F7 => NamedKey::F(7),
        WN::F8 => NamedKey::F(8),
        WN::F9 => NamedKey::F(9),
        WN::F10 => NamedKey::F(10),
        WN::F11 => NamedKey::F(11),
        WN::F12 => NamedKey::F(12),
        WN::Insert => NamedKey::Insert,
        WN::PrintScreen => NamedKey::PrintScreen,
        WN::ScrollLock => NamedKey::ScrollLock,
        WN::Pause => NamedKey::Pause,
        WN::NumLock => NamedKey::NumLock,
        WN::ContextMenu => NamedKey::ContextMenu,
        // Map any other named keys to a reasonable default
        _ => NamedKey::Enter, // fallback, should rarely happen
    }
}

/// Convert astra-gui Key to winit Key
/// Convert astra-gui Key to winit Key
#[allow(dead_code)]
pub fn convert_key_to_winit(key: &Key) -> WinitKey {
    match key {
        Key::Named(named) => WinitKey::Named(convert_named_key_to_winit(named)),
        Key::Character(s) => WinitKey::Character(s.as_str().into()),
        Key::Unknown => WinitKey::Unidentified(winit::keyboard::NativeKey::Unidentified),
    }
}

/// Convert astra-gui NamedKey to winit NamedKey
/// Convert astra-gui NamedKey to winit NamedKey
#[allow(dead_code)]
pub fn convert_named_key_to_winit(key: &NamedKey) -> winit::keyboard::NamedKey {
    use winit::keyboard::NamedKey as WN;

    match key {
        NamedKey::Enter => WN::Enter,
        NamedKey::Escape => WN::Escape,
        NamedKey::Backspace => WN::Backspace,
        NamedKey::Delete => WN::Delete,
        NamedKey::Tab => WN::Tab,
        NamedKey::Space => WN::Space,
        NamedKey::ArrowLeft => WN::ArrowLeft,
        NamedKey::ArrowRight => WN::ArrowRight,
        NamedKey::ArrowUp => WN::ArrowUp,
        NamedKey::ArrowDown => WN::ArrowDown,
        NamedKey::Home => WN::Home,
        NamedKey::End => WN::End,
        NamedKey::PageUp => WN::PageUp,
        NamedKey::PageDown => WN::PageDown,
        NamedKey::Shift => WN::Shift,
        NamedKey::Control => WN::Control,
        NamedKey::Alt => WN::Alt,
        NamedKey::Super => WN::Super,
        NamedKey::CapsLock => WN::CapsLock,
        NamedKey::F(1) => WN::F1,
        NamedKey::F(2) => WN::F2,
        NamedKey::F(3) => WN::F3,
        NamedKey::F(4) => WN::F4,
        NamedKey::F(5) => WN::F5,
        NamedKey::F(6) => WN::F6,
        NamedKey::F(7) => WN::F7,
        NamedKey::F(8) => WN::F8,
        NamedKey::F(9) => WN::F9,
        NamedKey::F(10) => WN::F10,
        NamedKey::F(11) => WN::F11,
        NamedKey::F(12) => WN::F12,
        NamedKey::F(_) => WN::F1, // fallback for F13+
        NamedKey::Insert => WN::Insert,
        NamedKey::PrintScreen => WN::PrintScreen,
        NamedKey::ScrollLock => WN::ScrollLock,
        NamedKey::Pause => WN::Pause,
        NamedKey::NumLock => WN::NumLock,
        NamedKey::ContextMenu => WN::ContextMenu,
    }
}
