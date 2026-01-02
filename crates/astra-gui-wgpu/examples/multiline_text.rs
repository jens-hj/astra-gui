//! Demonstrates multi-line text rendering with different wrapping modes.
//!
//! This example shows:
//! - Explicit newlines in text
//! - Word wrapping, glyph wrapping, and mixed wrapping modes
//! - Different line heights
//! - Text alignment with multi-line content
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Overflow, Shape, Size, Spacing, Style, StyledRect, TextContent, VerticalAlign, Wrap,
};
use astra_gui_text::Engine as TextEngine;
use shared::{run_example, ExampleApp};

struct MultilineTextExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
}

impl ExampleApp for MultilineTextExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Multi-Line Text Example - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (1200, 900)
    }

    fn build_ui(&mut self, width: f32, height: f32) -> Node {
        build_ui(width, height, &self.debug_options)
    }

    fn text_measurer(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn build_ui(_width: f32, _height: f32, _debug_options: &DebugOptions) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_zoom(2.0)
        .with_children(vec![
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_layout_direction(Layout::Vertical)
                .with_padding(Spacing::all(Size::lpx(30.0)))
                .with_gap(Size::lpx(20.0))
                .with_style(Style {
                    fill_color: Some(mocha::BASE),
                    ..Default::default()
                })
                .with_children(vec![
                    // Title
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::lpx(40.0))
                        .with_content(Content::Text(
                            TextContent::new("Multi-Line Text & Wrapping Examples")
                                .with_font_size(Size::lpx(36.0))
                                .with_color(mocha::TEXT)
                                .with_h_align(HorizontalAlign::Center),
                        )),
                    // Examples grid
                    Node::new()
                        .with_width(Size::Fill)
                        .with_height(Size::Fill)
                        .with_layout_direction(Layout::Horizontal)
                        .with_gap(Size::lpx(20.0))
                        .with_children(vec![
                            // Left column
                            column(vec![
                                example_box(
                                    "Explicit Newlines",
                                    "Line 1\nLine 2\nLine 3\nLine 4",
                                    Wrap::None,
                                    Size::FitContent,
                                    mocha::BLUE,
                                    1.2,
                                ),
                                example_box(
                                    "Word Wrap (default)",
                                    "This is a longer text that will automatically wrap at word boundaries when the container width is constrained. It's the default behavior.",
                                    Wrap::Word,
                                    Size::lpx(250.0),
                                    mocha::GREEN,
                                    1.2,
                                ),
                                example_box(
                                    "Line Height 3.0x",
                                    "This text has\nincreased line\nheight spacing\nfor better readability",
                                    Wrap::None,
                                    Size::FitContent,
                                    mocha::YELLOW,
                                    3.0,
                                ),
                            ]),
                            // Middle column
                            column(vec![
                                example_box(
                                    "No Wrap (Overflow)",
                                    "This is a very long text that will overflow the container instead of wrapping because wrapping is disabled.",
                                    Wrap::None,
                                    Size::lpx(250.0),
                                    mocha::RED,
                                    1.2,
                                ),
                                example_box(
                                    "Glyph Wrap",
                                    "Verylongwordthatwillwrapatanycharacterboundaryinsteadofwordswhenspaceisunavailable",
                                    Wrap::Glyph,
                                    Size::lpx(250.0),
                                    mocha::MAUVE,
                                    1.2,
                                ),
                                example_box(
                                    "WordOrGlyph Wrap",
                                    "Normal words wrap normally, but verylongwordswithoutspacesbreakanywheretofit",
                                    Wrap::WordOrGlyph,
                                    Size::lpx(250.0),
                                    mocha::PEACH,
                                    1.2,
                                ),
                            ]),
                            // Right column
                            column(vec![
                                alignment_example("Left Aligned\nMultiple Lines\nH: Left", HorizontalAlign::Left),
                                alignment_example("Center Aligned\nMultiple Lines\nH: Center", HorizontalAlign::Center),
                                alignment_example("Right Aligned\nMultiple Lines\nH: Right", HorizontalAlign::Right),
                            ]),
                        ]),
                ]),
        // Help text at bottom
        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::lpx(30.0))
            .with_padding(Spacing::horizontal(Size::lpx(10.0)))
            .with_style(Style {
                fill_color: Some(mocha::SURFACE0),
                ..Default::default()
            })
            .with_content(Content::Text(
                TextContent::new(
                    "M:Margins | P:Padding | B:Borders | C:Content | R:ClipRects | G:Gaps | O:Origins | T:Text | D:All | S:RenderMode | ESC:Exit",
                )
                .with_font_size(Size::lpx(16.0))
                .with_color(mocha::TEXT)
                .with_h_align(HorizontalAlign::Left)
                .with_v_align(VerticalAlign::Center),
            )),
    ])
}

fn column(children: Vec<Node>) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_gap(Size::lpx(15.0))
        .with_children(children)
}

fn example_box(
    title: &str,
    text: &str,
    wrap: Wrap,
    width: Size,
    color: Color,
    line_height: f32,
) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::FitContent)
        .with_layout_direction(Layout::Vertical)
        .with_gap(Size::lpx(8.0))
        .with_padding(Spacing::all(Size::lpx(15.0)))
        .with_overflow(Overflow::Hidden)
        .with_style(Style {
            fill_color: Some(mocha::SURFACE0),
            corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
            ..Default::default()
        })
        .with_children(vec![
            // Title
            Node::new()
                .with_width(Size::Fill)
                .with_content(Content::Text(
                    TextContent::new(title)
                        .with_font_size(Size::lpx(24.0))
                        .with_color(color),
                )),
            // Example text
            Node::new()
                .with_width(width)
                .with_padding(Spacing::all(Size::lpx(10.0)))
                .with_style(Style {
                    fill_color: Some(mocha::MANTLE),
                    corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                    ..Default::default()
                })
                .with_content(Content::Text(
                    TextContent::new(text)
                        .with_font_size(Size::lpx(16.0))
                        .with_color(color)
                        .with_wrap(wrap)
                        .with_line_height(line_height),
                )),
        ])
}

fn alignment_example(text: &str, h_align: HorizontalAlign) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(15.0)))
        .with_shape(Shape::Rect(
            StyledRect::new(Default::default(), mocha::SURFACE0)
                .with_corner_shape(CornerShape::Round(Size::lpx(8.0))),
        ))
        .with_content(Content::Text(
            TextContent::new(text)
                .with_font_size(Size::lpx(18.0))
                .with_color(mocha::TEAL)
                .with_h_align(h_align)
                .with_v_align(VerticalAlign::Center),
        ))
}

fn main() {
    run_example::<MultilineTextExample>();
}
