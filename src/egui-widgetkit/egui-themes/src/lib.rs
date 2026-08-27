//! Token-based theming, curated palette presets, smooth morph transitions, and live preview for egui.
//!
//! # State Ownership
//!
//! In accordance with `CODING_RULES §2`, `egui-themes` contains **no global or static mutable state**.
//! All theme state ([`ThemeState`], [`ThemePalette`]) is plain, value-based data owned directly by
//! the application and passed by reference.
//!
//! # Architecture
//!
//! - **Single Source of Truth**: [`ThemePalette`] defines all semantic UI colors (backgrounds, surfaces, typography, accents, and swatches).
//! - **12 Curated Presets**: [`ThemePreset`] offers Catppuccin (4 flavors), Tokyo Night, Dracula, Monokai Pro, Gruvbox, One Dark Pro, Rosé Pine, Night Owl, and Material Ocean.
//! - **Smooth Theme Morphing**: [`ThemeState::update`] smoothly lerps between palettes over time with continuous frame repainting.
//! - **Native Egui Synchronization**: [`ThemeState::apply_to_ctx`] translates the active palette directly to `egui::Visuals`.
//! - **Interactive Components**: [`ThemePreview`] provides a live scaled mock app card; [`ThemeGallery`] provides a responsive preset card grid.
//!
//! # Example
//!
//! ```rust
//! use egui_themes::{ThemePreset, ThemeState, ThemePreview, ThemeGallery};
//!
//! // In app state:
//! let mut theme = ThemeState::new(ThemePreset::CatppuccinMocha);
//!
//! // In update loop:
//! # egui::__run_test_ctx(|ctx| {
//! theme.update(0.016, ctx);
//! theme.apply_to_ctx(ctx);
//!
//! # egui::CentralPanel::default().show(ctx, |ui| {
//! // Render live preview:
//! ThemePreview::new().show(ui, &theme.current);
//!
//! // Render preset selection grid:
//! if let Some(new_preset) = ThemeGallery::new().show(ui, theme.active_preset) {
//!     theme.set_preset(new_preset);
//! }
//! # });
//! # });
//! ```

pub mod color_utils;
pub mod gallery;
pub mod palette;
pub mod preview;
pub mod switcher;
pub mod token;

pub use color_utils::lerp_color;
pub use gallery::ThemeGallery;
pub use palette::{ThemePalette, ThemePreset};
pub use preview::ThemePreview;
pub use switcher::ThemeState;
pub use token::ThemeToken;
