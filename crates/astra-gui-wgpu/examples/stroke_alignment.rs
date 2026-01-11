//! Demonstrates stroke alignment options (Inset, Centered, Outset, Custom)
//!
//! Shows how strokes can be positioned relative to the shape boundary:
//! - Inset: Stroke inside the shape (default)
//! - Centered: Stroke centered on the edge (half inside, half outside)
//! - Outset: Stroke outside the shape
//! - Custom: Custom pixel offset from the edge
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Orientation, Place, Shape, Size, Spacing, Stroke, StrokeAlignment, Style, StyledRect,
    StyledTriangle, TextContent, TriangleSpec, UiContext, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct StrokeAlignmentExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
}

impl ExampleApp for StrokeAlignmentExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Stroke Alignment Demo"
    }

    fn window_size() -> (u32, u32) {
        (1400, 900)
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn build_ui(&mut self, _ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        create_demo_ui().with_zoom(2.0)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn main() {
    run_example::<StrokeAlignmentExample>();
}

fn panel(fill: Color) -> Shape {
    Shape::Rect(StyledRect::new(Default::default(), fill))
}

fn create_demo_ui() -> Node {
    let stroke_width = 10.0;
    let cell_size = 100.0;

    // Create help bar
    let help_text = Node::new()
        .with_width(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(10.0)))
        .with_shape(panel(mocha::SURFACE0))
        .with_place(Place::Alignment {
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Bottom,
        })
        .with_content(Content::Text(
            TextContent::new(DEBUG_HELP_TEXT_ONELINE)
                .with_font_size(Size::lpx(16.0))
                .with_color(mocha::TEXT)
                .with_h_align(HorizontalAlign::Left)
                .with_v_align(VerticalAlign::Top),
        ));

    // Main content
    let content = Node::new()
        .with_padding(Spacing::all(Size::lpx(20.0)))
        .with_gap(Size::lpx(20.0))
        .with_layout_direction(Layout::Vertical)
        .with_shape(panel(mocha::BASE))
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![
            // Title
            title_text("Stroke Alignment Modes"),
            subtitle_text("Same 10px stroke with different alignments (fill: blue, stroke: pink)"),
            // Grid rows
            labeled_row(
                "Sharp Corners",
                demo_row(CornerShape::None, stroke_width, cell_size),
            ),
            labeled_row(
                "Round Corners",
                demo_row(CornerShape::Round(Size::lpx(20.0)), stroke_width, cell_size),
            ),
            labeled_row(
                "Cut Corners",
                demo_row(CornerShape::Cut(Size::lpx(15.0)), stroke_width, cell_size),
            ),
            labeled_row(
                "Squircle",
                demo_row(
                    CornerShape::Squircle {
                        radius: Size::lpx(20.0),
                        smoothness: 0.6,
                    },
                    stroke_width,
                    cell_size,
                ),
            ),
            labeled_row("Triangle", triangle_row(stroke_width)),
        ]);

    // Root layout
    Node::new()
        .with_layout_direction(Layout::Vertical)
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![content, help_text])
}

fn demo_row(corner_shape: CornerShape, stroke_width: f32, cell_size: f32) -> Node {
    Node::new()
        .with_layout_direction(Layout::Horizontal)
        .with_gap(Size::lpx(20.0))
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![
            alignment_cell("Inset", corner_shape, stroke_width, StrokeAlignment::Inset),
            alignment_cell(
                "Centered",
                corner_shape,
                stroke_width,
                StrokeAlignment::Centered,
            ),
            alignment_cell(
                "Outset",
                corner_shape,
                stroke_width,
                StrokeAlignment::Outset,
            ),
            alignment_cell(
                "Custom(15px)",
                corner_shape,
                stroke_width,
                StrokeAlignment::Custom(15.0),
            ),
            alignment_cell(
                "Custom(30px)",
                corner_shape,
                stroke_width,
                StrokeAlignment::Custom(30.0),
            ),
            alignment_cell(
                "Custom(-30px)",
                corner_shape,
                stroke_width,
                StrokeAlignment::Custom(-30.0),
            ),
        ])
}

fn triangle_row(stroke_width: f32) -> Node {
    Node::new()
        .with_layout_direction(Layout::Horizontal)
        .with_gap(Size::lpx(20.0))
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![
            triangle_cell("Inset", stroke_width, StrokeAlignment::Inset),
            triangle_cell("Centered", stroke_width, StrokeAlignment::Centered),
            triangle_cell("Outset", stroke_width, StrokeAlignment::Outset),
            triangle_cell("Custom(10px)", stroke_width, StrokeAlignment::Custom(10.0)),
        ])
}

