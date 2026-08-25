//! Generic, topology-agnostic focus graph for keyboard navigation.
//!
//! A [`FocusGraph<T>`] stores a set of navigable nodes and their directional
//! neighbor relationships. The consuming application defines the topology
//! (grid, tree, custom) by calling builder methods like
//! [`connect_horizontal`](FocusGraph::connect_horizontal) or
//! [`connect_grid`](FocusGraph::connect_grid). This crate has no opinion on
//! what the nodes *are* — it just stores edges between them.
//!
//! # State ownership
//!
//! `FocusGraph<T>` is a plain value the consuming application owns and persists
//! across frames. There is no hidden global instance. Multiple independent
//! focus graphs can coexist in the same application.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Cardinal direction for focus navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// Returns the opposite direction.
    ///
    /// ```
    /// use egui_vim_nav::Direction;
    /// assert_eq!(Direction::Left.opposite(), Direction::Right);
    /// assert_eq!(Direction::Up.opposite(), Direction::Down);
    /// ```
    pub fn opposite(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

/// Four-directional neighbor links for a single node.
#[derive(Clone, Debug)]
pub struct Neighbors<T> {
    pub left: Option<T>,
    pub right: Option<T>,
    pub up: Option<T>,
    pub down: Option<T>,
}

impl<T> Default for Neighbors<T> {
    fn default() -> Self {
        Self {
            left: None,
            right: None,
            up: None,
            down: None,
        }
    }
}

impl<T> Neighbors<T> {
    /// Returns the neighbor in the given direction, if any.
    pub fn get(&self, dir: Direction) -> Option<&T> {
        match dir {
            Direction::Left => self.left.as_ref(),
            Direction::Right => self.right.as_ref(),
            Direction::Up => self.up.as_ref(),
            Direction::Down => self.down.as_ref(),
        }
    }

    /// Sets the neighbor in the given direction.
    pub fn set(&mut self, dir: Direction, value: T) {
        match dir {
            Direction::Left => self.left = Some(value),
            Direction::Right => self.right = Some(value),
            Direction::Up => self.up = Some(value),
            Direction::Down => self.down = Some(value),
        }
    }

    /// Sets or clears the neighbor in the given direction.
    pub fn set_opt(&mut self, dir: Direction, value: Option<T>) {
        match dir {
            Direction::Left => self.left = value,
            Direction::Right => self.right = value,
            Direction::Up => self.up = value,
            Direction::Down => self.down = value,
        }
    }

    /// Clears the neighbor in the given direction.
    pub fn clear_dir(&mut self, dir: Direction) {
        self.set_opt(dir, None);
    }
}

/// Controls how focus is resolved when navigating from a parent node into a multi-child branch group.
///
/// # Note on `Copy`
/// `BranchStrategy<T>` implements `Copy` if and only if `T: Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BranchStrategy<T> {
    /// Remembers and returns to whichever child was last focused in this branch (Default).
    #[default]
    RememberLast,
    /// Always lands on the first child in the group.
    First,
    /// Always lands on the last child in the group.
    Last,
    /// Direction-aware boundary entry: lands on `last()` when moving `Up` or `Left`, and `first()` when moving `Down` or `Right`.
    EdgeAware,
    /// Always lands on a specific designated anchor child.
    Anchor(T),
}

/// Metadata defining a 1-to-many branch group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchGroup<T> {
    /// The parent node originating the branch.
    pub parent: T,
    /// Direction from parent to the branch children.
    pub direction: Direction,
    /// Ordered list of children in the branch group.
    pub children: Vec<T>,
    /// Sibling chaining direction (e.g. `Right` if children are chained left-to-right).
    pub sibling_direction: Direction,
    /// Resolution strategy.
    pub strategy: BranchStrategy<T>,
}

