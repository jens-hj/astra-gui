//! Demonstrates the built-in Catppuccin color schemes

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::{frappe, latte, macchiato, mocha},
    Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node, Shape, Size, Spacing,
    StyledRect, TextContent, UiContext, VerticalAlign,
};
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct CatppuccinExample {
    debug_options: DebugOptions,
}

impl ExampleApp for CatppuccinExample {
    fn new() -> Self {
        Self {
            debug_options: DebugOptions::default(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Catppuccin Themes"
    }

    fn window_size() -> (u32, u32) {
        (1600, 1200)
    }

    fn build_ui(&mut self, _ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        create_demo_ui()
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn theme_card(
    name: &str,
    crust: Color,
    base: Color,
    text: Color,
    mut colors: Vec<(&'static str, Color)>,
) -> Node {
    while colors.len() % 5 != 0 {
        colors.push(("", Color::transparent()));
    }

    let mut rows = vec![];
    for chunk in colors.chunks(5) {
        let row = Node::new()
            .with_height(Size::fraction(1.0 / 5.0))
            .with_layout_direction(Layout::Horizontal)
            .with_gap(Size::lpx(10.0))
            .with_children(
                chunk
                    .iter()
                    .map(|&(n, c)| color_swatch(n, c, base, text))
                    .collect(),
            );
        rows.push(row);
    }

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(20.0)))
        .with_shape(Shape::Rect(StyledRect::new(Default::default(), crust)))
        .with_layout_direction(Layout::Vertical)
        .with_gap(Size::lpx(15.0))
        .with_children(vec![
            // Title
            Node::new()
                .with_height(Size::lpx(40.0))
                .with_content(Content::Text(
                    TextContent::new(name)
                        .with_font_size(Size::lpx(32.0))
                        .with_color(text)
                        .with_h_align(HorizontalAlign::Center)
                        .with_v_align(VerticalAlign::Center),
                )),
            // Content box (Panel)
            Node::new()
                .with_height(Size::Fill)
                .with_padding(Spacing::all(Size::lpx(17.5)))
                .with_shape(Shape::Rect(
                    StyledRect::new(Default::default(), base)
                        .with_corner_shape(CornerShape::Cut(Size::lpx(40.0))),
                ))
                .with_layout_direction(Layout::Vertical)
                .with_gap(Size::lpx(10.0))
                .with_children(rows),
        ])
}

fn color_swatch(name: &str, color: Color, base_color: Color, text_color: Color) -> Node {
    if name.is_empty() {
        return Node::new().with_width(Size::Fill).with_height(Size::Fill);
    }

    let contrast_base = color.contrast_ratio(&base_color);
    let contrast_text = color.contrast_ratio(&text_color);

    let final_text_color = if contrast_base > contrast_text {
        base_color
    } else {
        text_color
    };

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_shape(Shape::Rect(
            StyledRect::new(Default::default(), color)
                .with_corner_shape(CornerShape::Cut(Size::lpx(30.0))),
        ))
        .with_content(Content::Text(
            TextContent::new(name)
                .with_font_size(Size::lpx(24.0))
                .with_color(final_text_color)
                .with_h_align(HorizontalAlign::Center)
                .with_v_align(VerticalAlign::Center),
        ))
}

fn create_demo_ui() -> Node {
    macro_rules! colors {
        ($m:ident) => {
            vec![
                ("Rosewater", $m::ROSEWATER),
                ("Flamingo", $m::FLAMINGO),
                ("Pink", $m::PINK),
                ("Mauve", $m::MAUVE),
                ("Red", $m::RED),
                ("Maroon", $m::MAROON),
                ("Peach", $m::PEACH),
                ("Yellow", $m::YELLOW),
                ("Green", $m::GREEN),
                ("Teal", $m::TEAL),
                ("Sky", $m::SKY),
                ("Sapphire", $m::SAPPHIRE),
                ("Blue", $m::BLUE),
                ("Lavender", $m::LAVENDER),
                ("Text", $m::TEXT),
                ("Subtext1", $m::SUBTEXT1),
                ("Subtext0", $m::SUBTEXT0),
                ("Overlay2", $m::OVERLAY2),
                ("Overlay1", $m::OVERLAY1),
                ("Overlay0", $m::OVERLAY0),
                ("Surface2", $m::SURFACE2),
                ("Surface1", $m::SURFACE1),
                ("Surface0", $m::SURFACE0),
                // ("Base", $m::BASE),
                ("Mantle", $m::MANTLE),
                ("Crust", $m::CRUST),
            ]
        };
    }

    // Root container - 2x2 grid
    let root = Node::new()
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![
            // Top Row
            Node::new()
                .with_height(Size::fraction(0.5))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![
                    theme_card(
                        "Latte",
                        latte::CRUST,
                        latte::BASE,
                        latte::TEXT,
                        colors!(latte),
                    ),
                    theme_card(
                        "Frappe",
                        frappe::CRUST,
                        frappe::BASE,
                        frappe::TEXT,
                        colors!(frappe),
                    ),
                ]),
            // Bottom Row
            Node::new()
                .with_height(Size::fraction(0.5))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![
                    theme_card(
                        "Macchiato",
                        macchiato::CRUST,
                        macchiato::BASE,
                        macchiato::TEXT,
                        colors!(macchiato),
                    ),
                    theme_card(
                        "Mocha",
                        mocha::CRUST,
                        mocha::BASE,
                        mocha::TEXT,
                        colors!(mocha),
                    ),
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

    // Overlay help text on top of the grid
    // Actually, let's put it below the grid, but the grid takes full height.
    // We can make the grid take (Fill - 30px) and help text 30px.

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_layout_direction(Layout::Vertical)
        .with_children(vec![
            root.with_height(Size::Fill), // Grid takes remaining space
            help_text,
        ])
        .with_zoom(1.5)
}

fn main() {
    run_example::<CatppuccinExample>();
}
