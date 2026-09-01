//! Persistent interactive state for resizable and collapsible `Split` layouts.

use spring_core::{Spring, SpringParams};

/// Persistent interactive state for a [`Split`](crate::Split) layout.
///
/// Auto-persisted in `ui.data()` keyed by [`egui::Id`]. The consuming app does not
/// need to create or manage this directly — `Split::show()` handles it automatically.
///
/// For programmatic access, use [`SplitState::load()`] / [`.store()`]:
///
/// ```rust
/// # let ui: &mut egui::Ui = return;
/// # let split_id = egui::Id::new("my_split");
/// use egui_layout::SplitState;
///
/// // After show():
/// let mut state = SplitState::load(ui, split_id);
/// state.reset();           // spring-animate all sections back to declared sizes
/// state.store(ui, split_id);
/// ```
///
/// # State Ownership
///
/// Multiple `SplitState` values coexist naturally (one per `Split`, each with
/// its own `Id`). No global or static state (`CODING_RULES §2`). State is
/// owned by egui's per-context temp storage, following the same pattern as
/// `egui::CollapsingHeader` and `egui::Window`.
#[derive(Clone, Debug)]
pub struct SplitState {
    /// Per-section animation springs driving the rendered size.
    /// `spring.target` = desired size, `spring.current` = animated rendered size.
    pub springs: Vec<Spring>,

    /// Per-section size overrides from drag interactions.
    /// `None` = use declared policy. `Some(px)` = user-dragged to this size.
    pub overrides: Vec<Option<f32>>,

    /// Which divider (by index) is currently being dragged, if any.
    /// The index refers to the divider AFTER section[index].
    pub active_divider: Option<usize>,

    /// Section count this state was last synced with.
    pub section_count: usize,

    /// Last known expanded size for each section (used to lock layout width during collapse animations).
    pub expanded_sizes: Vec<f32>,

    /// Whether this state has been initialized with actual sizes (false on first frame).
    pub initialized: bool,
}

impl Default for SplitState {
    fn default() -> Self {
        Self {
            springs: Vec::new(),
            overrides: Vec::new(),
            active_divider: None,
            section_count: 0,
            expanded_sizes: Vec::new(),
            initialized: false,
        }
    }
}

impl SplitState {
    /// Creates a new empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads this split's state from egui's temp storage, or creates a default.
    pub fn load(ui: &egui::Ui, id: egui::Id) -> Self {
        ui.data(|d| d.get_temp::<Self>(id)).unwrap_or_default()
    }

    /// Loads this split's state from an egui `Context` directly.
    pub fn load_ctx(ctx: &egui::Context, id: egui::Id) -> Self {
        ctx.data(|d| d.get_temp::<Self>(id)).unwrap_or_default()
    }

    /// Stores this split's state back into egui's temp storage.
    pub fn store(self, ui: &egui::Ui, id: egui::Id) {
        ui.data_mut(|d| d.insert_temp(id, self));
    }

    /// Stores this split's state into an egui `Context` directly.
    pub fn store_ctx(self, ctx: &egui::Context, id: egui::Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    /// Ensures the internal vectors match the given section count.
    /// New springs are initialized at `initial_size` with the given params.
    pub(crate) fn sync_to_count(
        &mut self,
        count: usize,
        initial_sizes: &[f32],
        params: SpringParams,
    ) {
        if self.section_count == count && self.springs.len() == count {
            return;
        }

        self.springs.resize_with(count, || {
            Spring::new(0.0, params)
        });
        self.overrides.resize(count, None);
        self.expanded_sizes.resize(count, 0.0);

        // Initialize springs to actual sizes on first sync
        if !self.initialized && !initial_sizes.is_empty() {
            for (i, spring) in self.springs.iter_mut().enumerate() {
                if let Some(&size) = initial_sizes.get(i) {
                    spring.reset(size);
                    self.expanded_sizes[i] = size;
                }
            }
            self.initialized = true;
        }

        self.section_count = count;
    }

    /// Resets ALL user drag overrides — sections will spring-animate back
    /// to their declared fractions.
    pub fn reset(&mut self) {
        for ov in &mut self.overrides {
            *ov = None;
        }
    }

    /// Resets a single section's drag override.
    pub fn reset_section(&mut self, index: usize) {
        if let Some(ov) = self.overrides.get_mut(index) {
            *ov = None;
        }
    }

    /// Returns the current animated size of section `index`, if available.
    pub fn current_size(&self, index: usize) -> Option<f32> {
        self.springs.get(index).map(|s| s.current)
    }

    /// Returns `true` if any spring is still animating (not settled).
    pub fn is_animating(&self) -> bool {
        self.springs.iter().any(|s| !s.is_settled())
    }

    /// Teleports a section's spring to a specific size with zero velocity.
    /// Used during drag to make sections follow the mouse directly.
    pub(crate) fn teleport(&mut self, index: usize, size: f32) {
        if let Some(spring) = self.springs.get_mut(index) {
            spring.reset(size);
            spring.set_target(size);
        }
        if let Some(ov) = self.overrides.get_mut(index) {
            *ov = Some(size);
        }
    }

    /// Sets the spring target for a section without teleporting.
    /// The spring will animate from current to target.
    pub(crate) fn set_target(&mut self, index: usize, target: f32) {
        if let Some(spring) = self.springs.get_mut(index) {
            spring.set_target(target);
        }
    }

    /// Advances all springs by `dt` seconds.
    pub(crate) fn advance(&mut self, dt: f32) {
        for spring in &mut self.springs {
            spring.update(dt);
        }
    }
}
