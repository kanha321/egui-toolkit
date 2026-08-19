//! Interactive physics demonstration for spring-core and egui-spring.

use egui::{
    Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};
use egui_spring::SpringRect;
use spring_core::{Spring, SpringParams};

/// The 5 available spring animation presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HighlightPreset {
    Gentle,
    Snappy,
    Bouncy,
    #[default]
    OpenRGB,
    Custom,
}

/// A shape component in the Bézier morphing showcase.
pub struct MorphComponent {
    pub label: &'static str,
    pub description: &'static str,
    pub size: Vec2,
    pub rounding: f32,
    pub is_circle: bool,
}

/// State for the Spring demo scene owned by `TestAppState`.
pub struct SpringDemoState {
    pub gentle_spring: Spring,
    pub snappy_spring: Spring,
    pub bouncy_spring: Spring,
    pub openrgb_spring: Spring,
    pub custom_spring: Spring,
    pub custom_frequency: f32,
    pub custom_damping: f32,
    pub speed_scale: f32,
    pub target_val: f32,
    pub history_gentle: Vec<f32>,
    pub history_snappy: Vec<f32>,
    pub history_bouncy: Vec<f32>,
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
        Self {
            gentle_spring: Spring::new(initial_target, SpringParams::gentle()),
            snappy_spring: Spring::new(initial_target, SpringParams::snappy()),
            bouncy_spring: Spring::new(initial_target, SpringParams::bouncy()),
            openrgb_spring: Spring::new(initial_target, SpringParams::openrgb()),
            custom_spring: Spring::new(initial_target, SpringParams::new(22.0, 0.65)),
            custom_frequency: 22.0,
            custom_damping: 0.65,
            speed_scale: 1.0,
            target_val: initial_target,
            history_gentle: Vec::new(),
            history_snappy: Vec::new(),
            history_bouncy: Vec::new(),
            history_openrgb: Vec::new(),
            history_custom: Vec::new(),
            selection_highlight: SpringRect::new(Rect::ZERO)
                .with_fill(Color32::from_rgba_unmultiplied(0, 255, 136, 14))
                .with_stroke(Stroke::new(1.5, Color32::from_rgb(0, 255, 136)))
                .with_rounding(22.0)
                .with_padding(3.0),
            selected_component: 0,
            highlight_preset: HighlightPreset::OpenRGB,
        }
    }
}

impl SpringDemoState {
    pub fn set_target(&mut self, target: f32) {
        self.target_val = target.clamp(0.0, 1.0);
        self.gentle_spring.set_target(self.target_val);
        self.snappy_spring.set_target(self.target_val);
        self.bouncy_spring.set_target(self.target_val);
        self.openrgb_spring.set_target(self.target_val);
        self.custom_spring.set_target(self.target_val);
    }

    pub fn apply_impulse(&mut self, velocity: f32) {
        self.gentle_spring.velocity += velocity;
        self.snappy_spring.velocity += velocity;
        self.bouncy_spring.velocity += velocity;
        self.openrgb_spring.velocity += velocity;
        self.custom_spring.velocity += velocity;
    }

