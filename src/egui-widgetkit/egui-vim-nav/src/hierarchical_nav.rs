//! 2-pass hierarchical navigation for sectioned UIs.
//!
//! In a typical vim-navigated dashboard, widgets are grouped into sections (cards,
//! panels, regions). Navigation works in two passes:
//!
//! 1. **Intra-section (Pass 1)**: Try moving focus within the current section's
//!    widget graph. If the result stays in the same section, accept it.
//! 2. **Cross-section fallback (Pass 2)**: If Pass 1 fails or would cross a section
//!    boundary, try the section-level graph and resolve the entry widget in the
//!    target section via a caller-supplied callback.
//!
//! This module provides [`hierarchical_move`], a stateless function that performs
//! both passes. The consuming app owns all state ([`Navigator`], [`FocusGraph`])
//! and supplies the topology callbacks.
//!
//! # State Ownership
//!
//! This module introduces no persistent state. It mutates the `Navigator`s passed
//! to it and returns a [`HierarchicalNavResult`] describing what happened.

use std::fmt::Debug;
use std::hash::Hash;

use crate::focus_graph::{Direction, FocusGraph};
use crate::navigator::Navigator;

/// Result of a successful hierarchical navigation.
///
/// Contains the widget and section that now have focus, whether a section
/// boundary was crossed, and the navigation direction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HierarchicalNavResult<W, S> {
    /// The widget that now has focus after navigation.
    pub focused_widget: W,
    /// The section that the focused widget belongs to.
    pub focused_section: S,
    /// Whether focus crossed a section boundary (Pass 2 was used).
    pub crossed_section: bool,
    /// The direction that was navigated.
    pub direction: Direction,
}

