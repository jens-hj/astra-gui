//! Hit-testing module for finding nodes at specific screen positions
//!
//! This module provides functions to determine which node(s) are at a given point,
//! respecting the layout hierarchy and overflow clipping.

use crate::layout::{Overflow, Transform2D};
use crate::node::{Node, NodeId};
use crate::primitives::{Point, Rect};

/// Result of a hit test against a node
#[derive(Debug, Clone)]
pub struct HitTestResult {
    /// The ID of the node that was hit (if it has one)
    pub node_id: Option<NodeId>,
    /// Position relative to the node's top-left corner
    pub local_pos: Point,
    /// The computed rectangle of the hit node
    pub node_rect: Rect,
}

/// Hit-test a point against a node tree
///
/// Returns all nodes that contain the point, ordered from root to leaf (shallow to deep).
/// This respects overflow clipping and transforms - nodes outside their parent's clip rect
/// or transformed space are excluded.
///
/// # Arguments
/// * `root` - The root node to test against
/// * `point` - The point in screen coordinates
///
/// # Returns
/// Vector of hit test results, ordered from shallowest (root) to deepest (leaf)
pub fn hit_test_point(root: &Node, point: Point) -> Vec<HitTestResult> {
    let mut results = Vec::new();
    hit_test_recursive(root, point, None, Transform2D::IDENTITY, &mut results);
    results
}

/// Find the deepest node at a given point
///
/// This is a convenience function that returns only the most specific (deepest) node
/// that contains the point, or None if no node contains the point.
///
/// # Arguments
/// * `root` - The root node to test against
/// * `point` - The point in screen coordinates
///
/// # Returns
/// The deepest node's hit test result, or None if no nodes contain the point
pub fn hit_test_deepest(root: &Node, point: Point) -> Option<HitTestResult> {
    hit_test_point(root, point).pop()
}

/// Recursive helper for hit testing
///
/// # Arguments
/// * `node` - Current node being tested
/// * `point` - The point in world screen coordinates
/// * `clip_rect` - The current clipping rectangle (None means no clipping)
/// * `parent_transform` - Accumulated transform from parent nodes
/// * `results` - Accumulator for hit test results
fn hit_test_recursive(
    node: &Node,
    point: Point,
    clip_rect: Option<Rect>,
    parent_transform: Transform2D,
    results: &mut Vec<HitTestResult>,
) {
    // Get the computed layout for this node (untransformed rect)
    let Some(computed) = node.computed_layout() else {
        return; // Node hasn't been laid out yet, skip it
    };

    let node_rect = computed.rect;

    // Build local transform from node properties
    let local_transform = Transform2D {
        translation: node.translation(),
        rotation: node.rotation(),
        origin: node.transform_origin(),
    };

    // Compute rect size for transform operations
    let rect_size = [
        node_rect.max[0] - node_rect.min[0],
        node_rect.max[1] - node_rect.min[1],
    ];

    // Accumulate transforms: parent → local
    let world_transform = parent_transform.then(&local_transform, rect_size);

    // Transform the point to local (untransformed) space using inverse transform
    let local_point_array = world_transform.apply_inverse([point.x, point.y], rect_size);
    let local_test_point = Point {
        x: local_point_array[0],
        y: local_point_array[1],
    };

    // Check if point is within the current clip rect (in world space)
    if let Some(clip) = clip_rect {
        if !clip.contains(point) {
            return; // Point is outside clip rect, early exit
        }
    }

    // Check if transformed point is within this node's untransformed bounds
    if !node_rect.contains(local_test_point) {
        return; // Point is outside this node, skip it and children
    }

    // Skip disabled nodes - they should not receive interaction events
    // However, we still need to test their children (they might not be disabled)
    if !node.is_disabled() {
        // Point is within this node! Add it to results
        // Use the transformed local point for the local position
        let local_pos = Point {
            x: local_test_point.x - node_rect.min[0],
            y: local_test_point.y - node_rect.min[1],
        };

        results.push(HitTestResult {
            node_id: node.id().cloned(),
            local_pos,
            node_rect,
        });
    }

    // Determine clip rect for children
    let child_clip_rect = match node.overflow() {
        Overflow::Hidden | Overflow::Scroll => {
            // This node clips its children - intersect with current clip
            let content_rect = Rect {
                min: [
                    node_rect.min[0] + node.padding().left,
                    node_rect.min[1] + node.padding().top,
                ],
                max: [
                    node_rect.max[0] - node.padding().right,
                    node_rect.max[1] - node.padding().bottom,
                ],
            };

            Some(if let Some(clip) = clip_rect {
                clip.intersect(&content_rect).unwrap_or(content_rect)
            } else {
                content_rect
            })
        }
        Overflow::Visible => {
            // This node allows overflow - pass through current clip rect
            clip_rect
        }
    };

    // Recursively test children with accumulated transform
    for child in node.children() {
        hit_test_recursive(child, point, child_clip_rect, world_transform, results);
    }
}
