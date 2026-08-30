//! Spring-animated, theme-aware in-place morphing dropdown menu widget.
//!
//! Provides [`Dropdown`] with in-place height unrolling, continuous text trajectory morphing,
//! two-tier simultaneous physics highlights (saved selection + sliding focus pill), full 4-state
//! micro-interactions, zero raw mouse hover dependencies, and Vim keyboard navigation.
//!
//! # State Ownership
//!
//! Interactive animations can use either automatic ID-scoped memory or an explicit caller-owned
//! [`DropdownState`] struct passed via [`.with_state()`](Dropdown::with_state) (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Align2, Color32, FontId, Id, Rect, Rounding, Sense, Shape, Stroke, TextStyle, Ui,
    Vec2, WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

use crate::button::{ButtonSize, ButtonVariant};

/// An individual selectable option within a [`Dropdown`].
#[derive(Clone)]
pub struct DropdownOption<T> {
    /// Associated value for this option.
    pub value: T,
    /// Primary text label.
    pub label: WidgetText,
    /// Optional leading icon (e.g. emoji or icon glyph).
    pub icon: Option<WidgetText>,
    /// Optional secondary subtitle / description.
    pub subtitle: Option<WidgetText>,
    /// Optional secondary description underneath the primary label.
    pub description: Option<WidgetText>,
    /// Optional badge text/number.
    pub badge: Option<WidgetText>,
    /// Optional status dot indicator color (e.g. green for online, red for error).
    pub status_dot: Option<Color32>,
    /// Optional explicit badge color override.
    pub badge_color: Option<Color32>,
    /// Optional explicit icon tint color override.
    pub icon_color: Option<Color32>,
    /// Whether this option is disabled / non-interactive.
    pub disabled: bool,
}

impl<T> DropdownOption<T> {
    /// Creates a new dropdown option with a value and primary label.
    pub fn new(value: T, label: impl Into<WidgetText>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            subtitle: None,
            description: None,
            badge: None,
            status_dot: None,
            badge_color: None,
            icon_color: None,
            disabled: false,
        }
    }

    /// Attaches an optional leading icon.
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Attaches an optional secondary subtitle.
    pub fn subtitle(mut self, subtitle: impl Into<WidgetText>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Attaches an optional secondary description displayed underneath the primary label.
    pub fn description(mut self, description: impl Into<WidgetText>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Attaches an optional badge.
    pub fn badge(mut self, badge: impl Into<WidgetText>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Attaches an optional status dot indicator color (e.g. green for online, red for offline).
    pub fn status_dot(mut self, color: Color32) -> Self {
        self.status_dot = Some(color);
        self
    }

    /// Explicitly overrides the badge border and text color.
    pub fn badge_color(mut self, color: Color32) -> Self {
        self.badge_color = Some(color);
        self
    }

    /// Explicitly overrides the leading icon tint color.
    pub fn icon_color(mut self, color: Color32) -> Self {
        self.icon_color = Some(color);
        self
    }

    /// Sets whether this option is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for DropdownOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropdownOption")
            .field("value", &self.value)
            .field("label", &self.label.text())
            .field("subtitle", &self.subtitle.as_ref().map(|s| s.text()))
            .field("description", &self.description.as_ref().map(|d| d.text()))
            .field("badge", &self.badge.as_ref().map(|b| b.text()))
            .field("disabled", &self.disabled)
            .finish()
    }
}

/// Context passed to custom dropdown item renderers providing layout geometry and interaction states.
#[derive(Clone)]
pub struct DropdownItemContext<'a, T> {
    /// The option data for this row.
    pub option: &'a DropdownOption<T>,
    /// Zero-based index of this option in the list.
    pub index: usize,
    /// Whether this option is currently selected.
    pub is_selected: bool,
    /// Whether this option is currently highlighted via hover or keyboard navigation.
    pub is_highlighted: bool,
    /// Whether this option is disabled.
    pub is_disabled: bool,
    /// The allocated screen-space bounding rect for this row.
    pub rect: Rect,
    /// Morphing opacity factor (0.0 to 1.0) during open/close transitions.
    pub opacity: f32,
    /// Theme palette reference if configured on the dropdown.
    pub palette: Option<&'a ThemePalette>,
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for DropdownItemContext<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropdownItemContext")
            .field("option", &self.option)
            .field("index", &self.index)
            .field("is_selected", &self.is_selected)
            .field("is_highlighted", &self.is_highlighted)
            .field("is_disabled", &self.is_disabled)
            .field("rect", &self.rect)
            .field("opacity", &self.opacity)
            .finish()
    }
}

/// Response returned by [`Dropdown::show`] providing standard egui interaction methods
/// alongside fine-grained lifecycle querying (`changed`, `is_open`, `is_pressed`, `clicked`).
#[derive(Clone, Debug)]
pub struct DropdownResponse<T> {
    /// The underlying [`egui::Response`] of the dropdown container.
    pub response: egui::Response,
    /// Newly selected value if a selection occurred on this frame.
    pub selected: Option<T>,
    /// Whether the selected value changed on this frame.
    pub changed: bool,
    /// Whether the dropdown menu is currently expanded open.
    pub is_open: bool,
    /// Whether the header / active item is currently pressed down.
    pub is_pressed: bool,
    /// Standard click action on the dropdown.
    pub clicked: bool,
    /// The active focus highlight rectangle (collapsed header or focused item row when open).
    pub highlight_rect: Rect,
    /// The full expanded bounding rectangle of the dropdown menu.
    pub expanded_rect: Rect,
    /// Whether the full expanded dropdown fits completely within the visible viewport bounds.
    pub is_fully_visible: bool,
    /// Whether the dropdown transitioned from closed to open on this frame.
    pub just_opened: bool,
    /// Whether the user actively navigated items with keyboard (J/K/Arrows) on this frame.
    pub is_navigating: bool,
}

