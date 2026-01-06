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
//! - Debug controls (M/P/B/C/R/G/O/T/D/S)
//! - ESC: quit

mod shared;

use astra_gui::{
    catppuccin::mocha, Content, DebugOptions, HorizontalAlign, Layout, Node, Size, Spacing, Style,
    TextContent, UiContext, VerticalAlign,
};
use astra_gui_interactive::{DragValue, DragValueStyle};
use astra_gui_text::Engine as TextEngine;
use shared::debug_controls::DEBUG_HELP_TEXT_ONELINE;
use shared::{run_example, ExampleApp};
use std::cell::RefCell;
use std::rc::Rc;

/// Shared application state that can be modified from callbacks
struct AppState {
    basic_value: f32,
    clamped_value: f32,
    stepped_value: f32,
    fast_drag_value: f32,
    disabled_value: f32,
}

struct DragValueExample {
    text_engine: TextEngine,
    debug_options: DebugOptions,
    state: Rc<RefCell<AppState>>,
}

impl ExampleApp for DragValueExample {
    fn new() -> Self {
        Self {
            text_engine: TextEngine::new_default(),
            debug_options: DebugOptions::none(),
            state: Rc::new(RefCell::new(AppState {
                basic_value: 42.5,
                clamped_value: 50.0,
                stepped_value: 10.0,
                fast_drag_value: 1000.0,
                disabled_value: 99.9,
            })),
        }
    }

    fn window_title() -> &'static str {
        "Drag Value Widget - Astra GUI"
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
        // Clone state for callbacks
        let state = self.state.clone();

        // Read current values for display
        let (basic_value, clamped_value, stepped_value, fast_drag_value, disabled_value) = {
            let s = state.borrow();
            (
                s.basic_value,
                s.clamped_value,
                s.stepped_value,
                s.fast_drag_value,
                s.disabled_value,
            )
        };

        // Clone state for each callback
        let state_basic = state.clone();
        let state_clamped = state.clone();
        let state_stepped = state.clone();
        let state_fast = state.clone();

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
                    ctx,
                    "Basic (no limits):",
                    basic_value,
                    None,
                    None,
                    0.1,
                    &DragValueStyle::default(),
                    false,
                    move |new_val| {
                        state_basic.borrow_mut().basic_value = new_val;
                        println!("Basic value: {:.2}", new_val);
                    },
                ),
                // Clamped drag value
                self.create_drag_row(
                    ctx,
                    "Clamped (0-100):",
                    clamped_value,
                    Some(0.0..=100.0),
                    None,
                    0.1,
                    &DragValueStyle::default(),
                    false,
                    move |new_val| {
                        state_clamped.borrow_mut().clamped_value = new_val;
                        println!("Clamped value: {:.2}", new_val);
                    },
                ),
                // Stepped drag value
                self.create_drag_row(
                    ctx,
                    "Stepped (5.0 steps):",
                    stepped_value,
                    Some(0.0..=100.0),
                    Some(5.0),
                    0.1,
                    &DragValueStyle::default().with_precision(1),
                    false,
                    move |new_val| {
                        state_stepped.borrow_mut().stepped_value = new_val;
                        println!("Stepped value: {:.1}", new_val);
                    },
                ),
                // Fast drag value
                self.create_drag_row(
                    ctx,
                    "Fast drag (10x speed):",
                    fast_drag_value,
                    None,
                    None,
                    1.0,
                    &DragValueStyle::default().with_precision(0),
                    false,
                    move |new_val| {
                        state_fast.borrow_mut().fast_drag_value = new_val;
                        println!("Fast drag value: {:.0}", new_val);
                    },
                ),
                // Disabled drag value
                self.create_drag_row_disabled(
                    ctx,
                    "Disabled:",
                    disabled_value,
                    &DragValueStyle::default(),
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
}

impl DragValueExample {
    fn create_drag_row<F>(
        &mut self,
        ctx: &mut UiContext,
        label: &str,
        mut value: f32,
        range: Option<std::ops::RangeInclusive<f32>>,
        step: Option<f32>,
        speed: f32,
        style: &DragValueStyle,
        disabled: bool,
        on_change: F,
    ) -> Node
    where
        F: FnMut(f32) + 'static,
    {
        let mut drag_value = DragValue::new(&mut value)
            .speed(speed)
            .disabled(disabled)
            .with_style(style.clone())
            .on_change(on_change);

        if let Some(r) = range {
            drag_value = drag_value.range(r);
        }

        if let Some(s) = step {
            drag_value = drag_value.step(s);
        }

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
                drag_value.build(ctx),
                // Spacer
                Node::new().with_width(Size::Fill),
            ])
    }

    fn create_drag_row_disabled(
        &mut self,
        ctx: &mut UiContext,
        label: &str,
        mut value: f32,
        style: &DragValueStyle,
    ) -> Node {
        let drag_value = DragValue::new(&mut value)
            .disabled(true)
            .with_style(style.clone());

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
                drag_value.build(ctx),
                // Spacer
                Node::new().with_width(Size::Fill),
            ])
    }
}

fn main() {
    run_example::<DragValueExample>();
}
