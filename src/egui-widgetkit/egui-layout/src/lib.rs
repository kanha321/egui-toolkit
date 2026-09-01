//! # egui-layout
//!
//! Declarative, responsive, constraint-aware nested split layouts for [egui](https://github.com/emilk/egui).
//!
//! Supports interactive resize dividers, collapsible sections with spring-animated
//! transitions, and visual card framing — all with zero boilerplate state management.
//!
//! ## State Ownership Contract
//!
//! `Split` is an ephemeral builder struct constructed per-frame in immediate mode.
//! Interactive state (`SplitState`) is auto-persisted in `ui.data()` via `egui::Id`,
//! following the same pattern as `egui::CollapsingHeader` (`CODING_RULES §2`).
//! Collapse expanded/collapsed booleans are owned by the consuming application.
//!
//! ## Quickstart
//!
//! ```rust
//! use egui_layout::{Split, Size, SplitState, CollapseMode};
//!
//! # egui::__run_test_ctx(|ctx| {
//! # egui::CentralPanel::default().show(ctx, |ui| {
//! Split::horizontal()
//!     .resizable(true)
//!     .section(0.33, |ui| {
//!         ui.label("Sidebar (33%)");
//!     })
//!     .section_min(0.67, 100.0, |ui| {
//!         ui.label("Content (Flexible with min 100px)");
//!     })
//!     .show(ui);
//! # });
//! # });
//! ```

pub mod collapse;
pub mod resize;
pub mod section;
pub mod size;
pub mod split;
pub mod style;

pub use collapse::{CollapseConfig, CollapseMode};
pub use resize::SplitState;
pub use section::Section;
pub use size::Size;
pub use split::{Split, SplitResponse};
pub use style::{DividerVisibility, SplitStyle};
