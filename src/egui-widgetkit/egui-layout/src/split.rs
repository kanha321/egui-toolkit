//! Declarative, responsive, constraint-aware nested split layouts for egui.
//!
//! Supports interactive resize dividers, collapsible sections with spring-animated
//! transitions, and visual card framing.

use egui::{
    Align, CursorIcon, Id, Layout, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2,
};
use spring_core::{MotionPhysics, SpringParams};

use crate::collapse::CollapseMode;
use crate::resize::SplitState;
use crate::section::Section;
use crate::size::Size;
use crate::style::{DividerVisibility, SplitStyle};

/// Layout axis for `Split`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Sections placed horizontally side-by-side (left to right).
    Horizontal,
    /// Sections stacked vertically (top to bottom).
    Vertical,
}

/// Response from [`Split::show()`], wrapping the base [`egui::Response`] with
/// interactive event information.
///
/// Implements [`std::ops::Deref`] to `Response`, so `.rect`, `.hovered()`, etc.
/// work transparently.
pub struct SplitResponse {
    /// The base egui Response for the split's allocated area.
    pub response: Response,
    /// The `Id` used for this split's persisted state.
    pub id: Id,
    /// Index of divider that was double-clicked this frame, if any.
    pub double_clicked_divider: Option<usize>,
    /// Index of section whose collapse chevron was clicked this frame, if any.
    pub toggled_section: Option<usize>,
}

impl std::ops::Deref for SplitResponse {
    type Target = Response;
    fn deref(&self) -> &Response {
        &self.response
    }
}

/// A declarative builder for responsive, constraint-aware multi-section split layouts.
///
/// Supports proportional fractions, fixed pixel sizes, 2D minimum/maximum constraints,
/// automatic remainder expansion, visual card framing, interactive resize dividers,
/// collapsible sections with spring-animated transitions, and automatic layout bounding.
///
/// # Interactive Features
///
/// - **Resizable**: Set `.resizable(true)` to enable drag-to-resize dividers between sections.
/// - **Collapsible**: Mark individual sections with `.collapsible()` for animated collapse/expand.
/// - **Spring Animation**: All transitions use `spring-core` physics. Customize via `.motion()`.
///
/// Interactive state is auto-persisted via `egui::Id` — no boilerplate needed,
/// even for deeply nested splits.
///
/// # Examples
///
/// ```rust
/// use egui_layout::{Split, Section, CollapseMode};
///
/// # egui::__run_test_ctx(|ctx| {
/// # egui::CentralPanel::default().show(ctx, |ui| {
/// let mut sidebar_open = true;
/// Split::horizontal()
///     .resizable(true)
///     .id_salt("demo_split")
///     .spacing(6.0)
///     .add_section(
///         Section::fraction(0.25)
///             .title("Sidebar")
///             .collapsible(&mut sidebar_open, CollapseMode::FixedBar(40.0))
///             .content(|ui| { ui.label("Sidebar content"); })
///     )
///     .section(0.75, |ui| { ui.label("Main content"); })
///     .show(ui);
/// # });
/// # });
/// ```
///
/// # State Ownership
///
/// `Split` is constructed and consumed each frame; it holds no global or static state (`CODING_RULES §2`).
pub struct Split<'a> {
    direction: Direction,
    style: SplitStyle,
    sections: Vec<Section<'a>>,
    resizable: bool,
    id_salt: Option<Id>,
    motion: MotionPhysics,
}

