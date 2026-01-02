//! Interactive components example
//!
//! Demonstrates button, toggle, and slider components with hover and click states.
//!
//! Controls:
//! - Click +/- buttons to change counter
//! - Click toggle to enable/disable buttons
//! - Drag slider to adjust value
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit
//!
//! Note: Debug controls are shared across examples via `shared::debug_controls`.

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape, Size, Spacing,
    StyledRect, TextContent, VerticalAlign,
};
use astra_gui_interactive::{
    button, button_clicked, slider, slider_drag, text_input, text_input_update, toggle,
    toggle_clicked, ButtonStyle, CursorShape, CursorStyle, SliderStyle, TextInputStyle,
    ToggleStyle,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::TargetedEvent;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp, InteractiveState};

struct Interactive {
    interactive: InteractiveState,
    text_engine: TextEngine,
    debug_options: DebugOptions,

    // Application state
    counter: i32,
    nodes_disabled: bool,
    slider_value: f32,
    continuous_slider_value: f32,
    text_input_value: String,
    text_input_cursor: usize,
    text_input_selection: Option<(usize, usize)>,
}

impl ExampleApp for Interactive {
    fn new() -> Self {
        Self {
            interactive: InteractiveState::new(),
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            counter: 0,
            nodes_disabled: false,
            slider_value: 7.0,
            continuous_slider_value: 50.0,
            text_input_value: String::new(),
            text_input_cursor: 0,
            text_input_selection: None,
        }
    }

    fn window_title() -> &'static str {
        "Interactive Components - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (1100, 800)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
        Node::new()
            // .with_zoom(2.0)
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
                        TextContent::new(format!("Count: {}", self.counter))
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
                    .with_child(
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    )
                    .with_children(vec![
                        // Decrement Button
                        button(
                            "decrement_btn",
                            "-",
                            self.nodes_disabled,
                            &ButtonStyle::default(),
                        ),
                        // Increment Button
                        button(
                            "increment_btn",
                            "+",
                            self.nodes_disabled,
                            &ButtonStyle::default(),
                        ),
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
                        toggle(
                            "enable_toggle",
                            !self.nodes_disabled, // Toggle is ON when buttons are enabled
                            false,                // Toggle itself is never disabled
                            &ToggleStyle::default(),
                        ),
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
                        text_input(
                            "text_input",
                            &self.text_input_value,
                            "Type something...",
                            self.interactive
                                .event_dispatcher
                                .focused_node()
                                .map(|id| id.as_str() == "text_input")
                                .unwrap_or(false),
                            self.nodes_disabled,
                            &TextInputStyle {
                                cursor_style: CursorStyle {
                                    shape: CursorShape::Underline,
                                    thickness: 3.0,
                                    ..CursorStyle::default()
                                },
                                ..TextInputStyle::default()
                            },
                            self.text_input_cursor,
                            self.text_input_selection,
                            &mut self.text_engine,
                            &mut self.interactive.event_dispatcher,
                        ),
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    ]),
                // Stepped slider section
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(16.0))
                    .with_child(
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    )
                    .with_children(vec![
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
                        slider(
                            "stepped_slider",
                            self.slider_value,
                            0.0..=30.0,
                            self.nodes_disabled,
                            &SliderStyle::default(),
                        ),
                        // Value display
                        Node::new()
                            .with_width(Size::lpx(55.0))
                            .with_height(Size::FitContent)
                            .with_content(Content::Text(
                                TextContent::new(format!("{:.0}", self.slider_value))
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
                    .with_child(
                        // Spacer
                        Node::new().with_width(Size::Fill),
                    )
                    .with_children(vec![
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
                        slider(
                            "continuous_slider",
                            self.continuous_slider_value,
                            0.0..=100.0,
                            self.nodes_disabled,
                            &SliderStyle::default(),
                        ),
                        // Value display
                        Node::new()
                            .with_width(Size::lpx(55.0))
                            .with_height(Size::FitContent)
                            .with_content(Content::Text(
                                TextContent::new(format!("{:.2}", self.continuous_slider_value))
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

    fn text_measurer(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn interactive_state(&mut self) -> Option<&mut InteractiveState> {
        Some(&mut self.interactive)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn handle_events(&mut self, events: &[TargetedEvent]) -> bool {
        let mut changed = false;

        // Update text input (handles focus, unfocus, and keyboard input automatically)
        if text_input_update(
            "text_input",
            &mut self.text_input_value,
            &mut self.text_input_cursor,
            &mut self.text_input_selection,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
        ) {
            println!("Text input value: {}", self.text_input_value);
            changed = true;
        }

        // Handle button clicks
        if button_clicked("increment_btn", events) {
            self.counter += 1;
            println!("Increment clicked! Counter: {}", self.counter);
            changed = true;
        }

        if button_clicked("decrement_btn", events) {
            self.counter -= 1;
            println!("Decrement clicked! Counter: {}", self.counter);
            changed = true;
        }

        if toggle_clicked("enable_toggle", events) {
            self.nodes_disabled = !self.nodes_disabled;
            println!(
                "Toggle clicked! Buttons are now {}",
                if self.nodes_disabled {
                    "disabled"
                } else {
                    "enabled"
                }
            );
            changed = true;
        }

        // Handle stepped slider drag
        if slider_drag(
            "stepped_slider",
            &mut self.slider_value,
            &(0.0..=30.0),
            events,
            &SliderStyle::default(),
            Some(7.0), // Step by 7.0
        ) {
            println!("Stepped slider value: {:.1}", self.slider_value);
            changed = true;
        }

        // Handle continuous slider drag
        if slider_drag(
            "continuous_slider",
            &mut self.continuous_slider_value,
            &(0.0..=100.0),
            events,
            &SliderStyle::default(),
            None, // No stepping - continuous
        ) {
            println!(
                "Continuous slider value: {:.1}",
                self.continuous_slider_value
            );
            changed = true;
        }

        changed
    }
}

fn main() {
    run_example::<Interactive>();
}
