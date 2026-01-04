use astra_gui::transition::lerp_style;
use astra_gui::{Node, NodeId, Style, Transition};
use std::collections::HashMap;
use std::time::Instant;

/// Check if two styles differ in any animatable property
fn styles_differ(a: &Style, b: &Style) -> bool {
    a.fill_color != b.fill_color
        || a.stroke != b.stroke
        || a.corner_shape != b.corner_shape
        || a.opacity != b.opacity
        || a.text_color != b.text_color
        || a.translation_x != b.translation_x
        || a.translation_y != b.translation_y
        || a.width_override != b.width_override
        || a.height_override != b.height_override
}

/// Current interaction state of a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionState {
    Idle,
    Hovered,
    Active,
    Disabled,
}

/// Transition state for a single node
#[derive(Debug)]
struct NodeTransitionState {
    /// Current interaction state
    current_state: InteractionState,

    /// Previous interaction state (for detecting changes)
    previous_state: InteractionState,

    /// Previous base style (for detecting property changes)
    previous_base_style: Option<Style>,

    /// When the transition started
    transition_start: Option<Instant>,

    /// Style we're transitioning from
    from_style: Option<Style>,

    /// Style we're transitioning to
    to_style: Option<Style>,

    /// Current computed style (result of interpolation)
    current_style: Option<Style>,

    /// Last resolved width (for detecting dimension changes)
    last_width: Option<f32>,

    /// Last resolved height (for detecting dimension changes)
    last_height: Option<f32>,
}

/// Manages interactive state and transitions for all nodes
///
/// This is the external state tracker that maintains node states across frames.
/// Since nodes are rebuilt every frame, this manager preserves transition state
/// and interpolates between styles smoothly.
pub struct InteractiveStateManager {
    states: HashMap<NodeId, NodeTransitionState>,
    current_time: Instant,
}

