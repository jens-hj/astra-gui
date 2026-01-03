//! Demonstrates node overflow behaviors: Hidden vs Visible vs Scroll (placeholder).
//!
//! This example is intentionally minimal: each column contains a single "viewport" container
//! and a single child text node that is positioned to extend beyond the viewport bounds.
//!
//! Notes:
//! - In astra-gui core, `Overflow::Hidden` is the default and enforces clip rect intersection.
//! - `Overflow::Scroll` is currently treated like `Hidden` (clipping only; no scroll offsets yet).
//! - In the WGPU backend, text uses per-shape scissor and respects `ClippedShape::clip_rect`.
//! Overflow example
//!
//! Demonstrates overflow clipping with nested elements.
//!
//! Controls:
//! - Use sliders to adjust overflow
//! - Click buttons to verify hit testing works
//! - Click buttons to verify hit testing works
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit
//!
//! Note: Debug controls are shared across examples via `shared::debug_controls`.

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Overflow, Shape, Size, Spacing, Stroke, StyledRect, TextContent, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::{debug_controls::DEBUG_HELP_TEXT_ONELINE, run_example, ExampleApp};

struct OverflowExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
}

impl ExampleApp for OverflowExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Overflow Demo"
    }

    fn window_size() -> (u32, u32) {
        (1180, 720)
    }

    fn build_ui(&mut self, width: f32, height: f32) -> Node {
        create_demo_ui(width, height, &self.debug_options).with_zoom(2.0)
    }

    fn text_measurer(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn panel(fill: Color) -> Shape {
    Shape::Rect(
        StyledRect::new(Default::default(), fill)
            .with_corner_shape(CornerShape::Round(Size::lpx(14.0)))
            .with_stroke(Stroke::new(Size::lpx(2.0), mocha::SURFACE1)),
    )
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
            .with_color(color)
            .with_h_align(h)
            .with_v_align(v),
    ))
}

fn demo_box(title: &str, overflow_mode: Overflow, color: Color) -> Node {
    // A tight viewport; inside we place a single child whose text is positioned so it
    // extends beyond the viewport bounds.
    //
    // - Hidden/Scroll: the overflowing portion must be clipped.
    // - Visible: the overflowing portion can render outside the viewport bounds.
    //
    // NOTE: `Overflow::Scroll` is currently treated like `Hidden` in core (clip only).
    let long_text =
        "OVERFLOW DEMO →→→ this text extends beyond the viewport bounds →→→ →→→ →→→ →→→";

    Node::new()
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_padding(Spacing::all(Size::lpx(16.0)))
        .with_gap(Size::lpx(14.0))
        .with_layout_direction(Layout::Vertical)
        .with_overflow(overflow_mode)
        .with_shape(panel(color))
        .with_children(vec![
            // Title row
            Node::new()
                .with_height(Size::lpx(40.0))
                .with_layout_direction(Layout::Horizontal)
                .with_children(vec![label(
                    title,
                    22.0,
                    mocha::TEXT,
                    HorizontalAlign::Left,
                    VerticalAlign::Center,
                )]),
            // Viewport content: one oversized child.
            // NOTE: We must propagate Overflow::Visible down through wrapper nodes,
            // otherwise the default Overflow::Hidden on intermediate nodes will clip.
            Node::new()
                .with_height(Size::Fill)
                .with_width(Size::Fill)
                .with_padding(Spacing::all(Size::lpx(14.0)))
                .with_shape(panel(mocha::SURFACE0))
                .with_layout_direction(Layout::Horizontal)
                .with_overflow(overflow_mode) // Propagate overflow mode
                .with_children(vec![Node::new()
                    .with_overflow(overflow_mode)
                    .with_children(vec![label(
                        long_text,
                        26.0,
                        mocha::TEXT,
                        HorizontalAlign::Left,
                        VerticalAlign::Top,
                    )])]),
        ])
}

fn create_demo_ui(_width: f32, _height: f32, _debug_options: &DebugOptions) -> Node {
    let root = Node::new()
        // Root clips by default (Overflow::Hidden default). Keep it Visible so the
        // "Visible" column can actually show overflow past its own viewport.
        .with_overflow(Overflow::Visible)
        .with_padding(Spacing::all(Size::lpx(24.0)))
        .with_gap(Size::lpx(18.0))
        .with_layout_direction(Layout::Vertical)
        .with_width(Size::Fill)
        .with_height(Size::Fill)
        .with_children(vec![
            // Header
            Node::new()
                .with_height(Size::lpx(120.0))
                .with_width(Size::Fill)
                .with_padding(Spacing::all(Size::lpx(18.0)))
                .with_shape(panel(mocha::SURFACE0))
                .with_children(vec![
                    label(
                        "astra-gui: Overflow demo",
                        34.0,
                        mocha::TEXT,
                        HorizontalAlign::Left,
                        VerticalAlign::Top,
                    )
                    .with_height(Size::Fill),
                    label(
                        "Each column has one viewport + one text child that starts outside the viewport. Hidden/Scroll clip; Visible does not.",
                        18.0,
                        mocha::SUBTEXT0,
                        HorizontalAlign::Left,
                        VerticalAlign::Bottom,
                    )
                    .with_height(Size::Fill),
                ]),
            // Columns
            Node::new()
                .with_width(Size::Fill)
                .with_height(Size::Fill)
                .with_gap(Size::lpx(18.0))
                .with_layout_direction(Layout::Vertical)
                .with_overflow(Overflow::Visible) // Allow children to overflow
                .with_children(vec![
                    Node::new()
                        .with_width(Size::lpx(400.0))
                        .with_height(Size::Fill)
                        .with_children(vec![demo_box(
                            "Overflow: Hidden (default)",
                            Overflow::Hidden,
                            mocha::CRUST,
                        )]),
                    Node::new()
                        .with_width(Size::lpx(400.0))
                        .with_height(Size::Fill)
                        .with_overflow(Overflow::Visible) // Allow child to overflow
                        .with_children(vec![demo_box(
                            "Overflow: Visible",
                            Overflow::Visible,
                            mocha::MANTLE,
                        )]),
                    Node::new()
                        .with_width(Size::lpx(400.0))
                        .with_height(Size::Fill)
                        .with_children(vec![demo_box(
                            "Overflow: Scroll (placeholder)",
                            Overflow::Scroll,
                            mocha::SURFACE0,
                        )]),
                ]),
            // Help bar
            Node::new()
                .with_height(Size::lpx(30.0))
                .with_width(Size::Fill)
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

fn main() {
    run_example::<OverflowExample>();
}
