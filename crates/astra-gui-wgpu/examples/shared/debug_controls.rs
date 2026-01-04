//! Shared debug controls for `astra-gui-wgpu` examples.
//!
//! This module centralizes the "debug visualization" keybinds so all examples can
//! share identical behavior and help text.
//!
//! Controls:
//! - M: margins (red overlay)
//! - P: padding (blue overlay)
//! - B: borders (green outline)
//! - C: content area (yellow outline)
//! - R: clip rects (red outline)
//! - G: gaps (purple overlay)
//! - O: transform origins (orange crosshair)
//! - T: text line bounds (cyan outline)
//! - D: toggle all debug visualizations
//!
//! Usage (in an example):
//! - Print `DEBUG_HELP_TEXT` once at startup
//! - Call `handle_debug_keybinds(&event, &mut debug_options)`
//!   early in `window_event` and return if it handled the key.

use astra_gui::DebugOptions;
use winit::event::{ElementState, KeyEvent, WindowEvent};

/// Multi-line help text suitable for printing in the console at startup.
///
/// Note: Some examples only use the on-screen footer (`DEBUG_HELP_TEXT_ONELINE`) and won't print
/// this string, but we keep it here since it's part of the shared public example API.
#[allow(dead_code)]
pub const DEBUG_HELP_TEXT: &str = "Debug controls:
  M - Toggle margins (red overlay)
  P - Toggle padding (blue overlay)
  B - Toggle borders (green outline)
  C - Toggle content area (yellow outline)
  R - Toggle clip rects (red outline)
  G - Toggle gaps (purple overlay)
  O - Toggle transform origins (orange crosshair)
  T - Toggle text line bounds (cyan outline)
  D - Toggle all debug visualizations
  F - Toggle frame profiling
  ESC - Exit";

/// Single-line help text suitable for an in-app HUD/footer label.
///
/// Note: Some examples don't render a footer, but we keep this available for any example that
/// wants to show an always-visible hint.
#[allow(dead_code)]
pub const DEBUG_HELP_TEXT_ONELINE: &str =
    "M:Margins | P:Padding | B:Borders | C:Content | R:ClipRects | G:Gaps | O:Origins | T:Text | D:All | F:Profiling | ESC:Exit";

/// Handles shared debug keybinds for examples.
///
/// Returns `true` if the event was handled (and callers should early-return),
/// otherwise `false`.
pub fn handle_debug_keybinds(event: &WindowEvent, debug_options: &mut DebugOptions) -> bool {
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
        winit::keyboard::KeyCode::KeyR => {
            debug_options.show_clip_rects = !debug_options.show_clip_rects;
            println!("Clip rects: {}", debug_options.show_clip_rects);
            true
        }
        winit::keyboard::KeyCode::KeyG => {
            debug_options.show_gaps = !debug_options.show_gaps;
            println!("Gaps: {}", debug_options.show_gaps);
            true
        }
        winit::keyboard::KeyCode::KeyO => {
            debug_options.show_transform_origins = !debug_options.show_transform_origins;
            println!(
                "Transform origins: {}",
                debug_options.show_transform_origins
            );
            true
        }
        winit::keyboard::KeyCode::KeyT => {
            debug_options.show_text_bounds = !debug_options.show_text_bounds;
            println!("Text bounds: {}", debug_options.show_text_bounds);
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
