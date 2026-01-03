//! Slider with value widget example
//!
//! Demonstrates the combined slider + drag value component.
//!
//! Controls:
//! - Drag slider or value field to adjust
//! - Hold Shift while dragging value for precise control (0.1x speed)
//! - Hold Ctrl while dragging value for fast control (10x speed)
//! - Click on value to enter text input mode
//! - Press Enter to confirm or Escape to cancel text input
//! - ESC: quit

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Shape, Size, Spacing,
    StyledRect, TextContent, VerticalAlign,
};
use astra_gui_interactive::{
    slider_with_value, slider_with_value_update, DragValueStyle, SliderStyle,
};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::TargetedEvent;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp, InteractiveState};
use std::ops::RangeInclusive;

struct SliderWithValueState {
    value: f32,
    text_buffer: String,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    focused: bool,
    drag_accumulator: f32,
}

impl SliderWithValueState {
    fn new(value: f32) -> Self {
        Self {
            value,
            text_buffer: String::new(),
            cursor_pos: 0,
            selection: None,
            focused: false,
            drag_accumulator: value,
        }
    }
}

struct SliderWithValueExample {
    interactive: InteractiveState,
    text_engine: TextEngine,
    debug_options: DebugOptions,

    // Application state
    basic_slider: SliderWithValueState,
    clamped_slider: SliderWithValueState,
    stepped_slider: SliderWithValueState,
    disabled_slider: SliderWithValueState,
}

impl ExampleApp for SliderWithValueExample {
    fn new() -> Self {
        Self {
            interactive: InteractiveState::new(),
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            basic_slider: SliderWithValueState::new(42.5),
            clamped_slider: SliderWithValueState::new(50.0),
            stepped_slider: SliderWithValueState::new(10.0),
            disabled_slider: SliderWithValueState::new(99.9),
        }
    }

    fn window_title() -> &'static str {
        "Slider with Value Widget - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (1200, 1000)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
        // Extract values to avoid borrow checker issues
        let basic_val = self.basic_slider.value;
        let basic_focused = self.basic_slider.focused;
        let basic_text = self.basic_slider.text_buffer.clone();
        let basic_cursor = self.basic_slider.cursor_pos;
        let basic_selection = self.basic_slider.selection;

        let clamped_val = self.clamped_slider.value;
        let clamped_focused = self.clamped_slider.focused;
        let clamped_text = self.clamped_slider.text_buffer.clone();
        let clamped_cursor = self.clamped_slider.cursor_pos;
        let clamped_selection = self.clamped_slider.selection;

        let stepped_val = self.stepped_slider.value;
        let stepped_focused = self.stepped_slider.focused;
        let stepped_text = self.stepped_slider.text_buffer.clone();
        let stepped_cursor = self.stepped_slider.cursor_pos;
        let stepped_selection = self.stepped_slider.selection;

        let disabled_val = self.disabled_slider.value;
        let disabled_focused = self.disabled_slider.focused;
        let disabled_text = self.disabled_slider.text_buffer.clone();
        let disabled_cursor = self.disabled_slider.cursor_pos;
        let disabled_selection = self.disabled_slider.selection;

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
                // Basic slider
                self.create_slider_row(
                    "Basic (0-100):",
                    "basic_slider",
                    "basic_value",
                    basic_val,
                    0.0..=100.0,
                    basic_focused,
                    &basic_text,
                    basic_cursor,
                    basic_selection,
                    false,
                ),
                // Clamped slider
                self.create_slider_row(
                    "Clamped (0-100):",
                    "clamped_slider",
                    "clamped_value",
                    clamped_val,
                    0.0..=100.0,
                    clamped_focused,
                    &clamped_text,
                    clamped_cursor,
                    clamped_selection,
                    false,
                ),
                // Stepped slider
                self.create_slider_row(
                    "Stepped (5.0 steps):",
                    "stepped_slider",
                    "stepped_value",
                    stepped_val,
                    0.0..=100.0,
                    stepped_focused,
                    &stepped_text,
                    stepped_cursor,
                    stepped_selection,
                    false,
                ),
                // Disabled slider
                self.create_slider_row(
                    "Disabled:",
                    "disabled_slider",
                    "disabled_value",
                    disabled_val,
                    0.0..=100.0,
                    disabled_focused,
                    &disabled_text,
                    disabled_cursor,
                    disabled_selection,
                    true,
                ),
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

        // Handle slider with value updates
        if slider_with_value_update(
            "basic_slider",
            "basic_value",
            &mut self.basic_slider.value,
            &mut self.basic_slider.text_buffer,
            &mut self.basic_slider.cursor_pos,
            &mut self.basic_slider.selection,
            &mut self.basic_slider.focused,
            &mut self.basic_slider.drag_accumulator,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
            0.0..=100.0,
            0.1, // speed
            None,
            1.0, // No zoom
        ) {
            println!("Basic value: {:.2}", self.basic_slider.value);
            changed = true;
        }

        if slider_with_value_update(
            "clamped_slider",
            "clamped_value",
            &mut self.clamped_slider.value,
            &mut self.clamped_slider.text_buffer,
            &mut self.clamped_slider.cursor_pos,
            &mut self.clamped_slider.selection,
            &mut self.clamped_slider.focused,
            &mut self.clamped_slider.drag_accumulator,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
            0.0..=100.0,
            0.1, // speed
            None,
            1.0, // No zoom
        ) {
            println!("Clamped value: {:.2}", self.clamped_slider.value);
            changed = true;
        }

        if slider_with_value_update(
            "stepped_slider",
            "stepped_value",
            &mut self.stepped_slider.value,
            &mut self.stepped_slider.text_buffer,
            &mut self.stepped_slider.cursor_pos,
            &mut self.stepped_slider.selection,
            &mut self.stepped_slider.focused,
            &mut self.stepped_slider.drag_accumulator,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
            0.0..=100.0,
            0.1,       // speed
            Some(5.0), // step
            1.0,       // No zoom
        ) {
            println!("Stepped value: {:.1}", self.stepped_slider.value);
            changed = true;
        }

        changed
    }
}

impl SliderWithValueExample {
    fn create_slider_row(
        &mut self,
        label: &str,
        slider_id: &str,
        value_id: &str,
        value: f32,
        range: RangeInclusive<f32>,
        focused: bool,
        text_buffer: &str,
        cursor_pos: usize,
        selection: Option<(usize, usize)>,
        disabled: bool,
    ) -> Node {
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
                slider_with_value(
                    slider_id,
                    value_id,
                    value,
                    range,
                    focused,
                    disabled,
                    &SliderStyle::default(),
                    &DragValueStyle::default().with_precision(1),
                    text_buffer,
                    cursor_pos,
                    selection,
                    &mut self.text_engine,
                    &mut self.interactive.event_dispatcher,
                ),
                // Spacer
                Node::new().with_width(Size::Fill),
            ])
    }
}

fn main() {
    run_example::<SliderWithValueExample>();
}
