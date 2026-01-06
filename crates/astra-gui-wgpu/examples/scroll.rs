//! Overflow::Scroll example
//!
//! Demonstrates scrollable containers with mouse wheel support.
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - Mouse wheel to scroll
//! - ESC: quit
//!
//! Note: Debug controls are shared across examples via `shared::debug_controls`.

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node, NodeId,
    Overflow, Size, Spacing, Style, TextContent, UiContext, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::{run_example, ExampleApp};

struct ScrollExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    item_heights: Vec<f32>,
    item_widths: Vec<f32>,
}

impl ExampleApp for ScrollExample {
    fn new() -> Self {
        // STRESS TEST: Generate many more items
        let item_heights: Vec<f32> = (0..200)
            .map(|_| rand::random::<f32>() * 100.0 + 50.0)
            .collect();

        let item_widths: Vec<f32> = (0..200)
            .map(|_| rand::random::<f32>() * 150.0 + 100.0)
            .collect();

        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            item_heights,
            item_widths,
        }
    }

    fn window_title() -> &'static str {
        "Astra GUI - Scroll Example"
    }

    fn window_size() -> (u32, u32) {
        (1200, 1000)
    }

    fn build_ui(&mut self, _ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        // Create a scrollable container with many items - STRESS TEST with nested children
        let mut items = Vec::new();
        for (i, &height) in self.item_heights.iter().enumerate() {
            // Create nested children for each item
            let nested_children = vec![
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::lpx(30.0))
                    .with_content(Content::Text(
                        TextContent::new(format!("Item {}", i + 1))
                            .with_font_size(Size::lpx(20.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Left)
                            .with_v_align(VerticalAlign::Center),
                    )),
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(5.0))
                    .with_children(vec![
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_style(Style {
                                fill_color: Some(mocha::BLUE),
                                corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                ..Default::default()
                            }),
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_style(Style {
                                fill_color: Some(mocha::GREEN),
                                corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                ..Default::default()
                            }),
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_style(Style {
                                fill_color: Some(mocha::RED),
                                corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                ..Default::default()
                            }),
                    ]),
            ];

            items.push(
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::lpx(height))
                    .with_padding(Spacing::all(Size::lpx(8.0)))
                    .with_gap(Size::lpx(5.0))
                    .with_layout_direction(Layout::Vertical)
                    .with_style(Style {
                        fill_color: Some(if i % 2 == 0 {
                            mocha::SURFACE0
                        } else {
                            mocha::SURFACE1
                        }),
                        corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                        ..Default::default()
                    })
                    .with_children(nested_children),
            );
        }

        // Scrollable container - scroll state is now managed automatically
        let scroll_container = Node::new()
            .with_id(NodeId::new("scroll_container"))
            .with_width(Size::lpx(400.0))
            .with_height(Size::Fill)
            .with_padding(Spacing::all(Size::lpx(10.0)))
            .with_gap(Size::lpx(10.0))
            .with_layout_direction(Layout::Vertical)
            .with_overflow(Overflow::Scroll)
            .with_style(Style {
                fill_color: Some(mocha::MANTLE),
                corner_shape: Some(CornerShape::Round(Size::lpx(12.0))),
                ..Default::default()
            })
            .with_children(items);

        // Create horizontal scrollable container - STRESS TEST with nested children
        let mut horizontal_items = Vec::new();
        for (i, &width) in self.item_widths.iter().enumerate() {
            horizontal_items.push(
                Node::new()
                    .with_width(Size::lpx(width))
                    .with_height(Size::Fill)
                    .with_padding(Spacing::all(Size::lpx(8.0)))
                    .with_gap(Size::lpx(5.0))
                    .with_layout_direction(Layout::Vertical)
                    .with_style(Style {
                        fill_color: Some(if i % 2 == 0 {
                            mocha::SURFACE0
                        } else {
                            mocha::SURFACE1
                        }),
                        corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                        ..Default::default()
                    })
                    .with_children(vec![
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::lpx(30.0))
                            .with_content(Content::Text(
                                TextContent::new(format!("H-Item {}", i + 1))
                                    .with_font_size(Size::lpx(18.0))
                                    .with_color(mocha::TEXT)
                                    .with_h_align(HorizontalAlign::Center)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_layout_direction(Layout::Vertical)
                            .with_gap(Size::lpx(3.0))
                            .with_children(vec![
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_height(Size::Fill)
                                    .with_style(Style {
                                        fill_color: Some(mocha::PEACH),
                                        corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                        ..Default::default()
                                    }),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_height(Size::Fill)
                                    .with_style(Style {
                                        fill_color: Some(mocha::YELLOW),
                                        corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                        ..Default::default()
                                    }),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_height(Size::Fill)
                                    .with_style(Style {
                                        fill_color: Some(mocha::TEAL),
                                        corner_shape: Some(CornerShape::Round(Size::lpx(4.0))),
                                        ..Default::default()
                                    }),
                            ]),
                    ]),
            );
        }

        let horizontal_scroll_container = Node::new()
            .with_id(NodeId::new("horizontal_scroll_container"))
            .with_width(Size::lpx(800.0))
            .with_height(Size::Fill)
            .with_padding(Spacing::all(Size::lpx(10.0)))
            .with_gap(Size::lpx(10.0))
            .with_layout_direction(Layout::Horizontal)
            .with_overflow(Overflow::Scroll)
            .with_style(Style {
                fill_color: Some(mocha::MANTLE),
                corner_shape: Some(CornerShape::Round(Size::lpx(12.0))),
                ..Default::default()
            })
            .with_children(horizontal_items);

        // Create 2D scrollable container (scrolls both X and Y) - STRESS TEST with nested children
        let mut grid_items = Vec::new();
        for row in 0..50 {
            let mut row_items = Vec::new();
            for col in 0..50 {
                row_items.push(
                    Node::new()
                        .with_width(Size::lpx(150.0))
                        .with_height(Size::lpx(100.0))
                        .with_padding(Spacing::all(Size::lpx(6.0)))
                        .with_gap(Size::lpx(3.0))
                        .with_layout_direction(Layout::Vertical)
                        .with_style(Style {
                            fill_color: Some(if (row + col) % 2 == 0 {
                                mocha::SURFACE0
                            } else {
                                mocha::SURFACE1
                            }),
                            corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                            ..Default::default()
                        })
                        .with_children(vec![
                            Node::new()
                                .with_width(Size::Fill)
                                .with_height(Size::lpx(25.0))
                                .with_content(Content::Text(
                                    TextContent::new(format!("R{} C{}", row + 1, col + 1))
                                        .with_font_size(Size::lpx(16.0))
                                        .with_color(mocha::TEXT)
                                        .with_h_align(HorizontalAlign::Center)
                                        .with_v_align(VerticalAlign::Center),
                                )),
                            Node::new()
                                .with_width(Size::Fill)
                                .with_height(Size::Fill)
                                .with_layout_direction(Layout::Horizontal)
                                .with_gap(Size::lpx(2.0))
                                .with_children(vec![
                                    Node::new()
                                        .with_width(Size::Fill)
                                        .with_height(Size::Fill)
                                        .with_style(Style {
                                            fill_color: Some(mocha::MAUVE),
                                            corner_shape: Some(CornerShape::Round(Size::lpx(3.0))),
                                            ..Default::default()
                                        }),
                                    Node::new()
                                        .with_width(Size::Fill)
                                        .with_height(Size::Fill)
                                        .with_style(Style {
                                            fill_color: Some(mocha::LAVENDER),
                                            corner_shape: Some(CornerShape::Round(Size::lpx(3.0))),
                                            ..Default::default()
                                        }),
                                ]),
                        ]),
                );
            }

            grid_items.push(
                Node::new()
                    .with_width(Size::FitContent)
                    .with_height(Size::lpx(100.0))
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(10.0))
                    .with_children(row_items),
            );
        }

        let grid_scroll_container = Node::new()
            .with_id(NodeId::new("grid_scroll_container"))
            .with_width(Size::lpx(600.0))
            .with_height(Size::Fill)
            .with_padding(Spacing::all(Size::lpx(10.0)))
            .with_gap(Size::lpx(10.0))
            .with_layout_direction(Layout::Vertical)
            .with_overflow(Overflow::Scroll)
            .with_style(Style {
                fill_color: Some(mocha::MANTLE),
                corner_shape: Some(CornerShape::Round(Size::lpx(12.0))),
                ..Default::default()
            })
            .with_children(grid_items);

        // Root with centered layout
        Node::new()
            .with_zoom(1.5)
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_padding(Spacing::all(Size::lpx(40.0)))
            .with_child(
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::Fill)
                    .with_layout_direction(Layout::Vertical)
                    .with_gap(Size::lpx(20.0))
                    .with_children(vec![
                        // Title
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::fraction(0.1))
                            .with_content(Content::Text(
                                TextContent::new(
                                    "Scroll Example - Use Mouse Wheel (Shift for horizontal)"
                                        .to_string(),
                                )
                                .with_font_size(Size::lpx(32.0))
                                .with_color(mocha::TEXT)
                                .with_h_align(HorizontalAlign::Center)
                                .with_v_align(VerticalAlign::Center),
                            )),
                        // Centered vertical scroll container
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::fraction(0.37))
                            .with_layout_direction(Layout::Horizontal)
                            .with_child(Node::new().with_width(Size::Fill)) // Left spacer
                            .with_child(scroll_container)
                            .with_child(Node::new().with_width(Size::Fill)), // Right spacer
                        // 2D grid scroll container
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::fraction(0.33))
                            .with_layout_direction(Layout::Horizontal)
                            .with_child(Node::new().with_width(Size::Fill)) // Left spacer
                            .with_child(grid_scroll_container)
                            .with_child(Node::new().with_width(Size::Fill)), // Right spacer
                        // Horizontal scroll container
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::fraction(0.2))
                            .with_layout_direction(Layout::Horizontal)
                            .with_child(Node::new().with_width(Size::Fill)) // Left spacer
                            .with_child(horizontal_scroll_container)
                            .with_child(Node::new().with_width(Size::Fill)), // Right spacer
                    ]),
            )
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn main() {
    println!("Use mouse wheel to scroll the containers");
    println!("Hold Shift while scrolling over the grid to scroll horizontally");
    println!();

    run_example::<ScrollExample>();
}
