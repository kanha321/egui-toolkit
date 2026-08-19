//! App-owned back-stack data structure for screen navigation.

/// An app-owned back stack of navigation destinations/screens.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavStack<K> {
    entries: Vec<K>,
}
