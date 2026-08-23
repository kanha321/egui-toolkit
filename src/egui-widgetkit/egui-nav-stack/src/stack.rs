//! App-owned back-stack and forward-stack data structure for screen and destination navigation.
//!
//! Modeled on the philosophy of Android Navigation 3 and browser-style undo/redo history:
//! the navigation state is a plain, app-owned list of screen keys (`NavStack<K>`), with zero global controller state.

use crate::display::NavAction;

#[cfg(feature = "animated-transitions")]
use crate::transition::NavTransition;

/// An app-owned back stack with undo/redo (forward) navigation history.
///
/// # State Ownership
///
/// The consuming application owns and stores `NavStack<K>` as plain state across frames
/// (e.g. inside an application state struct) and passes it by `&` / `&mut` reference (`CODING_RULES §2`).
///
/// # Undo / Redo Navigation
///
/// - Going **Back** ([`pop`](Self::pop) / [`go_back`](Self::go_back)) moves the active screen onto the forward history stack.
/// - Going **Forward** ([`go_forward`](Self::go_forward)) restores the next screen from the forward history stack.
/// - Pushing a new destination ([`push`](Self::push)) clears the forward history stack (branching navigation).
///
/// # Examples
///
/// ```rust
/// use egui_nav_stack::NavStack;
///
/// #[derive(Clone, Debug, PartialEq)]
/// enum Screen {
///     Home,
///     Catalog,
///     Detail(u32),
/// }
///
/// let mut stack = NavStack::new(Screen::Home);
/// stack.push(Screen::Catalog);
/// stack.push(Screen::Detail(42));
/// assert_eq!(stack.len(), 3);
///
/// // Back (Undo navigation)
/// let popped = stack.go_back();
/// assert_eq!(popped, Some(Screen::Detail(42)));
/// assert_eq!(stack.top(), Some(&Screen::Catalog));
/// assert!(stack.can_go_forward());
///
/// // Forward (Redo navigation)
/// let restored = stack.go_forward();
/// assert_eq!(restored, Some(Screen::Detail(42)));
/// assert_eq!(stack.top(), Some(&Screen::Detail(42)));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavStack<K> {
    entries: Vec<K>,
    forward_entries: Vec<K>,
}

impl<K> Default for NavStack<K> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            forward_entries: Vec::new(),
        }
    }
}

impl<K> NavStack<K> {
    /// Creates an empty `NavStack` with no initial screens.
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            forward_entries: Vec::new(),
        }
    }

    /// Creates a `NavStack` pre-populated with the provided list of entries.
    pub fn from_vec(entries: Vec<K>) -> Self {
        Self {
            entries,
            forward_entries: Vec::new(),
        }
    }

    /// Clears all entries and forward history from the stack.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.forward_entries.clear();
    }

    /// Clears only the forward redo history.
    pub fn clear_forward(&mut self) {
        self.forward_entries.clear();
    }

    /// Returns a reference to the active top destination, or `None` if the stack is empty.
    pub fn top(&self) -> Option<&K> {
        self.entries.last()
    }

    /// Returns a mutable reference to the active top destination.
    ///
    /// # Note on Static Updates vs Navigation
    ///
    /// Use `top_mut` to update in-place internal data of the active screen. To change destinations
    /// or trigger navigation transitions, use [`replace_top`](Self::replace_top) or [`push`](Self::push).
    pub fn top_mut(&mut self) -> Option<&mut K> {
        self.entries.last_mut()
    }

    /// Returns the entry at the specified 0-based stack index (`0` is root).
    pub fn get(&self, index: usize) -> Option<&K> {
        self.entries.get(index)
    }

    /// Returns the total number of destinations currently on the active stack.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the active stack contains zero destinations.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns `true` if the stack has at least one screen to pop without becoming empty (`len() > 1`).
    pub fn can_pop(&self) -> bool {
        self.entries.len() > 1
    }

    /// Alias for [`can_pop`](Self::can_pop).
    pub fn can_go_back(&self) -> bool {
        self.can_pop()
    }

    /// Returns `true` if there are forward destinations available for redo navigation.
    pub fn can_go_forward(&self) -> bool {
        !self.forward_entries.is_empty()
    }

    /// Returns the number of forward destinations available.
    pub fn forward_len(&self) -> usize {
        self.forward_entries.len()
    }

    /// Returns a slice of all forward destinations available for redo.
    pub fn forward_entries(&self) -> &[K] {
        &self.forward_entries
    }

    /// Returns an iterator over references to all destinations in the active stack from bottom to top.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &K> {
        self.entries.iter()
    }

    /// Returns a slice of all active destinations currently on the stack.
    pub fn entries(&self) -> &[K] {
        &self.entries
    }
}

