//! Sizing policies and constraints for layout sections.

/// Sizing policy for a section in a split layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Size {
    /// Takes a proportional fraction of available flexible space, with optional min/max bounds.
    Fraction {
        /// Proportional weight relative to other fractional sections.
        fraction: f32,
        /// Optional minimum size in logical points.
        min: Option<f32>,
        /// Optional maximum size in logical points.
        max: Option<f32>,
    },
    /// Takes an exact fixed size in logical points (does not grow or shrink).
    Exact(f32),
    /// Takes all remaining available space after fixed and fractional minimums are satisfied.
    Remainder {
        /// Optional minimum size in logical points.
        min: Option<f32>,
    },
}

impl Default for Size {
    fn default() -> Self {
        Self::Fraction {
            fraction: 1.0,
            min: None,
            max: None,
        }
    }
}

impl Size {
    /// Creates a fractional sizing policy.
    pub fn fraction(fraction: f32) -> Self {
        Self::Fraction {
            fraction: fraction.max(0.0),
            min: None,
            max: None,
        }
    }

    /// Creates an exact fixed pixel sizing policy.
    pub fn exact(px: f32) -> Self {
        Self::Exact(px.max(0.0))
    }

    /// Creates a remainder sizing policy that claims all remaining space.
    pub fn remainder() -> Self {
        Self::Remainder { min: None }
    }

    /// Adds a minimum size constraint in logical points.
    pub fn min_size(self, min_px: f32) -> Self {
        match self {
            Self::Fraction { fraction, max, .. } => Self::Fraction {
                fraction,
                min: Some(min_px.max(0.0)),
                max,
            },
            Self::Exact(px) => Self::Exact(px.max(min_px)),
            Self::Remainder { .. } => Self::Remainder {
                min: Some(min_px.max(0.0)),
            },
        }
    }

    /// Adds a maximum size constraint in logical points.
    pub fn max_size(self, max_px: f32) -> Self {
        match self {
            Self::Fraction { fraction, min, .. } => Self::Fraction {
                fraction,
                min,
                max: Some(max_px.max(0.0)),
            },
            Self::Exact(px) => Self::Exact(px.min(max_px.max(0.0))),
            Self::Remainder { min } => Self::Remainder { min },
        }
    }
}
