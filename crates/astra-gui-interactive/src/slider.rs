//! Slider component for interactive UI
//!
//! Provides a draggable slider for selecting values within a range.

use astra_gui::{
    catppuccin::mocha, Color, Component, CornerShape, Layout, Node, NodeId, Size, Style,
    Transition, Translation, UiContext,
};
use astra_gui_macros::WithBuilders;
use astra_gui_wgpu::{InteractionEvent, TargetedEvent};
use std::ops::RangeInclusive;

/// Visual styling for a slider
#[derive(Debug, Clone, WithBuilders)]
pub struct SliderStyle {
    /// Color of the track (unfilled portion)
    pub track_color: Color,
    /// Color of the filled portion of the track
    pub filled_color: Color,
    /// Color of the draggable thumb
    pub thumb_color: Color,
    /// Color of the thumb when hovered
    pub thumb_hover_color: Color,
    /// Color of the thumb when being dragged
    pub thumb_active_color: Color,
    /// Width of the slider track
    pub track_width: f32,
    /// Height of the slider track
    pub track_height: f32,
    /// Diameter of the thumb
    pub thumb_size: f32,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_color: mocha::SURFACE0,
            filled_color: mocha::LAVENDER,
            thumb_color: mocha::BASE,
            thumb_hover_color: mocha::SURFACE0,
            thumb_active_color: mocha::MAUVE.with_alpha(0.0),
            track_width: 200.0,
            track_height: 30.0,
            thumb_size: 26.0,
        }
    }
}

/// A slider component for selecting values within a range
///
/// # Example
///
/// ```ignore
/// Slider::new(value, 0.0..=100.0)
///     .on_change(|new_value| println!("Value: {}", new_value))
///     .node(&mut ctx)
/// ```
pub struct Slider {
    value: f32,
    range: RangeInclusive<f32>,
    step: Option<f32>,
    disabled: bool,
    style: SliderStyle,
    on_change: Option<Box<dyn FnMut(f32)>>,
}

impl Slider {
    /// Create a new slider with the given value and range
    pub fn new(value: f32, range: RangeInclusive<f32>) -> Self {
        Slider {
            value,
            range,
            step: None,
            disabled: false,
            style: SliderStyle::default(),
            on_change: None,
        }
    }

    /// Set the step size for value snapping
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set whether the slider is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a custom style for the slider
    pub fn with_style(mut self, style: SliderStyle) -> Self {
        self.style = style;
        self
    }