    pub fn update(&mut self, raw_dt: f32) {
        let dt = raw_dt * self.speed_scale;
        self.custom_spring.params = SpringParams::new(self.custom_frequency, self.custom_damping);

        // Sync active highlight preset parameters and matching colors
        let (stiffness, damping, accent_color) = match self.highlight_preset {
            HighlightPreset::Gentle => (18.0, 0.90, Color32::from_rgb(203, 166, 247)),
            HighlightPreset::Snappy => (32.0, 0.85, Color32::from_rgb(166, 227, 161)),
            HighlightPreset::Bouncy => (24.0, 0.50, Color32::from_rgb(250, 179, 135)),
            HighlightPreset::OpenRGB => (22.0, 0.65, Color32::from_rgb(0, 255, 136)),
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
        self.bouncy_spring.update(dt);
        self.openrgb_spring.update(dt);
        self.custom_spring.update(dt);

        self.selection_highlight.update(dt);

        // Record history trace for oscilloscope visualization (keep last 140 points)
        self.history_gentle.push(self.gentle_spring.value());
        self.history_snappy.push(self.snappy_spring.value());
        self.history_bouncy.push(self.bouncy_spring.value());
        self.history_openrgb.push(self.openrgb_spring.value());
        self.history_custom.push(self.custom_spring.value());

        if self.history_gentle.len() > 140 { self.history_gentle.remove(0); }
        if self.history_snappy.len() > 140 { self.history_snappy.remove(0); }
        if self.history_bouncy.len() > 140 { self.history_bouncy.remove(0); }
        if self.history_openrgb.len() > 140 { self.history_openrgb.remove(0); }
        if self.history_custom.len() > 140 { self.history_custom.remove(0); }
    }

    pub fn is_animating(&self) -> bool {
        !self.gentle_spring.is_settled()
            || !self.snappy_spring.is_settled()
            || !self.bouncy_spring.is_settled()
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

    ui.horizontal(|ui| {
        ui.colored_label(
            Color32::from_rgb(137, 180, 250),
            egui::RichText::new("⚡ spring-core & egui-spring Physics Showcase").strong().size(16.0),
        );
        ui.label(
            egui::RichText::new("• Analytical ODE 1D Solvers + Multi-Shape Bézier Morphing")
                .size(12.0)
                .color(Color32::from_rgb(166, 173, 200)),
        );
    });
    ui.add_space(2.0);

    // Control bar
    ui.horizontal(|ui| {
        ui.label("Quick Targets:");
        if ui.button("0% (Left)").clicked() { state.set_target(0.0); }
        if ui.button("25%").clicked() { state.set_target(0.25); }
        if ui.button("50% (Center)").clicked() { state.set_target(0.5); }
        if ui.button("75%").clicked() { state.set_target(0.75); }
        if ui.button("100% (Right)").clicked() { state.set_target(1.0); }

        ui.separator();
        if ui.button("💥 Physical Impulse (+3.0 v0)").clicked() {
            state.apply_impulse(3.0);
        }
        if ui.button("💥 Reverse Impulse (-3.0 v0)").clicked() {
            state.apply_impulse(-3.0);
        }
    });

    ui.add_space(2.0);

    // 5 Tracks Comparison
    let track_height = 30.0;
    render_spring_track(
        ui,
        "🟣 Gentle (ω0 = 18, ζ = 0.90)",
        state.gentle_spring.value(),
        state.gentle_spring.velocity(),
        state.target_val,
        Color32::from_rgb(203, 166, 247),
        track_height,
        |new_target| state.set_target(new_target),
    );

    ui.add_space(2.0);
    render_spring_track(
        ui,
        "🟢 Snappy (ω0 = 32, ζ = 0.85)",
        state.snappy_spring.value(),
        state.snappy_spring.velocity(),
        state.target_val,
        Color32::from_rgb(166, 227, 161),
        track_height,
        |new_target| state.set_target(new_target),
    );

    ui.add_space(2.0);
    render_spring_track(
        ui,
        "🟠 Bouncy (ω0 = 24, ζ = 0.50)",
        state.bouncy_spring.value(),
        state.bouncy_spring.velocity(),
        state.target_val,
        Color32::from_rgb(250, 179, 135),
        track_height,
        |new_target| state.set_target(new_target),
    );

    ui.add_space(2.0);
    render_spring_track(
        ui,
        "💚 OpenRGB / Neovide (ω0 = 22, ζ = 0.65)",
        state.openrgb_spring.value(),
        state.openrgb_spring.velocity(),
        state.target_val,
        Color32::from_rgb(0, 255, 136),
        track_height,
        |new_target| state.set_target(new_target),
    );

    ui.add_space(2.0);
    let regime_label = if state.custom_damping < 0.9999 {
        "Underdamped"
    } else if state.custom_damping > 1.0001 {
        "Overdamped"
    } else {
        "Critically Damped"
    };

    render_spring_track(
        ui,
        &format!("🔵 Custom (ω0 = {:.0}, ζ = {:.2}) — {}", state.custom_frequency, state.custom_damping, regime_label),
        state.custom_spring.value(),
        state.custom_spring.velocity(),
        state.target_val,
        Color32::from_rgb(137, 220, 235),
        track_height,
        |new_target| state.set_target(new_target),
    );

    ui.add_space(4.0);

    // Multi-Shape Bézier Morphing Showcase
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🎯 Multi-Shape Bézier Morphing Highlight").strong());
            ui.separator();
            ui.label("Preset:");
            ui.selectable_value(&mut state.highlight_preset, HighlightPreset::Gentle, "🟣 Gentle");
            ui.selectable_value(&mut state.highlight_preset, HighlightPreset::Snappy, "🟢 Snappy");
            ui.selectable_value(&mut state.highlight_preset, HighlightPreset::Bouncy, "🟠 Bouncy");
            ui.selectable_value(&mut state.highlight_preset, HighlightPreset::OpenRGB, "💚 OpenRGB");
            ui.selectable_value(&mut state.highlight_preset, HighlightPreset::Custom, "🔵 Custom");
        });
        ui.add_space(4.0);

