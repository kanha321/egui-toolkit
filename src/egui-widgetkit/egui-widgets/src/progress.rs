//! Spring-animated progress bar and meter controls.
//!
//! Provides [`ProgressBar`] with physical catchup animation via `spring-core`,
//! multiple status variants, label slots, and theme token styling.
//!
//! # State Ownership
//!
//! Animation smoothing can be owned via [`ProgressState`] or tracked in ID storage (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Color32, Id, Rect, Response, Rounding, Sense, Shape, Stroke, TextStyle, Ui,
    WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Color variants for [`ProgressBar`].
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ProgressVariant {
    /// Uses palette accent color.
    #[default]
    Accent,
    /// Uses palette positive/success color.
    Success,
    /// Uses palette warning color.
    Warning,
    /// Uses palette danger color.
    Danger,
    /// Uses palette info color.
    Info,
    /// Custom color fill.
    Custom(Color32),
}

/// Persistent animation state for smooth progress catchup.
#[derive(Clone, Debug)]
pub struct ProgressState {
    /// Spring driving animated progress value ($0.0 \to 1.0$).
    pub value_spring: Spring,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            value_spring: Spring::new(0.0, SpringParams::new(22.0, 0.50)),
        }
    }
}

impl ProgressState {
    /// Creates a new progress state initialized to the given value.
    pub fn new(initial_value: f32) -> Self {
        Self {
            value_spring: Spring::new(initial_value, SpringParams::new(22.0, 0.50)),
        }
    }

    /// Updates internal spring towards target progress and requests repaint if moving.
    pub fn update(&mut self, dt: f32, target: f32, ctx: &egui::Context) {
        self.value_spring.set_target(target.clamp(0.0, 1.0));
        self.value_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if progress has settled at target.
    pub fn is_settled(&self) -> bool {
        self.value_spring.is_settled()
    }
}

/// A theme-aware, spring-animated progress bar widget.
///
/// # Example
/// ```no_run
/// use egui_widgets::ProgressBar;
///
/// # egui::__run_test_ui(|ui| {
/// ProgressBar::new(0.72)
///     .label("Download Progress")
///     .show_percentage(true)
///     .show(ui);
/// # });
/// ```
pub struct ProgressBar<'a> {
    progress: f32,
    variant: ProgressVariant,
    height: f32,
    rounding: Option<Rounding>,
    track_fill: Option<Color32>,
    track_stroke: Option<Stroke>,
    fill: Option<Color32>,
    show_percentage: bool,
    label: Option<WidgetText>,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut ProgressState>,
}

