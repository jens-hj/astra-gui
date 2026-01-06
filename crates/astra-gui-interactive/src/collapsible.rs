//! Collapsible container component for interactive UI
//!
//! Provides an expandable/collapsible container with a clickable header,
//! similar to egui's collapsing header. Supports smooth animations and nesting.

use std::f32::consts::PI;

use astra_gui::{
    catppuccin::mocha, Color, Component, Content, CornerShape, HorizontalAlign, Layout, Node,
    NodeId, Orientation, Overflow, Shape, Size, Spacing, Stroke, Style, TextContent, Transition,
    TriangleSpec, UiContext, VerticalAlign, ZIndex,
};
use astra_gui_macros::WithBuilders;
use astra_gui_wgpu::{InteractionEvent, TargetedEvent};

/// Visual styling for a collapsible container
#[derive(Debug, Clone, WithBuilders)]
pub struct CollapsibleStyle {
    // Header styling
    /// Background color when header is idle
    pub header_idle_color: Color,
    /// Background color when header is hovered
    pub header_hover_color: Color,
    /// Background color when header is pressed
    pub header_active_color: Color,
    /// Background color when header is disabled
    pub header_disabled_color: Color,

    /// Stroke color when header is idle
    pub header_stroke_idle_color: Color,
    /// Stroke color when header is hovered
    pub header_stroke_hover_color: Color,
    /// Stroke color when header is pressed
    pub header_stroke_active_color: Color,
    /// Stroke color when header is disabled
    pub header_stroke_disabled_color: Color,

    /// Stroke width for the header
    pub stroke_idle_width: f32,
    /// Stroke width for the header when hovered
    pub stroke_hover_width: f32,
    /// Stroke width for the header when pressed
    pub stroke_active_width: f32,
    /// Stroke width for the header when disabled
    pub stroke_disabled_width: f32,

    // Triangle indicator
    /// Color of the triangle indicator
    pub indicator_color: Color,
    /// Color of the triangle indicator when disabled
    pub indicator_disabled_color: Color,
    /// Size of the triangle indicator in pixels
    pub indicator_size: f32,
    /// Stroke width for the triangle indicator
    pub indicator_stroke_width: f32,

    // Text styling
    /// Color of the title text
    pub title_color: Color,
    /// Color of the title text when disabled
    pub title_disabled_color: Color,
    /// Font size for the title
    pub title_font_size: f32,

    // Layout
    /// Padding inside the header
    pub header_padding: Spacing,
    /// Gap between triangle indicator and title
    pub header_gap: f32,
    /// Padding inside the content area
    pub content_padding: Spacing,
    /// Border radius for rounded corners on header
    pub corners: CornerShape,
}

impl Default for CollapsibleStyle {
    fn default() -> Self {
        Self {
            // Header colors
            header_idle_color: mocha::BASE,
            header_hover_color: mocha::MANTLE,
            header_active_color: mocha::CRUST,
            header_disabled_color: mocha::BASE.with_alpha(0.8),

            // Header stroke colors
            header_stroke_idle_color: mocha::SURFACE0,
            header_stroke_hover_color: mocha::SURFACE0,
            header_stroke_active_color: mocha::SURFACE0,
            header_stroke_disabled_color: mocha::SURFACE0.with_alpha(0.8),

            // Header stroke widths
            stroke_idle_width: 1.0,
            stroke_hover_width: 1.0,
            stroke_active_width: 2.0,
            stroke_disabled_width: 1.0,

            // Indicator colors
            indicator_color: mocha::SURFACE1,
            indicator_disabled_color: mocha::SURFACE0.with_alpha(0.8),
            indicator_size: 20.0,
            indicator_stroke_width: 1.0,

            // Text colors
            title_color: mocha::TEXT,
            title_disabled_color: mocha::SUBTEXT1,
            title_font_size: 24.0,

            // Layout
            header_padding: Spacing::all(Size::lpx(14.0)),
            header_gap: 12.0,
            content_padding: Spacing::trbl(
                Size::lpx(8.0),
                Size::lpx(16.0),
                Size::lpx(16.0),
                Size::lpx(16.0),
            ),
            corners: CornerShape::Round(Size::lpx(24.0)),
        }
    }
}

