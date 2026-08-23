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
    fn set(&mut self, dir: Direction, value: T) {
        match dir {
            Direction::Left => self.left = Some(value),
            Direction::Right => self.right = Some(value),
            Direction::Up => self.up = Some(value),
            Direction::Down => self.down = Some(value),
        }
    }
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
}

impl<T: Clone + Eq + Hash + Debug> Default for FocusGraph<T> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
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
