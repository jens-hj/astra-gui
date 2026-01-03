//! Drag value widget example
//!
//! Demonstrates the drag value component with different configurations.
//!
//! Controls:
//! - Drag left/right on values to adjust them
//! - Hold Shift while dragging for precise control (0.1x speed)
//! - Hold Ctrl while dragging for fast control (10x speed)
//! - Click on value to enter text input mode
//! - Press Enter to confirm or Escape to cancel text input
//! - ESC: quit

#![allow(unused_imports, unused_variables, dead_code)]

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Size, Spacing, Style,
    TextContent, VerticalAlign,
};
use astra_gui_interactive::{drag_value, drag_value_update, DragValueStyle};
use astra_gui_text::Engine as TextEngine;
use astra_gui_wgpu::TargetedEvent;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp, InteractiveState};

struct DragValueState {
    value: f32,
    text_buffer: String,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    focused: bool,
    drag_accumulator: f32,
}

impl DragValueState {
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

struct DragValueExample {
    interactive: InteractiveState,
    text_engine: TextEngine,
    debug_options: DebugOptions,

    // Application state
    basic_value: DragValueState,
    clamped_value: DragValueState,
    stepped_value: DragValueState,
    fast_drag_value: DragValueState,
    disabled_value: DragValueState,
}

impl ExampleApp for DragValueExample {
    fn new() -> Self {
        Self {
            interactive: InteractiveState::new(),
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            basic_value: DragValueState::new(42.5),
            clamped_value: DragValueState::new(50.0),
            stepped_value: DragValueState::new(10.0),
            fast_drag_value: DragValueState::new(1000.0),
            disabled_value: DragValueState::new(99.9),
        }
    }

    fn window_title() -> &'static str {
        "Drag Value Widget - Astra GUI"
    }

    fn window_size() -> (u32, u32) {
        (1100, 800)
    }

