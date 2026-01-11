//! Demonstrates text styling (bold, italic, font weight).
//!
//! This example tests the font weight and style features.

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, FontStyle, FontWeight,
    HorizontalAlign, Layout, Node, Shape, Size, Spacing, Stroke, StyledRect, TextContent,
    UiContext, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct TextStylingExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
}

impl ExampleApp for TextStylingExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Text Styling Demo"
    }

    fn window_size() -> (u32, u32) {
        (900, 700)
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn build_ui(&mut self, _ctx: &mut UiContext, width: f32, height: f32) -> Node {
        create_demo_ui(width, height, &self.debug_options)
    }
}

fn panel(fill: Color) -> Shape {
    Shape::Rect(
        StyledRect::new(Default::default(), fill)
            .with_corner_shape(CornerShape::Round(Size::lpx(18.0)))
            .with_stroke(Stroke::new(Size::lpx(2.0), mocha::SURFACE1)),
    )
}

fn text_sample(
    text: impl Into<String>,
    font_size: f32,
    color: Color,
    weight: FontWeight,
    style: FontStyle,
) -> Node {
    Node::new()
        .with_padding(Spacing::all(Size::lpx(12.0)))
        .with_shape(panel(mocha::SURFACE0))
        .with_content(Content::Text(
            TextContent::new(text)
                .with_font_size(Size::lpx(font_size))
                .with_color(color)
                .with_h_align(HorizontalAlign::Left)
                .with_v_align(VerticalAlign::Center)
                .with_font_weight(weight)
                .with_font_style(style),
        ))
}

fn create_demo_ui(_width: f32, _height: f32, _debug_options: &DebugOptions) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(24.0)))
        .with_gap(Size::lpx(18.0))
        .with_layout_direction(Layout::Vertical)
        .with_shape(Shape::Rect(
            StyledRect::new(Default::default(), mocha::BASE)
                .with_corner_shape(CornerShape::Round(Size::lpx(24.0)))
                .with_stroke(Stroke::new(Size::lpx(2.0), mocha::SURFACE0)),
        ))
        .with_children(vec![
            // Header
            Node::new()
                .with_height(Size::lpx(90.0))
                .with_padding(Spacing::all(Size::lpx(18.0)))
                .with_shape(panel(mocha::SURFACE0))
                .with_children(vec![
                    Node::new()
                        .with_height(Size::Fill)
                        .with_content(Content::Text(
                            TextContent::new("Text Styling Demo")
                                .with_font_size(Size::lpx(32.0))
                                .with_color(mocha::TEXT)
                                .with_h_align(HorizontalAlign::Left)
                                .with_v_align(VerticalAlign::Top)
                                .bold(),
                        )),
                    Node::new()
                        .with_height(Size::Fill)
                        .with_content(Content::Text(
                            TextContent::new("Testing font weight and italic styles")
                                .with_font_size(Size::lpx(16.0))
                                .with_color(mocha::SUBTEXT0)
                                .with_h_align(HorizontalAlign::Left)
                                .with_v_align(VerticalAlign::Bottom),
                        )),
                ]),
            // Main content area
            Node::new()
                .with_height(Size::Fill)
                .with_gap(Size::lpx(18.0))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![
                    // Left column: Font weights
                    Node::new()
                        .with_padding(Spacing::all(Size::lpx(16.0)))
                        .with_gap(Size::lpx(12.0))
                        .with_shape(panel(mocha::MANTLE))
                        .with_layout_direction(Layout::Vertical)
                        .with_children(vec![
                            Node::new()
                                .with_height(Size::lpx(30.0))
                                .with_content(Content::Text(
                                    TextContent::new("Font Weights")
                                        .with_font_size(Size::lpx(20.0))
                                        .with_color(mocha::TEXT)
                                        .with_h_align(HorizontalAlign::Left)
                                        .with_v_align(VerticalAlign::Center)
                                        .bold(),
                                )),
                            text_sample(
                                "Thin (100)",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Thin,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "Light (300)",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Light,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "Normal (400)",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Normal,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "Medium (500)",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Medium,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "SemiBold (600)",
                                22.0,
                                mocha::TEXT,
                                FontWeight::SemiBold,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "Bold (700)",
                                22.0,
                                mocha::PEACH,
                                FontWeight::Bold,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "Black (900)",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Black,
                                FontStyle::Normal,
                            ),
                        ]),
                    // Right column: Italic and combinations
                    Node::new()
                        .with_padding(Spacing::all(Size::lpx(16.0)))
                        .with_gap(Size::lpx(12.0))
                        .with_shape(panel(mocha::MANTLE))
                        .with_layout_direction(Layout::Vertical)
                        .with_children(vec![
                            Node::new()
                                .with_height(Size::lpx(30.0))
                                .with_content(Content::Text(
                                    TextContent::new("Italic & Combinations")
                                        .with_font_size(Size::lpx(20.0))
                                        .with_color(mocha::TEXT)
                                        .with_h_align(HorizontalAlign::Left)
                                        .with_v_align(VerticalAlign::Center)
                                        .bold(),
                                )),
                            text_sample(
                                "Normal text",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Normal,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "Italic text",
                                22.0,
                                mocha::SKY,
                                FontWeight::Normal,
                                FontStyle::Italic,
                            ),
                            text_sample(
                                "Bold text",
                                22.0,
                                mocha::PEACH,
                                FontWeight::Bold,
                                FontStyle::Normal,
                            ),
                            text_sample(
                                "Bold + Italic",
                                22.0,
                                mocha::MAUVE,
                                FontWeight::Bold,
                                FontStyle::Italic,
                            ),
                            text_sample(
                                "Light + Italic",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Light,
                                FontStyle::Italic,
                            ),
                            text_sample(
                                "SemiBold + Italic",
                                22.0,
                                mocha::TEXT,
                                FontWeight::SemiBold,
                                FontStyle::Italic,
                            ),
                            text_sample(
                                "Black + Italic",
                                22.0,
                                mocha::TEXT,
                                FontWeight::Black,
                                FontStyle::Italic,
                            ),
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
        ])
}

fn main() {
    run_example::<TextStylingExample>();
}
