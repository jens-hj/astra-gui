//! Drag value component for interactive UI
//!
//! Provides a draggable number input field similar to egui's DragValue.
//! Users can drag left/right to adjust the value, or click to enter text input mode.

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, HorizontalAlign, Layout, Node, NodeId, Shape,
    Size, Spacing, Stroke, Style, TextContent, Transition, VerticalAlign,
};
use astra_gui_wgpu::{EventDispatcher, InputState, InteractionEvent, Key, NamedKey, TargetedEvent};
use std::ops::RangeInclusive;

use crate::{text_input, text_input_update, TextInputStyle};

/// Visual styling for a drag value widget
#[derive(Debug, Clone)]
pub struct DragValueStyle {
    /// Background color when idle
    pub idle_color: Color,
    /// Background color when hovered
    pub hover_color: Color,
    /// Background color when being dragged
    pub active_color: Color,
    /// Background color when disabled
    pub disabled_color: Color,

    /// Border color when idle
    pub idle_border_color: Color,
    /// Border color when hovered
    pub hover_border_color: Color,
    /// Border color when being dragged
    pub active_border_color: Color,
    /// Border color when disabled
    pub disabled_border_color: Color,

    /// Border width
    pub border_width: f32,

    /// Text color
    pub text_color: Color,
    /// Text color when disabled
    pub disabled_text_color: Color,

    /// Internal padding
    pub padding: Spacing,
    /// Corner radius for rounded corners
    pub border_radius: f32,
    /// Font size
    pub font_size: f32,

    /// Minimum width of the widget
    pub min_width: f32,

    /// Number of decimal places to show
    pub precision: usize,

    /// Text input style (used when focused)
    pub text_input_style: TextInputStyle,
}

impl Default for DragValueStyle {
    fn default() -> Self {
        let min_width = 80.0;
        let mut text_input_style = TextInputStyle::default();
        text_input_style.text_align = HorizontalAlign::Center;
        text_input_style.width = min_width;

        Self {
            idle_color: mocha::SURFACE0,
            hover_color: mocha::SURFACE1,
            active_color: mocha::SURFACE2,
            disabled_color: mocha::SURFACE0.with_alpha(0.8),

            idle_border_color: mocha::LAVENDER,
            hover_border_color: mocha::MAUVE,
            active_border_color: mocha::LAVENDER,
            disabled_border_color: mocha::SURFACE2,

            border_width: 2.0,

            text_color: mocha::TEXT,
            disabled_text_color: mocha::SUBTEXT0,

            padding: Spacing::symmetric(Size::lpx(10.0), Size::lpx(8.0)),
            border_radius: 8.0,
            font_size: 20.0,
            min_width,
            precision: 2,

            text_input_style,
        }
    }
}

impl DragValueStyle {
    /// Set the precision (number of decimal places)
    pub fn with_precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Set the minimum width
    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self.text_input_style.width = min_width;
        self
    }

    /// Set the font size
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
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

/// Parse a string to an f32 value
fn parse_value(text: &str) -> Option<f32> {
    text.trim().parse::<f32>().ok()
}

