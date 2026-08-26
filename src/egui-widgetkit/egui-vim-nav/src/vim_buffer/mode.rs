//! Modal states for Vim editing.

use crate::vim_buffer::operator::VimOperator;

/// Visual selection type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualType {
    /// Character-wise selection (`v`).
    Character,
    /// Line-wise selection (`V`).
    Line,
    /// Block-wise rectangular selection (`Ctrl+v`).
    Block,
}

/// The active modal state of the Vim editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VimMode {
    /// Normal navigation and command mode.
    Normal,
    /// Native text typing / insert mode (`i`, `a`, `o`, `I`, `A`, `O`).
    Insert,
    /// Visual selection mode.
    Visual(VisualType),
    /// Overwrite replacement mode (`R`).
    Replace,
    /// Waiting for a motion or text-object following an operator (`d`, `c`, `y`, etc.).
    OperatorPending {
        /// The active operator being executed.
        operator: VimOperator,
        /// Explicit count prefix (e.g. `3` in `3dw`).
        count: usize,
    },
}

impl Default for VimMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl VimMode {
    /// Returns `true` if in Insert mode.
    #[inline]
    pub fn is_insert(&self) -> bool {
        matches!(self, Self::Insert)
    }

    /// Returns `true` if in any Visual selection mode.
    #[inline]
    pub fn is_visual(&self) -> bool {
        matches!(self, Self::Visual(_))
    }

    /// Returns `true` if in Normal mode.
    #[inline]
    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Returns `true` if waiting for an operator's target motion.
    #[inline]
    pub fn is_operator_pending(&self) -> bool {
        matches!(self, Self::OperatorPending { .. })
    }

    /// Returns a human-readable display label for UI status bars/badges.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual(VisualType::Character) => "VISUAL",
            Self::Visual(VisualType::Line) => "V-LINE",
            Self::Visual(VisualType::Block) => "V-BLOCK",
            Self::Replace => "REPLACE",
            Self::OperatorPending { .. } => "OPERATOR",
        }
    }
}
