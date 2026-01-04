//! Collapsible container example
//!
//! Demonstrates collapsible/expandable containers with smooth animations.
//!
//! Controls:
//! - Click headers to expand/collapse sections
//! - Nested collapsibles work independently
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape, Size, Spacing,
    TextContent, VerticalAlign,
};
use astra_gui_interactive::{
    button, button_clicked, collapsible, collapsible_clicked, slider, slider_drag, toggle,
    toggle_clicked, ButtonStyle, CollapsibleStyle, SliderStyle, ToggleStyle,
};
use astra_gui_text::Engine as TextEngine;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp, InteractiveState};

struct CollapsibleExample {
    interactive: InteractiveState,
    text_engine: TextEngine,
    debug_options: DebugOptions,

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

impl ExampleApp for CollapsibleExample {
    fn new() -> Self {
        Self {
            interactive: InteractiveState::new(),
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),

            section1_expanded: true,
            section2_expanded: false,
            section3_expanded: true,
            nested1_expanded: false,
            nested2_expanded: true,

            toggle_value: false,
            slider_value: 50.0,
            counter: 0,
        }
    }

    fn window_title() -> &'static str {
        "Collapsible Containers - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (800, 900)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
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
                // Section 1: Basic Collapsible
                collapsible(
                    "section1",
                    "Basic Collapsible",
                    self.section1_expanded,
                    false,
                    vec![
                        Node::new()
                            .with_width(Size::Fill)
                            .with_content(Content::Text(
                                TextContent::new(
                                    "This is a simple collapsible section with text content."
                                        .to_string(),
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
                                TextContent::new(
                                    "Click the header to collapse this section.".to_string(),
                                )
                                .with_font_size(Size::lpx(20.0))
                                .with_color(mocha::SUBTEXT1)
                                .with_h_align(HorizontalAlign::Left)
                                .with_v_align(VerticalAlign::Center),
                            )),
                    ],
                    &CollapsibleStyle::default(),
                ),
                // Section 2: Interactive Components
                collapsible(
                    "section2",
                    "Interactive Components",
                    self.section2_expanded,
                    false,
                    vec![
                        Node::new()
                            .with_width(Size::Fill)
                            .with_layout_direction(Layout::Horizontal)
                            .with_gap(Size::lpx(16.0))
                            .with_children(vec![Node::new().with_width(Size::Fill).with_content(
                                Content::Text(
                                    TextContent::new(format!("Counter: {}", self.counter))
                                        .with_font_size(Size::lpx(28.0))
                                        .with_color(mocha::LAVENDER)
                                        .with_h_align(HorizontalAlign::Center)
                                        .with_v_align(VerticalAlign::Center),
                                ),
                            )]),
                        Node::new().with_height(Size::lpx(8.0)),
                        Node::new()
                            .with_width(Size::Fill)
                            .with_layout_direction(Layout::Horizontal)
                            .with_gap(Size::lpx(16.0))
                            .with_children(vec![
                                Node::new().with_width(Size::Fill),
                                button("decrement", "-", false, &ButtonStyle::default()),
                                button("increment", "+", false, &ButtonStyle::default()),
                                Node::new().with_width(Size::Fill),
                            ]),
                        Node::new().with_height(Size::lpx(8.0)),
                        Node::new()
                            .with_width(Size::Fill)
                            .with_layout_direction(Layout::Horizontal)
                            .with_gap(Size::lpx(16.0))
                            .with_children(vec![
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new("Enable:".to_string())
                                            .with_font_size(Size::lpx(24.0))
                                            .with_color(mocha::TEXT)
                                            .with_h_align(HorizontalAlign::Right)
                                            .with_v_align(VerticalAlign::Center),
                                    )),
                                toggle("toggle", self.toggle_value, false, &ToggleStyle::default()),
                                Node::new().with_width(Size::Fill),
                            ]),
                        Node::new().with_height(Size::lpx(8.0)),
                        slider(
                            "slider",
                            self.slider_value,
                            0.0..=100.0,
                            false,
                            &SliderStyle::default(),
                        ),
                    ],
                    &CollapsibleStyle::default(),
                ),
                // Section 3: Nested Collapsibles
                collapsible(
                    "section3",
                    "Nested Collapsibles",
                    self.section3_expanded,
                    false,
                    vec![
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
                        // Nested collapsible 1
                        collapsible(
                            "nested1",
                            "Nested Level 1",
                            self.nested1_expanded,
                            false,
                            vec![Node::new()
                                .with_width(Size::Fill)
                                .with_content(Content::Text(
                                    TextContent::new(
                                        "This is content inside the first nested collapsible."
                                            .to_string(),
                                    )
                                    .with_font_size(Size::lpx(20.0))
                                    .with_color(mocha::SUBTEXT1)
                                    .with_h_align(HorizontalAlign::Left)
                                    .with_v_align(VerticalAlign::Center),
                                ))],
                            &CollapsibleStyle::default(),
                        ),
                        Node::new().with_height(Size::lpx(8.0)),
                        // Nested collapsible 2
                        collapsible(
                            "nested2",
                            "Nested Level 2",
                            self.nested2_expanded,
                            false,
                            vec![
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new(
                                            "This nested section contains a button:".to_string(),
                                        )
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
                                        button(
                                            "nested_btn",
                                            "Click Me!",
                                            false,
                                            &ButtonStyle::default(),
                                        ),
                                        Node::new().with_width(Size::Fill),
                                    ]),
                            ],
                            &CollapsibleStyle::default(),
                        ),
                    ],
                    &CollapsibleStyle::default(),
                ),
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

    fn handle_events(&mut self, events: &[astra_gui_wgpu::TargetedEvent]) -> bool {
        let mut changed = false;

        // Handle collapsible clicks
        if collapsible_clicked("section1", events) {
            self.section1_expanded = !self.section1_expanded;
            changed = true;
        }
        if collapsible_clicked("section2", events) {
            self.section2_expanded = !self.section2_expanded;
            changed = true;
        }
        if collapsible_clicked("section3", events) {
            self.section3_expanded = !self.section3_expanded;
            changed = true;
        }
        if collapsible_clicked("nested1", events) {
            self.nested1_expanded = !self.nested1_expanded;
            changed = true;
        }
        if collapsible_clicked("nested2", events) {
            self.nested2_expanded = !self.nested2_expanded;
            changed = true;
        }

        // Handle button clicks
        if button_clicked("increment", events) {
            self.counter += 1;
            changed = true;
        }
        if button_clicked("decrement", events) {
            self.counter -= 1;
            changed = true;
        }
        if button_clicked("nested_btn", events) {
            println!("Nested button clicked!");
            changed = true;
        }

        // Handle toggle
        if toggle_clicked("toggle", events) {
            self.toggle_value = !self.toggle_value;
            changed = true;
        }

        // Handle slider
        if slider_drag(
            "slider",
            &mut self.slider_value,
            &(0.0..=100.0),
            events,
            &SliderStyle::default(),
            Some(1.0),
        ) {
            changed = true;
        }

        changed
    }

    fn interactive_state(&mut self) -> Option<&mut InteractiveState> {
        Some(&mut self.interactive)
    }

    fn text_measurer(&mut self) -> Option<&mut TextEngine> {
        Some(&mut self.text_engine)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }
}

fn main() {
    run_example::<CollapsibleExample>();
}
