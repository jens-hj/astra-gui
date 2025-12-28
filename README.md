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
- **Text Rendering**: Powered by cosmic-text for high-quality text shaping
- **Modular Design**: Use only the crates you need

## Getting Started

Add astra-gui to your `Cargo.toml`:

```toml
[dependencies]
astra-gui = "0.1.0"
astra-gui-wgpu = "0.1.0"
```

## Examples

See the `crates/astra-gui-wgpu/examples/` directory for usage examples:

- `text.rs` - Text rendering
- `layout.rs` - Layout system
- `interactive.rs` - Interactive components
- `stroke.rs` - Stroke rendering
- `corner_shapes.rs` - Rounded corners and shapes
- `overflow.rs` - Overflow handling

Run an example:

```bash
cargo run --example text
```

## License

MIT OR Apache-2.0
