//! Spring-animated text inputs and search bars.
//!
//! Provides [`TextInput`] and [`SearchBar`] with animated focus glow rings, icon prefixes,
//! clear buttons, and theme palette synchronization.
//!
//! # State Ownership
//!
//! Focus animations are tracked in ID temporary storage or an app-owned [`InputState`] (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Align2, Color32, FontId, Id, Rect, Response, Rounding, Sense, Shape, Stroke,
    TextEdit, TextStyle, Ui, Vec2, WidgetText,
};
use egui_spring::SpringCursor;
use egui_themes::ThemePalette;
use egui_vim_nav::{VimBufferState, VimMode};
use spring_core::{Spring, SpringParams};

/// Persistent animation state for text input focus effects.
/// Represents a deleted text slice being physically wiped/faded away by the reversing spring cursor.
#[derive(Clone, Debug)]
pub struct DeletedSegment {
    /// The string slice that was deleted.
    pub text: String,
    /// Character offset in the text where deletion occurred.
    pub char_offset: usize,
    /// Spring driving continuous alpha fade out of this deleted segment ($1.0 \to 0.0$).
    pub fade_spring: Spring,
}

impl DeletedSegment {
    pub fn new(text: String, char_offset: usize) -> Self {
        let mut spring = Spring::new(1.0, SpringParams::new(26.0, 0.70));
        spring.set_target(0.0);
        Self {
            text,
            char_offset,
            fade_spring: spring,
        }
    }
}

/// Active swipe reveal / wipe dissolution animation for dynamic text typing and deletion.
#[derive(Clone, Debug)]
pub struct TextSwipeAnimation {
    /// Character index where insertion/deletion occurred.
    pub split_char_idx: usize,
    /// Physical spring driving trailing text horizontal slide offset (in pixels).
    /// Insertion: starts at -inserted_width -> relaxes to 0.0.
    /// Deletion:  starts at +deleted_width  -> relaxes to 0.0.
    pub shift_spring: Spring,
    /// For typing: newly inserted text swiping in with fade (0.0 -> 1.0).
    pub inserted_text: Option<String>,
    pub insert_progress: Spring,
    /// For deletion: deleted text segment wiping out with fade (1.0 -> 0.0).
    pub deleted_text: Option<String>,
    pub deleted_x: f32,
    pub delete_progress: Spring,
}

impl TextSwipeAnimation {
    pub fn new_insert(split_char_idx: usize, inserted_text: String, inserted_width: f32) -> Self {
        let mut shift = Spring::new(-inserted_width, SpringParams::new(28.0, 0.75));
        shift.set_target(0.0);
        let mut insert = Spring::new(0.0, SpringParams::new(32.0, 0.85));
        insert.set_target(1.0);
        Self {
            split_char_idx,
            shift_spring: shift,
            inserted_text: Some(inserted_text),
            insert_progress: insert,
            deleted_text: None,
            deleted_x: 0.0,
            delete_progress: Spring::new(0.0, SpringParams::new(28.0, 0.70)),
        }
    }

    pub fn new_delete(split_char_idx: usize, deleted_text: String, deleted_width: f32, deleted_x: f32) -> Self {
        let mut shift = Spring::new(deleted_width, SpringParams::new(28.0, 0.75));
        shift.set_target(0.0);
        let mut del_prog = Spring::new(1.0, SpringParams::new(26.0, 0.70));
        del_prog.set_target(0.0);
        Self {
            split_char_idx,
            shift_spring: shift,
            inserted_text: None,
            insert_progress: Spring::new(1.0, SpringParams::new(28.0, 0.70)),
            deleted_text: Some(deleted_text),
            deleted_x,
            delete_progress: del_prog,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.shift_spring.update(dt);
        self.insert_progress.update(dt);
        self.delete_progress.update(dt);
    }

    pub fn is_settled(&self) -> bool {
        self.shift_spring.is_settled()
            && (self.inserted_text.is_none() || self.insert_progress.is_settled())
            && (self.deleted_text.is_none() || self.delete_progress.is_settled())
    }
}

/// Persistent animation state for text inputs with spring-animated focus rings.
#[derive(Clone, Debug)]
pub struct InputState {
    /// Spring driving focus glow ring expansion ($0.0 \to 1.0$).
    pub focus_spring: Spring,
    /// Spring driving fluid 4-corner cursor motion physics and highlight morphing.
    pub cursor_spring: SpringCursor,
    /// Buffer of pre-deletion text string (legacy field preserved for compatibility).
    pub deleted_text_buffer: Option<String>,
    /// Deleted text segment undergoing physical cursor wipe animation.
    pub deleted_segment: Option<DeletedSegment>,
    /// Tracks previous focus state to implement the 1-Frame Activation Shield.
    pub was_focused: bool,
    /// Smoothly interpolated mode stroke color for crossfading between Vim modes.
    pub current_mode_color: Option<Color32>,
    /// Last seen text buffer to detect text insertions and deletions.
    pub last_text: Option<String>,
    /// Active swipe reveal / slide animation.
    pub swipe_anim: Option<TextSwipeAnimation>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            focus_spring: Spring::new(0.0, SpringParams::new(26.0, 0.46)),
            cursor_spring: SpringCursor::new(SpringParams::new(26.0, 0.52)),
            deleted_text_buffer: None,
            deleted_segment: None,
            was_focused: false,
            current_mode_color: None,
            last_text: None,
            swipe_anim: None,
        }
    }
}

