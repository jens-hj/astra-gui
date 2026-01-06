//! Interactive components example
//!
//! Demonstrates button, toggle, slider, and text input components using the
//! new builder pattern API with automatic state management via UiContext.
//!
//! Controls:
//! - Click +/- buttons to change counter
//! - Click toggle to enable/disable buttons
//! - Drag sliders to adjust values
//! - Click text input to type
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

mod shared;

use astra_gui::{
    catppuccin::mocha, Component, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape,
    Size, Spacing, StyledRect, TextContent, UiContext, VerticalAlign,
};
use astra_gui_interactive::{
    Button, ButtonStyle, CursorShape, CursorStyle, Slider, SliderStyle, TextInput, TextInputStyle,
    Toggle, ToggleStyle,
};
use astra_gui_text::Engine as TextEngine;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp};
use std::cell::RefCell;
use std::rc::Rc;

/// Shared application state that can be modified from callbacks
struct AppState {
    counter: i32,
    nodes_disabled: bool,
    slider_value: f32,
    continuous_slider_value: f32,
    text_input_value: String,
}

struct Interactive {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    state: Rc<RefCell<AppState>>,
}

impl ExampleApp for Interactive {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            state: Rc::new(RefCell::new(AppState {
                counter: 0,
                nodes_disabled: false,
                slider_value: 7.0,
                continuous_slider_value: 50.0,
                text_input_value: String::new(),
            })),
        }
    }

    fn window_title() -> &'static str {
        "Interactive Components - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (1100, 800)
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn build_ui(&mut self, ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        // Clone Rc for use in callbacks
        let state = self.state.clone();

        // Read current values for display
        let (counter, nodes_disabled, slider_value, continuous_slider_value, text_input_value) = {
            let s = state.borrow();
            (
                s.counter,
                s.nodes_disabled,
                s.slider_value,
                s.continuous_slider_value,
                s.text_input_value.clone(),
            )
        };

        // Clone state for each callback
        let state_dec = state.clone();
        let state_inc = state.clone();
        let state_toggle = state.clone();
        let state_stepped = state.clone();
        let state_continuous = state.clone();
        let state_text = state.clone();

        Node::new()
            .with_zoom(2.0)
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::lpx(24.0))
            .with_children(vec![
                // Spacer
                Node::new().with_height(Size::Fill),
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("Interactive Button Example".to_string())
                            .with_font_size(Size::lpx(32.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Counter display
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new(format!("Count: {}", counter))
                            .with_font_size(Size::lpx(48.0))
                            .with_color(mocha::LAVENDER)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Centered button container
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(16.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Decrement Button
                        Button::new("-")
                            .disabled(nodes_disabled)
                            .with_style(ButtonStyle::default())
                            .on_click(move || {
                                let mut s = state_dec.borrow_mut();
                                s.counter -= 1;
                                println!("Decrement clicked! Counter: {}", s.counter);
                            })
                            .node(ctx),
                        // Increment Button
                        Button::new("+")
                            .disabled(nodes_disabled)
                            .with_style(ButtonStyle::default())
                            .on_click(move || {
                                let mut s = state_inc.borrow_mut();
                                s.counter += 1;
                                println!("Increment clicked! Counter: {}", s.counter);
                            })
                            .node(ctx),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Toggle container
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(16.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Label
                        Node::new()
                            .with_width(Size::FitContent)
                            .with_height(Size::FitContent)
                            .with_content(Content::Text(
                                TextContent::new("Enable buttons:".to_string())
                                    .with_font_size(Size::lpx(20.0))
                                    .with_color(mocha::TEXT)
                                    .with_h_align(HorizontalAlign::Center)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                        // Toggle Switch
                        Toggle::new(!nodes_disabled)
                            .with_style(ToggleStyle::default())
                            .on_toggle(move |new_state| {
                                let mut s = state_toggle.borrow_mut();
                                s.nodes_disabled = !new_state;
                                println!(
                                    "Toggle clicked! Buttons are now {}",
                                    if s.nodes_disabled {
                                        "disabled"
                                    } else {
                                        "enabled"
                                    }
                                );
                            })
                            .node(ctx),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Instructions
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new(
                            "Use the toggle switch to enable/disable the counter buttons!"
                                .to_string(),
                        )
                        .with_font_size(Size::lpx(16.0))
                        .with_color(mocha::SUBTEXT0)
                        .with_h_align(HorizontalAlign::Center)
                        .with_v_align(VerticalAlign::Center),
                    )),
                // Text input section
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(16.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Text Input
                        {
                            // We need to pass a mutable reference to the string
                            // Since TextInput takes &mut String, we need to handle this carefully
                            let mut s = state_text.borrow_mut();
                            TextInput::new(&mut s.text_input_value)
                                .placeholder("Type something...")
                                .disabled(nodes_disabled)
                                .with_style(TextInputStyle {
                                    cursor_style: CursorStyle {
                                        shape: CursorShape::Underline,
                                        thickness: 3.0,
                                        ..CursorStyle::default()
                                    },
                                    ..TextInputStyle::default()
                                })
                                .on_change(|new_val| {
                                    println!("Text input value: {}", new_val);
                                })
                                .build(ctx)
                        },
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Stepped slider section
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(16.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Label
                        Node::new()
                            .with_width(Size::lpx(150.0))
                            .with_height(Size::FitContent)
                            .with_content(Content::Text(
                                TextContent::new("Stepped (7):".to_string())
                                    .with_font_size(Size::lpx(20.0))
                                    .with_color(mocha::TEXT)
                                    .with_h_align(HorizontalAlign::Right)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                        // Slider
                        Slider::new(slider_value, 0.0..=30.0)
                            .step(7.0)
                            .disabled(nodes_disabled)
                            .with_style(SliderStyle::default())
                            .on_change(move |new_val| {
                                state_stepped.borrow_mut().slider_value = new_val;
                                println!("Stepped slider value: {:.1}", new_val);
                            })
                            .node(ctx),
                        // Value display
                        Node::new()
                            .with_width(Size::lpx(55.0))
                            .with_height(Size::FitContent)
                            .with_content(Content::Text(
                                TextContent::new(format!("{:.0}", slider_value))
                                    .with_font_size(Size::lpx(20.0))
                                    .with_color(mocha::LAVENDER)
                                    .with_h_align(HorizontalAlign::Right)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Continuous slider section
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(16.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Label
                        Node::new()
                            .with_width(Size::lpx(150.0))
                            .with_height(Size::FitContent)
                            .with_content(Content::Text(
                                TextContent::new("Continuous:".to_string())
                                    .with_font_size(Size::lpx(20.0))
                                    .with_color(mocha::TEXT)
                                    .with_h_align(HorizontalAlign::Right)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                        // Slider
                        Slider::new(continuous_slider_value, 0.0..=100.0)
                            .disabled(nodes_disabled)
                            .with_style(SliderStyle::default())
                            .on_change(move |new_val| {
                                state_continuous.borrow_mut().continuous_slider_value = new_val;
                                println!("Continuous slider value: {:.2}", new_val);
                            })
                            .node(ctx),
                        // Value display
                        Node::new()
                            .with_width(Size::lpx(55.0))
                            .with_height(Size::FitContent)
                            .with_content(Content::Text(
                                TextContent::new(format!("{:.2}", continuous_slider_value))
                                    .with_font_size(Size::lpx(20.0))
                                    .with_color(mocha::LAVENDER)
                                    .with_h_align(HorizontalAlign::Right)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Spacer
                Node::new().with_height(Size::Fill),
                // Help bar
                Node::new()
                    .with_width(Size::Fill)
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
                    )),
            ])
    }
}

fn main() {
    run_example::<Interactive>();
}
