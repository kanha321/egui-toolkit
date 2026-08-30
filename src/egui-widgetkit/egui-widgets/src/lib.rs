//! # egui-widgets
//!
//! An app-agnostic, token-aware, spring-animated UI component library for [`egui`].
//!
//! `egui-widgets` acts as a polished design-system component layer between stock egui primitives
//! and consuming applications.
//!
//! ## Core Guarantees
//!
//! - **Palette-Driven (Single Source of Truth)**: Every widget automatically derives its fills,
//!   strokes, text colors, and states from [`ThemePalette`](egui_themes::ThemePalette) or [`egui::Visuals`].
//! - **Spring Motion Physics**: Interactive micro-interactions (press bounce, elastic thumb travel,
//!   sliding pills, hover elevation, focus glow rings) are powered by continuous closed-form ODE
//!   physics from [`spring-core`].
//! - **Complete Customization**: Every visual parameter (colors, strokes, roundings, paddings,
//!   spring physics parameters, motion toggles) is exposed via fluent builder methods.
//! - **Strict State Ownership**: No global or hidden static state. Components support zero-boilerplate
//!   immediate mode (with ID-scoped memory) or explicit caller-owned state structs (`CODING_RULES §2`).
//! - **`egui::Shape` Only**: Zero custom GPU shaders or raw draw callbacks (`CODING_RULES §4`).
//! - **Continuous Repaint**: Animated components automatically request repaints each frame while moving
//!   and consume 0% idle CPU once settled.
//!
//! ## Component Inventory
//!
//! - [`Button`], [`ButtonVariant`], [`ButtonSize`], [`ButtonState`]: Animated buttons with press bounce and hover luminance glide.
//! - [`Switch`], [`SwitchSize`], [`SwitchState`]: Fluid spring toggle switches.
//! - [`SegmentedTabs`], [`TabItem`], [`TabsState`]: Continuous sliding pill tab switcher.
//! - [`ProgressBar`], [`ProgressVariant`], [`ProgressState`]: Smooth progress meters with physical catchup.
//! - [`Slider`], [`SliderState`]: Interactive sliders with spring-scaling knob thumbs.
//! - [`Checkbox`], [`CheckboxState`], [`RadioButton`]: Animated checkmarks and radio dots with overshoot bounce.
//! - [`Card`], [`CardState`]: Container surfaces with optional spring hover lift.
//! - [`Badge`], [`BadgeVariant`]: Semantic status badges and tag indicators.
//! - [`TextInput`], [`InputState`]: Text inputs with animated spring focus glow rings.
//! - [`Dropdown`], [`DropdownOption`], [`DropdownResponse`], [`DropdownState`]: Spring-animated dropdown menus.

pub mod badge;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod dropdown;
pub mod input;
pub mod progress;
pub mod slider;
pub mod switch;
pub mod tabs;

pub use badge::{Badge, BadgeState, BadgeStyle, BadgeVariant};
pub use button::{Button, ButtonResponse, ButtonSize, ButtonState, ButtonVariant};
pub use card::{Card, CardState};
pub use checkbox::{Checkbox, CheckboxResponse, CheckboxState, RadioButton, RadioResponse, RadioState};
pub use dropdown::{Dropdown, DropdownItemContext, DropdownOption, DropdownResponse, DropdownState};
pub use input::{
    lerp_color, resolve_vim_mode_color, DragSelectionState, DropFlightAnim, InputState, TextAlign,
    TextInput,
};
pub use progress::{ProgressBar, ProgressState, ProgressVariant};
pub use slider::{Slider, SliderLayout, SliderState};
pub use switch::{Switch, SwitchResponse, SwitchSize, SwitchState};
pub use tabs::{SegmentedTabs, TabItem, TabsResponse, TabsState};

/// Returns whether pointer hover should be rendered visually.
///
/// Returns `false` if cursor autohide has hidden the cursor (e.g. during keyboard navigation),
/// preventing ghost hover highlights on widgets when scrolling or moving focus.
#[inline]
pub(crate) fn is_hover_active(ctx: &egui::Context) -> bool {
    !ctx.data(|d| d.get_temp::<bool>(egui::Id::new("_egui_cursor_autohide_hidden"))).unwrap_or(false)
}