/// Create a drag value node
///
/// The drag value displays a number that can be adjusted by dragging left/right,
/// or edited directly by clicking to enter text input mode.
///
/// # Arguments
/// * `id` - Unique identifier for the drag value (used for event targeting)
/// * `value` - Current numeric value
/// * `focused` - Whether the widget is in text input mode
/// * `disabled` - Whether the widget is disabled
/// * `style` - Visual styling configuration
/// * `text_buffer` - Text buffer for text input mode
/// * `cursor_pos` - Cursor position in text buffer
/// * `selection` - Optional text selection range
/// * `measurer` - Text measurer for layout calculations
/// * `event_dispatcher` - Event dispatcher for cursor blink management
///
/// # Returns
/// A configured `Node` representing the drag value widget
pub fn drag_value(
    id: impl Into<String>,
    value: f32,
    focused: bool,
    disabled: bool,
    style: &DragValueStyle,
    text_buffer: &str,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    measurer: &mut impl astra_gui::ContentMeasurer,
    event_dispatcher: &mut EventDispatcher,
) -> Node {
    let id_string = id.into();

    // If focused, use text_input with center alignment
    if focused {
        return text_input(
            &id_string,
            text_buffer,
            "", // no placeholder
            focused,
            disabled,
            &style.text_input_style,
            cursor_pos,
            selection,
            measurer,
            event_dispatcher,
        );
    }

    // Otherwise, return drag-enabled display
    let display_text = format_value(value, style.precision);

    Node::new()
        .with_id(NodeId::new(format!("{}_container", id_string)))
        .with_width(Size::lpx(style.min_width))
        .with_height(Size::lpx(style.font_size + style.padding.get_vertical()))
        .with_padding(style.padding)
        .with_layout_direction(Layout::Stack)
        .with_shape(Shape::rect())
        .with_style(Style {
            fill_color: Some(style.idle_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.border_width),
                style.idle_border_color,
            )),
            corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                style.border_radius,
            ))),
            ..Default::default()
        })
        .with_hover_style(Style {
            fill_color: Some(style.hover_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.border_width),
                style.hover_border_color,
            )),
            ..Default::default()
        })
        .with_active_style(Style {
            fill_color: Some(style.active_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.border_width),
                style.active_border_color,
            )),
            ..Default::default()
        })
        .with_disabled_style(Style {
            fill_color: Some(style.disabled_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.border_width),
                style.disabled_border_color,
            )),
            ..Default::default()
        })
        .with_disabled(disabled)
        .with_transition(Transition::quick())
        .with_children(vec![
            // Text display
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_content(Content::Text(TextContent {
                    text: display_text,
                    font_size: Size::lpx(style.font_size),
                    color: style.text_color,
                    h_align: HorizontalAlign::Center,
                    v_align: VerticalAlign::Center,
                }))
                .with_style(Style {
                    text_color: Some(style.text_color),
                    ..Default::default()
                })
                .with_disabled_style(Style {
                    text_color: Some(style.disabled_text_color),
                    ..Default::default()
                })
                .with_disabled(disabled)
                .with_transition(Transition::quick()),
            // Hitbox for drag detection
            Node::new()
                .with_id(NodeId::new(format!("{}_hitbox", id_string)))
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_disabled(disabled),
        ])
}

