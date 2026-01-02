//! Demonstrates the node-based layout system with nested elements

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node, Shape,
    Size, Spacing, Stroke, StyledRect, TextContent, VerticalAlign,
};
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct LayoutExample {
    debug_options: DebugOptions,
}

impl ExampleApp for LayoutExample {
    fn new() -> Self {
        Self {
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Layout Nodes Demo"
    }

    fn window_size() -> (u32, u32) {
        (1200, 800)
    }

    fn build_ui(&mut self, width: f32, height: f32) -> Node {
        create_demo_ui(width, height, &self.debug_options)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn child() -> Node {
    Node::new().with_height(Size::Fill).with_shape(Shape::Rect(
        StyledRect::new(Default::default(), mocha::SURFACE1)
            .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
            .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
    ))
}

fn create_demo_ui(width: f32, height: f32, debug_options: &DebugOptions) -> Node {
    // Root container - full window with padding
    let root = Node::new()
        .with_padding(Spacing::all(Size::lpx(20.0)))
        .with_gap(Size::lpx(25.0))
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![
            // Header
            Node::new()
                .with_layout_direction(Layout::Horizontal)
                .with_gap(Size::lpx(10.0))
                .with_height(Size::fraction(0.15))
                .with_shape(Shape::Rect(
                    StyledRect::new(Default::default(), mocha::SURFACE0)
                        .with_corner_shape(CornerShape::Round(Size::lpx(50.0)))
                        .with_stroke(Stroke::new(Size::lpx(3.0), mocha::BLUE)),
                ))
                .with_padding(Spacing::all(Size::lpx(20.0)))
                .with_children(vec![
                    Node::new()
                        .with_width(Size::Relative(0.7))
                        .with_shape(Shape::Rect(
                            StyledRect::new(Default::default(), mocha::SURFACE1)
                                .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                                .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                        )),
                    Node::new()
                        .with_width(Size::Fill)
                        .with_layout_direction(Layout::Vertical)
                        .with_gap(Size::lpx(10.0))
                        .with_children(vec![
                            Node::new()
                                .with_shape(Shape::Rect(
                                    StyledRect::new(Default::default(), mocha::SURFACE1)
                                        .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                                        .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                                ))
                                .with_height(Size::Fill),
                            Node::new()
                                .with_shape(Shape::Rect(
                                    StyledRect::new(Default::default(), mocha::SURFACE1)
                                        .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                                        .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                                ))
                                .with_height(Size::Fill),
                        ]),
                ]),
            // Main content area - horizontal layout
            Node::new()
                .with_height(Size::fraction(0.75))
                .with_gap(Size::lpx(25.0))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![
                    // Left sidebar - 25% width
                    Node::new()
                        .with_width(Size::fraction(0.25))
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_gap(Size::lpx(10.0))
                        .with_shape(Shape::Rect(
                            StyledRect::new(Default::default(), mocha::SURFACE0)
                                .with_corner_shape(CornerShape::Round(Size::lpx(50.0)))
                                .with_stroke(Stroke::new(Size::lpx(3.0), mocha::MAUVE)),
                        ))
                        .with_layout_direction(Layout::Vertical)
                        .with_children(vec![
                            // Sidebar items
                            child(),
                            child(),
                            child(),
                            child(),
                            child(),
                            child(),
                            child(),
                            child(),
                            child(),
                            child(),
                        ]),
                    // Right of sidebar
                    Node::new()
                        .with_width(Size::fraction(0.75))
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_gap(Size::lpx(20.0))
                        .with_shape(Shape::Rect(
                            StyledRect::new(Default::default(), mocha::SURFACE0)
                                .with_corner_shape(CornerShape::Round(Size::lpx(50.0)))
                                .with_stroke(Stroke::new(Size::lpx(3.0), mocha::PEACH)),
                        ))
                        .with_layout_direction(Layout::Vertical)
                        .with_children(vec![
                            // Content cards in vertical layout
                            Node::new()
                                .with_height(Size::fraction(0.3))
                                .with_shape(Shape::Rect(
                                    StyledRect::new(Default::default(), mocha::SURFACE1)
                                        .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                                        .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                                )),
                            // Horizontal row of smaller cards
                            Node::new()
                                .with_height(Size::fraction(0.3))
                                .with_gap(Size::lpx(20.0))
                                .with_layout_direction(Layout::Horizontal)
                                .with_children(vec![
                                    Node::new().with_width(Size::fraction(0.5)).with_shape(
                                        Shape::Rect(
                                            StyledRect::new(Default::default(), mocha::SURFACE1)
                                                .with_corner_shape(CornerShape::Cut(Size::lpx(
                                                    30.0,
                                                )))
                                                .with_stroke(Stroke::new(
                                                    Size::lpx(3.0),
                                                    mocha::SURFACE2,
                                                )),
                                        ),
                                    ),
                                    Node::new().with_width(Size::fraction(0.5)).with_shape(
                                        Shape::Rect(
                                            StyledRect::new(Default::default(), mocha::SURFACE1)
                                                .with_corner_shape(CornerShape::Cut(Size::lpx(
                                                    30.0,
                                                )))
                                                .with_stroke(Stroke::new(
                                                    Size::lpx(3.0),
                                                    mocha::SURFACE2,
                                                )),
                                        ),
                                    ),
                                ]),
                            Node::new()
                                .with_height(Size::fraction(0.4))
                                .with_shape(Shape::Rect(
                                    StyledRect::new(Default::default(), mocha::SURFACE1)
                                        .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                                        .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                                )),
                        ]),
                ]),
            // Footer - 10% height with three Fill children laid out horizontally with gap
            Node::new()
                .with_height(Size::fraction(0.1))
                .with_padding(Spacing::all(Size::lpx(20.0)))
                .with_gap(Size::lpx(20.0))
                .with_layout_direction(Layout::Horizontal)
                .with_shape(Shape::Rect(
                    StyledRect::new(Default::default(), mocha::SURFACE0)
                        .with_corner_shape(CornerShape::Round(Size::lpx(50.0)))
                        .with_stroke(Stroke::new(Size::lpx(3.0), mocha::BLUE)),
                ))
                .with_children(vec![
                    Node::new().with_width(Size::Fill).with_shape(Shape::Rect(
                        StyledRect::new(Default::default(), mocha::SURFACE1)
                            .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                            .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                    )),
                    Node::new().with_width(Size::Fill).with_shape(Shape::Rect(
                        StyledRect::new(Default::default(), mocha::SURFACE1)
                            .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                            .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                    )),
                    Node::new().with_width(Size::Fill).with_shape(Shape::Rect(
                        StyledRect::new(Default::default(), mocha::SURFACE1)
                            .with_corner_shape(CornerShape::Round(Size::lpx(30.0)))
                            .with_stroke(Stroke::new(Size::lpx(3.0), mocha::SURFACE2)),
                    )),
                ]),
        ]);

    // Create help bar at the bottom
    let help_text = Node::new()
        .with_height(Size::lpx(30.0))
        .with_padding(Spacing::horizontal(Size::lpx(10.0)))
        .with_shape(Shape::Rect(StyledRect::new(
            Default::default(),
            mocha::SURFACE0,
        )))
        .with_content(Content::Text(
            TextContent::new(DEBUG_HELP_TEXT_ONELINE)
                .with_font_size(Size::lpx(16.0))
                .with_color(mocha::TEXT)
                .with_h_align(HorizontalAlign::Left)
                .with_v_align(VerticalAlign::Center),
        ));

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![root, help_text])
}

fn main() {
    run_example::<LayoutExample>();
}
