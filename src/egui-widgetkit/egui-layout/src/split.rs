//! Declarative, responsive, constraint-aware nested split layouts for egui.

use egui::{Align, Layout, Pos2, Rect, Response, Sense, Ui, Vec2};
use crate::size::Size;

/// Layout axis for `Split`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    /// Sections placed horizontally side-by-side (left to right).
    Horizontal,
    /// Sections stacked vertically (top to bottom).
    Vertical,
}

struct Section<'a> {
    size: Size,
    add_contents: Box<dyn FnOnce(&mut Ui) + 'a>,
}

/// A declarative builder for responsive, constraint-aware multi-section split layouts.
///
/// Supports proportional fractions, fixed pixel sizes, minimum/maximum size constraints,
/// and automatic remainder expansion.
///
/// # State Ownership
///
/// `Split` is constructed and consumed each frame; it holds no global or static state (`CODING_RULES §2`).
///
/// # Examples
///
/// ```rust
/// use egui_layout::Split;
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
    spacing: f32,
    sections: Vec<Section<'a>>,
}

impl<'a> Split<'a> {
    /// Creates a horizontal split layout (columns placed left-to-right).
    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Horizontal,
            spacing: 4.0,
            sections: Vec::new(),
        }
    }

    /// Creates a vertical split layout (rows stacked top-to-bottom).
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Vertical,
            spacing: 4.0,
            sections: Vec::new(),
        }
    }

    /// Sets the inter-section spacing in logical points (default: `4.0`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px.max(0.0);
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
        self.sections.push(Section {
            size: Size::fraction(fraction),
            add_contents: Box::new(add_contents),
        });
        self
    }

    /// Adds a fractional section with a minimum size constraint in logical points.
    ///
    /// The section scales proportionally in large windows, but will never shrink below `min_px`.
    pub fn section_min(
        mut self,
        fraction: f32,
        min_px: f32,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section {
            size: Size::fraction(fraction).min_size(min_px),
            add_contents: Box::new(add_contents),
        });
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
        self.sections.push(Section {
            size: Size::fraction(fraction).min_size(min_px).max_size(max_px),
            add_contents: Box::new(add_contents),
        });
        self
    }

    /// Adds an exact fixed-size section in logical points (does not grow or shrink).
    pub fn section_fixed(
        mut self,
        px: f32,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section {
            size: Size::exact(px),
            add_contents: Box::new(add_contents),
        });
        self
    }

    /// Adds a section that expands to claim all remaining available space.
    pub fn section_remainder(
        mut self,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section {
            size: Size::remainder(),
            add_contents: Box::new(add_contents),
        });
        self
    }

    /// Adds a section with a custom `Size` policy.
    pub fn section_custom(
        mut self,
        size: Size,
        add_contents: impl FnOnce(&mut Ui) + 'a,
    ) -> Self {
        self.sections.push(Section {
            size,
            add_contents: Box::new(add_contents),
        });
        self
    }

    /// Calculates the minimum required length along the primary axis
    /// to satisfy all section constraints and inter-section spacing.
    pub fn min_primary_length(&self) -> f32 {
        let policies: Vec<Size> = self.sections.iter().map(|s| s.size).collect();
        Self::compute_min_length(self.spacing, &policies)
    }

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
    ///
    /// Maintained for backwards compatibility and pure fractional layouts.
    pub fn compute_sizes(available_length: f32, spacing: f32, fractions: &[f32]) -> Vec<f32> {
        let policies: Vec<Size> = fractions.iter().map(|&f| Size::fraction(f)).collect();
        Self::compute_sizes_with_policy(available_length, spacing, &policies)
    }

    /// Resolves and calculates exact section lengths using the constraint solver.
    ///
    /// Handles fixed sizes, fractional distributions, min/max bounds, graceful fallbacks,
    /// and sub-pixel float rounding remainder absorption.
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

    /// Renders the split layout, allocating sub-UIs for each section and consuming available space.
    pub fn show(self, ui: &mut Ui) -> Response {
        let n = self.sections.len();
        if n == 0 {
            return ui.allocate_response(Vec2::ZERO, Sense::hover());
        }

        let available = ui.available_size();
        let (primary_len, _cross_len) = match self.direction {
            Direction::Horizontal => (available.x, available.y),
            Direction::Vertical => (available.y, available.x),
        };

        let policies: Vec<Size> = self.sections.iter().map(|s| s.size).collect();
        let sizes = Self::compute_sizes_with_policy(primary_len, self.spacing, &policies);

        let (total_rect, response) = ui.allocate_exact_size(available, Sense::hover());

        match self.direction {
            Direction::Horizontal => {
                let mut current_x = total_rect.min.x;
                for (i, section) in self.sections.into_iter().enumerate() {
                    let w = sizes[i];
                    let section_rect = Rect::from_min_size(
                        Pos2::new(current_x, total_rect.min.y),
                        Vec2::new(w, total_rect.height()),
                    );

                    let mut child_ui = ui.child_ui(section_rect, Layout::top_down(Align::Min));
                    child_ui.set_clip_rect(child_ui.clip_rect().intersect(section_rect));
                    (section.add_contents)(&mut child_ui);

                    current_x += w + self.spacing;
                }
            }
            Direction::Vertical => {
                let mut current_y = total_rect.min.y;
                for (i, section) in self.sections.into_iter().enumerate() {
                    let h = sizes[i];
                    let section_rect = Rect::from_min_size(
                        Pos2::new(total_rect.min.x, current_y),
                        Vec2::new(total_rect.width(), h),
                    );

                    let mut child_ui = ui.child_ui(section_rect, Layout::top_down(Align::Min));
                    child_ui.set_clip_rect(child_ui.clip_rect().intersect(section_rect));
                    (section.add_contents)(&mut child_ui);

                    current_y += h + self.spacing;
                }
            }
        }

        response
    }
}
