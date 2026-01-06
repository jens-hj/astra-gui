//! Text input component for interactive UI
//!
//! Provides an editable text input field with cursor, selection, and keyboard support.

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, HorizontalAlign, Layout, MeasureTextRequest,
    Node, NodeId, Overflow, Rect, Shape, Size, Spacing, Stroke, Style, StyledRect, TextContent,
    Transition, Translation, UiContext, VerticalAlign,
};
use astra_gui_macros::WithBuilders;
use astra_gui_wgpu::{InteractionEvent, Key, MouseButton, NamedKey};
use std::time::Duration;

/// Cursor shape for text input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    /// Vertical line (classic text cursor)
    Line,
    /// Underline under the character
    Underline,
    /// Block covering the character
    Block,
}

/// Cursor/caret styling for text input
#[derive(Debug, Clone, WithBuilders)]
pub struct CursorStyle {
    /// Shape of the cursor
    pub shape: CursorShape,
    /// Cursor color (if None, uses text color)
    pub color: Option<Color>,
    /// Cursor width (for Line shape)
    pub thickness: f32,
    /// Blink interval (time between blinks)
    pub blink_interval: Duration,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self {
            shape: CursorShape::Line,
            color: None, // Use text color
            thickness: 2.0,
            blink_interval: Duration::from_millis(530), // Standard blink rate
        }
    }
}

/// Visual styling for a text input
#[derive(Debug, Clone, WithBuilders)]
pub struct TextInputStyle {
    /// Background color when idle
    pub idle_color: Color,
    /// Background color when focused
    pub focused_color: Color,
    /// Background color when disabled
    pub disabled_color: Color,

    /// Stroke color when idle
    pub idle_stroke_color: Color,
    /// Stroke color when focused
    pub focused_stroke_color: Color,
    /// Stroke color when disabled
    pub disabled_stroke_color: Color,

    // Stroke width
    pub idle_stroke_width: f32,
    pub focused_stroke_width: f32,
    pub disabled_stroke_width: f32,

    /// Text color
    pub text_color: Color,
    /// Placeholder text color
    pub placeholder_text_color: Color,
    /// Disabled text color
    pub disabled_text_color: Color,

    /// Selection highlight color
    pub selection_color: Color,
    /// Internal padding
    pub padding: Spacing,
    /// Corner radius for rounded corners
    pub border_radius: f32,
    /// Font size
    pub font_size: f32,
    /// Cursor/caret styling
    pub cursor_style: CursorStyle,
    /// Text horizontal alignment
    pub text_align: HorizontalAlign,
    /// Width of the text input widget
    pub width: f32,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            // Fill Colors
            idle_color: mocha::SURFACE0,
            focused_color: mocha::SURFACE1,
            disabled_color: mocha::SURFACE0.with_alpha(0.8),
            // Stroke Colors
            idle_stroke_color: mocha::LAVENDER,
            focused_stroke_color: mocha::LAVENDER,
            disabled_stroke_color: mocha::SURFACE2,
            // Stroke Width
            idle_stroke_width: 2.0,
            focused_stroke_width: 3.0,
            disabled_stroke_width: 2.0,
            // Text Colors
            text_color: mocha::TEXT,
            placeholder_text_color: mocha::SUBTEXT0,
            disabled_text_color: mocha::SUBTEXT0,
            // Other
            selection_color: mocha::LAVENDER.with_alpha(0.3),
            padding: Spacing::symmetric(Size::lpx(10.0), Size::lpx(8.0)),
            border_radius: 8.0,
            font_size: 20.0,
            cursor_style: CursorStyle::default(),
            text_align: HorizontalAlign::Left,
            width: 300.0,
        }
    }
}

/// Internal state stored in WidgetMemory for text input
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    /// Current cursor position (character index)
    pub cursor_pos: usize,
    /// Selection range (start, end) in character indices
    pub selection: Option<(usize, usize)>,
}

