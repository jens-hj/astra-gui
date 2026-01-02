//! Demonstrates rendering text nodes.
//!
//! This example exercises the `astra-gui-wgpu` backend's `Shape::Text` rendering path,
//! including alignment, padding/content rect behavior, and scissor-based clipping.

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Overflow, Shape, Size, Spacing, Stroke, StyledRect, TextContent, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct TextExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
}

impl ExampleApp for TextExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Text Demo"
    }

    fn window_size() -> (u32, u32) {
        (1100, 700)
    }

    fn build_ui(&mut self, width: f32, height: f32) -> Node {
        create_demo_ui(width, height, &self.debug_options)
    }

    fn text_measurer(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn label(
    text: impl Into<String>,
    font_size: f32,
    color: Color,
    h: HorizontalAlign,
    v: VerticalAlign,
) -> Node {
    Node::new().with_content(Content::Text(
        TextContent::new(text)
            .with_font_size(Size::lpx(font_size))
            .with_wrap(astra_gui::Wrap::None)
            .with_color(color)
            .with_h_align(h)
            .with_v_align(v),
    ))
}

fn panel(fill: Color) -> Shape {
    Shape::Rect(
        StyledRect::new(Default::default(), fill)
            .with_corner_shape(CornerShape::Round(Size::lpx(18.0)))
            .with_stroke(Stroke::new(Size::lpx(2.0), mocha::SURFACE1)),
    )
}

fn create_demo_ui(_width: f32, _height: f32, _debug_options: &DebugOptions) -> Node {
    // Root: whole window, a little padding
    let root = Node::new()
        .with_padding(Spacing::all(Size::lpx(24.0)))
        .with_gap(Size::lpx(18.0))
        .with_layout_direction(Layout::Vertical)
        .with_shape(Shape::Rect(
            StyledRect::new(Default::default(), Color::transparent())
                .with_corner_shape(CornerShape::Round(Size::lpx(24.0)))
                .with_stroke(Stroke::new(Size::lpx(2.0), mocha::SURFACE0)),
        ))
        .with_children(vec![
            // Header
            Node::new()
                .with_height(Size::lpx(110.0))
                .with_padding(Spacing::all(Size::lpx(18.0)))
                .with_shape(panel(mocha::SURFACE0))
                .with_children(vec![
                    // Title: large, left/top aligned
                    label(
                        "astra-gui: text nodes",
                        34.0,
                        mocha::TEXT,
                        HorizontalAlign::Left,
                        VerticalAlign::Top,
                    )
                    .with_height(Size::Fill),
                    // Subtitle: smaller
                    label(
                        "alignment, padding content rects, and clipping (when implemented)",
                        16.0,
                        mocha::SUBTEXT0,
                        HorizontalAlign::Left,
                        VerticalAlign::Bottom,
                    )
                    .with_height(Size::Fill),
                ]),
            // Main area: 2 columns
            Node::new()
                .with_height(Size::Fill)
                .with_gap(Size::lpx(18.0))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![
                    // Left: alignment grid
                    Node::new()
                        .with_width(Size::fraction(0.55))
                        .with_padding(Spacing::all(Size::lpx(16.0)))
                        .with_gap(Size::lpx(12.0))
                        .with_shape(panel(mocha::MANTLE))
                        .with_layout_direction(Layout::Vertical)
                        .with_children(vec![
                            label(
                                "Alignment grid (L/C/R × T/C/B)",
                                18.0,
                                mocha::TEXT,
                                HorizontalAlign::Left,
                                VerticalAlign::Top,
                            )
                            .with_height(Size::lpx(24.0)),
                            Node::new()
                                .with_height(Size::Fill)
                                .with_gap(Size::lpx(12.0))
                                .with_layout_direction(Layout::Vertical)
                                .with_children(vec![
                                    alignment_row("Top", VerticalAlign::Top),
                                    alignment_row("Center", VerticalAlign::Center),
                                    alignment_row("Bottom", VerticalAlign::Bottom),
                                ]),
                        ]),
                    // Right: varied sizes and clipping candidate
                    Node::new()
                        .with_width(Size::fraction(0.45))
                        .with_padding(Spacing::all(Size::lpx(16.0)))
                        .with_gap(Size::lpx(14.0))
                        .with_shape(panel(mocha::MANTLE))
                        .with_layout_direction(Layout::Vertical)
                        .with_children(vec![
                            label(
                                "Font sizes + padding behavior",
                                18.0,
                                mocha::TEXT,
                                HorizontalAlign::Left,
                                VerticalAlign::Top,
                            )
                            .with_height(Size::lpx(24.0)),
                            Node::new()
                                .with_height(Size::lpx(70.0))
                                .with_padding(Spacing::all(Size::lpx(10.0)))
                                .with_shape(panel(mocha::SURFACE0))
                                .with_children(vec![label(
                                    "Small (14px) in padded panel",
                                    14.0,
                                    mocha::SKY,
                                    HorizontalAlign::Left,
                                    VerticalAlign::Top,
                                )
                                .with_height(Size::Fill)]),
                            Node::new()
                                .with_height(Size::lpx(90.0))
                                .with_padding(Spacing::all(Size::lpx(10.0)))
                                .with_shape(panel(mocha::SURFACE0))
                                .with_children(vec![label(
                                    "Medium (22px)",
                                    22.0,
                                    mocha::PEACH,
                                    HorizontalAlign::Left,
                                    VerticalAlign::Center,
                                )
                                .with_height(Size::Fill)]),
                            Node::new()
                                .with_height(Size::lpx(120.0))
                                .with_padding(Spacing::all(Size::lpx(10.0)))
                                .with_shape(panel(mocha::SURFACE0))
                                .with_children(vec![label(
                                    "Large (42px)",
                                    42.0,
                                    mocha::MAUVE,
                                    HorizontalAlign::Left,
                                    VerticalAlign::Bottom,
                                )
                                .with_height(Size::Fill)]),
                            // Clipping candidate: a tight box with long text.
                            Node::new()
                                .with_height(Size::lpx(80.0))
                                .with_padding(Spacing::all(Size::lpx(10.0)))
                                .with_overflow(Overflow::Hidden)
                                .with_shape(Shape::Rect(
                                    StyledRect::new(Default::default(), mocha::CRUST)
                                        .with_corner_shape(CornerShape::Round(Size::lpx(14.0)))
                                        .with_stroke(Stroke::new(Size::lpx(2.0), mocha::SURFACE0)),
                                ))
                                .with_children(vec![label(
                                    "This string is intentionally very long to demonstrate clipping/scissoring. So let's make this even longer to make sure it clips.",
                                    18.0,
                                    mocha::RED,
                                    HorizontalAlign::Left,
                                    VerticalAlign::Top,
                                )
                                .with_height(Size::Fill)]),
                        ]),
                ]),
            // Help bar
            Node::new()
                .with_height(Size::lpx(30.0))
                .with_padding(Spacing::horizontal(Size::lpx(10.0)))
                .with_shape(panel(mocha::SURFACE0))
                .with_content(Content::Text(
                    TextContent::new(DEBUG_HELP_TEXT_ONELINE)
                        .with_font_size(Size::lpx(16.0))
                        .with_color(mocha::TEXT)
                        .with_h_align(HorizontalAlign::Left)
                        .with_v_align(VerticalAlign::Center),
                )),
        ]);

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![root])
}