/// Performs 2-pass hierarchical navigation.
///
/// This is a **stateless algorithm** — it reads from and writes to the provided
/// `Navigator`/`FocusGraph` pairs, and returns a result describing what happened.
///
/// # Arguments
///
/// - `dir`: The navigation direction (from HJKL or arrow keys).
/// - `widget_nav`: Mutable reference to the widget-level navigator.
/// - `widget_graph`: The widget-level focus graph (intra-section connections).
/// - `section_nav`: Mutable reference to the section-level navigator.
/// - `section_graph`: The section-level focus graph (inter-section connections).
/// - `widget_to_section`: Maps any widget ID to its parent section ID.
/// - `section_entry_widget`: Given a target section, the navigation direction,
///   and the last-remembered widget in that section, returns the appropriate
///   entry widget. For horizontal transitions this typically restores the last
///   active widget; for vertical transitions it picks the edge-most widget.
///
/// # Returns
///
/// - `Some(result)` if focus moved (either within a section or across sections).
/// - `None` if no movement was possible (at the edge of both graphs).
///
/// # Pass 1 (Intra-Section)
///
/// Tries `widget_nav.move_focus(widget_graph, dir)`. If the resulting widget
/// is in the **same section** as the current widget, accepts the move. If it
/// would cross a section boundary (the widget graph has edges between sections),
/// the move is rolled back and Pass 2 is attempted.
///
/// # Pass 2 (Cross-Section Fallback)
///
/// Tries `section_nav.move_focus(section_graph, dir)`. If successful, resolves
/// the entry widget in the target section via `section_entry_widget` and focuses
/// it in the widget navigator.
pub fn hierarchical_move<W, S>(
    dir: Direction,
    widget_nav: &mut Navigator<W>,
    widget_graph: &FocusGraph<W>,
    section_nav: &mut Navigator<S>,
    section_graph: &FocusGraph<S>,
    widget_to_section: impl Fn(&W) -> S,
    section_entry_widget: impl Fn(&S, Direction) -> W,
) -> Option<HierarchicalNavResult<W, S>>
where
    W: Clone + Eq + Hash + Debug,
    S: Clone + Eq + Hash + Debug,
{
    let current_widget = widget_nav.focused().cloned()?;
    let current_section = widget_to_section(&current_widget);

    // ── Pass 1: Try intra-section movement ──
    if let Some(event) = widget_nav.move_focus(widget_graph, dir) {
        let new_section = widget_to_section(&event.current);
        if new_section == current_section {
            // Stayed within the same section — accept
            return Some(HierarchicalNavResult {
                focused_widget: event.current,
                focused_section: current_section,
                crossed_section: false,
                direction: dir,
            });
        }
        // Crossed a section boundary via widget graph edge — roll back
        widget_nav.set_focus_with_graph(Some(current_widget.clone()), widget_graph);
    }

    // ── Pass 2: Cross-section fallback ──
    if let Some(sec_event) = section_nav.move_focus(section_graph, dir) {
        let target_section = sec_event.current;
        let entry = section_entry_widget(&target_section, dir);
        widget_nav.set_focus_with_graph(Some(entry.clone()), widget_graph);

        return Some(HierarchicalNavResult {
            focused_widget: entry,
            focused_section: target_section,
            crossed_section: true,
            direction: dir,
        });
    }

    // Both passes failed — at the edge of the graph
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple widget IDs for testing.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum W {
        A1,
        A2,
        B1,
        B2,
    }

    /// Simple section IDs for testing.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum S {
        SectionA,
        SectionB,
    }

    fn widget_to_section(w: &W) -> S {
        match w {
            W::A1 | W::A2 => S::SectionA,
            W::B1 | W::B2 => S::SectionB,
        }
    }

    fn section_entry(sec: &S, dir: Direction) -> W {
        match (sec, dir) {
            (S::SectionA, Direction::Up) => W::A2,   // enter from bottom
            (S::SectionA, Direction::Down) => W::A1,  // enter from top
            (S::SectionA, _) => W::A1,
            (S::SectionB, Direction::Up) => W::B2,
            (S::SectionB, Direction::Down) => W::B1,
            (S::SectionB, _) => W::B1,
        }
    }

    fn setup() -> (FocusGraph<W>, Navigator<W>, FocusGraph<S>, Navigator<S>) {
        let mut wg = FocusGraph::new();
        wg.connect_vertical(W::A1, W::A2); // Section A: A1 ↕ A2
        wg.connect_vertical(W::B1, W::B2); // Section B: B1 ↕ B2

        let mut sg = FocusGraph::new();
        sg.connect_vertical(S::SectionA, S::SectionB); // A ↕ B

        let wn = Navigator::new().with_initial_focus(W::A1);
        let sn = Navigator::new().with_initial_focus(S::SectionA);

        (wg, wn, sg, sn)
    }

    #[test]
    fn intra_section_movement() {
        let (wg, mut wn, sg, mut sn) = setup();

        let result = hierarchical_move(
            Direction::Down,
            &mut wn, &wg,
            &mut sn, &sg,
            widget_to_section,
            section_entry,
        );

        let r = result.expect("should move down within section A");
        assert_eq!(r.focused_widget, W::A2);
        assert_eq!(r.focused_section, S::SectionA);
        assert!(!r.crossed_section);
        assert_eq!(r.direction, Direction::Down);
    }

    #[test]
    fn cross_section_fallback() {
        let (wg, mut wn, sg, mut sn) = setup();

        // Move to A2 first (bottom of section A)
        wn.move_focus(&wg, Direction::Down);

        // Now move down again — should cross to section B
        let result = hierarchical_move(
            Direction::Down,
            &mut wn, &wg,
            &mut sn, &sg,
            widget_to_section,
            section_entry,
        );

        let r = result.expect("should cross to section B");
        assert_eq!(r.focused_widget, W::B1); // enters section B from top
        assert_eq!(r.focused_section, S::SectionB);
        assert!(r.crossed_section);
    }

    #[test]
    fn edge_of_graph_returns_none() {
        let (wg, mut wn, sg, mut sn) = setup();

        // Try moving up from A1 — at the top of everything
        let result = hierarchical_move(
            Direction::Up,
            &mut wn, &wg,
            &mut sn, &sg,
            widget_to_section,
            section_entry,
        );

        assert!(result.is_none(), "should return None at graph edge");
        // Focus should remain unchanged
        assert_eq!(wn.focused(), Some(&W::A1));
    }

    #[test]
    fn no_focus_returns_none() {
        let (wg, sg, mut sn) = {
            let mut wg = FocusGraph::new();
            wg.connect_vertical(W::A1, W::A2);
            let mut sg = FocusGraph::new();
            sg.connect_vertical(S::SectionA, S::SectionB);
            let sn = Navigator::new().with_initial_focus(S::SectionA);
            (wg, sg, sn)
        };
        let mut wn: Navigator<W> = Navigator::new(); // No initial focus

        let result = hierarchical_move(
            Direction::Down,
            &mut wn, &wg,
            &mut sn, &sg,
            widget_to_section,
            section_entry,
        );

        assert!(result.is_none(), "should return None with no focus");
    }

    #[test]
    fn cross_section_with_graph_edge_between_widgets() {
        // Test the case where widget_graph has an edge crossing sections
        // (e.g., A2 → B1 directly). hierarchical_move should detect this
        // and use section graph instead.
        let mut wg = FocusGraph::new();
        wg.connect_vertical(W::A1, W::A2);
        wg.connect_vertical(W::A2, W::B1); // cross-section edge in widget graph
        wg.connect_vertical(W::B1, W::B2);

        let mut sg = FocusGraph::new();
        sg.connect_vertical(S::SectionA, S::SectionB);

        let mut wn = Navigator::new().with_initial_focus(W::A2);
        let mut sn = Navigator::new().with_initial_focus(S::SectionA);

        let result = hierarchical_move(
            Direction::Down,
            &mut wn, &wg,
            &mut sn, &sg,
            widget_to_section,
            section_entry,
        );

        let r = result.expect("should cross section via section graph");
        // Should use section_entry_widget, not the raw widget graph edge
        assert_eq!(r.focused_widget, W::B1);
        assert_eq!(r.focused_section, S::SectionB);
        assert!(r.crossed_section);
    }
}