/// A text input component
///
/// # Example
///
/// ```ignore
/// TextInput::new(&mut text_value)
///     .placeholder("Enter text...")
///     .on_change(|new_value| println!("Changed: {}", new_value))
///     .on_submit(|value| println!("Submitted: {}", value))
///     .node(&mut ctx)
/// ```
pub struct TextInput<'a> {
    value: &'a mut String,
    placeholder: String,
    disabled: bool,
    style: TextInputStyle,
    on_change: Option<Box<dyn FnMut(&str) + 'a>>,
    on_submit: Option<Box<dyn FnMut(&str) + 'a>>,
}

impl<'a> TextInput<'a> {
    /// Create a new text input bound to a mutable string reference
    pub fn new(value: &'a mut String) -> Self {
        TextInput {
            value,
            placeholder: String::new(),
            disabled: false,
            style: TextInputStyle::default(),
            on_change: None,
            on_submit: None,
        }
    }

    /// Set the placeholder text shown when empty
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set whether the text input is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a custom style for the text input
    pub fn with_style(mut self, style: TextInputStyle) -> Self {
        self.style = style;
        self
    }

    /// Set a callback to be called when the text changes
    pub fn on_change(mut self, f: impl FnMut(&str) + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Set a callback to be called when Enter is pressed
    pub fn on_submit(mut self, f: impl FnMut(&str) + 'a) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Build the text input node
    ///
    /// Note: This is not implementing Component trait because we need lifetime 'a
    /// for the mutable reference to the value string.
    pub fn build(mut self, ctx: &mut UiContext) -> Node {
        // Generate unique ID
        let id = ctx.generate_id("text_input");
        let hitbox_id = format!("{}_hitbox", id);
        let _node_id = NodeId::new(&id);

        // Get or create state from widget memory
        let state = ctx.memory().text_input(&id);
        let mut cursor_pos = state.cursor_pos;
        let mut selection = state.selection;

        // Check if focused
        let is_focused = ctx.is_focused(&id);

        // Handle click to focus
        let was_clicked = ctx.events().iter().any(|e| {
            matches!(e.event, InteractionEvent::Click { .. })
                && (e.target.as_str() == id || e.target.as_str() == hitbox_id)
        });

        if was_clicked && !self.disabled {
            ctx.set_focus(Some(&id));
        }

        // Handle unfocus: clicking outside or pressing ESC
        let input = ctx.input().clone();
        let mouse_clicked_outside = input.is_button_just_pressed(MouseButton::Left) && !was_clicked;
        let escape_pressed = input
            .keys_just_pressed
            .iter()
            .any(|key| matches!(key, Key::Named(NamedKey::Escape)));

        if (mouse_clicked_outside || escape_pressed) && is_focused {
            ctx.set_focus(None);
        }

        // Re-check focus after potential changes
        let focused = ctx.is_focused(&id);

        // Process keyboard input if focused
        let mut value_changed = false;

        if focused && !self.disabled {
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
                        self.value.replace_range(start..end, "");
                        cursor_pos = start;
                        selection = None;
                        value_changed = true;
                    }
                }

                // Insert character at cursor position
                if cursor_pos <= self.value.len() {
                    self.value.insert(cursor_pos, *ch);
                    cursor_pos += ch.len_utf8();
                    value_changed = true;
                    ctx.reset_cursor_blink(&id);
                }
            }

            // Process special keys
            for key in &input.keys_just_pressed {
                match key {
                    // Ctrl/Cmd+A: Select all
                    Key::Character(ref ch) if ch == "a" && ctrl_held => {
                        if !self.value.is_empty() {
                            selection = Some((0, self.value.len()));
                            cursor_pos = self.value.len();
                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    Key::Named(NamedKey::Enter) => {
                        if let Some(ref mut on_submit) = self.on_submit {
                            on_submit(self.value);
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        // Delete selection if exists
                        if let Some((start, end)) = selection {
                            if start < end {
                                self.value.replace_range(start..end, "");
                                cursor_pos = start;
                                selection = None;
                                value_changed = true;
                                ctx.reset_cursor_blink(&id);
                            }
                        } else if cursor_pos > 0 && !self.value.is_empty() {
                            if ctrl_held {
                                let new_pos = find_prev_word_boundary(self.value, cursor_pos);
                                self.value.replace_range(new_pos..cursor_pos, "");
                                cursor_pos = new_pos;
                            } else {
                                let mut new_pos = cursor_pos - 1;
                                while new_pos > 0 && !self.value.is_char_boundary(new_pos) {
                                    new_pos -= 1;
                                }
                                self.value.remove(new_pos);
                                cursor_pos = new_pos;
                            }
                            value_changed = true;
                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    Key::Named(NamedKey::Delete) => {
                        // Delete selection if exists
                        if let Some((start, end)) = selection {
                            if start < end {
                                self.value.replace_range(start..end, "");
                                cursor_pos = start;
                                selection = None;
                                value_changed = true;
                                ctx.reset_cursor_blink(&id);
                            }
                        } else if cursor_pos < self.value.len() {
                            if ctrl_held {
                                let new_pos = find_next_word_boundary(self.value, cursor_pos);
                                self.value.replace_range(cursor_pos..new_pos, "");
                            } else {
                                self.value.remove(cursor_pos);
                            }
                            value_changed = true;
                            ctx.reset_cursor_blink(&id);
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if cursor_pos > 0 {
                            let old_pos = cursor_pos;

                            if ctrl_held {
                                cursor_pos = find_prev_word_boundary(self.value, cursor_pos);
                            } else {
                                cursor_pos -= 1;
                                while cursor_pos > 0 && !self.value.is_char_boundary(cursor_pos) {
                                    cursor_pos -= 1;
                                }
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
                        if cursor_pos < self.value.len() {
                            let old_pos = cursor_pos;

                            if ctrl_held {
                                cursor_pos = find_next_word_boundary(self.value, cursor_pos);
                            } else {
                                cursor_pos += 1;
                                while cursor_pos < self.value.len()
                                    && !self.value.is_char_boundary(cursor_pos)
                                {
                                    cursor_pos += 1;
                                }
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
                        cursor_pos = self.value.len();

                        if shift_held {
                            if let Some(anchor) = selection_anchor {
                                selection = Some(if self.value.len() > anchor {
                                    (anchor, self.value.len())
                                } else {
                                    (self.value.len(), anchor)
                                });
                            } else {
                                selection = Some((old_pos, self.value.len()));
                            }
                        } else {
                            selection = None;
                        }

                        ctx.reset_cursor_blink(&id);
                    }
                    _ => {}
                }
            }
        }

        // Fire on_change callback if value changed
        if value_changed {
            if let Some(ref mut on_change) = self.on_change {
                on_change(self.value);
            }
        }

        // Update state in widget memory
        let state = ctx.memory().text_input(&id);
        state.cursor_pos = cursor_pos;
        state.selection = selection;

        // Update cursor blink
        let cursor_visible = if focused {
            ctx.update_cursor_blink(
                &id,
                self.style.cursor_style.blink_interval.as_millis() as u64,
            )
        } else {
            false
        };

        // Build the node
        build_text_input_node(
            &id,
            self.value,
            &self.placeholder,
            focused,
            self.disabled,
            &self.style,
            cursor_pos,
            selection,
            cursor_visible,
            ctx,
        )
    }
}

/// Build the visual node for a text input
fn build_text_input_node(
    id: &str,
    value: &str,
    placeholder: &str,
    focused: bool,
    disabled: bool,
    style: &TextInputStyle,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    cursor_visible: bool,
    ctx: &mut UiContext,
) -> Node {
    let node_id = NodeId::new(id);
    let hitbox_id = format!("{}_hitbox", id);

    // Determine what text to display
    let display_text = if value.is_empty() {
        placeholder.to_string()
    } else {
        value.to_string()
    };

    // Determine text color
    let text_color = if value.is_empty() {
        style.placeholder_text_color
    } else {
        style.text_color
    };

    // Determine cursor color
    let cursor_color = style.cursor_style.color.unwrap_or(style.text_color);

    // Calculate text measurements if we have a measurer
    let (_total_text_width, cursor_x_offset, selection_info) =
        if let Some(measurer) = ctx.measurer() {
            let total_width = if !value.is_empty() {
                measurer
                    .measure_text(MeasureTextRequest {
                        text: value,
                        font_size: style.font_size,
                        h_align: style.text_align,
                        v_align: VerticalAlign::Center,
                        family: None,
                        max_width: None,
                        wrap: astra_gui::Wrap::None,
                        line_height_multiplier: 1.2,
                    })
                    .width
            } else {
                0.0
            };

            let text_container_width = style.width - style.padding.get_horizontal();
            let text_start_x = match style.text_align {
                HorizontalAlign::Left => 0.0,
                HorizontalAlign::Center => (text_container_width - total_width) / 2.0,
                HorizontalAlign::Right => text_container_width - total_width,
            };

            let text_before_cursor = value.chars().take(cursor_pos).collect::<String>();
            let cursor_offset = text_start_x
                + if !text_before_cursor.is_empty() {
                    measurer
                        .measure_text(MeasureTextRequest {
                            text: &text_before_cursor,
                            font_size: style.font_size,
                            h_align: HorizontalAlign::Left,
                            v_align: VerticalAlign::Center,
                            family: None,
                            max_width: None,
                            wrap: astra_gui::Wrap::None,
                            line_height_multiplier: 1.2,
                        })
                        .width
                } else {
                    0.0
                };

            // Calculate selection info
            let sel_info = if let Some((start, end)) = selection {
                if start < end && !value.is_empty() {
                    let text_before_selection = value.chars().take(start).collect::<String>();
                    let selection_x = text_start_x
                        + if !text_before_selection.is_empty() {
                            measurer
                                .measure_text(MeasureTextRequest {
                                    text: &text_before_selection,
                                    font_size: style.font_size,
                                    h_align: HorizontalAlign::Left,
                                    v_align: VerticalAlign::Center,
                                    family: None,
                                    max_width: None,
                                    wrap: astra_gui::Wrap::None,
                                    line_height_multiplier: 1.2,
                                })
                                .width
                        } else {
                            0.0
                        };

                    let selected_text = value
                        .chars()
                        .skip(start)
                        .take(end - start)
                        .collect::<String>();
                    let sel_width = if !selected_text.is_empty() {
                        measurer
                            .measure_text(MeasureTextRequest {
                                text: &selected_text,
                                font_size: style.font_size,
                                h_align: HorizontalAlign::Left,
                                v_align: VerticalAlign::Center,
                                family: None,
                                max_width: None,
                                wrap: astra_gui::Wrap::None,
                                line_height_multiplier: 1.2,
                            })
                            .width
                    } else {
                        0.0
                    };

                    Some((selection_x, sel_width))
                } else {
                    None
                }
            } else {
                None
            };

            (total_width, cursor_offset, sel_info)
        } else {
            // No measurer available, use approximate values
            let char_width = style.font_size * 0.6;
            let total_width = value.len() as f32 * char_width;
            let cursor_offset = cursor_pos as f32 * char_width;
            (total_width, cursor_offset, None)
        };

    let mut children = vec![];

    // Add selection highlight if there is a selection
    if let Some((selection_x, selection_width)) = selection_info {
        children.push(
            Node::new()
                .with_width(Size::lpx(selection_width))
                .with_height(Size::lpx(style.font_size))
                .with_translation(Translation::x(astra_gui::Size::Logical(selection_x)))
                .with_style(Style {
                    fill_color: Some(style.selection_color),
                    corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(5.0))),
                    ..Default::default()
                }),
        );
    }

    // Text content
    children.push(
        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_content(Content::Text(TextContent {
                text: display_text,
                font_size: Size::lpx(style.font_size),
                color: text_color,
                h_align: style.text_align,
                v_align: VerticalAlign::Center,
                wrap: astra_gui::Wrap::None,
                line_height_multiplier: 1.2,
            }))
            .with_style(Style {
                text_color: Some(text_color),
                ..Default::default()
            })
            .with_disabled_style(Style {
                text_color: Some(style.disabled_text_color),
                ..Default::default()
            })
            .with_disabled(disabled)
            .with_transition(Transition::quick()),
    );

    // Add cursor if focused and visible
    if focused && cursor_visible && !disabled {
        let cursor_node = match style.cursor_style.shape {
            CursorShape::Line => Node::new()
                .with_width(Size::lpx(style.cursor_style.thickness))
                .with_height(Size::lpx(style.font_size))
                .with_translation(Translation::x(astra_gui::Size::Logical(cursor_x_offset)))
                .with_shape(Shape::Rect(StyledRect::new(Rect::default(), cursor_color))),
            CursorShape::Underline => {
                let cursor_width = style.font_size * 0.6;
                Node::new()
                    .with_width(Size::lpx(cursor_width))
                    .with_height(Size::lpx(style.cursor_style.thickness))
                    .with_translation(Translation::new(
                        astra_gui::Size::Logical(cursor_x_offset),
                        astra_gui::Size::Logical(style.font_size),
                    ))
                    .with_shape(Shape::Rect(StyledRect::new(Rect::default(), cursor_color)))
            }
            CursorShape::Block => {
                let cursor_width = style.font_size * 0.6;
                Node::new()
                    .with_width(Size::lpx(cursor_width))
                    .with_height(Size::lpx(style.font_size))
                    .with_translation(Translation::x(astra_gui::Size::Logical(
                        (cursor_x_offset - cursor_width).max(0.0),
                    )))
                    .with_shape(Shape::Rect(StyledRect::new(
                        Rect::default(),
                        cursor_color.with_alpha(0.3),
                    )))
            }
        };
        children.push(cursor_node);
    }

    // Add hitbox node
    children.push(
        Node::new()
            .with_id(NodeId::new(&hitbox_id))
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_disabled(disabled),
    );

    let fill_color = if focused {
        style.focused_color
    } else {
        style.idle_color
    };

    let stroke_color = if focused {
        style.focused_stroke_color
    } else {
        style.idle_stroke_color
    };

    let stroke_width = if focused {
        style.focused_stroke_width
    } else {
        style.idle_stroke_width
    };

    Node::new()
        .with_id(node_id)
        .with_width(Size::lpx(style.width))
        .with_height(Size::lpx(style.font_size + style.padding.get_vertical()))
        .with_padding(style.padding)
        .with_layout_direction(Layout::Stack)
        .with_overflow(Overflow::Hidden)
        .with_style(Style {
            fill_color: Some(fill_color),
            stroke: Some(Stroke::new(Size::lpx(stroke_width), stroke_color)),
            corner_shape: Some(CornerShape::Round(astra_gui::Size::Logical(
                style.border_radius,
            ))),
            ..Default::default()
        })
        .with_disabled_style(Style {
            fill_color: Some(style.disabled_color),
            stroke: Some(Stroke::new(
                Size::lpx(stroke_width),
                style.disabled_stroke_color,
            )),
            ..Default::default()
        })
        .with_disabled(disabled)
        .with_transition(Transition::quick())
        .with_children(children)
}

/// Find the next word boundary to the left (backward)
fn find_prev_word_boundary(text: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }

    let mut new_pos = pos;

    // Move back one character first
    new_pos -= 1;
    while new_pos > 0 && !text.is_char_boundary(new_pos) {
        new_pos -= 1;
    }

    // Skip whitespace
    while new_pos > 0
        && text[..new_pos]
            .chars()
            .last()
            .map_or(false, |c| c.is_whitespace())
    {
        new_pos -= 1;
        while new_pos > 0 && !text.is_char_boundary(new_pos) {
            new_pos -= 1;
        }
    }

    // Skip non-whitespace (the word itself)
    while new_pos > 0 {
        let prev_char = text[..new_pos].chars().last();
        if prev_char.map_or(false, |c| c.is_whitespace()) {
            break;
        }
        new_pos -= 1;
        while new_pos > 0 && !text.is_char_boundary(new_pos) {
            new_pos -= 1;
        }
    }

    new_pos
}

/// Find the next word boundary to the right (forward)
fn find_next_word_boundary(text: &str, pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }

    let mut new_pos = pos;
    let chars: Vec<char> = text.chars().collect();
    let mut char_idx = text[..pos].chars().count();

    // Skip current word (non-whitespace)
    while char_idx < chars.len() && !chars[char_idx].is_whitespace() {
        new_pos += chars[char_idx].len_utf8();
        char_idx += 1;
    }

    // Skip whitespace
    while char_idx < chars.len() && chars[char_idx].is_whitespace() {
        new_pos += chars[char_idx].len_utf8();
        char_idx += 1;
    }

    new_pos
}
