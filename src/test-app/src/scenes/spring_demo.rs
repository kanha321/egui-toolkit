//! Interactive physics demonstration for spring-core and egui-spring,
//! structured responsively using egui-layout nested splits.

use egui::{
    Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};
use egui_layout::Split;
use egui_spring::SpringRect;
use spring_core::{Spring, SpringParams};

/// The 5 available spring animation presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HighlightPreset {
    Gentle,
    Snappy,
    #[default]
    Preferred,
    OpenRGB,
    Custom,
}

/// A shape component in the Bézier morphing showcase.
pub struct MorphComponent {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub rounding: Rounding,
}

/// State for the Spring demo scene owned by `TestAppState`.
pub struct SpringDemoState {
    pub gentle_spring: Spring,
    pub snappy_spring: Spring,
    pub preferred_spring: Spring,
    pub openrgb_spring: Spring,
    pub custom_spring: Spring,
    pub custom_frequency: f32,
    pub custom_damping: f32,
    pub speed_scale: f32,
    pub target_val: f32,
    pub history_gentle: Vec<f32>,
    pub history_snappy: Vec<f32>,
    pub history_preferred: Vec<f32>,
    pub history_openrgb: Vec<f32>,
    pub history_custom: Vec<f32>,
    // 2D SpringRect elastic smear selection highlight demo
    pub selection_highlight: SpringRect,
    pub selected_component: usize,
    pub highlight_preset: HighlightPreset,
}

impl Default for SpringDemoState {
    fn default() -> Self {
        let initial_target = 0.5;
        let default_freq = 20.0;
        let default_damp = 0.5;
        let default_speed = 1.75;

        Self {
            gentle_spring: Spring::new(initial_target, SpringParams::gentle()),
            snappy_spring: Spring::new(initial_target, SpringParams::snappy()),
            preferred_spring: Spring::new(initial_target, SpringParams::new(default_freq, default_damp)),
            openrgb_spring: Spring::new(initial_target, SpringParams::openrgb()),
            custom_spring: Spring::new(initial_target, SpringParams::new(default_freq, default_damp)),
            custom_frequency: default_freq,
            custom_damping: default_damp,
            speed_scale: default_speed,
            target_val: initial_target,
            history_gentle: Vec::new(),
            history_snappy: Vec::new(),
            history_preferred: Vec::new(),
            history_openrgb: Vec::new(),
            history_custom: Vec::new(),
            selection_highlight: SpringRect::new(Rect::ZERO)
                .with_fill(Color32::from_rgba_unmultiplied(0, 255, 136, 14))
                .with_stroke(Stroke::new(1.5, Color32::from_rgb(0, 255, 136)))
                .with_corner_rounding(Rounding { nw: 24.0, ne: 4.0, se: 24.0, sw: 4.0 })
                .with_padding(3.0),
            selected_component: 0,
            highlight_preset: HighlightPreset::Preferred,
        }
    }
}

