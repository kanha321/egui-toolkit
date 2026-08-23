//! Declarative, responsive, constraint-aware nested split layouts for egui.

use egui::{Align, Layout, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2};
use crate::section::Section;
use crate::size::Size;
use crate::style::SplitStyle;

/// Layout axis for `Split`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Sections placed horizontally side-by-side (left to right).
    Horizontal,
    /// Sections stacked vertically (top to bottom).
    Vertical,
}

/// A declarative builder for responsive, constraint-aware multi-section split layouts.
///
/// Supports proportional fractions, fixed pixel sizes, 2D minimum/maximum constraints,
/// automatic remainder expansion, visual card framing, and automatic layout bounding.
///
/// # Customization
///
/// Use [`SplitStyle`] to configure global defaults for all card sections:
///
/// ```rust
/// use egui_layout::{Split, SplitStyle};
///
/// # egui::__run_test_ctx(|ctx| {
/// # egui::CentralPanel::default().show(ctx, |ui| {
/// Split::horizontal()
///     .style(
///         SplitStyle::default()
///             .with_spacing(8.0)
///             .with_card_rounding(12.0)
///             .with_card_padding(10.0)
///     )
///     .section(0.5, |ui| { ui.label("Left"); })
///     .section(0.5, |ui| { ui.label("Right"); })
///     .show(ui);
/// # });
/// # });
/// ```
///
/// # State Ownership
///
/// `Split` is constructed and consumed each frame; it holds no global or static state (`CODING_RULES §2`).
///
/// # Examples
///
/// ```rust
/// use egui_layout::{Split, Section};
///
/// # egui::__run_test_ctx(|ctx| {
/// # egui::CentralPanel::default().show(ctx, |ui| {
/// Split::horizontal()
///     .spacing(6.0)
///     .section(0.33, |ui| {
///         ui.label("Sidebar (33%)");
///     })
///     .section_min(0.67, 100.0, |ui| {
///         Split::vertical()
///             .section_fixed(48.0, |ui| {
///                 ui.label("Fixed Header (48px)");
///             })
///             .section_remainder(|ui| {
///                 ui.label("Flexible Workspace");
///             })
///             .show(ui);
///     })
///     .show(ui);
/// # });
/// # });
/// ```
pub struct Split<'a> {
    direction: Direction,
    style: SplitStyle,
    sections: Vec<Section<'a>>,
}

