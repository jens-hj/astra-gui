//! Toggle (switch) component for interactive UI
//!
//! Provides an iOS-style toggle switch with smooth animations.

use astra_gui::{
    catppuccin::mocha, Color, Component, CornerShape, Layout, Node, NodeId, Size, Spacing, Style,
    Transition, UiContext,
};
use astra_gui_macros::WithBuilders;
use astra_gui_wgpu::{InteractionEvent, TargetedEvent};

/// Visual styling for a toggle switch
#[derive(Debug, Clone, WithBuilders)]
pub struct ToggleStyle {
    /// Background color when toggle is off
    pub off_color: Color,
    /// Background color when toggle is on
    pub on_color: Color,
    /// Color of the sliding knob
    pub knob_color: Color,
    /// Width of the track
    pub track_width: f32,
    /// Height of the track
    pub track_height: f32,
    /// Diameter of the knob
    pub knob_width: f32,
    /// Margin between knob and track edges
    pub knob_margin: f32,
}

impl Default for ToggleStyle {
    fn default() -> Self {
        Self {
            off_color: mocha::SURFACE0,
            on_color: mocha::LAVENDER,
            knob_color: mocha::BASE,
            track_width: 50.0,
            track_height: 30.0,
            knob_width: 26.0,
            knob_margin: 2.0,
        }
    }
}

/// A toggle switch component
///
/// # Example
///
/// ```ignore
/// Toggle::new(enabled)
///     .on_toggle(|new_value| println!("Toggled: {}", new_value))
///     .node(&mut ctx)
/// ```
pub struct Toggle {
    value: bool,
    disabled: bool,
    style: ToggleStyle,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
}

impl Toggle {
    /// Create a new toggle with the given initial value
    pub fn new(value: bool) -> Self {
        Toggle {
            value,
            disabled: false,
            style: ToggleStyle::default(),
            on_toggle: None,
        }
    }

    /// Set whether the toggle is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a custom style for the toggle
    pub fn with_style(mut self, style: ToggleStyle) -> Self {
        self.style = style;
        self
    }

    /// Set a callback to be called when the toggle is clicked
    ///
    /// The callback receives the new value (opposite of current value)
    pub fn on_toggle(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }
}

impl Component for Toggle {
    fn node(mut self, ctx: &mut UiContext) -> Node {
        // Generate unique IDs for the toggle and its knob
        let id = ctx.generate_id("toggle");
        let knob_id = format!("{}_knob", id);

        // Check for click events from last frame and fire callback
        if !self.disabled {
            let was_clicked = ctx.was_clicked(&id) || ctx.was_clicked(&knob_id);
            if was_clicked {
                if let Some(ref mut on_toggle) = self.on_toggle {
                    on_toggle(!self.value);
                }
            }
        }

        let knob_offset_x = if self.value {
            self.style.track_width - self.style.knob_width - self.style.knob_margin * 2.0
        } else {
            0.0
        };

        // Track (background)
        Node::new()
            .with_id(NodeId::new(&id))
            .with_width(Size::lpx(self.style.track_width))
            .with_height(Size::lpx(self.style.track_height))
            .with_layout_direction(Layout::Horizontal)
            .with_padding(Spacing::all(Size::lpx(self.style.knob_margin)))
            .with_style(Style {
                fill_color: Some(if self.value {
                    self.style.on_color
                } else {
                    self.style.off_color
                }),
                corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                    self.style.track_height / 2.0,
                ))),
                opacity: Some(1.0),
                ..Default::default()
            })
            .with_hover_style(Style {
                fill_color: Some(mocha::SURFACE1),
                opacity: Some(0.9),
                ..Default::default()
            })
            .with_active_style(Style {
                opacity: Some(0.7),
                ..Default::default()
            })
            .with_disabled_style(Style {
                fill_color: Some(mocha::SURFACE0),
                opacity: Some(0.5),
                ..Default::default()
            })
            .with_disabled(self.disabled)
            .with_transition(Transition::quick())
            .with_child(
                // Knob (sliding circle with smooth offset animation)
                Node::new()
                    .with_id(NodeId::new(&knob_id))
                    .with_width(Size::lpx(self.style.knob_width))
                    .with_height(Size::Fill)
                    .with_style(Style {
                        fill_color: Some(self.style.knob_color),
                        corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                            self.style.knob_width / 2.0,
                        ))),
                        translation_x: Some(astra_gui::Size::Logical(knob_offset_x)),
                        ..Default::default()
                    })
                    .with_transition(Transition::quick()),
            )
    }
}

/// Check if a toggle with the given ID was clicked this frame
///
/// # Arguments
/// * `toggle_id` - The ID of the toggle to check
/// * `events` - Slice of targeted events from this frame
///
/// # Returns
/// `true` if the toggle was clicked, `false` otherwise
pub fn toggle_clicked(toggle_id: &str, events: &[TargetedEvent]) -> bool {
    let knob_id = format!("{}_knob", toggle_id);
    events.iter().any(|e| {
        matches!(e.event, InteractionEvent::Click { .. })
            && (e.target.as_str() == toggle_id || e.target.as_str() == knob_id)
    })
}

/// Create a toggle switch node
///
/// This is a backward-compatible function that wraps the new `Toggle` struct.
/// For new code, prefer using `Toggle::new()` with the builder pattern.
///
/// # Arguments
/// * `id` - Unique identifier for the toggle (used for event targeting)
/// * `value` - Current state of the toggle (true = on, false = off)
/// * `disabled` - Whether the toggle is disabled
/// * `style` - Visual styling configuration
///
/// # Returns
/// A configured `Node` representing the toggle switch with automatic state transitions
#[deprecated(
    since = "0.8.0",
    note = "Use Toggle::new() with the builder pattern instead"
)]
pub fn toggle(id: impl Into<String>, value: bool, disabled: bool, style: &ToggleStyle) -> Node {
    let id_str = id.into();
    let knob_offset_x = if value {
        style.track_width - style.knob_width - style.knob_margin * 2.0
    } else {
        0.0
    };

    // Track (background)
    Node::new()
        .with_id(NodeId::new(id_str.clone()))
        .with_width(Size::lpx(style.track_width))
        .with_height(Size::lpx(style.track_height))
        .with_layout_direction(Layout::Horizontal)
        .with_padding(Spacing::all(Size::lpx(style.knob_margin)))
        .with_style(Style {
            fill_color: Some(if value {
                style.on_color
            } else {
                style.off_color
            }),
            corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                style.track_height / 2.0,
            ))),
            opacity: Some(1.0),
            ..Default::default()
        })
        .with_hover_style(Style {
            fill_color: Some(mocha::SURFACE1),
            opacity: Some(0.9),
            ..Default::default()
        })
        .with_active_style(Style {
            opacity: Some(0.7),
            ..Default::default()
        })
        .with_disabled_style(Style {
            fill_color: Some(mocha::SURFACE0),
            opacity: Some(0.5),
            ..Default::default()
        })
        .with_disabled(disabled)
        .with_transition(Transition::quick())
        .with_child(
            // Knob (sliding circle with smooth offset animation)
            Node::new()
                .with_id(NodeId::new(format!("{}_knob", id_str)))
                .with_width(Size::lpx(style.knob_width))
                .with_height(Size::Fill)
                .with_style(Style {
                    fill_color: Some(style.knob_color),
                    corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                        style.knob_width / 2.0,
                    ))),
                    translation_x: Some(astra_gui::Size::Logical(knob_offset_x)),
                    ..Default::default()
                })
                .with_transition(Transition::quick()),
        )
}
