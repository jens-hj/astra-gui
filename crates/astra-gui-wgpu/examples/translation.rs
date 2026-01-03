//! Translation example
//!
//! Demonstrates translation (offset) with nested elements.
//!
//! Controls:
//! - Use sliders to adjust translations
//! - Click buttons to verify hit testing works
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit
//!
//! Note: Debug controls are shared across examples via `shared::debug_controls`.

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape, Size, Spacing,
    Stroke, StyledRect, TextContent, Translation, VerticalAlign,
};
use astra_gui_interactive::{
    button, button_clicked, slider, slider_drag, toggle, toggle_clicked, ButtonStyle, SliderStyle,
    ToggleStyle,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::TargetedEvent;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp, InteractiveState};

struct TranslationExample {
    interactive: InteractiveState,
    text_engine: TextEngine,
    debug_options: DebugOptions,

    // Application state
    outer_translation_x: f32,
    outer_translation_y: f32,
    inner_translation_x: f32,
    inner_translation_y: f32,
    counter: i32,
    toggle_state: bool,
}

impl ExampleApp for TranslationExample {
    fn new() -> Self {
        Self {
            interactive: InteractiveState::new(),
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            outer_translation_x: 50.0,
            outer_translation_y: 30.0,
            inner_translation_x: 20.0,
            inner_translation_y: 20.0,
            counter: 0,
            toggle_state: false,
        }
    }