impl<'a> Split<'a> {
    /// Creates a horizontal split layout (columns placed left-to-right).
    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Horizontal,
            style: SplitStyle::default(),
            sections: Vec::new(),
        }
    }

    /// Creates a vertical split layout (rows stacked top-to-bottom).
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Vertical,
            style: SplitStyle::default(),
            sections: Vec::new(),
        }
    }

    // ── Style Configuration ──

    /// Sets the full visual style for this split and all its card sections.
    ///
    /// Individual `Section` overrides still take precedence over `SplitStyle` defaults.
    pub fn style(mut self, style: SplitStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the inter-section spacing in logical points (default: `4.0`).
    ///
    /// Shorthand for modifying `self.style.spacing`.
    pub fn spacing(mut self, px: f32) -> Self {
        self.style.spacing = px.max(0.0);
        self
    }

    /// Sets the default card corner rounding for all sections.
    ///
    /// Shorthand for modifying `self.style.card_rounding`.
    pub fn card_rounding(mut self, radius: f32) -> Self {
        self.style.card_rounding = Rounding::same(radius.max(0.0));
        self
    }

    /// Sets the default card inner padding for all sections.
    ///
    /// Shorthand for modifying `self.style.card_padding`.
    pub fn card_padding(mut self, padding: f32) -> Self {
        self.style.card_padding = padding.max(0.0);
        self
    }

    // ── Section Addition ──

    /// Appends a fully configured `Section` instance.
    pub fn add_section(mut self, section: Section<'a>) -> Self {
        self.sections.push(section);
        self
    }

    /// Configures and adds a section via a builder closure.
    pub fn section_with(mut self, build: impl FnOnce(Section<'a>) -> Section<'a>) -> Self {
        self.sections.push(build(Section::new()));
        self
    }

    /// Adds a section with a relative fraction of available space.
    ///
    /// Fractions do not need to sum to `1.0`; they are normalized proportionally across flexible sections.
    pub fn section(
        mut self,
        fraction: f32,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section::fraction(fraction).content(add_contents));
        self
    }

    /// Adds a fractional section with a minimum size constraint in logical points.
    pub fn section_min(
        mut self,
        fraction: f32,
        min_px: f32,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(
            Section::fraction(fraction)
                .min_size(min_px)
                .content(add_contents),
        );
        self
    }

    /// Adds a fractional section with both minimum and maximum size constraints.
    pub fn section_constrained(
        mut self,
        fraction: f32,
        min_px: f32,
        max_px: f32,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(
            Section::fraction(fraction)
                .min_size(min_px)
                .max_size(max_px)
                .content(add_contents),
        );
        self
    }

    /// Adds an exact fixed-size section in logical points (does not grow or shrink).
    pub fn section_fixed(
        mut self,
        px: f32,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section::fixed(px).content(add_contents));
        self
    }

    /// Adds a section that expands to claim all remaining available space.
    pub fn section_remainder(
        mut self,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section::remainder().content(add_contents));
        self
    }

    /// Adds a section with a custom `Size` policy.
    pub fn section_custom(
        mut self,
        size: Size,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section::new().size(size).content(add_contents));
        self
    }

    /// Adds a styled card section configured via a builder closure.
    pub fn section_card(
        mut self,
        fraction: f32,
        build: impl FnOnce(Section<'a>) -> Section<'a>,
    ) -> Self {
        let s = build(Section::fraction(fraction).card());
        self.sections.push(s);
        self
    }

    /// Generates `n` equal-width (or equal-height) sections using a generator closure.
    pub fn sections_equal(
        mut self,
        n: usize,
        add_content: impl Fn(usize, &mut Ui) + 'a + Clone,
    ) -> Self {
        if n == 0 {
            return self;
        }
        let fraction = 1.0 / (n as f32);
        for i in 0..n {
            let add = add_content.clone();
            self = self.section(fraction, move |ui| add(i, ui));
        }
        self
    }

    /// Generates multiple sections with specified proportional fractions.
    pub fn sections_proportional(
        mut self,
        fractions: &[f32],
        add_content: impl Fn(usize, f32, &mut Ui) + 'a + Clone,
    ) -> Self {
        for (i, &fraction) in fractions.iter().enumerate() {
            let add = add_content.clone();
            self = self.section(fraction, move |ui| add(i, fraction, ui));
        }
        self
    }

    // ── Queries ──

    /// Returns the number of sections currently added to this split.
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Returns whether this split currently has zero sections.
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Returns the layout direction of this split.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Calculates the minimum required length along the primary axis
    /// to satisfy all section constraints and inter-section spacing.
    pub fn min_primary_length(&self) -> f32 {
        let policies: Vec<Size> = self.sections.iter().map(|s| s.size).collect();
        Self::compute_min_length(self.style.spacing, &policies)
    }

    /// Calculates the minimum required length along the cross axis across all sections.
    pub fn min_cross_length(&self) -> f32 {
        self.sections
            .iter()
            .map(|s| s.cross_min())
            .fold(0.0_f32, |max, val| max.max(val))
    }

    /// Automatically computes the total 2D minimum dimensions (`Vec2`) required by this split layout tree.
    pub fn min_size(&self) -> Vec2 {
        match self.direction {
            Direction::Horizontal => Vec2::new(self.min_primary_length(), self.min_cross_length()),
            Direction::Vertical => Vec2::new(self.min_cross_length(), self.min_primary_length()),
        }
    }

    /// Returns the minimum required width in logical points.
    pub fn min_width(&self) -> f32 {
        self.min_size().x
    }

    /// Returns the minimum required height in logical points.
    pub fn min_height(&self) -> f32 {
        self.min_size().y
    }

    /// One-liner to enforce the computed 2D minimum size on the active egui viewport window.
    pub fn enforce_min_size(&self, ctx: &egui::Context) {
        let min_size = self.min_size();
        if min_size.x > 0.0 || min_size.y > 0.0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(min_size));
        }
    }

    // ── Static computation helpers ──

    /// Calculates the minimum required length along the primary axis for a set of policies and spacing.
    pub fn compute_min_length(spacing: f32, policies: &[Size]) -> f32 {
        let n = policies.len();
        if n == 0 {
            return 0.0;
        }
        let total_spacing = (n as f32 - 1.0) * spacing;
        let min_sum: f32 = policies
            .iter()
            .map(|p| match p {
                Size::Exact(px) => px.max(0.0),
                Size::Fraction { min, .. } => min.unwrap_or(0.0),
                Size::Remainder { min } => min.unwrap_or(0.0),
            })
            .sum();
        min_sum + total_spacing
    }

    /// Resolves and calculates exact section lengths given available space and pure fraction weights.
    pub fn compute_sizes(available_length: f32, spacing: f32, fractions: &[f32]) -> Vec<f32> {
        let policies: Vec<Size> = fractions.iter().map(|&f| Size::fraction(f)).collect();
        Self::compute_sizes_with_policy(available_length, spacing, &policies)
    }

    /// Resolves and calculates exact section lengths using the constraint solver.
    pub fn compute_sizes_with_policy(
        available_length: f32,
        spacing: f32,
        policies: &[Size],
    ) -> Vec<f32> {
        let n = policies.len();
        if n == 0 {
            return Vec::new();
        }
        if n == 1 {
            return vec![available_length.max(0.0)];
        }

        let total_spacing = (n as f32 - 1.0) * spacing;
        let usable_length = (available_length - total_spacing).max(0.0);

        let mut sizes = vec![0.0; n];
        let mut is_locked = vec![false; n];

        // Step 1: Pre-allocate exact fixed sizes
        for (i, &policy) in policies.iter().enumerate() {
            if let Size::Exact(px) = policy {
                sizes[i] = px.max(0.0);
                is_locked[i] = true;
            }
        }

        // Step 2: Iterative constraint relaxation loop for flexible/fractional sections
        let max_iterations = n + 2;
        for _ in 0..max_iterations {
            let mut allocated_locked = 0.0;
            let mut unconstrained_fraction_sum = 0.0;
            let mut unconstrained_count = 0;

            for (i, &policy) in policies.iter().enumerate() {
                if is_locked[i] {
                    allocated_locked += sizes[i];
                } else {
                    unconstrained_count += 1;
                    match policy {
                        Size::Fraction { fraction, .. } => {
                            unconstrained_fraction_sum += fraction.max(0.0);
                        }
                        Size::Remainder { .. } => {
                            unconstrained_fraction_sum += 1.0;
                        }
                        Size::Exact(_) => {}
                    }
                }
            }

            if unconstrained_count == 0 {
                break;
            }

            let space_for_unconstrained = (usable_length - allocated_locked).max(0.0);
            let mut newly_clamped = false;

            for (i, &policy) in policies.iter().enumerate() {
                if is_locked[i] {
                    continue;
                }

                let tentative_size = if unconstrained_fraction_sum > 0.0 {
                    let weight = match policy {
                        Size::Fraction { fraction, .. } => fraction.max(0.0),
                        Size::Remainder { .. } => 1.0,
                        Size::Exact(_) => 0.0,
                    };
                    (weight / unconstrained_fraction_sum) * space_for_unconstrained
                } else {
                    space_for_unconstrained / unconstrained_count as f32
                };

                let (min, max) = match policy {
                    Size::Fraction { min, max, .. } => (min, max),
                    Size::Remainder { min } => (min, None),
                    Size::Exact(_) => (None, None),
                };

                let mut final_size = tentative_size;
                let mut should_lock = false;

                if let Some(min_val) = min {
                    if tentative_size < min_val {
                        final_size = min_val;
                        should_lock = true;
                    }
                }
                if let Some(max_val) = max {
                    if final_size > max_val {
                        final_size = max_val;
                        should_lock = true;
                    }
                }

                sizes[i] = final_size;
                if should_lock {
                    is_locked[i] = true;
                    newly_clamped = true;
                }
            }

            if !newly_clamped {
                break;
            }
        }

        // Step 3: If space is severely constrained (total sizes > usable_length), scale down proportionally
        let total_assigned: f32 = sizes.iter().sum();
        if total_assigned > usable_length && total_assigned > 0.0 {
            let scale = usable_length / total_assigned;
            for s in &mut sizes {
                *s *= scale;
            }
        } else if total_assigned < usable_length && n > 1 {
            // Float rounding remainder absorption: allocate exact leftover to the last flexible or last section
            let leading_sum: f32 = sizes[..n - 1].iter().sum();
            sizes[n - 1] = (usable_length - leading_sum).max(0.0);
        }

        sizes
    }

    // ── Rendering ──

    /// Renders the split layout, allocating sub-UIs for each section and consuming available space.
    pub fn show(self, ui: &mut Ui) -> Response {
        let n = self.sections.len();
        if n == 0 {
            return ui.allocate_response(Vec2::ZERO, Sense::hover());
        }

        let available = ui.available_size();
        let frame_padding = self.style.frame_padding;

        // Account for outer frame padding in available space
        let inner_available = Vec2::new(
            (available.x - frame_padding * 2.0).max(0.0),
            (available.y - frame_padding * 2.0).max(0.0),
        );

        let (primary_len, _cross_len) = match self.direction {
            Direction::Horizontal => (inner_available.x, inner_available.y),
            Direction::Vertical => (inner_available.y, inner_available.x),
        };

        let policies: Vec<Size> = self.sections.iter().map(|s| s.size).collect();
        let sizes = Self::compute_sizes_with_policy(primary_len, self.style.spacing, &policies);

        let (total_rect, response) = ui.allocate_exact_size(available, Sense::hover());

        // Draw outer frame if configured
        let has_outer_frame = self.style.frame_bg.is_some() || self.style.frame_stroke.is_some();
        if has_outer_frame {
            let frame_bg = self.style.frame_bg.unwrap_or(egui::Color32::TRANSPARENT);
            let frame_stroke = self.style.frame_stroke.unwrap_or(egui::Stroke::NONE);
            ui.painter().rect(total_rect, self.style.frame_rounding, frame_bg, frame_stroke);
        }

        // Inner rect after frame padding
        let inner_rect = if frame_padding > 0.0 {
            total_rect.shrink(frame_padding)
        } else {
            total_rect
        };

        match self.direction {
            Direction::Horizontal => {
                let mut current_x = inner_rect.min.x;
                for (i, section) in self.sections.into_iter().enumerate() {
                    let w = sizes[i];
                    let section_rect = Rect::from_min_size(
                        Pos2::new(current_x, inner_rect.min.y),
                        Vec2::new(w, inner_rect.height()),
                    );

                    Self::render_section_slot(ui, section_rect, section, &self.style);
                    current_x += w + self.style.spacing;
                }
            }
            Direction::Vertical => {
                let mut current_y = inner_rect.min.y;
                for (i, section) in self.sections.into_iter().enumerate() {
                    let h = sizes[i];
                    let section_rect = Rect::from_min_size(
                        Pos2::new(inner_rect.min.x, current_y),
                        Vec2::new(inner_rect.width(), h),
                    );

                    Self::render_section_slot(ui, section_rect, section, &self.style);
                    current_y += h + self.style.spacing;
                }
            }
        }

        response
    }

    fn render_section_slot(ui: &mut Ui, section_rect: Rect, section: Section<'a>, style: &SplitStyle) {
        if let Some(on_rect) = section.on_rect {
            on_rect(section_rect);
        }

        if section.is_card {
            if section_rect.width() <= 0.0 || section_rect.height() <= 0.0 {
                return;
            }

            // Resolution order: Section override > SplitStyle > ui.visuals() fallback
            let bg = section.bg
                .or(style.card_bg)
                .unwrap_or(ui.visuals().faint_bg_color);
            let stroke = section.stroke
                .or(style.card_stroke)
                .unwrap_or(ui.visuals().window_stroke);
            let rounding = section.rounding
                .unwrap_or(style.card_rounding);

            // Guaranteed intact 4 rounded corners
            ui.painter().rect(section_rect, rounding, bg, stroke);

            let padding = section.padding.unwrap_or(style.card_padding);
            let content_rect = section_rect.shrink(padding);

            if content_rect.width() > 0.0 && content_rect.height() > 0.0 {
                let mut child_ui = ui.child_ui(content_rect, Layout::top_down(Align::Min));
                child_ui.set_clip_rect(child_ui.clip_rect().intersect(content_rect));

                if let Some(title) = &section.title {
                    let title_color = section.title_color
                        .or(style.title_color)
                        .unwrap_or(ui.visuals().strong_text_color());
                    child_ui.colored_label(
                        title_color,
                        egui::RichText::new(title).strong().size(style.title_size),
                    );
                }

                if let Some(subtitle) = &section.subtitle {
                    if content_rect.height() > 28.0 {
                        let subtitle_color = section.subtitle_color
                            .or(style.subtitle_color)
                            .unwrap_or(ui.visuals().text_color());
                        child_ui.label(
                            egui::RichText::new(subtitle).size(style.subtitle_size).color(subtitle_color),
                        );
                    }
                }

                (section.add_contents)(&mut child_ui);
            }
        } else {
            let mut child_ui = ui.child_ui(section_rect, Layout::top_down(Align::Min));
            child_ui.set_clip_rect(child_ui.clip_rect().intersect(section_rect));
            (section.add_contents)(&mut child_ui);
        }
    }
}