impl InputState {
    /// Updates focus spring and requests repaint if moving.
    pub fn update(&mut self, dt: f32, is_focused: bool, ctx: &egui::Context) {
        if is_focused && self.focus_spring.target < 0.5 {
            self.focus_spring.velocity = 18.0;
        }
        self.focus_spring.set_target(if is_focused { 1.0 } else { 0.0 });
        self.focus_spring.update(dt);

        if let Some(ref mut anim) = self.swipe_anim {
            anim.update(dt);
            if anim.is_settled() {
                self.swipe_anim = None;
            }
        }

        if let Some(ref mut del) = self.deleted_segment {
            del.fade_spring.update(dt);
            if del.fade_spring.is_settled() {
                self.deleted_segment = None;
            }
        }

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all focus and cursor springs have settled.
    pub fn is_settled(&self) -> bool {
        self.focus_spring.is_settled()
            && self.cursor_spring.is_settled()
            && self.deleted_segment.as_ref().map_or(true, |d| d.fade_spring.is_settled())
            && self.swipe_anim.as_ref().map_or(true, |s| s.is_settled())
    }
}

/// A theme-aware text input widget with spring-animated focus rings.
///
/// # Example
/// ```no_run
/// use egui_widgets::TextInput;
///
/// # egui::__run_test_ui(|ui| {
/// let mut query = String::new();
/// TextInput::new(&mut query)
///     .placeholder("Search packages...")
///     .icon("🔍")
///     .clear_button(true)
///     .show(ui);
/// # });
/// ```
pub struct TextInput<'a> {
    text: &'a mut String,
    placeholder: Option<&'a str>,
    icon: Option<WidgetText>,
    clear_button: bool,
    password: bool,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    focus_stroke: Option<Stroke>,
    text_color: Option<Color32>,
    rounding: Option<Rounding>,
    padding: Vec2,
    min_width: f32,
    desired_width: Option<f32>,
    desired_height: Option<f32>,
    glow_ring: bool,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut InputState>,
    vim_buffer: Option<&'a mut VimBufferState>,
    focused: bool,
    editing: bool,
    mode_indicator: bool,
    spawn_origin: Option<(Rect, f32)>,
}

