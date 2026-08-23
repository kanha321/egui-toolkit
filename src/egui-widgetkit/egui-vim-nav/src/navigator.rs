//! Focus cursor and directional navigation engine.
//!
//! A [`Navigator<T>`] tracks which node in a [`FocusGraph<T>`] currently has
//! focus and provides [`move_focus`](Navigator::move_focus) to traverse the
//! graph in response to directional input.
//!
//! # State ownership
//!
//! `Navigator<T>` is a plain value the consuming application owns and persists
//! across frames. There is no hidden global instance. Multiple independent
//! navigators can coexist (e.g. one per panel or overlay).

use std::fmt::Debug;
use std::hash::Hash;

use super::focus_graph::{Direction, FocusGraph};

/// Controls behavior when navigation reaches a graph edge (no neighbor).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FocusWrap {
    /// Stop at the edge — `move_focus` returns `None` (default).
    #[default]
    Clamp,
    // NOTE: `Wrap` mode (wrap around to opposite edge) is a plausible future
    // addition but requires the graph to declare its own wrap-targets, which
    // is topology-dependent. Left out of v1 to keep the API simple.
}

/// Describes a focus change that occurred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusEvent<T> {
    /// The node that previously had focus (if any).
    pub previous: Option<T>,
    /// The node that now has focus.
    pub current: T,
    /// The direction that caused this change.
    pub direction: Direction,
}

/// Tracks the currently focused node and navigates a [`FocusGraph<T>`].
///
/// # State ownership
///
/// `Navigator<T>` is a plain value the consuming application owns and persists
/// across frames (e.g. as a field on an application state struct). There is no
/// hidden global instance — multiple independent navigators can coexist.
///
/// # Graceful degradation
///
/// - If the focused node is not found in the graph, `move_focus` falls back
///   to the graph's first node (if any) without panicking.
/// - If no neighbor exists in the requested direction, `move_focus` returns
///   `None` and focus stays where it is (clamp behavior).
/// - No method on `Navigator` will ever panic due to caller-supplied input.
#[derive(Clone, Debug)]
pub struct Navigator<T: Clone + Eq + Hash + Debug> {
    current_focus: Option<T>,
    wrap: FocusWrap,
}

impl<T: Clone + Eq + Hash + Debug> Default for Navigator<T> {
    fn default() -> Self {
        Self {
            current_focus: None,
            wrap: FocusWrap::default(),
        }
    }
}

impl<T: Clone + Eq + Hash + Debug> Navigator<T> {
    /// Creates a new navigator with no initial focus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a navigator with the given initial focus.
    pub fn with_initial_focus(mut self, focus: T) -> Self {
        self.current_focus = Some(focus);
        self
    }

    /// Sets the wrap mode for edge behavior.
    pub fn with_wrap(mut self, wrap: FocusWrap) -> Self {
        self.wrap = wrap;
        self
    }

    /// Returns the currently focused node, if any.
    pub fn focused(&self) -> Option<&T> {
        self.current_focus.as_ref()
    }

    /// Sets focus to the given node (or clears focus with `None`).
    pub fn set_focus(&mut self, node: Option<T>) {
        self.current_focus = node;
    }

    /// Moves focus in the given direction within the graph.
    ///
    /// Returns a [`FocusEvent`] describing the change if focus actually moved,
    /// or `None` if:
    /// - No node is currently focused and the graph is empty
    /// - The focused node has no neighbor in that direction (clamp)
    ///
    /// # Graceful fallback
    ///
    /// If the currently focused node is not found in the graph (e.g. the app
    /// removed the region while it was focused), focus falls back to the
    /// graph's first available node. This never panics.
    pub fn move_focus(
        &mut self,
        graph: &FocusGraph<T>,
        dir: Direction,
    ) -> Option<FocusEvent<T>> {
        // If nothing is focused, try to focus the first node in the graph
        let current = match &self.current_focus {
            Some(c) => {
                // Verify the focused node still exists in the graph
                if graph.contains(c) {
                    c.clone()
                } else {
                    // Focused node was removed — fall back to first available
                    let fallback = graph.first_node()?.clone();
                    let previous = self.current_focus.take();
                    self.current_focus = Some(fallback.clone());
                    return Some(FocusEvent {
                        previous,
                        current: fallback,
                        direction: dir,
                    });
                }
            }
            None => {
                // No focus at all — try to establish initial focus
                let first = graph.first_node()?.clone();
                self.current_focus = Some(first.clone());
                return Some(FocusEvent {
                    previous: None,
                    current: first,
                    direction: dir,
                });
            }
        };

        // Look up the neighbor in the requested direction
        match graph.get_neighbor(&current, dir) {
            Some(next) => {
                let next = next.clone();
                let previous = self.current_focus.replace(next.clone());
                Some(FocusEvent {
                    previous,
                    current: next,
                    direction: dir,
                })
            }
            None => {
                // No neighbor in that direction
                match self.wrap {
                    FocusWrap::Clamp => None, // Stay where we are
                }
            }
        }
    }
}
