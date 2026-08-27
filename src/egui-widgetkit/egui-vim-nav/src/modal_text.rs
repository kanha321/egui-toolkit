//! Modal text focus state machine for 2-tier UI navigation.
//!
//! In a vim-navigated UI, there are two levels of keyboard control:
//!
//! - **Level 1 (Navigation)**: HJKL moves focus between widgets. Keys like `i`, `a`,
//!   Enter, Space, or `f` on a text-capable widget transition to Level 2.
//! - **Level 2 (Text Editing)**: Keys go to the active `VimBuffer`. Pressing Escape
//!   in clean Normal mode (no pending operator) transitions back to Level 1.
//!
//! This module provides a pure, deterministic state machine for those transitions.
//! The consuming app owns the [`FocusLevel`] enum as a single field and calls
//! [`check_modal_transition`] each frame to determine whether a transition occurred.
//!
//! # State Ownership
//!
//! This module introduces no persistent state of its own. [`FocusLevel`] is a plain
//! enum the consuming app stores. [`check_modal_transition`] is a pure function that
//! reads egui input state and returns a [`ModalTransition`] describing what happened.
//!
//! # Example
//!
//! ```rust,ignore
//! use egui_vim_nav::modal_text::{FocusLevel, ModalTransition, check_modal_transition};
//!
//! let transition = check_modal_transition(
//!     state.focus_level,
//!     is_on_text_widget,
//!     is_in_clean_normal_mode,
//!     slider_externally_exited,
//!     section_nav_triggered,
//!     &ctx,
//! );
//! match transition {
//!     ModalTransition::EnteredText => {
//!         state.focus_level = FocusLevel::TextEditing;
//!         // App-specific: set the right VimBuffer to Insert mode
//!     }
//!     ModalTransition::ExitedText => {
//!         state.focus_level = FocusLevel::Navigation;
//!         // App-specific: reset all buffers to Normal
//!     }
//!     ModalTransition::None => {}
//! }
//! ```

/// The two focus levels in a vim-navigated UI.
///
/// The consuming application owns this value as a plain field and updates it
/// based on [`ModalTransition`] results each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FocusLevel {
    /// HJKL navigates between widgets. Text inputs are not active.
    /// Pressing `i`, `a`, Enter, Space, or `f` on a text-capable widget
    /// transitions to [`FocusLevel::TextEditing`].
    #[default]
    Navigation,
    /// Keys go to the active text buffer (VimBuffer). Pressing Escape
    /// in clean Normal mode (no pending operator/keys) transitions back
    /// to [`FocusLevel::Navigation`].
    TextEditing,
}

impl FocusLevel {
    /// Returns `true` if currently in navigation mode.
    #[inline]
    pub fn is_navigation(self) -> bool {
        self == FocusLevel::Navigation
    }

    /// Returns `true` if currently in text editing mode.
    #[inline]
    pub fn is_text_editing(self) -> bool {
        self == FocusLevel::TextEditing
    }
}

/// Describes the result of a modal focus transition check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModalTransition {
    /// No transition occurred. Stay in the current [`FocusLevel`].
    None,
    /// Entered text editing mode (`Navigation` → `TextEditing`).
    ///
    /// The app should set the appropriate VimBuffer to Insert mode and
    /// position the cursor.
    EnteredText,
    /// Exited text editing mode (`TextEditing` → `Navigation`).
    ///
    /// The app should reset all VimBuffers to Normal mode and call
    /// `ctx.memory_mut(|m| m.stop_text_input())`.
    ExitedText,
}

impl ModalTransition {
    /// Returns `true` if this transition entered text editing mode.
    #[inline]
    pub fn entered(self) -> bool {
        self == ModalTransition::EnteredText
    }

    /// Returns `true` if this transition exited text editing mode.
    #[inline]
    pub fn exited(self) -> bool {
        self == ModalTransition::ExitedText
    }