/// A generic, topology-agnostic graph of navigable elements.
///
/// Stores nodes keyed by an application-supplied ID type `T` and their
/// directional neighbor relationships. The consuming application defines
/// the graph shape via builder methods; this crate never assumes a specific
/// topology (grid, tree, ring, etc.).
///
/// # State ownership
///
/// `FocusGraph<T>` is a plain value the consuming application owns. There is
/// no hidden global instance — multiple independent focus graphs can coexist.
///
/// # Type parameter
///
/// `T` must implement `Clone + Eq + Hash + Debug`. Common choices:
/// - `&'static str` for simple string IDs
/// - An application-defined enum of focusable regions
/// - `egui::Id` (if using egui's built-in ID system)
#[derive(Clone, Debug)]
pub struct FocusGraph<T: Clone + Eq + Hash + Debug> {
    nodes: HashMap<T, Neighbors<T>>,
    branches: HashMap<(T, Direction), BranchGroup<T>>,
}

impl<T: Clone + Eq + Hash + Debug> Default for FocusGraph<T> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            branches: HashMap::new(),
        }
    }
}

impl<T: Clone + Eq + Hash + Debug> FocusGraph<T> {
    /// Creates an empty focus graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a node with explicit neighbor links.
    ///
    /// If the node already exists, its neighbors are replaced.
    pub fn insert(&mut self, node: T, neighbors: Neighbors<T>) {
        self.nodes.insert(node, neighbors);
    }

    /// Connects two nodes horizontally (bidirectional).
    ///
    /// Sets `left.right = right` AND `right.left = left`.
    /// Both nodes are created if they don't already exist.
    pub fn connect_horizontal(&mut self, left: T, right: T) {
        self.nodes.entry(left.clone()).or_default().right = Some(right.clone());
        self.nodes.entry(right).or_default().left = Some(left);
    }

    /// Connects two nodes vertically (bidirectional).
    ///
    /// Sets `top.down = bottom` AND `bottom.up = top`.
    /// Both nodes are created if they don't already exist.
    pub fn connect_vertical(&mut self, top: T, bottom: T) {
        self.nodes.entry(top.clone()).or_default().down = Some(bottom.clone());
        self.nodes.entry(bottom).or_default().up = Some(top);
    }

    /// Connects a one-way directional edge from `from` to `to`.
    ///
    /// Only sets the neighbor in the specified direction on `from`.
    /// Does NOT set the reverse direction on `to`.
    pub fn connect_directed(&mut self, from: T, to: T, dir: Direction) {
        self.nodes.entry(from).or_default().set(dir, to);
    }

    /// Connects a parent node to a child branch group along an inferred orthogonal axis,
    /// using the default [`BranchStrategy::RememberLast`] memory strategy.
    ///
    /// Siblings are automatically chained perpendicular to `dir` (e.g. `Down` connects
    /// children left-to-right via `Right`), and all children receive reverse links to `parent`.
    pub fn connect_branch(&mut self, parent: T, dir: Direction, children: &[T]) -> &mut Self {
        self.connect_branch_with_strategy(parent, dir, children, BranchStrategy::RememberLast)
    }

    /// Connects a parent node to a child branch group along an inferred orthogonal axis,
    /// using an explicit [`BranchStrategy`].
    pub fn connect_branch_with_strategy(
        &mut self,
        parent: T,
        dir: Direction,
        children: &[T],
        strategy: BranchStrategy<T>,
    ) -> &mut Self {
        let sibling_dir = match dir {
            Direction::Up | Direction::Down => Direction::Right,
            Direction::Left | Direction::Right => Direction::Down,
        };
        self.connect_branch_full(parent, dir, children, sibling_dir, strategy)
    }

    /// Connects a parent node to a child branch group along an explicit sibling axis,
    /// using the default [`BranchStrategy::RememberLast`] strategy.
    pub fn connect_branch_with_axis(
        &mut self,
        parent: T,
        dir: Direction,
        children: &[T],
        sibling_dir: Direction,
    ) -> &mut Self {
        self.connect_branch_full(parent, dir, children, sibling_dir, BranchStrategy::RememberLast)
    }

