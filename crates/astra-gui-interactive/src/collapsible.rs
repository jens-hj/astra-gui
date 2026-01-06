//! Collapsible container component for interactive UI
//!
//! Provides an expandable/collapsible container with a clickable header,
//! similar to egui's collapsing header. Supports smooth animations and nesting.

use std::f32::consts::PI;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, HorizontalAlign, Layout, Node, NodeId,
    Orientation, Overflow, Shape, Size, Spacing, Stroke, Style, TextContent, Transition,
    TriangleSpec, VerticalAlign, ZIndex,
};
use astra_gui_macros::WithBuilders;
use astra_gui_wgpu::{InteractionEvent, TargetedEvent};

/// Visual styling for a collapsible container<
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

/// Create a collapsible container node
///
/// The collapsible container features a clickable header with a title and rotating
/// triangle indicator, plus a content area that smoothly expands/collapses with
/// slide and fade animations.
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
///
/// # Example
/// ```rust
/// use astra_gui_interactive::{button, collapsible, collapsible_clicked, ButtonStyle, CollapsibleStyle};
///
/// let mut expanded = true;
/// let events = Vec::new();
///
/// let ui = collapsible(
///     "settings",
///     "Settings",
///     expanded,
///     false,
///     vec![
///         button("btn1", "Button 1", false, &ButtonStyle::default()),
///         button("btn2", "Button 2", false, &ButtonStyle::default()),
///     ],
///     &CollapsibleStyle::default(),
/// );
///
/// // Handle clicks
/// if collapsible_clicked("settings", &events) {
///     expanded = !expanded;
/// }
/// ```
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
    // NOTE: No hover/active styles to prevent mouse interaction from interrupting animation
    // Z-index below header so translated overlap doesn't intercept clicks
    let content_wrapper = Node::new()
        .with_id(NodeId::new(format!("{}_content", id_str)))
        .with_width(Size::Fill)
        .with_z_index(ZIndex(0)) // Below header's ZIndex(1)
        // .with_translation(Translation::y(Size::lpx(-style.border_radius * 2.0)))
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
///
/// # Example
/// ```rust
/// use astra_gui_interactive::collapsible_clicked;
///
/// let events = Vec::new();
/// let mut expanded = true;
///
/// if collapsible_clicked("settings", &events) {
///     expanded = !expanded;
/// }
/// ```
pub fn collapsible_clicked(collapsible_id: &str, events: &[TargetedEvent]) -> bool {
    let header_id = format!("{}_header", collapsible_id);
    let indicator_id = format!("{}_indicator", collapsible_id);

    events.iter().any(|e| {
        matches!(e.event, InteractionEvent::Click { .. })
            && (e.target.as_str() == header_id || e.target.as_str() == indicator_id)
    })
}
