//! Vim-style (HJKL) keyboard navigation for egui applications.
//!
//! `egui-vim-nav` provides generic, topology-agnostic intra-screen focus
//! navigation. The consuming application defines the focus graph shape
//! (grid, tree, custom) and this crate provides the traversal engine,
//! key handler, and immediate-mode rendering helpers.
//!
//! # No global state
//!
//! Every type in this crate ([`FocusGraph`], [`Navigator`], [`VimKeyHandler`])
//! is a plain value the consuming application owns and persists across frames.
//! There is no hidden global or static mutable instance. Multiple independent
//! focus graphs and navigators can coexist in the same application.
//!
//! # Dependency isolation
//!
//! This crate depends on `egui` only. It has zero dependencies on any other
//! `egui-widgetkit` crate (`egui-layout`, `egui-spring`, `egui-nav-stack`,
//! `egui-themes`, `spring-core`). It can be pulled in alone by any egui app.

pub mod cursor_autohide;
pub mod focus_graph;
pub mod hierarchical_nav;
pub mod key_handler;
pub mod modal_text;
pub mod navigator;
pub mod register;
pub mod scrolloff;
pub mod vim_buffer;

pub use cursor_autohide::{is_cursor_hidden, is_hover_active, CursorAutohide};
pub use focus_graph::{BranchGroup, BranchStrategy, Direction, FocusGraph, Neighbors};
pub use hierarchical_nav::{hierarchical_move, HierarchicalNavResult};
pub use key_handler::{ActionTracker, VimAction, VimActionState, VimKeyHandler};
pub use modal_text::{check_modal_transition, FocusLevel, ModalTransition};
pub use navigator::{FocusEvent, FocusWrap, Navigator};
pub use register::{FocusRegion, FocusRegionResponse};
pub use scrolloff::{compute_scrolloff_delta, NavDirection, Scrolloff};
pub use vim_buffer::{
    calculate_motion, clamp_cursor, HistoryEntry, ParsedCommand, TextObject, TextObjectScope,
    TextObjectTarget, UndoHistory, VimBufferState, VimMode, VimMotion, VimOperator, VimParser,
    VimRegisters, VisualType,
};