impl<'a> ProgressBar<'a> {
    /// Creates a new progress bar with a target fraction between `0.0` and `1.0`.
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            variant: ProgressVariant::Accent,
            height: 8.0,
            rounding: None,
            track_fill: None,
            track_stroke: None,
            fill: None,
            show_percentage: false,
            label: None,
            spring_params: SpringParams::new(22.0, 0.50),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
        }
    }

    /// Sets the status color variant.
    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the height of the progress track (default `8.0pt`).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Explicitly overrides track rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Explicitly overrides the background track fill color.
    pub fn track_fill(mut self, fill: Color32) -> Self {
        self.track_fill = Some(fill);
        self
    }

    /// Explicitly overrides the background track border stroke.
    pub fn track_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.track_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the filled bar color.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Whether to display the numeric percentage text (e.g. `72%`).
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Sets an optional header/status label above the progress bar.
    pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets custom spring physics parameters for progress smoothing.
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

    /// Binds an external [`ProgressState`] struct.
    pub fn with_state(mut self, state: &'a mut ProgressState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the progress bar into the UI.
    pub fn show(self, ui: &mut Ui) -> Response {
        let label_galley = self.label.as_ref().map(|l| {
            l.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Body,
            )
        });

        let percent_galley = if self.show_percentage {
            let pct_text = format!("{:.0}%", self.progress * 100.0);
            Some(
                WidgetText::from(pct_text).into_galley(
                    ui,
                    Some(false),
                    f32::INFINITY,
                    TextStyle::Small,
                ),
            )
        } else {
            None
        };

        let mut total_height = self.height;
        if label_galley.is_some() || percent_galley.is_some() {
            let text_h = label_galley
                .as_ref()
                .map(|g| g.size().y)
                .unwrap_or(14.0)
                .max(percent_galley.as_ref().map(|g| g.size().y).unwrap_or(14.0));
            total_height += text_h + 4.0;
        }

        let desired_size = vec2(ui.available_width().max(100.0), total_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        // Resolve colors
        let (track_fill, track_stroke, bar_fill, text_color) = if let Some(p) = self.palette {
            let bar_color = match self.variant {
                ProgressVariant::Accent => p.accent,
                ProgressVariant::Success => p.success,
                ProgressVariant::Warning => p.warning,
                ProgressVariant::Danger => p.danger,
                ProgressVariant::Info => p.info,
                ProgressVariant::Custom(c) => c,
            };
            (
                self.track_fill.unwrap_or(p.surface0),
                self.track_stroke.unwrap_or(Stroke::new(1.0, p.surface1)),
                self.fill.unwrap_or(bar_color),
                p.text,
            )
        } else {
            let v = ui.visuals();
            let bar_color = match self.variant {
                ProgressVariant::Accent => v.selection.bg_fill,
                ProgressVariant::Success => Color32::from_rgb(40, 167, 69),
                ProgressVariant::Warning => v.warn_fg_color,
                ProgressVariant::Danger => v.error_fg_color,
                ProgressVariant::Info => v.hyperlink_color,
                ProgressVariant::Custom(c) => c,
            };
            (
                self.track_fill.unwrap_or(v.widgets.inactive.bg_fill),
                self.track_stroke.unwrap_or(v.widgets.inactive.bg_stroke),
                self.fill.unwrap_or(bar_color),
                v.widgets.inactive.fg_stroke.color,
            )
        };

        // Motion physics handling
        let dt = ui.input(|i| i.stable_dt).min(0.05);

        let animated_progress = if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, self.progress, ui.ctx());
                state.value_spring.value()
            } else {
                let id = self.id_source.unwrap_or_else(|| response.id);
                let mut state: ProgressState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_else(|| ProgressState::new(self.progress))
                });

                state.update(dt, self.progress, ui.ctx());
                let val = state.value_spring.value();
                ui.data_mut(|d| d.insert_temp(id, state));
                val
            }
        } else {
            self.progress
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let mut top_y = rect.top();

            // Render labels row if present
            if label_galley.is_some() || percent_galley.is_some() {
                if let Some(ref lg) = label_galley {
                    painter.galley(pos2(rect.left(), top_y), lg.clone(), text_color);
                }
                if let Some(ref pg) = percent_galley {
                    painter.galley(
                        pos2(rect.right() - pg.size().x, top_y),
                        pg.clone(),
                        text_color.linear_multiply(0.7),
                    );
                }
                top_y += 18.0;
            }

            let track_rect = Rect::from_min_size(pos2(rect.left(), top_y), vec2(rect.width(), self.height));
            let rounding = self.rounding.unwrap_or(Rounding::same(self.height * 0.5));

            // Paint track
            painter.add(Shape::rect_filled(track_rect, rounding, track_fill));
            if track_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(track_rect, rounding, track_stroke));
            }

            // Paint filled progress bar
            let fill_width = (track_rect.width() * animated_progress.clamp(0.0, 1.0)).max(0.0);
            if fill_width > 0.5 {
                let fill_rect = Rect::from_min_size(track_rect.min, vec2(fill_width, self.height));
                painter.add(Shape::rect_filled(fill_rect, rounding, bar_fill));
            }
        }

        response
    }
}
