//! Combined slider with drag value component
//!
//! Provides an egui-style slider with an integrated drag value field on the right.

use astra_gui::{Component, Layout, Node, Size, UiContext, VerticalAlign};
use std::ops::RangeInclusive;

use crate::{
    drag_value, drag_value_update, slider, slider_drag, DragValue, DragValueStyle, Slider,
    SliderStyle,
};

/// Combined slider with drag value component
///
/// Creates a horizontal layout with a slider on the left and a drag value field on the right.
/// This provides an egui-style interface where users can either drag the slider or
/// directly edit the numeric value.
///
/// # Example
///
/// ```ignore
/// SliderWithValue::new(&mut value, 0.0..=100.0)
///     .step(1.0)
///     .speed(0.5)
///     .on_change(|v| println!("Value changed to: {}", v))
///     .build(&mut ctx);
/// ```
pub struct SliderWithValue<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    step: Option<f32>,
    speed: f32,
    gap: f32,
    disabled: bool,
    slider_style: SliderStyle,
    value_style: DragValueStyle,
    on_change: Option<Box<dyn FnMut(f32) + 'a>>,
}

impl<'a> SliderWithValue<'a> {
    /// Create a new slider with value bound to a mutable float reference
    ///
    /// # Arguments
    /// * `value` - Mutable reference to the value being controlled
    /// * `range` - Range of valid values for both slider and drag value
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        SliderWithValue {
            value,
            range,
            step: None,
            speed: 0.1,
            gap: 8.0,
            disabled: false,
            slider_style: SliderStyle::default(),
            value_style: DragValueStyle::default(),
            on_change: None,
        }
    }

    /// Set the step size for value snapping (applies to both slider and drag value)
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the drag speed for the drag value field (pixels to value multiplier)
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set the gap between slider and drag value field
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set whether the component is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a custom style for the slider
    pub fn with_slider_style(mut self, style: SliderStyle) -> Self {
        self.slider_style = style;
        self
    }

    /// Set a custom style for the drag value
    pub fn with_value_style(mut self, style: DragValueStyle) -> Self {
        self.value_style = style;
        self
    }

    /// Set both slider and drag value styles at once
    pub fn with_styles(mut self, slider_style: SliderStyle, value_style: DragValueStyle) -> Self {
        self.slider_style = slider_style;
        self.value_style = value_style;
        self
    }

    /// Set a callback to be called when the value changes (from either component)
    pub fn on_change(mut self, f: impl FnMut(f32) + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Build the slider with value node
    ///
    /// Note: This is not implementing Component trait because we need lifetime 'a
    /// for the mutable reference to the value.
    pub fn build(mut self, ctx: &mut UiContext) -> Node {
        // Track the original value to detect changes
        let original_value = *self.value;

        // Build slider component
        let mut slider_builder = Slider::new(*self.value, self.range.clone())
            .disabled(self.disabled)
            .with_style(self.slider_style);

        if let Some(step) = self.step {
            slider_builder = slider_builder.step(step);
        }

        // Capture value changes from slider
        let value_ptr = self.value as *mut f32;
        slider_builder = slider_builder.on_change(move |new_val| {
            // SAFETY: We know the value is still valid during build
            unsafe {
                *value_ptr = new_val;
            }
        });

        let slider_node = slider_builder.node(ctx);

        // Build drag value component
        let mut drag_value_builder = DragValue::new(self.value)
            .range(self.range)
            .speed(self.speed)
            .disabled(self.disabled)
            .with_style(self.value_style);

        if let Some(step) = self.step {
            drag_value_builder = drag_value_builder.step(step);
        }

        let drag_value_node = drag_value_builder.build(ctx);

        // Fire the on_change callback if value changed from either component
        if let Some(ref mut callback) = self.on_change {
            if (*self.value - original_value).abs() > f32::EPSILON {
                callback(*self.value);
            }
        }

        // Build the horizontal layout combining both components
        Node::new()
            .with_layout_direction(Layout::Horizontal)
            .with_gap(Size::lpx(self.gap))
            .with_children(vec![
                Node::new()
                    .with_height(Size::Fill)
                    .with_v_align(VerticalAlign::Center)
                    .with_child(slider_node),
                drag_value_node,
            ])
    }
}

// =============================================================================
// DEPRECATED API - For backwards compatibility
// =============================================================================

/// Combined slider with drag value component (DEPRECATED)
///
/// Creates a horizontal layout with a slider on the left and a drag value field on the right.
///
/// # Deprecated
/// Use `SliderWithValue::new(...).build(ctx)` instead.
#[deprecated(
    since = "0.2.0",
    note = "Use SliderWithValue::new(...).build(ctx) instead"
)]
#[allow(clippy::too_many_arguments)]
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
    event_dispatcher: &mut astra_gui_wgpu::EventDispatcher,
) -> Node {
    Node::new()
        .with_layout_direction(Layout::Horizontal)
        .with_gap(Size::lpx(8.0))
        .with_children(vec![
            Node::new()
                .with_height(Size::Fill)
                .with_v_align(VerticalAlign::Center)
                .with_child(slider(
                    slider_id,
                    value,
                    range.clone(),
                    disabled,
                    slider_style,
                )),
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

/// Update combined slider with drag value from events (DEPRECATED)
///
/// Handles both slider dragging and drag value interactions.
///
/// # Deprecated
/// Use `SliderWithValue::new(...).on_change(...).build(ctx)` instead.
/// The new API handles updates automatically.
#[deprecated(
    since = "0.2.0",
    note = "Use SliderWithValue::new(...).on_change(...).build(ctx) instead"
)]
#[allow(clippy::too_many_arguments)]
pub fn slider_with_value_update(
    slider_id: &str,
    value_id: &str,
    value: &mut f32,
    text_buffer: &mut String,
    cursor_pos: &mut usize,
    selection: &mut Option<(usize, usize)>,
    focused: &mut bool,
    drag_accumulator: &mut f32,
    events: &[astra_gui_wgpu::TargetedEvent],
    input_state: &astra_gui_wgpu::InputState,
    event_dispatcher: &mut astra_gui_wgpu::EventDispatcher,
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