    /// Set a callback to be called when the slider value changes
    pub fn on_change(mut self, f: impl FnMut(f32) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Calculate new value from local position
    fn calculate_value_from_position(&self, local_x: f32, zoom: f32) -> f32 {
        let adjusted_x = local_x / zoom;
        let usable_width = self.style.track_width - self.style.thumb_size;
        let adjusted_x = (adjusted_x - self.style.thumb_size / 2.0).clamp(0.0, usable_width);
        let percentage = if usable_width > 0.0 {
            (adjusted_x / usable_width).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let range_size = self.range.end() - self.range.start();
        let mut new_value = self.range.start() + range_size * percentage;

        // Apply step if provided
        if let Some(step_size) = self.step {
            if step_size > 0.0 {
                // Snap to range boundaries if we're very close
                if percentage < 0.02 {
                    new_value = *self.range.start();
                } else if percentage > 0.98 {
                    new_value = *self.range.end();
                } else {
                    let steps_from_start = ((new_value - self.range.start()) / step_size).round();
                    new_value = self.range.start() + steps_from_start * step_size;
                    new_value = new_value.clamp(*self.range.start(), *self.range.end());
                }
            }
        }

        new_value
    }
}

impl Component for Slider {
    fn node(mut self, ctx: &mut UiContext) -> Node {
        // Generate unique ID for the slider hitbox
        let id = ctx.generate_id("slider");
        let hitbox_id = format!("{}_hitbox", id);

        // Check for drag events from last frame and fire callback
        if !self.disabled {
            for event in ctx.events() {
                if event.target.as_str() != hitbox_id {
                    continue;
                }

                match &event.event {
                    InteractionEvent::Click { .. }
                    | InteractionEvent::DragStart { .. }
                    | InteractionEvent::DragMove { .. } => {
                        let new_value =
                            self.calculate_value_from_position(event.local_position.x, event.zoom);

                        if (self.value - new_value).abs() > f32::EPSILON {
                            if let Some(ref mut on_change) = self.on_change {
                                on_change(new_value);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Calculate percentage (0.0 to 1.0)
        let range_size = self.range.end() - self.range.start();
        let percentage = if range_size > 0.0 {
            ((self.value - self.range.start()) / range_size).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Calculate thumb position
        let thumb_inset = (self.style.track_height - self.style.thumb_size) / 2.0;
        let usable_width = self.style.track_width
            - self.style.thumb_size
            - (self.style.track_height - self.style.thumb_size) * 2.0;
        let thumb_offset_x = (usable_width - (self.style.thumb_size - self.style.track_height))
            * percentage
            + thumb_inset;

        // Calculate filled width
        let filled_width = thumb_offset_x + self.style.track_height - thumb_inset;

        // Create the slider node
        Node::new()
            .with_width(Size::lpx(self.style.track_width))
            .with_height(Size::lpx(
                self.style.thumb_size.max(self.style.track_height),
            ))
            .with_layout_direction(Layout::Stack)
            .with_children(vec![
                // Track background (unfilled)
                Node::new()
                    .with_width(Size::lpx(self.style.track_width))
                    .with_height(Size::lpx(self.style.track_height))
                    .with_style(Style {
                        fill_color: Some(self.style.track_color),
                        corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                            self.style.track_height / 2.0,
                        ))),
                        ..Default::default()
                    })
                    .with_disabled(self.disabled)
                    .with_transition(Transition::quick()),
                // Filled portion of track
                Node::new()
                    .with_width(Size::lpx(filled_width))
                    .with_height(Size::lpx(self.style.track_height))
                    .with_style(Style {
                        fill_color: Some(self.style.filled_color),
                        corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                            self.style.track_height / 2.0,
                        ))),
                        ..Default::default()
                    })
                    .with_disabled_style(Style {
                        fill_color: Some(mocha::SURFACE1),
                        ..Default::default()
                    })
                    .with_disabled(self.disabled)
                    .with_transition(Transition::quick()),
                // Thumb
                Node::new()
                    .with_width(Size::lpx(self.style.thumb_size))
                    .with_height(Size::lpx(self.style.thumb_size))
                    .with_translation(Translation::new(
                        astra_gui::Size::Logical(thumb_offset_x),
                        astra_gui::Size::Logical(thumb_inset),
                    ))
                    .with_style(Style {
                        fill_color: Some(self.style.thumb_color),
                        opacity: Some(1.0),
                        corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                            self.style.thumb_size / 2.0,
                        ))),
                        ..Default::default()
                    })
                    .with_hover_style(Style {
                        fill_color: Some(self.style.thumb_hover_color),
                        ..Default::default()
                    })
                    .with_active_style(Style {
                        fill_color: Some(self.style.thumb_active_color),
                        ..Default::default()
                    })
                    .with_disabled_style(Style {
                        opacity: Some(0.0),
                        ..Default::default()
                    })
                    .with_disabled(self.disabled)
                    .with_transition(Transition::quick()),
                // Hitbox node
                Node::new()
                    .with_id(NodeId::new(&hitbox_id))
                    .with_width(Size::Fill)
                    .with_height(Size::Fill)
                    .with_disabled(self.disabled),
            ])
    }
}

