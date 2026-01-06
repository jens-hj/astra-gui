//! Event dispatching system for interactive UI components
//!
//! This module re-exports the event types from astra-gui core and provides
//! any wgpu-specific event handling utilities.

// Re-export all event types from astra-gui core
pub use astra_gui::{
    EventDispatcher, InteractionEvent, InteractionState, InteractiveStateManager, TargetedEvent,
};