/// A collapsible container component
///
/// # Example
///
/// ```ignore
/// Collapsible::new("Settings", expanded)
///     .on_toggle(|new_expanded| {
///         println!("Toggled: {}", new_expanded);
///     })
///     .children(vec![
///         Button::new("Option 1").node(&mut ctx),
///         Button::new("Option 2").node(&mut ctx),
///     ])
///     .node(&mut ctx)
/// ```
pub struct Collapsible {
    title: String,
    expanded: bool,
    disabled: bool,
    style: CollapsibleStyle,
    children: Vec<Node>,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
}

impl Collapsible {
    /// Create a new collapsible with the given title and expanded state
    pub fn new(title: impl Into<String>, expanded: bool) -> Self {
        Collapsible {
            title: title.into(),
            expanded,
            disabled: false,
            style: CollapsibleStyle::default(),
            children: Vec::new(),
            on_toggle: None,
        }
    }

    /// Set whether the collapsible is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a custom style for the collapsible
    pub fn with_style(mut self, style: CollapsibleStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the children to display when expanded
    pub fn children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }

    /// Add a single child
    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    /// Set a callback to be called when the collapsible is toggled
    ///
    /// The callback receives the new expanded state (opposite of current)
    pub fn on_toggle(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }
}

impl Component for Collapsible {
    fn node(mut self, ctx: &mut UiContext) -> Node {
        // Generate unique IDs
        let id = ctx.generate_id("collapsible");
        let header_id = format!("{}_header", id);
        let indicator_id = format!("{}_indicator", id);
        let content_id = format!("{}_content", id);

        // Check for click events from last frame and fire callback
        if !self.disabled {
            let was_clicked = ctx.was_clicked(&header_id) || ctx.was_clicked(&indicator_id);
            if was_clicked {
                if let Some(ref mut on_toggle) = self.on_toggle {
                    on_toggle(!self.expanded);
                }
            }
        }

        // Triangle indicator - changes orientation to show expanded/collapsed state
        let triangle = Node::new()
            .with_id(NodeId::new(&indicator_id))
            .with_width(Size::lpx(self.style.indicator_size))
            .with_height(Size::lpx(self.style.indicator_size))
            .with_shape(Shape::triangle_with_spec(TriangleSpec::Equilateral {
                orientation: if self.expanded {
                    Orientation::Down
                } else {
                    Orientation::Right
                },
            }))
            .with_rotation(if self.expanded { PI / 2.0 } else { 0.0 })
            .with_style(Style {
                fill_color: Some(self.style.header_idle_color),
                stroke: Some(Stroke::new(
                    Size::lpx(self.style.indicator_stroke_width),
                    self.style.indicator_color,
                )),
                ..Default::default()
            })
            .with_disabled_style(Style {
                fill_color: Some(self.style.header_disabled_color),
                ..Default::default()
            })
            .with_transition(Transition::quick());

        // Title text
        let title_node = Node::new()
            .with_width(Size::Fill)
            .with_height(Size::FitContent)
            .with_content(Content::Text(TextContent {
                text: self.title,
                font_size: Size::lpx(self.style.title_font_size),
                color: self.style.title_color,
                h_align: HorizontalAlign::Left,
                v_align: VerticalAlign::Center,
                wrap: astra_gui::Wrap::Word,
                line_height_multiplier: 1.2,
            }));

        // Clickable header with hover/active states
        let header = Node::new()
            .with_id(NodeId::new(&header_id))
            .with_width(Size::Fill)
            .with_height(Size::FitContent)
            .with_layout_direction(Layout::Horizontal)
            .with_v_align(VerticalAlign::Center)
            .with_gap(Size::lpx(self.style.header_gap))
            .with_padding(self.style.header_padding)
            .with_z_index(ZIndex(1))
            .with_style(Style {
                fill_color: Some(self.style.header_idle_color),
                text_color: Some(self.style.title_color),
                corner_shape: Some(self.style.corners),
                stroke: Some(Stroke::new(
                    Size::lpx(self.style.stroke_idle_width),
                    if self.expanded {
                        self.style.header_stroke_active_color
                    } else {
                        self.style.header_stroke_idle_color
                    },
                )),
                ..Default::default()
            })
            .with_hover_style(Style {
                fill_color: Some(self.style.header_hover_color),
                stroke: Some(Stroke::new(
                    Size::lpx(self.style.stroke_hover_width),
                    self.style.header_stroke_hover_color,
                )),
                ..Default::default()
            })
            .with_active_style(Style {
                fill_color: Some(self.style.header_active_color),
                stroke: Some(Stroke::new(
                    Size::lpx(self.style.stroke_active_width),
                    self.style.header_stroke_active_color,
                )),
                ..Default::default()
            })
            .with_disabled_style(Style {
                fill_color: Some(self.style.header_disabled_color),
                text_color: Some(self.style.title_disabled_color),
                stroke: Some(Stroke::new(
                    Size::lpx(self.style.stroke_disabled_width),
                    self.style.header_stroke_disabled_color,
                )),
                ..Default::default()
            })
            .with_disabled(self.disabled)
            .with_transition(Transition::quick())
            .with_children(vec![triangle, title_node]);

        // Content panel
        let content_panel = Node::new()
            .with_width(Size::Fill)
            .with_height(Size::FitContent)
            .with_layout_direction(Layout::Vertical)
            .with_padding(self.style.content_padding)
            .with_children(self.children);

        // Wrapper with overflow clipping for smooth height animation
        let content_wrapper = Node::new()
            .with_id(NodeId::new(&content_id))
            .with_width(Size::Fill)
            .with_z_index(ZIndex(0)) // Below header's ZIndex(1)
            .with_padding(Spacing::top(Size::lpx(
                self.style.header_padding.get_vertical()
                    + self.style.title_font_size
                    + self.style.content_padding.get_top(),
            )))
            .with_style(Style {
                fill_color: Some(self.style.header_idle_color),
                stroke: Some(Stroke::new(
                    Size::lpx(self.style.stroke_idle_width),
                    self.style.header_stroke_active_color,
                )),
                corner_shape: Some(self.style.corners),
                ..Default::default()
            })
            .with_height(if self.expanded {
                Size::FitContent
            } else {
                Size::lpx(0.0)
            })
            .with_overflow(Overflow::Hidden)
            .with_transition(Transition::standard())
            .with_child(content_panel);

        // Main container
        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::FitContent)
            .with_layout_direction(Layout::Stack)
            .with_children(vec![header, content_wrapper])
    }
}