impl SpringDemoState {
    pub fn update(&mut self, raw_dt: f32) {
        let dt = raw_dt * self.speed_scale;
        self.custom_spring.params = SpringParams::new(self.custom_frequency, self.custom_damping);

        // Sync active highlight preset parameters and matching colors
        let (stiffness, damping, accent_color) = match self.highlight_preset {
            HighlightPreset::Gentle => (18.0, 0.90, Color32::from_rgb(203, 166, 247)),
            HighlightPreset::Snappy => (32.0, 0.85, Color32::from_rgb(166, 227, 161)),
            HighlightPreset::Preferred => (20.0, 0.50, Color32::from_rgb(0, 255, 136)),
            HighlightPreset::OpenRGB => (22.0, 0.65, Color32::from_rgb(137, 180, 250)),
            HighlightPreset::Custom => (
                self.custom_frequency,
                self.custom_damping,
                Color32::from_rgb(137, 220, 235),
            ),
        };

        self.selection_highlight.corners.base_stiffness = stiffness;
        self.selection_highlight.corners.base_damping = damping;
        self.selection_highlight.stroke.color = accent_color;
        self.selection_highlight.fill_color = Color32::from_rgba_unmultiplied(
            accent_color.r(),
            accent_color.g(),
            accent_color.b(),
            14,
        );

        self.gentle_spring.update(dt);
        self.snappy_spring.update(dt);
        self.preferred_spring.update(dt);
        self.openrgb_spring.update(dt);
        self.custom_spring.update(dt);

        self.selection_highlight.update(dt);

        // Record history trace for oscilloscope visualization (keep last 140 points)
        self.history_gentle.push(self.gentle_spring.value());
        self.history_snappy.push(self.snappy_spring.value());
        self.history_preferred.push(self.preferred_spring.value());
        self.history_openrgb.push(self.openrgb_spring.value());
        self.history_custom.push(self.custom_spring.value());

        if self.history_gentle.len() > 140 { self.history_gentle.remove(0); }
        if self.history_snappy.len() > 140 { self.history_snappy.remove(0); }
        if self.history_preferred.len() > 140 { self.history_preferred.remove(0); }
        if self.history_openrgb.len() > 140 { self.history_openrgb.remove(0); }
        if self.history_custom.len() > 140 { self.history_custom.remove(0); }
    }

    pub fn is_animating(&self) -> bool {
        !self.gentle_spring.is_settled()
            || !self.snappy_spring.is_settled()
            || !self.preferred_spring.is_settled()
            || !self.openrgb_spring.is_settled()
            || !self.custom_spring.is_settled()
            || !self.selection_highlight.is_settled()
    }
}

