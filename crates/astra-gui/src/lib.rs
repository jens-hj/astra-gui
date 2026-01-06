//! # astra-gui
//!
//! Graphics backend agnostic UI library.
//!
//! This crate provides the core UI primitives and logic with zero dependencies
//! on any specific graphics API. Rendering is handled by separate backend crates
//! like `astra-gui-wgpu`.
//!
//! ## Core Types
//!
//! - [`Node`] - The fundamental building block of the UI tree
//! - [`UiContext`] - Central coordinator for the UI system
//! - [`Component`] - Trait for reusable UI widgets
//!
//! ## Input & Events
//!
//! - [`InputState`] - Tracks mouse, keyboard, and other input
//! - [`EventDispatcher`] - Generates interaction events from input
//! - [`InteractionEvent`] - Types of UI interactions (click, hover, drag, etc.)
//! - [`TargetedEvent`] - An event targeted at a specific node
//!
//! ## State Management
//!
//! - [`InteractiveStateManager`] - Manages style transitions for nodes
//! - [`WidgetMemory`] - Stores internal widget state (text buffers, etc.)
//!
//! ## Layout & Styling
//!
//! - [`Style`] - Visual styling properties
//! - [`Transition`] - Animation configuration for style changes
//! - [`ContentMeasurer`] - Trait for text measurement

mod color;
mod component;
mod content;
mod context;
mod debug;
mod events;
mod hit_test;
mod input;
mod interactive_state;
mod layout;
mod measure;
mod memory;
mod node;
mod output;
mod primitives;
mod style;
pub mod transition;

// Core types
pub use color::*;
pub use component::*;
pub use content::*;
pub use context::*;
pub use debug::*;
pub use hit_test::*;
pub use layout::*;
pub use measure::*;
pub use node::*;
pub use output::*;
pub use primitives::*;
pub use style::*;
pub use transition::*;

// Input & Events
pub use events::*;
pub use input::*;

// State Management
pub use interactive_state::*;
pub use memory::*;
