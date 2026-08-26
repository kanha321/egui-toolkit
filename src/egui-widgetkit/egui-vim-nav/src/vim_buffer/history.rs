//! Multi-level Undo / Redo history with cursor preservation.

/// A single snapshot of buffer state in history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    /// Buffer text at this point in time.
    pub text: String,
    /// Cursor byte index at this point in time.
    pub cursor: usize,
}

/// Linear undo/redo stack.
#[derive(Clone, Debug)]
pub struct UndoHistory {
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    max_depth: usize,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 100,
        }
    }
}

impl UndoHistory {
    /// Creates a new history stack with default max depth (100).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum history depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Pushes a new snapshot prior to a mutation.
    pub fn push(&mut self, text: &str, cursor: usize) {
        // If the top of the stack is identical to current, don't duplicate
        if let Some(top) = self.undo_stack.last() {
            if top.text == text && top.cursor == cursor {
                return;
            }
        }

        self.undo_stack.push(HistoryEntry {
            text: text.to_owned(),
            cursor,
        });

        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }

        // New edit clears redo stack
        self.redo_stack.clear();
    }

    /// Undoes the last change, returning the previous state and saving the current state to the redo stack.
    pub fn undo(&mut self, current_text: &str, current_cursor: usize) -> Option<HistoryEntry> {
        let prev = self.undo_stack.pop()?;

        self.redo_stack.push(HistoryEntry {
            text: current_text.to_owned(),
            cursor: current_cursor,
        });

        Some(prev)
    }

    /// Redoes the undone change, returning the next state and saving current state to the undo stack.
    pub fn redo(&mut self, current_text: &str, current_cursor: usize) -> Option<HistoryEntry> {
        let next = self.redo_stack.pop()?;

        self.undo_stack.push(HistoryEntry {
            text: current_text.to_owned(),
            cursor: current_cursor,
        });

        Some(next)
    }

    /// Clears both undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
