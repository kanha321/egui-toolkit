//! Spring-animated text inputs and search bars.
//!
//! Provides [`TextInput`] and [`SearchBar`] with animated focus glow rings, icon prefixes,
//! clear buttons, and theme palette synchronization.
//!
//! # State Ownership
//!
//! Focus animations are tracked in ID temporary storage or an app-owned [`InputState`] (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Align2, Color32, FontId, Id, LayerId, Order, Rect, Response, Rounding, Sense, Shape, Stroke,
    TextEdit, TextStyle, Ui, Vec2, WidgetText,
};
use egui_spring::SpringCursor;
use egui_themes::ThemePalette;
use egui_vim_nav::{VimBufferState, VimMode, VisualType};
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

/// State of an active selection drag-and-drop operation.
#[derive(Clone, Debug)]
pub struct DragSelectionState {
    /// Sliced text content being dragged.
    pub text: String,
    /// Last seen mouse pointer position for velocity estimation.
    pub last_pos: egui::Pos2,
    /// Offset from pointer position to top-left of dragged text slice (grab offset).
    pub grab_offset: egui::Vec2,
    /// Estimated pointer velocity (pixels / second).
    pub velocity: egui::Vec2,
    /// Target drop character index in remaining text buffer.
    pub target_char_idx: usize,
    /// Character index where text was extracted from in original buffer.
    pub split_char_idx: usize,
    /// Spring animating the closure of the extraction gap (extracted_width -> 0.0).
    pub close_spring: Spring,
}

/// Physics-driven 2D spring flight animation when releasing dragged text.
#[derive(Clone, Debug)]
pub struct DropFlightAnim {
    /// Sliced text being flown into place.
    pub text: String,
    /// 2D spring for horizontal position.
    pub x_spring: Spring,
    /// 2D spring for vertical position.
    pub y_spring: Spring,
    /// Spring driving scale transition (1.10 -> 1.0) on landing.
    pub scale_spring: Spring,
    /// Spring driving the parting gap opening for the incoming text (0.0 -> target_gap_width).
    pub gap_spring: Spring,
    /// Target destination position on baseline.
    pub dest_pos: egui::Pos2,
    /// Character index where text was inserted in the final text buffer.
    pub split_char_idx: usize,
    /// Target width of the opening gap.
    pub target_gap_width: f32,
    /// Destination byte range in newly spliced text.
    pub dest_range: std::ops::Range<usize>,
}

impl DropFlightAnim {
    pub fn new(
        text: String,
        start_pos: egui::Pos2,
        velocity: egui::Vec2,
        dest_pos: egui::Pos2,
        split_char_idx: usize,
        target_gap_width: f32,
        dest_range: std::ops::Range<usize>,
    ) -> Self {
        let mut x_spring = Spring::new(start_pos.x, SpringParams::new(26.0, 0.65));
        x_spring.velocity = velocity.x;
        x_spring.set_target(dest_pos.x);

        let mut y_spring = Spring::new(start_pos.y, SpringParams::new(26.0, 0.65));
        y_spring.velocity = velocity.y;
        y_spring.set_target(dest_pos.y);

        let mut scale_spring = Spring::new(1.10, SpringParams::new(26.0, 0.65));
        scale_spring.set_target(1.0);

        let mut gap_spring = Spring::new(0.0, SpringParams::new(26.0, 0.65));
        gap_spring.set_target(target_gap_width);

        Self {
            text,
            x_spring,
            y_spring,
            scale_spring,
            gap_spring,
            dest_pos,
            split_char_idx,
            target_gap_width,
            dest_range,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x_spring.set_target(self.dest_pos.x);
        self.y_spring.set_target(self.dest_pos.y);
        self.scale_spring.set_target(1.0);
        self.gap_spring.set_target(self.target_gap_width);
        self.x_spring.update(dt);
        self.y_spring.update(dt);
        self.scale_spring.update(dt);
        self.gap_spring.update(dt);
    }

    pub fn is_settled(&self) -> bool {
        self.x_spring.is_settled()
            && self.y_spring.is_settled()
            && self.scale_spring.is_settled()
            && self.gap_spring.is_settled()
    }
}

/// Selection overlay fade-out state when selected text is pulled / extracted into a drag.
#[derive(Clone, Debug)]
pub struct PullFade {
    /// Rect of the selection overlay that was lifted.
    pub rect: egui::Rect,
    /// Mode color of the selection when pulled.
    pub color: Color32,
    /// Spring driving continuous alpha fade out ($1.0 \to 0.0$).
    pub fade_spring: Spring,
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
    /// Spring driving smooth horizontal autoscroll when text exceeds input width.
    pub scroll_spring: Spring,
    /// Active text selection drag operation.
    pub drag_selection: Option<DragSelectionState>,
    /// Physics-driven 2D spring flight animation for dropped text docking into place.
    pub drop_flight: Option<DropFlightAnim>,
    /// Spring driving smooth selection overlay fade-in ($0.0 \to 1.0$) on drop release / landing.
    pub selection_fade_spring: Spring,
    /// Active selection overlay fade-out animation when text is pulled into a drag.
    pub pull_fade: Option<PullFade>,
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
            scroll_spring: Spring::new(0.0, SpringParams::new(26.0, 0.48)),
            drag_selection: None,
            drop_flight: None,
            selection_fade_spring: Spring::new(1.0, SpringParams::new(22.0, 0.60)),
            pull_fade: None,
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
        self.scroll_spring.update(dt);
        self.selection_fade_spring.update(dt);

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

