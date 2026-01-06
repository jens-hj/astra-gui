//! Event dispatching system for interactive UI components
//!
//! This module provides types and functions for generating interaction events
//! from input state and hit-testing results. It is backend-agnostic and does
//! not depend on any specific windowing library.

use crate::{hit_test_point, InputState, MouseButton, Node, NodeId, Overflow, Point};
use std::collections::HashMap;

/// Interaction state of a node (for style transitions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionState {
    /// Node is idle (not being interacted with)
    Idle,
    /// Mouse is hovering over the node
    Hovered,
    /// Node is being actively interacted with (pressed/dragged)
    Active,
    /// Node is disabled and cannot be interacted with
    Disabled,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Type of interaction event
#[derive(Debug, Clone)]
pub enum InteractionEvent {
    /// Mouse click event (button pressed and released on same target)
    Click {
        /// Which mouse button was clicked
        button: MouseButton,
        /// Position of the click in window coordinates
        position: Point,
    },
    /// Mouse hover event (cursor over node)
    Hover {
        /// Current cursor position
        position: Point,
    },
    /// Drag start event (button pressed and moved)
    DragStart {
        /// Which mouse button started the drag
        button: MouseButton,
        /// Position where drag started
        position: Point,
    },
    /// Drag move event (while dragging)
    DragMove {
        /// Current cursor position
        position: Point,
        /// Movement delta since last frame
        delta: Point,
    },
    /// Drag end event (button released while dragging)
    DragEnd {
        /// Which mouse button ended the drag
        button: MouseButton,
        /// Position where drag ended
        position: Point,
    },
    /// Node gained focus
    Focus,
    /// Node lost focus
    Blur,
    /// Mouse wheel scroll event
    Scroll {
        /// Scroll delta (horizontal, vertical)
        delta: (f32, f32),
        /// Position of the scroll
        position: Point,
    },
}

/// An interaction event targeted at a specific node
#[derive(Debug, Clone)]
pub struct TargetedEvent {
    /// The interaction event
    pub event: InteractionEvent,
    /// The ID of the target node
    pub target: NodeId,
    /// Position relative to the target node's top-left corner
    pub local_position: Point,
    /// The accumulated zoom/scale factor at this node (from root to node)
    /// This is 1.0 for no zoom, 2.0 for 2x zoom, etc.
    pub zoom: f32,
}

/// State tracking for drag operations
#[derive(Debug, Clone)]
struct DragState {
    /// Which button is being used for the drag
    button: MouseButton,
    /// The node being dragged
    target: NodeId,
    /// Last known position during drag
    last_pos: Point,
    /// The origin (top-left) of the target node in screen coordinates
    node_origin: Point,
    /// The zoom factor at the target node
    zoom: f32,
}

/// Cursor blink state tracker
#[derive(Debug, Clone)]
struct CursorBlinkState {
    /// When the cursor last blinked
    last_blink: std::time::Instant,
    /// Whether the cursor is currently visible
    visible: bool,
}