    fn window_title() -> &'static str {
        "Translation Example - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (1200, 800)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
        Node::new()
            .with_zoom(1.5)
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
                    .with_height(Size::lpx(60.0))
                    .with_padding(Spacing::vertical(Size::lpx(10.0)))
                    .with_content(Content::Text(
                        TextContent::new("Transform Translation Example".to_string())
                            .with_font_size(Size::lpx(32.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Instructions
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("Adjust sliders to translate containers. Nested translations should accumulate.".to_string())
                            .with_font_size(Size::lpx(16.0))
                            .with_color(mocha::SUBTEXT0)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Main content area with horizontal layout
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(40.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Left side - Controls
                        Node::new()
                            .with_width(Size::lpx(300.0))
                            .with_height(Size::Fill)
                            .with_padding(Spacing::all(Size::lpx(20.0)))
                            .with_layout_direction(Layout::Vertical)
                            .with_gap(Size::lpx(20.0))
                            .with_children(vec![
                                // Outer translation X slider
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new("Outer Container X".to_string())
                                            .with_font_size(Size::lpx(20.0))
                                            .with_color(mocha::LAVENDER)
                                            .with_h_align(HorizontalAlign::Center)
                                            .with_v_align(VerticalAlign::Center),
                                    )),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::lpx(12.0))
                                    .with_children(vec![
                                        slider(
                                            "outer_x_slider",
                                            self.outer_translation_x,
                                            -200.0..=200.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", self.outer_translation_x))
                                                    .with_font_size(Size::lpx(18.0))
                                                    .with_color(mocha::LAVENDER)
                                                    .with_h_align(HorizontalAlign::Right)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                                // Outer translation Y slider
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new("Outer Container Y".to_string())
                                            .with_font_size(Size::lpx(20.0))
                                            .with_color(mocha::LAVENDER)
                                            .with_h_align(HorizontalAlign::Center)
                                            .with_v_align(VerticalAlign::Center),
                                    )),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::lpx(12.0))
                                    .with_children(vec![
                                        slider(
                                            "outer_y_slider",
                                            self.outer_translation_y,
                                            -200.0..=200.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", self.outer_translation_y))
                                                    .with_font_size(Size::lpx(18.0))
                                                    .with_color(mocha::LAVENDER)
                                                    .with_h_align(HorizontalAlign::Right)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                                // Inner translation X slider
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new("Inner Container X".to_string())
                                            .with_font_size(Size::lpx(20.0))
                                            .with_color(mocha::GREEN)
                                            .with_h_align(HorizontalAlign::Center)
                                            .with_v_align(VerticalAlign::Center),
                                    )),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::lpx(12.0))
                                    .with_children(vec![
                                        slider(
                                            "inner_x_slider",
                                            self.inner_translation_x,
                                            -100.0..=100.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", self.inner_translation_x))
                                                    .with_font_size(Size::lpx(18.0))
                                                    .with_color(mocha::LAVENDER)
                                                    .with_h_align(HorizontalAlign::Right)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                                // Inner translation Y slider
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new("Inner Container Y".to_string())
                                            .with_font_size(Size::lpx(20.0))
                                            .with_color(mocha::GREEN)
                                            .with_h_align(HorizontalAlign::Center)
                                            .with_v_align(VerticalAlign::Center),
                                    )),
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_layout_direction(Layout::Horizontal)
                                    .with_gap(Size::lpx(12.0))
                                    .with_children(vec![
                                        slider(
                                            "inner_y_slider",
                                            self.inner_translation_y,
                                            -100.0..=100.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", self.inner_translation_y))
                                                    .with_font_size(Size::lpx(18.0))
                                                    .with_color(mocha::LAVENDER)
                                                    .with_h_align(HorizontalAlign::Right)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                                // Counter display
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_height(Size::Fill)
                                    .with_layout_direction(Layout::Vertical)
                                    .with_gap(Size::lpx(10.0))
                                    .with_children(vec![
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(
                                                TextContent::new("Counter".to_string())
                                                    .with_font_size(Size::lpx(20.0))
                                                    .with_color(mocha::TEXT)
                                                    .with_h_align(HorizontalAlign::Center)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{}", self.counter))
                                                    .with_font_size(Size::lpx(48.0))
                                                    .with_color(mocha::PEACH)
                                                    .with_h_align(HorizontalAlign::Center)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(
                                                TextContent::new(format!("Toggle: {}", if self.toggle_state { "ON" } else { "OFF" }))
                                                    .with_font_size(Size::lpx(20.0))
                                                    .with_color(if self.toggle_state { mocha::GREEN } else { mocha::RED })
                                                    .with_h_align(HorizontalAlign::Center)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                            ]),
                        // Right side - Translated containers with interactive elements
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_children(vec![
                                // Outer translated container (lavender border)
                                Node::new()
                                    .with_width(Size::lpx(400.0))
                                    .with_height(Size::lpx(400.0))
                                    .with_translation(Translation::new(
                                        Size::Logical(self.outer_translation_x),
                                        Size::Logical(self.outer_translation_y),
                                    ))
                                    .with_shape(Shape::Rect(
                                        StyledRect::new(Default::default(), mocha::CRUST)
                                            .with_stroke(Stroke::new(Size::lpx(3.0), mocha::LAVENDER))
                                            .with_corner_shape(astra_gui::CornerShape::Round(
                                                Size::lpx(50.0),
                                            )),
                                    ))
                                    .with_padding(Spacing::all(Size::lpx(30.0)))
                                    .with_layout_direction(Layout::Vertical)
                                    .with_gap(Size::lpx(20.0))
                                    .with_children(vec![
                                        // Label for outer container
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(
                                                TextContent::new("Outer Container".to_string())
                                                    .with_font_size(Size::lpx(24.0))
                                                    .with_color(mocha::TEXT)
                                                    .with_h_align(HorizontalAlign::Center)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                        // Counter buttons in outer container
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_layout_direction(Layout::Horizontal)
                                            .with_gap(Size::lpx(12.0))
                                            .with_children(vec![
                                                button(
                                                    "decrement_btn",
                                                    "-",
                                                    false,
                                                    &ButtonStyle::default(),
                                                ),
                                                button(
                                                    "increment_btn",
                                                    "+",
                                                    false,
                                                    &ButtonStyle::default(),
                                                ),
                                                button(
                                                    "reset_btn",
                                                    "Reset",
                                                    false,
                                                    &ButtonStyle::default(),
                                                ),
                                            ]),
                                        // Inner translated container (green border)
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_height(Size::lpx(200.0))
                                            .with_translation(Translation::new(
                                                Size::Logical(self.inner_translation_x),
                                                Size::Logical(self.inner_translation_y),
                                            ))
                                            .with_shape(Shape::Rect(
                                                StyledRect::new(Default::default(), mocha::CRUST)
                                                    .with_stroke(Stroke::new(Size::lpx(2.0), mocha::GREEN))
                                                    .with_corner_shape(astra_gui::CornerShape::Cut(
                                                        Size::lpx(20.0),
                                                    )),
                                            ))
                                            .with_padding(Spacing::all(Size::lpx(20.0)))
                                            .with_layout_direction(Layout::Vertical)
                                            .with_gap(Size::lpx(15.0))
                                            .with_children(vec![
                                                // Label for inner container
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_content(Content::Text(
                                                        TextContent::new("Inner Container".to_string())
                                                            .with_font_size(Size::lpx(20.0))
                                                            .with_color(mocha::TEXT)
                                                            .with_h_align(HorizontalAlign::Center)
                                                            .with_v_align(VerticalAlign::Center),
                                                    )),
                                                // Toggle in inner container
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_layout_direction(Layout::Horizontal)
                                                    .with_gap(Size::lpx(12.0))
                                                    .with_children(vec![
                                                        Node::new()
                                                            .with_width(Size::Fill)
                                                            .with_content(Content::Text(
                                                                TextContent::new("Toggle:".to_string())
                                                                    .with_font_size(Size::lpx(18.0))
                                                                    .with_color(mocha::TEXT)
                                                                    .with_h_align(HorizontalAlign::Right)
                                                                    .with_v_align(VerticalAlign::Center),
                                                            )),
                                                        toggle(
                                                            "toggle_switch",
                                                            self.toggle_state,
                                                            false,
                                                            &ToggleStyle::default(),
                                                        ),
                                                    ]),
                                                // Nested translation info
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_content(Content::Text(
                                                        TextContent::new(format!(
                                                            "Total: ({:.0}, {:.0})",
                                                            self.outer_translation_x + self.inner_translation_x,
                                                            self.outer_translation_y + self.inner_translation_y
                                                        ))
                                                            .with_font_size(Size::lpx(16.0))
                                                            .with_color(mocha::TEXT)
                                                            .with_h_align(HorizontalAlign::Center)
                                                            .with_v_align(VerticalAlign::Center),
                                                    )),
                                            ]),
                                    ]),
                            ]),
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

        if button_clicked("reset_btn", events) {
            self.counter = 0;
            self.outer_translation_x = 0.0;
            self.outer_translation_y = 0.0;
            self.inner_translation_x = 0.0;
            self.inner_translation_y = 0.0;
            println!("Reset clicked! Counter: {}", self.counter);
            changed = true;
        }

        // Handle toggle
        if toggle_clicked("toggle_switch", events) {
            self.toggle_state = !self.toggle_state;
            println!("Toggle switched! State: {}", self.toggle_state);
            changed = true;
        }

        // Handle translation sliders
        if slider_drag(
            "outer_x_slider",
            &mut self.outer_translation_x,
            &(-200.0..=200.0),
            events,
            &SliderStyle::default(),
            Some(1.0),
        ) {
            println!("Outer X: {:.1}", self.outer_translation_x);
            changed = true;
        }

        if slider_drag(
            "outer_y_slider",
            &mut self.outer_translation_y,
            &(-200.0..=200.0),
            events,
            &SliderStyle::default(),
            Some(1.0),
        ) {
            println!("Outer Y: {:.1}", self.outer_translation_y);
            changed = true;
        }

        if slider_drag(
            "inner_x_slider",
            &mut self.inner_translation_x,
            &(-100.0..=100.0),
            events,
            &SliderStyle::default(),
            Some(1.0),
        ) {
            println!("Inner X: {:.1}", self.inner_translation_x);
            changed = true;
        }

        if slider_drag(
            "inner_y_slider",
            &mut self.inner_translation_y,
            &(-100.0..=100.0),
            events,
            &SliderStyle::default(),
            Some(1.0),
        ) {
            println!("Inner Y: {:.1}", self.inner_translation_y);
            changed = true;
        }

        changed
    }
}

fn main() {
    run_example::<TranslationExample>();
}
