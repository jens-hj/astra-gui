//! Shared modules for `astra-gui-wgpu` examples.
//!
//! Keep example utilities (like debug keybinds/help text) here so behavior is
//! consistent across all examples.

pub mod debug_controls;
pub mod example_app;
pub mod gpu_state;
pub mod interactive;
pub mod runner;

// Re-export commonly used items
pub use example_app::ExampleApp;
pub use interactive::InteractiveState;
pub use runner::run_example;
