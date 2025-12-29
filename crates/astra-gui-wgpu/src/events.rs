//! Event dispatching system for interactive UI components
//!
//! This module provides types and functions for generating interaction events
//! from input state and hit-testing results.

use astra_gui::{hit_test_point, Node, NodeId, Point};
use std::collections::HashMap;
use winit::event::MouseButton;

use crate::input::InputState;
use crate::interactive_state::InteractionState;

/// Type of interaction event
#[derive(Debug, Clone)]
pub enum InteractionEvent {
    /// Mouse click event
    Click {
        button: MouseButton,
        position: Point,
    },
    /// Mouse hover event (cursor over node)
    Hover { position: Point },
    /// Drag start event
    DragStart {
        button: MouseButton,
        position: Point,
    },
    /// Drag move event (while dragging)
    DragMove { position: Point, delta: Point },
    /// Drag end event (button released while dragging)
    DragEnd {
        button: MouseButton,
        position: Point,
    },
    /// Node gained focus
    Focus,
    /// Node lost focus
    Blur,
    /// Mouse wheel scroll event
    Scroll {
        delta: (f32, f32), // (horizontal, vertical) scroll delta
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
}

/// State tracking for drag operations
#[derive(Debug, Clone)]
struct DragState {
    button: MouseButton,
    target: NodeId,
    start_pos: Point,
    last_pos: Point,
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
}

/// Cursor blink state tracker (internal to EventDispatcher)
#[derive(Debug, Clone)]
struct CursorBlinkState {
    last_blink: std::time::Instant,
    visible: bool,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            hovered_nodes: Vec::new(),
            drag_state: None,
            focused_node: None,
            cursor_blink_states: HashMap::new(),
        }
    }

    /// Get the currently focused node ID
    pub fn focused_node(&self) -> Option<&NodeId> {
        self.focused_node.as_ref()
    }

    /// Set focus to a specific node, generating Focus/Blur events as needed
    ///
    /// Returns events for the focus change (Blur for old focus, Focus for new focus)
    pub fn set_focus(&mut self, node_id: Option<NodeId>) -> Vec<TargetedEvent> {
        let mut events = Vec::new();

        // Generate Blur event for previously focused node
        if let Some(old_focus) = &self.focused_node {
            if Some(old_focus) != node_id.as_ref() {
                events.push(TargetedEvent {
                    event: InteractionEvent::Blur,
                    target: old_focus.clone(),
                    local_position: Point { x: 0.0, y: 0.0 },
                });
            }
        }

        // Generate Focus event for newly focused node
        if let Some(new_focus) = &node_id {
            if Some(new_focus) != self.focused_node.as_ref() {
                events.push(TargetedEvent {
                    event: InteractionEvent::Focus,
                    target: new_focus.clone(),
                    local_position: Point { x: 0.0, y: 0.0 },
                });
            }
        }

        self.focused_node = node_id;
        events
    }

    /// Update cursor blink state for a specific node and return whether cursor should be visible
    ///
    /// Call this each frame for text inputs that need a blinking cursor.
    /// The blink state is automatically managed per node ID.
    pub fn update_cursor_blink(
        &mut self,
        node_id: &NodeId,
        blink_interval: std::time::Duration,
    ) -> bool {
        let state = self
            .cursor_blink_states
            .entry(node_id.clone())
            .or_insert(CursorBlinkState {
                last_blink: std::time::Instant::now(),
                visible: true,
            });

        let now = std::time::Instant::now();
        if now.duration_since(state.last_blink) >= blink_interval {
            state.visible = !state.visible;
            state.last_blink = now;
        }
        state.visible
    }

    /// Reset cursor blink state for a node (makes cursor visible and restarts timer)
    ///
    /// Call this when the user types or moves the cursor to ensure visibility.
    pub fn reset_cursor_blink(&mut self, node_id: &NodeId) {
        if let Some(state) = self.cursor_blink_states.get_mut(node_id) {
            state.visible = true;
            state.last_blink = std::time::Instant::now();
        }
    }

    /// Get current cursor visibility for a node without updating
    pub fn is_cursor_visible(&self, node_id: &NodeId) -> bool {
        self.cursor_blink_states
            .get(node_id)
            .map(|state| state.visible)
            .unwrap_or(true) // Default to visible if no state exists
    }

    /// Process input state and generate interaction events
    ///
    /// This performs hit-testing against the UI tree and generates events
    /// based on what the input state contains. Auto-IDs are automatically assigned
    /// to nodes with interactive styles to enable hover/active states without
    /// requiring manual IDs.
    ///
    /// # Arguments
    /// * `input` - Current input state (mouse position, buttons)
    /// * `root` - Root node of the UI tree (with computed layout)
    ///
    /// # Returns
    /// Tuple of (events, interaction_states) where:
    /// - events: Vector of targeted events for this frame
    /// - interaction_states: Map of node IDs to their current interaction state (Idle/Hovered/Active)
    pub fn dispatch(
        &mut self,
        input: &InputState,
        root: &mut Node,
    ) -> (Vec<TargetedEvent>, HashMap<NodeId, InteractionState>) {
        // Automatically assign IDs to nodes with interactive styles
        // This allows hover/active/disabled styles to work without manual IDs
        crate::interactive_state::InteractiveStateManager::assign_auto_ids(root);
        let mut events = Vec::new();
        let mut interaction_states = HashMap::new();

        // Get cursor position, if available
        let Some(cursor_pos) = input.cursor_position else {
            // No cursor position - clear hover state and handle drag end if needed
            self.hovered_nodes.clear();
            if let Some(drag) = self.drag_state.take() {
                // Drag ended because cursor left window
                events.push(TargetedEvent {
                    event: InteractionEvent::DragEnd {
                        button: drag.button,
                        position: drag.last_pos,
                    },
                    target: drag.target,
                    local_position: Point {
                        x: drag.last_pos.x - drag.start_pos.x,
                        y: drag.last_pos.y - drag.start_pos.y,
                    },
                });
            }
            return (events, interaction_states);
        };

        // Hit-test to find all nodes under cursor (shallow to deep)
        let hits = hit_test_point(root, cursor_pos);

        // Get deepest node with an ID (most specific target)
        let deepest_target = hits
            .iter()
            .rev()
            .find(|hit| hit.node_id.is_some())
            .and_then(|hit| {
                hit.node_id
                    .as_ref()
                    .map(|id| (id.clone(), hit.local_pos, hit.node_rect))
            });

        // Update hover state and generate hover events
        let new_hovered: Vec<NodeId> = hits.iter().filter_map(|hit| hit.node_id.clone()).collect();

        // Only generate hover event for the deepest target
        if let Some((target_id, local_pos, _)) = &deepest_target {
            if !self.hovered_nodes.contains(target_id) {
                events.push(TargetedEvent {
                    event: InteractionEvent::Hover {
                        position: cursor_pos,
                    },
                    target: target_id.clone(),
                    local_position: *local_pos,
                });
            }
        }

        self.hovered_nodes = new_hovered;

        // Populate interaction states for all nodes with IDs
        // This determines whether each node should be rendered as Idle, Hovered, or Active
        for hit in &hits {
            if let Some(node_id) = &hit.node_id {
                let is_pressed = input.is_button_down(MouseButton::Left);

                let state = if is_pressed {
                    InteractionState::Active
                } else {
                    InteractionState::Hovered
                };

                interaction_states.insert(node_id.clone(), state);
            }
        }

        // Handle drag state
        if let Some(drag) = &mut self.drag_state {
            // Currently dragging
            if input.is_button_down(drag.button) {
                // Still dragging - generate DragMove event
                let delta = Point {
                    x: cursor_pos.x - drag.last_pos.x,
                    y: cursor_pos.y - drag.last_pos.y,
                };

                // Hit-test to get local position relative to the drag target
                let local_pos = hits
                    .iter()
                    .rev()
                    .find(|hit| hit.node_id.as_ref() == Some(&drag.target))
                    .map(|hit| hit.local_pos)
                    .unwrap_or(Point { x: 0.0, y: 0.0 });

                events.push(TargetedEvent {
                    event: InteractionEvent::DragMove {
                        position: cursor_pos,
                        delta,
                    },
                    target: drag.target.clone(),
                    local_position: local_pos,
                });

                drag.last_pos = cursor_pos;
            } else {
                // Button released - end drag
                let completed_drag = self.drag_state.take().unwrap();
                events.push(TargetedEvent {
                    event: InteractionEvent::DragEnd {
                        button: completed_drag.button,
                        position: cursor_pos,
                    },
                    target: completed_drag.target,
                    local_position: Point {
                        x: cursor_pos.x - completed_drag.start_pos.x,
                        y: cursor_pos.y - completed_drag.start_pos.y,
                    },
                });
            }
        }

        // Check for new clicks (only if not currently dragging)
        if self.drag_state.is_none() {
            if let Some((target_id, local_pos, _)) = &deepest_target {
                // Check for left-click
                if input.is_button_just_pressed(MouseButton::Left) {
                    events.push(TargetedEvent {
                        event: InteractionEvent::Click {
                            button: MouseButton::Left,
                            position: cursor_pos,
                        },
                        target: target_id.clone(),
                        local_position: *local_pos,
                    });

                    // Start drag state for potential drag
                    self.drag_state = Some(DragState {
                        button: MouseButton::Left,
                        target: target_id.clone(),
                        start_pos: cursor_pos,
                        last_pos: cursor_pos,
                    });

                    // Also generate DragStart event
                    events.push(TargetedEvent {
                        event: InteractionEvent::DragStart {
                            button: MouseButton::Left,
                            position: cursor_pos,
                        },
                        target: target_id.clone(),
                        local_position: *local_pos,
                    });
                }

                // Check for right-click (no drag for right-click in this implementation)
                if input.is_button_just_pressed(MouseButton::Right) {
                    events.push(TargetedEvent {
                        event: InteractionEvent::Click {
                            button: MouseButton::Right,
                            position: cursor_pos,
                        },
                        target: target_id.clone(),
                        local_position: *local_pos,
                    });
                }
            }
        }

        // Handle scroll events - apply to deepest scrollable container
        if input.scroll_delta != (0.0, 0.0) {
            if let Some((target_id, local_pos, _)) = &deepest_target {
                events.push(TargetedEvent {
                    event: InteractionEvent::Scroll {
                        delta: input.scroll_delta,
                        position: cursor_pos,
                    },
                    target: target_id.clone(),
                    local_position: *local_pos,
                });

                // Automatically process scroll events for nodes with Overflow::Scroll
                Self::process_scroll_event(root, target_id, input.scroll_delta, input.shift_held);
            }
        }

        (events, interaction_states)
    }

    /// Automatically process scroll events for scrollable containers
    fn process_scroll_event(
        root: &mut Node,
        target_id: &NodeId,
        delta: (f32, f32),
        shift_held: bool,
    ) {
        // Find the target node
        if let Some(node) = Self::find_node_by_id_mut(root, target_id.as_str()) {
            // Only process if node has Overflow::Scroll
            if node.overflow() != astra_gui::Overflow::Scroll {
                return;
            }

            let scroll_speed = node.scroll_speed();
            let scroll_direction = node.scroll_direction();
            let layout_direction = node.layout_direction();

            let current_target = node.scroll_target();

            // Apply scroll speed and direction
            let direction_multiplier = match scroll_direction {
                astra_gui::ScrollDirection::Normal => 1.0,
                astra_gui::ScrollDirection::Inverted => -1.0,
            };

            let adjusted_delta = (
                delta.0 * scroll_speed * direction_multiplier,
                delta.1 * scroll_speed * direction_multiplier,
            );

            // Calculate max scroll based on content size
            let max_scroll = Self::calculate_max_scroll(node);

            // Determine scroll behavior based on layout and shift key
            let new_target = match layout_direction {
                astra_gui::Layout::Horizontal => {
                    // Horizontal layout: vertical scroll delta -> horizontal scroll
                    (
                        (current_target.0 + adjusted_delta.1).clamp(0.0, max_scroll.0),
                        current_target.1,
                    )
                }
                astra_gui::Layout::Vertical => {
                    // Vertical layout: check if shift is held for horizontal scrolling
                    if shift_held && max_scroll.0 > 0.0 {
                        // Shift+scroll for horizontal scrolling
                        (
                            (current_target.0 + adjusted_delta.1).clamp(0.0, max_scroll.0),
                            current_target.1,
                        )
                    } else {
                        // Normal vertical scrolling
                        (
                            current_target.0,
                            (current_target.1 + adjusted_delta.1).clamp(0.0, max_scroll.1),
                        )
                    }
                }
                astra_gui::Layout::Stack => {
                    // Stack layout: both directions scrollable
                    (
                        (current_target.0 + adjusted_delta.0).clamp(0.0, max_scroll.0),
                        (current_target.1 + adjusted_delta.1).clamp(0.0, max_scroll.1),
                    )
                }
            };

            node.set_scroll_target(new_target);
        }
    }

    /// Find a node by ID (mutable)
    fn find_node_by_id_mut<'a>(node: &'a mut Node, id: &str) -> Option<&'a mut Node> {
        if node.id().map(|node_id| node_id.as_str()) == Some(id) {
            return Some(node);
        }

        for child in node.children_mut() {
            if let Some(found) = Self::find_node_by_id_mut(child, id) {
                return Some(found);
            }
        }

        None
    }

    /// Calculate maximum scroll offset for a container
    fn calculate_max_scroll(container: &Node) -> (f32, f32) {
        let Some(layout) = container.computed_layout() else {
            return (0.0, 0.0);
        };

        // Get container dimensions (after padding)
        let padding = container.padding();
        let container_width =
            layout.rect.max[0] - layout.rect.min[0] - padding.left - padding.right;
        let container_height =
            layout.rect.max[1] - layout.rect.min[1] - padding.top - padding.bottom;

        // Calculate total content size based on layout direction
        let gap = container.gap();
        let children = container.children();
        let layout_direction = container.layout_direction();

        if children.is_empty() {
            return (0.0, 0.0);
        }

        let mut content_width = 0.0f32;
        let mut content_height = 0.0f32;

        match layout_direction {
            astra_gui::Layout::Vertical => {
                // For vertical layout: accumulate heights, track max width
                // For nested layouts (like grid), we need to look at the intrinsic width
                for (i, child) in children.iter().enumerate() {
                    if let Some(child_layout) = child.computed_layout() {
                        let child_width = child_layout.rect.max[0] - child_layout.rect.min[0];
                        let child_height = child_layout.rect.max[1] - child_layout.rect.min[1];

                        // For horizontal child layouts, calculate their full content width
                        let actual_child_width =
                            if child.layout_direction() == astra_gui::Layout::Horizontal {
                                let mut row_width = 0.0f32;
                                let child_gap = child.gap();
                                let child_padding = child.padding();

                                for (j, grandchild) in child.children().iter().enumerate() {
                                    if let Some(gc_layout) = grandchild.computed_layout() {
                                        row_width += gc_layout.rect.max[0] - gc_layout.rect.min[0];
                                        if j < child.children().len() - 1 {
                                            row_width += child_gap;
                                        }
                                    }
                                }
                                row_width + child_padding.left + child_padding.right
                            } else {
                                child_width
                            };

                        content_width = content_width.max(actual_child_width);
                        content_height += child_height;

                        if i < children.len() - 1 {
                            content_height += gap;
                        }
                    }
                }
            }
            astra_gui::Layout::Horizontal => {
                // For horizontal layout: accumulate widths, track max height
                for (i, child) in children.iter().enumerate() {
                    if let Some(child_layout) = child.computed_layout() {
                        let child_width = child_layout.rect.max[0] - child_layout.rect.min[0];
                        let child_height = child_layout.rect.max[1] - child_layout.rect.min[1];

                        content_width += child_width;
                        content_height = content_height.max(child_height);

                        if i < children.len() - 1 {
                            content_width += gap;
                        }
                    }
                }
            }
            astra_gui::Layout::Stack => {
                // For stack layout: track max width and max height
                for child in children.iter() {
                    if let Some(child_layout) = child.computed_layout() {
                        let child_width = child_layout.rect.max[0] - child_layout.rect.min[0];
                        let child_height = child_layout.rect.max[1] - child_layout.rect.min[1];

                        content_width = content_width.max(child_width);
                        content_height = content_height.max(child_height);
                    }
                }
            }
        }

        // Max scroll is the amount content exceeds container size
        let max_scroll_x = (content_width - container_width).max(0.0);
        let max_scroll_y = (content_height - container_height).max(0.0);

        (max_scroll_x, max_scroll_y)
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