impl<K: Clone> NavStack<K> {
    /// Creates a new `NavStack` initialized with a root home screen.
    ///
    /// This is the primary constructor, ensuring the stack always has a starting home screen.
    pub fn new(root: K) -> Self {
        Self {
            entries: vec![root],
            forward_entries: Vec::new(),
        }
    }

    /// Pushes a new destination key onto the top of the stack and clears any forward redo history.
    pub fn push(&mut self, key: K) {
        self.forward_entries.clear();
        self.entries.push(key);
    }

    /// Pops the active destination and moves it onto the forward history stack (Undo navigation).
    ///
    /// Returns `None` if the stack is empty. Never panics (`CODING_RULES §3`).
    pub fn pop(&mut self) -> Option<K> {
        if let Some(entry) = self.entries.pop() {
            self.forward_entries.push(entry.clone());
            Some(entry)
        } else {
            None
        }
    }

    /// Alias for [`pop`](Self::pop) representing back / undo navigation.
    pub fn go_back(&mut self) -> Option<K> {
        self.pop()
    }

    /// Moves the next destination from the forward history stack back onto the active stack (Redo navigation).
    ///
    /// Returns `None` if there are no forward entries.
    pub fn go_forward(&mut self) -> Option<K> {
        if let Some(entry) = self.forward_entries.pop() {
            self.entries.push(entry.clone());
            Some(entry)
        } else {
            None
        }
    }

    /// Pops destinations from the stack until `predicate(&key)` returns `true` for the top item.
    ///
    /// Popped items are stored in the forward history stack in order.
    /// Returns the removed entries in the order they were popped.
    pub fn pop_to(&mut self, predicate: impl Fn(&K) -> bool) -> Vec<K> {
        let target_index = self.entries.iter().rposition(&predicate);

        match target_index {
            Some(idx) => {
                let mut popped = Vec::new();
                while self.entries.len() > idx + 1 {
                    if let Some(entry) = self.entries.pop() {
                        self.forward_entries.push(entry.clone());
                        popped.push(entry);
                    }
                }
                popped
            }
            None => Vec::new(),
        }
    }

    /// Pops all destinations above the root item (index `0`), preserving them in forward history.
    pub fn pop_to_root(&mut self) -> Vec<K> {
        if self.entries.len() <= 1 {
            return Vec::new();
        }
        let mut popped = Vec::new();
        while self.entries.len() > 1 {
            if let Some(entry) = self.entries.pop() {
                self.forward_entries.push(entry.clone());
                popped.push(entry);
            }
        }
        popped
    }

    /// Replaces the current top destination with a new key and clears forward history.
    pub fn replace_top(&mut self, key: K) -> Option<K> {
        self.forward_entries.clear();
        let prev = self.entries.pop();
        self.entries.push(key);
        prev
    }

    /// Replaces the entire back stack with the given entries and clears forward history.
    pub fn set_stack(&mut self, entries: Vec<K>) {
        self.forward_entries.clear();
        self.entries = entries;
    }