pub fn show(ui: &mut Ui, state: &mut SpringDemoState) {
    let raw_dt = ui.input(|i| i.stable_dt).min(0.05);
    state.update(raw_dt);

    // Continuous motion repaint rule (CODING_RULES §4):
    if state.is_animating() {
        ui.ctx().request_repaint();
    }

    let SpringDemoState {
        gentle_spring,
        snappy_spring,
        preferred_spring,
        openrgb_spring,
        custom_spring,
        custom_frequency,
        custom_damping,
        speed_scale,
        target_val,
        history_gentle,
        history_snappy,
        history_preferred,
        history_openrgb,
        history_custom,
        selection_highlight,
        selected_component,
        highlight_preset,
    } = state;

    let target_copy = *target_val;
    let custom_freq_copy = *custom_frequency;
    let custom_damp_copy = *custom_damping;
    let current_rounding_copy = selection_highlight.current_rounding;

    // Outer Vertical Split: Top 2D Morphing Showcase, Bottom 1D Physics & Controls
    Split::vertical()
        .spacing(10.0)
        .section_fixed(205.0, |ui| {
            // Section 1: 2D Multi-Shape Bézier Morphing Showcase
            ui.group(|ui| {
                // Clean header bar
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(
                        Color32::from_rgb(0, 255, 136),
                        egui::RichText::new("🎯 2D Bézier Morphing Highlight").strong().size(13.5),
                    );
                    ui.label(
                        egui::RichText::new("• 4-Corner Elastic Highlight")
                            .size(11.0)
                            .color(Color32::from_rgb(166, 173, 200)),
                    );

                    ui.separator();
                    ui.label("Preset:");
                    ui.selectable_value(highlight_preset, HighlightPreset::Preferred, "Default (20/0.5)");
                    ui.selectable_value(highlight_preset, HighlightPreset::Snappy, "Snappy");
                    ui.selectable_value(highlight_preset, HighlightPreset::Gentle, "Gentle");
                    ui.selectable_value(highlight_preset, HighlightPreset::OpenRGB, "OpenRGB");
                    ui.selectable_value(highlight_preset, HighlightPreset::Custom, "Custom");
                });

                ui.add_space(8.0);

                let components = [
                    MorphComponent {
                        title: "Diagonal Leaf",
                        subtitle: "NW:24 NE:4 SE:24 SW:4",
                        rounding: Rounding { nw: 24.0, ne: 4.0, se: 24.0, sw: 4.0 },
                    },
                    MorphComponent {
                        title: "Callout Bubble",
                        subtitle: "NW:18 NE:18 SE:2 SW:18",
                        rounding: Rounding { nw: 18.0, ne: 18.0, se: 2.0, sw: 18.0 },
                    },
                    MorphComponent {
                        title: "Tab Arch",
                        subtitle: "NW:20 NE:20 SE:0 SW:0",
                        rounding: Rounding { nw: 20.0, ne: 20.0, se: 0.0, sw: 0.0 },
                    },
                    MorphComponent {
                        title: "Inverted Cut",
                        subtitle: "NW:0 NE:22 SE:0 SW:22",
                        rounding: Rounding { nw: 0.0, ne: 22.0, se: 0.0, sw: 22.0 },
                    },
                    MorphComponent {
                        title: "Pill Capsule",
                        subtitle: "All Corners r: 22px",
                        rounding: Rounding::same(22.0),
                    },
                    MorphComponent {
                        title: "Horizon Bar",
                        subtitle: "All Corners r: 12px",
                        rounding: Rounding::same(12.0),
                    },
                    MorphComponent {
                        title: "Smooth Squircle",
                        subtitle: "All Corners r: 16px",
                        rounding: Rounding::same(16.0),
                    },
                    MorphComponent {
                        title: "Sharp Square",
                        subtitle: "All Corners r: 0px",
                        rounding: Rounding::ZERO,
                    },
                ];

                let mut target_rect = None;
                let mut target_rounding = Rounding::ZERO;

                // Enforce minimum card width constraint so cards never get crushed
                let col_spacing = 8.0;
                let min_card_w = 160.0;
                let min_grid_w = min_card_w * 4.0 + col_spacing * 3.0; // 664px min width
                let total_w = ui.available_width().max(min_grid_w);
                let card_w = ((total_w - col_spacing * 3.0) / 4.0).floor();
                let card_h = 52.0;

                egui::ScrollArea::horizontal()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Row 1: First 4 Shapes
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = col_spacing;
                            for (idx, comp) in components.iter().take(4).enumerate() {
                                let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, card_h), Sense::click());
                                if resp.clicked() {
                                    *selected_component = idx;
                                }
                                if idx == *selected_component {
                                    target_rect = Some(rect);
                                    target_rounding = comp.rounding;
                                }

                                ui.painter().rect(
                                    rect,
                                    comp.rounding,
                                    Color32::from_rgb(30, 30, 46),
                                    Stroke::new(1.0, Color32::from_rgb(49, 50, 68)),
                                );

                                ui.painter().text(
                                    rect.min + Vec2::new(12.0, 8.0),
                                    egui::Align2::LEFT_TOP,
                                    comp.title,
                                    egui::FontId::proportional(12.0),
                                    Color32::from_rgb(205, 214, 244),
                                );
                                ui.painter().text(
                                    rect.min + Vec2::new(12.0, 28.0),
                                    egui::Align2::LEFT_TOP,
                                    comp.subtitle,
                                    egui::FontId::monospace(9.0),
                                    Color32::from_rgb(147, 153, 178),
                                );
                            }
                        });

                        ui.add_space(8.0);

                        // Row 2: Next 4 Shapes
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = col_spacing;
                            for (idx, comp) in components.iter().skip(4).enumerate() {
                                let real_idx = idx + 4;
                                let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, card_h), Sense::click());
                                if resp.clicked() {
                                    *selected_component = real_idx;
                                }
                                if real_idx == *selected_component {
                                    target_rect = Some(rect);
                                    target_rounding = comp.rounding;
                                }

                                ui.painter().rect(
                                    rect,
                                    comp.rounding,
                                    Color32::from_rgb(30, 30, 46),
                                    Stroke::new(1.0, Color32::from_rgb(49, 50, 68)),
                                );

                                ui.painter().text(
                                    rect.min + Vec2::new(12.0, 8.0),
                                    egui::Align2::LEFT_TOP,
                                    comp.title,
                                    egui::FontId::proportional(12.0),
                                    Color32::from_rgb(205, 214, 244),
                                );
                                ui.painter().text(
                                    rect.min + Vec2::new(12.0, 28.0),
                                    egui::Align2::LEFT_TOP,
                                    comp.subtitle,
                                    egui::FontId::monospace(9.0),
                                    Color32::from_rgb(147, 153, 178),
                                );
                            }
                        });

                        if let Some(target) = target_rect {
                            selection_highlight.set_target_with_corner_rounding(target, target_rounding);
                        }

                        // Paint the SpringRect highlight morphing across shapes
                        selection_highlight.paint(ui.painter());
                    });
            });
        })
        .section_remainder(|ui| {
            // Section 2: Split horizontally between 1D Physics Tracks (50%) and Controls/Trace (50%)
            Split::horizontal()
                .spacing(10.0)
                .section_min(0.50, 420.0, |ui| {
                    ui.group(|ui| {
                        let mut new_target: Option<f32> = None;
                        let mut apply_impulse = 0.0f32;

                        ui.horizontal_wrapped(|ui| {
                            ui.colored_label(
                                Color32::from_rgb(137, 180, 250),
                                egui::RichText::new("⚡ 1D Analytical ODE Tracks").strong().size(13.0),
                            );

                            ui.separator();
                            ui.label("Target:");
                            if ui.button("0%").clicked() { new_target = Some(0.0); }
                            if ui.button("25%").clicked() { new_target = Some(0.25); }
                            if ui.button("50%").clicked() { new_target = Some(0.5); }
                            if ui.button("75%").clicked() { new_target = Some(0.75); }
                            if ui.button("100%").clicked() { new_target = Some(1.0); }
                            if ui.button("+3.0 Impulse").clicked() { apply_impulse += 3.0; }
                        });

                        if let Some(t) = new_target {
                            let target_clamped = t.clamp(0.0, 1.0);
                            *target_val = target_clamped;
                            gentle_spring.set_target(target_clamped);
                            snappy_spring.set_target(target_clamped);
                            preferred_spring.set_target(target_clamped);
                            openrgb_spring.set_target(target_clamped);
                            custom_spring.set_target(target_clamped);
                        }

                        if apply_impulse != 0.0 {
                            gentle_spring.velocity += apply_impulse;
                            snappy_spring.velocity += apply_impulse;
                            preferred_spring.velocity += apply_impulse;
                            openrgb_spring.velocity += apply_impulse;
                            custom_spring.velocity += apply_impulse;
                        }

                        ui.add_space(4.0);

                        let track_height = 42.0;
                        render_spring_track(
                            ui,
                            "Default (1.75x • ω0 = 20, ζ = 0.50)",
                            preferred_spring.value(),
                            preferred_spring.velocity(),
                            target_copy,
                            Color32::from_rgb(0, 255, 136),
                            track_height,
                            |t| {
                                *target_val = t;
                                gentle_spring.set_target(t);
                                snappy_spring.set_target(t);
                                preferred_spring.set_target(t);
                                openrgb_spring.set_target(t);
                                custom_spring.set_target(t);
                            },
                        );

                        ui.add_space(3.0);
                        render_spring_track(
                            ui,
                            "Snappy (ω0 = 32, ζ = 0.85)",
                            snappy_spring.value(),
                            snappy_spring.velocity(),
                            target_copy,
                            Color32::from_rgb(166, 227, 161),
                            track_height,
                            |t| {
                                *target_val = t;
                                gentle_spring.set_target(t);
                                snappy_spring.set_target(t);
                                preferred_spring.set_target(t);
                                openrgb_spring.set_target(t);
                                custom_spring.set_target(t);
                            },
                        );

                        ui.add_space(3.0);
                        render_spring_track(
                            ui,
                            "Gentle (ω0 = 18, ζ = 0.90)",
                            gentle_spring.value(),
                            gentle_spring.velocity(),
                            target_copy,
                            Color32::from_rgb(203, 166, 247),
                            track_height,
                            |t| {
                                *target_val = t;
                                gentle_spring.set_target(t);
                                snappy_spring.set_target(t);
                                preferred_spring.set_target(t);
                                openrgb_spring.set_target(t);
                                custom_spring.set_target(t);
                            },
                        );

                        ui.add_space(3.0);
                        render_spring_track(
                            ui,
                            "OpenRGB (ω0 = 22, ζ = 0.65)",
                            openrgb_spring.value(),
                            openrgb_spring.velocity(),
                            target_copy,
                            Color32::from_rgb(137, 180, 250),
                            track_height,
                            |t| {
                                *target_val = t;
                                gentle_spring.set_target(t);
                                snappy_spring.set_target(t);
                                preferred_spring.set_target(t);
                                openrgb_spring.set_target(t);
                                custom_spring.set_target(t);
                            },
                        );

                        ui.add_space(3.0);
                        let regime_label = if custom_damp_copy < 0.9999 {
                            "Underdamped"
                        } else if custom_damp_copy > 1.0001 {
                            "Overdamped"
                        } else {
                            "Critical"
                        };

                        render_spring_track(
                            ui,
                            &format!("Custom ({:.0}/{:.2} — {})", custom_freq_copy, custom_damp_copy, regime_label),
                            custom_spring.value(),
                            custom_spring.velocity(),
                            target_copy,
                            Color32::from_rgb(137, 220, 235),
                            track_height,
                            |t| {
                                *target_val = t;
                                gentle_spring.set_target(t);
                                snappy_spring.set_target(t);
                                preferred_spring.set_target(t);
                                openrgb_spring.set_target(t);
                                custom_spring.set_target(t);
                            },
                        );
                    });
                })
                .section_remainder(|ui| {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("📈 Real-Time Trajectory Trace").strong().size(13.0));
                        ui.add_space(4.0);
                        render_oscilloscope_views(
                            ui,
                            target_copy,
                            history_gentle,
                            history_snappy,
                            history_preferred,
                            history_openrgb,
                            history_custom,
                        );

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("⚙️ Live Physics Tuning").strong().size(13.0));
                        ui.add_space(4.0);
                        ui.add(egui::Slider::new(speed_scale, 0.5..=3.0).text("Speed Scale").suffix("x"));
                        ui.add(egui::Slider::new(custom_frequency, 5.0..=60.0).text("Frequency ω0 (rad/s)"));
                        ui.add(egui::Slider::new(custom_damping, 0.05..=2.5).text("Damping Ratio ζ"));

                        let r = current_rounding_copy;
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "Active 4-Corner Radii: [NW:{:.0} NE:{:.0} SE:{:.0} SW:{:.0}]",
                                r.nw, r.ne, r.se, r.sw
                            ))
                            .size(11.0)
                            .color(Color32::from_rgb(166, 173, 200)),
                        );
                    });
                })
                .show(ui);
        })
        .show(ui);
}

