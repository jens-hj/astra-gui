//! Demonstrates all available corner shapes for rectangles
//! Corner shapes example
//!
//! Demonstrates different corner shapes (sharp, round, etc).
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit
//!
//! Note: Debug controls are shared across examples via `shared::debug_controls`.

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Overflow, Shape, Size, Spacing, Stroke, StyledRect, TextContent, VerticalAlign,
};
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct CornerShapesExample {
    debug_options: DebugOptions,
}

impl ExampleApp for CornerShapesExample {
    fn new() -> Self {
        Self {
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Corner Shapes Demo (Nodes)"
    }

    fn window_size() -> (u32, u32) {
        (1400, 900)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
        create_demo_ui()
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn card(stroke_color: Color, corner_shape: CornerShape, stroke_width: f32) -> Shape {
    Shape::Rect(
        StyledRect::new(Default::default(), mocha::SURFACE0)
            .with_corner_shape(corner_shape)
            .with_stroke(Stroke::new(Size::lpx(stroke_width), stroke_color)),
    )
}

fn create_demo_ui() -> Node {
    // Layout:
    // Root (padding)
    //  - Row 1: 3 equal-width cards
    //  - Row 2: 3 equal-width cards
    //
    // Sizes are chosen to roughly match the old shape-based showcase.
    let corner_size = 50.0;
    let stroke_width = 20.0;

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

    let content = Node::new()
        .with_padding(Spacing::all(Size::lpx(40.0)))
        .with_gap(Size::lpx(40.0))
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![
            Node::new()
                .with_height(Size::Fill)
                .with_gap(Size::lpx(40.0))
                .with_layout_direction(Layout::Horizontal)
                .with_overflow(Overflow::Visible)
                .with_children(vec![
                    // None
                    Node::new()
                        .with_width(Size::Fill)
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_shape(card(mocha::MAROON, CornerShape::None, stroke_width)),
                    // Round
                    Node::new()
                        .with_width(Size::Fill)
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_shape(card(
                            mocha::GREEN,
                            CornerShape::Round(Size::lpx(corner_size)),
                            stroke_width,
                        )),
                    // Cut
                    Node::new()
                        .with_width(Size::Fill)
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_shape(card(
                            mocha::BLUE,
                            CornerShape::Cut(Size::lpx(corner_size)),
                            stroke_width,
                        )),
                ]),
            Node::new()
                .with_height(Size::Fill)
                .with_gap(Size::lpx(40.0))
                .with_layout_direction(Layout::Horizontal)
                .with_overflow(Overflow::Visible)
                .with_children(vec![
                    // InverseRound
                    Node::new()
                        .with_width(Size::Fill)
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_shape(card(
                            mocha::YELLOW,
                            CornerShape::InverseRound(Size::lpx(corner_size)),
                            stroke_width,
                        )),
                    // Squircle low smoothness
                    Node::new()
                        .with_width(Size::Fill)
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_shape(card(
                            mocha::MAUVE,
                            CornerShape::Squircle {
                                radius: Size::lpx(corner_size),
                                smoothness: 0.5,
                            },
                            stroke_width,
                        )),
                    // Squircle high smoothness
                    Node::new()
                        .with_width(Size::Fill)
                        .with_padding(Spacing::all(Size::lpx(20.0)))
                        .with_shape(card(
                            mocha::TEAL,
                            CornerShape::Squircle {
                                radius: Size::lpx(corner_size),
                                smoothness: 3.0,
                            },
                            stroke_width,
                        )),
                ]),
        ]);

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![content, help_text])
}

fn main() {
    run_example::<CornerShapesExample>();
}