        // Define distinct geometric shapes to demonstrate dynamic curvature morphing
        let components = [
            MorphComponent {
                label: "⭕ Circle",
                description: "r = 22px",
                size: Vec2::new(56.0, 44.0),
                rounding: 22.0,
                is_circle: true,
            },
            MorphComponent {
                label: "⬛ Sharp Card",
                description: "r = 0px (Square)",
                size: Vec2::new(125.0, 44.0),
                rounding: 0.0,
                is_circle: false,
            },
            MorphComponent {
                label: "💊 Pill Capsule",
                description: "r = 16px (Badge)",
                size: Vec2::new(115.0, 44.0),
                rounding: 16.0,
                is_circle: false,
            },
            MorphComponent {
                label: "📱 Squircle Tile",
                description: "r = 12px (Smooth)",
                size: Vec2::new(110.0, 44.0),
                rounding: 12.0,
                is_circle: false,
            },
            MorphComponent {
                label: "🔲 Standard Card",
                description: "r = 6px (Sleek)",
                size: Vec2::new(120.0, 44.0),
                rounding: 6.0,
                is_circle: false,
            },
            MorphComponent {
                label: "🔘 Mini-Circle",
                description: "r = 16px (Icon)",
                size: Vec2::new(44.0, 44.0),
                rounding: 22.0,
                is_circle: true,
            },
        ];

        let mut target_rect = None;
        let mut target_rounding = 6.0;

        // Row 1: First 3 diverse shapes
        ui.horizontal(|ui| {
            for (idx, comp) in components.iter().take(3).enumerate() {
                let (rect, resp) = ui.allocate_exact_size(comp.size, Sense::click());
                if resp.clicked() {
                    state.selected_component = idx;
                }
                if idx == state.selected_component {
                    target_rect = Some(rect);
                    target_rounding = comp.rounding;
                }

                // Render component geometry
                ui.painter().rect(
                    rect,
                    Rounding::same(comp.rounding),
                    Color32::from_rgb(30, 30, 46),
                    Stroke::new(1.0, Color32::from_rgb(49, 50, 68)),
                );

                let center = rect.center();
                if comp.is_circle && comp.size.x <= 56.0 {
                    ui.painter().text(
                        center - Vec2::new(0.0, 6.0),
                        egui::Align2::CENTER_CENTER,
                        comp.label,
                        egui::FontId::proportional(11.5),
                        Color32::from_rgb(205, 214, 244),
                    );
                    ui.painter().text(
                        center + Vec2::new(0.0, 8.0),
                        egui::Align2::CENTER_CENTER,
                        comp.description,
                        egui::FontId::monospace(9.0),
                        Color32::from_rgb(147, 153, 178),
                    );
                } else {
                    ui.painter().text(
                        rect.min + Vec2::new(12.0, 8.0),
                        egui::Align2::LEFT_TOP,
                        comp.label,
                        egui::FontId::proportional(12.5),
                        Color32::from_rgb(205, 214, 244),
                    );
                    ui.painter().text(
                        rect.min + Vec2::new(12.0, 24.0),
                        egui::Align2::LEFT_TOP,
                        comp.description,
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(147, 153, 178),
                    );
                }
                ui.add_space(6.0);
            }
        });

