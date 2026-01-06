//! Drag value component for interactive UI
//!
//! Provides a draggable number input field similar to egui's DragValue.
//! Users can drag left/right to adjust the value, or click to enter text input mode.

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, HorizontalAlign, Layout, Node, NodeId, Size,
    Spacing, Stroke, Style, TextContent, Transition, UiContext, VerticalAlign,
};
use astra_gui_macros::WithBuilders;
use astra_gui_wgpu::{InteractionEvent, Key, NamedKey};
use std::ops::RangeInclusive;

use crate::TextInputStyle;

/// Visual styling for a drag value widget
#[derive(Debug, Clone, WithBuilders)]
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
    #[with_builders(skip)]
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
    /// Set the minimum width
    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self.text_input_style.width = min_width;
        self
    }
}

// Note: DragValueState is defined in astra_gui::memory and re-exported from astra_gui
// It uses a nested TextInputState structure:
// - text_input.text: String (text buffer)
// - text_input.cursor_pos: usize
// - text_input.selection: Option<(usize, usize)>
// - drag_accumulator: f32
// - text_mode: bool (editing mode)

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

/// A drag value component
///
/// # Example
///
/// ```ignore
/// DragValue::new(&mut value)
///     .range(0.0..=100.0)
///     .speed(0.1)
///     .on_change(|new_value| println!("Value: {}", new_value))
///     .node(&mut ctx)
/// ```
pub struct DragValue<'a> {
    value: &'a mut f32,
    range: Option<RangeInclusive<f32>>,
    step: Option<f32>,
    speed: f32,
    disabled: bool,
    style: DragValueStyle,
    on_change: Option<Box<dyn FnMut(f32) + 'a>>,
}

impl<'a> DragValue<'a> {
    /// Create a new drag value bound to a mutable float reference
    pub fn new(value: &'a mut f32) -> Self {
        DragValue {
            value,
            range: None,
            step: None,
            speed: 0.1,
            disabled: false,
            style: DragValueStyle::default(),
            on_change: None,
        }
    }

