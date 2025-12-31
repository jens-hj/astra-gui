//! Combined slider with drag value component
//!
//! Provides an egui-style slider with an integrated drag value field on the right.

use astra_gui::{Layout, Node, Size};
use astra_gui_wgpu::{EventDispatcher, InputState, TargetedEvent};
use std::ops::RangeInclusive;

use crate::{drag_value, drag_value_update, slider, slider_drag, DragValueStyle, SliderStyle};

/// Combined slider with drag value component
///
/// Creates a horizontal layout with a slider on the left and a drag value field on the right.
///
/// # Arguments
/// * `slider_id` - Unique identifier for the slider
/// * `value_id` - Unique identifier for the drag value
/// * `value` - Current value
/// * `range` - Range of valid values
/// * `focused` - Whether the drag value field is focused
/// * `disabled` - Whether the components are disabled
/// * `slider_style` - Visual styling for the slider
/// * `value_style` - Visual styling for the drag value
/// * `text_buffer` - Text buffer for drag value text input
/// * `cursor_pos` - Cursor position in text buffer
/// * `selection` - Optional text selection range
/// * `measurer` - Text measurer for layout calculations
/// * `event_dispatcher` - Event dispatcher for focus management
///
/// # Returns
/// A horizontal layout node containing the slider and drag value
pub fn slider_with_value(
    slider_id: impl Into<String>,
    value_id: impl Into<String>,
    value: f32,
    range: RangeInclusive<f32>,
    focused: bool,
    disabled: bool,
    slider_style: &SliderStyle,
    value_style: &DragValueStyle,
    text_buffer: &str,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    measurer: &mut impl astra_gui::ContentMeasurer,
    event_dispatcher: &mut EventDispatcher,
) -> Node {
    Node::new()
        .with_layout_direction(Layout::Horizontal)
        .with_gap(Size::lpx(8.0))
        .with_children(vec![
            slider(slider_id, value, range.clone(), disabled, slider_style),
            drag_value(
                value_id,
                value,
                focused,
                disabled,
                value_style,
                text_buffer,
                cursor_pos,
                selection,
                measurer,
                event_dispatcher,
            ),
        ])
}

/// Update combined slider with drag value from events
///
/// Handles both slider dragging and drag value interactions.
///
/// # Arguments
/// * `slider_id` - The ID of the slider
/// * `value_id` - The ID of the drag value
/// * `value` - Current value (will be modified)
/// * `text_buffer` - Text buffer for drag value text input (will be modified)
/// * `cursor_pos` - Cursor position in text buffer (will be modified)
/// * `selection` - Optional text selection range (will be modified)
/// * `focused` - Whether drag value is in text input mode (will be modified)
/// * `drag_accumulator` - Continuous accumulator for drag value movements (will be modified)
/// * `events` - Slice of targeted events from this frame
/// * `input_state` - Current input state
/// * `event_dispatcher` - EventDispatcher for focus management
/// * `range` - Range to clamp values
/// * `speed` - Base drag speed for drag value (pixels to value multiplier)
/// * `step` - Optional step size for snapping
///
/// # Returns
/// `true` if the value was changed, `false` otherwise
pub fn slider_with_value_update(
    slider_id: &str,
    value_id: &str,
    value: &mut f32,
    text_buffer: &mut String,
    cursor_pos: &mut usize,
    selection: &mut Option<(usize, usize)>,
    focused: &mut bool,
    drag_accumulator: &mut f32,
    events: &[TargetedEvent],
    input_state: &InputState,
    event_dispatcher: &mut EventDispatcher,
    range: RangeInclusive<f32>,
    speed: f32,
    step: Option<f32>,
) -> bool {
    let mut changed = false;

    // Check slider drag (use default style)
    if slider_drag(
        slider_id,
        value,
        &range,
        events,
        &SliderStyle::default(),
        step,
    ) {
        *drag_accumulator = *value; // Sync accumulator with slider value
        changed = true;
    }

    // Check drag value update
    if drag_value_update(
        value_id,
        value,
        text_buffer,
        cursor_pos,
        selection,
        focused,
        drag_accumulator,
        events,
        input_state,
        event_dispatcher,
        Some(range),
        speed,
        step,
    ) {
        changed = true;
    }

    changed
}
