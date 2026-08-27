//! Mouse cursor auto-hiding for keyboard and Vim-navigated applications.
//!
//! Provides [`CursorAutohide`], a lightweight state manager that hides the mouse cursor
//! (via `ctx.set_cursor_icon(egui::CursorIcon::None)`) when the user interacts via keyboard,
//! and automatically restores it when the mouse pointer moves or clicks.
//!
//! # State Ownership
//!
//! Like all `egui-vim-nav` types, [`CursorAutohide`] is a plain struct owned by the
//! consuming application and updated each frame via `cursor_autohide.update(ctx)`.

use egui::{Context, CursorIcon, Event, Vec2};

/// Manages mouse cursor visibility during keyboard navigation and text entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CursorAutohide {
    /// Whether the cursor is currently hidden due to keyboard activity.
    pub hidden: bool,
}

/// Unique Context data storage ID for cursor autohide state.
pub const CURSOR_HIDDEN_ID: &str = "_egui_cursor_autohide_hidden";

/// Returns `true` if the mouse cursor has been hidden by [`CursorAutohide`].
pub fn is_cursor_hidden(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(CURSOR_HIDDEN_ID))).unwrap_or(false)
}

/// Returns `true` if mouse hover effects should be active (i.e. cursor is not hidden).
pub fn is_hover_active(ctx: &Context) -> bool {
    !is_cursor_hidden(ctx)
}

impl CursorAutohide {
    /// Creates a new `CursorAutohide` instance with the cursor initially visible.
    pub fn new() -> Self {
        Self { hidden: false }
    }

    /// Sets the initial hidden state.
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Returns `true` if the mouse cursor is currently hidden.
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Manually sets whether the cursor should be hidden.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    /// Evaluates input events for the current frame and updates cursor visibility:
    /// - Pointer movement or button clicks immediately unhide the cursor.
    /// - Key presses or text input immediately hide the cursor.
    ///
    /// When hidden, completely removes the cursor:
    /// - Sends `ViewportCommand::CursorVisible(false)` to hide the native OS hardware cursor.
    /// - Sets `ctx.output_mut(|o| o.cursor_icon = CursorIcon::None)`.
    /// - Emits `Event::PointerGone` so egui clears hover interaction.
    /// - Stores cursor hidden flag in `ctx.data_mut` for widgets to suppress hover states.
    pub fn update(&mut self, ctx: &Context) {
        let prev_hidden = self.hidden;

        ctx.input(|i| {
            let pointer_active = i.pointer.delta() != Vec2::ZERO
                || i.pointer.any_click()
                || i.pointer.any_down();

            if pointer_active {
                self.hidden = false;
            } else {
                let has_keyboard_activity = !i.events.is_empty() && i.events.iter().any(|e| {
                    matches!(e, Event::Key { pressed: true, .. } | Event::Text(_))
                });

                if has_keyboard_activity {
                    self.hidden = true;
                }
            }
        });

        // Store state in ctx.data for library widgets
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new(CURSOR_HIDDEN_ID), self.hidden);
        });

        if self.hidden {
            // 1. Native OS level: hide hardware cursor
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
            // 2. egui frame output level: force cursor icon to None
            ctx.set_cursor_icon(CursorIcon::None);
            ctx.output_mut(|o| o.cursor_icon = CursorIcon::None);
            // 3. Pointer event level: signal to egui that the pointer is gone
            ctx.input_mut(|i| {
                i.events.push(Event::PointerGone);
            });
        } else if prev_hidden && !self.hidden {
            // Restore native hardware cursor as soon as pointer moves
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_autohide_default() {
        let autohide = CursorAutohide::new();
        assert!(!autohide.is_hidden());
    }

    #[test]
    fn test_cursor_autohide_builder() {
        let autohide = CursorAutohide::new().with_hidden(true);
        assert!(autohide.is_hidden());
    }

    #[test]
    fn test_cursor_autohide_manual_set() {
        let mut autohide = CursorAutohide::new();
        autohide.set_hidden(true);
        assert!(autohide.is_hidden());
        autohide.set_hidden(false);
        assert!(!autohide.is_hidden());
    }

    #[test]
    fn test_egui_ctx_introspection() {
        let ctx = Context::default();
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("cursor_hidden"), true);
        });
        let hidden = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("cursor_hidden"))).unwrap_or(false);
        assert!(hidden);
    }
}