impl<'a> TextInput<'a> {
    /// Creates a new text input bound to the given string buffer.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            icon: None,
            clear_button: true,
            password: false,
            fill: None,
            stroke: None,
            focus_stroke: None,
            text_color: None,
            rounding: None,
            padding: vec2(10.0, 6.0),
            min_width: 140.0,
            desired_width: None,
            desired_height: None,
            glow_ring: false,
            spring_params: SpringParams::new(26.0, 0.46),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
            vim_buffer: None,
            focused: false,
            editing: false,
            mode_indicator: true,
            spawn_origin: None,
        }
    }

    /// Sets whether to display the mode indicator label (`[NOR]`, `[INS]`) on the right.
    pub fn mode_indicator(mut self, show: bool) -> Self {
        self.mode_indicator = show;
        self
    }

    /// Binds an external [`VimBufferState`] to enable pure Vim modal editing.
    pub fn vim_buffer(mut self, buffer: &'a mut VimBufferState) -> Self {
        self.vim_buffer = Some(buffer);
        self
    }

    /// Sets whether this text input is selected by Vim navigation.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets whether this text input is actively in typing/editing mode.
    pub fn editing(mut self, editing: bool) -> Self {
        self.editing = editing;
        self
    }

    /// Sets an external highlight rectangle and rounding for focus morphing (e.g. when embedded in a Slider).
    pub fn spawn_origin(mut self, origin_rect: Rect, origin_rounding: f32) -> Self {
        self.spawn_origin = Some((origin_rect, origin_rounding));
        self
    }

    /// Sets placeholder hint text.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Sets an optional leading icon (e.g. search magnifying glass).
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Enables or disables the quick clear (`✖`) button (default `true`).
    pub fn clear_button(mut self, clear: bool) -> Self {
        self.clear_button = clear;
        self
    }

    /// Enables password masking mode.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Explicitly overrides background fill.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Explicitly overrides unfocused border stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides focused border stroke.
    pub fn focus_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.focus_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides text color.
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Explicitly overrides corner rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Sets inner margin padding (default `vec2(10.0, 6.0)`).
    pub fn padding(mut self, padding: impl Into<Vec2>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets minimum input width (default `140.0pt`).
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Sets explicit desired width in pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    /// Sets explicit desired width in pixels.
    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    /// Sets explicit desired height in pixels (default `32.0`).
    pub fn height(mut self, height: f32) -> Self {
        self.desired_height = Some(height);
        self
    }

    /// Enables or disables the outer expanding focus glow ring (default `true`).
    pub fn glow_ring(mut self, show: bool) -> Self {
        self.glow_ring = show;
        self
    }

    /// Sets custom spring physics parameters for the focus ring.
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

    /// Binds an external [`InputState`] struct.
    pub fn with_state(mut self, state: &'a mut InputState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the text input into the UI.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let height = self.desired_height.unwrap_or(32.0);
        let width = if let Some(w) = self.desired_width {
            w.max(self.min_width)
        } else if ui.layout().main_dir() == egui::Direction::LeftToRight {
            240.0f32.min(ui.available_width()).max(self.min_width)
        } else {
            ui.available_width().max(self.min_width)
        };
        let desired_size = vec2(width, height);

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        // Resolve colors
        let (bg_fill, base_stroke, focus_stroke, text_color, placeholder_color) =
            if let Some(p) = self.palette {
                (
                    self.fill.unwrap_or(p.crust),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.surface1)),
                    self.focus_stroke.unwrap_or(Stroke::new(1.5, p.accent)),
                    self.text_color.unwrap_or(p.text),
                    p.subtext1,
                )
            } else {
                let v = ui.visuals();
                (
                    self.fill.unwrap_or(v.extreme_bg_color),
                    self.stroke.unwrap_or(v.widgets.inactive.bg_stroke),
                    self.focus_stroke.unwrap_or(Stroke::new(1.5, v.selection.stroke.color)),
                    self.text_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                    v.widgets.noninteractive.fg_stroke.color.linear_multiply(0.5),
                )
            };

        let rounding = self.rounding.unwrap_or(Rounding::same(6.0));
        let dt = ui.input(|i| i.stable_dt).min(0.05);

        // Sub-layout for internal TextEdit
        let mut left_offset = self.padding.x;
        let mut right_offset = self.padding.x;

        if self.icon.is_some() {
            left_offset += 20.0;
        }
        if self.clear_button && !self.text.is_empty() {
            right_offset += 22.0;
        }

        let edit_rect = Rect::from_min_max(
            pos2(rect.left() + left_offset, rect.top() + self.padding.y),
            pos2(rect.right() - right_offset, rect.bottom() - self.padding.y),
        );

        let id = self.id_source.unwrap_or_else(|| {
            if let Some(p) = self.placeholder {
                ui.make_persistent_id(p)
            } else {
                response.id
            }
        });

        let mut temp_state = if self.external_state.is_none() {
            Some(ui.data_mut(|d| d.get_temp::<InputState>(id).unwrap_or_default()))
        } else {
            None
        };

        let state: &mut InputState = if let Some(ref mut ext) = self.external_state {
            ext
        } else {
            temp_state.as_mut().unwrap()
        };

        state.focus_spring.params = self.spring_params;
        state.cursor_spring.set_spring_params(self.spring_params);

        state.update(dt, self.focused, ui.ctx());

        // 1. Paint background, glow ring, and border BEFORE TextEdit so text is drawn on top!
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.add(Shape::rect_filled(rect, rounding, bg_fill));

            // Static base border stroke (clean boundary; focus highlighting is owned by outer SpringRect)
            if base_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(rect, rounding, base_stroke));
            }

            // Leading icon
            if let Some(ref icon) = self.icon {
                let icon_galley = icon.clone().into_galley(
                    ui,
                    Some(false),
                    f32::INFINITY,
                    TextStyle::Button,
                );
                let icon_pos = pos2(
                    rect.left() + self.padding.x,
                    rect.center().y - icon_galley.size().y * 0.5,
                );
                let icon_color = if self.focused {
                    focus_stroke.color
                } else {
                    placeholder_color
                };
                painter.galley(icon_pos, icon_galley, icon_color);
            }
        }

        // 2. Render text and caret
        let mut vim_mode_info = None;
        let edit_response = if let Some(vbuf) = self.vim_buffer {
            // Synchronize text if caller modified it externally
            if vbuf.text() != self.text.as_str() {
                vbuf.set_text(self.text.as_str());
            }

            // 1-Frame Activation Shield:
            // When transitioning from unfocused -> focused on this frame,
            // the activation keystroke (e.g. 'i', 'a', 'Enter', 'Space') in egui's
            // event queue belongs to the UI navigator layer, NOT to the internal text buffer.
            // Skip processing input on this activation frame to guarantee zero event leak.
            let just_activated = self.focused && !state.was_focused;
            state.was_focused = self.focused;

            // Let Vim engine process input when focused (excluding the initial activation frame)
            if self.focused && !just_activated {
                vbuf.handle_input(ui.ctx());
                *self.text = vbuf.text().to_owned();
            } else if just_activated {
                *self.text = vbuf.text().to_owned();
            }

            let mode = vbuf.mode();
            let pending = vbuf.parser.pending_keys_label();

            let target_mode_color = resolve_vim_mode_color(mode, self.palette, ui.visuals());
            let cur_mode_col = state.current_mode_color.unwrap_or(target_mode_color);
            let smoothed_mode_color = lerp_color(cur_mode_col, target_mode_color, (dt * 16.0).clamp(0.0, 1.0));
            state.current_mode_color = Some(smoothed_mode_color);
            if smoothed_mode_color != target_mode_color {
                ui.ctx().request_repaint();
            }

            vim_mode_info = Some((mode, pending, smoothed_mode_color));

            let resp = ui.interact(
                edit_rect,
                ui.make_persistent_id(self.placeholder.unwrap_or("vim_input")),
                Sense::click(),
            );
            if resp.clicked() {
                vbuf.mode = VimMode::Insert;
            }

            if ui.is_rect_visible(edit_rect) {
                let painter = ui.painter().with_clip_rect(edit_rect);
                let font_id = FontId::monospace(13.0);

                // Update deleted segment fade spring
                if let Some(ref mut del) = state.deleted_segment {
                    del.fade_spring.update(dt);
                    if del.fade_spring.is_settled() {
                        state.deleted_segment = None;
                    } else {
                        ui.ctx().request_repaint();
                    }
                }

                // Base text is ALWAYS the current valid text (100% stable, unclipped in normal mode, never disappears!)
                let display_text = if self.password {
                    "•".repeat(self.text.chars().count())
                } else {
                    self.text.clone()
                };

                let galley = painter.layout_no_wrap(display_text.clone(), font_id.clone(), text_color);
                // Stabilized vertical baseline: prevents 8px jumping when text transitions between empty and non-empty
                let text_pos = pos2(edit_rect.left(), edit_rect.center().y - 8.0);

                // Detect text insertion or deletion compared to previous frame's buffer
                if let Some(ref last_txt) = state.last_text {
                    let old_count = last_txt.chars().count();
                    let new_count = self.text.chars().count();
                    if new_count > old_count {
                        // Insertion: characters typed or pasted
                        let num_inserted = new_count - old_count;
                        let cursor_char_idx = self.text[..vbuf.cursor.min(self.text.len())].chars().count();
                        let split_char_idx = cursor_char_idx.saturating_sub(num_inserted);
                        let inserted_substr: String = self.text.chars().skip(split_char_idx).take(num_inserted).collect();
                        let ins_galley = painter.layout_no_wrap(inserted_substr.clone(), font_id.clone(), text_color);
                        let ins_w = ins_galley.size().x.max(7.8);
                        state.swipe_anim = Some(TextSwipeAnimation::new_insert(split_char_idx, inserted_substr, ins_w));
                        ui.ctx().request_repaint();
                    } else if new_count < old_count {
                        // Deletion: characters deleted (single char, backspace, or visual selection cut)
                        let num_deleted = old_count - new_count;
                        let cursor_char_idx = self.text[..vbuf.cursor.min(self.text.len())].chars().count();
                        let split_char_idx = cursor_char_idx.min(old_count);
                        let deleted_substr: String = last_txt.chars().skip(split_char_idx).take(num_deleted).collect();
                        let del_galley = painter.layout_no_wrap(deleted_substr.clone(), font_id.clone(), text_color);
                        let del_w = del_galley.size().x.max(7.8);

                        // Calculate position where the deleted characters were located
                        let prefix_substr: String = last_txt.chars().take(split_char_idx).collect();
                        let prefix_galley = painter.layout_no_wrap(prefix_substr, font_id.clone(), text_color);
                        let deleted_x = text_pos.x + prefix_galley.size().x;

                        state.swipe_anim = Some(TextSwipeAnimation::new_delete(split_char_idx, deleted_substr, del_w, deleted_x));
                        ui.ctx().request_repaint();
                    }
                }
                state.last_text = Some(self.text.clone());

                // Compute target character rect for the fluid spring cursor based on the CURRENT text and mode
                let (target_caret_rect, target_rounding, fill_mult, stroke_width) = if self.text.is_empty() {
                    let caret_x = edit_rect.left();
                    match mode {
                        VimMode::Insert => (
                            Rect::from_min_size(
                                pos2(caret_x, edit_rect.center().y - 8.0),
                                vec2(1.8, 16.0),
                            ),
                            0.5,
                            1.0,
                            0.0,
                        ),
                        VimMode::Replace => (
                            Rect::from_min_size(
                                pos2(caret_x, edit_rect.center().y + 5.0),
                                vec2(8.0, 3.0),
                            ),
                            0.5,
                            1.0,
                            1.0,
                        ),
                        VimMode::OperatorPending { operator, .. } => {
                            use egui_vim_nav::VimOperator;
                            match operator {
                                VimOperator::Delete => (
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y),
                                        vec2(8.0, 8.0),
                                    ),
                                    1.0,
                                    0.35,
                                    1.5,
                                ),
                                VimOperator::Change => (
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y - 8.0),
                                        vec2(8.0, 8.0),
                                    ),
                                    1.0,
                                    0.35,
                                    1.5,
                                ),
                                VimOperator::Yank => (
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y - 8.0),
                                        vec2(8.0, 16.0),
                                    ),
                                    1.5,
                                    0.0,
                                    2.0,
                                ),
                                _ => (
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y),
                                        vec2(8.0, 8.0),
                                    ),
                                    1.0,
                                    0.35,
                                    1.5,
                                ),
                            }
                        }
                        _ => (
                            Rect::from_min_size(
                                pos2(caret_x, edit_rect.center().y - 8.0),
                                vec2(8.0, 16.0),
                            ),
                            1.5,
                            1.0,
                            1.0,
                        ),
                    }
                } else {
                    let cursor_char_idx = self.text[..vbuf.cursor.min(self.text.len())].chars().count();
                    let cur = galley.from_ccursor(egui::text::CCursor::new(cursor_char_idx));
                    let cur_rect = galley.pos_from_cursor(&cur);
                    let caret_x = text_pos.x + cur_rect.left();
                    let next_cur = galley.from_ccursor(egui::text::CCursor::new(cursor_char_idx + 1));
                    let next_rect = galley.pos_from_cursor(&next_cur);
                    let char_w = (next_rect.left() - cur_rect.left()).max(8.0);

                    match mode {
                        VimMode::Normal => {
                            if vbuf.parser.is_pending_find() {
                                // Find target pending (f, t, F, T) -> hollow targeting frame
                                (
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y - 8.0),
                                        vec2(char_w, 16.0),
                                    ),
                                    1.5,
                                    0.20,
                                    1.5,
                                )
                            } else {
                                (
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y - 8.0),
                                        vec2(char_w, 16.0),
                                    ),
                                    1.5,
                                    1.0,
                                    1.0,
                                )
                            }
                        }
                        VimMode::Insert => (
                            Rect::from_min_size(
                                pos2(caret_x.max(edit_rect.left()), edit_rect.center().y - 8.0),
                                vec2(1.8, 16.0),
                            ),
                            0.5,
                            1.0,
                            0.0,
                        ),
                        VimMode::Replace => (
                            // Bottom underline bar for Replace mode (R) and Replace char (r)
                            Rect::from_min_size(
                                pos2(caret_x, edit_rect.center().y + 5.0),
                                vec2(char_w, 3.0),
                            ),
                            0.5,
                            1.0,
                            1.0,
                        ),
                        VimMode::OperatorPending { operator, .. } => {
                            use egui_vim_nav::VimOperator;
                            match operator {
                                VimOperator::Delete => (
                                    // Bottom half-block for Delete ('d')
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y),
                                        vec2(char_w, 8.0),
                                    ),
                                    1.0,
                                    1.0,
                                    1.0,
                                ),
                                VimOperator::Change => (
                                    // Top half-block for Change ('c')
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y - 8.0),
                                        vec2(char_w, 8.0),
                                    ),
                                    1.0,
                                    1.0,
                                    1.0,
                                ),
                                VimOperator::Yank => (
                                    // Hollow outline frame for Yank ('y')
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y - 8.0),
                                        vec2(char_w, 16.0),
                                    ),
                                    1.5,
                                    0.0,
                                    2.0,
                                ),
                                _ => (
                                    Rect::from_min_size(
                                        pos2(caret_x, edit_rect.center().y),
                                        vec2(char_w, 8.0),
                                    ),
                                    1.0,
                                    1.0,
                                    1.0,
                                ),
                            }
                        }
                        VimMode::Visual(_) => (
                            Rect::from_min_size(
                                pos2(caret_x, edit_rect.center().y - 8.0),
                                vec2(char_w, 16.0),
                            ),
                            1.5,
                            1.0,
                            1.0,
                        ),
                    }
                };

                let target_mode_color = resolve_vim_mode_color(mode, self.palette, ui.visuals());
                let cur_mode_col = state.current_mode_color.unwrap_or(target_mode_color);
                let smoothed_mode_color = lerp_color(cur_mode_col, target_mode_color, (dt * 16.0).clamp(0.0, 1.0));
                state.current_mode_color = Some(smoothed_mode_color);
                if smoothed_mode_color != target_mode_color {
                    ui.ctx().request_repaint();
                }

                // Macro-to-micro focus morphing:
                // When focused transitions from false -> true: spawn from the outer highlight rect!
                let (highlight_spawn_origin, highlight_spawn_rounding) = self
                    .spawn_origin
                    .unwrap_or_else(|| (rect.expand(3.0), 6.0));
                if self.focused && !state.cursor_spring.active {
                    state.cursor_spring.spawn_from(highlight_spawn_origin, highlight_spawn_rounding, target_caret_rect, target_rounding);
                } else if !self.focused && state.cursor_spring.active {
                    state.cursor_spring.exit_to(highlight_spawn_origin, highlight_spawn_rounding);
                }

                // Advance fluid 4-corner cursor simulation
                state.cursor_spring.update(target_caret_rect, self.focused, dt, ui.ctx());

                if self.text.is_empty() {
                    if let Some(ph) = self.placeholder {
                        painter.text(
                            pos2(edit_rect.left(), edit_rect.center().y),
                            Align2::LEFT_CENTER,
                            ph,
                            font_id.clone(),
                            placeholder_color,
                        );
                    }
                } else {
                    // Visual selection highlight
                    if let Some(range) = vbuf.selection_range() {
                        let min_char = self.text[..range.start.min(self.text.len())].chars().count();
                        let max_char = self.text[..range.end.min(self.text.len())].chars().count();
                        let cur1 = galley.from_ccursor(egui::text::CCursor::new(min_char));
                        let cur2 = galley.from_ccursor(egui::text::CCursor::new(max_char));
                        let r1 = galley.pos_from_cursor(&cur1);
                        let r2 = galley.pos_from_cursor(&cur2);
                        let sel_rect = Rect::from_min_max(
                            pos2(text_pos.x + r1.left().min(r2.left()), edit_rect.top() + 2.0),
                            pos2(text_pos.x + r1.left().max(r2.left()), edit_rect.bottom() - 2.0),
                        );
                        painter.rect_filled(sel_rect, Rounding::same(2.0), smoothed_mode_color.linear_multiply(0.25));
                    }

                    // Render Base Text with dynamic caret-driven swipe reveal and trailing slide
                    if let Some(ref anim) = state.swipe_anim {
                        let total_chars = display_text.chars().count();
                        let k = anim.split_char_idx.min(total_chars);

                        // 1. Prefix text (0..k)
                        let prefix_str: String = display_text.chars().take(k).collect();
                        let prefix_w = if !prefix_str.is_empty() {
                            let prefix_galley = painter.layout_no_wrap(prefix_str, font_id.clone(), text_color);
                            let pw = prefix_galley.size().x;
                            painter.galley(text_pos, prefix_galley, text_color);
                            pw
                        } else {
                            0.0
                        };

                        let cursor_pts = state.cursor_spring.corners.positions();
                        let cursor_min_x = cursor_pts[0].x.min(cursor_pts[3].x);
                        let cursor_max_x = cursor_pts[1].x.max(cursor_pts[2].x);

                        // 2. Insertion swipe case (Caret-Attached Reveal)
                        if let Some(ref ins_text) = anim.inserted_text {
                            let num_inserted = ins_text.chars().count();
                            let ins_str: String = display_text.chars().skip(k).take(num_inserted).collect();
                            let ins_alpha = anim.insert_progress.value().clamp(0.0, 1.0);
                            let ins_color = text_color.linear_multiply(ins_alpha);

                            let ins_galley = painter.layout_no_wrap(ins_str, font_id.clone(), ins_color);
                            let ins_w = ins_galley.size().x;
                            let ins_pos = pos2(text_pos.x + prefix_w, text_pos.y);

                            // Physical Caret-Attached Reveal Window:
                            // The character is revealed horizontally in direct lockstep with the cursor's leading edge
                            let reveal_min_x = text_pos.x + prefix_w;
                            let reveal_max_x = reveal_min_x + ins_w;
                            let clip_right = cursor_max_x.clamp(reveal_min_x, reveal_max_x + 4.0);
                            let reveal_clip = Rect::from_min_max(
                                pos2(reveal_min_x - 1.0, edit_rect.top() - 2.0),
                                pos2(clip_right, edit_rect.bottom() + 2.0),
                            );
                            let ins_painter = painter.with_clip_rect(reveal_clip);
                            ins_painter.galley(ins_pos, ins_galley, ins_color);

                            // Trailing suffix text (k + num_inserted..) sliding smoothly into place
                            let suffix_str: String = display_text.chars().skip(k + num_inserted).collect();
                            if !suffix_str.is_empty() {
                                let suffix_galley = painter.layout_no_wrap(suffix_str, font_id.clone(), text_color);
                                let shift_offset = anim.shift_spring.value();
                                let suffix_pos = pos2(text_pos.x + prefix_w + ins_w + shift_offset, text_pos.y);
                                painter.galley(suffix_pos, suffix_galley, text_color);
                            }
                        } else if let Some(ref del_text) = anim.deleted_text {
                            // 3. Deletion wipe-out dissolution case (Caret-Attached Wipe)
                            let del_alpha = anim.delete_progress.value().clamp(0.0, 1.0);
                            if del_alpha > 0.01 {
                                let del_color = text_color.linear_multiply(del_alpha * 0.75);
                                let del_galley = painter.layout_no_wrap(del_text.clone(), font_id.clone(), del_color);
                                let del_w = del_galley.size().x;
                                let del_min_x = anim.deleted_x;
                                let del_max_x = del_min_x + del_w;

                                // Physical Caret-Attached Wipe Window:
                                // As the cursor retreats leftward, the visible portion shrinks to cursor_min_x
                                let wipe_right = cursor_min_x.clamp(del_min_x, del_max_x);
                                if wipe_right > del_min_x {
                                    let wipe_clip = Rect::from_min_max(
                                        pos2(del_min_x - 1.0, edit_rect.top() - 2.0),
                                        pos2(wipe_right, edit_rect.bottom() + 2.0),
                                    );
                                    let del_painter = painter.with_clip_rect(wipe_clip);
                                    del_painter.galley(pos2(anim.deleted_x, text_pos.y), del_galley, del_color);
                                }
                            }

                            // Trailing suffix text (k..) sliding left to fill gap
                            let suffix_str: String = display_text.chars().skip(k).collect();
                            if !suffix_str.is_empty() {
                                let suffix_galley = painter.layout_no_wrap(suffix_str, font_id.clone(), text_color);
                                let shift_offset = anim.shift_spring.value();
                                let suffix_pos = pos2(text_pos.x + prefix_w + shift_offset, text_pos.y);
                                painter.galley(suffix_pos, suffix_galley, text_color);
                            }
                        } else {
                            painter.galley(text_pos, galley.clone(), text_color);
                        }
                    } else {
                        // Fully settled: 100% stable single galley
                        painter.galley(text_pos, galley.clone(), text_color);
                    }
                }

                // Render animated cursor polygon with smooth mode color crossfading
                let cursor_fill = smoothed_mode_color.linear_multiply(fill_mult);
                let cursor_stroke = Stroke::new(stroke_width, smoothed_mode_color);
                state.cursor_spring.paint(ui.painter(), cursor_fill, cursor_stroke);

                // High-Contrast Character Inversion:
                // For solid block modes (Normal, Visual, OperatorPending Delete/Change), re-render the character
                // intersecting the cursor polygon using inverted palette.crust foreground text on top of the solid block!
                if self.focused && !self.text.is_empty() && (mode == VimMode::Normal || mode.is_operator_pending() || mode.is_visual()) && fill_mult >= 0.8 {
                    let inverted_text_color = if let Some(p) = self.palette {
                        p.crust
                    } else {
                        ui.visuals().extreme_bg_color
                    };

                    let cursor_pts = state.cursor_spring.corners.positions();
                    let c_min_x = cursor_pts[0].x.min(cursor_pts[3].x);
                    let c_max_x = cursor_pts[1].x.max(cursor_pts[2].x);
                    let c_min_y = cursor_pts[0].y.min(cursor_pts[1].y);
                    let c_max_y = cursor_pts[2].y.max(cursor_pts[3].y);

                    let cursor_clip = Rect::from_min_max(
                        pos2(c_min_x - 0.5, c_min_y - 0.5),
                        pos2(c_max_x + 0.5, c_max_y + 0.5),
                    );

                    let inv_painter = ui.painter().with_clip_rect(cursor_clip.intersect(edit_rect));
                    let inv_galley = inv_painter.layout_no_wrap(display_text, font_id.clone(), inverted_text_color);
                    inv_painter.galley(text_pos, inv_galley, inverted_text_color);
                }
            }

            resp
        } else {
            // Standard egui::TextEdit fallback for non-Vim text inputs
            let mut edit = TextEdit::singleline(self.text)
                .password(self.password)
                .text_color(text_color)
                .frame(false);

            if let Some(ph) = self.placeholder {
                edit = edit.hint_text(WidgetText::from(ph).color(placeholder_color));
            }

            let edit_response = ui.put(edit_rect, edit);

            if self.editing && !edit_response.has_focus() {
                edit_response.request_focus();
            }

            let has_focus = edit_response.has_focus();

            if has_focus && ui.input(|i| {
                i.key_pressed(egui::Key::Escape) || (i.modifiers.ctrl && (
                    i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::L)
                    || i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowRight)
                ))
            }) {
                edit_response.surrender_focus();
                ui.ctx().memory_mut(|m| m.stop_text_input());
            }

            edit_response
        };

        // 3. Quick clear button
        if self.clear_button && !self.text.is_empty() {
            let clear_rect = Rect::from_center_size(
                pos2(rect.right() - 14.0, rect.center().y),
                vec2(16.0, 16.0),
            );
            let clear_id = ui.make_persistent_id(self.placeholder.unwrap_or("txt_clear")).with("clear");
            let clear_resp = ui.interact(clear_rect, clear_id, Sense::click());
            let clear_color = if clear_resp.hovered() { text_color } else { placeholder_color };
            ui.painter().text(
                clear_rect.center(),
                Align2::CENTER_CENTER,
                "✖",
                FontId::monospace(10.0),
                clear_color,
            );
            if clear_resp.clicked() {
                self.text.clear();
            }
        }

        // Mode hint indicator on the right
        if self.focused && self.mode_indicator {
            let hint_pos = pos2(rect.right() - right_offset - 4.0, rect.center().y);
            if let Some((mode, pending, badge_col)) = vim_mode_info {
                let mode_label = mode.label();
                let badge_text = if pending.is_empty() {
                    format!("[{}]", mode_label)
                } else {
                    format!("[{} {}]", pending, mode_label)
                };
                ui.painter().text(
                    hint_pos,
                    Align2::RIGHT_CENTER,
                    badge_text,
                    FontId::monospace(9.5),
                    badge_col,
                );
            } else if self.text.is_empty() {
                let (badge_text, badge_col) = if self.editing || edit_response.has_focus() {
                    ("[esc] done", focus_stroke.color.linear_multiply(0.7))
                } else {
                    ("[i] edit", placeholder_color.linear_multiply(0.8))
                };
                ui.painter().text(
                    hint_pos,
                    Align2::RIGHT_CENTER,
                    badge_text,
                    FontId::monospace(9.5),
                    badge_col,
                );
            }
        }

        if let Some(st) = temp_state {
            ui.data_mut(|d| d.insert_temp(id, st));
        }

        edit_response.union(response)
    }
}