fn render_spring_track(
    ui: &mut Ui,
    title: &str,
    current_val: f32,
    velocity: f32,
    target_val: f32,
    accent: Color32,
    height: f32,
    mut on_click: impl FnMut(f32),
) {
    let available_w = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(available_w, height), Sense::click_and_drag());

    if response.clicked() || response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            let pad = 20.0;
            let track_w = rect.width() - pad * 2.0;
            let frac = ((pos.x - (rect.min.x + pad)) / track_w).clamp(0.0, 1.0);
            on_click(frac);
        }
    }

    let painter = ui.painter();

    // Track card background with clean rounded borders
    painter.rect(
        rect,
        Rounding::same(6.0),
        Color32::from_rgb(30, 32, 48),
        Stroke::new(1.0, Color32::from_rgb(69, 71, 90)),
    );

    // Top Label Row: Short Title on Left, Telemetry on Right
    painter.text(
        Pos2::new(rect.min.x + 10.0, rect.min.y + 5.0),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(11.0),
        Color32::from_rgb(205, 214, 244),
    );

    let telemetry = format!("p:{:.2} v:{:+.1}", current_val, velocity);
    painter.text(
        Pos2::new(rect.max.x - 10.0, rect.min.y + 5.0),
        egui::Align2::RIGHT_TOP,
        telemetry,
        egui::FontId::monospace(9.5),
        Color32::from_rgb(147, 153, 178),
    );

    // Bottom Track Rail Row (Dedicated y-position at rect.min.y + 28.0)
    let pad = 20.0;
    let track_min_x = rect.min.x + pad;
    let track_max_x = rect.max.x - pad;
    let track_w = (track_max_x - track_min_x).max(10.0);
    let track_y = rect.min.y + 28.0;

    // Track rail
    painter.line_segment(
        [Pos2::new(track_min_x, track_y), Pos2::new(track_max_x, track_y)],
        Stroke::new(2.5, Color32::from_rgb(49, 50, 68)),
    );

    // Target marker (vertical pin)
    let target_x = track_min_x + target_val * track_w;
    painter.line_segment(
        [Pos2::new(target_x, track_y - 6.0), Pos2::new(target_x, track_y + 6.0)],
        Stroke::new(2.0, Color32::from_rgb(186, 194, 222)),
    );

    // Spring coil lines
    let current_x = track_min_x + current_val.clamp(-0.1, 1.1) * track_w;
    let coil_segments = 14;
    let mut prev_pt = Pos2::new(track_min_x, track_y);
    for i in 1..=coil_segments {
        let frac = i as f32 / coil_segments as f32;
        let x = track_min_x + (current_x - track_min_x) * frac;
        let y_offset = if i % 2 == 1 { -3.0 } else { 3.0 };
        let pt = Pos2::new(x, track_y + y_offset);
        painter.line_segment([prev_pt, pt], Stroke::new(1.0, Color32::from_rgb(108, 112, 134)));
        prev_pt = pt;
    }
    painter.line_segment([prev_pt, Pos2::new(current_x, track_y)], Stroke::new(1.0, Color32::from_rgb(108, 112, 134)));

    // Animated ball head
    painter.circle(
        Pos2::new(current_x, track_y),
        6.0,
        accent,
        Stroke::new(1.5, Color32::WHITE),
    );
}

