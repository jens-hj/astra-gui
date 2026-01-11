//! Demonstrates the fractional sizing (fr units) feature

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Size, Spacing, Stroke, Style, TextContent, UiContext, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct FractionalExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
}

impl ExampleApp for FractionalExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Fractional Sizing Demo"
    }

    fn window_size() -> (u32, u32) {
        (1200, 800)
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn build_ui(&mut self, _ctx: &mut UiContext, width: f32, height: f32) -> Node {
        create_fractional_demo(width, height, &self.debug_options)
    }
}

fn create_fractional_demo(_width: f32, _height: f32, _debug_options: &DebugOptions) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_style(Style {
            fill_color: Some(mocha::BASE),
            ..Default::default()
        })
        .with_padding(Spacing::all(Size::lpx(20.0)))
        .with_gap(Size::lpx(30.0))
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![
            // Example 1: Basic fractional distribution [fr(1), fr(2), fr(3)]
            section_with_label(
                "Example 1: [fr(1), fr(2), fr(3)] - proportional (1:2:3 ratio)",
                "Total weight: 6 → fr(1)=1/6, fr(2)=2/6, fr(3)=3/6 of available width",
                vec![
                    colored_box(Size::fr(1.0), mocha::BLUE, "fr(1)"),
                    colored_box(Size::fr(2.0), mocha::TEAL, "fr(2)"),
                    colored_box(Size::fr(3.0), mocha::GREEN, "fr(3)"),
                ],
            ),
            // Example 2: Fill and Fractional mix [Fill, fr(2)]
            section_with_label(
                "Example 2: [Fill, fr(2)] - Fill is equivalent to fr(1)",
                "Total weight: 3 → Fill=1/3, fr(2)=2/3 of available width",
                vec![
                    colored_box(Size::Fill, mocha::MAUVE, "Fill"),
                    colored_box(Size::fr(2.0), mocha::TEAL, "fr(2)"),
                ],
            ),
            // Example 3: Fractional with fixed [lpx(200), fr(1), fr(2)]
            section_with_label(
                "Example 3: [200px, fr(1), fr(2)] - fixed size allocated first",
                "200px takes fixed space, then fr(1)=1/3 and fr(2)=2/3 of remainder",
                vec![
                    colored_box(Size::lpx(200.0), mocha::YELLOW, "200px"),
                    colored_box(Size::fr(1.0), mocha::BLUE, "fr(1)"),
                    colored_box(Size::fr(2.0), mocha::TEAL, "fr(2)"),
                ],
            ),
            // Example 4: All Fill children [Fill, Fill, Fill]
            section_with_label(
                "Example 4: [Fill, Fill, Fill]",
                "Each Fill=fr(1), total weight: 3 → equal thirds (1/3 each)",
                vec![
                    colored_box(Size::Fill, mocha::MAUVE, "Fill"),
                    colored_box(Size::Fill, mocha::MAUVE, "Fill"),
                    colored_box(Size::Fill, mocha::MAUVE, "Fill"),
                ],
            ),
            // Example 5: Combining with FitContent [FitContent, Fill, fr(2)]
            section_with_label(
                "Example 5: [FitContent, Fill, fr(2)]",
                "FitContent takes the space it needs, then Fill = 1/3, fr(2) = 2/3 of remainder",
                vec![
                    colored_box(Size::FitContent, mocha::MAROON, "FitContent"),
                    colored_box(Size::Fill, mocha::MAUVE, "Fill"),
                    colored_box(Size::fr(2.0), mocha::TEAL, "fr(2)"),
                ],
            ),
            // Example 6: Combining with FitContent and Logical pixels [FitContent, Fill, fr(2), 700px]
            section_with_label(
                "Example 6: [FitContent, Fill, fr(2), 700px]",
                "FitContent takes the space it needs, 700px fixed, then Fill = 1/3, fr(2) = 2/3 of remainder",
                vec![
                    colored_box(Size::FitContent, mocha::MAROON, "FitContent"),
                    colored_box(Size::Fill, mocha::MAUVE, "Fill"),
                    colored_box(Size::fr(2.0), mocha::TEAL, "fr(2)"),
                    colored_box(Size::lpx(700.0), mocha::YELLOW, "700px"),
                ],
            ),
            // Example 7: Combining with FitContent and Logical pixels [FitContent, Fill, fr(2), 700px, 200px]
            section_with_label(
                "Example 7: [FitContent, Fill, fr(2), 700px, 200px]",
                "FitContent takes the space it needs, 700px (logical) and 200px (physical) fixed, then Fill = 1/3, fr(2) = 2/3 of remainder",
                vec![
                    colored_box(Size::FitContent, mocha::MAROON, "FitContent"),
                    colored_box(Size::Fill, mocha::MAUVE, "Fill"),
                    colored_box(Size::fr(2.0), mocha::TEAL, "fr(2)"),
                    colored_box(Size::lpx(700.0), mocha::YELLOW, "700px"),
                    colored_box(Size::ppx(200.0), mocha::PEACH, "200px"),
                ],
            ),
            // Help text
            Node::new()
                .with_width(Size::Fill)
                .with_padding(Spacing::all(Size::lpx(8.0)))
                .with_content(Content::Text(
                    TextContent::new(DEBUG_HELP_TEXT_ONELINE)
                        .with_font_size(Size::lpx(14.0))
                        .with_color(mocha::SUBTEXT0),
                )),
        ])
}

fn section_with_label(label: &str, description: &str, children: Vec<Node>) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::fr(1.0))
        .with_layout_direction(Layout::Vertical)
        .with_gap(Size::lpx(10.0))
        .with_children(vec![
            // Label
            Node::new()
                .with_width(Size::Fill)
                .with_content(Content::Text(
                    TextContent::new(label)
                        .with_font_size(Size::lpx(32.0))
                        .with_color(mocha::TEXT),
                )),
            // Description
            Node::new()
                .with_width(Size::Fill)
                .with_margin(Spacing::bottom(Size::lpx(15.0)))
                .with_content(Content::Text(
                    TextContent::new(description)
                        .with_font_size(Size::lpx(18.0))
                        .with_color(mocha::SUBTEXT0),
                )),
            // Container for children
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_layout_direction(Layout::Horizontal)
                .with_gap(Size::lpx(10.0))
                .with_children(children),
        ])
}

fn colored_box(width: Size, color: Color, text: &str) -> Node {
    Node::new()
        .with_width(width)
        .with_height(Size::Fill)
        .with_style(Style {
            fill_color: Some(color),
            corner_shape: Some(CornerShape::Round(Size::lpx(20.0))),
            ..Default::default()
        })
        .with_padding(Spacing::all(Size::lpx(10.0)))
        .with_h_align(HorizontalAlign::Center)
        .with_v_align(VerticalAlign::Center)
        .with_child(
            Node::new()
                .with_width(Size::FitContent)
                .with_height(Size::FitContent)
                .with_content(Content::Text(
                    TextContent::new(text)
                        .with_font_size(Size::lpx(32.0))
                        .with_color(mocha::CRUST)
                        .with_h_align(HorizontalAlign::Center),
                )),
        )
}

fn main() {
    run_example::<FractionalExample>();
}
