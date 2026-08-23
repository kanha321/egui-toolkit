//! Immediate-mode display widget rendering the active top of a `NavStack`.

use egui::{Align, Layout, PointerButton, Rect, Response, Sense, Ui, Vec2};
use crate::stack::NavStack;

#[cfg(feature = "animated-transitions")]
use crate::transition::{NavTransition, SlideDirection, TransitionKind};

/// Navigation action returned from `NavDisplay::show` to apply mutations safely after rendering.
///
/// Eliminates borrow-checker conflicts by separating immutable display reading from stack mutation (`CODING_RULES §3`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavAction<K> {
    /// Pushes a new destination screen onto the back stack.
    Push(K),
    /// Pops the active screen from the back stack and moves it to forward history (Back / Undo).
    Pop,
    /// Restores the next destination screen from forward history (Forward / Redo).
    Forward,
    /// Pops all screens down to the root screen (index `0`).
    PopToRoot,
    /// Replaces the active top screen with a new destination.
    ReplaceTop(K),
}

/// The response returned from rendering a [`NavDisplay`].
pub struct NavResponse<K> {
    /// The navigation action requested by the screen closure or mouse navigation during rendering, if any.
    pub action: Option<NavAction<K>>,
    /// The `egui::Response` of the allocated display area.
    pub response: Response,
}

impl<K: Clone> NavResponse<K> {
    /// Applies any contained [`NavAction`] directly to the given `NavStack`, returning the inner `egui::Response`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_nav_stack::{NavStack, NavDisplay, NavAction};
    /// # egui::__run_test_ctx(|ctx| {
    /// # egui::CentralPanel::default().show(ctx, |ui| {
    /// let mut stack = NavStack::new("Home");
    ///
    /// NavDisplay::new(&stack)
    ///     .show(ui, |screen, ui| {
    ///         if ui.button("Details").clicked() {
    ///             Some(NavAction::Push("Details"))
    ///         } else {
    ///             None
    ///         }
    ///     })
    ///     .apply_to(&mut stack);
    /// # });
    /// # });
    /// ```
    pub fn apply_to(self, stack: &mut NavStack<K>) -> Response {
        if let Some(action) = self.action {
            stack.apply(action);
        }
        self.response
    }

    /// Applies any contained [`NavAction`] directly to the stack while explicitly notifying the transition.
    #[cfg(feature = "animated-transitions")]
    pub fn apply_to_animated(
        self,
        stack: &mut NavStack<K>,
        transition: &mut NavTransition<K>,
    ) -> Response
    where
        K: Clone + PartialEq,
    {
        if let Some(action) = self.action {
            stack.apply_animated(action, transition);
        }
        self.response
    }
}

/// An immediate-mode builder widget that renders the active top destination of a [`NavStack`].
pub struct NavDisplay<'a, K> {
    stack: &'a NavStack<K>,
    empty_fallback: Option<Box<dyn FnOnce(&mut Ui) + 'a>>,
    mouse_nav: bool,
    #[cfg(feature = "animated-transitions")]
    transition: Option<&'a mut NavTransition<K>>,
}

impl<'a, K> NavDisplay<'a, K> {
    /// Creates a new `NavDisplay` reading the provided `NavStack`.
    pub fn new(stack: &'a NavStack<K>) -> Self {
        Self {
            stack,
            empty_fallback: None,
            mouse_nav: true,
            #[cfg(feature = "animated-transitions")]
            transition: None,
        }
    }

    /// Enables or disables automatic mouse thumb buttons navigation (`PointerButton::Extra1` for Back, `PointerButton::Extra2` for Forward).
    ///
    /// Default is `true`.
    pub fn mouse_nav(mut self, enabled: bool) -> Self {
        self.mouse_nav = enabled;
        self
    }

    /// Sets a fallback closure to render when the stack is completely empty.
    pub fn empty_fallback(mut self, fallback: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.empty_fallback = Some(Box::new(fallback));
        self
    }

