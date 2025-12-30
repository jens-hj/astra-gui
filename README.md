# Astra GUI

A graphics backend agnostic UI library for Rust.

## Overview

Astra GUI is a modular UI library that separates core UI logic from rendering backends. It provides the fundamental building blocks for creating user interfaces with full control over the rendering pipeline.

## Architecture

The library is split into several crates:

- **astra-gui**: Core UI primitives and logic with zero graphics API dependencies
- **astra-gui-fonts**: Bundled default fonts (Inter, JetBrains Mono)
- **astra-gui-text**: Backend-agnostic text shaping and glyph rasterization
- **astra-gui-wgpu**: WGPU rendering backend
- **astra-gui-interactive**: Interactive components library

## Features

- **Backend Agnostic**: Core logic is independent of any graphics API
- **WGPU Backend**: High-performance GPU rendering via wgpu
- **Text Rendering**: Powered by cosmic-text for high-quality text shaping with aggressive caching
- **Modular Design**: Use only the crates you need
- **Transform Support**: Full translation and rotation support with proper transform composition
- **Performance Optimized**: 
  - Text shaping cache for reusable shaped text
  - Glyph metrics and atlas placement caching
  - Pre-allocated buffers to minimize allocations
  - Optimized rendering pipeline achieving 60+ FPS even with complex UIs

## Compatibility

| Astra GUI Version | wgpu Version | cosmic-text Version |
|-------------------|--------------|---------------------|
| 0.3.x             | 28.x         | 0.16.x              |
| 0.2.x             | 27.x         | 0.15.x              |

## Getting Started

Add astra-gui to your `Cargo.toml`:

```toml
[dependencies]
astra-gui = "0.3.0"
astra-gui-wgpu = "0.3.0"
```

## Examples

See the `crates/astra-gui-wgpu/examples/` directory for usage examples:

- `alignment.rs` - Text and layout alignment
- `corner_shapes.rs` - Rounded corners and shapes
- `drag_value.rs` - Draggable value widget
- `interactive.rs` - Interactive components
- `layout.rs` - Layout system
- `overflow.rs` - Overflow handling
- `rotation.rs` - Transform rotation with interactive controls
- `scroll.rs` - Scrollable containers with nested scrolling
- `slider_with_value.rs` - Slider with value display
- `stroke.rs` - Stroke rendering
- `text.rs` - Text rendering
- `translation.rs` - Transform translation with nested transforms

Run an example with optimized performance:

```bash
cargo run --release --example scroll
```

**Note**: Always use `--release` mode for accurate performance testing. Debug builds can be 3-5x slower.

## License

MIT OR Apache-2.0
