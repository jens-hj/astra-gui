//! Rotation example
//!
//! Demonstrates rotation with nested rotations and interactive elements.
//!
//! Controls:
//! - Click buttons inside rotated containers to verify hit testing
//! - Drag sliders to adjust rotation angles
//! - Toggle switches work even when rotated
//! - ESC: quit

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape, Size, Spacing,
    Stroke, StyledRect, TextContent, TransformOrigin, VerticalAlign,
};
use astra_gui_interactive::{
    button, button_clicked, slider, slider_drag, toggle, toggle_clicked, ButtonStyle, SliderStyle,
    ToggleStyle,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::TargetedEvent;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp, InteractiveState};

struct RotationExample {
    interactive: InteractiveState,
    text_engine: TextEngine,
    debug_options: DebugOptions,

    // Application state
    outer_rotation: f32, // Degrees
    inner_rotation: f32, // Degrees
    counter: i32,
    toggle_state: bool,
}

impl ExampleApp for RotationExample {
    fn new() -> Self {
        Self {
            interactive: InteractiveState::new(),
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            outer_rotation: 30.0,
            inner_rotation: 0.0,
            counter: 0,
            toggle_state: true,
        }
    }

    fn window_title() -> &'static str {
        "Rotation Example - Astra GUI"
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
                        TextContent::new("Transform Rotation Example".to_string())
                            .with_font_size(Size::lpx(32.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Instructions
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("Adjust sliders to rotate containers. Click buttons to verify hit testing works!".to_string())
                            .with_font_size(Size::lpx(16.0))
                            .with_color(mocha::SUBTEXT0)
                            .with_h_align(HorizontalAlign::Center)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Main content area with rotated containers
                Node::new()
                    .with_width(Size::Fill)
                    .with_height(Size::Fill)
                    .with_layout_direction(Layout::Horizontal)
                    .with_gap(Size::lpx(40.0))
                    .with_children(vec![
                        // Spacer
                        Node::new().with_width(Size::Fill),
                        // Left side - Outer rotation control
                        Node::new()
                            .with_width(Size::lpx(300.0))
                            .with_height(Size::Fill)
                            .with_padding(Spacing::all(Size::lpx(20.0)))
                            .with_layout_direction(Layout::Vertical)
                            .with_gap(Size::lpx(20.0))
                            .with_children(vec![
                                // Outer rotation slider section
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new("Outer Container Rotation".to_string())
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
                                            "outer_rotation_slider",
                                            self.outer_rotation,
                                            -180.0..=180.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}°", self.outer_rotation))
                                                    .with_font_size(Size::lpx(18.0))
                                                    .with_color(mocha::LAVENDER)
                                                    .with_h_align(HorizontalAlign::Right)
                                                    .with_v_align(VerticalAlign::Center),
                                            )),
                                    ]),
                                // Inner rotation slider section
                                Node::new()
                                    .with_width(Size::Fill)
                                    .with_content(Content::Text(
                                        TextContent::new("Inner Container Rotation".to_string())
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
                                            "inner_rotation_slider",
                                            self.inner_rotation,
                                            -180.0..=180.0,
                                            false,
                                            &SliderStyle::default(),
                                        ),
                                        Node::new()
                                            .with_width(Size::lpx(60.0))
                                            .with_content(Content::Text(
                                                TextContent::new(format!("{:.0}°", self.inner_rotation))
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
                        // Right side - Rotated containers with interactive elements
                        Node::new()
                            .with_width(Size::Fill)
                            .with_height(Size::Fill)
                            .with_children(vec![
                                // Outer rotated container (lavender)
                                Node::new()
                                    .with_width(Size::lpx(400.0))
                                    .with_height(Size::lpx(400.0))
                                    .with_rotation(self.outer_rotation.to_radians())
                                    .with_transform_origin(TransformOrigin::center())
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
                                        // Inner rotated container (green)
                                        Node::new()
                                            .with_width(Size::Fill)
                                            .with_height(Size::lpx(200.0))
                                            .with_rotation(self.inner_rotation.to_radians())
                                            .with_transform_origin(TransformOrigin::center())
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
                                                // Nested rotation info
                                                Node::new()
                                                    .with_width(Size::Fill)
                                                    .with_content(Content::Text(
                                                        TextContent::new(format!(
                                                            "Total: {:.0}°",
                                                            self.outer_rotation + self.inner_rotation
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
            self.outer_rotation = 0.0;
            self.inner_rotation = 0.0;
            println!("Reset clicked! Counter: {}", self.counter);
            changed = true;
        }

        if toggle_clicked("toggle_switch", events) {
            self.toggle_state = !self.toggle_state;
            println!("Toggle clicked! State: {}", self.toggle_state);
            changed = true;
        }

        // Handle rotation sliders
        if slider_drag(
            "outer_rotation_slider",
            &mut self.outer_rotation,
            &(-180.0..=180.0),
            events,
            &SliderStyle::default(),
            Some(1.0),
            1.0, // No zoom
        ) {
            println!("Outer rotation: {:.1}°", self.outer_rotation);
            changed = true;
        }

        if slider_drag(
            "inner_rotation_slider",
            &mut self.inner_rotation,
            &(-180.0..=180.0),
            events,
            &SliderStyle::default(),
            Some(1.0),
            1.0, // No zoom
        ) {
            println!("Inner rotation: {:.1}°", self.inner_rotation);
            changed = true;
        }

        changed
    }
}

fn main() {
    run_example::<RotationExample>();
}
