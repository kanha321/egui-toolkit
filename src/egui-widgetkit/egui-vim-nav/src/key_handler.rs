//! Vim-style key handler mapping HJKL, arrow keys, Ctrl+HJKL for sections, and action keys (F/Enter, D, Q/Esc, Mouse Back/Forward).
//!
//! [`VimKeyHandler`] reads egui input state and produces navigation events and
//! action events. Key reads are **unconditional** — they always fire regardless
//! of egui's internal text-input focus. The application layer is responsible for
//! suppressing navigation when the user is actively typing in a text field.
//!
//! # State ownership
//!
//! `VimKeyHandler` is a lightweight configuration struct. It holds no mutable
//! state across frames — it reads input and produces events statelessly.

use std::fmt::Debug;
use std::hash::Hash;

use egui::Context;

use super::focus_graph::{Direction, FocusGraph};
use super::navigator::{FocusEvent, Navigator};

/// High-level user actions triggered via Vim navigation keys or mouse buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VimAction {
    /// Primary Click / Enter / Select (triggered by 'F', 'Enter', or primary mouse click).
    PrimaryClick,
    /// Secondary Click / Context Action / Alternate (triggered by 'D' or secondary mouse click).
    SecondaryClick,
    /// Back / Cancel / Undo (triggered by 'Q', 'Escape', or mouse lower thumb button `PointerButton::Extra1`).
    Back,
    /// Forward / Redo (triggered by mouse upper thumb button `PointerButton::Extra2`).
    Forward,
    /// Legacy alias for `PrimaryClick`.
    Enter,
}

/// Configuration for which key bindings are active.
///
/// Built via chained setters:
/// ```ignore
/// let handler = VimKeyHandler::new()
///     .with_hjkl(true)
///     .with_arrows(true)
///     .with_ctrl_sections(true)
///     .with_tab(false)
///     .with_actions(true);
/// ```
#[derive(Clone, Debug)]
pub struct VimKeyHandler {
    /// Whether HJKL keys trigger navigation (H=Left, J=Down, K=Up, L=Right).
    hjkl_enabled: bool,
    /// Whether arrow keys trigger navigation.
    arrows_enabled: bool,
    /// Whether Ctrl + HJKL / Ctrl + Arrows trigger section-level navigation.
    ctrl_sections_enabled: bool,
    /// Whether Tab / Shift+Tab cycle focus sequentially. Default is `false` to
    /// preserve pure Vim navigation and avoid interfering with UI focus rings.
    tab_enabled: bool,
    /// Whether action keys are enabled ('F'/Enter for PrimaryClick, 'D' for SecondaryClick, 'Q'/Escape/MouseBack for Back).
    actions_enabled: bool,
}

impl Default for VimKeyHandler {
    fn default() -> Self {
        Self {
            hjkl_enabled: true,
            arrows_enabled: true,
            ctrl_sections_enabled: true,
            tab_enabled: false,
            actions_enabled: true,
        }
    }
}

impl VimKeyHandler {
    /// Creates a handler with default bindings (HJKL + Arrows + Ctrl-sections + Actions enabled, Tab disabled).
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables HJKL key bindings.
    pub fn with_hjkl(mut self, enabled: bool) -> Self {
        self.hjkl_enabled = enabled;
        self
    }

    /// Enables or disables arrow key bindings.
    pub fn with_arrows(mut self, enabled: bool) -> Self {
        self.arrows_enabled = enabled;
        self
    }

    /// Enables or disables Ctrl + H/J/K/L section navigation.
    pub fn with_ctrl_sections(mut self, enabled: bool) -> Self {
        self.ctrl_sections_enabled = enabled;
        self
    }

    /// Enables or disables Tab / Shift+Tab focus cycling.
    pub fn with_tab(mut self, enabled: bool) -> Self {
        self.tab_enabled = enabled;
        self
    }

    /// Enables or disables action key bindings ('F'/Enter for PrimaryClick, 'D' for SecondaryClick, 'Q'/Escape/MouseBack for Back).
    pub fn with_actions(mut self, enabled: bool) -> Self {
        self.actions_enabled = enabled;
        self
    }