    /// Attaches an app-owned [`NavTransition`] state to render spring-animated directional slide, shrink, fade, and pop overshoot transitions.
    #[cfg(feature = "animated-transitions")]
    pub fn transition(mut self, transition: &'a mut NavTransition<K>) -> Self
    where
        K: Clone + PartialEq,
    {
        self.transition = Some(transition);
        self
    }

    /// Renders the active top screen of the stack, consuming available UI space.
    ///
    /// # Arguments
    ///
    /// - `ui`: The egui UI handle.
    /// - `render_screen`: A closure taking `(&K, &mut Ui)` and returning an optional [`NavAction<K>`].
    ///
    /// # Safe Mutation
    ///
    /// Returns a [`NavResponse<K>`] containing any action produced by the closure or mouse thumb buttons.
    /// The caller can apply this action immediately via `.apply_to(&mut stack)` or `.apply_to_animated(&mut stack, &mut transition)`.
    pub fn show(
        self,
        ui: &mut Ui,
        mut render_screen: impl FnMut(&K, &mut Ui) -> Option<NavAction<K>>,
    ) -> NavResponse<K>
    where
        K: Clone + PartialEq,
    {
        let top = self.stack.top();

        if top.is_none() {
            if let Some(fallback) = self.empty_fallback {
                let available = ui.available_size();
                let (rect, response) = ui.allocate_exact_size(available, Sense::hover());
                let mut child_ui = ui.child_ui(rect, Layout::top_down(Align::Min));
                child_ui.set_clip_rect(child_ui.clip_rect().intersect(rect));
                fallback(&mut child_ui);
                return NavResponse {
                    action: None,
                    response,
                };
            } else {
                let response = ui.allocate_response(Vec2::ZERO, Sense::hover());
                return NavResponse {
                    action: None,
                    response,
                };
            }
        }

        let active_key = top.unwrap();

        // 1. Mouse Thumb Buttons Listening (Button 4 / Extra1: Back, Button 5 / Extra2: Forward)
        let mut mouse_action = None;
        if self.mouse_nav {
            if ui.input(|i| i.pointer.button_clicked(PointerButton::Extra1)) {
                if self.stack.can_go_back() {
                    mouse_action = Some(NavAction::Pop);
                }
            } else if ui.input(|i| i.pointer.button_clicked(PointerButton::Extra2)) {
                if self.stack.can_go_forward() {
                    mouse_action = Some(NavAction::Forward);
                }
            }
        }

        #[cfg(feature = "animated-transitions")]
        if let Some(t) = self.transition {
            // First display frame initialization
            if t.current_screen.is_none() {
                t.current_screen = Some(active_key.clone());
                t.settled = true;
            }

            // Advance transition spring simulation
            if !t.settled {
                let dt = ui.input(|i| i.stable_dt.min(0.1));
                t.spring.update(dt);
                if t.spring.is_settled() {
                    t.previous_screen = None;
                    t.settled = true;
                } else {
                    ui.ctx().request_repaint();
                }
            }

            // Simultaneous dual-layer rendering with spring physics
            if !t.settled && t.previous_screen.is_some() {
                let available = ui.available_size();
                let (total_rect, response) = ui.allocate_exact_size(available, Sense::hover());
                let (w, h) = (total_rect.width(), total_rect.height());
                let progress = t.spring.value(); // Preserves natural spring overshoot oscillation

                // Compute directional slide vector based on entry axis
                let slide_vector = match t.slide_direction {
                    SlideDirection::FromRight => Vec2::new(w, 0.0),
                    SlideDirection::FromLeft => Vec2::new(-w, 0.0),
                    SlideDirection::FromTop => Vec2::new(0.0, -h),
                    SlideDirection::FromBottom => Vec2::new(0.0, h),
                };

                match t.kind {
                    TransitionKind::Push => {
                        // 1. Bottom Layer: Outgoing background screen simultaneously shrinks and fades
                        let scale = (1.0 - (t.shrink_factor * progress.min(1.0))).max(0.5);
                        let bg_rect = Rect::from_center_size(total_rect.center(), total_rect.size() * scale);
                        let mut bg_ui = ui.child_ui(bg_rect, Layout::top_down(Align::Min));
                        bg_ui.set_clip_rect(bg_ui.clip_rect().intersect(total_rect));
                        bg_ui.set_opacity((1.0 - progress).clamp(0.0, 1.0));
                        bg_ui.add_enabled_ui(false, |ui| {
                            if let Some(prev_key) = &t.previous_screen {
                                render_screen(prev_key, ui);
                            }
                        });

                        // 2. Top Layer: New incoming screen simultaneously slides into view with spring overshoot
                        let fg_offset = (1.0 - progress) * slide_vector;
                        let fg_rect = total_rect.translate(fg_offset);
                        let mut fg_ui = ui.child_ui(fg_rect, Layout::top_down(Align::Min));
                        fg_ui.set_clip_rect(fg_ui.clip_rect().intersect(total_rect));
                        let user_action = render_screen(active_key, &mut fg_ui);
                        let action = user_action.or(mouse_action);

                        return NavResponse { action, response };
                    }
                    TransitionKind::Pop => {
                        // 1. Bottom Layer: Returning screen expands from behind with motion physics & spring overshoot (up to ~110%)
                        let base_scale = (1.0 - t.shrink_factor) + (t.shrink_factor * progress.min(1.0));
                        let overshoot_scale = if progress > 1.0 {
                            (progress - 1.0) * t.overshoot_factor
                        } else {
                            0.0
                        };
                        let scale = base_scale + overshoot_scale;

                        let bg_rect = Rect::from_center_size(total_rect.center(), total_rect.size() * scale);
                        let mut bg_ui = ui.child_ui(bg_rect, Layout::top_down(Align::Min));
                        bg_ui.set_clip_rect(bg_ui.clip_rect().intersect(ui.clip_rect()));
                        bg_ui.set_opacity(progress.clamp(0.0, 1.0));
                        let user_action = render_screen(active_key, &mut bg_ui);
                        let action = user_action.or(mouse_action);

                        // 2. Top Layer: Outgoing screen slides out towards the same side it entered from
                        let fg_offset = progress.max(0.0) * slide_vector;
                        let fg_rect = total_rect.translate(fg_offset);
                        let mut fg_ui = ui.child_ui(fg_rect, Layout::top_down(Align::Min));
                        fg_ui.set_clip_rect(fg_ui.clip_rect().intersect(total_rect));
                        fg_ui.set_opacity((1.0 - progress).clamp(0.0, 1.0));
                        fg_ui.add_enabled_ui(false, |ui| {
                            if let Some(prev_key) = &t.previous_screen {
                                render_screen(prev_key, ui);
                            }
                        });

                        return NavResponse { action, response };
                    }
                }
            }
        }

        // Standard single-layer rendering when settled or transitions disabled
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::hover());
        let mut child_ui = ui.child_ui(rect, Layout::top_down(Align::Min));
        child_ui.set_clip_rect(child_ui.clip_rect().intersect(rect));
        let user_action = render_screen(active_key, &mut child_ui);
        let action = user_action.or(mouse_action);

        NavResponse { action, response }
    }
}
