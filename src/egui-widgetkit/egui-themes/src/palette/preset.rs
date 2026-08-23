//! Curated theme preset enum definition.

use crate::palette::ThemePalette;
use crate::palette::{
    catppuccin, dracula, gruvbox, material_ocean, monokai_pro, night_owl, one_dark_pro, rose_pine,
    tokyo_night,
};

/// Curated theme palette preset identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ThemePreset {
    /// Catppuccin Mocha (darkest pastel soothing theme).
    #[default]
    CatppuccinMocha,
    /// Catppuccin Macchiato (medium dark pastel theme).
    CatppuccinMacchiato,
    /// Catppuccin Frappé (muted dark pastel theme).
    CatppuccinFrappe,
    /// Catppuccin Latte (crisp light pastel theme).
    CatppuccinLatte,
    /// Tokyo Night (clean, neon dark cyber theme).
    TokyoNight,
    /// One Dark Pro (classic Atom/VSCode editor theme).
    OneDarkPro,
    /// Dracula (vibrant dark gothic theme).
    Dracula,
    /// Monokai Pro (warm, high-contrast dark theme).
    MonokaiPro,
    /// Night Owl (deep blue nighttime coding theme).
    NightOwl,
    /// Material Ocean (deep oceanic material theme).
    MaterialOcean,
    /// Rosé Pine (soho vibes warm aesthetic theme).
    RosePine,
    /// Gruvbox (retro groove warm earthy theme).
    Gruvbox,
}

impl ThemePreset {
    /// Returns the human-readable display name for this preset.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::TokyoNight => "Tokyo Night",
            Self::OneDarkPro => "One Dark Pro",
            Self::Dracula => "Dracula",
            Self::MonokaiPro => "Monokai Pro",
            Self::NightOwl => "Night Owl",
            Self::MaterialOcean => "Material Ocean",
            Self::RosePine => "Rosé Pine",
            Self::Gruvbox => "Gruvbox",
        }
    }

    /// Returns `true` if this preset is a dark theme.
    pub const fn is_dark(&self) -> bool {
        !matches!(self, Self::CatppuccinLatte)
    }

    /// Constructs the concrete [`ThemePalette`] corresponding to this preset.
    pub fn palette(&self) -> ThemePalette {
        match self {
            Self::CatppuccinMocha => catppuccin::mocha(),
            Self::CatppuccinMacchiato => catppuccin::macchiato(),
            Self::CatppuccinFrappe => catppuccin::frappe(),
            Self::CatppuccinLatte => catppuccin::latte(),
            Self::TokyoNight => tokyo_night::tokyo_night(),
            Self::OneDarkPro => one_dark_pro::one_dark_pro(),
            Self::Dracula => dracula::dracula(),
            Self::MonokaiPro => monokai_pro::monokai_pro(),
            Self::NightOwl => night_owl::night_owl(),
            Self::MaterialOcean => material_ocean::material_ocean(),
            Self::RosePine => rose_pine::rose_pine(),
            Self::Gruvbox => gruvbox::gruvbox(),
        }
    }

    /// Returns an array slice of all 12 curated presets.
    pub const fn all() -> &'static [ThemePreset] {
        &[
            Self::CatppuccinMocha,
            Self::CatppuccinMacchiato,
            Self::CatppuccinFrappe,
            Self::CatppuccinLatte,
            Self::TokyoNight,
            Self::OneDarkPro,
            Self::Dracula,
            Self::MonokaiPro,
            Self::NightOwl,
            Self::MaterialOcean,
            Self::RosePine,
            Self::Gruvbox,
        ]
    }
}