fn custom_offset_row(stroke_width: f32, cell_size: f32) -> Node {
    Node::new()
        .with_layout_direction(Layout::Horizontal)
        .with_gap(Size::lpx(20.0))
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![
            alignment_cell(
                "Custom(-15px)\nFar Inside",
                CornerShape::Round(Size::lpx(15.0)),
                stroke_width,
                StrokeAlignment::Custom(-15.0),
            ),
            alignment_cell(
                "Custom(-4px)\nSlightly Inside",
                CornerShape::Round(Size::lpx(15.0)),
                stroke_width,
                StrokeAlignment::Custom(-4.0),
            ),
            alignment_cell(
                "Custom(4px)\nSlightly Outside",
                CornerShape::Round(Size::lpx(15.0)),
                stroke_width,
                StrokeAlignment::Custom(4.0),
            ),
            alignment_cell(
                "Custom(15px)\nFar Outside",
                CornerShape::Round(Size::lpx(15.0)),
                stroke_width,
                StrokeAlignment::Custom(15.0),
            ),
            alignment_cell(
                "Custom(30px)\nFar Outside",
                CornerShape::Round(Size::lpx(15.0)),
                stroke_width,
                StrokeAlignment::Custom(30.0),
            ),
            alignment_cell(
                "Custom(-30px)\nFar Inside",
                CornerShape::Round(Size::lpx(15.0)),
                stroke_width,
                StrokeAlignment::Custom(-30.0),
            ),
        ])
}

fn alignment_cell(
    label: &str,
    corner_shape: CornerShape,
    stroke_width: f32,
    alignment: StrokeAlignment,
) -> Node {
    Node::new()
        .with_gap(Size::lpx(8.0))
        .with_layout_direction(Layout::Vertical)
        .with_h_align(HorizontalAlign::Center)
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![
            label_text(label),
            Node::new()
                .with_width(Size::rel(0.8))
                .with_height(Size::rel(0.8))
                .with_style(Style {
                    fill_color: Some(mocha::RED),
                    corner_shape: Some(corner_shape),
                    stroke: Some(
                        Stroke::new(Size::lpx(stroke_width), mocha::GREEN)
                            .with_alignment(alignment),
                    ),
                    ..Default::default()
                }),
        ])
}

fn triangle_cell(label: &str, stroke_width: f32, alignment: StrokeAlignment) -> Node {
    Node::new()
        .with_gap(Size::lpx(8.0))
        .with_layout_direction(Layout::Vertical)
        .with_h_align(HorizontalAlign::Center)
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![
            label_text(label),
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_h_align(HorizontalAlign::Center)
                .with_v_align(VerticalAlign::Center)
                .with_shape(Shape::Triangle(
                    StyledTriangle::new(
                        Default::default(),
                        TriangleSpec::Isosceles {
                            orientation: Orientation::Up,
                        },
                        mocha::CRUST,
                    )
                    .with_stroke(
                        Stroke::new(Size::lpx(stroke_width), mocha::BLUE).with_alignment(alignment),
                    ),
                )),
        ])
}

fn labeled_row(label: &str, content: Node) -> Node {
    Node::new()
        .with_gap(Size::lpx(10.0))
        .with_layout_direction(Layout::Vertical)
        .with_height(Size::Fill)
        .with_width(Size::Fill)
        .with_children(vec![section_text(label), content])
}

fn title_text(text: &str) -> Node {
    Node::new().with_content(Content::Text(
        TextContent::new(text)
            .with_font_size(Size::lpx(24.0))
            .with_color(mocha::TEXT),
    ))
}

fn subtitle_text(text: &str) -> Node {
    Node::new().with_content(Content::Text(
        TextContent::new(text)
            .with_font_size(Size::lpx(14.0))
            .with_color(mocha::SUBTEXT0),
    ))
}

fn section_text(text: &str) -> Node {
    Node::new().with_content(Content::Text(
        TextContent::new(text)
            .with_font_size(Size::lpx(16.0))
            .with_color(mocha::TEXT),
    ))
}

fn label_text(text: &str) -> Node {
    Node::new()
        .with_margin(Spacing::bottom(Size::lpx(30.0)))
        .with_content(Content::Text(
            TextContent::new(text)
                .with_font_size(Size::lpx(12.0))
                .with_color(mocha::SUBTEXT1),
        ))
}