impl<'a> Split<'a> {
    /// Creates a horizontal split layout (columns placed left-to-right).
    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Horizontal,
            style: SplitStyle::default(),
            sections: Vec::new(),
            resizable: false,
            id_salt: None,
            motion: MotionPhysics::Responsive,
        }
    }

    /// Creates a vertical split layout (rows stacked top-to-bottom).
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Vertical,
            style: SplitStyle::default(),
            sections: Vec::new(),
            resizable: false,
            id_salt: None,
            motion: MotionPhysics::Responsive,
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

    /// Returns a reference to the current visual style.
    pub fn get_style(&self) -> &SplitStyle {
        &self.style
    }

    /// Sets the inter-section spacing in logical points (default: `4.0`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.style.spacing = px.max(0.0);
        self
    }

    /// Sets the default card corner rounding for all sections.
    pub fn card_rounding(mut self, radius: f32) -> Self {
        self.style.card_rounding = Rounding::same(radius.max(0.0));
        self
    }

    /// Sets the default card inner padding for all sections.
    pub fn card_padding(mut self, padding: f32) -> Self {
        self.style.card_padding = padding.max(0.0);
        self
    }

    // ── Interactive Configuration ──

    /// Enables interactive drag-resize dividers between sections (default: `false`).
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets the visual visibility mode for resize dividers (default: [`DividerVisibility::HoverOnly`]).
    /// - `HoverOnly`: invisible at rest, visible on hover/drag.
    /// - `Visible`: always visible.
    /// - `Hidden`: never rendered visually (resizing still functions).
    pub fn divider_visibility(mut self, visibility: DividerVisibility) -> Self {
        self.style.divider_visibility = visibility;
        self
    }

    /// Convenience helper to set dividers to always visible.
    pub fn divider_always_visible(mut self) -> Self {
        self.style.divider_visibility = DividerVisibility::Visible;
        self
    }

    /// Convenience helper to set dividers to visible on hover only.
    pub fn divider_hover_only(mut self) -> Self {
        self.style.divider_visibility = DividerVisibility::HoverOnly;
        self
    }

    /// Convenience helper to completely hide dividers visually.
    pub fn divider_hidden(mut self) -> Self {
        self.style.divider_visibility = DividerVisibility::Hidden;
        self
    }

    /// Sets whether divider lines are visible when idle (`true` = Visible, `false` = HoverOnly).
    pub fn divider_idle_visible(mut self, visible: bool) -> Self {
        self.style.divider_visibility = if visible {
            DividerVisibility::Visible
        } else {
            DividerVisibility::HoverOnly
        };
        self
    }

    /// Alias for [`Self::divider_idle_visible`].
    pub fn divider_visible(self, visible: bool) -> Self {
        self.divider_idle_visible(visible)
    }

    /// Sets a stable identity for this split's persisted interactive state.
    ///
    /// Recommended for splits whose section count or position changes dynamically.
    /// If not set, an `Id` is auto-derived from the split's position in the UI tree.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }

    /// Sets the spring physics preset for collapse/expand/reset animations.
    ///
    /// Default: [`MotionPhysics::Snappy`]. Use [`MotionPhysics::Off`] for instant snap.
    pub fn motion(mut self, physics: MotionPhysics) -> Self {
        self.motion = physics;
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

    /// Calculates the minimum required length along the primary axis.
    pub fn min_primary_length(&self) -> f32 {
        let policies: Vec<Size> = self.sections.iter().map(|s| s.size).collect();
        Self::compute_min_length(self.style.spacing, &policies)
    }

    /// Calculates the minimum required length along the cross axis.
    pub fn min_cross_length(&self) -> f32 {
        self.sections
            .iter()
            .map(|s| s.cross_min())
            .fold(0.0_f32, |max, val| max.max(val))
    }

    /// Automatically computes the total 2D minimum dimensions required by this split layout tree.
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

        // Step 3: If space is severely constrained, scale down proportionally
        let total_assigned: f32 = sizes.iter().sum();
        if total_assigned > usable_length && total_assigned > 0.0 {
            let scale = usable_length / total_assigned;
            for s in &mut sizes {
                *s *= scale;
            }
        } else if total_assigned < usable_length && n > 1 {
            // Float rounding remainder absorption
            let leading_sum: f32 = sizes[..n - 1].iter().sum();
            sizes[n - 1] = (usable_length - leading_sum).max(0.0);
        }

        sizes
    }

    // ── Rendering ──

    /// Renders the split layout, allocating sub-UIs for each section and consuming available space.
    ///
    /// Interactive state (resize overrides, collapse animation springs) is auto-persisted
    /// via `egui::Id`. Use `.id_salt()` for stable identity if needed.
    ///
    /// Returns a [`SplitResponse`] wrapping the base `egui::Response` with event info.
    pub fn show(self, ui: &mut Ui) -> SplitResponse {
        let n = self.sections.len();
        if n == 0 {
            let response = ui.allocate_response(Vec2::ZERO, Sense::hover());
            return SplitResponse {
                id: Id::NULL,
                response,
                double_clicked_divider: None,
                toggled_section: None,
            };
        }

        // ── 1. Resolve Id ──
        let split_id = self.id_salt.unwrap_or_else(|| ui.auto_id_with("split"));

        // ── 2. Load persistent state ──
        let mut state: SplitState = ui.data(|d| d.get_temp(split_id)).unwrap_or_default();

        // ── 3. Spatial setup ──
        let available = ui.available_size();
        let frame_padding = self.style.frame_padding;
        let inner_available = Vec2::new(
            (available.x - frame_padding * 2.0).max(0.0),
            (available.y - frame_padding * 2.0).max(0.0),
        );

        let (primary_len, _cross_len) = match self.direction {
            Direction::Horizontal => (inner_available.x, inner_available.y),
            Direction::Vertical => (inner_available.y, inner_available.x),
        };

        // ── 4. Determine spring params ──
        let spring_params = self.motion.to_params()
            .unwrap_or(SpringParams::responsive());
        let use_springs = self.motion.is_enabled();

        // ── 5. Determine collapse states and build effective policies ──
        let mut is_collapsed = vec![false; n];
        let mut collapsed_targets = vec![0.0_f32; n];

        // Header-only height estimate (title text line + padding)
        let header_only_height = self.style.title_size + self.style.card_padding * 2.0 + 4.0;

        for (i, section) in self.sections.iter().enumerate() {
            if let Some(ref expanded) = section.collapse_expanded {
                if !**expanded {
                    is_collapsed[i] = true;
                    if let Some(ref config) = section.collapse_config {
                        collapsed_targets[i] = match config.mode {
                            CollapseMode::Hidden => 0.0,
                            CollapseMode::FixedBar(px) => px.max(0.0),
                            CollapseMode::HeaderOnly => header_only_height,
                        };
                    }
                }
            }
        }

        // Count visible sections for spacing calculation
        let visible_count = (0..n)
            .filter(|&i| {
                if is_collapsed[i] {
                    if let Some(ref config) = self.sections[i].collapse_config {
                        !matches!(config.mode, CollapseMode::Hidden)
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .count();

        let effective_spacing = if visible_count > 1 {
            self.style.spacing
        } else {
            0.0
        };

        // Build effective policies: collapsed sections → Exact, others → declared or override
        let mut effective_policies: Vec<Size> = Vec::with_capacity(n);
        for (i, section) in self.sections.iter().enumerate() {
            if is_collapsed[i] {
                effective_policies.push(Size::Exact(collapsed_targets[i]));
            } else if let Some(ov) = state.overrides.get(i).copied().flatten() {
                effective_policies.push(Size::Exact(ov));
            } else {
                effective_policies.push(section.size);
            }
        }

        // ── 6. Run constraint solver for target sizes and steady-state expanded sizes ──
        let target_sizes = Self::compute_sizes_with_policy(primary_len, effective_spacing, &effective_policies);

        // Compute true steady-state expanded sizes (without any collapse overrides)
        // so that during collapse/expand animations (and spring overshoots/bounces),
        // the expanded content layout width remains 100% constant and never oscillates or reflows.
        let mut expanded_policies: Vec<Size> = Vec::with_capacity(n);
        for (i, section) in self.sections.iter().enumerate() {
            if let Some(ov) = state.overrides.get(i).copied().flatten() {
                expanded_policies.push(Size::Exact(ov));
            } else {
                expanded_policies.push(section.size);
            }
        }
        let steady_expanded_spacing = if n > 1 { self.style.spacing } else { 0.0 };
        let steady_expanded_sizes = Self::compute_sizes_with_policy(primary_len, steady_expanded_spacing, &expanded_policies);

        // ── 7. Sync state and animate springs ──
        state.sync_to_count(n, &target_sizes, spring_params);

        // Track expanded sizes and set spring targets
        for (i, &target) in target_sizes.iter().enumerate() {
            if !is_collapsed[i] {
                if let Some(exp) = state.expanded_sizes.get_mut(i) {
                    *exp = target.max(*exp).max(80.0);
                }
            }
            state.set_target(i, target);
            // Update spring params
            if let Some(spring) = state.springs.get_mut(i) {
                spring.params = spring_params;
            }
        }

        // Advance springs
        let dt = ui.ctx().input(|i| i.stable_dt);
        if use_springs {
            state.advance(dt);
        } else {
            // Instant snap — teleport all springs to targets
            for spring in &mut state.springs {
                spring.current = spring.target;
                spring.velocity = 0.0;
            }
        }

        // ── 8. Compute rendered sizes from springs, normalized ──
        let mut rendered_sizes: Vec<f32> = state.springs.iter().map(|s| s.current.max(0.0)).collect();
        if rendered_sizes.len() != n {
            rendered_sizes = target_sizes.clone();
        }

        // Normalize: ensure rendered sizes sum to usable_length
        let visible_spacing = if visible_count > 1 {
            (visible_count as f32 - 1.0) * self.style.spacing
        } else {
            0.0
        };
        let usable = (primary_len - visible_spacing).max(0.0);
        let rendered_sum: f32 = rendered_sizes.iter().sum();
        if rendered_sum > 0.0 && (rendered_sum - usable).abs() > 0.5 {
            let scale = usable / rendered_sum;
            for s in &mut rendered_sizes {
                *s *= scale;
            }
        }

        // ── 9. Allocate and render ──
        let (total_rect, response) = ui.allocate_exact_size(available, Sense::hover());

        // Draw outer frame if configured
        let has_outer_frame = self.style.frame_bg.is_some() || self.style.frame_stroke.is_some();
        if has_outer_frame {
            let frame_bg = self.style.frame_bg.unwrap_or(egui::Color32::TRANSPARENT);
            let frame_stroke = self.style.frame_stroke.unwrap_or(egui::Stroke::NONE);
            ui.painter().rect(total_rect, self.style.frame_rounding, frame_bg, frame_stroke);
        }

        let inner_rect = if frame_padding > 0.0 {
            total_rect.shrink(frame_padding)
        } else {
            total_rect
        };

        let mut double_clicked_divider: Option<usize> = None;
        let mut toggled_section: Option<usize> = None;

        // Collect section rects
        let mut section_rects = Vec::with_capacity(n);

        match self.direction {
            Direction::Horizontal => {
                let mut current_x = inner_rect.min.x;
                for i in 0..n {
                    let w = rendered_sizes[i];

                    // Skip spacing for Hidden-collapsed sections
                    let skip_section = is_collapsed[i]
                        && self.sections[i].collapse_config.as_ref()
                            .map_or(false, |c| matches!(c.mode, CollapseMode::Hidden))
                        && state.springs.get(i).map_or(true, |s| s.is_settled());

                    if skip_section && w < 0.5 {
                        section_rects.push(Rect::NOTHING);
                        continue;
                    }

                    let section_rect = Rect::from_min_size(
                        Pos2::new(current_x, inner_rect.min.y),
                        Vec2::new(w, inner_rect.height()),
                    );
                    section_rects.push(section_rect);
                    current_x += w + self.style.spacing;
                }
            }
            Direction::Vertical => {
                let mut current_y = inner_rect.min.y;
                for i in 0..n {
                    let h = rendered_sizes[i];

                    let skip_section = is_collapsed[i]
                        && self.sections[i].collapse_config.as_ref()
                            .map_or(false, |c| matches!(c.mode, CollapseMode::Hidden))
                        && state.springs.get(i).map_or(true, |s| s.is_settled());

                    if skip_section && h < 0.5 {
                        section_rects.push(Rect::NOTHING);
                        continue;
                    }

                    let section_rect = Rect::from_min_size(
                        Pos2::new(inner_rect.min.x, current_y),
                        Vec2::new(inner_rect.width(), h),
                    );
                    section_rects.push(section_rect);
                    current_y += h + self.style.spacing;
                }
            }
        }

        // ── 10. Render dividers (if resizable) ──
        if self.resizable && n > 1 {
            for i in 0..(n - 1) {
                // Skip divider if either adjacent section is Hidden-collapsed and settled
                let left_hidden = is_collapsed[i]
                    && self.sections[i].collapse_config.as_ref()
                        .map_or(false, |c| matches!(c.mode, CollapseMode::Hidden))
                    && state.springs.get(i).map_or(true, |s| s.is_settled());
                let right_hidden = is_collapsed[i + 1]
                    && self.sections[i + 1].collapse_config.as_ref()
                        .map_or(false, |c| matches!(c.mode, CollapseMode::Hidden))
                    && state.springs.get(i + 1).map_or(true, |s| s.is_settled());

                if left_hidden || right_hidden {
                    continue;
                }

                let left_rect = section_rects[i];
                let right_rect = section_rects[i + 1];
                if left_rect == Rect::NOTHING || right_rect == Rect::NOTHING {
                    continue;
                }

                let divider_rect = match self.direction {
                    Direction::Horizontal => {
                        let x_start = left_rect.max.x;
                        let x_end = right_rect.min.x;
                        let center_x = (x_start + x_end) / 2.0;
                        let half_hit = self.style.divider_hit_width / 2.0;
                        Rect::from_min_max(
                            Pos2::new(center_x - half_hit, inner_rect.min.y),
                            Pos2::new(center_x + half_hit, inner_rect.max.y),
                        )
                    }
                    Direction::Vertical => {
                        let y_start = left_rect.max.y;
                        let y_end = right_rect.min.y;
                        let center_y = (y_start + y_end) / 2.0;
                        let half_hit = self.style.divider_hit_width / 2.0;
                        Rect::from_min_max(
                            Pos2::new(inner_rect.min.x, center_y - half_hit),
                            Pos2::new(inner_rect.max.x, center_y + half_hit),
                        )
                    }
                };

                let divider_id = split_id.with(("divider", i));
                let divider_response = ui.interact(divider_rect, divider_id, Sense::click_and_drag());

                // Cursor
                let cursor = match self.direction {
                    Direction::Horizontal => CursorIcon::ResizeHorizontal,
                    Direction::Vertical => CursorIcon::ResizeVertical,
                };
                if divider_response.hovered() || divider_response.dragged() {
                    ui.ctx().set_cursor_icon(cursor);
                }

                // Visual: draw divider line according to configured DividerVisibility mode
                let is_active = divider_response.hovered() || divider_response.dragged();
                let should_draw = match self.style.divider_visibility {
                    DividerVisibility::Visible => true,
                    DividerVisibility::HoverOnly => is_active,
                    DividerVisibility::Hidden => false,
                };

                if should_draw {
                    let divider_color = if divider_response.dragged() {
                        self.style.divider_drag_color
                            .unwrap_or(ui.visuals().selection.bg_fill)
                    } else if divider_response.hovered() {
                        self.style.divider_hover_color
                            .unwrap_or(ui.visuals().widgets.hovered.bg_stroke.color)
                    } else {
                        self.style.divider_color
                            .unwrap_or(ui.visuals().widgets.noninteractive.bg_stroke.color)
                    };

                    let center = divider_rect.center();
                    match self.direction {
                        Direction::Horizontal => {
                            let half_thick = self.style.divider_thickness / 2.0;
                            let line_rect = Rect::from_min_max(
                                Pos2::new(center.x - half_thick, divider_rect.min.y),
                                Pos2::new(center.x + half_thick, divider_rect.max.y),
                            );
                            ui.painter().rect_filled(line_rect, Rounding::ZERO, divider_color);

                            // Grip dots (only shown when actively hovering/dragging)
                            if self.style.divider_show_grip && is_active {
                                let dot_r = 1.5;
                                for dy in [-6.0, 0.0, 6.0] {
                                    ui.painter().circle_filled(
                                        Pos2::new(center.x, center.y + dy),
                                        dot_r,
                                        divider_color,
                                    );
                                }
                            }
                        }
                        Direction::Vertical => {
                            let half_thick = self.style.divider_thickness / 2.0;
                            let line_rect = Rect::from_min_max(
                                Pos2::new(divider_rect.min.x, center.y - half_thick),
                                Pos2::new(divider_rect.max.x, center.y + half_thick),
                            );
                            ui.painter().rect_filled(line_rect, Rounding::ZERO, divider_color);

                            if self.style.divider_show_grip && is_active {
                                let dot_r = 1.5;
                                for dx in [-6.0, 0.0, 6.0] {
                                    ui.painter().circle_filled(
                                        Pos2::new(center.x + dx, center.y),
                                        dot_r,
                                        divider_color,
                                    );
                                }
                            }
                        }
                    }
                }

                // Handle drag
                if divider_response.dragged() {
                    let delta = match self.direction {
                        Direction::Horizontal => divider_response.drag_delta().x,
                        Direction::Vertical => divider_response.drag_delta().y,
                    };

                    if delta.abs() > 0.0 {
                        let left_size = rendered_sizes[i];
                        let right_size = rendered_sizes[i + 1];

                        let left_min = self.sections[i].primary_min();
                        let right_min = self.sections[i + 1].primary_min();

                        let new_left = (left_size + delta).max(left_min);

                        // Verify total doesn't change
                        let total = left_size + right_size;
                        let clamped_left = new_left.min(total - right_min);
                        let clamped_right = total - clamped_left;

                        state.teleport(i, clamped_left);
                        state.teleport(i + 1, clamped_right);
                    }
                }

                // Handle double-click
                if divider_response.double_clicked() {
                    double_clicked_divider = Some(i);
                }
            }
        }

        // ── 11. Render sections ──
        for (i, mut section) in self.sections.into_iter().enumerate() {
            // Extract the mutable collapse bool so we can toggle it after chevron clicks
            let mut collapse_bool_ref = section.collapse_expanded.take();
            let section_rect = section_rects[i];
            if section_rect == Rect::NOTHING {
                // Call on_rect even for hidden sections
                if let Some(on_rect) = section.on_rect {
                    on_rect(section_rect);
                }
                // Consume closures
                drop(section.add_contents);
                drop(section.collapsed_content);
                continue;
            }

            if section_rect.width() <= 0.0 || section_rect.height() <= 0.0 {
                if let Some(on_rect) = section.on_rect {
                    on_rect(section_rect);
                }
                drop(section.add_contents);
                drop(section.collapsed_content);
                continue;
            }

            let section_is_collapsed = is_collapsed[i];
            let section_is_collapsible = section.collapse_config.is_some();
            let show_chevron = section.collapse_config.as_ref()
                .map_or(false, |c| c.show_chevron);
            let collapse_mode = section.collapse_config.as_ref()
                .map(|c| c.mode);

            if let Some(on_rect) = section.on_rect {
                on_rect(section_rect);
            }

            if section.is_card {
                let bg = section.bg
                    .or(self.style.card_bg)
                    .unwrap_or(ui.visuals().faint_bg_color);
                let stroke = section.stroke
                    .or(self.style.card_stroke)
                    .unwrap_or(ui.visuals().window_stroke);
                let rounding = section.rounding
                    .unwrap_or(self.style.card_rounding);

                // Guaranteed intact 4 rounded corners
                ui.painter().rect(section_rect, rounding, bg, stroke);

                let padding = section.padding.unwrap_or(self.style.card_padding);
                let content_rect = section_rect.shrink(padding);

                let steady_expanded_len = steady_expanded_sizes.get(i).copied().unwrap_or(240.0).max(80.0);

                if content_rect.width() > 0.0 && content_rect.height() > 0.0 {
                    // Fixed steady layout width for the Expanded State:
                    // Regardless of whether the spring is currently expanding, collapsing, overshooting,
                    // or bouncing, the expanded view is ALWAYS laid out at its resting expanded size.
                    let steady_content_width = (steady_expanded_len - padding * 2.0).max(0.0);
                    let steady_content_height = (steady_expanded_len - padding * 2.0).max(0.0);

                    let layout_width = match self.direction {
                        Direction::Horizontal => {
                            if section_is_collapsible {
                                steady_content_width
                            } else {
                                content_rect.width()
                            }
                        }
                        Direction::Vertical => content_rect.width(),
                    };

                    let layout_height = match self.direction {
                        Direction::Vertical => {
                            if section_is_collapsible {
                                steady_content_height
                            } else {
                                content_rect.height()
                            }
                        }
                        Direction::Horizontal => content_rect.height(),
                    };

                    let virtual_rect = Rect::from_min_size(
                        content_rect.min,
                        Vec2::new(layout_width, layout_height),
                    );

                    let mut child_ui = ui.child_ui(virtual_rect, Layout::top_down(Align::Min));
                    // Scissor clip is locked to the physical animated card rectangle
                    child_ui.set_clip_rect(child_ui.clip_rect().intersect(section_rect.shrink(1.0)));

                    // ── Chevron + Title header ──
                    let mut header_rendered = false;
                    let mut chevron_clicked = false;

                    if section_is_collapsible && (section.title.is_some() || show_chevron) {
                        child_ui.horizontal(|ui| {
                            // Chevron toggle
                            if show_chevron {
                                let chevron = if section_is_collapsed { "▸" } else { "▾" };
                                let chevron_color = self.style.collapse_chevron_color
                                    .unwrap_or(ui.visuals().text_color());
                                let chevron_text = egui::RichText::new(chevron)
                                    .size(self.style.collapse_chevron_size)
                                    .color(chevron_color);
                                if ui.add(egui::Label::new(chevron_text).sense(Sense::click())).clicked() {
                                    chevron_clicked = true;
                                }
                            }

                            // Title
                            if let Some(title) = &section.title {
                                let title_color = section.title_color
                                    .or(self.style.title_color)
                                    .unwrap_or(ui.visuals().strong_text_color());
                                let title_text = egui::RichText::new(title)
                                    .strong()
                                    .size(self.style.title_size)
                                    .color(title_color);

                                if section_is_collapsible {
                                    // Make title clickable too for convenience
                                    if ui.add(egui::Label::new(title_text).sense(Sense::click())).clicked() {
                                        chevron_clicked = true;
                                    }
                                } else {
                                    ui.label(title_text);
                                }
                            }
                        });
                        header_rendered = true;

                        // Toggle the collapse bool if chevron/title was clicked
                        if chevron_clicked {
                            if let Some(ref mut expanded) = collapse_bool_ref {
                                **expanded = !**expanded;
                            }
                            toggled_section = Some(i);
                        }
                    } else if let Some(title) = &section.title {
                        let title_color = section.title_color
                            .or(self.style.title_color)
                            .unwrap_or(ui.visuals().strong_text_color());
                        child_ui.colored_label(
                            title_color,
                            egui::RichText::new(title).strong().size(self.style.title_size),
                        );
                        header_rendered = true;
                    }

                    // Subtitle (only when expanded or enough space)
                    if !section_is_collapsed || !matches!(collapse_mode, Some(CollapseMode::HeaderOnly)) {
                        if let Some(subtitle) = &section.subtitle {
                            if content_rect.height() > 28.0 {
                                let subtitle_color = section.subtitle_color
                                    .or(self.style.subtitle_color)
                                    .unwrap_or(ui.visuals().text_color());
                                child_ui.label(
                                    egui::RichText::new(subtitle)
                                        .size(self.style.subtitle_size)
                                        .color(subtitle_color),
                                );
                            }
                        }
                    }

                    // ── Content Transition Engine ──
                    // Decide whether to render collapsed representation (compact view) or expanded stable content
                    let show_collapsed_view = if section_is_collapsed {
                        match collapse_mode {
                            Some(CollapseMode::Hidden) => content_rect.width() <= 4.0 || content_rect.height() <= 4.0,
                            Some(CollapseMode::FixedBar(bar_px)) => {
                                let current_dim = match self.direction {
                                    Direction::Horizontal => section_rect.width(),
                                    Direction::Vertical => section_rect.height(),
                                };
                                current_dim <= bar_px + 20.0
                            }
                            Some(CollapseMode::HeaderOnly) => {
                                let current_dim = match self.direction {
                                    Direction::Horizontal => section_rect.width(),
                                    Direction::Vertical => section_rect.height(),
                                };
                                current_dim <= header_only_height + 12.0
                            }
                            None => false,
                        }
                    } else {
                        false
                    };

                    if show_collapsed_view {
                        match collapse_mode {
                            Some(CollapseMode::Hidden) => {
                                drop(section.add_contents);
                                drop(section.collapsed_content);
                            }
                            Some(CollapseMode::HeaderOnly) => {
                                if let Some(collapsed_fn) = section.collapsed_content {
                                    collapsed_fn(&mut child_ui);
                                }
                                drop(section.add_contents);
                            }
                            Some(CollapseMode::FixedBar(_)) => {
                                if let Some(collapsed_fn) = section.collapsed_content {
                                    collapsed_fn(&mut child_ui);
                                }
                                drop(section.add_contents);
                            }
                            None => {
                                (section.add_contents)(&mut child_ui);
                                drop(section.collapsed_content);
                            }
                        }
                    } else {
                        (section.add_contents)(&mut child_ui);
                        drop(section.collapsed_content);
                    }

                    let _ = header_rendered;
                }
            } else {
                // Non-card section
                let steady_expanded_len = steady_expanded_sizes.get(i).copied().unwrap_or(240.0).max(80.0);
                let layout_width = match self.direction {
                    Direction::Horizontal => {
                        if section_is_collapsible {
                            steady_expanded_len
                        } else {
                            section_rect.width()
                        }
                    }
                    Direction::Vertical => section_rect.width(),
                };
                let layout_height = match self.direction {
                    Direction::Vertical => {
                        if section_is_collapsible {
                            steady_expanded_len
                        } else {
                            section_rect.height()
                        }
                    }
                    Direction::Horizontal => section_rect.height(),
                };
                let virtual_rect = Rect::from_min_size(section_rect.min, Vec2::new(layout_width, layout_height));

                let show_collapsed_view = if section_is_collapsed {
                    match collapse_mode {
                        Some(CollapseMode::Hidden) => section_rect.width() <= 4.0 || section_rect.height() <= 4.0,
                        Some(CollapseMode::FixedBar(bar_px)) => {
                            let current_dim = match self.direction {
                                Direction::Horizontal => section_rect.width(),
                                Direction::Vertical => section_rect.height(),
                            };
                            current_dim <= bar_px + 20.0
                        }
                        Some(CollapseMode::HeaderOnly) => true,
                        None => false,
                    }
                } else {
                    false
                };

                if show_collapsed_view {
                    if matches!(collapse_mode, Some(CollapseMode::Hidden)) {
                        drop(section.add_contents);
                        drop(section.collapsed_content);
                    } else if let Some(collapsed_fn) = section.collapsed_content {
                        let mut child_ui = ui.child_ui(section_rect, Layout::top_down(Align::Min));
                        child_ui.set_clip_rect(child_ui.clip_rect().intersect(section_rect.expand(2.0)));
                        collapsed_fn(&mut child_ui);
                        drop(section.add_contents);
                    } else {
                        drop(section.add_contents);
                        drop(section.collapsed_content);
                    }
                } else {
                    let mut child_ui = ui.child_ui(virtual_rect, Layout::top_down(Align::Min));
                    child_ui.set_clip_rect(child_ui.clip_rect().intersect(section_rect.expand(2.0)));
                    (section.add_contents)(&mut child_ui);
                    drop(section.collapsed_content);
                }
            }
        }

        // ── 12. Apply chevron toggles ──
        // We need to toggle via the response because the sections have been consumed.
        // The actual toggle is done by the caller checking `toggled_section`.
        // BUT — Section borrows `&'a mut bool`, which was consumed. The chevron click
        // was already detected. We need to toggle via a different mechanism.
        //
        // Since sections have been consumed (moved into the loop), we can't toggle
        // the bools anymore here. Instead, we handle this in the rendering loop above
        // by detecting clicks, but we need to actually toggle the bool there.
        //
        // SOLUTION: We captured `toggled_section = Some(i)` during rendering,
        // but the sections and their `&mut bool` refs are already consumed.
        // To solve this properly, we extract the bools BEFORE consuming sections.
        // Let's handle this via the returned `toggled_section` — the caller toggles.

        // ── 13. Request repaint if animating ──
        if state.is_animating() {
            ui.ctx().request_repaint();
        }

        // ── 14. Store state ──
        ui.data_mut(|d| d.insert_temp(split_id, state));

        SplitResponse {
            response,
            id: split_id,
            double_clicked_divider,
            toggled_section,
        }
    }
}