    /// Connects a multi-child branch strip between two parent nodes (e.g. `TopNode <-> [Strip] <-> BottomNode`).
    ///
    /// Automatically establishes:
    /// - A branch group from `top_parent` in `dir` to `children` with [`BranchStrategy::RememberLast`].
    /// - A branch group from `bottom_parent` in `dir.opposite()` to `children` with [`BranchStrategy::RememberLast`].
    /// - Reverse links on all `children` pointing to `top_parent` (in `dir.opposite()`) and `bottom_parent` (in `dir`).
    /// - Sibling chaining along the inferred orthogonal axis.
    pub fn connect_branch_between(
        &mut self,
        top_parent: T,
        dir: Direction,
        children: &[T],
        bottom_parent: T,
    ) -> &mut Self {
        self.connect_branch(top_parent, dir, children);
        self.connect_branch(bottom_parent, dir.opposite(), children);
        self
    }

    /// Connects a parent node to a child branch group with full control over sibling direction
    /// and focus resolution strategy.
    ///
    /// # Panics
    /// Panics if `children` is empty, or if `strategy` is `BranchStrategy::Anchor(anchor)`
    /// and `anchor` is not present in `children`.
    pub fn connect_branch_full(
        &mut self,
        parent: T,
        dir: Direction,
        children: &[T],
        sibling_dir: Direction,
        strategy: BranchStrategy<T>,
    ) -> &mut Self {
        assert!(!children.is_empty(), "FocusGraph::connect_branch: children slice cannot be empty");

        if let BranchStrategy::Anchor(ref anchor) = strategy {
            assert!(
                children.contains(anchor),
                "FocusGraph::connect_branch: Anchor node {:?} is not in the provided children list",
                anchor
            );
        }

        // Ensure parent node entry exists
        self.nodes.entry(parent.clone()).or_default();

        // Ensure all child node entries exist and connect reverse link back to parent
        for child in children {
            self.nodes.entry(child.clone()).or_default().set(dir.opposite(), parent.clone());
        }

        // Chain siblings along sibling_dir
        for i in 0..children.len().saturating_sub(1) {
            let a = children[i].clone();
            let b = children[i + 1].clone();
            self.connect_directed(a.clone(), b.clone(), sibling_dir);
            self.connect_directed(b, a, sibling_dir.opposite());
        }

        // Default initial edge from parent points to first child
        self.connect_directed(parent.clone(), children[0].clone(), dir);

        // Record branch group metadata
        self.branches.insert(
            (parent.clone(), dir),
            BranchGroup {
                parent,
                direction: dir,
                children: children.to_vec(),
                sibling_direction: sibling_dir,
                strategy,
            },
        );

        self
    }

    /// Disconnects a branch originating from `parent` in direction `dir`.
    ///
    /// Removes the child-to-parent reverse links, unlinks parent's edge, and returns
    /// the removed [`BranchGroup`], if any.
    pub fn disconnect_branch(&mut self, parent: &T, dir: Direction) -> Option<BranchGroup<T>> {
        let branch = self.branches.remove(&(parent.clone(), dir))?;

        // Remove parent's forward edge if it points to a child in this branch
        if let Some(neighbors) = self.nodes.get_mut(parent) {
            if let Some(target) = neighbors.get(dir) {
                if branch.children.contains(target) {
                    neighbors.clear_dir(dir);
                }
            }
        }

        // Remove each child's reverse link back to parent
        for child in &branch.children {
            if let Some(neighbors) = self.nodes.get_mut(child) {
                if neighbors.get(dir.opposite()) == Some(parent) {
                    neighbors.clear_dir(dir.opposite());
                }
            }
        }

        Some(branch)
    }

