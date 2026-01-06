//! Button component for interactive UI
//!
//! Provides a clickable button widget with hover and press states.

use astra_gui::{
    catppuccin::mocha, Color, Component, Content, CornerShape, HorizontalAlign, Node, NodeId, Size,
    Spacing, Stroke, Style, TextContent, Transition, UiContext, VerticalAlign,
};
use astra_gui_macros::WithBuilders;
use astra_gui_wgpu::{InteractionEvent, TargetedEvent};

/// Visual state of a button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Button is idle (not being interacted with)
    Idle,
    /// Mouse is hovering over the button
    Hovered,
    /// Button is being pressed
    Pressed,
    /// Button is disabled (not interactive)
    Disabled,
}

impl ButtonState {
    /// Update the button state based on interaction flags
    ///
    /// # Arguments
    /// * `is_hovered` - Whether the button is currently hovered
    /// * `is_pressed` - Whether the button is currently pressed
    /// * `enabled` - Whether the button is enabled
    pub fn update(&mut self, is_hovered: bool, is_pressed: bool, enabled: bool) {
        if !enabled {
            *self = ButtonState::Disabled;
        } else if is_pressed {
            *self = ButtonState::Pressed;
        } else if is_hovered {
            *self = ButtonState::Hovered;
        } else {
            *self = ButtonState::Idle;
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Idle
    }
}

/// Visual styling for a button
#[derive(Debug, Clone, WithBuilders)]
pub struct ButtonStyle {
    /// Background color when idle
    pub idle_color: Color,
    /// Background color when hovered
    pub hover_color: Color,
    /// Background color when pressed
    pub pressed_color: Color,
    /// Background color when disabled
    pub disabled_color: Color,

    /// Stroke color when idle
    pub idle_stroke_color: Color,
    /// Stroke color when hovered
    pub hover_stroke_color: Color,
    /// Stroke color when pressed
    pub pressed_stroke_color: Color,
    /// Stroke color when disabled
    pub disabled_stroke_color: Color,

    /// Text color
    pub text_color: Color,
    /// Disabled text color
    pub disabled_text_color: Color,

    /// Internal padding
    pub padding: Spacing,
    /// Corner radius for rounded corners
    pub border_radius: f32,
    /// Font size
    pub font_size: f32,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            // Fill Colors
            idle_color: mocha::BASE,
            hover_color: mocha::MANTLE,
            pressed_color: mocha::CRUST,
            disabled_color: mocha::BASE.with_alpha(0.8),
            // Stroke Colors
            idle_stroke_color: mocha::SURFACE0,
            hover_stroke_color: mocha::SURFACE0,
            pressed_stroke_color: mocha::SURFACE0,
            disabled_stroke_color: mocha::SURFACE0.with_alpha(0.8),
            // Text Colors
            text_color: mocha::TEXT,
            disabled_text_color: mocha::SUBTEXT1,
            // Others
            padding: Spacing::symmetric(Size::lpx(18.0), Size::lpx(10.0)),
            border_radius: 24.0,
            font_size: 24.0,
        }
    }
}

/// A clickable button component
///
/// # Example
///
/// ```ignore
/// Button::new("Click me")
///     .on_click(|| println!("Clicked!"))
///     .node(&mut ctx)
/// ```
pub struct Button {
    label: String,
    disabled: bool,
    style: ButtonStyle,
    on_click: Option<Box<dyn FnMut()>>,
    on_hover: Option<Box<dyn FnMut()>>,
}

impl Button {
    /// Create a new button with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Button {
            label: label.into(),
            disabled: false,
            style: ButtonStyle::default(),
            on_click: None,
            on_hover: None,
        }
    }

    /// Set whether the button is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a custom style for the button
    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set a callback to be called when the button is clicked
    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Set a callback to be called when the button is hovered
    pub fn on_hover(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_hover = Some(Box::new(f));
        self
    }
}