        if let Some(ref mut flight) = self.drop_flight {
            flight.update(dt);
            if flight.is_settled() {
                self.drop_flight = None;
            }
        }

        if let Some(ref mut drag_st) = self.drag_selection {
            drag_st.close_spring.update(dt);
        }

        if let Some(ref mut pull) = self.pull_fade {
            pull.fade_spring.update(dt);
            if pull.fade_spring.is_settled() {
                self.pull_fade = None;
            }
        }

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all focus, flight, and cursor springs have settled.
    pub fn is_settled(&self) -> bool {
        self.focus_spring.is_settled()
            && self.cursor_spring.is_settled()
            && self.scroll_spring.is_settled()
            && self.deleted_segment.as_ref().map_or(true, |d| d.fade_spring.is_settled())
            && self.swipe_anim.as_ref().map_or(true, |s| s.is_settled())
            && self.drop_flight.as_ref().map_or(true, |f| f.is_settled())
            && self.drag_selection.as_ref().map_or(true, |d| d.close_spring.is_settled())
            && self.selection_fade_spring.is_settled()
            && self.pull_fade.as_ref().map_or(true, |p| p.fade_spring.is_settled())
    }
}

/// Horizontal text alignment within a [`TextInput`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextAlign {
    /// Align text to the left / leading edge (default).
    #[default]
    Left,
    /// Align text to the center.
    Center,
    /// Align text to the right / trailing edge.
    Right,
}

