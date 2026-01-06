//! Slider with value widget example
//!
//! Demonstrates the combined slider + drag value component using the new
//! builder pattern API with automatic state management via UiContext.
//!
//! Controls:
//! - Drag slider or value field to adjust
//! - Hold Shift while dragging value for precise control (0.1x speed)
//! - Hold Ctrl while dragging value for fast control (10x speed)
//! - Click on value to enter text input mode
//! - Press Enter to confirm or Escape to cancel text input
//! - ESC: quit

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape, Size, Spacing,
    StyledRect, TextContent, UiContext, VerticalAlign,
};
use astra_gui_interactive::{DragValueStyle, SliderStyle, SliderWithValue};
use astra_gui_text::Engine as TextEngine;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp};

struct SliderWithValueExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,

    // Application state - values that the sliders control
    basic_value: f32,
    clamped_value: f32,
    stepped_value: f32,
    disabled_value: f32,
}

impl ExampleApp for SliderWithValueExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            basic_value: 42.5,
            clamped_value: 50.0,
            stepped_value: 10.0,
            disabled_value: 99.9,
        }
    }

    fn window_title() -> &'static str {
        "Slider with Value Widget - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (1200, 1000)
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn build_ui(&mut self, ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        // Build the basic slider
        let basic_slider = SliderWithValue::new(&mut self.basic_value, 0.0..=100.0)
            .speed(0.1)
            .with_slider_style(SliderStyle::default())
            .with_value_style(DragValueStyle::default())
            .on_change(|new_val| {
                println!("Basic value: {:.2}", new_val);
            })
            .build(ctx);

        // Build the clamped slider
        let clamped_slider = SliderWithValue::new(&mut self.clamped_value, 0.0..=100.0)
            .speed(0.1)
            .with_slider_style(SliderStyle::default())
            .with_value_style(DragValueStyle::default())
            .on_change(|new_val| {
                println!("Clamped value: {:.2}", new_val);
            })
            .build(ctx);

        // Build the stepped slider
        let stepped_slider = SliderWithValue::new(&mut self.stepped_value, 0.0..=100.0)
            .step(5.0)
            .speed(0.1)
            .with_slider_style(SliderStyle::default())
            .with_value_style(DragValueStyle::default())
            .on_change(|new_val| {
                println!("Stepped value: {:.1}", new_val);
            })
            .build(ctx);

        // Build the disabled slider
        let disabled_slider = SliderWithValue::new(&mut self.disabled_value, 0.0..=100.0)
            .disabled(true)
            .with_slider_style(SliderStyle::default())
            .with_value_style(DragValueStyle::default())
            .build(ctx);

        Node::new()
            .with_zoom(1.5)
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::ppx(20.0))
            .with_children(vec![
                // Spacer
                Node::new().with_height(Size::Fill),
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("Slider with Value Widget Example".to_string())
                            .with_font_size(Size::lpx(32.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Instructions
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new(
                            "Drag slider or value • Click value to type • Shift=precise, Ctrl=fast"
                                .to_string(),
                        )
                        .with_font_size(Size::lpx(16.0))
                        .with_color(mocha::SUBTEXT0)
                        .with_h_align(HorizontalAlign::Center)
                        .with_v_align(VerticalAlign::Center),
                    )),
                Node::new().with_height(Size::lpx(20.0)),
                // Basic slider row
                create_slider_row("Basic (0-100):", basic_slider),
                // Clamped slider row
                create_slider_row("Clamped (0-100):", clamped_slider),
                // Stepped slider row
                create_slider_row("Stepped (5.0 steps):", stepped_slider),
                // Disabled slider row
                create_slider_row("Disabled:", disabled_slider),
                // Spacer
                Node::new().with_height(Size::Fill),
                // Help bar
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::lpx(30.0))
                    .with_padding(Spacing::horizontal(Size::ppx(10.0)))
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
                    )),
            ])
    }
}

/// Helper to create a labeled slider row
fn create_slider_row(label: &str, slider_widget: Node) -> Node {
    Node::new()
        .with_width(Size::Fill)
        .with_layout_direction(Layout::Horizontal)
        .with_gap(Size::ppx(16.0))
        .with_children(vec![
            // Spacer
            Node::new().with_width(Size::Fill),
            // Label
            Node::new()
                .with_width(Size::lpx(200.0))
                .with_height(Size::Fill)
                .with_content(Content::Text(
                    TextContent::new(label.to_string())
                        .with_font_size(Size::lpx(20.0))
                        .with_color(mocha::TEXT)
                        .with_h_align(HorizontalAlign::Right)
                        .with_v_align(VerticalAlign::Center),
                )),
            // Slider with value widget
            slider_widget,
            // Spacer
            Node::new().with_width(Size::Fill),
        ])
}

fn main() {
    run_example::<SliderWithValueExample>();
}