    /// Returns `true` if no transition occurred.
    #[inline]
    pub fn is_none(self) -> bool {
        self == ModalTransition::None
    }
}

/// Checks whether the current frame should transition between focus levels.
///
/// This is a **pure decision function** — it reads egui input state but does not
/// mutate any state. The consuming app applies the returned [`ModalTransition`]
/// by updating its own [`FocusLevel`] field and performing app-specific actions
/// (setting VimBuffer modes, stopping text input, etc.).
///
/// # Arguments
///
/// - `current_level`: the app's current [`FocusLevel`].
/// - `is_on_text_widget`: whether the currently focused widget supports text editing
///   (e.g., a text input, slider with edit mode, or any widget that accepts keyboard text).
/// - `externally_entered`: whether the widget entered text editing through a non-keyboard
///   mechanism (e.g., a slider badge was clicked to start editing). When `true` and the
///   current level is `Navigation`, this triggers an immediate `EnteredText` transition
///   without requiring a keypress.
/// - `is_in_clean_normal_mode`: whether the active VimBuffer is in Normal mode
///   with no pending operator or key sequence. This determines whether Escape should
///   exit text editing entirely (clean Normal → Navigation) or just return from
///   Insert/Visual to Normal (handled by the VimBuffer itself, not this function).
/// - `externally_exited`: whether the widget exited text editing through a non-keyboard
///   mechanism (e.g., a slider's editing flag was cleared by clicking outside). When `true`,
///   this triggers an immediate `ExitedText` transition regardless of key state.
/// - `section_nav_triggered`: whether a Ctrl+HJKL section jump was detected. Section
///   navigation always exits text editing immediately — the user is expressing intent
///   to leave the current widget entirely.
/// - `ctx`: egui context for reading key input.
///
/// # Transition Rules
///
/// | Current Level | Condition | Result |
/// |---|---|---|
/// | `Navigation` | Not on a text widget | `None` (also: app should force-clear text_focused) |
/// | `Navigation` | `externally_entered` is `true` | `EnteredText` |
/// | `Navigation` | `i`/`a`/Enter/Space/`f` pressed (no Ctrl/Alt) | `EnteredText` |
/// | `Navigation` | No trigger key pressed | `None` |
/// | `TextEditing` | `externally_exited` is `true` | `ExitedText` |
/// | `TextEditing` | `section_nav_triggered` is `true` | `ExitedText` |
/// | `TextEditing` | Escape pressed AND `is_in_clean_normal_mode` | `ExitedText` |
/// | `TextEditing` | Escape pressed but NOT clean Normal | `None` (VimBuffer handles it) |
/// | `TextEditing` | No exit condition | `None` |
pub fn check_modal_transition(
    current_level: FocusLevel,
    is_on_text_widget: bool,
    externally_entered: bool,
    is_in_clean_normal_mode: bool,
    externally_exited: bool,
    section_nav_triggered: bool,
    ctx: &egui::Context,
) -> ModalTransition {
    match current_level {
        FocusLevel::Navigation => {
            // Not on a text widget — no transition possible
            if !is_on_text_widget {
                return ModalTransition::None;
            }

            // External trigger (e.g., slider badge click started editing)
            if externally_entered {
                return ModalTransition::EnteredText;
            }

            // Check keyboard triggers for entering text editing
            let enter_text = ctx.input(|i| {
                !i.modifiers.ctrl
                    && !i.modifiers.alt
                    && (i.key_pressed(egui::Key::I)
                        || i.key_pressed(egui::Key::A)
                        || i.key_pressed(egui::Key::Enter)
                        || i.key_pressed(egui::Key::Space)
                        || i.key_pressed(egui::Key::F))
            });

            if enter_text {
                ModalTransition::EnteredText
            } else {
                ModalTransition::None
            }
        }

        FocusLevel::TextEditing => {
            // External exit (e.g., slider editing finished by clicking outside)
            if externally_exited {
                return ModalTransition::ExitedText;
            }

            // Section navigation always exits text editing
            if section_nav_triggered {
                return ModalTransition::ExitedText;
            }

            // 2-Tier Escape handling:
            // - If in Insert/Visual/Replace/OperatorPending → VimBuffer handles Escape
            //   (transitions to Normal mode internally). We return None.
            // - If already in clean Normal mode → Escape exits text editing entirely.
            let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
            if esc_pressed && is_in_clean_normal_mode {
                ModalTransition::ExitedText
            } else {
                ModalTransition::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a minimal egui context for testing.
    fn test_ctx() -> egui::Context {
        let ctx = egui::Context::default();
        // Run a frame so input state is initialized
        let raw_input = egui::RawInput::default();
        let _ = ctx.run(raw_input, |_ctx| {});
        ctx
    }

    #[test]
    fn navigation_on_non_text_widget_returns_none() {
        let ctx = test_ctx();
        let result = check_modal_transition(
            FocusLevel::Navigation,
            false, // not on text widget
            false,
            false,
            false,
            false,
            &ctx,
        );
        assert_eq!(result, ModalTransition::None);
    }

    #[test]
    fn navigation_external_enter_triggers_entered_text() {
        let ctx = test_ctx();
        let result = check_modal_transition(
            FocusLevel::Navigation,
            true,  // on text widget
            true,  // externally entered (e.g. slider badge click)
            false,
            false,
            false,
            &ctx,
        );
        assert_eq!(result, ModalTransition::EnteredText);
    }

    #[test]
    fn text_editing_external_exit_triggers_exited_text() {
        let ctx = test_ctx();
        let result = check_modal_transition(
            FocusLevel::TextEditing,
            true,
            false,
            false,
            true,  // externally exited (e.g. slider editing finished)
            false,
            &ctx,
        );
        assert_eq!(result, ModalTransition::ExitedText);
    }

    #[test]
    fn text_editing_section_nav_triggers_exit() {
        let ctx = test_ctx();
        let result = check_modal_transition(
            FocusLevel::TextEditing,
            true,
            false,
            false,
            false,
            true, // section nav triggered
            &ctx,
        );
        assert_eq!(result, ModalTransition::ExitedText);
    }

    #[test]
    fn text_editing_no_exit_condition_returns_none() {
        let ctx = test_ctx();
        let result = check_modal_transition(
            FocusLevel::TextEditing,
            true,
            false,
            false, // not in clean normal mode
            false,
            false,
            &ctx,
        );
        assert_eq!(result, ModalTransition::None);
    }

    #[test]
    fn navigation_no_trigger_key_returns_none() {
        let ctx = test_ctx();
        // No keys pressed, on a text widget but no trigger
        let result = check_modal_transition(
            FocusLevel::Navigation,
            true, // on text widget
            false,
            false,
            false,
            false,
            &ctx,
        );
        assert_eq!(result, ModalTransition::None);
    }

    #[test]
    fn focus_level_helper_methods() {
        assert!(FocusLevel::Navigation.is_navigation());
        assert!(!FocusLevel::Navigation.is_text_editing());
        assert!(FocusLevel::TextEditing.is_text_editing());
        assert!(!FocusLevel::TextEditing.is_navigation());
    }

    #[test]
    fn modal_transition_helper_methods() {
        assert!(ModalTransition::None.is_none());
        assert!(!ModalTransition::None.entered());
        assert!(!ModalTransition::None.exited());

        assert!(ModalTransition::EnteredText.entered());
        assert!(!ModalTransition::EnteredText.exited());
        assert!(!ModalTransition::EnteredText.is_none());

        assert!(ModalTransition::ExitedText.exited());
        assert!(!ModalTransition::ExitedText.entered());
        assert!(!ModalTransition::ExitedText.is_none());
    }

    #[test]
    fn default_focus_level_is_navigation() {
        assert_eq!(FocusLevel::default(), FocusLevel::Navigation);
    }
}
