//! Pure Rust Vim modal editing engine.
//!
//! Provides a complete, zero-dependency modal state machine, motion
//! calculator, text-object resolver, grammar parser, and undo history stack.

pub mod history;
pub mod mode;
pub mod motion;
pub mod operator;
pub mod parser;
pub mod register;
pub mod state;
pub mod text_object;

pub use history::{HistoryEntry, UndoHistory};
pub use mode::{VimMode, VisualType};
pub use motion::{calculate_motion, clamp_cursor, VimMotion};
pub use operator::VimOperator;
pub use parser::{ParsedCommand, VimParser};
pub use register::VimRegisters;
pub use state::VimBufferState;
pub use text_object::{TextObject, TextObjectScope, TextObjectTarget};
