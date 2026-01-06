//! Translation example demonstrating transform translations with interactive components.
//!
//! Controls:
//! - Use sliders to adjust X/Y translation of outer/inner containers
//! - Click +/- buttons to change counter
//! - Click toggle switch to test hit testing in translated containers
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

mod shared;

use astra_gui::{
    catppuccin::mocha, Component, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape,
    Size, Spacing, Stroke, StyledRect, TextContent, Translation, UiContext, VerticalAlign,
};
use astra_gui_interactive::{Button, ButtonStyle, Slider, SliderStyle, Toggle, ToggleStyle};
use astra_gui_text::Engine as TextEngine;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp};
use std::cell::RefCell;
use std::rc::Rc;

/// Shared application state that can be modified from callbacks
struct AppState {
    outer_translation_x: f32,
    outer_translation_y: f32,
    inner_translation_x: f32,
    inner_translation_y: f32,
    counter: i32,
    toggle_state: bool,
}

struct TranslationExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    state: Rc<RefCell<AppState>>,
}

impl ExampleApp for TranslationExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            state: Rc::new(RefCell::new(AppState {
                outer_translation_x: 50.0,
                outer_translation_y: 30.0,
                inner_translation_x: 20.0,
                inner_translation_y: 20.0,
                counter: 0,
                toggle_state: false,
            })),
        }
    }

    fn window_title() -> &'static str {
        "Translation Example - Astra GUI"
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

    fn build_ui(&mut self, ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        // Clone state for callbacks
        let state = self.state.clone();

        // Read current values for display
        let (outer_x, outer_y, inner_x, inner_y, counter, toggle_state) = {
            let s = state.borrow();
            (
                s.outer_translation_x,
                s.outer_translation_y,
                s.inner_translation_x,
                s.inner_translation_y,
                s.counter,
                s.toggle_state,
            )
        };

        // Clone state for each callback
        let state_outer_x = state.clone();
        let state_outer_y = state.clone();
        let state_inner_x = state.clone();
        let state_inner_y = state.clone();
        let state_dec = state.clone();
        let state_inc = state.clone();
        let state_reset = state.clone();
        let state_toggle = state.clone();

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
                        TextContent::new(
                            "Adjust sliders to translate containers. Nested translations should accumulate."
                                .to_string(),
                        )
                        .with_font_size(Size::lpx(16.0))
                        .with_color(mocha::SUBTEXT0)
                        .with_h_align(HorizontalAlign::Center)
                        .with_v_align(VerticalAlign::Center),
                    )),
                // Main content area with horizontal layout
                Node::new()
                    .with_width(Size::Fill)
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
                                        Slider::new(outer_x, -200.0..=200.0)
                                            .step(1.0)
                                            .with_style(SliderStyle::default())
                                            .on_change(move |new_val| {
                                                state_outer_x.borrow_mut().outer_translation_x =
                                                    new_val;
                                                println!("Outer X: {:.1}", new_val);
                                            })
                                            .node(ctx),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", outer_x))
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
                                        Slider::new(outer_y, -200.0..=200.0)
                                            .step(1.0)
                                            .with_style(SliderStyle::default())
                                            .on_change(move |new_val| {
                                                state_outer_y.borrow_mut().outer_translation_y =
                                                    new_val;
                                                println!("Outer Y: {:.1}", new_val);
                                            })
                                            .node(ctx),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", outer_y))
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
                                        Slider::new(inner_x, -100.0..=100.0)
                                            .step(1.0)
                                            .with_style(SliderStyle::default())
                                            .on_change(move |new_val| {
                                                state_inner_x.borrow_mut().inner_translation_x =
                                                    new_val;
                                                println!("Inner X: {:.1}", new_val);
                                            })
                                            .node(ctx),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", inner_x))
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
                                        Slider::new(inner_y, -100.0..=100.0)
                                            .step(1.0)
                                            .with_style(SliderStyle::default())
                                            .on_change(move |new_val| {
                                                state_inner_y.borrow_mut().inner_translation_y =
                                                    new_val;
                                                println!("Inner Y: {:.1}", new_val);
                                            })
                                            .node(ctx),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_height(Size::lpx(24.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}", inner_y))
                                                    .with_font_size(Size::lpx(18.0))
                                                    .with_color(mocha::LAVENDER)
                                                    .with_h_align(HorizontalAlign::Right)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                                // Counter display
                                Node::new()
                                    .with_width(Size::Fill)
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
                                                TextContent::new(format!("{}", counter))
                                                    .with_font_size(Size::lpx(48.0))
                                                    .with_color(mocha::PEACH)
                                                    .with_h_align(HorizontalAlign::Center)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_content(Content::Text(
                                                TextContent::new(format!(
                                                    "Toggle: {}",
                                                    if toggle_state { "ON" } else { "OFF" }
                                                ))
                                                .with_font_size(Size::lpx(20.0))
                                                .with_color(if toggle_state {
                                                    mocha::GREEN
                                                } else {
                                                    mocha::RED
                                                })
                                                .with_h_align(HorizontalAlign::Center)
                                                .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                            ]),
                        // Right side - Translated containers with interactive elements
                        Node::new()
                            .with_width(Size::Fill)
                            .with_children(vec![
                                // Outer translated container (lavender border)
                                Node::new()
                                    .with_width(Size::lpx(400.0))
                                    .with_height(Size::lpx(400.0))
                                    .with_translation(Translation::new(
                                        Size::Logical(outer_x),
                                        Size::Logical(outer_y),
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
                                                Button::new("-")
                                                    .with_style(ButtonStyle::default())
                                                    .on_click({
                                                        let state = state_dec.clone();
                                                        move || {
                                                            let mut s = state.borrow_mut();
                                                            s.counter -= 1;
                                                            println!(
                                                                "Decrement clicked! Counter: {}",
                                                                s.counter
                                                            );
                                                        }
                                                    })
                                                    .node(ctx),
                                                Button::new("+")
                                                    .with_style(ButtonStyle::default())
                                                    .on_click({
                                                        let state = state_inc.clone();
                                                        move || {
                                                            let mut s = state.borrow_mut();
                                                            s.counter += 1;
                                                            println!(
                                                                "Increment clicked! Counter: {}",
                                                                s.counter
                                                            );
                                                        }
                                                    })
                                                    .node(ctx),
                                                Button::new("Reset")
                                                    .with_style(ButtonStyle::default())
                                                    .on_click({
                                                        let state = state_reset.clone();
                                                        move || {
                                                            let mut s = state.borrow_mut();
                                                            s.counter = 0;
                                                            s.outer_translation_x = 0.0;
                                                            s.outer_translation_y = 0.0;
                                                            s.inner_translation_x = 0.0;
                                                            s.inner_translation_y = 0.0;
                                                            println!("Reset clicked!");
                                                        }
                                                    })
                                                    .node(ctx),
                                            ]),
                                        // Inner translated container (green border)
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_height(Size::lpx(200.0))
                                            .with_translation(Translation::new(
                                                Size::Logical(inner_x),
                                                Size::Logical(inner_y),
                                            ))
                                            .with_shape(Shape::Rect(
                                                StyledRect::new(Default::default(), mocha::CRUST)
                                                    .with_stroke(Stroke::new(
                                                        Size::lpx(2.0),
                                                        mocha::GREEN,
                                                    ))
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
                                                        TextContent::new(
                                                            "Inner Container".to_string(),
                                                        )
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
                                                                TextContent::new(
                                                                    "Toggle:".to_string(),
                                                                )
                                                                .with_font_size(Size::lpx(18.0))
                                                                .with_color(mocha::TEXT)
                                                                .with_h_align(HorizontalAlign::Right)
                                                                .with_v_align(VerticalAlign::Center),
                                                            )),
                                                        Toggle::new(toggle_state)
                                                            .with_style(ToggleStyle::default())
                                                            .on_toggle({
                                                                let state = state_toggle.clone();
                                                                move |new_state| {
                                                                    state.borrow_mut().toggle_state =
                                                                        new_state;
                                                                    println!(
                                                                        "Toggle clicked! State: {}",
                                                                        new_state
                                                                    );
                                                                }
                                                            })
                                                            .node(ctx),
                                                    ]),
                                                // Nested translation info
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_content(Content::Text(
                                                        TextContent::new(format!(
                                                            "Total: ({:.0}, {:.0})",
                                                            outer_x + inner_x,
                                                            outer_y + inner_y
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
}

fn main() {
    run_example::<TranslationExample>();
}