impl Component for Button {
    /// Create a button node with declarative hover/active/disabled states
    ///
    /// This version uses the new style system with automatic state management.
    /// No need to manually track button state - hover and active states are
    /// applied automatically based on mouse interaction.
    fn node(mut self, ctx: &mut UiContext) -> Node {
        // Generate a unique ID for this button
        let id = ctx.generate_id("button");

        // Check for events from last frame and fire callbacks
        if !self.disabled {
            if ctx.was_clicked(&id) {
                if let Some(ref mut on_click) = self.on_click {
                    on_click();
                }
            }

            if ctx.is_hovered(&id) {
                if let Some(ref mut on_hover) = self.on_hover {
                    on_hover();
                }
            }
        }

        Node::new()
            .with_id(NodeId::new(&id))
            .with_width(Size::FitContent)
            .with_height(Size::FitContent)
            .with_padding(self.style.padding)
            .with_shape(astra_gui::Shape::rect())
            .with_content(Content::Text(TextContent {
                text: self.label,
                font_size: Size::lpx(self.style.font_size),
                color: self.style.text_color,
                h_align: HorizontalAlign::Center,
                v_align: VerticalAlign::Center,
                wrap: astra_gui::Wrap::Word,
                line_height_multiplier: 1.2,
            }))
            // Declarative styles - no manual state tracking needed!
            .with_style(Style {
                fill_color: Some(self.style.idle_color),
                text_color: Some(self.style.text_color),
                corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                    self.style.border_radius,
                ))),
                stroke: Some(Stroke::new(Size::lpx(1.0), self.style.idle_stroke_color)),
                ..Default::default()
            })
            .with_hover_style(Style {
                fill_color: Some(self.style.hover_color),
                stroke: Some(Stroke::new(Size::lpx(1.0), self.style.hover_stroke_color)),
                ..Default::default()
            })
            .with_active_style(Style {
                fill_color: Some(self.style.pressed_color),
                stroke: Some(Stroke::new(Size::lpx(2.0), self.style.pressed_stroke_color)),
                ..Default::default()
            })
            .with_disabled_style(Style {
                fill_color: Some(self.style.disabled_color),
                text_color: Some(self.style.disabled_text_color),
                stroke: Some(Stroke::new(
                    Size::lpx(1.0),
                    self.style.disabled_stroke_color,
                )),
                ..Default::default()
            })
            .with_disabled(self.disabled)
            .with_transition(Transition::quick())
    }
}

/// Check if a button with the given ID was clicked this frame
///
/// # Arguments
/// * `button_id` - The ID of the button to check
/// * `events` - Slice of targeted events from this frame
///
/// # Returns
/// `true` if the button was clicked, `false` otherwise
pub fn button_clicked(button_id: &str, events: &[TargetedEvent]) -> bool {
    events.iter().any(|e| {
        matches!(e.event, InteractionEvent::Click { .. }) && e.target.as_str() == button_id
    })
}

/// Check if a button with the given ID is currently hovered
///
/// # Arguments
/// * `button_id` - The ID of the button to check
/// * `events` - Slice of targeted events from this frame
///
/// # Returns
/// `true` if the button is hovered, `false` otherwise
pub fn button_hovered(button_id: &str, events: &[TargetedEvent]) -> bool {
    events.iter().any(|e| {
        matches!(e.event, InteractionEvent::Hover { .. }) && e.target.as_str() == button_id
    })
}

/// Create a button node with declarative hover/active/disabled states
///
/// This is a backward-compatible function that wraps the new `Button` struct.
/// For new code, prefer using `Button::new()` with the builder pattern.
///
/// # Arguments
/// * `id` - Unique identifier for the button (used for event targeting)
/// * `label` - Text label displayed on the button
/// * `disabled` - Whether the button is disabled (cannot be interacted with)
/// * `style` - Visual styling configuration
///
/// # Returns
/// A configured `Node` representing the button with automatic state transitions
///
/// # Example
///
/// ```ignore
/// // Old style (still works):
/// let node = button("my_button", "Click me", false, &ButtonStyle::default());
///
/// // New style (preferred):
/// let node = Button::new("Click me")
///     .on_click(|| println!("Clicked!"))
///     .node(&mut ctx);
/// ```
#[deprecated(
    since = "0.8.0",
    note = "Use Button::new() with the builder pattern instead"
)]
pub fn button(
    id: impl Into<String>,
    label: impl Into<String>,
    disabled: bool,
    style: &ButtonStyle,
) -> Node {
    let id_str = id.into();
    Node::new()
        .with_id(NodeId::new(&id_str))
        .with_width(Size::FitContent)
        .with_height(Size::FitContent)
        .with_padding(style.padding)
        .with_shape(astra_gui::Shape::rect())
        .with_content(Content::Text(TextContent {
            text: label.into(),
            font_size: Size::lpx(style.font_size),
            color: style.text_color,
            h_align: HorizontalAlign::Center,
            v_align: VerticalAlign::Center,
            wrap: astra_gui::Wrap::Word,
            line_height_multiplier: 1.2,
        }))
        // Declarative styles - no manual state tracking needed!
        .with_style(Style {
            fill_color: Some(style.idle_color),
            text_color: Some(style.text_color),
            corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                style.border_radius,
            ))),
            stroke: Some(Stroke::new(Size::lpx(1.0), style.idle_stroke_color)),
            ..Default::default()
        })
        .with_hover_style(Style {
            fill_color: Some(style.hover_color),
            stroke: Some(Stroke::new(Size::lpx(1.0), style.hover_stroke_color)),
            ..Default::default()
        })
        .with_active_style(Style {
            fill_color: Some(style.pressed_color),
            stroke: Some(Stroke::new(Size::lpx(2.0), style.pressed_stroke_color)),
            ..Default::default()
        })
        .with_disabled_style(Style {
            fill_color: Some(style.disabled_color),
            text_color: Some(style.disabled_text_color),
            stroke: Some(Stroke::new(Size::lpx(1.0), style.disabled_stroke_color)),
            ..Default::default()
        })
        .with_disabled(disabled)
        .with_transition(Transition::quick())
}