    /// Set the valid range of values
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = Some(range);
        self
    }

    /// Set the step size for value snapping
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the drag speed (pixels to value multiplier)
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set whether the drag value is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a custom style for the drag value
    pub fn with_style(mut self, style: DragValueStyle) -> Self {
        self.style = style;
        self
    }

    /// Set a callback to be called when the value changes
    pub fn on_change(mut self, f: impl FnMut(f32) + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Build the drag value node
    ///
    /// Note: This is not implementing Component trait because we need lifetime 'a
    /// for the mutable reference to the value.
    pub fn build(mut self, ctx: &mut UiContext) -> Node {
        // Generate unique ID
        let id = ctx.generate_id("drag_value");
        let hitbox_id = format!("{}_hitbox", id);
        let container_id = format!("{}_container", id);

        // Get or create state from widget memory
        let state = ctx.memory().drag_value(&id, *self.value);
        let mut text_buffer = state.text_input.text.clone();
        let mut cursor_pos = state.text_input.cursor_pos;
        let mut selection = state.text_input.selection;
        let mut drag_accumulator = state.drag_accumulator;
        let mut editing = state.text_mode;

        // Check if this widget is actually focused according to the event dispatcher
        let is_focused = ctx.is_focused(&id);

        // Sync local editing state with focus
        if !is_focused && editing {
            editing = false;
        }

        // Process drag events
        let mut value_changed = false;
        let mut was_dragged = false;
        let input = ctx.input().clone();

        // Track focus changes to apply after event loop
        let mut should_focus: Option<bool> = None; // Some(true) = focus, Some(false) = unfocus

        // Collect events to avoid borrow issues
        let events: Vec<_> = ctx.events().to_vec();

        for event in &events {
            let target_str = event.target.as_str();

            if target_str != hitbox_id && target_str != container_id {
                continue;
            }

            match &event.event {
                InteractionEvent::DragStart { .. } => {
                    // Initialize accumulator with current value when drag starts
                    drag_accumulator = *self.value;
                    // Unfocus if currently focused (user is starting to drag)
                    if is_focused {
                        editing = false;
                        should_focus = Some(false);
                    }
                }
                InteractionEvent::DragMove { delta, .. } => {
                    was_dragged = true;
                    // Calculate value change from horizontal drag
                    let mut drag_speed = self.speed;

                    // Apply speed modifiers
                    if input.shift_held {
                        drag_speed *= 0.1; // Slower, more precise
                    }
                    if input.ctrl_held {
                        drag_speed *= 10.0; // Faster
                    }

                    let delta_value = delta.x * drag_speed;

                    // Update the continuous accumulator
                    drag_accumulator += delta_value;

                    // Apply range clamping to accumulator
                    if let Some(ref value_range) = self.range {
                        drag_accumulator =
                            drag_accumulator.clamp(*value_range.start(), *value_range.end());
                    }

                    // Calculate the stepped value from the accumulator
                    let mut new_value = drag_accumulator;

                    if let Some(step_size) = self.step {
                        if step_size > 0.0 {
                            if let Some(ref value_range) = self.range {
                                let steps_from_start =
                                    ((new_value - value_range.start()) / step_size).round();
                                new_value = value_range.start() + steps_from_start * step_size;
                                new_value =
                                    new_value.clamp(*value_range.start(), *value_range.end());
                            } else {
                                // Snap to nearest step from 0
                                new_value = (new_value / step_size).round() * step_size;
                            }
                        }
                    }

                    // Only update the exposed value if it changed
                    if (*self.value - new_value).abs() > f32::EPSILON {
                        *self.value = new_value;
                        value_changed = true;
                    }
                }
                InteractionEvent::DragEnd { .. } => {
                    // Only enter text input mode if we didn't actually drag
                    if !was_dragged {
                        editing = true;
                        text_buffer = format_value(*self.value, 6); // Use high precision for editing
                        cursor_pos = text_buffer.len(); // Place cursor at end
                        selection = None;
                        should_focus = Some(true);
                    }
                }
                _ => {}
            }
        }

        // Apply focus changes after event loop
        match should_focus {
            Some(true) => ctx.set_focus(Some(&id)),
            Some(false) => ctx.set_focus(None),
            None => {}
        }

        // If editing, handle text input
        if editing && !was_dragged {
            let shift_held = input.shift_held;
            let ctrl_held = input.ctrl_held;

            // Track selection anchor point
            let selection_anchor = if let Some((start, end)) = selection {
                if cursor_pos == end {
                    Some(start)
                } else {
                    Some(end)
                }
            } else {
                None
            };

            // Process typed characters
            for ch in &input.characters_typed {
                // Delete selection if exists before inserting
                if let Some((start, end)) = selection {
                    if start < end {
                        text_buffer.replace_range(start..end, "");
                        cursor_pos = start;
                        selection = None;
                    }
                }

                // Insert character at cursor position
                if cursor_pos <= text_buffer.len() {
                    text_buffer.insert(cursor_pos, *ch);
                    cursor_pos += ch.len_utf8();
                    ctx.reset_cursor_blink(&id);
                }
            }

            // Process special keys
            for key in &input.keys_just_pressed {
                match key {
                    Key::Named(NamedKey::Enter) => {
                        // Parse text and update value
                        if let Some(new_value) = parse_value(&text_buffer) {
                            let mut clamped_value = new_value;

                            // Apply range clamping
                            if let Some(ref value_range) = self.range {
                                clamped_value =
                                    clamped_value.clamp(*value_range.start(), *value_range.end());
                            }

                            // Apply step snapping
                            if let Some(step_size) = self.step {
                                if step_size > 0.0 {
                                    if let Some(ref value_range) = self.range {
                                        let steps_from_start =
                                            ((clamped_value - value_range.start()) / step_size)
                                                .round();
                                        clamped_value =
                                            value_range.start() + steps_from_start * step_size;
                                        clamped_value = clamped_value
                                            .clamp(*value_range.start(), *value_range.end());
                                    } else {
                                        clamped_value =
                                            (clamped_value / step_size).round() * step_size;
                                    }
                                }
                            }

                            *self.value = clamped_value;
                            drag_accumulator = clamped_value;
                            value_changed = true;
                        }

                        // Unfocus after accepting the value
                        editing = false;
                        ctx.set_focus(None);
                    }
                    Key::Named(NamedKey::Escape) => {
                        // Cancel editing without applying changes
                        editing = false;
                        ctx.set_focus(None);
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if let Some((start, end)) = selection {
                            if start < end {
                                text_buffer.replace_range(start..end, "");
                                cursor_pos = start;
                                selection = None;
                                ctx.reset_cursor_blink(&id);
                            }
                        } else if cursor_pos > 0 && !text_buffer.is_empty() {
                            let mut new_pos = cursor_pos - 1;
                            while new_pos > 0 && !text_buffer.is_char_boundary(new_pos) {
                                new_pos -= 1;
                            }
                            text_buffer.remove(new_pos);
                            cursor_pos = new_pos;
                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    Key::Named(NamedKey::Delete) => {
                        if let Some((start, end)) = selection {
                            if start < end {
                                text_buffer.replace_range(start..end, "");
                                cursor_pos = start;
                                selection = None;
                                ctx.reset_cursor_blink(&id);
                            }
                        } else if cursor_pos < text_buffer.len() {
                            text_buffer.remove(cursor_pos);
                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if cursor_pos > 0 {
                            let old_pos = cursor_pos;
                            cursor_pos -= 1;
                            while cursor_pos > 0 && !text_buffer.is_char_boundary(cursor_pos) {
                                cursor_pos -= 1;
                            }

                            if shift_held {
                                if let Some(anchor) = selection_anchor {
                                    selection = Some(if cursor_pos < anchor {
                                        (cursor_pos, anchor)
                                    } else {
                                        (anchor, cursor_pos)
                                    });
                                } else {
                                    selection = Some((cursor_pos, old_pos));
                                }
                            } else {
                                selection = None;
                            }

                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if cursor_pos < text_buffer.len() {
                            let old_pos = cursor_pos;
                            cursor_pos += 1;
                            while cursor_pos < text_buffer.len()
                                && !text_buffer.is_char_boundary(cursor_pos)
                            {
                                cursor_pos += 1;
                            }

                            if shift_held {
                                if let Some(anchor) = selection_anchor {
                                    selection = Some(if cursor_pos < anchor {
                                        (cursor_pos, anchor)
                                    } else {
                                        (anchor, cursor_pos)
                                    });
                                } else {
                                    selection = Some((old_pos, cursor_pos));
                                }
                            } else {
                                selection = None;
                            }

                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    Key::Named(NamedKey::Home) => {
                        let old_pos = cursor_pos;
                        cursor_pos = 0;

                        if shift_held {
                            if let Some(anchor) = selection_anchor {
                                selection = Some((0, anchor));
                            } else {
                                selection = Some((0, old_pos));
                            }
                        } else {
                            selection = None;
                        }

                        ctx.reset_cursor_blink(&id);
                    }
                    Key::Named(NamedKey::End) => {
                        let old_pos = cursor_pos;
                        cursor_pos = text_buffer.len();

                        if shift_held {
                            if let Some(anchor) = selection_anchor {
                                selection = Some(if text_buffer.len() > anchor {
                                    (anchor, text_buffer.len())
                                } else {
                                    (text_buffer.len(), anchor)
                                });
                            } else {
                                selection = Some((old_pos, text_buffer.len()));
                            }
                        } else {
                            selection = None;
                        }

                        ctx.reset_cursor_blink(&id);
                    }
                    Key::Character(ref ch) if ch == "a" && ctrl_held => {
                        if !text_buffer.is_empty() {
                            selection = Some((0, text_buffer.len()));
                            cursor_pos = text_buffer.len();
                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Fire on_change callback if value changed
        if value_changed {
            if let Some(ref mut on_change) = self.on_change {
                on_change(*self.value);
            }
        }

        // Update state in widget memory
        let state = ctx.memory().drag_value(&id, *self.value);
        state.text_input.text = text_buffer.clone();
        state.text_input.cursor_pos = cursor_pos;
        state.text_input.selection = selection;
        state.drag_accumulator = drag_accumulator;
        state.text_mode = editing;

        // Build the appropriate node based on editing state
        if editing {
            // Use text_input-style rendering when editing
            build_editing_node(
                &id,
                &text_buffer,
                cursor_pos,
                selection,
                self.disabled,
                &self.style,
                ctx,
            )
        } else {
            // Use drag display rendering
            build_drag_display_node(&id, *self.value, self.disabled, &self.style)
        }
    }
}

/// Build the visual node for drag value in editing mode
fn build_editing_node(
    id: &str,
    text_buffer: &str,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    disabled: bool,
    style: &DragValueStyle,
    ctx: &mut UiContext,
) -> Node {
    let node_id = NodeId::new(id);
    let hitbox_id = format!("{}_hitbox", id);

    // Update cursor blink
    let cursor_visible = ctx.update_cursor_blink(
        id,
        style
            .text_input_style
            .cursor_style
            .blink_interval
            .as_millis() as u64,
    );

    // Determine cursor color
    let cursor_color = style
        .text_input_style
        .cursor_style
        .color
        .unwrap_or(style.text_color);

    // Calculate cursor x position (simplified without measurer)
    let char_width = style.font_size * 0.6;
    let total_width = text_buffer.len() as f32 * char_width;
    let text_container_width = style.min_width - style.padding.get_horizontal();
    let text_start_x = (text_container_width - total_width) / 2.0;
    let cursor_x_offset = text_start_x + cursor_pos as f32 * char_width;

    let mut children = vec![];

    // Add selection highlight if there is a selection
    if let Some((start, end)) = selection {
        if start < end && !text_buffer.is_empty() {
            let selection_x = text_start_x + start as f32 * char_width;
            let selection_width = (end - start) as f32 * char_width;

            children.push(
                Node::new()
                    .with_width(Size::lpx(selection_width))
                    .with_height(Size::lpx(style.font_size))
                    .with_translation(astra_gui::Translation::x(astra_gui::Size::Logical(
                        selection_x,
                    )))
                    .with_style(Style {
                        fill_color: Some(style.text_input_style.selection_color),
                        corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(5.0))),
                        ..Default::default()
                    }),
            );
        }
    }

    // Text content
    children.push(
        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_content(Content::Text(TextContent {
                text: text_buffer.to_string(),
                font_size: Size::lpx(style.font_size),
                color: style.text_color,
                h_align: HorizontalAlign::Center,
                v_align: VerticalAlign::Center,
                wrap: astra_gui::Wrap::None,
                line_height_multiplier: 1.2,
            })),
    );

    // Add cursor if visible
    if cursor_visible && !disabled {
        children.push(
            Node::new()
                .with_width(Size::lpx(style.text_input_style.cursor_style.thickness))
                .with_height(Size::lpx(style.font_size))
                .with_translation(astra_gui::Translation::x(astra_gui::Size::Logical(
                    cursor_x_offset,
                )))
                .with_shape(astra_gui::Shape::Rect(astra_gui::StyledRect::new(
                    astra_gui::Rect::default(),
                    cursor_color,
                ))),
        );
    }

    // Add hitbox
    children.push(
        Node::new()
            .with_id(NodeId::new(&hitbox_id))
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_disabled(disabled),
    );

    Node::new()
        .with_id(node_id)
        .with_width(Size::lpx(style.min_width))
        .with_height(Size::lpx(style.font_size + style.padding.get_vertical()))
        .with_padding(style.padding)
        .with_layout_direction(Layout::Stack)
        .with_overflow(astra_gui::Overflow::Hidden)
        .with_style(Style {
            fill_color: Some(style.text_input_style.focused_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.text_input_style.focused_stroke_width),
                style.text_input_style.focused_stroke_color,
            )),
            corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                style.border_radius,
            ))),
            ..Default::default()
        })
        .with_disabled(disabled)
        .with_transition(Transition::quick())
        .with_children(children)
}

/// Build the visual node for drag value in display mode
fn build_drag_display_node(id: &str, value: f32, disabled: bool, style: &DragValueStyle) -> Node {
    let container_id = format!("{}_container", id);
    let hitbox_id = format!("{}_hitbox", id);
    let display_text = format_value(value, style.precision);

    Node::new()
        .with_id(NodeId::new(&container_id))
        .with_width(Size::lpx(style.min_width))
        .with_height(Size::lpx(style.font_size + style.padding.get_vertical()))
        .with_padding(style.padding)
        .with_layout_direction(Layout::Stack)
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
                    wrap: astra_gui::Wrap::Word,
                    line_height_multiplier: 1.2,
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
                .with_id(NodeId::new(&hitbox_id))
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_disabled(disabled),
        ])
}