fn alignment_cell(h: HorizontalAlign, v: VerticalAlign, label_text: &'static str) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(10.0)))
        .with_shape(Shape::Rect(
            StyledRect::new(Default::default(), mocha::SURFACE0)
                .with_corner_shape(CornerShape::Round(Size::lpx(14.0)))
                .with_stroke(Stroke::new(Size::lpx(2.0), mocha::SURFACE2)),
        ))
        .with_children(vec![
            label(label_text, 16.0, mocha::TEXT, h, v).with_height(Size::Fill)
        ])
}

fn alignment_row(v_name: &'static str, v: VerticalAlign) -> Node {
    Node::new()
        .with_height(Size::Fill)
        .with_gap(Size::lpx(12.0))
        .with_layout_direction(Layout::Horizontal)
        .with_children(vec![
            alignment_cell(
                HorizontalAlign::Left,
                v,
                match v_name {
                    "Top" => "L / Top",
                    "Center" => "L / Center",
                    "Bottom" => "L / Bottom",
                    _ => "L",
                },
            ),
            alignment_cell(
                HorizontalAlign::Center,
                v,
                match v_name {
                    "Top" => "C / Top",
                    "Center" => "C / Center",
                    "Bottom" => "C / Bottom",
                    _ => "C",
                },
            ),
            alignment_cell(
                HorizontalAlign::Right,
                v,
                match v_name {
                    "Top" => "R / Top",
                    "Center" => "R / Center",
                    "Bottom" => "R / Bottom",
                    _ => "R",
                },
            ),
        ])
}

fn main() {
    run_example::<TextExample>();
}
