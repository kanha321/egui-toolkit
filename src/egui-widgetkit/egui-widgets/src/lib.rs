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

pub mod badge;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod input;
pub mod progress;
pub mod slider;
pub mod switch;
pub mod tabs;

pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonSize, ButtonState, ButtonVariant};
pub use card::{Card, CardState};
pub use checkbox::{Checkbox, CheckboxState, RadioButton};
pub use input::{InputState, TextInput};
pub use progress::{ProgressBar, ProgressState, ProgressVariant};
pub use slider::{Slider, SliderState};
pub use switch::{Switch, SwitchSize, SwitchState};
pub use tabs::{SegmentedTabs, TabItem, TabsState};
