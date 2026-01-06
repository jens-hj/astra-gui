//! Collapsible container example
//!
//! Demonstrates collapsible/expandable containers with smooth animations
//! using the new builder pattern API with automatic state management.
//!
//! Controls:
//! - Click headers to expand/collapse sections
//! - Nested collapsibles work independently
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::CornerShape;
use astra_gui::{
    catppuccin::mocha, Component, Content, DebugOptions, HorizontalAlign, Layout, Node, Size,
    Spacing, TextContent, UiContext, VerticalAlign,
};
use astra_gui_interactive::{
    Button, ButtonStyle, Collapsible, CollapsibleStyle, Slider, SliderStyle, Toggle, ToggleStyle,
};
use astra_gui_text::Engine as TextEngine;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp};
use std::cell::RefCell;
use std::rc::Rc;

/// Shared application state
struct AppState {
    // Collapsible states
    section1_expanded: bool,
    section2_expanded: bool,
    section3_expanded: bool,
    nested1_expanded: bool,
    nested2_expanded: bool,

    // Component states
    toggle_value: bool,
    slider_value: f32,
    counter: i32,
}

struct CollapsibleExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    state: Rc<RefCell<AppState>>,
}

impl ExampleApp for CollapsibleExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            state: Rc::new(RefCell::new(AppState {
                section1_expanded: true,
                section2_expanded: false,
                section3_expanded: true,
                nested1_expanded: false,
                nested2_expanded: true,
                toggle_value: false,
                slider_value: 50.0,
                counter: 0,
            })),
        }
    }

    fn window_title() -> &'static str {
        "Collapsible Containers - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (800, 900)
    }

    fn text_engine(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn build_ui(&mut self, ctx: &mut UiContext, _width: f32, _height: f32) -> Node {
        // Read current state
        let (
            section1_expanded,
            section2_expanded,
            section3_expanded,
            nested1_expanded,
            nested2_expanded,
            toggle_value,
            slider_value,
            counter,
        ) = {
            let s = self.state.borrow();
            (
                s.section1_expanded,
                s.section2_expanded,
                s.section3_expanded,
                s.nested1_expanded,
                s.nested2_expanded,
                s.toggle_value,
                s.slider_value,
                s.counter,
            )
        };

        // Clone state for callbacks
        let state_section1 = self.state.clone();
        let state_section2 = self.state.clone();
        let state_section3 = self.state.clone();
        let state_nested1 = self.state.clone();
        let state_nested2 = self.state.clone();
        let state_dec = self.state.clone();
        let state_inc = self.state.clone();
        let state_toggle = self.state.clone();
        let state_slider = self.state.clone();

        // Build nested collapsibles first (they need ctx)
        let nested1 = Collapsible::new("Nested Level 1", nested1_expanded)
            .with_style(
                CollapsibleStyle::default().with_corners(CornerShape::Round(Size::lpx(10.0))),
            )
            .on_toggle(move |expanded| {
                state_nested1.borrow_mut().nested1_expanded = expanded;
            })
            .children(vec![Node::new().with_width(Size::Fill).with_content(
                Content::Text(
                    TextContent::new(
                        "This is content inside the first nested collapsible.".to_string(),
                    )
                    .with_font_size(Size::lpx(20.0))
                    .with_color(mocha::SUBTEXT1)
                    .with_h_align(HorizontalAlign::Left)
                    .with_v_align(VerticalAlign::Center),
                ),
            )])
            .node(ctx);

        let nested2_btn = Button::new("Click Me!")
            .with_style(ButtonStyle::default())
            .on_click(|| {
                println!("Nested button clicked!");
            })
            .node(ctx);

        let nested2 = Collapsible::new("Nested Level 2", nested2_expanded)
            .with_style(
                CollapsibleStyle::default().with_corners(CornerShape::Round(Size::lpx(10.0))),
            )
            .on_toggle(move |expanded| {
                state_nested2.borrow_mut().nested2_expanded = expanded;
            })
            .children(vec![
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("This nested section contains a button:".to_string())
                            .with_font_size(Size::lpx(20.0))
                            .with_color(mocha::SUBTEXT1)
                            .with_h_align(HorizontalAlign::Left)
                            .with_v_align(VerticalAlign::Center),
                    )),
                Node::new().with_height(Size::lpx(8.0)),
                Node::new()
                    .with_width(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_children(vec![
                        Node::new().with_width(Size::Fill),
                        nested2_btn,
                        Node::new().with_width(Size::Fill),
                    ]),
            ])
            .node(ctx);

        // Build interactive components for section 2
        let dec_btn = Button::new("-")
            .with_style(ButtonStyle::default())
            .on_click(move || {
                state_dec.borrow_mut().counter -= 1;
            })
            .node(ctx);

        let inc_btn = Button::new("+")
            .with_style(ButtonStyle::default())
            .on_click(move || {
                state_inc.borrow_mut().counter += 1;
            })
            .node(ctx);

        let toggle_widget = Toggle::new(toggle_value)
            .with_style(ToggleStyle::default())
            .on_toggle(move |value| {
                state_toggle.borrow_mut().toggle_value = value;
            })
            .node(ctx);

        let slider_widget = Slider::new(slider_value, 0.0..=100.0)
            .step(1.0)
            .with_style(SliderStyle::default())
            .on_change(move |value| {
                state_slider.borrow_mut().slider_value = value;
            })
            .node(ctx);

        // Build main collapsibles
        let section1 = Collapsible::new("Basic Collapsible", section1_expanded)
            .with_style(CollapsibleStyle::default().with_corners(CornerShape::Cut(Size::lpx(16.0))))
            .on_toggle(move |expanded| {
                state_section1.borrow_mut().section1_expanded = expanded;
            })
            .children(vec![
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new(
                            "This is a simple collapsible section with text content.".to_string(),
                        )
                        .with_font_size(Size::lpx(24.0))
                        .with_color(mocha::TEXT)
                        .with_h_align(HorizontalAlign::Left)
                        .with_v_align(VerticalAlign::Center),
                    )),
                Node::new().with_height(Size::lpx(8.0)),
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("Click the header to collapse this section.".to_string())
                            .with_font_size(Size::lpx(20.0))
                            .with_color(mocha::SUBTEXT1)
                            .with_h_align(HorizontalAlign::Left)
                            .with_v_align(VerticalAlign::Center),
                    )),
            ])
            .node(ctx);

        let section2 = Collapsible::new("Interactive Components", section2_expanded)
            .with_style(
                CollapsibleStyle::default()
                    .with_corners(CornerShape::InverseRound(Size::lpx(14.0))),
            )
            .on_toggle(move |expanded| {
                state_section2.borrow_mut().section2_expanded = expanded;
            })
            .children(vec![Node::new()
                .with_width(Size::Fill)
                .with_layout_direction(Layout::Vertical)
                .with_h_align(HorizontalAlign::Center)
                .with_gap(Size::lpx(16.0))
                .with_children(vec![
                    Node::new()
                        .with_width(Size::Fill)
                        .with_content(Content::Text(
                            TextContent::new(format!("Counter: {}", counter))
                                .with_font_size(Size::lpx(28.0))
                                .with_color(mocha::LAVENDER)
                                .with_h_align(HorizontalAlign::Center)
                                .with_v_align(VerticalAlign::Center),
                        )),
                    Node::new()
                        .with_layout_direction(Layout::Horizontal)
                        .with_gap(Size::lpx(16.0))
                        .with_children(vec![dec_btn, inc_btn]),
                    Node::new()
                        .with_layout_direction(Layout::Horizontal)
                        .with_gap(Size::lpx(16.0))
                        .with_children(vec![
                            Node::new().with_content(Content::Text(
                                TextContent::new("Enable:".to_string())
                                    .with_font_size(Size::lpx(24.0))
                                    .with_color(mocha::TEXT)
                                    .with_h_align(HorizontalAlign::Right)
                                    .with_v_align(VerticalAlign::Center),
                            )),
                            toggle_widget,
                        ]),
                    slider_widget,
                ])])
            .node(ctx);

        let section3 = Collapsible::new("Nested Collapsibles", section3_expanded)
            .with_style(CollapsibleStyle::default())
            .on_toggle(move |expanded| {
                state_section3.borrow_mut().section3_expanded = expanded;
            })
            .children(vec![
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new(
                            "Collapsibles can be nested inside each other:".to_string(),
                        )
                        .with_font_size(Size::lpx(24.0))
                        .with_color(mocha::TEXT)
                        .with_h_align(HorizontalAlign::Left)
                        .with_v_align(VerticalAlign::Center),
                    )),
                Node::new().with_height(Size::lpx(8.0)),
                nested1,
                Node::new().with_height(Size::lpx(8.0)),
                nested2,
            ])
            .node(ctx);

        Node::new()
            .with_zoom(1.5)
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::lpx(16.0))
            .with_padding(Spacing::all(Size::lpx(24.0)))
            .with_children(vec![
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("Collapsible Containers".to_string())
                            .with_font_size(Size::lpx(40.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Subtitle
                Node::new()
                    .with_width(Size::Fill)
                    .with_margin(Spacing::bottom(Size::lpx(50.0)))
                    .with_content(Content::Text(
                        TextContent::new("Click headers to expand/collapse".to_string())
                            .with_font_size(Size::lpx(20.0))
                            .with_color(mocha::SUBTEXT0)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Sections
                section1,
                section2,
                section3,
                // Spacer
                Node::new().with_height(Size::Fill),
                // Debug help
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new(DEBUG_HELP_TEXT_ONELINE.to_string())
                            .with_font_size(Size::lpx(16.0))
                            .with_color(mocha::OVERLAY0)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
            ])
    }
}

fn main() {
    run_example::<CollapsibleExample>();
}
