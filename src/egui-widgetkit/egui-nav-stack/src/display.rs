//! Immediate-mode display widget rendering the active top of a `NavStack`.

use crate::stack::NavStack;

/// Navigation action returned from `NavDisplay::show` to apply mutations safely after rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavAction<K> {
    Push(K),
    Pop,
    ReplaceTop(K),
}

/// A widget that displays the current top entry of a `NavStack`.
pub struct NavDisplay<'a, K> {
    stack: &'a NavStack<K>,
}

impl<'a, K> NavDisplay<'a, K> {
    /// Creates a new `NavDisplay` for the given `NavStack`.
    pub fn new(stack: &'a NavStack<K>) -> Self {
        Self { stack }
    }

    /// Accesses the underlying stack reference.
    pub fn stack(&self) -> &'a NavStack<K> {
        self.stack
    }
}