    /// Applies a [`NavAction`] mutation directly to this stack.
    pub fn apply(&mut self, action: NavAction<K>) {
        match action {
            NavAction::Push(key) => self.push(key),
            NavAction::Pop => {
                self.pop();
            }
            NavAction::Forward => {
                self.go_forward();
            }
            NavAction::PopToRoot => {
                self.pop_to_root();
            }
            NavAction::ReplaceTop(key) => {
                self.replace_top(key);
            }
        }
    }
}

#[cfg(feature = "animated-transitions")]
impl<K: Clone + PartialEq> NavStack<K> {
    /// Pushes a new destination key onto the stack and explicitly triggers a push transition.
    pub fn push_animated(&mut self, key: K, transition: &mut NavTransition<K>) {
        if let Some(outgoing) = self.top().cloned() {
            transition.trigger_push(outgoing, key.clone());
        }
        self.push(key);
    }

    /// Pops the active destination from the stack and explicitly triggers a pop transition (Undo navigation).
    pub fn pop_animated(&mut self, transition: &mut NavTransition<K>) -> Option<K> {
        let outgoing = self.top().cloned();
        let popped = self.pop();
        if let (Some(out), Some(inc)) = (outgoing, self.top().cloned()) {
            transition.trigger_pop(out, inc);
        }
        popped
    }

    /// Alias for [`pop_animated`](Self::pop_animated).
    pub fn go_back_animated(&mut self, transition: &mut NavTransition<K>) -> Option<K> {
        self.pop_animated(transition)
    }

    /// Restores the next forward destination from history and explicitly triggers a push transition (Redo navigation).
    pub fn go_forward_animated(&mut self, transition: &mut NavTransition<K>) -> Option<K> {
        let outgoing = self.top().cloned();
        let restored = self.go_forward();
        if let (Some(out), Some(inc)) = (outgoing, self.top().cloned()) {
            transition.trigger_push(out, inc);
        }
        restored
    }

    /// Pops destinations until predicate matches and explicitly triggers a pop transition.
    pub fn pop_to_animated(
        &mut self,
        predicate: impl Fn(&K) -> bool,
        transition: &mut NavTransition<K>,
    ) -> Vec<K> {
        let outgoing = self.top().cloned();
        let popped = self.pop_to(predicate);
        if let (Some(out), Some(inc)) = (outgoing, self.top().cloned()) {
            if out != inc {
                transition.trigger_pop(out, inc);
            }
        }
        popped
    }

    /// Pops all destinations above root and explicitly triggers a pop transition.
    pub fn pop_to_root_animated(&mut self, transition: &mut NavTransition<K>) -> Vec<K> {
        let outgoing = self.top().cloned();
        let popped = self.pop_to_root();
        if let (Some(out), Some(inc)) = (outgoing, self.top().cloned()) {
            if out != inc {
                transition.trigger_pop(out, inc);
            }
        }
        popped
    }

    /// Replaces the current top destination and explicitly triggers a transition.
    pub fn replace_top_animated(
        &mut self,
        key: K,
        transition: &mut NavTransition<K>,
    ) -> Option<K> {
        let outgoing = self.top().cloned();
        let replaced = self.replace_top(key.clone());
        if let Some(out) = outgoing {
            transition.trigger_push(out, key);
        }
        replaced
    }

    /// Applies a [`NavAction`] and explicitly triggers the corresponding transition.
    pub fn apply_animated(&mut self, action: NavAction<K>, transition: &mut NavTransition<K>) {
        match action {
            NavAction::Push(key) => self.push_animated(key, transition),
            NavAction::Pop => {
                self.pop_animated(transition);
            }
            NavAction::Forward => {
                self.go_forward_animated(transition);
            }
            NavAction::PopToRoot => {
                self.pop_to_root_animated(transition);
            }
            NavAction::ReplaceTop(key) => {
                self.replace_top_animated(key, transition);
            }
        }
    }
}
