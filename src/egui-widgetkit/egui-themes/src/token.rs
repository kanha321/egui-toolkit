//! Semantic color token slots for extensible UI theming.

/// Semantic color token definitions used across widgets and styling.
///
/// Tokens provide a high-level abstraction over raw RGB colors, allowing widgets to style
/// themselves consistently regardless of the active palette preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThemeToken {
    // ── Background / Structural Layers ──
    /// The primary background color of the application canvas / panels.
    Base,
    /// Darker / secondary background level for sidebars and toolbars.
    Mantle,
    /// Darkest / deepest background level for headers, status bars, and borders.
    Crust,
    /// Surface layer 0 for cards, raised containers, and inactive elements.
    Surface0,
    /// Surface layer 1 for hovered items, subtle borders, and secondary controls.
    Surface1,
    /// Surface layer 2 for active items, selected rows, and pressed controls.
    Surface2,
    /// Overlay layer 0 for muted decorative elements and outlines.
    Overlay0,
    /// Overlay layer 1 for subtle separators, scrollbars, and disabled text.
    Overlay1,
    /// Overlay layer 2 for secondary highlights and prominent borders.
    Overlay2,

    // ── Typography / Text Hierarchy ──
    /// High-contrast primary text and prominent headings.
    Text,
    /// Secondary body text, subtitles, and labels.
    Subtext0,
    /// Tertiary captions, timestamps, and placeholder text.
    Subtext1,

    // ── Semantic & Accent Roles ──
    /// Brand/primary accent color used for key interactive elements, focus highlights, and sliders.
    Accent,
    /// Positive / Success feedback indicator (e.g. green).
    Success,
    /// Caution / Warning feedback indicator (e.g. yellow/peach).
    Warning,
    /// Negative / Error / Danger feedback indicator (e.g. red).
    Danger,
    /// Informational notices and hyperlinks (e.g. blue).
    Info,
    /// Alternate secondary accent / info tone (e.g. teal/cyan).
    InfoAlt,
    /// Functional controls and system indicators (e.g. sapphire/lavender).
    SysControls,
}

impl ThemeToken {
    /// Returns the human-readable display name for this token.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Base => "Base (Canvas)",
            Self::Mantle => "Mantle (Sidebar)",
            Self::Crust => "Crust (Header/Border)",
            Self::Surface0 => "Surface 0 (Card)",
            Self::Surface1 => "Surface 1 (Hovered)",
            Self::Surface2 => "Surface 2 (Active)",
            Self::Overlay0 => "Overlay 0 (Muted)",
            Self::Overlay1 => "Overlay 1 (Border)",
            Self::Overlay2 => "Overlay 2 (Highlight)",
            Self::Text => "Text (Primary)",
            Self::Subtext0 => "Subtext 0 (Secondary)",
            Self::Subtext1 => "Subtext 1 (Caption)",
            Self::Accent => "Accent (Primary)",
            Self::Success => "Success (Green)",
            Self::Warning => "Warning (Yellow)",
            Self::Danger => "Danger (Red)",
            Self::Info => "Info (Blue)",
            Self::InfoAlt => "Info Alt (Teal)",
            Self::SysControls => "System Controls",
        }
    }

    /// Returns the category name for grouping in UI token inspectors.
    pub const fn category(&self) -> &'static str {
        match self {
            Self::Base | Self::Mantle | Self::Crust => "Background",
            Self::Surface0 | Self::Surface1 | Self::Surface2 => "Surfaces",
            Self::Overlay0 | Self::Overlay1 | Self::Overlay2 => "Overlays",
            Self::Text | Self::Subtext0 | Self::Subtext1 => "Typography",
            Self::Accent | Self::Success | Self::Warning | Self::Danger | Self::Info | Self::InfoAlt | Self::SysControls => "Semantic",
        }
    }

    /// Returns an array slice of all available tokens.
    pub const fn all() -> &'static [ThemeToken] {
        &[
            Self::Base,
            Self::Mantle,
            Self::Crust,
            Self::Surface0,
            Self::Surface1,
            Self::Surface2,
            Self::Overlay0,
            Self::Overlay1,
            Self::Overlay2,
            Self::Text,
            Self::Subtext0,
            Self::Subtext1,
            Self::Accent,
            Self::Success,
            Self::Warning,
            Self::Danger,
            Self::Info,
            Self::InfoAlt,
            Self::SysControls,
        ]
    }
}