fn render_oscilloscope_views(
    ui: &mut Ui,
    target_val: f32,
    history_gentle: &[f32],
    history_snappy: &[f32],
    history_preferred: &[f32],
    history_openrgb: &[f32],
    history_custom: &[f32],
) {
    let height = 80.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let painter = ui.painter();

    painter.rect(
        rect,
        Rounding::same(4.0),
        Color32::from_rgb(24, 24, 37),
        Stroke::new(1.0, Color32::from_rgb(49, 50, 68)),
    );

    // Target baseline
    let target_y = rect.max.y - (target_val * (rect.height() - 16.0) + 8.0);
    painter.line_segment(
        [Pos2::new(rect.min.x, target_y), Pos2::new(rect.max.x, target_y)],
        Stroke::new(1.0, Color32::from_rgb(69, 71, 90)),
    );

    draw_trace(&painter, rect, history_gentle, Color32::from_rgb(203, 166, 247));
    draw_trace(&painter, rect, history_snappy, Color32::from_rgb(166, 227, 161));
    draw_trace(&painter, rect, history_preferred, Color32::from_rgb(0, 255, 136));
    draw_trace(&painter, rect, history_openrgb, Color32::from_rgb(137, 180, 250));
    draw_trace(&painter, rect, history_custom, Color32::from_rgb(137, 220, 235));
}

fn draw_trace(painter: &egui::Painter, rect: Rect, history: &[f32], color: Color32) {
    if history.len() < 2 {
        return;
    }
    let n = history.len();
    let step_x = rect.width() / (n as f32 - 1.0);

    for i in 0..n - 1 {
        let x1 = rect.min.x + (i as f32) * step_x;
        let y1 = rect.max.y - (history[i].clamp(-0.5, 1.5) * (rect.height() - 16.0) + 8.0);

        let x2 = rect.min.x + ((i + 1) as f32) * step_x;
        let y2 = rect.max.y - (history[i + 1].clamp(-0.5, 1.5) * (rect.height() - 16.0) + 8.0);

        painter.line_segment(
            [Pos2::new(x1, y1), Pos2::new(x2, y2)],
            Stroke::new(1.5, color),
        );
    }
}
