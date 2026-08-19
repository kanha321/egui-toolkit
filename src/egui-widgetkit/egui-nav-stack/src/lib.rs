//! # egui-nav-stack
//!
//! App-owned back-stack screen navigation system for egui, modeled on Android Navigation 3.
//!
//! ## State Ownership Contract
//!
//! The back stack is a plain `NavStack<K>` value owned by the consuming application.
//! There is no global or hidden controller state (`CODING_RULES §2`).

pub mod display;
pub mod stack;
pub mod transition;

pub use display::{NavAction, NavDisplay};
pub use stack::NavStack;