/// Resolves the theme color token associated with a given `VimMode`.
pub fn resolve_vim_mode_color(
    mode: VimMode,
    palette: Option<&ThemePalette>,
    visuals: &egui::Visuals,
) -> Color32 {
    use egui_vim_nav::VimOperator;
    if let Some(p) = palette {
        match mode {
            VimMode::Normal => p.info,
            VimMode::Insert => p.accent,
            VimMode::Visual(_) => p.warning,
            VimMode::Replace => p.danger,
            VimMode::OperatorPending { operator, .. } => match operator {
                VimOperator::Delete => p.danger,
                VimOperator::Change => p.accent,
                VimOperator::Yank => p.info,
                _ => p.info_alt,
            },
        }
    } else {
        match mode {
            VimMode::Normal => Color32::from_rgb(137, 180, 250),
            VimMode::Insert => visuals.selection.stroke.color,
            VimMode::Visual(_) => Color32::from_rgb(250, 179, 135),
            VimMode::Replace => Color32::from_rgb(243, 139, 168),
            VimMode::OperatorPending { operator, .. } => match operator {
                VimOperator::Delete => Color32::from_rgb(243, 139, 168),
                VimOperator::Change => visuals.selection.stroke.color,
                VimOperator::Yank => Color32::from_rgb(137, 180, 250),
                _ => Color32::from_rgb(203, 166, 247),
            },
        }
    }
}

/// Helper function to interpolate between two `Color32` values.
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}
