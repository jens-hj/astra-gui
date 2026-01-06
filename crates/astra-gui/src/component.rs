//! Component trait for building reusable UI widgets
//!
//! Components are the building blocks of the UI. They encapsulate the logic
//! for creating nodes with specific behavior and styling.
//!
//! # Example
//!
//! ```ignore
//! use astra_gui::{Component, Node, UiContext};
//!
//! struct MyButton {
//!     label: String,
//!     on_click: Option<Box<dyn FnMut()>>,
//! }
//!
//! impl MyButton {
//!     pub fn new(label: impl Into<String>) -> Self {
//!         Self {
//!             label: label.into(),
//!             on_click: None,
//!         }
//!     }
//!
//!     pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
//!         self.on_click = Some(Box::new(f));
//!         self
//!     }
//! }
//!
//! impl Component for MyButton {
//!     fn node(mut self, ctx: &mut UiContext) -> Node {
//!         let id = ctx.generate_id("button");
//!
//!         // Check for click events from last frame
//!         if ctx.was_clicked(&id) {
//!             if let Some(ref mut on_click) = self.on_click {
//!                 on_click();
//!             }
//!         }
//!
//!         // Build and return the node
//!         Node::new()
//!             .with_id(NodeId::new(id))
//!             // ... styling ...
//!     }
//! }
//! ```

use crate::{Node, UiContext};

/// A component that can be rendered as a UI node
///
/// Components implement this trait to define how they render themselves
/// into the node tree. The `node` method receives the UI context which
/// provides access to:
/// - Events from the last frame (for handling interactions)
/// - Widget memory (for storing internal state)
/// - Content measurer (for text measurement)
/// - ID generation (for unique widget identification)
///
/// # Note on `self` consumption
///
/// The trait takes `self` by value (consuming it) because:
/// 1. Components are typically created inline: `Button::new("Click").node(&mut ctx)`
/// 2. It allows moving owned data like callbacks into the component
/// 3. The component's job is done after producing a `Node`
///
/// If you need to keep a reference to component data, store it externally
/// and reference it when creating the component.
pub trait Component {
    /// Build the node tree for this component
    ///
    /// This method is called during UI building to produce the actual `Node`
    /// that will be laid out and rendered.
    ///
    /// # Arguments
    /// * `ctx` - The UI context providing access to events, state, and utilities
    ///
    /// # Returns
    /// The root `Node` of this component's subtree
    fn node(self, ctx: &mut UiContext) -> Node;
}

/// Extension trait for optional components
///
/// This allows conditionally including a component in the UI.
pub trait ComponentExt: Component + Sized {
    /// Conditionally render this component
    ///
    /// If `condition` is true, renders the component normally.
    /// If false, returns an empty node.
    fn when(self, condition: bool, ctx: &mut UiContext) -> Node {
        if condition {
            self.node(ctx)
        } else {
            Node::new()
        }
    }
}

// Implement ComponentExt for all Components
impl<T: Component> ComponentExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        label: String,
    }

    impl TestComponent {
        fn new(label: impl Into<String>) -> Self {
            Self {
                label: label.into(),
            }
        }
    }

    impl Component for TestComponent {
        fn node(self, ctx: &mut UiContext) -> Node {
            let _id = ctx.generate_id(&self.label);
            Node::new()
        }
    }

    #[test]
    fn test_component_node() {
        let mut ctx = UiContext::new();
        let component = TestComponent::new("test");
        let _node = component.node(&mut ctx);
    }

    #[test]
    fn test_component_when() {
        let mut ctx = UiContext::new();

        let component = TestComponent::new("visible");
        let _node = component.when(true, &mut ctx);

        let component = TestComponent::new("hidden");
        let _node = component.when(false, &mut ctx);
    }
}