        ui.add_space(6.0);

        // Row 2: Remaining 3 shapes
        ui.horizontal(|ui| {
            for (idx, comp) in components.iter().skip(3).enumerate() {
                let real_idx = idx + 3;
                let (rect, resp) = ui.allocate_exact_size(comp.size, Sense::click());
                if resp.clicked() {
                    state.selected_component = real_idx;
                }
                if real_idx == state.selected_component {
                    target_rect = Some(rect);
                    target_rounding = comp.rounding;
                }

                ui.painter().rect(
                    rect,
                    Rounding::same(comp.rounding),
                    Color32::from_rgb(30, 30, 46),
                    Stroke::new(1.0, Color32::from_rgb(49, 50, 68)),
                );

                let center = rect.center();
                if comp.is_circle && comp.size.x <= 56.0 {
                    ui.painter().text(
                        center - Vec2::new(0.0, 6.0),
                        egui::Align2::CENTER_CENTER,
                        comp.label,
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(205, 214, 244),
                    );
                    ui.painter().text(
                        center + Vec2::new(0.0, 8.0),
                        egui::Align2::CENTER_CENTER,
                        comp.description,
                        egui::FontId::monospace(9.0),
                        Color32::from_rgb(147, 153, 178),
                    );
                } else {
                    ui.painter().text(
                        rect.min + Vec2::new(12.0, 8.0),
                        egui::Align2::LEFT_TOP,
                        comp.label,
                        egui::FontId::proportional(12.5),
                        Color32::from_rgb(205, 214, 244),
                    );
                    ui.painter().text(
                        rect.min + Vec2::new(12.0, 24.0),
                        egui::Align2::LEFT_TOP,
                        comp.description,
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(147, 153, 178),
                    );
                }
                ui.add_space(6.0);
            }
        });

        if let Some(target) = target_rect {
            state.selection_highlight.set_target_with_rounding(target, target_rounding);
        }