    /// Updates the child set for an existing branch, reconciling links cleanly.
    ///
    /// Preserves the existing sibling axis and resolution strategy. If the strategy was
    /// `BranchStrategy::Anchor(x)` and `x` is not in `new_children`, the strategy safely
    /// self-heals to `BranchStrategy::First`.
    pub fn update_branch_children(
        &mut self,
        parent: &T,
        dir: Direction,
        new_children: &[T],
    ) -> &mut Self {
        assert!(!new_children.is_empty(), "FocusGraph::update_branch_children: new_children cannot be empty");

        let (sibling_dir, strategy) = if let Some(existing) = self.branches.get(&(parent.clone(), dir)) {
            let mut strat = existing.strategy.clone();
            if let BranchStrategy::Anchor(ref a) = strat {
                if !new_children.contains(a) {
                    strat = BranchStrategy::First;
                }
            }
            (existing.sibling_direction, strat)
        } else {
            let sibling_dir = match dir {
                Direction::Up | Direction::Down => Direction::Right,
                Direction::Left | Direction::Right => Direction::Down,
            };
            (sibling_dir, BranchStrategy::RememberLast)
        };

        self.disconnect_branch(parent, dir);
        self.connect_branch_full(parent.clone(), dir, new_children, sibling_dir, strategy);
        self
    }

    /// Returns a reference to all registered branch groups in the graph.
    pub fn branches(&self) -> &HashMap<(T, Direction), BranchGroup<T>> {
        &self.branches
    }

    /// Returns the branch group originating from `parent` in direction `dir`, if any.
    pub fn get_branch(&self, parent: &T, dir: Direction) -> Option<&BranchGroup<T>> {
        self.branches.get(&(parent.clone(), dir))
    }

    /// Builds a 2D grid topology from a row-major matrix.
    ///
    /// Connects all horizontally and vertically adjacent cells bidirectionally.
    /// All nodes in the grid are inserted (created if absent).
    ///
    /// # Example
    ///
    /// ```
    /// use egui_vim_nav::FocusGraph;
    ///
    /// let mut graph = FocusGraph::new();
    /// graph.connect_grid(&[
    ///     &["a", "b", "c"],
    ///     &["d", "e", "f"],
    /// ]);
    /// assert_eq!(graph.get_neighbor(&"a", egui_vim_nav::Direction::Right), Some(&"b"));
    /// assert_eq!(graph.get_neighbor(&"a", egui_vim_nav::Direction::Down), Some(&"d"));
    /// ```
    pub fn connect_grid(&mut self, grid: &[&[T]]) {
        // Ensure all nodes exist
        for row in grid {
            for node in *row {
                self.nodes.entry(node.clone()).or_default();
            }
        }

        // Connect horizontal neighbors
        for row in grid {
            for col in 0..row.len().saturating_sub(1) {
                self.connect_horizontal(row[col].clone(), row[col + 1].clone());
            }
        }

        // Connect vertical neighbors
        for r in 0..grid.len().saturating_sub(1) {
            let top_row = grid[r];
            let bot_row = grid[r + 1];
            let cols = top_row.len().min(bot_row.len());
            for c in 0..cols {
                self.connect_vertical(top_row[c].clone(), bot_row[c].clone());
            }
        }
    }

    /// Returns the neighbor of `node` in the given direction, if any.
    ///
    /// Returns `None` if the node doesn't exist in the graph or has no
    /// neighbor in that direction. Never panics.
    pub fn get_neighbor(&self, node: &T, dir: Direction) -> Option<&T> {
        self.nodes.get(node).and_then(|n| n.get(dir))
    }

    /// Returns `true` if the graph contains the given node.
    pub fn contains(&self, node: &T) -> bool {
        self.nodes.contains_key(node)
    }

    /// Returns an iterator over all node IDs in the graph.
    pub fn nodes(&self) -> impl Iterator<Item = &T> {
        self.nodes.keys()
    }

    /// Returns the number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the graph contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the first node in the graph, if any.
    ///
    /// Useful as a fallback when the focused node has been removed.
    /// Note: `HashMap` iteration order is not deterministic, so this
    /// returns an arbitrary node.
    pub fn first_node(&self) -> Option<&T> {
        self.nodes.keys().next()
    }

    /// Removes all nodes and edges from the graph.
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}
