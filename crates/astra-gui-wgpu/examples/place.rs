//! Example: `Place`
//!
//! Demonstrates per-child placement overrides inside a `Layout::Stack` container using `Place`.
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit
//!
//! This example places multiple panels in different corners of the same stack parent:
//! - top-left via `Place::Alignment { Left, Top }`
//! - top-right via `Place::Alignment { Right, Top }`
//! - bottom-left via `Place::Alignment { Left, Bottom }`
//! - bottom-right via `Place::Alignment { Right, Bottom }`
//! - an absolute-positioned badge via `Place::Absolute { x, y }`

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Place, Size, Spacing, Stroke, Style, TextContent, UiContext, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::{run_example, ExampleApp};

struct PlaceExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    zoom_level: f32,
}

impl PlaceExample {
    fn panel(title: &str, body: &str, accent: Color) -> Node {
        let title = Node::new().with_content(Content::Text(
            TextContent::new(title)
                .with_font_size(Size::lpx(18.0))
                .with_color(mocha::TEXT),
        ));

        let body = Node::new().with_content(Content::Text(
            TextContent::new(body)
                .with_font_size(Size::lpx(14.0))
                .with_color(mocha::SUBTEXT1),
        ));

        Node::new()
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::lpx(8.0))
            .with_padding(Spacing::all(Size::lpx(12.0)))
            .with_width(Size::fit())
            .with_height(Size::fit())
            .with_style(Style {
                fill_color: Some(mocha::BASE.with_alpha(0.98)),
                stroke: Some(Stroke::new(Size::lpx(1.0), mocha::SURFACE2)),
                corner_shape: Some(CornerShape::Round(Size::lpx(10.0))),
                ..Default::default()
            })
            .with_children(vec![
                // Accent bar
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::lpx(3.0))
                    .with_style(Style {
                        fill_color: Some(accent),
                        ..Default::default()
                    }),
                title,
                body,
            ])
    }

    fn badge(text: &str) -> Node {
        Node::new()
            .with_layout_direction(Layout::Horizontal)
            .with_padding(Spacing::all(Size::lpx(8.0)))
            .with_width(Size::fit())
            .with_height(Size::fit())
            .with_style(Style {
                fill_color: Some(mocha::MANTLE.with_alpha(0.95)),
                stroke: Some(Stroke::new(Size::lpx(1.0), mocha::SURFACE1)),
                corner_shape: Some(CornerShape::Round(Size::lpx(999.0))),
                ..Default::default()
            })
            .with_content(Content::Text(
                TextContent::new(text)
                    .with_font_size(Size::lpx(13.0))
                    .with_color(mocha::OVERLAY1),
            ))
    }
}

impl ExampleApp for PlaceExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            zoom_level: 1.0,
        }
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn zoom_level(&self) -> f32 {
        self.zoom_level
    }

    fn build_ui(&mut self, _ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        // Root fills the window. Stack children are positioned using per-child `Place`.
        let root = Node::new()
            .with_id("root")
            .with_layout_direction(Layout::Stack)
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_style(Style {
                fill_color: Some(mocha::CRUST),
                ..Default::default()
            })
            .with_children(vec![
                // Top-left
                Self::panel(
                    "Top Left",
                    "Placed with Place::Alignment { Left, Top }",
                    mocha::BLUE,
                )
                .with_place(Place::Alignment {
                    h_align: HorizontalAlign::Left,
                    v_align: VerticalAlign::Top,
                })
                .with_margin(Spacing {
                    left: Size::lpx(12.0),
                    top: Size::lpx(12.0),
                    ..Spacing::ZERO
                }),
                // Top-right
                Self::panel(
                    "Top Right",
                    "Placed with Place::Alignment { Right, Top }",
                    mocha::MAUVE,
                )
                .with_place(Place::Alignment {
                    h_align: HorizontalAlign::Right,
                    v_align: VerticalAlign::Top,
                })
                .with_margin(Spacing {
                    right: Size::lpx(12.0),
                    top: Size::lpx(12.0),
                    ..Spacing::ZERO
                }),
                // Bottom-left
                Self::panel(
                    "Bottom Left",
                    "Placed with Place::Alignment { Left, Bottom }",
                    mocha::GREEN,
                )
                .with_place(Place::Alignment {
                    h_align: HorizontalAlign::Left,
                    v_align: VerticalAlign::Bottom,
                })
                .with_margin(Spacing {
                    left: Size::lpx(12.0),
                    bottom: Size::lpx(12.0),
                    ..Spacing::ZERO
                }),
                // Bottom-right
                Self::panel(
                    "Bottom Right",
                    "Placed with Place::Alignment { Right, Bottom }",
                    mocha::PEACH,
                )
                .with_place(Place::Alignment {
                    h_align: HorizontalAlign::Right,
                    v_align: VerticalAlign::Bottom,
                })
                .with_margin(Spacing {
                    right: Size::lpx(12.0),
                    bottom: Size::lpx(12.0),
                    ..Spacing::ZERO
                }),
                // Center (the parent's align is still supported; this child overrides with Place too)
                Self::panel(
                    "Center",
                    "Placed with Place::Alignment { Center, Center }",
                    mocha::YELLOW,
                )
                .with_place(Place::Alignment {
                    h_align: HorizontalAlign::Center,
                    v_align: VerticalAlign::Center,
                }),
                // Absolute badge (uses `Size` so callers can choose logical/physical/relative units)
                Self::badge("Absolute @ (240lpx, 800lpx)").with_place(Place::Absolute {
                    x: Size::lpx(240.0),
                    y: Size::lpx(800.0),
                }),
            ]);

        // The example runner applies `zoom_level` by calling `with_zoom(zoom)`.
        // We keep the default 1.0 here.
        root
    }
}

fn main() {
    run_example::<PlaceExample>();
}
