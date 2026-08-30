//! Spring-animated segmented tabs and pill switchers.
//!
//! Provides [`SegmentedTabs`] with a continuous sliding selection pill powered by `spring-core`,
//! custom tab items with icons and badges, and full theme integration.
//!
//! # State Ownership
//!
//! Animations are tracked via ID temporary storage or an app-owned [`TabsState`] (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Color32, Id, Rect, Rounding, Sense, Shape, Stroke, TextStyle, Ui,
    Vec2, WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// An individual tab option within [`SegmentedTabs`].
#[derive(Clone)]
pub struct TabItem<T> {
    /// Associated value for this tab.
    pub value: T,
    /// Text label for this tab.
    pub label: WidgetText,
    /// Optional leading icon.
    pub icon: Option<WidgetText>,
    /// Optional badge text/number.
    pub badge: Option<WidgetText>,
}

impl<T> TabItem<T> {
    /// Creates a new tab item with a value and label.
    pub fn new(value: T, label: impl Into<WidgetText>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            badge: None,
        }
    }

    /// Attaches an optional leading icon.
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Attaches an optional badge.
    pub fn badge(mut self, badge: impl Into<WidgetText>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

/// Response returned by [`SegmentedTabs::show`] providing standard egui interaction methods
/// alongside fine-grained lifecycle querying (`clicked`, `is_pressed`, `is_held`, `changed`).
#[derive(Clone, Debug)]
pub struct TabsResponse {
    /// The underlying [`egui::Response`].
    pub response: egui::Response,
    /// Whether the tabs container is currently pressed / held down (mouse or keyboard).
    pub is_pressed: bool,
    /// Standard click action: fired strictly on release.
    pub clicked: bool,
}

impl TabsResponse {
    /// Returns `true` if a tab was clicked on release.
    #[inline]
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` while a tab is currently pressed down.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` while a tab is currently held down (alias for `is_pressed`).
    #[inline]
    pub fn is_held(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` if the tab selection changed this frame.
    #[inline]
    pub fn changed(&self) -> bool {
        self.response.changed()
    }

    /// Unwraps and returns the inner [`egui::Response`].
    #[inline]
    pub fn into_inner(self) -> egui::Response {
        self.response
    }
}

impl std::ops::Deref for TabsResponse {
    type Target = egui::Response;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl std::ops::DerefMut for TabsResponse {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

impl From<TabsResponse> for egui::Response {
    #[inline]
    fn from(r: TabsResponse) -> Self {
        r.response
    }
}

/// Persistent animation state for sliding segmented tabs.
#[derive(Clone, Debug)]
pub struct TabsState {
    /// Spring driving the horizontal position of the selection pill.
    pub x_spring: Spring,
    /// Spring driving the width of the selection pill.
    pub width_spring: Spring,
    /// Spring driving vertical pill compression and depression ($0.0 \to 1.0$).
    pub press_spring: Spring,
    /// Whether the tab position has been initialized.
    pub initialized: bool,
}

impl Default for TabsState {
    fn default() -> Self {
        Self {
            x_spring: Spring::new(0.0, SpringParams::new(24.0, 0.48)),
            width_spring: Spring::new(0.0, SpringParams::new(24.0, 0.48)),
            press_spring: Spring::new(0.0, SpringParams::new(20.0, 0.45)),
            initialized: false,
        }
    }
}

impl TabsState {
    /// Updates the sliding springs towards the target rect and requests repaint if moving.
    pub fn update(
        &mut self,
        dt: f32,
        target_rect: Rect,
        is_pressed: bool,
        clicked: bool,
        ctx: &egui::Context,
    ) {
        if !self.initialized {
            self.x_spring = Spring::new(target_rect.min.x, SpringParams::new(24.0, 0.48));
            self.width_spring = Spring::new(target_rect.width(), SpringParams::new(24.0, 0.48));
            self.press_spring = Spring::new(0.0, SpringParams::new(20.0, 0.45));
            self.initialized = true;
            return;
        }

        if is_pressed {
            self.press_spring.set_target(1.0);
        } else if clicked {
            self.press_spring.velocity = (self.press_spring.velocity + 14.0).min(20.0);
            self.press_spring.set_target(0.0);
        } else {
            self.press_spring.set_target(0.0);
        }

        if (target_rect.min.x - self.x_spring.target).abs() > 0.5 {
            self.x_spring.velocity += (target_rect.min.x - self.x_spring.current) * 6.0;
            self.width_spring.velocity += (target_rect.width() - self.width_spring.current) * 4.0;
        }

        self.x_spring.set_target(target_rect.min.x);
        self.width_spring.set_target(target_rect.width());

        self.x_spring.update(dt);
        self.width_spring.update(dt);
        self.press_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have settled on target.
    pub fn is_settled(&self) -> bool {
        self.x_spring.is_settled() && self.width_spring.is_settled() && self.press_spring.is_settled()
    }
}

/// A theme-aware, spring-animated segmented tab bar.
///
/// # Example
/// ```no_run
/// use egui_widgets::{SegmentedTabs, TabItem};
///
/// #[derive(Clone, Copy, PartialEq)]
/// enum ViewMode { Grid, List, Table }
///
/// # egui::__run_test_ui(|ui| {
/// let mut mode = ViewMode::Grid;
/// SegmentedTabs::new(&mut mode)
///     .tab(ViewMode::Grid, "Grid View")
///     .tab(ViewMode::List, "List View")
///     .tab(ViewMode::Table, "Table View")
///     .show(ui);
/// # });
/// ```
pub struct SegmentedTabs<'a, T: PartialEq + Clone> {
    selected: &'a mut T,
    items: Vec<TabItem<T>>,
    height: f32,
    rounding: Option<Rounding>,
    container_fill: Option<Color32>,
    container_stroke: Option<Stroke>,
    pill_fill: Option<Color32>,
    pill_stroke: Option<Stroke>,
    active_text_color: Option<Color32>,
    inactive_text_color: Option<Color32>,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut TabsState>,
    focused: bool,
    triggered: bool,
    pressed: bool,
}

impl<'a, T: PartialEq + Clone> SegmentedTabs<'a, T> {
    /// Creates a new segmented tabs widget bound to the given selected value.
    pub fn new(selected: &'a mut T) -> Self {
        Self {
            selected,
            items: Vec::new(),
            height: 36.0,
            rounding: None,
            container_fill: None,
            container_stroke: None,
            pill_fill: None,
            pill_stroke: None,
            active_text_color: None,
            inactive_text_color: None,
            spring_params: SpringParams::new(24.0, 0.48),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
            focused: false,
            triggered: false,
            pressed: false,
        }
    }

    /// Adds a tab item with a value and label.
    pub fn tab(mut self, value: T, label: impl Into<WidgetText>) -> Self {
        self.items.push(TabItem::new(value, label));
        self
    }

    /// Adds a fully configured [`TabItem`].
    pub fn item(mut self, item: TabItem<T>) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the height of the segmented tab container (default `36.0pt`).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Explicitly overrides container rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Explicitly overrides the outer container background fill.
    pub fn container_fill(mut self, fill: Color32) -> Self {
        self.container_fill = Some(fill);
        self
    }

    /// Explicitly overrides the outer container border stroke.
    pub fn container_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.container_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the sliding selection pill background fill.
    pub fn pill_fill(mut self, fill: Color32) -> Self {
        self.pill_fill = Some(fill);
        self
    }

    /// Explicitly overrides the sliding selection pill border stroke.
    pub fn pill_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.pill_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the active tab label text color.
    pub fn active_text_color(mut self, color: Color32) -> Self {
        self.active_text_color = Some(color);
        self
    }

    /// Explicitly overrides the inactive tab label text color.
    pub fn inactive_text_color(mut self, color: Color32) -> Self {
        self.inactive_text_color = Some(color);
        self
    }

    /// Sets custom spring physics parameters for the sliding pill.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }

    /// Enables or disables spring motion animations (default `true`).
    pub fn motion(mut self, motion: bool) -> Self {
        self.motion = motion;
        self
    }

    /// Explicitly attaches a [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Binds an external [`TabsState`] struct.
    pub fn with_state(mut self, state: &'a mut TabsState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Explicitly marks the tabs as focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Explicitly marks the tabs as pressed / held down.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Explicitly triggers a selection action this frame (e.g. from Enter/Space/F key release).
    pub fn triggered(mut self, triggered: bool) -> Self {
        self.triggered = triggered;
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the segmented tabs and updates `selected` when a tab is clicked.
    pub fn show(self, ui: &mut Ui) -> TabsResponse {
        let n_items = self.items.len();
        if n_items == 0 {
            let resp = ui.allocate_response(Vec2::ZERO, Sense::hover());
            return TabsResponse {
                response: resp,
                is_pressed: false,
                clicked: false,
            };
        }

        let padding = vec2(12.0, 6.0);
        let pill_margin = 3.0;

        // Measure tab galleys
        let mut tab_galleys = Vec::with_capacity(n_items);
        let mut total_content_width = 0.0;

        for item in &self.items {
            let label = item.label.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Button,
            );
            let icon = item.icon.as_ref().map(|i| {
                i.clone().into_galley(
                    ui,
                    Some(false),
                    f32::INFINITY,
                    TextStyle::Button,
                )
            });
            let badge = item.badge.as_ref().map(|b| {
                b.clone().into_galley(
                    ui,
                    Some(false),
                    f32::INFINITY,
                    TextStyle::Small,
                )
            });

            let mut tab_w = label.size().x + padding.x * 2.0;
            if let Some(ref ic) = icon {
                tab_w += ic.size().x + 6.0;
            }
            if let Some(ref bg) = badge {
                tab_w += bg.size().x + 8.0;
            }

            total_content_width += tab_w;
            tab_galleys.push((label, icon, badge, tab_w));
        }

        let available_w = ui.available_width().max(total_content_width + pill_margin * 2.0);
        let container_size = vec2(available_w, self.height);

        let (container_rect, mut response) =
            ui.allocate_exact_size(container_size, Sense::click_and_drag());

        let is_focused = self.focused || response.has_focus();
        let (is_key_down, is_key_released) = if is_focused {
            ui.input(|i| {
                if i.modifiers.ctrl || i.modifiers.alt {
                    (false, false)
                } else {
                    let down = i.key_down(egui::Key::F) || i.key_down(egui::Key::Enter) || i.key_down(egui::Key::Space);
                    let released = i.key_released(egui::Key::F) || i.key_released(egui::Key::Enter) || i.key_released(egui::Key::Space);
                    (down, released)
                }
            })
        } else {
            (false, false)
        };

        let is_pressed = self.pressed || response.is_pointer_button_down_on() || is_key_down;
        let is_clicked = response.clicked() || is_key_released || self.triggered;

        // Resolve colors
        let (bg_fill, bg_stroke, pill_fill, pill_stroke, active_text, inactive_text) =
            if let Some(p) = self.palette {
                (
                    self.container_fill.unwrap_or(p.crust),
                    self.container_stroke.unwrap_or(Stroke::new(1.0, p.surface0)),
                    self.pill_fill.unwrap_or(p.surface0),
                    self.pill_stroke.unwrap_or(Stroke::new(1.0, p.surface1)),
                    self.active_text_color.unwrap_or(p.text),
                    self.inactive_text_color.unwrap_or(p.subtext0),
                )
            } else {
                let v = ui.visuals();
                (
                    self.container_fill.unwrap_or(v.faint_bg_color),
                    self.container_stroke.unwrap_or(v.widgets.inactive.bg_stroke),
                    self.pill_fill.unwrap_or(v.widgets.active.bg_fill),
                    self.pill_stroke.unwrap_or(v.widgets.active.bg_stroke),
                    self.active_text_color.unwrap_or(v.widgets.active.fg_stroke.color),
                    self.inactive_text_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                )
            };

        // Divide tabs evenly across available width
        let tab_width = (container_rect.width() - pill_margin * 2.0) / n_items as f32;
        let mut target_pill_rect = Rect::ZERO;

        for (idx, item) in self.items.iter().enumerate() {
            let tab_x = container_rect.left() + pill_margin + (idx as f32 * tab_width);
            let tab_rect = Rect::from_min_size(
                pos2(tab_x, container_rect.top() + pill_margin),
                vec2(tab_width, container_rect.height() - pill_margin * 2.0),
            );

            if item.value == *self.selected {
                target_pill_rect = tab_rect;
            }

            if is_clicked {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(mouse_pos) && item.value != *self.selected {
                        *self.selected = item.value.clone();
                        response.mark_changed();
                    }
                }
            }
        }

        // Motion physics handling
        let dt = ui.input(|i| i.stable_dt).min(0.05);

        let (pill_x, pill_w, press_factor) = if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, target_pill_rect, is_pressed, is_clicked, ui.ctx());
                (state.x_spring.value(), state.width_spring.value(), state.press_spring.value())
            } else {
                let id = self.id_source.unwrap_or_else(|| ui.make_persistent_id("segmented_tabs"));
                let mut state: TabsState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_default()
                });

                state.update(dt, target_pill_rect, is_pressed, is_clicked, ui.ctx());
                let values = (state.x_spring.value(), state.width_spring.value(), state.press_spring.value());
                ui.data_mut(|d| d.insert_temp(id, state));
                values
            }
        } else {
            (
                target_pill_rect.min.x,
                target_pill_rect.width(),
                if is_pressed { 1.0 } else { 0.0 },
            )
        };

        let y_sink = press_factor * 1.5;
        let pill_h_scale = (1.0 - press_factor * 0.10).max(0.7);
        let base_pill_h = container_rect.height() - pill_margin * 2.0;
        let active_pill_rect = Rect::from_center_size(
            pos2(pill_x + pill_w * 0.5, container_rect.center().y + y_sink),
            vec2(pill_w, base_pill_h * pill_h_scale),
        );

        if ui.is_rect_visible(container_rect) {
            let painter = ui.painter();
            let rounding = self.rounding.unwrap_or(Rounding::same(8.0));
            let pill_rounding = Rounding::same(rounding.nw.max(4.0) - 2.0);

            // Container background & stroke
            painter.add(Shape::rect_filled(container_rect, rounding, bg_fill));
            if bg_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(container_rect, rounding, bg_stroke));
            }

            // Sliding selection pill
            painter.add(Shape::rect_filled(active_pill_rect, pill_rounding, pill_fill));
            if pill_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(active_pill_rect, pill_rounding, pill_stroke));
            }

            // Render tab contents
            for (idx, (item, (label, icon, badge, _))) in self.items.iter().zip(tab_galleys.into_iter()).enumerate() {
                let tab_x = container_rect.left() + pill_margin + (idx as f32 * tab_width);
                let tab_rect = Rect::from_min_size(
                    pos2(tab_x, container_rect.top() + pill_margin + y_sink),
                    vec2(tab_width, base_pill_h),
                );

                let is_active = item.value == *self.selected;
                let text_color = if is_active { active_text } else { inactive_text };

                let mut content_w = label.size().x;
                if let Some(ref ic) = icon {
                    content_w += ic.size().x + 6.0;
                }
                if let Some(ref bg) = badge {
                    content_w += bg.size().x + 8.0;
                }

                let mut cur_x = tab_rect.center().x - content_w * 0.5;
                let center_y = tab_rect.center().y;

                if let Some(ic) = icon {
                    painter.galley(
                        pos2(cur_x, center_y - ic.size().y * 0.5),
                        ic.clone(),
                        text_color,
                    );
                    cur_x += ic.size().x + 6.0;
                }

                painter.galley(
                    pos2(cur_x, center_y - label.size().y * 0.5),
                    label.clone(),
                    text_color,
                );
                cur_x += label.size().x;

                if let Some(bg) = badge {
                    cur_x += 6.0;
                    let badge_rect = Rect::from_min_size(
                        pos2(cur_x, center_y - bg.size().y * 0.5 - 1.0),
                        vec2(bg.size().x + 6.0, bg.size().y + 2.0),
                    );
                    let badge_bg = if is_active {
                        text_color.linear_multiply(0.2)
                    } else {
                        text_color.linear_multiply(0.1)
                    };
                    painter.rect_filled(badge_rect, Rounding::same(4.0), badge_bg);
                    painter.galley(badge_rect.min + vec2(3.0, 1.0), bg, text_color);
                }
            }
        }

        TabsResponse {
            response,
            is_pressed,
            clicked: is_clicked,
        }
    }
}
