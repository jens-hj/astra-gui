//! Tests stroke rendering with various widths and corner types

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Overflow, Shape, Size, Spacing, Stroke, StyledRect, TextContent, VerticalAlign,
};
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct StrokeExample {
    debug_options: DebugOptions,
}

impl ExampleApp for StrokeExample {
    fn new() -> Self {
        Self {
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Stroke Test"
    }

    fn window_size() -> (u32, u32) {
        (1600, 1000)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
        create_stroke_test_ui().with_zoom(2.0)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn rect_with_stroke(
    fill_color: Color,
    stroke_color: Color,
    corner_shape: CornerShape,
    stroke_width: f32,
) -> Shape {
    Shape::Rect(
        StyledRect::new(Default::default(), fill_color)
            .with_corner_shape(corner_shape)
            .with_stroke(Stroke::new(Size::lpx(stroke_width), stroke_color)),
    )
}

fn create_stroke_test_ui() -> Node {
    // Test matrix:
    // - Rows: Different stroke widths (0.5px, 1px, 3px, 10px, 20px)
    // - Columns: Different corner types (None, Round, Cut, InverseRound, Squircle)

    let stroke_widths = vec![0.0, 0.5, 1.0, 3.0, 10.0, 20.0];

    let corner_types = vec![
        ("None", CornerShape::None, mocha::BLUE),
        ("Round", CornerShape::Round(Size::lpx(30.0)), mocha::MAUVE),
        ("Cut", CornerShape::Cut(Size::lpx(30.0)), mocha::RED),
        (
            "InverseRound",
            CornerShape::InverseRound(Size::lpx(30.0)),
            mocha::YELLOW,
        ),
        (
            "Squircle",
            CornerShape::Squircle {
                radius: Size::lpx(30.0),
                smoothness: 1.0,
            },
            mocha::GREEN,
        ),
    ];

    let mut rows = vec![];

    for stroke_width in stroke_widths {
        let mut cells = vec![];

        for (_, corner_shape, corner_color) in &corner_types {
            cells.push(
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::lpx(100.0))
                    .with_shape(rect_with_stroke(
                        mocha::SURFACE0,
                        *corner_color,
                        *corner_shape,
                        stroke_width,
                    )),
            );
        }

        rows.push(
            Node::new()
                .with_height(Size::lpx(100.0))
                .with_gap(Size::ppx(60.0))
                .with_layout_direction(Layout::Horizontal)
                .with_children(cells),
        );
    }

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
        .with_padding(Spacing::all(Size::ppx(60.0)))
        .with_gap(Size::ppx(60.0))
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_overflow(Overflow::Visible)
        .with_children(rows);

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![content, help_text])
}

fn main() {
    println!("Stroke Test - Testing various stroke widths on all corner types");
    println!("Rows: 0.5px, 1px, 3px, 10px, 20px stroke widths");
    println!("Columns: None, Round, Cut, InverseRound, Squircle");
    println!();

    run_example::<StrokeExample>();
}