    fn build_ui(&mut self, _width: f32, _height: f32) -> Node {
        // Extract values to avoid borrow checker issues
        let basic_val = self.basic_value.value;
        let basic_focused = self.basic_value.focused;
        let basic_text = self.basic_value.text_buffer.clone();
        let basic_cursor = self.basic_value.cursor_pos;
        let basic_selection = self.basic_value.selection;

        let clamped_val = self.clamped_value.value;
        let clamped_focused = self.clamped_value.focused;
        let clamped_text = self.clamped_value.text_buffer.clone();
        let clamped_cursor = self.clamped_value.cursor_pos;
        let clamped_selection = self.clamped_value.selection;

        let stepped_val = self.stepped_value.value;
        let stepped_focused = self.stepped_value.focused;
        let stepped_text = self.stepped_value.text_buffer.clone();
        let stepped_cursor = self.stepped_value.cursor_pos;
        let stepped_selection = self.stepped_value.selection;

        let fast_val = self.fast_drag_value.value;
        let fast_focused = self.fast_drag_value.focused;
        let fast_text = self.fast_drag_value.text_buffer.clone();
        let fast_cursor = self.fast_drag_value.cursor_pos;
        let fast_selection = self.fast_drag_value.selection;

        let disabled_val = self.disabled_value.value;
        let disabled_focused = self.disabled_value.focused;
        let disabled_text = self.disabled_value.text_buffer.clone();
        let disabled_cursor = self.disabled_value.cursor_pos;
        let disabled_selection = self.disabled_value.selection;

        Node::new()
            .with_zoom(2.0)
            .with_width(Size::Fill)
            .with_height(Size::Fill)
            .with_layout_direction(Layout::Vertical)
            .with_gap(Size::lpx(20.0))
            .with_children(vec![
                // Spacer
                Node::new().with_height(Size::Fill),
                // Title
                Node::new()
                    .with_width(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new("Drag Value Widget Example".to_string())
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
                            "Drag left/right to adjust • Click to type • Shift=precise, Ctrl=fast"
                                .to_string(),
                        )
                        .with_font_size(Size::lpx(16.0))
                        .with_color(mocha::SUBTEXT0)
                        .with_h_align(HorizontalAlign::Center)
                        .with_v_align(VerticalAlign::Center),
                    )),
                Node::new().with_height(Size::lpx(20.0)),
                // Basic drag value
                self.create_drag_row(
                    "Basic (no limits):",
                    "basic_drag",
                    basic_val,
                    basic_focused,
                    &basic_text,
                    basic_cursor,
                    basic_selection,
                    &DragValueStyle::default(),
                    false,
                ),
                // Clamped drag value
                self.create_drag_row(
                    "Clamped (0-100):",
                    "clamped_drag",
                    clamped_val,
                    clamped_focused,
                    &clamped_text,
                    clamped_cursor,
                    clamped_selection,
                    &DragValueStyle::default(),
                    false,
                ),
                // Stepped drag value
                self.create_drag_row(
                    "Stepped (5.0 steps):",
                    "stepped_drag",
                    stepped_val,
                    stepped_focused,
                    &stepped_text,
                    stepped_cursor,
                    stepped_selection,
                    &DragValueStyle::default().with_precision(1),
                    false,
                ),
                // Fast drag value
                self.create_drag_row(
                    "Fast drag (10x speed):",
                    "fast_drag",
                    fast_val,
                    fast_focused,
                    &fast_text,
                    fast_cursor,
                    fast_selection,
                    &DragValueStyle::default().with_precision(0),
                    false,
                ),
                // Disabled drag value
                self.create_drag_row(
                    "Disabled:",
                    "disabled_drag",
                    disabled_val,
                    disabled_focused,
                    &disabled_text,
                    disabled_cursor,
                    disabled_selection,
                    &DragValueStyle::default(),
                    true,
                ),
                // Spacer
                Node::new().with_height(Size::Fill),
                // Help bar
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

    fn interactive_state(&mut self) -> Option<&mut InteractiveState> {
        Some(&mut self.interactive)
    }

    fn debug_options_mut(&mut self) -> Option<&mut DebugOptions> {
        Some(&mut self.debug_options)
    }

    fn handle_events(&mut self, events: &[TargetedEvent]) -> bool {
        let mut changed = false;

        // Handle drag value updates
        if drag_value_update(
            "basic_drag",
            &mut self.basic_value.value,
            &mut self.basic_value.text_buffer,
            &mut self.basic_value.cursor_pos,
            &mut self.basic_value.selection,
            &mut self.basic_value.focused,
            &mut self.basic_value.drag_accumulator,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
            None,
            0.1, // speed
            None,
        ) {
            println!("Basic value: {:.2}", self.basic_value.value);
            changed = true;
        }

        if drag_value_update(
            "clamped_drag",
            &mut self.clamped_value.value,
            &mut self.clamped_value.text_buffer,
            &mut self.clamped_value.cursor_pos,
            &mut self.clamped_value.selection,
            &mut self.clamped_value.focused,
            &mut self.clamped_value.drag_accumulator,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
            Some(0.0..=100.0), // range
            0.1,               // speed
            None,
        ) {
            println!("Clamped value: {:.2}", self.clamped_value.value);
            changed = true;
        }

        if drag_value_update(
            "stepped_drag",
            &mut self.stepped_value.value,
            &mut self.stepped_value.text_buffer,
            &mut self.stepped_value.cursor_pos,
            &mut self.stepped_value.selection,
            &mut self.stepped_value.focused,
            &mut self.stepped_value.drag_accumulator,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
            Some(0.0..=100.0), // range
            0.1,               // speed
            Some(5.0),         // step
        ) {
            println!("Stepped value: {:.1}", self.stepped_value.value);
            changed = true;
        }

        if drag_value_update(
            "fast_drag",
            &mut self.fast_drag_value.value,
            &mut self.fast_drag_value.text_buffer,
            &mut self.fast_drag_value.cursor_pos,
            &mut self.fast_drag_value.selection,
            &mut self.fast_drag_value.focused,
            &mut self.fast_drag_value.drag_accumulator,
            events,
            &self.interactive.input_state,
            &mut self.interactive.event_dispatcher,
            None,
            1.0, // faster base speed
            None,
        ) {
            println!("Fast drag value: {:.2}", self.fast_drag_value.value);
            changed = true;
        }

        changed
    }
}

impl DragValueExample {
    fn create_drag_row(
        &mut self,
        label: &str,
        id: &str,
        value: f32,
        focused: bool,
        text_buffer: &str,
        cursor_pos: usize,
        selection: Option<(usize, usize)>,
        style: &DragValueStyle,
        disabled: bool,
    ) -> Node {
        Node::new()
            .with_width(Size::Fill)
            .with_layout_direction(Layout::Horizontal)
            .with_gap(Size::lpx(16.0))
            .with_children(vec![
                // Spacer
                Node::new().with_width(Size::Fill),
                // Label
                Node::new()
                    .with_width(Size::lpx(220.0))
                    .with_height(Size::Fill)
                    .with_content(Content::Text(
                        TextContent::new(label.to_string())
                            .with_font_size(Size::lpx(20.0))
                            .with_color(mocha::TEXT)
                            .with_h_align(HorizontalAlign::Right)
                            .with_v_align(VerticalAlign::Center),
                    )),
                // Drag value widget
                drag_value(
                    id,
                    value,
                    focused,
                    disabled,
                    style,
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
    run_example::<DragValueExample>();
}