/// Update drag value from events
///
/// Call this each frame with the events to update the value based on
/// drag interactions or text input.
///
/// # Arguments
/// * `id` - The ID of the drag value widget
/// * `value` - Current value (will be modified)
/// * `text_buffer` - Text buffer for text input mode (will be modified)
/// * `cursor_pos` - Cursor position in text buffer (will be modified)
/// * `selection` - Optional text selection range (will be modified)
/// * `focused` - Whether widget is in text input mode (will be modified)
/// * `drag_accumulator` - Continuous accumulator for drag movements (will be modified)
/// * `events` - Slice of targeted events from this frame
/// * `input_state` - Current input state
/// * `event_dispatcher` - EventDispatcher for focus management
/// * `range` - Optional range to clamp values
/// * `speed` - Base drag speed (pixels to value multiplier)
/// * `step` - Optional step size for snapping
///
/// # Returns
/// `true` if the value was changed, `false` otherwise
pub fn drag_value_update(
    id: &str,
    value: &mut f32,
    text_buffer: &mut String,
    cursor_pos: &mut usize,
    selection: &mut Option<(usize, usize)>,
    focused: &mut bool,
    drag_accumulator: &mut f32,
    events: &[TargetedEvent],
    input_state: &InputState,
    event_dispatcher: &mut EventDispatcher,
    range: Option<RangeInclusive<f32>>,
    speed: f32,
    step: Option<f32>,
) -> bool {
    let hitbox_id = format!("{}_hitbox", id);
    let container_id = format!("{}_container", id);
    let node_id = NodeId::new(id);

    // Check if this widget is actually focused according to the event dispatcher
    let is_focused = event_dispatcher
        .focused_node()
        .map(|fid| fid == &node_id)
        .unwrap_or(false);

    // Sync local focused state with event dispatcher
    *focused = is_focused;

    // First, check for drag events (works whether focused or not)
    let mut value_changed = false;
    let mut was_dragged = false;

    for event in events {
        let target_str = event.target.as_str();

        if target_str != hitbox_id && target_str != container_id {
            continue;
        }

        match &event.event {
            InteractionEvent::DragStart { .. } => {
                // Initialize accumulator with current value when drag starts
                *drag_accumulator = *value;
                // Unfocus if currently focused (user is starting to drag)
                if is_focused {
                    *focused = false;
                    event_dispatcher.set_focus(None);
                }
            }
            InteractionEvent::DragMove { delta, .. } => {
                was_dragged = true;
                // Calculate value change from horizontal drag
                let mut drag_speed = speed;

                // Apply speed modifiers
                if input_state.shift_held {
                    drag_speed *= 0.1; // Slower, more precise
                }
                if input_state.ctrl_held {
                    drag_speed *= 10.0; // Faster
                }

                let delta_value = delta.x * drag_speed;

                // Update the continuous accumulator
                *drag_accumulator += delta_value;

                // Apply range clamping to accumulator
                if let Some(ref value_range) = range {
                    *drag_accumulator =
                        drag_accumulator.clamp(*value_range.start(), *value_range.end());
                }

                // Calculate the stepped value from the accumulator
                let mut new_value = *drag_accumulator;

                if let Some(step_size) = step {
                    if step_size > 0.0 {
                        if let Some(ref value_range) = range {
                            let steps_from_start =
                                ((new_value - value_range.start()) / step_size).round();
                            new_value = value_range.start() + steps_from_start * step_size;
                            new_value = new_value.clamp(*value_range.start(), *value_range.end());
                        } else {
                            // Snap to nearest step from 0
                            new_value = (new_value / step_size).round() * step_size;
                        }
                    }
                }

                // Only update the exposed value if it changed
                if (*value - new_value).abs() > f32::EPSILON {
                    *value = new_value;
                    value_changed = true;
                }
            }
            InteractionEvent::DragEnd { .. } => {
                // Only enter text input mode if we didn't actually drag
                if !was_dragged {
                    *focused = true;
                    *text_buffer = format_value(*value, 6); // Use high precision for editing
                    *cursor_pos = text_buffer.len(); // Place cursor at end
                    *selection = None;
                    event_dispatcher.set_focus(Some(node_id.clone()));
                }
            }
            _ => {}
        }
    }

    // If we dragged, we're done (don't process text input)
    if was_dragged {
        return value_changed;
    }

    // When focused, delegate to text_input_update
    if *focused {
        // Let text_input_update handle all the text editing
        let text_changed = text_input_update(
            id,
            text_buffer,
            cursor_pos,
            selection,
            events,
            input_state,
            event_dispatcher,
        );

        // Check if we should parse and apply the value
        let enter_pressed = input_state
            .keys_just_pressed
            .iter()
            .any(|key| matches!(key, Key::Named(NamedKey::Enter)));

        let escape_pressed = input_state
            .keys_just_pressed
            .iter()
            .any(|key| matches!(key, Key::Named(NamedKey::Escape)));

        if enter_pressed {
            // Parse text and update value
            let changed = if let Some(new_value) = parse_value(text_buffer) {
                let mut clamped_value = new_value;

                // Apply range clamping
                if let Some(ref value_range) = range {
                    clamped_value = clamped_value.clamp(*value_range.start(), *value_range.end());
                }

                // Apply step snapping
                if let Some(step_size) = step {
                    if step_size > 0.0 {
                        if let Some(ref value_range) = range {
                            let steps_from_start =
                                ((clamped_value - value_range.start()) / step_size).round();
                            clamped_value = value_range.start() + steps_from_start * step_size;
                            clamped_value =
                                clamped_value.clamp(*value_range.start(), *value_range.end());
                        } else {
                            // Snap to nearest step from 0
                            clamped_value = (clamped_value / step_size).round() * step_size;
                        }
                    }
                }

                *value = clamped_value;
                *drag_accumulator = clamped_value; // Reset accumulator to new value
                true
            } else {
                false
            };

            // Unfocus after accepting the value
            *focused = false;
            event_dispatcher.set_focus(None);
            return changed;
        } else if escape_pressed {
            // text_input_update already handled unfocusing
            return false;
        }

        return text_changed;
    }

    // Not focused and no text changes
    false
}
