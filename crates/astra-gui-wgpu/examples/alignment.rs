//! Alignment example
//!
//! Demonstrates h_align and v_align working together for different layout directions.
//!
//! Controls:
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

mod shared;

use astra_gui::{
    catppuccin::mocha, Color, Content, CornerShape, DebugOptions, HorizontalAlign, Layout, Node,
    Size, Spacing, Stroke, Style, TextContent, VerticalAlign,
};
use astra_gui_text::Engine as TextEngine;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp};
use winit::window::Window;

const MM_PER_INCH: f32 = 25.4;
const REFERENCE_PPI: f32 = 100.0;

struct Alignment {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    zoom_level: f32,
}

impl ExampleApp for Alignment {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            zoom_level: 1.0,
        }
    }

    fn window_title() -> &'static str {
        "Alignment Example"
    }

    fn window_size() -> (u32, u32) {
        (1200, 1200)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
        // Helper function to create a colored box
        let create_box = |color: Color, text: &str| {
            Node::new()
                .with_width(Size::fraction(1.0 / 4.0))
                .with_height(Size::fraction(1.0 / 4.0))
                .with_style(Style {
                    fill_color: Some(mocha::CRUST),
                    stroke: Some(Stroke::new(Size::lpx(2.0), color)),
                    corner_shape: Some(CornerShape::Round(Size::lpx(8.0))),
                    ..Default::default()
                })
                .with_h_align(HorizontalAlign::Center)
                .with_v_align(VerticalAlign::Center)
                .with_child(
                    Node::new().with_content(Content::Text(
                        TextContent::new(text)
                            .with_font_size(Size::lpx(16.0))
                            .with_color(mocha::TEXT),
                    )),
                )
        };

        // Helper to create a container with alignment settings
        let create_container = |h_align: HorizontalAlign, v_align: VerticalAlign| {
            let h_label = match h_align {
                HorizontalAlign::Left => "Left",
                HorizontalAlign::Center => "Center",
                HorizontalAlign::Right => "Right",
            };
            let v_label = match v_align {
                VerticalAlign::Top => "Top",
                VerticalAlign::Center => "Center",
                VerticalAlign::Bottom => "Bottom",
            };

            Node::new()
                .with_layout_direction(Layout::Vertical)
                .with_children(vec![
                    // Label
                    Node::new()
                        .with_width(Size::Fill)
                        .with_margin(Spacing::bottom(Size::lpx(20.0)))
                        .with_content(Content::Text(
                            TextContent::new(format!("{} {}", h_label, v_label))
                                .with_font_size(Size::lpx(24.0))
                                .with_color(mocha::SUBTEXT0)
                                .with_h_align(HorizontalAlign::Center)
                                .with_v_align(VerticalAlign::Center),
                        )),
                    // Container with alignment
                    Node::new()
                        .with_width(Size::lpx(300.0))
                        .with_height(Size::lpx(300.0))
                        .with_style(Style {
                            fill_color: Some(mocha::CRUST),
                            stroke: Some(Stroke::new(Size::lpx(2.0), mocha::SURFACE0)),
                            corner_shape: Some(CornerShape::Round(Size::lpx(18.0))),
                            ..Default::default()
                        })
                        .with_padding(Spacing::all(Size::lpx(12.0)))
                        .with_layout_direction(Layout::Vertical)
                        .with_h_align(h_align)
                        .with_v_align(v_align)
                        .with_gap(Size::lpx(10.0))
                        .with_children(vec![
                            create_box(mocha::RED, "Red"),
                            create_box(mocha::GREEN, "Green"),
                        ]),
                ])
        };

        Node::new()
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_style(Style {
                fill_color: Some(mocha::BASE),
                ..Default::default()
            })
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::lpx(24.0))
            .with_children(vec![
                // Spacer
                Node::new().with_height(Size::Fill),
                // Title
                Node::new()
                    .with_margin(Spacing::bottom(Size::lpx(50.0)))
                    .with_gap(Size::lpx(10.0))
                    .with_width(Size::Fill)
                    .with_children(vec![
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::lpx(60.0))
                            .with_content(Content::Text(
                                TextContent::new("Alignment Examples")
                                    .with_font_size(Size::lpx(46.0))
                                    .with_color(mocha::TEXT)
                                    .with_h_align(HorizontalAlign::Center)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                        // Instructions
                        Node::new()
                            .with_width(Size::Fill)
                            .with_content(Content::Text(
                            TextContent::new(
                                "h_align and v_align control child positioning within containers"
                                    .to_string(),
                            )
                            .with_font_size(Size::lpx(24.0))
                            .with_color(mocha::SUBTEXT0)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                        )),
                    ]),
                // Main content area
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::ppx(40.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Content container
                        Node::new()
                            .with_layout_direction(Layout::Vertical)
                            .with_gap(Size::ppx(36.0))
                            .with_children(vec![
                                // Horizontal Layout Examples
                                Node::new()
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::ppx(36.0))
                                    .with_children(vec![
                                        create_container(HorizontalAlign::Left, VerticalAlign::Top),
                                        create_container(
                                            HorizontalAlign::Center,
                                            VerticalAlign::Top,
                                        ),
                                        create_container(
                                            HorizontalAlign::Right,
                                            VerticalAlign::Top,
                                        ),
                                    ]),
                                // Vertical Layout Examples
                                Node::new()
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::ppx(36.0))
                                    .with_children(vec![
                                        create_container(
                                            HorizontalAlign::Left,
                                            VerticalAlign::Center,
                                        ),
                                        create_container(
                                            HorizontalAlign::Center,
                                            VerticalAlign::Center,
                                        ),
                                        create_container(
                                            HorizontalAlign::Right,
                                            VerticalAlign::Center,
                                        ),
                                    ]),
                                // Stack Layout Examples
                                Node::new()
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::ppx(36.0))
                                    .with_children(vec![
                                        create_container(
                                            HorizontalAlign::Left,
                                            VerticalAlign::Bottom,
                                        ),
                                        create_container(
                                            HorizontalAlign::Center,
                                            VerticalAlign::Bottom,
                                        ),
                                        create_container(
                                            HorizontalAlign::Right,
                                            VerticalAlign::Bottom,
                                        ),
                                    ]),
                            ]),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Spacer
                Node::new().with_height(Size::Fill),
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
                        TextContent::new(DEBUG_HELP_TEXT_ONELINE)
                            .with_font_size(Size::lpx(16.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Left)
                            .with_v_align(VerticalAlign::Center),
                    )),
            ])
    }

    fn text_measurer(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn zoom_level(&self) -> f32 {
        self.zoom_level
    }

    fn on_window_created(&mut self, window: &Window) {
        // Calculate zoom based on PPI
        // We want MacBook Pro 16" (254 PPI) to have 1.75 zoom
        // Reference PPI = 254 / 1.75 = ~145.14
        if let Ok(displays) = display_info::DisplayInfo::all() {
            if !displays.is_empty() {
                let monitor = window.current_monitor();

                // Try to find a matching display and monitor pair
                let display_match = if let Some(monitor) = monitor {
                    let monitor_pos = monitor.position();
                    displays
                        .iter()
                        .find(|d| d.x == monitor_pos.x && d.y == monitor_pos.y)
                        .map(|d| (d, monitor.size().width as f32, monitor.size().height))
                } else {
                    None
                };

                // Fallback to first display if no match found or no monitor detected
                let (display, width_px, height_px) = display_match.unwrap_or_else(|| {
                    let d = &displays[0];
                    (
                        d,
                        d.width as f32 * d.scale_factor,
                        (d.height as f32 * d.scale_factor) as u32,
                    )
                });

                let width_mm = display.width_mm as f32;

                if width_mm > 0.0 {
                    let width_inches = width_mm / MM_PER_INCH;
                    let ppi = width_px / width_inches;

                    self.zoom_level = ppi / REFERENCE_PPI;

                    println!(
                        "Detected Display: {}x{}px ({}mm wide). PPI: {:.2}. Setting zoom to {:.2}",
                        width_px, height_px, width_mm, ppi, self.zoom_level
                    );
                }
            }
        }
    }
}

fn main() {
    run_example::<Alignment>();
}