        // Paint the SpringRect highlight with Bézier curves morphing live across shapes
        state.selection_highlight.paint(ui.painter());
    });

    ui.add_space(4.0);

    // Tuning Sliders and Real-time Oscilloscope
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.label(egui::RichText::new("⚙️ Live Parameter Tuning").strong());
            ui.add(egui::Slider::new(&mut state.speed_scale, 0.5..=3.0).text("Speed Scale (Multiplier)").suffix("x"));
            ui.add(egui::Slider::new(&mut state.custom_frequency, 5.0..=60.0).text("Frequency ω0 (rad/s)"));
            ui.add(egui::Slider::new(&mut state.custom_damping, 0.05..=2.5).text("Damping Ratio ζ"));
            ui.label(format!(
                "Status: {} | Rounding: {:.1}px",
                if state.is_animating() { "⚡ Active Morphing" } else { "✓ Settled" },
                state.selection_highlight.current_rounding
            ));
        });

        cols[1].group(|ui| {
            ui.label(egui::RichText::new("📈 Real-Time Trajectory Trace").strong());
            render_oscilloscope(ui, state);
        });
    });
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
            let pad = 24.0;
            let track_w = rect.width() - pad * 2.0;
            let frac = ((pos.x - (rect.min.x + pad)) / track_w).clamp(0.0, 1.0);
            on_click(frac);
        }
    }

    let painter = ui.painter();

    // Track card background with intact rounded corners
    painter.rect(
        rect,
        Rounding::same(6.0),
        Color32::from_rgb(30, 32, 48),
        Stroke::new(1.0, Color32::from_rgb(69, 71, 90)),
    );

    let pad = 24.0;
    let track_min_x = rect.min.x + pad;
    let track_max_x = rect.max.x - pad;
    let track_w = track_max_x - track_min_x;
    let track_y = rect.center().y + 3.0;

    // Track rail
    painter.line_segment(
        [Pos2::new(track_min_x, track_y), Pos2::new(track_max_x, track_y)],
        Stroke::new(3.0, Color32::from_rgb(49, 50, 68)),
    );

    // Target marker (dotted vertical pin)
    let target_x = track_min_x + target_val * track_w;
    painter.line_segment(
        [Pos2::new(target_x, track_y - 8.0), Pos2::new(target_x, track_y + 8.0)],
        Stroke::new(2.0, Color32::from_rgb(186, 194, 222)),
    );

    // Spring coil lines from left to current ball
    let current_x = track_min_x + current_val.clamp(-0.2, 1.2) * track_w;
    let coil_segments = 14;
    let mut prev_pt = Pos2::new(track_min_x, track_y);
    for i in 1..=coil_segments {
        let frac = i as f32 / coil_segments as f32;
        let x = track_min_x + (current_x - track_min_x) * frac;
        let y_offset = if i % 2 == 1 { -3.5 } else { 3.5 };
        let pt = Pos2::new(x, track_y + y_offset);
        painter.line_segment([prev_pt, pt], Stroke::new(1.0, Color32::from_rgb(108, 112, 134)));
        prev_pt = pt;
    }
    painter.line_segment([prev_pt, Pos2::new(current_x, track_y)], Stroke::new(1.0, Color32::from_rgb(108, 112, 134)));

    // Animated ball head
    painter.circle(
        Pos2::new(current_x, track_y),
        7.0,
        accent,
        Stroke::new(1.5, Color32::WHITE),
    );

    // Title and telemetry text
    painter.text(
        Pos2::new(rect.min.x + 10.0, rect.min.y + 7.0),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(11.0),
        Color32::from_rgb(205, 214, 244),
    );

    let telemetry = format!("pos: {:.2} • vel: {:+.1}", current_val, velocity);
    painter.text(
        Pos2::new(rect.max.x - 10.0, rect.min.y + 7.0),
        egui::Align2::RIGHT_TOP,
        telemetry,
        egui::FontId::monospace(9.5),
        Color32::from_rgb(147, 153, 178),
    );
}

fn render_oscilloscope(ui: &mut Ui, state: &SpringDemoState) {
    let height = 74.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let painter = ui.painter();

    painter.rect(
        rect,
        Rounding::same(4.0),
        Color32::from_rgb(24, 24, 37),
        Stroke::new(1.0, Color32::from_rgb(49, 50, 68)),
    );

    // Target baseline
    let target_y = rect.max.y - (state.target_val * (rect.height() - 16.0) + 8.0);
    painter.line_segment(
        [Pos2::new(rect.min.x, target_y), Pos2::new(rect.max.x, target_y)],
        Stroke::new(1.0, Color32::from_rgb(69, 71, 90)),
    );

    draw_trace(&painter, rect, &state.history_gentle, Color32::from_rgb(203, 166, 247));
    draw_trace(&painter, rect, &state.history_snappy, Color32::from_rgb(166, 227, 161));
    draw_trace(&painter, rect, &state.history_bouncy, Color32::from_rgb(250, 179, 135));
    draw_trace(&painter, rect, &state.history_openrgb, Color32::from_rgb(0, 255, 136));
    draw_trace(&painter, rect, &state.history_custom, Color32::from_rgb(137, 220, 235));
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