/// Update slider value from drag events
///
/// Call this each frame with the events to update the slider value based on
/// drag interactions.
///
/// # Arguments
/// * `slider_id` - The ID of the slider (thumb ID)
/// * `value` - Current slider value (will be modified if dragged)
/// * `range` - The valid range of values
/// * `events` - Slice of targeted events from this frame
/// * `style` - The slider style (needed for track width calculation)
/// * `step` - Optional step size. If provided, values will snap to multiples of this increment
///
/// # Returns
/// `true` if the value was changed, `false` otherwise
pub fn slider_drag(
    slider_id: &str,
    value: &mut f32,
    range: &RangeInclusive<f32>,
    events: &[TargetedEvent],
    style: &SliderStyle,
    step: Option<f32>,
) -> bool {
    let container_id = format!("{}_hitbox", slider_id);

    // Only handle events from container
    for event in events {
        let target_str = event.target.as_str();

        if target_str != container_id {
            continue;
        }

        match &event.event {
            InteractionEvent::Click { .. }
            | InteractionEvent::DragStart { .. }
            | InteractionEvent::DragMove { .. } => {
                // Divide by zoom to get logical coordinates
                let local_x = event.local_position.x / event.zoom;

                // Adjust for thumb half-width so clicking centers the thumb at cursor
                let usable_width = style.track_width - style.thumb_size;
                let adjusted_x = (local_x - style.thumb_size / 2.0).clamp(0.0, usable_width);
                let percentage = if usable_width > 0.0 {
                    (adjusted_x / usable_width).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let range_size = range.end() - range.start();
                let mut new_value = range.start() + range_size * percentage;

                // Apply step if provided
                if let Some(step_size) = step {
                    if step_size > 0.0 {
                        // Snap to range boundaries if we're very close
                        if percentage < 0.02 {
                            new_value = *range.start();
                        } else if percentage > 0.98 {
                            new_value = *range.end();
                        } else {
                            let steps_from_start =
                                ((new_value - range.start()) / step_size).round();
                            new_value = range.start() + steps_from_start * step_size;
                            new_value = new_value.clamp(*range.start(), *range.end());
                        }
                    }
                }

                if (*value - new_value).abs() > f32::EPSILON {
                    *value = new_value;
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

/// Check if a slider with the given ID is currently being hovered
pub fn slider_hovered(slider_id: &str, events: &[TargetedEvent]) -> bool {
    let container_id = format!("{}_container", slider_id);

    events.iter().any(|e| {
        matches!(e.event, InteractionEvent::Hover { .. })
            && (e.target.as_str() == slider_id || e.target.as_str() == container_id)
    })
}

/// Check if a slider with the given ID is currently being dragged
pub fn slider_dragging(slider_id: &str, events: &[TargetedEvent]) -> bool {
    let container_id = format!("{}_container", slider_id);

    events.iter().any(|e| {
        matches!(
            e.event,
            InteractionEvent::DragStart { .. } | InteractionEvent::DragMove { .. }
        ) && (e.target.as_str() == slider_id || e.target.as_str() == container_id)
    })
}

/// Create a slider node
///
/// This is a backward-compatible function that wraps the new `Slider` struct.
/// For new code, prefer using `Slider::new()` with the builder pattern.
///
/// # Arguments
/// * `id` - Unique identifier for the slider (used for event targeting)
/// * `value` - Current value (should be within the range)
/// * `range` - The valid range of values
/// * `disabled` - Whether the slider is disabled
/// * `style` - Visual styling configuration
///
/// # Returns
/// A configured `Node` representing the slider
#[deprecated(
    since = "0.8.0",
    note = "Use Slider::new() with the builder pattern instead"
)]
pub fn slider(
    id: impl Into<String>,
    value: f32,
    range: RangeInclusive<f32>,
    disabled: bool,
    style: &SliderStyle,
) -> Node {
    let id_str = id.into();

    // Calculate percentage (0.0 to 1.0)
    let range_size = range.end() - range.start();
    let percentage = if range_size > 0.0 {
        ((value - range.start()) / range_size).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Calculate thumb position
    let thumb_inset = (style.track_height - style.thumb_size) / 2.0;
    let usable_width =
        style.track_width - style.thumb_size - (style.track_height - style.thumb_size) * 2.0;
    let thumb_offset_x =
        (usable_width - (style.thumb_size - style.track_height)) * percentage + thumb_inset;

    // Calculate filled width
    let filled_width = thumb_offset_x + style.track_height - thumb_inset;

    Node::new()
        .with_width(Size::lpx(style.track_width))
        .with_height(Size::lpx(style.thumb_size.max(style.track_height)))
        .with_layout_direction(Layout::Stack)
        .with_children(vec![
            // Track background (unfilled)
            Node::new()
                .with_width(Size::lpx(style.track_width))
                .with_height(Size::lpx(style.track_height))
                .with_style(Style {
                    fill_color: Some(style.track_color),
                    corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                        style.track_height / 2.0,
                    ))),
                    ..Default::default()
                })
                .with_disabled(disabled)
                .with_transition(Transition::quick()),
            // Filled portion of track
            Node::new()
                .with_width(Size::lpx(filled_width))
                .with_height(Size::lpx(style.track_height))
                .with_style(Style {
                    fill_color: Some(style.filled_color),
                    corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                        style.track_height / 2.0,
                    ))),
                    ..Default::default()
                })
                .with_disabled_style(Style {
                    fill_color: Some(mocha::SURFACE1),
                    ..Default::default()
                })
                .with_disabled(disabled)
                .with_transition(Transition::quick()),
            // Thumb
            Node::new()
                .with_width(Size::lpx(style.thumb_size))
                .with_height(Size::lpx(style.thumb_size))
                .with_translation(Translation::new(
                    astra_gui::Size::Logical(thumb_offset_x),
                    astra_gui::Size::Logical(thumb_inset),
                ))
                .with_style(Style {
                    fill_color: Some(style.thumb_color),
                    opacity: Some(1.0),
                    corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                        style.thumb_size / 2.0,
                    ))),
                    ..Default::default()
                })
                .with_hover_style(Style {
                    fill_color: Some(style.thumb_hover_color),
                    ..Default::default()
                })
                .with_active_style(Style {
                    fill_color: Some(style.thumb_active_color),
                    ..Default::default()
                })
                .with_disabled_style(Style {
                    opacity: Some(0.0),
                    ..Default::default()
                })
                .with_disabled(disabled)
                .with_transition(Transition::quick()),
            // Hitbox node
            Node::new()
                .with_id(NodeId::new(format!("{}_hitbox", id_str)))
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_disabled(disabled),
        ])
}