impl<T> DropdownResponse<T> {
    /// Returns `true` if a new option was selected on this frame.
    #[inline]
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Returns `true` if the dropdown container is expanded open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns `true` while the dropdown is currently pressed down.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` if the dropdown was clicked on release.
    #[inline]
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns the active focus highlight rectangle.
    ///
    /// When collapsed, returns the header rect; when expanded open, returns the exact
    /// row rect of the currently focused / navigated item for [`HighlightGroup`] gliding.
    #[inline]
    pub fn highlight_rect(&self) -> Rect {
        self.highlight_rect
    }

    /// Returns the full expanded bounding rectangle of the dropdown menu.
    #[inline]
    pub fn expanded_rect(&self) -> Rect {
        self.expanded_rect
    }

    /// Returns whether the full expanded dropdown fits completely within the visible viewport bounds.
    #[inline]
    pub fn is_fully_visible(&self) -> bool {
        self.is_fully_visible
    }

    /// Returns `true` only on the frame the dropdown transitioned from closed to open.
    #[inline]
    pub fn just_opened(&self) -> bool {
        self.just_opened
    }

    /// Returns `true` only on frames when the user actively navigated items with keyboard (J/K/Arrows).
    #[inline]
    pub fn is_navigating(&self) -> bool {
        self.is_navigating
    }

    /// Unwraps and returns the inner [`egui::Response`].
    #[inline]
    pub fn into_inner(self) -> egui::Response {
        self.response
    }
}

impl<T> std::ops::Deref for DropdownResponse<T> {
    type Target = egui::Response;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl<T> std::ops::DerefMut for DropdownResponse<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

/// Persistent animation and interactive state for spring-driven in-place morphing dropdowns.
///
/// Can be owned directly by the caller or stored in egui ID temporary storage (`CODING_RULES §2`).
#[derive(Clone, Debug)]
pub struct DropdownState {
    /// Spring driving header press compression and rebound pop ($0.0 \to 1.0 \to 0.0$).
    pub press_spring: Spring,
    /// Spring driving smooth header hover glow/luminance transition ($0.0 \to 1.0$).
    pub hover_spring: Spring,
    /// Spring driving instantaneous organic expand-and-shrink hover/focus bounce ($0.0 \to 0.0$).
    pub hover_bounce_spring: Spring,
    /// Spring driving in-place container height expansion ($0.0 \leftrightarrow 1.0$).
    pub open_spring: Spring,
    /// Spring driving chevron arrow rotation ($0.0 \leftrightarrow 1.0$).
    pub chevron_spring: Spring,
    /// Spring driving sliding focus highlight vertical position across options.
    pub focus_y_spring: Spring,
    /// Spring driving sliding focus highlight vertical height.
    pub focus_h_spring: Spring,
    /// Spring driving saved active selection highlight vertical position.
    pub saved_y_spring: Spring,
    /// Spring driving saved active selection highlight vertical height.
    pub saved_h_spring: Spring,
    /// Whether the dropdown container is currently open and expanded.
    pub is_open: bool,
    /// Currently keyboard/mouse focused item index in the expanded menu.
    pub highlighted_index: Option<usize>,
    /// Tracks previous frame's hover/focus state.
    pub was_hovered: bool,
    /// Tracks previous frame's open state.
    pub was_open: bool,
    /// Whether the sliding focus highlight spring has been initialized.
    pub focus_initialized: bool,
    /// Whether the saved selection highlight spring has been initialized.
    pub saved_initialized: bool,
    /// Spring driving vertical scroll offset in pixels for long option lists.
    pub scroll_offset_spring: Spring,
    /// Whether scroll offset spring has been initialized.
    pub scroll_initialized: bool,
    /// Last allocated bounding rectangle of the dropdown container.
    pub last_rect: Option<Rect>,
    /// Whether the dropdown needs to be auto-scrolled into view upon expanding.
    pub needs_expand_scroll: bool,
    /// Whether the dropdown was focused on the previous frame.
    pub was_focused: bool,
}

impl Default for DropdownState {
    fn default() -> Self {
        Self {
            press_spring: Spring::new(0.0, SpringParams::new(18.0, 0.48)),
            hover_spring: Spring::new(0.0, SpringParams::new(22.0, 0.60)),
            hover_bounce_spring: Spring::new(0.0, SpringParams::new(20.0, 0.50)),
            open_spring: Spring::new(0.0, SpringParams::new(28.0, 0.75)),
            chevron_spring: Spring::new(0.0, SpringParams::new(24.0, 0.75)),
            focus_y_spring: Spring::new(0.0, SpringParams::new(32.0, 0.85)),
            focus_h_spring: Spring::new(0.0, SpringParams::new(32.0, 0.85)),
            saved_y_spring: Spring::new(0.0, SpringParams::new(32.0, 0.85)),
            saved_h_spring: Spring::new(0.0, SpringParams::new(32.0, 0.85)),
            scroll_offset_spring: Spring::new(0.0, SpringParams::new(28.0, 0.75)),
            is_open: false,
            highlighted_index: None,
            was_hovered: false,
            was_open: false,
            focus_initialized: false,
            saved_initialized: false,
            scroll_initialized: false,
            last_rect: None,
            needs_expand_scroll: false,
            was_focused: false,
        }
    }
}

impl DropdownState {
    /// Creates a new default dropdown state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the dropdown is open and the pointer is currently inside its bounding rect.
    pub fn contains_pointer(&self, ctx: &egui::Context) -> bool {
        if self.is_open {
            if let Some(rect) = self.last_rect {
                return ctx.input(|i| {
                    if let Some(pos) = i.pointer.latest_pos() {
                        rect.contains(pos)
                    } else {
                        false
                    }
                });
            }
        }
        false
    }

    /// Computes the target scroll row index for a given highlighted item index,
    /// maintaining a 1-item scrolloff margin from the top and bottom of the visible window.
    pub fn compute_scrolloff_row(
        highlighted_index: usize,
        current_scroll_row: usize,
        total_items: usize,
        max_visible: usize,
    ) -> usize {
        if total_items <= max_visible {
            return 0;
        }
        let scrolloff = 1usize;
        let max_scroll_idx = total_items.saturating_sub(max_visible);
        let lower_thresh = current_scroll_row + max_visible.saturating_sub(1 + scrolloff);
        let upper_thresh = current_scroll_row + scrolloff;

        if highlighted_index > lower_thresh {
            (highlighted_index.saturating_sub(max_visible.saturating_sub(1 + scrolloff))).min(max_scroll_idx)
        } else if highlighted_index < upper_thresh {
            highlighted_index.saturating_sub(scrolloff).min(max_scroll_idx)
        } else {
            current_scroll_row.min(max_scroll_idx)
        }
    }

    /// Triggers an elastic squash-and-rebound pop animation on click release.
    pub fn trigger_click(&mut self) {
        self.press_spring.velocity = 18.0;
        self.press_spring.set_target(0.0);
    }

    /// Triggers an instantaneous size bounce impulse on hover/focus arrival.
    pub fn trigger_hover_bounce(&mut self) {
        self.hover_bounce_spring.current = 0.0;
        self.hover_bounce_spring.velocity = 20.0;
        self.hover_bounce_spring.set_target(0.0);
    }

    /// Toggles the open/closed expanded state of the dropdown menu.
    pub fn toggle(&mut self) {
        self.set_open(!self.is_open);
    }

    /// Opens and expands the dropdown menu.
    pub fn open(&mut self) {
        self.set_open(true);
    }

    /// Closes and collapses the dropdown menu.
    pub fn close(&mut self) {
        self.set_open(false);
    }

    /// Sets the open/closed state of the dropdown menu.
    pub fn set_open(&mut self, open: bool) {
        if open && !self.is_open {
            self.needs_expand_scroll = true;
            self.scroll_initialized = false;
        } else if !open {
            self.needs_expand_scroll = false;
            self.scroll_initialized = false;
        }
        self.is_open = open;
        self.open_spring.set_target(if open { 1.0 } else { 0.0 });
        self.chevron_spring.set_target(if open { 1.0 } else { 0.0 });
    }

    /// Updates the dropdown's internal springs and requests repaint if still moving.
    pub fn update(
        &mut self,
        dt: f32,
        is_hovered: bool,
        is_pressed: bool,
        clicked: bool,
        ctx: &egui::Context,
    ) {
        if is_hovered && !self.was_hovered {
            self.trigger_hover_bounce();
        }
        self.was_hovered = is_hovered;

        self.hover_spring.set_target(if is_hovered { 1.0 } else { 0.0 });

        if is_pressed {
            self.press_spring.set_target(1.0);
        } else if clicked {
            self.trigger_click();
        } else {
            self.press_spring.set_target(0.0);
        }

        self.open_spring.set_target(if self.is_open { 1.0 } else { 0.0 });
        self.chevron_spring.set_target(if self.is_open { 1.0 } else { 0.0 });

        self.hover_spring.update(dt);
        self.hover_bounce_spring.update(dt);
        self.press_spring.update(dt);
        self.open_spring.update(dt);
        self.chevron_spring.update(dt);
        self.focus_y_spring.update(dt);
        self.focus_h_spring.update(dt);
        self.saved_y_spring.update(dt);
        self.saved_h_spring.update(dt);
        self.scroll_offset_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have settled within tolerance.
    pub fn is_settled(&self) -> bool {
        self.hover_spring.is_settled()
            && self.hover_bounce_spring.is_settled()
            && self.press_spring.is_settled()
            && self.open_spring.is_settled()
            && self.chevron_spring.is_settled()
            && self.focus_y_spring.is_settled()
            && self.focus_h_spring.is_settled()
            && self.saved_y_spring.is_settled()
            && self.saved_h_spring.is_settled()
            && self.scroll_offset_spring.is_settled()
    }
}

/// A spring-animated, theme-aware in-place morphing dropdown component.
pub struct Dropdown<'a, T> {
    selected: &'a mut T,
    options: Vec<DropdownOption<T>>,
    placeholder: Option<WidgetText>,
    icon: Option<WidgetText>,
    variant: ButtonVariant,
    size: ButtonSize,
    width: Option<f32>,
    palette: Option<&'a ThemePalette>,
    external_state: Option<&'a mut DropdownState>,
    focused: bool,
    triggered: bool,
    pressed: bool,
    motion: bool,
    spring_params: SpringParams,
    id_source: Option<Id>,
    rounding: Option<Rounding>,
    padding: Option<Vec2>,
    show_chevron: bool,
    chevron_icon: Option<String>,
    max_visible_items: usize,
    item_height: Option<f32>,
    item_renderer: Option<Box<dyn Fn(&mut Ui, &DropdownItemContext<'_, T>) + 'a>>,
    auto_scroll: bool,
    close_on_focus_lost: bool,
    close_on_outside_click: bool,
    show_internal_focus: bool,
}

impl<'a, T: Clone + PartialEq> Dropdown<'a, T> {
    /// Creates a new dropdown bound to `selected` with a collection of options.
    pub fn new(selected: &'a mut T, options: impl IntoIterator<Item = DropdownOption<T>>) -> Self {
        Self {
            selected,
            options: options.into_iter().collect(),
            placeholder: None,
            icon: None,
            variant: ButtonVariant::Secondary,
            size: ButtonSize::Medium,
            width: None,
            palette: None,
            external_state: None,
            focused: false,
            triggered: false,
            pressed: false,
            motion: true,
            spring_params: SpringParams::new(14.0, 0.42),
            id_source: None,
            rounding: None,
            padding: None,
            show_chevron: true,
            chevron_icon: None,
            max_visible_items: 5,
            item_height: None,
            item_renderer: None,
            auto_scroll: true,
            close_on_focus_lost: false,
            close_on_outside_click: false,
            show_internal_focus: false,
        }
    }

    /// Sets placeholder text displayed when no option matches `selected`.
    pub fn placeholder(mut self, placeholder: impl Into<WidgetText>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Attaches an optional leading icon to the dropdown header.
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the visual styling variant of the dropdown container.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets primary visual variant using theme accent.
    pub fn primary(self) -> Self {
        self.variant(ButtonVariant::Primary)
    }

    /// Sets secondary visual variant using theme surfaces.
    pub fn secondary(self) -> Self {
        self.variant(ButtonVariant::Secondary)
    }

    /// Sets ghost borderless visual variant.
    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Ghost)
    }

    /// Sets outline visual variant.
    pub fn outline(self) -> Self {
        self.variant(ButtonVariant::Outline)
    }

    /// Sets the sizing preset for the collapsed dropdown box.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Sets compact small sizing.
    pub fn small(self) -> Self {
        self.size(ButtonSize::Small)
    }

    /// Sets large prominent sizing.
    pub fn large(self) -> Self {
        self.size(ButtonSize::Large)
    }

    /// Overrides explicit widget width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Explicitly provides an active [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Binds an external [`DropdownState`] instance.
    pub fn with_state(mut self, state: &'a mut DropdownState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Sets whether this dropdown is focused by keyboard/navigator.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Explicitly triggers a click activation for keyboard navigation.
    pub fn triggered(mut self, triggered: bool) -> Self {
        self.triggered = triggered;
        self
    }

    /// Explicitly flags the button as held down.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Enables or disables spring motion animations (default `true`).
    pub fn motion(mut self, motion: bool) -> Self {
        self.motion = motion;
        self
    }

    /// Sets custom spring physics parameters for press/hover animations.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }

    /// Sets an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Explicitly overrides the corner rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Explicitly overrides the inner margin padding.
    pub fn padding(mut self, padding: impl Into<Vec2>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    /// Sets whether to render the trailing animated chevron indicator (default `true`).
    pub fn show_chevron(mut self, show: bool) -> Self {
        self.show_chevron = show;
        self
    }

    /// Explicitly overrides the trailing chevron icon symbol (defaults to `\u{f078}`).
    pub fn chevron_icon(mut self, icon: impl Into<String>) -> Self {
        self.chevron_icon = Some(icon.into());
        self
    }

    /// Sets the maximum number of visible items before enabling scrollable viewport (default 5).
    pub fn max_visible_items(mut self, max: usize) -> Self {
        self.max_visible_items = max;
        self
    }

    /// Sets an explicit item row height (defaults to collapsed button height, min 28.0px).
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = Some(height);
        self
    }

    /// Sets a custom item renderer closure for full developer control over row design and layout.
    pub fn item_renderer(
        mut self,
        renderer: impl Fn(&mut Ui, &DropdownItemContext<'_, T>) + 'a,
    ) -> Self {
        self.item_renderer = Some(Box::new(renderer));
        self
    }

    /// Sets whether to automatically request viewport scrolling when the expanded menu overflows the visible bounds (default `true`).
    pub fn auto_scroll(mut self, enabled: bool) -> Self {
        self.auto_scroll = enabled;
        self
    }

    /// Sets whether the dropdown should automatically close when focus is lost (default `false`).
    pub fn close_on_focus_lost(mut self, close: bool) -> Self {
        self.close_on_focus_lost = close;
        self
    }

    /// Sets whether the dropdown should automatically close when clicking outside (default `false`).
    pub fn close_on_outside_click(mut self, close: bool) -> Self {
        self.close_on_outside_click = close;
        self
    }

    /// Sets whether to render an internal focus stroke rectangle on the focused item (default `false`).
    pub fn show_internal_focus(mut self, show: bool) -> Self {
        self.show_internal_focus = show;
        self
    }

    /// Renders the in-place morphing dropdown widget and returns a [`DropdownResponse`].
    pub fn show(mut self, ui: &mut Ui) -> DropdownResponse<T> {
        let (default_padding, _font_size) = self.size.metrics();
        let padding = self.padding.unwrap_or(default_padding);

        let collapsed_h = match self.size {
            ButtonSize::Small => 26.0,
            ButtonSize::Medium => 30.0,
            ButtonSize::Large => 38.0,
            ButtonSize::Custom { padding, font_size } => font_size + padding.y * 2.0,
        };

        let card_h = self.item_height.unwrap_or(collapsed_h.max(28.0));
        let spacing_y = 8.0;
        let padding_y = 8.0;
        let item_step = card_h + spacing_y;
        let total_items = self.options.len();
        let max_visible = self.max_visible_items.max(1);
        let visible_count = total_items.min(max_visible);

        let full_expanded_h = padding_y * 2.0
            + (visible_count as f32 * card_h)
            + (visible_count.saturating_sub(1) as f32 * spacing_y);

        let id = self.id_source.unwrap_or_else(|| ui.make_persistent_id("dropdown_inplace"));

        let mut temp_state = if self.external_state.is_none() {
            Some(ui.data_mut(|d| d.get_temp::<DropdownState>(id).unwrap_or_default()))
        } else {
            None
        };

        let state: &mut DropdownState = if let Some(ref mut ext) = self.external_state {
            ext
        } else {
            temp_state.as_mut().unwrap()
        };

        // If dropdown is open and lost focus (e.g. user hovered/highlighted something else with mouse), close it
        if state.is_open && self.close_on_focus_lost && !self.focused && state.was_focused {
            state.close();
        }
        state.was_focused = self.focused;

        let was_open = state.is_open;
        let dt = ui.input(|i| i.stable_dt).min(0.05);

        // Find active option to display in header
        let selected_pos = self.options.iter().position(|opt| &opt.value == self.selected);
        let active_option = selected_pos.and_then(|idx| self.options.get(idx));

        let header_text = if let Some(opt) = active_option {
            opt.label.clone()
        } else if let Some(ref ph) = self.placeholder {
            ph.clone()
        } else {
            WidgetText::from("Select...")
        };

        let header_icon = if let Some(opt) = active_option {
            opt.icon.as_ref().or(self.icon.as_ref()).cloned()
        } else {
            self.icon.clone()
        };

        let text_layout = header_text.into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
        let icon_layout = header_icon.as_ref().map(|icon| icon.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button));

        // Auto-calculate content width based on widest option + right metadata + generous padding
        let mut max_content_w: f32 = text_layout.size().x;
        if let Some(ref icon_g) = icon_layout {
            max_content_w += icon_g.size().x + 6.0;
        }
        for opt in &self.options {
            let l_g = opt.label.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
            let mut opt_w = l_g.size().x + 16.0;
            if let Some(ref ic) = opt.icon {
                let i_g = ic.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
                opt_w += i_g.size().x + 6.0;
            }
            if opt.status_dot.is_some() {
                opt_w += 14.0;
            }
            if let Some(ref desc) = opt.description {
                let d_g = desc.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Small);
                if d_g.size().x + 16.0 > opt_w {
                    opt_w = d_g.size().x + 16.0;
                }
            }
            if let Some(ref sub) = opt.subtitle {
                let s_g = sub.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Small);
                opt_w += s_g.size().x + 12.0;
            }
            if let Some(ref bdg) = opt.badge {
                let b_g = bdg.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Small);
                opt_w += b_g.size().x + 16.0;
            }
            opt_w += 24.0; // checkmark space
            if opt_w > max_content_w {
                max_content_w = opt_w;
            }
        }
        if self.show_chevron {
            max_content_w += 22.0;
        }
        if total_items > max_visible {
            max_content_w += 8.0;
        }

        let desired_w = self.width.unwrap_or(max_content_w + padding.x * 2.0).max(180.0);

        // Compute current morphing height based on open spring
        let open_t = state.open_spring.value().clamp(0.0, 1.0);
        let current_h = collapsed_h + open_t * (full_expanded_h - collapsed_h);

        let (rect, mut response) = ui.allocate_exact_size(vec2(desired_w, current_h), Sense::click_and_drag());
        state.last_rect = Some(rect);

        let full_expanded_rect = Rect::from_min_size(rect.min, vec2(desired_w, full_expanded_h));
        let viewport = ui.clip_rect();
        let is_fully_visible = full_expanded_rect.top() >= viewport.top() - 1.0
            && full_expanded_rect.bottom() <= viewport.bottom() + 1.0;

        let just_opened = state.needs_expand_scroll || (!was_open && state.is_open) || (state.is_open && !state.scroll_initialized);
        state.needs_expand_scroll = false;

        if state.is_open && just_opened {
            let initial_idx = selected_pos.unwrap_or(0);
            state.highlighted_index = Some(initial_idx);
            let max_scroll_idx = total_items.saturating_sub(max_visible);
            let target_scroll_row = (initial_idx.saturating_sub(1)).min(max_scroll_idx);
            let target_scroll_px = target_scroll_row as f32 * item_step;
            state.scroll_offset_spring.reset(target_scroll_px);
            state.scroll_offset_spring.set_target(target_scroll_px);
            state.scroll_initialized = true;
            state.saved_initialized = false;
            state.focus_initialized = false;
        }

        // Auto-scroll parent ScrollArea if menu expanded and is partially or completely off-screen
        if self.auto_scroll && just_opened && !is_fully_visible {
            ui.scroll_to_rect(full_expanded_rect, None);
        }

        // Resolve theme colors: strictly outlined with NO fill color
        let (base_stroke, text_color, subtext_color, accent_color, saved_color) =
            if let Some(p) = self.palette {
                match self.variant {
                    ButtonVariant::Primary => (
                        Stroke::new(1.0, p.surface1),
                        p.text,
                        p.subtext0,
                        p.accent,
                        p.warning, // Peach in Catppuccin Mocha, matching openrgb!
                    ),
                    ButtonVariant::Secondary => (
                        Stroke::new(1.0, p.surface1),
                        p.text,
                        p.subtext0,
                        p.accent,
                        p.accent,
                    ),
                    ButtonVariant::Outline => (
                        Stroke::new(1.0, p.accent),
                        p.text,
                        p.subtext0,
                        p.accent,
                        p.warning,
                    ),
                    ButtonVariant::Ghost => (
                        Stroke::NONE,
                        p.text,
                        p.subtext0,
                        p.accent,
                        p.accent,
                    ),
                    ButtonVariant::Danger => (
                        Stroke::new(1.0, p.danger),
                        p.on_danger,
                        p.subtext0,
                        p.danger,
                        p.danger,
                    ),
                    ButtonVariant::Success => (
                        Stroke::new(1.0, p.success),
                        p.on_success,
                        p.subtext0,
                        p.success,
                        p.success,
                    ),
                    ButtonVariant::Warning => (
                        Stroke::new(1.0, p.warning),
                        p.on_warning,
                        p.subtext0,
                        p.warning,
                        p.warning,
                    ),
                }
            } else {
                let v = ui.visuals();
                (
                    v.widgets.inactive.bg_stroke,
                    v.widgets.inactive.fg_stroke.color,
                    v.widgets.noninteractive.fg_stroke.color,
                    v.selection.stroke.color,
                    v.selection.stroke.color,
                )
            };

        let rounding = self.rounding.unwrap_or(Rounding::same(4.0));

        // Focus vs Hover Paradigm: Respect cursor autohide
        let is_hovered = (response.hovered() && crate::is_hover_active(ui.ctx())) || self.focused;
        let is_focused = self.focused || response.has_focus() || is_hovered;

        let pointer_pos = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
        let pointer_clicked = ui.input(|i| i.pointer.primary_clicked());

        let mut new_selection: Option<T> = None;
        let mut selection_changed = false;
        let mut is_key_navigating = false;

        let (header_pressed, header_clicked) = if !was_open {
            // Dropdown is collapsed: clicking or releasing Enter/Space/F opens it
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

            let pressed = self.pressed
                || (is_focused && (is_key_down || ui.input(|i| i.pointer.primary_down() && response.hovered())))
                || response.is_pointer_button_down_on();

            let clicked = self.triggered
                || (is_focused && is_key_released)
                || response.clicked();

            if clicked {
                state.open();
                let initial_idx = selected_pos.unwrap_or(0);
                state.highlighted_index = Some(initial_idx);
                let max_scroll_idx = total_items.saturating_sub(max_visible);
                let target_scroll_row = (initial_idx.saturating_sub(1)).min(max_scroll_idx);
                let target_scroll_px = target_scroll_row as f32 * item_step;
                state.scroll_offset_spring.reset(target_scroll_px);
                state.scroll_offset_spring.set_target(target_scroll_px);
                state.scroll_initialized = true;
                state.saved_initialized = false;
                state.focus_initialized = false;
            }

            (pressed, clicked)
        } else {
            // Dropdown was already open at frame start: handle mouse wheel scroll, item hover, and item click/key selection
            let scroll_y = state.scroll_offset_spring.value();

            if state.is_open && total_items > max_visible && ui.rect_contains_pointer(rect) {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta.abs() > 0.1 {
                    let max_scroll_idx = total_items.saturating_sub(max_visible);
                    let max_scroll_px = max_scroll_idx as f32 * item_step;
                    let current_target = state.scroll_offset_spring.target;
                    let new_target = (current_target - scroll_delta.signum() * item_step).clamp(0.0, max_scroll_px);
                    state.scroll_offset_spring.set_target(new_target);
                    ui.input_mut(|i| {
                        i.raw_scroll_delta = Vec2::ZERO;
                        i.smooth_scroll_delta = Vec2::ZERO;
                    });

                    // Keep highlighted index strictly inside the newly scrolled visible window!
                    let first_vis = (new_target / item_step).floor().max(0.0) as usize;
                    let last_vis = (first_vis + max_visible.saturating_sub(1)).min(total_items.saturating_sub(1));
                    if let Some(pos) = pointer_pos {
                        if rect.contains(pos) {
                            let scrolled_local_y = pos.y - rect.min.y + new_target;
                            for item_idx in 0..total_items {
                                let row_top = padding_y + (item_idx as f32) * item_step;
                                let row_bottom = row_top + card_h;
                                if scrolled_local_y >= row_top && scrolled_local_y <= row_bottom {
                                    if let Some(opt) = self.options.get(item_idx) {
                                        if !opt.disabled {
                                            state.highlighted_index = Some(item_idx);
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    if let Some(h) = state.highlighted_index {
                        state.highlighted_index = Some(h.clamp(first_vis, last_vis));
                    }
                }
            }

            if open_t > 0.05 {
                if let Some(pos) = pointer_pos {
                    if rect.contains(pos) && crate::is_hover_active(ui.ctx()) {
                        let scrolled_local_y = pos.y - rect.min.y + scroll_y;
                        for item_idx in 0..total_items {
                            let row_top = padding_y + (item_idx as f32) * item_step;
                            let row_bottom = row_top + card_h;
                            if scrolled_local_y >= row_top && scrolled_local_y <= row_bottom {
                                if let Some(opt) = self.options.get(item_idx) {
                                    if !opt.disabled {
                                        state.highlighted_index = Some(item_idx);
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }

            // Keyboard navigation across dropdown items (J/K/Arrows navigate on key press)
            let (nav_up, nav_down, is_key_down, is_key_released, close_pressed) = ui.input(|i| {
                let close = i.key_pressed(egui::Key::Escape) || (i.modifiers.ctrl && i.key_pressed(egui::Key::C)) || i.key_pressed(egui::Key::Q);
                let down = i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown);
                let up = i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp);
                let k_down = i.key_down(egui::Key::Enter) || i.key_down(egui::Key::Space) || i.key_down(egui::Key::F);
                let k_rel = i.key_released(egui::Key::Enter) || i.key_released(egui::Key::Space) || i.key_released(egui::Key::F);
                (up, down, k_down, k_rel, close)
            });

            let pressed = self.pressed || (is_focused && is_key_down);

            let is_nav = nav_down || nav_up;
            is_key_navigating = is_nav;
            if is_nav && self.auto_scroll && !is_fully_visible {
                ui.scroll_to_rect(full_expanded_rect, None);
            }

            if close_pressed {
                state.close();
            } else if nav_down {
                let cur = state.highlighted_index.unwrap_or(0);
                let next = (cur + 1).min(total_items.saturating_sub(1));
                state.highlighted_index = Some(next);
            } else if nav_up {
                let cur = state.highlighted_index.unwrap_or(0);
                let next = cur.saturating_sub(1);
                state.highlighted_index = Some(next);
            } else if self.triggered || (is_focused && is_key_released) {
                // Item selection triggers on key release, completing the single click cycle cleanly
                if let Some(idx) = state.highlighted_index {
                    if let Some(opt) = self.options.get(idx) {
                        if !opt.disabled {
                            new_selection = Some(opt.value.clone());
                            selection_changed = true;
                            *self.selected = opt.value.clone();
                            state.trigger_click();
                            state.close();
                        }
                    }
                }
            }

            // Pointer click on items or outside to close
            if pointer_clicked {
                if let Some(pos) = pointer_pos {
                    if rect.contains(pos) {
                        let scrolled_local_y = pos.y - rect.min.y + scroll_y;
                        for item_idx in 0..total_items {
                            let row_top = padding_y + (item_idx as f32) * item_step;
                            let row_bottom = row_top + card_h;
                            if scrolled_local_y >= row_top && scrolled_local_y <= row_bottom {
                                if let Some(opt) = self.options.get(item_idx) {
                                    if !opt.disabled {
                                        new_selection = Some(opt.value.clone());
                                        selection_changed = true;
                                        *self.selected = opt.value.clone();
                                        state.trigger_click();
                                        state.close();
                                    }
                                }
                                break;
                            }
                        }
                    } else if self.close_on_outside_click {
                        state.close();
                    }
                }
            }

            // Synchronize scrolloff=1 margin viewport offset ONLY on keyboard navigation (nav_down / nav_up)
            // or when freshly opening the dropdown to ensure selected item starts in view.
            // Mouse hover must NEVER trigger scrolloff scrolling so the list remains stationary under cursor!
            let max_scroll_idx = total_items.saturating_sub(max_visible);
            let max_scroll_px = max_scroll_idx as f32 * item_step;

            if nav_down || nav_up {
                let current_scroll_row = if item_step > 0.0 {
                    (state.scroll_offset_spring.target / item_step).round() as usize
                } else {
                    0
                };
                let target_scroll_row = if let Some(h_idx) = state.highlighted_index {
                    DropdownState::compute_scrolloff_row(h_idx, current_scroll_row, total_items, max_visible)
                } else {
                    0
                };
                let target_scroll_px = (target_scroll_row as f32 * item_step).clamp(0.0, max_scroll_px);
                state.scroll_offset_spring.set_target(target_scroll_px);
            }

            (pressed, selection_changed)
        };

        // Motion physics updates
        let (hover_factor, hover_bounce, press_factor, _chevron_factor) = if self.motion {
            state.press_spring.params = self.spring_params;
            state.update(dt, is_hovered, header_pressed, header_clicked, ui.ctx());
            (
                state.hover_spring.value(),
                state.hover_bounce_spring.value(),
                state.press_spring.value(),
                state.chevron_spring.value(),
            )
        } else {
            (
                if is_hovered { 1.0 } else { 0.0 },
                0.0,
                if header_pressed { 1.0 } else { 0.0 },
                if state.is_open { 1.0 } else { 0.0 },
            )
        };

        // Energetic, clearly noticeable tactile bounce (4.5% arrival pop, 6% press compression).
        // For mouse interaction, offsets/shifts are disabled so the widget doesn't move under the pointer.
        let is_pointer_driven = response.hovered() || response.clicked() || response.is_pointer_button_down_on();
        let scale = if !state.is_open && self.focused && !is_pointer_driven {
            (1.0 + (hover_bounce * 0.045) - (press_factor * 0.06)).clamp(0.90, 1.10)
        } else {
            1.0
        };
        let y_offset = if !state.is_open && self.focused && !is_pointer_driven { press_factor * 2.5 } else { 0.0 };
        let center = rect.center() + vec2(0.0, y_offset);
        let animated_rect = Rect::from_center_size(center, vec2(rect.width() * scale, rect.height() * scale));
        let animated_rounding = rounding * scale;

        let current_stroke = if !state.is_open && self.focused {
            Stroke::new(1.0, accent_color)
        } else if state.is_open {
            if let Some(p) = self.palette {
                Stroke::new(1.0, p.surface1)
            } else {
                Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color)
            }
        } else if hover_factor > 0.05 {
            if let Some(p) = self.palette {
                Stroke::new(1.0, p.surface2)
            } else {
                Stroke::new(base_stroke.width + 0.5, base_stroke.color)
            }
        } else {
            base_stroke
        };

        // =========================================================================
        // Render Outlined In-Place Container & Highlights (NO fill color, no clipped borders)
        // =========================================================================
        if ui.is_rect_visible(rect) {
            // 1. Outlined Container Border (Strictly outlined, no fill color)
            if current_stroke.width > 0.0 {
                ui.painter().add(Shape::rect_stroke(animated_rect, animated_rounding, current_stroke));
            }

            let scroll_y = if state.is_open {
                state.scroll_offset_spring.value()
            } else {
                0.0
            };

            // 2. Inner Clipping for Scrollable Viewport (preserves outer border and rounded corners!)
            let inner_clip_rect = Rect::from_min_max(
                pos2(rect.left() + 2.0, rect.top() + padding_y - 2.0),
                pos2(rect.right() - 2.0, rect.bottom() - padding_y + 2.0),
            );
            let items_clip = inner_clip_rect.intersect(ui.clip_rect());

            // 3. Saved Selection Highlight (Tier 1: Outlined Peach/Warning box with NO fill)
            if open_t > 0.01 {
                let items_painter = ui.painter().with_clip_rect(items_clip);
                if let Some(s_idx) = selected_pos {
                    let target_saved_y = rect.top() + padding_y + (s_idx as f32) * item_step - scroll_y;
                    if !state.saved_initialized {
                        state.saved_y_spring.reset(target_saved_y);
                        state.saved_h_spring.reset(card_h);
                        state.saved_initialized = true;
                    } else {
                        state.saved_y_spring.set_target(target_saved_y);
                        state.saved_h_spring.set_target(card_h);
                    }

                    let saved_y = state.saved_y_spring.value();
                    let saved_h = state.saved_h_spring.value();
                    let saved_rect = Rect::from_min_size(
                        pos2(rect.left() + 4.0, saved_y),
                        vec2(rect.width() - 8.0, saved_h),
                    );

                    let saved_stroke = Stroke::new(1.0, saved_color.linear_multiply(open_t));
                    items_painter.add(Shape::rect_stroke(saved_rect, Rounding::same(4.0), saved_stroke));
                }

                // Sliding focus pill (Tier 2: Only drawn if explicitly enabled and widget is focused)
                if self.show_internal_focus && self.focused {
                    if let Some(h_idx) = state.highlighted_index {
                        let target_focus_y = rect.top() + padding_y + (h_idx as f32) * item_step - scroll_y;
                        if !state.focus_initialized {
                            state.focus_y_spring.reset(target_focus_y);
                            state.focus_h_spring.reset(card_h);
                            state.focus_initialized = true;
                        } else {
                            state.focus_y_spring.set_target(target_focus_y);
                            state.focus_h_spring.set_target(card_h);
                        }

                        let focus_y = state.focus_y_spring.value();
                        let focus_h = state.focus_h_spring.value();
                        let focus_rect = Rect::from_min_size(
                            pos2(rect.left() + 4.0, focus_y),
                            vec2(rect.width() - 8.0, focus_h),
                        );

                        let focus_stroke = Stroke::new(1.0, accent_color.linear_multiply(open_t));
                        items_painter.add(Shape::rect_stroke(focus_rect, Rounding::same(4.0), focus_stroke));
                    }
                }
            }

            // 4. Render Items
            let collapsed_center_y = rect.top() + collapsed_h * 0.5;

            // A. When collapsed, render ONLY the selected label and icon (clean, un-cluttered)
            if open_t < 0.999 {
                let header_opacity = (1.0 - open_t).clamp(0.0, 1.0);
                if header_opacity > 0.01 && !state.is_open {
                    let mut cur_x = animated_rect.left() + padding.x;
                    if let Some(ref icon) = header_icon {
                        let i_g = icon.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
                        ui.painter().galley(
                            pos2(cur_x, collapsed_center_y + y_offset - i_g.size().y * 0.5),
                            i_g.clone(),
                            accent_color.linear_multiply(header_opacity),
                        );
                        cur_x += i_g.size().x + 6.0;
                    }
                    ui.painter().galley(
                        pos2(cur_x, collapsed_center_y + y_offset - text_layout.size().y * 0.5),
                        text_layout.clone(),
                        text_color.linear_multiply(header_opacity),
                    );
                }
            }

            // B. When open, render options list with items_painter (clipped inside viewport)
            if open_t > 0.01 {
                for (item_idx, opt) in self.options.iter().enumerate() {
                    let is_selected = selected_pos == Some(item_idx);
                    let is_choice_focused = state.is_open && self.focused && state.highlighted_index == Some(item_idx);

                    let expanded_row_top = padding_y + (item_idx as f32) * item_step - scroll_y;
                    let expanded_row_center_y = rect.top() + expanded_row_top + card_h * 0.5;

                    let item_center_y = if is_selected {
                        collapsed_center_y + open_t * (expanded_row_center_y - collapsed_center_y)
                    } else {
                        expanded_row_center_y
                    };

                    let opacity = (open_t * open_t).clamp(0.0, 1.0);
                    if opacity < 0.01 {
                        continue;
                    }

                    let row_rect = Rect::from_min_size(
                        pos2(rect.left() + 4.0, rect.top() + expanded_row_top),
                        vec2(rect.width() - 8.0, card_h),
                    );

                    // Skip culling if completely outside visible inner viewport
                    if row_rect.bottom() < inner_clip_rect.top() - 10.0 || row_rect.top() > inner_clip_rect.bottom() + 10.0 {
                        continue;
                    }

                    if let Some(ref custom_renderer) = self.item_renderer {
                        let item_ctx = DropdownItemContext {
                            option: opt,
                            index: item_idx,
                            is_selected,
                            is_highlighted: is_choice_focused,
                            is_disabled: opt.disabled,
                            rect: row_rect,
                            opacity,
                            palette: self.palette,
                        };
                        let mut child_ui = ui.child_ui(row_rect, *ui.layout());
                        child_ui.set_clip_rect(inner_clip_rect.intersect(ui.clip_rect()));
                        custom_renderer(&mut child_ui, &item_ctx);
                    } else {
                        let items_painter = ui.painter().with_clip_rect(items_clip);
                        // 1. Right-anchored metadata (never collides with label)
                        let mut right_x = row_rect.right() - 8.0;

                        // Active Checkmark Indicator
                        if is_selected {
                            items_painter.text(
                                pos2(right_x, item_center_y),
                                Align2::RIGHT_CENTER,
                                "✓",
                                FontId::monospace(13.0),
                                accent_color.linear_multiply(opacity),
                            );
                            right_x -= 16.0;
                        }

                        // Subtitle metadata (e.g. "24ms")
                        if let Some(ref sub) = opt.subtitle {
                            let sub_g = sub.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Small);
                            items_painter.galley(
                                pos2(right_x - sub_g.size().x, item_center_y - sub_g.size().y * 0.5),
                                sub_g.clone(),
                                subtext_color.linear_multiply(opacity),
                            );
                            right_x -= sub_g.size().x + 8.0;
                        }

                        // Micro-Badge (e.g. "Fast")
                        if let Some(ref badge) = opt.badge {
                            let badge_g = badge.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Small);
                            let b_w = badge_g.size().x + 8.0;
                            let b_h = 16.0;
                            let b_rect = Rect::from_min_size(
                                pos2(right_x - b_w, item_center_y - b_h * 0.5),
                                vec2(b_w, b_h),
                            );
                            let badge_col = opt.badge_color.unwrap_or(accent_color);

                            items_painter.add(Shape::rect_stroke(
                                b_rect,
                                Rounding::same(3.0),
                                Stroke::new(1.0, badge_col.linear_multiply(0.50 * opacity)),
                            ));
                            items_painter.galley(
                                pos2(b_rect.left() + 4.0, item_center_y - badge_g.size().y * 0.5),
                                badge_g,
                                badge_col.linear_multiply(opacity),
                            );
                            let _ = right_x;
                        }

                        // 2. Left-anchored content (Status Dot + Icon + Label + Description)
                        let slide_x = if is_selected { 0.0 } else { (1.0 - open_t) * 8.0 };
                        let mut left_x = row_rect.left() + 8.0 + slide_x;

                        // Status Dot Indicator
                        if let Some(dot_col) = opt.status_dot {
                            let dot_center = pos2(left_x + 3.0, item_center_y);
                            items_painter.circle_filled(dot_center, 3.0, dot_col.linear_multiply(opacity));
                            items_painter.circle_stroke(
                                dot_center,
                                4.5,
                                Stroke::new(1.0, dot_col.linear_multiply(0.35 * opacity)),
                            );
                            left_x += 12.0;
                        }

                        // Leading Icon
                        if let Some(ref icon) = opt.icon {
                            let default_icon_col = if is_selected || is_choice_focused { accent_color } else { subtext_color };
                            let icon_col = opt.icon_color.unwrap_or(default_icon_col);
                            let icon_g = icon.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
                            items_painter.galley(
                                pos2(left_x, item_center_y - icon_g.size().y * 0.5),
                                icon_g.clone(),
                                icon_col.linear_multiply(opacity),
                            );
                            left_x += icon_g.size().x + 6.0;
                        }

                        // Primary Label & Optional Description
                        let label_col = if opt.disabled {
                            if let Some(p) = self.palette { p.surface2 } else { Color32::GRAY }
                        } else if is_selected {
                            accent_color
                        } else if is_choice_focused {
                            accent_color
                        } else {
                            text_color
                        };

                        let label_g = opt.label.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);

                        if let Some(ref desc) = opt.description {
                            let desc_g = desc.clone().into_galley(ui, Some(false), f32::INFINITY, TextStyle::Small);
                            let total_text_h = label_g.size().y + desc_g.size().y + 2.0;
                            let start_y = item_center_y - total_text_h * 0.5;

                            items_painter.galley(
                                pos2(left_x, start_y),
                                label_g,
                                label_col.linear_multiply(opacity),
                            );
                            items_painter.galley(
                                pos2(left_x, start_y + total_text_h - desc_g.size().y),
                                desc_g,
                                subtext_color.linear_multiply(opacity),
                            );
                        } else {
                            items_painter.galley(
                                pos2(left_x, item_center_y - label_g.size().y * 0.5),
                                label_g,
                                label_col.linear_multiply(opacity),
                            );
                        }
                    }
                }

                // 5. Sleek Micro-Scrollbar Indicator
                if total_items > max_visible && open_t > 0.1 {
                    let track_top = rect.top() + padding_y;
                    let track_bottom = rect.bottom() - padding_y;
                    let track_h = (track_bottom - track_top).max(10.0);
                    let thumb_h = (visible_count as f32 / total_items as f32 * track_h).max(16.0);
                    let max_scroll_idx = total_items.saturating_sub(max_visible);
                    let max_scroll_px = max_scroll_idx as f32 * item_step;
                    let progress = if max_scroll_px > 0.0 {
                        (scroll_y / max_scroll_px).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let thumb_top = track_top + progress * (track_h - thumb_h);
                    let thumb_rect = Rect::from_min_size(
                        pos2(rect.right() - 6.0, thumb_top),
                        vec2(3.0, thumb_h),
                    );
                    ui.painter().add(Shape::rect_filled(
                        thumb_rect,
                        Rounding::same(1.5),
                        subtext_color.linear_multiply(0.35 * open_t),
                    ));
                }
            }

            // 6. Trailing Chevron Icon Indicator (Only in collapsed header)
            if self.show_chevron {
                let arrow_opacity = (1.0 - open_t).clamp(0.0, 1.0);
                if arrow_opacity > 0.01 {
                    let chevron_x = animated_rect.right() - 12.0;
                    let chev_cy = collapsed_center_y + y_offset;
                    let chev_col = if is_focused {
                        accent_color
                    } else {
                        subtext_color
                    };
                    let chev_col_faded = chev_col.linear_multiply(arrow_opacity);

                    let icon_str = self.chevron_icon.as_deref().unwrap_or("\u{f078}");
                    let font_id = FontId::proportional(11.0);

                    ui.painter().text(
                        pos2(chevron_x, chev_cy),
                        Align2::RIGHT_CENTER,
                        icon_str,
                        font_id,
                        chev_col_faded,
                    );
                }
            }
        }

        let is_open = state.is_open;

        let highlight_rect = if !state.is_open {
            animated_rect
        } else {
            let scroll_y = state.scroll_offset_spring.value();
            let h_idx = state.highlighted_index.unwrap_or_else(|| selected_pos.unwrap_or(0));

            let active_row_top = padding_y + (h_idx as f32) * item_step - scroll_y;
            let min_top = padding_y;
            let max_top = (rect.height() - padding_y - card_h).max(min_top);
            let safe_row_top = active_row_top.clamp(min_top, max_top);

            Rect::from_min_size(
                pos2(rect.left() + 4.0, rect.top() + safe_row_top),
                vec2(rect.width() - 8.0, card_h),
            )
        };

        if let Some(st) = temp_state {
            ui.data_mut(|d| d.insert_temp(id, st));
        }

        if selection_changed {
            response.mark_changed();
        }

        DropdownResponse {
            response,
            selected: new_selection,
            changed: selection_changed,
            is_open,
            is_pressed: header_pressed,
            clicked: header_clicked,
            highlight_rect,
            expanded_rect: full_expanded_rect,
            is_fully_visible,
            just_opened,
            is_navigating: is_key_navigating,
        }
    }
}
