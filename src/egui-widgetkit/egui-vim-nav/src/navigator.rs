use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use super::focus_graph::{BranchStrategy, Direction, FocusGraph};

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
    /// Stores the last-visited active child for each branch: `(parent, dir) -> active_child`.
    branch_memory: HashMap<(T, Direction), T>,
    /// Runtime strategy overrides: `(parent, dir) -> strategy`.
    branch_strategies: HashMap<(T, Direction), BranchStrategy<T>>,
}

impl<T: Clone + Eq + Hash + Debug> Default for Navigator<T> {
    fn default() -> Self {
        Self {
            current_focus: None,
            wrap: FocusWrap::default(),
            branch_memory: HashMap::new(),
            branch_strategies: HashMap::new(),
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

    /// Sets focus to the given node and updates branch memory against the provided graph.
    pub fn set_focus_with_graph(&mut self, node: Option<T>, graph: &FocusGraph<T>) {
        if let Some(ref n) = node {
            self.record_focus(n, graph);
        }
        self.current_focus = node;
    }

    /// Records branch memory whenever focus lands on a node that belongs to any registered branch.
    ///
    /// Ensures that intra-branch lateral navigation (e.g. `H`/`L` across siblings) continuously
    /// updates `(parent, dir) -> active_child` memory in real time.
    pub fn record_focus(&mut self, node: &T, graph: &FocusGraph<T>) {
        for ((parent, dir), branch) in graph.branches() {
            if branch.children.contains(node) {
                self.branch_memory.insert((parent.clone(), *dir), node.clone());
            }
        }
    }

    /// Moves focus in the given direction within the graph.
    ///
    /// Resolves 1-to-many branch groups dynamically using the branch's [`BranchStrategy`]
    /// (e.g. returning to the last-visited child under `RememberLast`).
    ///
    /// Returns a [`FocusEvent`] describing the change if focus actually moved, or `None`.
    pub fn move_focus(
        &mut self,
        graph: &FocusGraph<T>,
        dir: Direction,
    ) -> Option<FocusEvent<T>> {
        // If nothing is focused, try to focus the first node in the graph
        let current = match &self.current_focus {
            Some(c) => {
                if graph.contains(c) {
                    c.clone()
                } else {
                    let fallback = graph.first_node()?.clone();
                    let previous = self.current_focus.take();
                    self.record_focus(&fallback, graph);
                    self.current_focus = Some(fallback.clone());
                    return Some(FocusEvent {
                        previous,
                        current: fallback,
                        direction: dir,
                    });
                }
            }
            None => {
                let first = graph.first_node()?.clone();
                self.record_focus(&first, graph);
                self.current_focus = Some(first.clone());
                return Some(FocusEvent {
                    previous: None,
                    current: first,
                    direction: dir,
                });
            }
        };

        // Check if there is a 1-to-many branch originating from `current` in `dir`
        let target_node = if let Some(branch) = graph.get_branch(&current, dir) {
            let strategy = self.branch_strategies.get(&(current.clone(), dir))
                .cloned()
                .unwrap_or_else(|| branch.strategy.clone());

            let resolved = match strategy {
                BranchStrategy::First => {
                    branch.children.first()
                        .filter(|c| graph.contains(c))
                        .or_else(|| branch.children.first())
                }
                BranchStrategy::Last => {
                    branch.children.last()
                        .filter(|c| graph.contains(c))
                        .or_else(|| branch.children.first())
                }
                BranchStrategy::EdgeAware => {
                    let target_child = match dir {
                        Direction::Up | Direction::Left => branch.children.last(),
                        Direction::Down | Direction::Right => branch.children.first(),
                    };
                    target_child
                        .filter(|c| graph.contains(c))
                        .or_else(|| branch.children.first())
                }
                BranchStrategy::Anchor(ref anchor) => {
                    if branch.children.contains(anchor) && graph.contains(anchor) {
                        Some(anchor)
                    } else {
                        // Self-healing fallback if anchor was removed
                        branch.children.first()
                    }
                }
                BranchStrategy::RememberLast => {
                    if let Some(mem) = self.branch_memory.get(&(current.clone(), dir)) {
                        if branch.children.contains(mem) && graph.contains(mem) {
                            Some(mem)
                        } else {
                            // Self-healing fallback if remembered child was removed
                            self.branch_memory.remove(&(current.clone(), dir));
                            branch.children.first()
                        }
                    } else {
                        branch.children.first()
                    }
                }
            };

            resolved.cloned()
        } else {
            // Standard single neighbor
            graph.get_neighbor(&current, dir).cloned()
        };

        match target_node {
            Some(next) => {
                self.record_focus(&next, graph);
                let previous = self.current_focus.replace(next.clone());
                Some(FocusEvent {
                    previous,
                    current: next,
                    direction: dir,
                })
            }
            None => {
                match self.wrap {
                    FocusWrap::Clamp => None,
                }
            }
        }
    }

    /// Returns the remembered child for branch `(parent, dir)`, if any.
    pub fn get_last_branch_focus(&self, parent: &T, dir: Direction) -> Option<&T> {
        self.branch_memory.get(&(parent.clone(), dir))
    }

    /// Explicitly sets the remembered child for branch `(parent, dir)`.
    pub fn set_last_branch_focus(&mut self, parent: T, dir: Direction, child: T) {
        self.branch_memory.insert((parent, dir), child);
    }

    /// Overrides the resolution strategy for a specific branch at runtime.
    pub fn set_branch_strategy(&mut self, parent: T, dir: Direction, strategy: BranchStrategy<T>) {
        self.branch_strategies.insert((parent, dir), strategy);
    }

    /// Clears all recorded branch memory globally.
    pub fn clear_branch_memory(&mut self) {
        self.branch_memory.clear();
    }

    /// Clears all branch memory entries originating from `parent`.
    pub fn clear_branch_memory_for(&mut self, parent: &T) {
        self.branch_memory.retain(|(p, _), _| p != parent);
    }

    /// Clears branch memory specifically for `(parent, dir)`.
    pub fn clear_branch_memory_for_dir(&mut self, parent: &T, dir: Direction) {
        self.branch_memory.remove(&(parent.clone(), dir));
    }
}
