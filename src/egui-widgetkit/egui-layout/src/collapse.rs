//! Collapse mode and configuration types for collapsible layout sections.

/// How a section should render when in its collapsed state.
///
/// Used with [`Section::collapsible()`](crate::Section::collapsible) to declare
/// the collapse behavior for a section.
///
/// # Examples
///
/// ```rust
/// use egui_layout::CollapseMode;
///
/// // Sidebar collapses to a 40px icon rail
/// let mode = CollapseMode::FixedBar(40.0);
///
/// // Panel disappears completely
/// let mode = CollapseMode::Hidden;
///
/// // Card shows only its header row
/// let mode = CollapseMode::HeaderOnly;
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CollapseMode {
    /// Collapses to zero size — completely hidden, no space consumed.
    /// Spacing gaps adjacent to this section are also suppressed.
    Hidden,

    /// Collapses to a fixed size in logical points along the primary axis.
    /// Ideal for icon rails, toolbar strips, or mini-previews.
    ///
    /// - In a horizontal split → collapsed **width** in px.
    /// - In a vertical split   → collapsed **height** in px.
    FixedBar(f32),

    /// Collapses to show only the card header (title + chevron toggle).
    /// Content closure is not called. Only meaningful for sections
    /// with `.card()` / `.title()` enabled.
    HeaderOnly,
}

/// Configuration bundle for a collapsible section.
///
/// Created internally by [`Section::collapsible()`](crate::Section::collapsible)
/// and tunable via follow-up builder methods like
/// [`Section::no_chevron()`](crate::Section::no_chevron).
#[derive(Clone, Debug)]
pub struct CollapseConfig {
    /// What the section looks like when collapsed.
    pub mode: CollapseMode,

    /// If `true`, render a clickable chevron toggle in the card header.
    /// Default: `true` (only rendered when the section is also a card).
    pub show_chevron: bool,
}

impl CollapseConfig {
    /// Creates a new collapse configuration with the given mode and chevron enabled.
    pub fn new(mode: CollapseMode) -> Self {
        Self {
            mode,
            show_chevron: true,
        }
    }
}