impl InteractiveStateManager {
    /// Create a new interactive state manager
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current_time: Instant::now(),
        }
    }

    /// Call at start of each frame to update the current time
    pub fn begin_frame(&mut self) {
        self.current_time = Instant::now();
    }

    /// Assign auto-generated IDs to nodes that need them for interactivity
    ///
    /// This must be called BEFORE event dispatch so that hit-testing can find
    /// nodes with hover/active styles. Call this after building the UI tree
    /// but before calling dispatch() on the event dispatcher.
    pub fn assign_auto_ids(node: &mut Node) {
        Self::assign_auto_ids_recursive(node, &mut vec![]);
    }

    /// Internal recursive helper for assign_auto_ids
    fn assign_auto_ids_recursive(node: &mut Node, path: &mut Vec<usize>) {
        // Check if node needs an auto-ID for interactivity
        let needs_auto_id = node.id().is_none()
            && (node.hover_style().is_some()
                || node.active_style().is_some()
                || node.disabled_style().is_some());

        if needs_auto_id {
            // Generate a stable auto-ID based on tree path
            let path_str = path
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join("_");
            let auto_id = NodeId::new(format!("__auto_path_{}", path_str));
            node.set_auto_id(auto_id);
        }

        // Recursively process children with updated path
        for (idx, child) in node.children_mut().iter_mut().enumerate() {
            path.push(idx);
            Self::assign_auto_ids_recursive(child, path);
            path.pop();
        }
    }

    /// Update interaction state for a node and return the computed style
    ///
    /// This is called for each interactive node during rendering to compute
    /// its current style based on its interaction state and transition progress.
    pub fn update_state(
        &mut self,
        node_id: &NodeId,
        new_state: InteractionState,
        base_style: &Style,
        hover_style: Option<&Style>,
        active_style: Option<&Style>,
        disabled_style: Option<&Style>,
        transition: Option<&Transition>,
        current_width: Option<f32>,
        current_height: Option<f32>,
    ) -> Style {
        let entry = self
            .states
            .entry(node_id.clone())
            .or_insert_with(|| NodeTransitionState {
                current_state: InteractionState::Idle,
                previous_state: InteractionState::Idle,
                previous_base_style: Some(base_style.clone()),
                transition_start: None,
                from_style: None,
                to_style: None,
                current_style: Some(base_style.clone()),
                last_width: current_width,
                last_height: current_height,
            });

        // Determine target style based on state
        let mut target_style = match new_state {
            InteractionState::Idle => base_style.clone(),
            InteractionState::Hovered => {
                let mut style = base_style.clone();
                if let Some(hover) = hover_style {
                    style = style.merge(hover);
                }
                style
            }
            InteractionState::Active => {
                let mut style = base_style.clone();
                if let Some(hover) = hover_style {
                    style = style.merge(hover);
                }
                if let Some(active) = active_style {
                    style = style.merge(active);
                }
                style
            }
            InteractionState::Disabled => {
                let mut style = base_style.clone();
                if let Some(disabled) = disabled_style {
                    style = style.merge(disabled);
                } else {
                    // Fallback: apply reduced opacity to base style
                    style.opacity = Some(0.5);
                }
                style
            }
        };

        // Only detect dimension changes when NOT currently transitioning
        // (to avoid capturing interpolated values during transitions)
        let is_transitioning = entry.transition_start.is_some();
        let dimensions_changed = if !is_transitioning {
            let width_changed = entry.last_width != current_width;
            let height_changed = entry.last_height != current_height;
            width_changed || height_changed
        } else {
            false // Don't detect changes during transitions
        };

        // Add current resolved dimensions to target style for interpolation
        // During transitions, keep using the target dimensions we already set
        if !is_transitioning {
            target_style.width_override = current_width;
            target_style.height_override = current_height;
        } else {
            // Keep the target dimensions from the ongoing transition
            target_style.width_override = entry.to_style.as_ref().and_then(|s| s.width_override);
            target_style.height_override = entry.to_style.as_ref().and_then(|s| s.height_override);
        }

        // Detect state change OR style property change OR dimension change
        let state_changed = new_state != entry.current_state;
        let style_changed = entry
            .previous_base_style
            .as_ref()
            .map(|prev| styles_differ(prev, base_style))
            .unwrap_or(true);

        if state_changed || style_changed || dimensions_changed {
            entry.previous_state = entry.current_state;
            entry.current_state = new_state;
            entry.previous_base_style = Some(base_style.clone());

            // Create from_style with last known dimensions for smooth transition
            let mut from_style = entry
                .current_style
                .clone()
                .unwrap_or_else(|| base_style.clone());
            from_style.width_override = entry.last_width;
            from_style.height_override = entry.last_height;

            entry.from_style = Some(from_style);
            entry.to_style = Some(target_style.clone());
            entry.transition_start = Some(self.current_time);
        }

        // Update transition
        if let (Some(start), Some(from), Some(to), Some(trans)) = (
            entry.transition_start,
            &entry.from_style,
            &entry.to_style,
            transition,
        ) {
            let elapsed = (self.current_time - start).as_secs_f32();

            if elapsed >= trans.duration {
                // Transition complete
                entry.current_style = Some(to.clone());
                entry.transition_start = None;
                // Update last known dimensions after transition completes
                entry.last_width = current_width;
                entry.last_height = current_height;
            } else {
                // Interpolate
                let progress = elapsed / trans.duration;
                let eased = (trans.easing)(progress);
                let interpolated = lerp_style(from, to, eased);
                entry.current_style = Some(interpolated);
            }
        } else {
            // No transition, use target directly
            entry.current_style = Some(target_style);
            // Update last known dimensions when not transitioning
            entry.last_width = current_width;
            entry.last_height = current_height;
        }

        entry
            .current_style
            .clone()
            .unwrap_or_else(|| base_style.clone())
    }

    /// Check if any transitions are currently active
    ///
    /// Returns true if any node is mid-transition, indicating that
    /// continuous redraws are needed for smooth animation.
    pub fn has_active_transitions(&self) -> bool {
        self.states.values().any(|s| s.transition_start.is_some())
    }

    /// Apply interactive styles to a node tree
    ///
    /// This traverses the tree and applies computed styles to nodes with IDs.
    /// Auto-IDs should have been assigned via `assign_auto_ids()` before calling this.
    pub fn apply_styles(
        &mut self,
        node: &mut Node,
        interaction_states: &HashMap<NodeId, InteractionState>,
    ) {
        // Apply styles if node has an ID and base style
        if let Some(node_id) = node.id() {
            if let Some(base_style) = node.base_style() {
                // Capture resolved dimensions AFTER layout
                let resolved_width = node
                    .computed_layout()
                    .map(|layout| layout.rect.max[0] - layout.rect.min[0]);
                let resolved_height = node
                    .computed_layout()
                    .map(|layout| layout.rect.max[1] - layout.rect.min[1]);

                // Check if node is disabled - if so, force Disabled state
                let state = if node.is_disabled() {
                    InteractionState::Disabled
                } else {
                    interaction_states
                        .get(node_id)
                        .copied()
                        .unwrap_or(InteractionState::Idle)
                };

                // Compute the current style with transitions
                let computed_style = self.update_state(
                    node_id,
                    state,
                    base_style,
                    node.hover_style(),
                    node.active_style(),
                    node.disabled_style(),
                    node.transition(),
                    resolved_width,
                    resolved_height,
                );

                // Apply the computed style to the node
                computed_style.apply_to_node(node);
            }
        }

        // Recursively apply to children
        for child in node.children_mut() {
            self.apply_styles(child, interaction_states);
        }
    }
}

impl Default for InteractiveStateManager {
    fn default() -> Self {
        Self::new()
    }
}
