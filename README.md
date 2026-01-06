# Astra GUI

A graphics backend agnostic UI library for Rust.

## Overview

Astra GUI is a modular UI library that separates core UI logic from rendering backends. It provides the fundamental building blocks for creating user interfaces with full control over the rendering pipeline.

## Architecture

The library is split into several crates:

- **astra-gui**: Core UI primitives, logic, input handling, and event system
- **astra-gui-fonts**: Bundled default fonts (Inter, JetBrains Mono)
- **astra-gui-text**: Backend-agnostic text shaping and glyph rasterization
- **astra-gui-wgpu**: WGPU rendering backend with winit integration
- **astra-gui-interactive**: Interactive components library (Button, Slider, Toggle, etc.)

### Core Types

- **`UiContext`**: Central coordinator for the UI system - holds input state, events, widget memory, and handles the frame lifecycle
- **`Component`**: Trait for building reusable widgets with automatic event handling
- **`Node`**: The fundamental building block of the UI tree
- **`InputState`**: Backend-agnostic input tracking (mouse, keyboard)
- **`EventDispatcher`**: Generates interaction events from input state

## Features

- **Backend Agnostic**: Core logic is independent of any graphics API
- **Immediate Mode API**: Build UI each frame with automatic state management
- **UiContext Pattern**: Clean API inspired by egui - context handles all internal complexity
- **Component System**: Build reusable widgets with `.on_click()`, `.on_hover()` callbacks
- **WGPU Backend**: High-performance GPU rendering via wgpu
- **Text Rendering**: Powered by cosmic-text for high-quality text shaping with aggressive caching
- **Modular Design**: Use only the crates you need
- **Transform Support**: Full translation and rotation support with proper transform composition
- **Per-child Placement (Stack)**: Override placement of individual children via `Place`
- **Performance Optimized**: Targeting 500+ FPS for typical UIs
  - Text shaping cache for reusable shaped text
  - Glyph metrics and atlas placement caching
  - Pre-allocated buffers to minimize allocations
  - Optimized rendering pipeline

## Quick Example

```rust
use astra_gui::{UiContext, Component, Node};
use astra_gui_interactive::Button;

// In your app's update loop:
fn build_ui(ctx: &mut UiContext) -> Node {
    Node::new()
        .with_child(
            Button::new("Click me!")
                .on_click(|| println!("Button clicked!"))
                .node(ctx)
        )
}
```

## Component API

The new `Component` trait enables clean, callback-based widgets:

```rust
// Simple button with click handler
Button::new("Submit")
    .disabled(is_loading)
    .on_click(|| save_data())
    .node(&mut ctx)

// Slider with drag value field
SliderWithValue::new(&mut value, 0.0..=100.0)
    .step(5.0)
    .speed(0.5)
    .on_change(|new_val| println!("Value: {}", new_val))
    .build(&mut ctx)
```

The `UiContext` manages:
- **Events**: Check interactions with `ctx.was_clicked("id")`, `ctx.is_hovered("id")`
- **Widget Memory**: Internal state (text buffers, cursors) stored automatically
- **ID Generation**: Unique IDs generated via `ctx.generate_id("label")`
- **Focus Management**: `ctx.is_focused("id")`, `ctx.set_focus(Some("id"))`

## Compatibility

| Astra GUI Version | wgpu Version | cosmic-text Version |
|-------------------|--------------|---------------------|
| 0.8.x             | 28.x         | 0.16.x              |
| 0.4.x             | 28.x         | 0.16.x              |
| 0.3.x             | 28.x         | 0.16.x              |
| 0.2.x             | 27.x         | 0.15.x              |

## Getting Started

Add astra-gui to your `Cargo.toml`:

```toml
[dependencies]
astra-gui = "0.8.0"
astra-gui-wgpu = "0.8.0"
astra-gui-interactive = "0.8.0"  # For Button, Slider, etc.
```

### Using with winit

```rust
use astra_gui::{InputState, UiContext};
use astra_gui_wgpu::WinitInputExt;

// In your event loop:
fn handle_event(input: &mut InputState, event: &WindowEvent) {
    input.handle_winit_event(event);  // Extension trait for winit conversion
}
```

## Examples

See the `crates/astra-gui-wgpu/examples/` directory for usage examples:

- `alignment.rs` - Text and layout alignment
- `collapsible.rs` - Collapsible sections
- `corner_shapes.rs` - Rounded corners and shapes
- `drag_value.rs` - Draggable value widget
- `interactive.rs` - Interactive components (buttons, toggles, sliders)
- `layout.rs` - Layout system
- `overflow.rs` - Overflow handling
- `place.rs` - Per-child placement overrides in `Layout::Stack`
- `rotation.rs` - Transform rotation with interactive controls
- `scroll.rs` - Scrollable containers with nested scrolling
- `slider_with_value.rs` - Slider with value display
- `stroke.rs` - Stroke rendering
- `text.rs` - Text rendering
- `translation.rs` - Transform translation with nested transforms
- `zoom.rs` - Browser-style zoom and pan

Run an example with optimized performance:

```bash
cargo run --release --example interactive
```

**Note**: Always use `--release` mode for accurate performance testing. Debug builds can be 3-5x slower.

## Migration from 0.7.x

The main changes in 0.8.x:

1. **Input/Events moved to core**: `InputState`, `EventDispatcher`, `InteractiveStateManager` are now in `astra-gui` (not `astra-gui-wgpu`)

2. **New `UiContext`**: Central context for UI operations
   ```rust
   let mut ctx = UiContext::new();
   ctx.begin_frame(&input);
   let root = build_ui(&mut ctx);
   ctx.end_frame(&mut root);
   ```

3. **Component trait changed**: Now takes `self` by value and requires `UiContext`
   ```rust
   // Old
   impl Component for MyWidget {
       fn node(&self) -> Node { ... }
   }
   
   // New
   impl Component for MyWidget {
       fn node(self, ctx: &mut UiContext) -> Node { ... }
   }
   ```

4. **Winit adapter**: Use `WinitInputExt` trait for winit event handling
   ```rust
   use astra_gui_wgpu::WinitInputExt;
   input.handle_winit_event(&event);  // Instead of input.handle_event(&event)
   ```

## License

MIT OR Apache-2.0