/// Check if a collapsible with the given ID was clicked this frame
///
/// This checks if either the header or the triangle indicator was clicked.
///
/// # Arguments
/// * `collapsible_id` - The ID of the collapsible to check
/// * `events` - Slice of targeted events from this frame
///
/// # Returns
/// `true` if the collapsible header was clicked, `false` otherwise
pub fn collapsible_clicked(collapsible_id: &str, events: &[TargetedEvent]) -> bool {
    let header_id = format!("{}_header", collapsible_id);
    let indicator_id = format!("{}_indicator", collapsible_id);

    events.iter().any(|e| {
        matches!(e.event, InteractionEvent::Click { .. })
            && (e.target.as_str() == header_id || e.target.as_str() == indicator_id)
    })
}

/// Create a collapsible container node
///
/// This is a backward-compatible function that wraps the new `Collapsible` struct.
/// For new code, prefer using `Collapsible::new()` with the builder pattern.
///
/// # Arguments
/// * `id` - Unique identifier for the collapsible (used for event targeting)
/// * `title` - Text displayed in the header
/// * `expanded` - Current state (true = expanded, false = collapsed)
/// * `disabled` - Whether the collapsible is disabled
/// * `children` - Child nodes displayed in the content area when expanded
/// * `style` - Visual styling configuration
///
/// # Returns
/// A configured `Node` representing the collapsible container
#[deprecated(
    since = "0.8.0",
    note = "Use Collapsible::new() with the builder pattern instead"
)]
pub fn collapsible(
    id: impl Into<String>,
    title: impl Into<String>,
    expanded: bool,
    disabled: bool,
    children: Vec<Node>,
    style: &CollapsibleStyle,
) -> Node {
    let id_str = id.into();
    let title_str = title.into();

    // Triangle indicator - changes orientation to show expanded/collapsed state
    let triangle = Node::new()
        .with_id(NodeId::new(format!("{}_indicator", id_str)))
        .with_width(Size::lpx(style.indicator_size))
        .with_height(Size::lpx(style.indicator_size))
        .with_shape(Shape::triangle_with_spec(TriangleSpec::Equilateral {
            orientation: if expanded {
                Orientation::Down
            } else {
                Orientation::Right
            },
        }))
        .with_rotation(if expanded { PI / 2.0 } else { 0.0 })
        .with_style(Style {
            fill_color: Some(style.header_idle_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.indicator_stroke_width),
                style.indicator_color,
            )),
            ..Default::default()
        })
        .with_disabled_style(Style {
            fill_color: Some(style.header_disabled_color),
            ..Default::default()
        })
        .with_transition(Transition::quick());

    // Title text
    let title_node = Node::new()
        .with_width(Size::Fill)
        .with_height(Size::FitContent)
        .with_content(Content::Text(TextContent {
            text: title_str,
            font_size: Size::lpx(style.title_font_size),
            color: style.title_color,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Center,
            wrap: astra_gui::Wrap::Word,
            line_height_multiplier: 1.2,
        }));

    // Clickable header with hover/active states
    let header = Node::new()
        .with_id(NodeId::new(format!("{}_header", id_str)))
        .with_width(Size::Fill)
        .with_height(Size::FitContent)
        .with_layout_direction(Layout::Horizontal)
        .with_v_align(VerticalAlign::Center)
        .with_gap(Size::lpx(style.header_gap))
        .with_padding(style.header_padding)
        .with_z_index(ZIndex(1))
        .with_style(Style {
            fill_color: Some(style.header_idle_color),
            text_color: Some(style.title_color),
            corner_shape: Some(style.corners),
            stroke: Some(Stroke::new(
                Size::lpx(style.stroke_idle_width),
                if expanded {
                    style.header_stroke_active_color
                } else {
                    style.header_stroke_idle_color
                },
            )),
            ..Default::default()
        })
        .with_hover_style(Style {
            fill_color: Some(style.header_hover_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.stroke_hover_width),
                style.header_stroke_hover_color,
            )),
            ..Default::default()
        })
        .with_active_style(Style {
            fill_color: Some(style.header_active_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.stroke_active_width),
                style.header_stroke_active_color,
            )),
            ..Default::default()
        })
        .with_disabled_style(Style {
            fill_color: Some(style.header_disabled_color),
            text_color: Some(style.title_disabled_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.stroke_disabled_width),
                style.header_stroke_disabled_color,
            )),
            ..Default::default()
        })
        .with_disabled(disabled)
        .with_transition(Transition::quick())
        .with_children(vec![triangle, title_node]);

    // Content panel
    let content_panel = Node::new()
        .with_width(Size::Fill)
        .with_height(Size::FitContent)
        .with_layout_direction(Layout::Vertical)
        .with_padding(style.content_padding)
        .with_children(children);

    // Wrapper with overflow clipping for smooth height animation
    let content_wrapper = Node::new()
        .with_id(NodeId::new(format!("{}_content", id_str)))
        .with_width(Size::Fill)
        .with_z_index(ZIndex(0)) // Below header's ZIndex(1)
        .with_padding(Spacing::top(Size::lpx(
            style.header_padding.get_vertical()
                + style.title_font_size
                + style.content_padding.get_top(),
        )))
        .with_style(Style {
            fill_color: Some(style.header_idle_color),
            stroke: Some(Stroke::new(
                Size::lpx(style.stroke_idle_width),
                style.header_stroke_active_color,
            )),
            corner_shape: Some(style.corners),
            ..Default::default()
        })
        .with_height(if expanded {
            Size::FitContent
        } else {
            Size::lpx(0.0)
        })
        .with_overflow(Overflow::Hidden)
        .with_transition(Transition::standard())
        .with_child(content_panel);

    // Main container
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::FitContent)
        .with_layout_direction(Layout::Stack)
        .with_children(vec![header, content_wrapper])
}
