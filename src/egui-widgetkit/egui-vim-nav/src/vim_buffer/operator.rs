//! Vim operators and buffer transformations.

/// Fundamental Vim editing operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VimOperator {
    /// Delete targeted range into unnamed register (`d`).
    Delete,
    /// Delete targeted range into unnamed register and switch to Insert mode (`c`).
    Change,
    /// Copy targeted range into register without deleting (`y`).
    Yank,
    /// Paste text from register before or after cursor (`p` / `P`).
    Put {
        /// If `true`, pastes after cursor (`p`); otherwise before (`P`).
        after: bool,
    },
    /// Replace character under cursor with a specific character (`r`).
    ReplaceChar(char),
    /// Substitute character under cursor (`s`).
    Substitute,
    /// Substitute entire line (`S` / `cc`).
    SubstituteLine,
    /// Delete single character under or before cursor (`x` / `X`).
    DeleteChar {
        /// If `true`, deletes under cursor (`x`); otherwise before cursor (`X`).
        forward: bool,
    },
    /// Toggle character case under cursor or selection (`~`).
    ToggleCase,
    /// Transform selection or target to lowercase (`gu`).
    ToLower,
    /// Transform selection or target to uppercase (`gU`).
    ToUpper,
    /// Indent line or selection (`>`).
    Indent,
    /// Outdent line or selection (`<`).
    Outdent,
}