/// Event dispatcher that generates interaction events from input state
///
/// This maintains state across frames to detect interactions like clicks,
/// hovers, and drags.
pub struct EventDispatcher {
    /// Currently hovered node IDs
    hovered_nodes: Vec<NodeId>,
    /// Current drag state, if dragging
    drag_state: Option<DragState>,
    /// Currently focused node ID, if any
    focused_node: Option<NodeId>,
    /// Cursor blink states for focused text inputs (node_id -> blink_state)
    cursor_blink_states: HashMap<NodeId, CursorBlinkState>,
    /// Persistent scroll state (node_id -> (scroll_offset, scroll_target))
    scroll_state: HashMap<String, ((f32, f32), (f32, f32))>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            hovered_nodes: Vec::new(),
            drag_state: None,
            focused_node: None,
            cursor_blink_states: HashMap::new(),
            scroll_state: HashMap::new(),
        }
    }

    /// Get the currently focused node ID, if any
    pub fn focused_node(&self) -> Option<&NodeId> {
        self.focused_node.as_ref()
    }

    /// Set the focused node
    ///
    /// This will generate Blur events for the previously focused node
    /// and Focus events for the newly focused node on the next dispatch.
    pub fn set_focus(&mut self, node_id: Option<NodeId>) {
        // If there was a previously focused node that's different, clean up its cursor state
        if let Some(ref old_id) = self.focused_node {
            if node_id.as_ref() != Some(old_id) {
                self.cursor_blink_states.remove(old_id);
            }
        }

        // If there's a new focused node, initialize its cursor blink state
        if let Some(ref new_id) = node_id {
            if self.focused_node.as_ref() != Some(new_id) {
                self.cursor_blink_states.insert(
                    new_id.clone(),
                    CursorBlinkState {
                        last_blink: std::time::Instant::now(),
                        visible: true,
                    },
                );
            }
        }

        self.focused_node = node_id;
    }

    /// Update cursor blink state for a focused node
    ///
    /// Call this each frame to update the cursor visibility.
    /// Returns the current visibility state.
    pub fn update_cursor_blink(&mut self, node_id: &NodeId, blink_rate_ms: u64) -> bool {
        if let Some(state) = self.cursor_blink_states.get_mut(node_id) {
            let elapsed = state.last_blink.elapsed().as_millis() as u64;
            if elapsed >= blink_rate_ms {
                state.visible = !state.visible;
                state.last_blink = std::time::Instant::now();
            }
            state.visible
        } else {
            // Not a focused text input, default to visible
            true
        }
    }

    /// Reset cursor blink to visible (call when text changes)
    pub fn reset_cursor_blink(&mut self, node_id: &NodeId) {
        if let Some(state) = self.cursor_blink_states.get_mut(node_id) {
            state.visible = true;
            state.last_blink = std::time::Instant::now();
        }
    }

    /// Check if the cursor should be visible for a node
    pub fn is_cursor_visible(&self, node_id: &NodeId) -> bool {
        self.cursor_blink_states
            .get(node_id)
            .map(|s| s.visible)
            .unwrap_or(true)
    }

    /// Dispatch events based on input state and UI tree
    ///
    /// This is the main method that should be called each frame after
    /// processing input. It will generate interaction events based on
    /// the current input state and the UI tree structure.
    ///
    /// # Arguments
    /// * `input` - Current input state
    /// * `root` - Root node of the UI tree (must have computed layout)
    ///
    /// # Returns
    /// A tuple of:
    /// - Vec of targeted events for this frame
    /// - HashMap of node interaction states for style transitions
    pub fn dispatch(
        &mut self,
        input: &InputState,
        root: &mut Node,
    ) -> (Vec<TargetedEvent>, HashMap<NodeId, InteractionState>) {
        let mut events = Vec::new();
        let mut interaction_states = HashMap::new();

        // Get current cursor position
        let cursor_pos = match input.cursor_position {
            Some(pos) => pos,
            None => {
                // Cursor left window - clear hover states
                self.hovered_nodes.clear();
                return (events, interaction_states);
            }
        };

        // Hit test to find nodes under cursor
        let hits = hit_test_point(root, cursor_pos);

        // Build list of currently hovered node IDs
        let mut current_hovered: Vec<NodeId> = Vec::new();
        for hit in &hits {
            if let Some(id) = hit.node_id.clone() {
                current_hovered.push(id);
            }
        }

        // Check for focus changes (click to focus)
        if input.is_button_just_pressed(MouseButton::Left) {
            // Find the topmost focusable node (must have an ID)
            let new_focus = hits
                .iter()
                .find(|h| h.node_id.is_some())
                .and_then(|hit| hit.node_id.clone());

            // Generate blur event for previously focused node
            if let Some(ref old_focus) = self.focused_node {
                if new_focus.as_ref() != Some(old_focus) {
                    events.push(TargetedEvent {
                        event: InteractionEvent::Blur,
                        target: old_focus.clone(),
                        local_position: Point::zero(),
                        zoom: 1.0,
                    });
                }
            }

            // Generate focus event for newly focused node
            if let Some(ref new_focus_id) = new_focus {
                if self.focused_node.as_ref() != Some(new_focus_id) {
                    events.push(TargetedEvent {
                        event: InteractionEvent::Focus,
                        target: new_focus_id.clone(),
                        local_position: Point::zero(),
                        zoom: 1.0,
                    });
                }
            }

            self.set_focus(new_focus);
        }

        // Handle drag state
        if let Some(ref mut drag) = self.drag_state {
            // Check if drag button was released
            if input.is_button_just_released(drag.button) {
                // Calculate local position within the target node
                let local_position = Point {
                    x: cursor_pos.x - drag.node_origin.x,
                    y: cursor_pos.y - drag.node_origin.y,
                };

                // Generate DragEnd event
                events.push(TargetedEvent {
                    event: InteractionEvent::DragEnd {
                        button: drag.button,
                        position: cursor_pos,
                    },
                    target: drag.target.clone(),
                    local_position,
                    zoom: drag.zoom,
                });

                // Mark the drag target as active in interaction states
                interaction_states.insert(drag.target.clone(), InteractionState::Active);
            } else {
                // Generate DragMove event
                let delta = Point {
                    x: cursor_pos.x - drag.last_pos.x,
                    y: cursor_pos.y - drag.last_pos.y,
                };

                if delta.x.abs() > 0.001 || delta.y.abs() > 0.001 {
                    // Calculate local position within the target node
                    let local_position = Point {
                        x: cursor_pos.x - drag.node_origin.x,
                        y: cursor_pos.y - drag.node_origin.y,
                    };

                    events.push(TargetedEvent {
                        event: InteractionEvent::DragMove {
                            position: cursor_pos,
                            delta,
                        },
                        target: drag.target.clone(),
                        local_position,
                        zoom: drag.zoom,
                    });
                }

                drag.last_pos = cursor_pos;

                // Mark drag target as active
                interaction_states.insert(drag.target.clone(), InteractionState::Active);
            }
        }

        // Clear drag state if button released
        if self
            .drag_state
            .as_ref()
            .map(|d| input.is_button_just_released(d.button))
            .unwrap_or(false)
        {
            self.drag_state = None;
        }

        // Check for new drag start
        if self.drag_state.is_none() {
            for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
                if input.is_button_down(button) && !input.is_button_just_pressed(button) {
                    // Button held from previous frame - check if we should start a drag
                    // Find first hit with a node_id (skip nodes without IDs)
                    if let Some(hit) = hits.iter().rfind(|h| h.node_id.is_some()) {
                        if let Some(ref node_id) = hit.node_id {
                            // Calculate node origin from cursor position and local position
                            let node_origin = Point {
                                x: cursor_pos.x - hit.local_pos.x,
                                y: cursor_pos.y - hit.local_pos.y,
                            };

                            // Start drag
                            self.drag_state = Some(DragState {
                                button,
                                target: node_id.clone(),
                                last_pos: cursor_pos,
                                node_origin,
                                zoom: hit.zoom,
                            });

                            events.push(TargetedEvent {
                                event: InteractionEvent::DragStart {
                                    button,
                                    position: cursor_pos,
                                },
                                target: node_id.clone(),
                                local_position: hit.local_pos,
                                zoom: hit.zoom,
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Generate click events (button just released without dragging)
        if self.drag_state.is_none() {
            for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
                if input.is_button_just_released(button) {
                    // Find the first hit that has a node_id (skip nodes without IDs)
                    if let Some(hit) = hits.iter().rfind(|h| h.node_id.is_some()) {
                        if let Some(ref node_id) = hit.node_id {
                            events.push(TargetedEvent {
                                event: InteractionEvent::Click {
                                    button,
                                    position: cursor_pos,
                                },
                                target: node_id.clone(),
                                local_position: hit.local_pos,
                                zoom: hit.zoom,
                            });
                        }
                    }
                }
            }
        }

        // Generate hover events
        for hit in &hits {
            if let Some(ref node_id) = hit.node_id {
                events.push(TargetedEvent {
                    event: InteractionEvent::Hover {
                        position: cursor_pos,
                    },
                    target: node_id.clone(),
                    local_position: hit.local_pos,
                    zoom: hit.zoom,
                });

                // Mark as hovered (unless being dragged)
                if self.drag_state.is_none()
                    || self.drag_state.as_ref().map(|d| &d.target) != Some(node_id)
                {
                    interaction_states
                        .entry(node_id.clone())
                        .or_insert(InteractionState::Hovered);
                }
            }
        }

        // Handle scroll events
        if input.scroll_delta.0.abs() > 0.001 || input.scroll_delta.1.abs() > 0.001 {
            self.process_scroll_event(root, cursor_pos, input.scroll_delta, &mut events);
        }

        // Update hovered nodes list
        self.hovered_nodes = current_hovered;

        (events, interaction_states)
    }

    /// Restore scroll state to nodes after UI rebuild
    pub fn restore_scroll_state(&self, root: &mut Node) {
        self.restore_scroll_state_recursive(root);
    }

    fn restore_scroll_state_recursive(&self, node: &mut Node) {
        // Check if this node has saved scroll state
        if let Some(id) = node.id() {
            if let Some(&(offset, target)) = self.scroll_state.get(id.as_str()) {
                node.set_scroll_offset(offset);
                node.set_scroll_target(target);
            }
        }

        // Recursively restore for children
        for child in node.children_mut() {
            self.restore_scroll_state_recursive(child);
        }
    }

    /// Sync scroll state from nodes to internal storage
    pub fn sync_scroll_state(&mut self, root: &Node) {
        self.sync_scroll_state_recursive(root);
    }

    fn sync_scroll_state_recursive(&mut self, node: &Node) {
        // Save scroll state if node has an ID and non-zero scroll
        if let Some(id) = node.id() {
            let offset = node.scroll_offset();
            let target = node.scroll_target();

            if offset != (0.0, 0.0) || target != (0.0, 0.0) {
                self.scroll_state
                    .insert(id.as_str().to_string(), (offset, target));
            }
        }

        // Recursively sync for children
        for child in node.children() {
            self.sync_scroll_state_recursive(child);
        }
    }

    fn process_scroll_event(
        &mut self,
        root: &mut Node,
        position: Point,
        delta: (f32, f32),
        events: &mut Vec<TargetedEvent>,
    ) {
        // Find scrollable nodes under cursor
        let hits = hit_test_point(root, position);

        // Find the first scrollable node in the hit chain
        for hit in &hits {
            if let Some(ref node_id) = hit.node_id {
                // Find the node and check if it's scrollable
                if let Some(node) = self.find_node_by_id_mut(root, node_id) {
                    // A node is scrollable if it has Scroll overflow
                    let is_scrollable = node.overflow() == Overflow::Scroll;

                    if is_scrollable {
                        // Apply scroll to this node
                        node.scroll_by((-delta.0, -delta.1));

                        // Generate scroll event
                        events.push(TargetedEvent {
                            event: InteractionEvent::Scroll { delta, position },
                            target: node_id.clone(),
                            local_position: hit.local_pos,
                            zoom: hit.zoom,
                        });

                        // Save scroll state
                        self.scroll_state.insert(
                            node_id.as_str().to_string(),
                            (node.scroll_offset(), node.scroll_target()),
                        );

                        // Only scroll the first scrollable ancestor
                        break;
                    }
                }
            }
        }
    }

    fn find_node_by_id_mut<'a>(
        &self,
        node: &'a mut Node,
        target_id: &NodeId,
    ) -> Option<&'a mut Node> {
        if node.id() == Some(target_id) {
            return Some(node);
        }

        for child in node.children_mut() {
            if let Some(found) = self.find_node_by_id_mut(child, target_id) {
                return Some(found);
            }
        }

        None
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_dispatcher_creation() {
        let dispatcher = EventDispatcher::new();
        assert!(dispatcher.focused_node().is_none());
    }

    #[test]
    fn test_focus_management() {
        let mut dispatcher = EventDispatcher::new();

        let node_id = NodeId::new("test_node");
        dispatcher.set_focus(Some(node_id.clone()));

        assert_eq!(dispatcher.focused_node(), Some(&node_id));

        dispatcher.set_focus(None);
        assert!(dispatcher.focused_node().is_none());
    }

    #[test]
    fn test_cursor_blink() {
        let mut dispatcher = EventDispatcher::new();

        let node_id = NodeId::new("text_input");
        dispatcher.set_focus(Some(node_id.clone()));

        // Initially visible
        assert!(dispatcher.is_cursor_visible(&node_id));

        // Reset should keep visible
        dispatcher.reset_cursor_blink(&node_id);
        assert!(dispatcher.is_cursor_visible(&node_id));
    }
}