/// A theme-aware text input widget with spring-animated focus rings.
///
/// # Example
/// ```no_run
/// use egui_widgets::{TextInput, TextAlign};
///
/// # egui::__run_test_ui(|ui| {
/// let mut query = String::new();
/// TextInput::new(&mut query)
///     .placeholder("Search packages...")
///     .icon("🔍")
///     .clear_button(true)
///     .align_center()
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
    align: TextAlign,
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
            align: TextAlign::Left,
        }
    }

    /// Sets the horizontal text alignment (Left, Center, Right). Default is [`TextAlign::Left`].
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Sets horizontal text alignment to left (default).
    pub fn align_left(self) -> Self {
        self.align(TextAlign::Left)
    }

    /// Sets horizontal text alignment to center.
    pub fn align_center(self) -> Self {
        self.align(TextAlign::Center)
    }

    /// Sets horizontal text alignment to right.
    pub fn align_right(self) -> Self {
        self.align(TextAlign::Right)
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

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

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

            let click_id = id.with("vim_click");
            let resp = ui.interact(
                rect,
                click_id,
                Sense::click_and_drag(),
            );
            let interact_resp = response.union(resp);

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
                let mut display_text = if self.password {
                    "•".repeat(self.text.chars().count())
                } else {
                    self.text.clone()
                };

                let mut galley = painter.layout_no_wrap(display_text.clone(), font_id.clone(), text_color);
                let view_width = edit_rect.width();
                let mut text_width = galley.size().x;

                let scroll_x = state.scroll_spring.value();
                let base_text_x = match self.align {
                    TextAlign::Left => edit_rect.left(),
                    TextAlign::Center => {
                        if text_width <= view_width {
                            edit_rect.left() + ((view_width - text_width) * 0.5).max(0.0)
                        } else {
                            edit_rect.left()
                        }
                    }
                    TextAlign::Right => {
                        if text_width <= view_width {
                            edit_rect.left() + (view_width - text_width).max(0.0)
                        } else {
                            edit_rect.left()
                        }
                    }
                };
                // Stabilized vertical baseline with horizontal autoscroll displacement
                let text_pos = pos2(base_text_x - scroll_x, edit_rect.center().y - 8.0);

                // Precision Mouse Hit-Testing & Multi-Click / Drag Selection / Selection Move
                let pointer_pos = interact_resp
                    .interact_pointer_pos()
                    .or_else(|| ui.input(|i| i.pointer.latest_pos().or(i.pointer.hover_pos())));
                let press_origin = ui.input(|i| i.pointer.press_origin()).or(pointer_pos);

                if interact_resp.triple_clicked() {
                    // Triple-click: Select entire text buffer in Visual mode
                    let total_chars = self.text.chars().count();
                    let last_char_idx = total_chars.saturating_sub(1);
                    vbuf.anchor = Some(0);
                    vbuf.cursor = char_index_to_byte_offset(self.text, last_char_idx);
                    vbuf.mode = VimMode::Visual(VisualType::Character);
                    *self.text = vbuf.text().to_owned();
                    display_text = if self.password { "•".repeat(self.text.chars().count()) } else { self.text.clone() };
                    galley = painter.layout_no_wrap(display_text.clone(), font_id.clone(), text_color);
                    text_width = galley.size().x;
                    state.selection_fade_spring.reset(1.0);
                    state.drag_selection = None;
                    ui.ctx().request_repaint();
                } else if interact_resp.double_clicked() {
                    // Double-click: Select word under pointer in Visual mode
                    if let Some(pos) = pointer_pos {
                        let char_idx = pointer_to_char_index(pos, text_pos, &galley, &display_text);
                        let byte_offset = char_index_to_byte_offset(self.text, char_idx);
                        let (start_byte, end_byte) = word_bounds_around_byte(self.text, byte_offset);
                        let word_end_char = byte_offset_to_char_index(self.text, end_byte);
                        let word_last_char = word_end_char.saturating_sub(1);
                        let last_char_byte = char_index_to_byte_offset(self.text, word_last_char);
                        vbuf.anchor = Some(start_byte);
                        vbuf.cursor = last_char_byte;
                        vbuf.mode = VimMode::Visual(VisualType::Character);
                        *self.text = vbuf.text().to_owned();
                        display_text = if self.password { "•".repeat(self.text.chars().count()) } else { self.text.clone() };
                        galley = painter.layout_no_wrap(display_text.clone(), font_id.clone(), text_color);
                        text_width = galley.size().x;
                        state.selection_fade_spring.reset(1.0);
                        state.drag_selection = None;
                        ui.ctx().request_repaint();
                    }
                } else if interact_resp.dragged() {
                    if state.drag_selection.is_none() && interact_resp.drag_started() {
                        // Check if drag started INSIDE an existing active visual selection range
                        if let (Some(range), Some(origin_pos)) = (vbuf.selection_range(), press_origin) {
                            let origin_char = pointer_to_char_index(origin_pos, text_pos, &galley, &display_text);
                            let origin_byte = char_index_to_byte_offset(self.text, origin_char);
                            if origin_byte >= range.start && origin_byte < range.end {
                                let selected_slice = safe_byte_slice(self.text, range.clone()).to_string();
                                let sel_start_char = byte_offset_to_char_index(self.text, range.start);
                                let sel_start_cur = galley.from_ccursor(egui::text::CCursor::new(sel_start_char));
                                let sel_start_x = text_pos.x + galley.pos_from_cursor(&sel_start_cur).left();
                                let sel_start_pos = pos2(sel_start_x, text_pos.y);
                                let grab_offset = sel_start_pos - origin_pos;

                                let slice_galley = painter.layout_no_wrap(selected_slice.clone(), font_id.clone(), text_color);
                                let extracted_width = slice_galley.size().x;
                                let mut close_spring = Spring::new(extracted_width, SpringParams::new(26.0, 0.65));
                                close_spring.set_target(0.0);

                                // Record pull fade-out for the selection overlay
                                let min_char = byte_offset_to_char_index(self.text, range.start);
                                let max_char = byte_offset_to_char_index(self.text, range.end);
                                let cur1 = galley.from_ccursor(egui::text::CCursor::new(min_char));
                                let cur2 = galley.from_ccursor(egui::text::CCursor::new(max_char));
                                let r1 = galley.pos_from_cursor(&cur1);
                                let r2 = galley.pos_from_cursor(&cur2);
                                let sel_rect = Rect::from_min_max(
                                    pos2(text_pos.x + r1.left().min(r2.left()), edit_rect.top() + 2.0),
                                    pos2(text_pos.x + r1.left().max(r2.left()), edit_rect.bottom() - 2.0),
                                );
                                let mut pull_spring = Spring::new(1.0, SpringParams::new(26.0, 0.70));
                                pull_spring.set_target(0.0);
                                state.pull_fade = Some(PullFade {
                                    rect: sel_rect,
                                    color: smoothed_mode_color,
                                    fade_spring: pull_spring,
                                });

                                // Immediately extract the text from the buffer and exit Visual mode!
                                let mut remaining_text = self.text.clone();
                                safe_replace_range(&mut remaining_text, range.clone(), "");
                                *self.text = remaining_text.clone();
                                vbuf.set_text(&remaining_text);
                                vbuf.anchor = None;
                                vbuf.cursor = char_index_to_byte_offset(&remaining_text, sel_start_char);
                                vbuf.mode = VimMode::Insert;
                                state.swipe_anim = None;

                                display_text = if self.password { "•".repeat(self.text.chars().count()) } else { self.text.clone() };
                                galley = painter.layout_no_wrap(display_text.clone(), font_id.clone(), text_color);
                                text_width = galley.size().x;

                                state.drag_selection = Some(DragSelectionState {
                                    text: selected_slice,
                                    last_pos: origin_pos,
                                    grab_offset,
                                    velocity: egui::Vec2::ZERO,
                                    target_char_idx: sel_start_char.min(remaining_text.chars().count()),
                                    split_char_idx: sel_start_char,
                                    close_spring,
                                });
                                ui.ctx().request_repaint();
                            }
                        }
                    }

                    if let Some(ref mut drag_st) = state.drag_selection {
                        // Dragging selected text: update floating position & target insertion index
                        if let Some(pos) = pointer_pos {
                            let dt_safe = dt.max(0.001);
                            drag_st.velocity = (pos - drag_st.last_pos) / dt_safe;
                            drag_st.last_pos = pos;

                            let drop_char = pointer_to_char_index(pos, text_pos, &galley, &display_text);
                            drag_st.target_char_idx = drop_char.min(self.text.chars().count());

                            // Edge autoscroll during drag
                            if pos.x > edit_rect.right() - 20.0 {
                                state.scroll_spring.set_target(state.scroll_spring.target + 8.0);
                            } else if pos.x < edit_rect.left() + 20.0 {
                                state.scroll_spring.set_target((state.scroll_spring.target - 8.0).max(0.0));
                            }
                            ui.ctx().request_repaint();
                        }
                    } else {
                        // Normal click-and-drag: Establish anchor at press origin and expand selection to active pointer
                        if let (Some(origin_pos), Some(pos)) = (press_origin, pointer_pos) {
                            let start_char_idx = pointer_to_char_index(origin_pos, text_pos, &galley, &display_text);
                            let cur_char_idx = pointer_to_char_index(pos, text_pos, &galley, &display_text);

                            if cur_char_idx > start_char_idx {
                                let start_byte = char_index_to_byte_offset(self.text, start_char_idx);
                                let last_char_byte = char_index_to_byte_offset(self.text, cur_char_idx.saturating_sub(1));
                                vbuf.anchor = Some(start_byte);
                                vbuf.cursor = last_char_byte;
                                vbuf.mode = VimMode::Visual(VisualType::Character);
                            } else if cur_char_idx < start_char_idx {
                                let last_start_byte = char_index_to_byte_offset(self.text, start_char_idx.saturating_sub(1));
                                let cur_byte = char_index_to_byte_offset(self.text, cur_char_idx);
                                vbuf.anchor = Some(last_start_byte);
                                vbuf.cursor = cur_byte;
                                vbuf.mode = VimMode::Visual(VisualType::Character);
                            } else {
                                let cur_byte = char_index_to_byte_offset(self.text, cur_char_idx);
                                vbuf.anchor = None;
                                vbuf.cursor = cur_byte;
                                vbuf.mode = VimMode::Insert;
                            }
                            *self.text = vbuf.text().to_owned();
                            state.selection_fade_spring.reset(1.0);

                            // Edge autoscroll during drag
                            if pos.x > edit_rect.right() - 20.0 {
                                state.scroll_spring.set_target(state.scroll_spring.target + 8.0);
                            } else if pos.x < edit_rect.left() + 20.0 {
                                state.scroll_spring.set_target((state.scroll_spring.target - 8.0).max(0.0));
                            }
                            ui.ctx().request_repaint();
                        }
                    }
                } else if interact_resp.clicked() {
                    // Single-click: Place caret at exact clicked character index
                    if let Some(pos) = pointer_pos {
                        let char_idx = pointer_to_char_index(pos, text_pos, &galley, &display_text);
                        let byte_offset = char_index_to_byte_offset(self.text, char_idx);
                        vbuf.cursor = byte_offset;
                        vbuf.anchor = None;
                        vbuf.mode = VimMode::Insert;
                        *self.text = vbuf.text().to_owned();
                        display_text = if self.password { "•".repeat(self.text.chars().count()) } else { self.text.clone() };
                        galley = painter.layout_no_wrap(display_text.clone(), font_id.clone(), text_color);
                        text_width = galley.size().x;
                        state.drag_selection = None;
                        ui.ctx().request_repaint();
                    }
                }

                // Handle drop release on drag stop
                if state.drag_selection.is_some() && (!ui.input(|i| i.pointer.primary_down()) || interact_resp.drag_stopped()) {
                    if let Some(drag_st) = state.drag_selection.take() {
                        let selected_text = drag_st.text;
                        let target_char = drag_st.target_char_idx.min(self.text.chars().count());
                        let target_byte = char_index_to_byte_offset(self.text, target_char);

                        let mut new_text = self.text.clone();
                        new_text.insert_str(target_byte, &selected_text);
                        let new_range = target_byte..(target_byte + selected_text.len());

                        let dropped_char_count = selected_text.chars().count();
                        let last_dropped_char_idx = target_char + dropped_char_count.saturating_sub(1);
                        let last_dropped_byte = char_index_to_byte_offset(&new_text, last_dropped_char_idx);

                        *self.text = new_text.clone();
                        vbuf.set_text(new_text.as_str());
                        vbuf.anchor = Some(new_range.start);
                        vbuf.cursor = last_dropped_byte;
                        vbuf.mode = VimMode::Visual(VisualType::Character);

                        let mut drop_sel_spring = Spring::new(0.0, SpringParams::new(22.0, 0.60));
                        drop_sel_spring.set_target(1.0);
                        state.selection_fade_spring = drop_sel_spring;

                        // Destination baseline landing coordinate from current pre-drop galley layout
                        let dest_cur = galley.from_ccursor(egui::text::CCursor::new(target_char));
                        let dest_x = text_pos.x + galley.pos_from_cursor(&dest_cur).left();
                        let dest_pos = pos2(dest_x, text_pos.y);

                        let slice_galley = painter.layout_no_wrap(selected_text.clone(), font_id.clone(), text_color);
                        let target_gap_width = slice_galley.size().x;

                        // Refresh layout with newly spliced text for downstream rendering
                        display_text = if self.password { "•".repeat(self.text.chars().count()) } else { self.text.clone() };
                        galley = painter.layout_no_wrap(display_text.clone(), font_id.clone(), text_color);
                        text_width = galley.size().x;

                        // The ghost text position on the final drag frame:
                        let flight_start_pos = drag_st.last_pos + drag_st.grab_offset;

                        // Launch 2D spring flight from exact ghost release point to dest_pos
                        state.swipe_anim = None;
                        state.drop_flight = Some(DropFlightAnim::new(
                            selected_text,
                            flight_start_pos,
                            drag_st.velocity,
                            dest_pos,
                            target_char,
                            target_gap_width,
                            new_range,
                        ));
                        ui.ctx().request_repaint();
                    }
                }

                // Determine cursor position in text space for autoscroll calculation
                let cursor_char_idx = byte_offset_to_char_index(self.text, vbuf.cursor);
                let cur = galley.from_ccursor(egui::text::CCursor::new(cursor_char_idx));
                let cursor_local_x = galley.pos_from_cursor(&cur).left();

                // Compute horizontal autoscroll bounds to keep cursor in view
                let max_scroll = (text_width - view_width + 30.0).max(0.0);
                if self.focused {
                    let margin_left = 20.0;
                    let margin_right = (view_width - 28.0).max(margin_left + 10.0);
                    let cur_target = state.scroll_spring.target;
                    let cursor_view_x = cursor_local_x - cur_target;

                    if cursor_view_x < margin_left {
                        state.scroll_spring.set_target((cursor_local_x - margin_left).max(0.0));
                    } else if cursor_view_x > margin_right {
                        state.scroll_spring.set_target((cursor_local_x - margin_right).min(max_scroll));
                    }
                    state.scroll_spring.set_target(state.scroll_spring.target.clamp(0.0, max_scroll));
                }

                state.scroll_spring.update(dt);
                if !state.scroll_spring.is_settled() {
                    ui.ctx().request_repaint();
                }

                // Detect visual text insertion or deletion compared to previous frame's rendered display text (only during keyboard editing)
                if state.drag_selection.is_none() && state.drop_flight.is_none() {
                    if let Some(ref last_txt) = state.last_text {
                        let old_count = last_txt.chars().count();
                        let new_count = display_text.chars().count();
                        if new_count > old_count {
                            // Insertion: characters typed or pasted
                            let num_inserted = new_count - old_count;
                            let split_char_idx = cursor_char_idx.saturating_sub(num_inserted);
                            let inserted_substr: String = display_text.chars().skip(split_char_idx).take(num_inserted).collect();
                            let ins_galley = painter.layout_no_wrap(inserted_substr.clone(), font_id.clone(), text_color);
                            let ins_w = ins_galley.size().x.max(7.8);
                            state.swipe_anim = Some(TextSwipeAnimation::new_insert(split_char_idx, inserted_substr, ins_w));
                            ui.ctx().request_repaint();
                        } else if new_count < old_count {
                            // Deletion: characters deleted (single char, backspace, or visual selection cut)
                            let num_deleted = old_count - new_count;
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
                }
                state.last_text = Some(display_text.clone());

                // Compute target character rect for the fluid spring cursor based on the CURRENT text and mode
                let (target_caret_rect, target_rounding, fill_mult, stroke_width) = if let Some(ref drag_st) = state.drag_selection {
                    // During text drag-and-drop: The fluid 4-corner cursor glides to the drop insertion target!
                    let target_char = drag_st.target_char_idx.min(self.text.chars().count());
                    let target_cur = galley.from_ccursor(egui::text::CCursor::new(target_char));
                    let base_drop_x = text_pos.x + galley.pos_from_cursor(&target_cur).left();
                    let drop_x = if target_char > drag_st.split_char_idx {
                        base_drop_x + drag_st.close_spring.value()
                    } else {
                        base_drop_x
                    };

                    (
                        Rect::from_min_size(
                            pos2(drop_x.max(edit_rect.left()), edit_rect.center().y - 8.0),
                            vec2(2.0, 16.0),
                        ),
                        0.5,
                        1.0,
                        0.0,
                    )
                } else if self.text.is_empty() {
                    let caret_x = match self.align {
                        TextAlign::Left => edit_rect.left(),
                        TextAlign::Center => edit_rect.center().x,
                        TextAlign::Right => edit_rect.right() - 2.0,
                    };
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
                    let cursor_char_idx = byte_offset_to_char_index(self.text, vbuf.cursor);
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
                        VimMode::Visual(_) => {
                            if let Some(anchor_byte) = vbuf.anchor {
                                let anchor_char_idx = byte_offset_to_char_index(self.text, anchor_byte);
                                let min_char = anchor_char_idx.min(cursor_char_idx);
                                let max_char = anchor_char_idx.max(cursor_char_idx);
                                let min_cur = galley.from_ccursor(egui::text::CCursor::new(min_char));
                                let min_x = text_pos.x + galley.pos_from_cursor(&min_cur).left();
                                let max_next_cur = galley.from_ccursor(egui::text::CCursor::new((max_char + 1).min(self.text.chars().count())));
                                let max_x = text_pos.x + galley.pos_from_cursor(&max_next_cur).left();
                                let visual_w = (max_x - min_x).max(char_w);
                                (
                                    Rect::from_min_size(
                                        pos2(min_x, edit_rect.center().y - 8.0),
                                        vec2(visual_w, 16.0),
                                    ),
                                    1.5,
                                    0.55,
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
                    state.cursor_spring.fade_out();
                }

                // Advance fluid 4-corner cursor simulation
                state.cursor_spring.update(target_caret_rect, self.focused, dt, ui.ctx());

                if self.text.is_empty() {
                    if let Some(ph) = self.placeholder {
                        let (ph_pos, ph_align) = match self.align {
                            TextAlign::Left => (pos2(edit_rect.left(), edit_rect.center().y), Align2::LEFT_CENTER),
                            TextAlign::Center => (pos2(edit_rect.center().x, edit_rect.center().y), Align2::CENTER_CENTER),
                            TextAlign::Right => (pos2(edit_rect.right(), edit_rect.center().y), Align2::RIGHT_CENTER),
                        };
                        painter.text(
                            ph_pos,
                            ph_align,
                            ph,
                            font_id.clone(),
                            placeholder_color,
                        );
                    }
                } else {
                    // Visual selection highlight with smooth fade-out on pull and fade-in on drop landing
                    if let Some(ref pull) = state.pull_fade {
                        let pull_alpha = pull.fade_spring.value().clamp(0.0, 1.0);
                        if pull_alpha > 0.01 {
                            painter.rect_filled(pull.rect, Rounding::same(2.0), pull.color.linear_multiply(0.25 * pull_alpha));
                        }
                    }

                    if state.drag_selection.is_none() {
                        if let Some(range) = vbuf.selection_range() {
                            let min_char = byte_offset_to_char_index(self.text, range.start);
                            let max_char = byte_offset_to_char_index(self.text, range.end);
                            let cur1 = galley.from_ccursor(egui::text::CCursor::new(min_char));
                            let cur2 = galley.from_ccursor(egui::text::CCursor::new(max_char));
                            let r1 = galley.pos_from_cursor(&cur1);
                            let r2 = galley.pos_from_cursor(&cur2);
                            let sel_rect = Rect::from_min_max(
                                pos2(text_pos.x + r1.left().min(r2.left()), edit_rect.top() + 2.0),
                                pos2(text_pos.x + r1.left().max(r2.left()), edit_rect.bottom() - 2.0),
                            );
                            let sel_alpha = state.selection_fade_spring.value().clamp(0.0, 1.0);
                            if sel_alpha > 0.01 {
                                painter.rect_filled(sel_rect, Rounding::same(2.0), smoothed_mode_color.linear_multiply(0.25 * sel_alpha));
                            }
                        }
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
                    } else if let Some(ref drag_st) = state.drag_selection {
                        // While pulling / dragging text:
                        // Characters on the right (split_char_idx..) smoothly slide left from close_spring.value() to 0.0 to close the gap!
                        let start_char = drag_st.split_char_idx;

                        // 1. Prefix (0..start_char)
                        let prefix_str: String = display_text.chars().take(start_char).collect();
                        let prefix_w = if !prefix_str.is_empty() {
                            let prefix_galley = painter.layout_no_wrap(prefix_str, font_id.clone(), text_color);
                            let pw = prefix_galley.size().x;
                            painter.galley(text_pos, prefix_galley, text_color);
                            pw
                        } else {
                            0.0
                        };

                        // 2. Suffix (start_char..) shifted right by close_spring.value() (animating extracted_width -> 0.0)
                        let suffix_str: String = display_text.chars().skip(start_char).collect();
                        if !suffix_str.is_empty() {
                            let shift_offset = drag_st.close_spring.value();
                            let suffix_x = text_pos.x + prefix_w + shift_offset;
                            let suffix_galley = painter.layout_no_wrap(suffix_str, font_id.clone(), text_color);
                            painter.galley(pos2(suffix_x, text_pos.y), suffix_galley, text_color);
                        }
                    } else if let Some(ref flight) = state.drop_flight {
                        // While drop flight is in progress:
                        // Suffix smoothly parts open to make room for incoming text using gap_spring!
                        let start_char = flight.split_char_idx;
                        let num_chars = flight.text.chars().count();

                        // 1. Prefix (0..start_char)
                        let prefix_str: String = display_text.chars().take(start_char).collect();
                        let prefix_w = if !prefix_str.is_empty() {
                            let prefix_galley = painter.layout_no_wrap(prefix_str, font_id.clone(), text_color);
                            let pw = prefix_galley.size().x;
                            painter.galley(text_pos, prefix_galley, text_color);
                            pw
                        } else {
                            0.0
                        };

                        // 2. Suffix (start_char + num_chars..) parts open rightward by gap_spring.value()
                        let suffix_str: String = display_text.chars().skip(start_char + num_chars).collect();
                        if !suffix_str.is_empty() {
                            let gap_offset = flight.gap_spring.value();
                            let suffix_x = text_pos.x + prefix_w + gap_offset;
                            let suffix_galley = painter.layout_no_wrap(suffix_str, font_id.clone(), text_color);
                            painter.galley(pos2(suffix_x, text_pos.y), suffix_galley, text_color);
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
                if state.drop_flight.is_none() && state.drag_selection.is_none() && self.focused && !self.text.is_empty() && (mode == VimMode::Normal || mode.is_operator_pending() || mode.is_visual()) && fill_mult >= 0.8 {
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
                    let inv_galley = inv_painter.layout_no_wrap(display_text.clone(), font_id.clone(), inverted_text_color);
                    inv_painter.galley(text_pos, inv_galley, inverted_text_color);
                }

                // 2. Render Floating Drag Ghost attached to mouse pointer on TOP layer (above all sections/panels)
                if let Some(ref drag_st) = state.drag_selection {
                    let overlay_painter = ui.ctx().layer_painter(LayerId::new(
                        Order::Tooltip,
                        ui.id().with("drag_ghost_overlay"),
                    ));
                    let ghost_text_pos = drag_st.last_pos + drag_st.grab_offset;
                    let ghost_galley = overlay_painter.layout_no_wrap(
                        drag_st.text.clone(),
                        font_id.clone(),
                        if let Some(p) = self.palette { p.text } else { Color32::WHITE },
                    );
                    let pill_rect = Rect::from_min_size(
                        pos2(ghost_text_pos.x - 6.0, ghost_text_pos.y - 3.0),
                        vec2(ghost_galley.size().x + 12.0, ghost_galley.size().y + 6.0),
                    );
                    let ghost_bg = if let Some(p) = self.palette {
                        p.surface1.linear_multiply(0.95)
                    } else {
                        Color32::from_black_alpha(220)
                    };
                    overlay_painter.add(Shape::rect_filled(pill_rect, Rounding::same(4.0), ghost_bg));
                    overlay_painter.add(Shape::rect_stroke(pill_rect, Rounding::same(4.0), Stroke::new(1.5, smoothed_mode_color)));
                    overlay_painter.galley(ghost_text_pos, ghost_galley, if let Some(p) = self.palette { p.text } else { Color32::WHITE });
                }

                // 3. Render 2D Spring Flight for dropped text docking into place on TOP layer
                if let Some(ref flight) = state.drop_flight {
                    let overlay_painter = ui.ctx().layer_painter(LayerId::new(
                        Order::Tooltip,
                        ui.id().with("drop_flight_overlay"),
                    ));
                    let flight_x = flight.x_spring.value();
                    let flight_y = flight.y_spring.value();
                    let flight_scale = flight.scale_spring.value();
                    let flight_galley = overlay_painter.layout_no_wrap(
                        flight.text.clone(),
                        font_id.clone(),
                        if let Some(p) = self.palette { p.text } else { Color32::WHITE },
                    );
                    let pill_rect = Rect::from_min_size(
                        pos2(flight_x - 6.0, flight_y - 3.0),
                        vec2((flight_galley.size().x + 12.0) * flight_scale, (flight_galley.size().y + 6.0) * flight_scale),
                    );
                    let dist = (pos2(flight_x, flight_y) - flight.dest_pos).length();
                    let fade = (dist / 35.0).clamp(0.0, 1.0);
                    let flight_bg = if let Some(p) = self.palette {
                        p.surface1.linear_multiply(0.95 * fade)
                    } else {
                        Color32::from_black_alpha((220.0 * fade) as u8)
                    };
                    if fade > 0.05 {
                        overlay_painter.add(Shape::rect_filled(pill_rect, Rounding::same(4.0), flight_bg));
                        overlay_painter.add(Shape::rect_stroke(pill_rect, Rounding::same(4.0), Stroke::new(1.5 * fade, smoothed_mode_color.linear_multiply(fade))));
                    }
                    overlay_painter.galley(pos2(flight_x, flight_y), flight_galley, if let Some(p) = self.palette { p.text } else { Color32::WHITE });
                }
            }

            interact_resp
        } else {
            // Standard egui::TextEdit fallback for non-Vim text inputs
            let mut edit = TextEdit::singleline(self.text)
                .password(self.password)
                .text_color(text_color)
                .horizontal_align(match self.align {
                    TextAlign::Left => egui::Align::LEFT,
                    TextAlign::Center => egui::Align::Center,
                    TextAlign::Right => egui::Align::RIGHT,
                })
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
            let clear_color = if clear_resp.hovered() && crate::is_hover_active(ui.ctx()) { text_color } else { placeholder_color };
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
///
/// This is a re-export of [`egui_themes::lerp_color`] for backward compatibility.
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    egui_themes::lerp_color(a, b, t)
}

#[inline]
fn pointer_to_char_index(
    pointer_pos: egui::Pos2,
    text_pos: egui::Pos2,
    galley: &egui::Galley,
    display_text: &str,
) -> usize {
    let local_vec = egui::vec2(pointer_pos.x - text_pos.x, 4.0);
    let ccursor = galley.cursor_from_pos(local_vec).ccursor;
    ccursor.index.min(display_text.chars().count())
}

#[inline]
fn byte_offset_to_char_index(text: &str, byte_offset: usize) -> usize {
    text.char_indices().take_while(|(b, _)| *b < byte_offset).count()
}

#[inline]
fn safe_byte_slice(text: &str, range: std::ops::Range<usize>) -> &str {
    let start = text.char_indices().map(|(b, _)| b).find(|&b| b >= range.start).unwrap_or(text.len());
    let end = text.char_indices().map(|(b, _)| b).find(|&b| b >= range.end).unwrap_or(text.len());
    if start <= end && end <= text.len() {
        &text[start..end]
    } else {
        ""
    }
}

#[inline]
fn safe_replace_range(text: &mut String, range: std::ops::Range<usize>, replacement: &str) {
    let start = text.char_indices().map(|(b, _)| b).find(|&b| b >= range.start).unwrap_or(text.len());
    let end = text.char_indices().map(|(b, _)| b).find(|&b| b >= range.end).unwrap_or(text.len());
    if start <= end && end <= text.len() {
        text.replace_range(start..end, replacement);
    }
}

#[inline]
fn char_index_to_byte_offset(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

fn word_bounds_around_byte(text: &str, byte_offset: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }
    let clamped_offset = byte_offset.min(text.len());
    let char_indices: Vec<(usize, char)> = text.char_indices().collect();
    if char_indices.is_empty() {
        return (0, 0);
    }
    let target_idx = char_indices
        .iter()
        .position(|&(b, _)| b >= clamped_offset)
        .unwrap_or(char_indices.len().saturating_sub(1));
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
    let target_is_word = is_word_char(char_indices[target_idx].1);

    let mut start_idx = target_idx;
    while start_idx > 0
        && is_word_char(char_indices[start_idx - 1].1) == target_is_word
        && !char_indices[start_idx - 1].1.is_whitespace()
    {
        start_idx -= 1;
    }

    let mut end_idx = target_idx;
    while end_idx + 1 < char_indices.len()
        && is_word_char(char_indices[end_idx + 1].1) == target_is_word
        && !char_indices[end_idx + 1].1.is_whitespace()
    {
        end_idx += 1;
    }

    let start_byte = char_indices[start_idx].0;
    let end_byte = if end_idx + 1 < char_indices.len() {
        char_indices[end_idx + 1].0
    } else {
        text.len()
    };
    (start_byte, end_byte)
}