    /// Reads the current frame's input and moves focus within the given focus graph
    /// if a standard navigation key (`HJKL` or `Arrows` without Ctrl) was pressed.
    ///
    /// Returns a [`FocusEvent`] if focus actually changed, or `None` if no
    /// navigation key was pressed, a text field has focus, or focus couldn't
    /// Reads the current frame's input and returns the requested navigation direction (without Ctrl).
    ///
    /// **HJKL and Arrow keys are always read** from egui input regardless of text-input focus.
    /// If you have text input widgets, suppress navigation at the call site when the user
    /// is actively typing (e.g. using a `text_field_focused` boolean set from the `TextEdit` response).
    pub fn get_nav_direction(&self, ctx: &Context) -> Option<Direction> {
        ctx.input(|i| {
            // Standard intra-section navigation (only without Ctrl)
            if i.modifiers.ctrl {
                return None;
            }

            // Tab / Shift+Tab — mapped to Right / Left (only if explicitly enabled)
            if self.tab_enabled {
                if i.key_pressed(egui::Key::Tab) {
                    if i.modifiers.shift {
                        return Some(Direction::Left);
                    } else {
                        return Some(Direction::Right);
                    }
                }
            }

            // Shift modifier is reserved for element-specific actions (e.g. Shift+H/L slider steps)
            if i.modifiers.shift {
                return None;
            }

            // HJKL
            if self.hjkl_enabled {
                if i.key_pressed(egui::Key::H) {
                    return Some(Direction::Left);
                }
                if i.key_pressed(egui::Key::J) {
                    return Some(Direction::Down);
                }
                if i.key_pressed(egui::Key::K) {
                    return Some(Direction::Up);
                }
                if i.key_pressed(egui::Key::L) {
                    return Some(Direction::Right);
                }
            }

            // Arrow keys
            if self.arrows_enabled {
                if i.key_pressed(egui::Key::ArrowLeft) {
                    return Some(Direction::Left);
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    return Some(Direction::Down);
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    return Some(Direction::Up);
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    return Some(Direction::Right);
                }
            }

            None
        })
    }

    /// Reads the current frame's input and moves focus within the given focus graph
    /// if a standard navigation key (`HJKL` or `Arrows` without Ctrl) was pressed.
    ///
    /// Returns a [`FocusEvent`] if focus actually changed, or `None` if no
    /// navigation key was pressed or focus couldn't move (e.g. at a graph edge).
    pub fn handle_input<T: Clone + Eq + Hash + Debug>(
        &self,
        ctx: &Context,
        nav: &mut Navigator<T>,
        graph: &FocusGraph<T>,
    ) -> Option<FocusEvent<T>> {
        self.get_nav_direction(ctx).and_then(|d| nav.move_focus(graph, d))
    }

    /// Reads the current frame's input and checks for section navigation keys
    /// (`Ctrl + H/J/K/L` or `Ctrl + Arrows`).
    ///
    /// Returns `Some(Direction)` if a section navigation shortcut was pressed, or `None`.
    pub fn handle_section_input(&self, ctx: &Context) -> Option<Direction> {
        if !self.ctrl_sections_enabled {
            return None;
        }

        ctx.input(|i| {
            if !i.modifiers.ctrl {
                return None;
            }

            if self.hjkl_enabled {
                if i.key_pressed(egui::Key::H) {
                    return Some(Direction::Left);
                }
                if i.key_pressed(egui::Key::J) {
                    return Some(Direction::Down);
                }
                if i.key_pressed(egui::Key::K) {
                    return Some(Direction::Up);
                }
                if i.key_pressed(egui::Key::L) {
                    return Some(Direction::Right);
                }
            }

            if self.arrows_enabled {
                if i.key_pressed(egui::Key::ArrowLeft) {
                    return Some(Direction::Left);
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    return Some(Direction::Down);
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    return Some(Direction::Up);
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    return Some(Direction::Right);
                }
            }

            None
        })
    }

    /// Convenience helper that navigates an inter-section [`FocusGraph`] using `Ctrl + H/J/K/L`.
    pub fn handle_section_nav<T: Clone + Eq + Hash + Debug>(
        &self,
        ctx: &Context,
        section_nav: &mut Navigator<T>,
        section_graph: &FocusGraph<T>,
    ) -> Option<FocusEvent<T>> {
        self.handle_section_input(ctx)
            .and_then(|dir| section_nav.move_focus(section_graph, dir))
    }

    /// Checks if a Vim action key or mouse button was triggered in the current frame.
    ///
    /// Mouse thumb buttons work globally:
    /// - `PointerButton::Extra1` (Lower Thumb) -> `VimAction::Back`
    /// - `PointerButton::Extra2` (Upper Thumb) -> `VimAction::Forward`
    ///
    /// **All action keys always fire** regardless of egui's text-input state.
    /// If you have text input widgets, suppress actions at the call site when the user
    /// is actively typing (e.g. using a `text_field_focused` boolean set from the `TextEdit` response).
    pub fn handle_action(&self, ctx: &Context) -> Option<VimAction> {
        if !self.actions_enabled {
            return None;
        }

        // Lower thumb button (Button 4 / Back)
        if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra1)) {
            return Some(VimAction::Back);
        }

        // Upper thumb button (Button 5 / Forward)
        if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra2)) {
            return Some(VimAction::Forward);
        }

        ctx.input(|i| {
            if i.modifiers.ctrl {
                return None;
            }

            if i.key_pressed(egui::Key::Escape) {
                Some(VimAction::Back)
            } else if i.key_pressed(egui::Key::F) || i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space) {
                Some(VimAction::PrimaryClick)
            } else if i.key_pressed(egui::Key::D) {
                Some(VimAction::SecondaryClick)
            } else if i.key_pressed(egui::Key::Q) {
                Some(VimAction::Back)
            } else {
                None
            }
        })
    }
}
