//! # egui-layout
//!
//! Declarative, responsive, constraint-aware nested split layouts for [egui](https://github.com/emilk/egui).
//!
//! ## State Ownership Contract
//!
//! `Split` is an ephemeral builder struct constructed per-frame in immediate mode.
//! It does not hold, require, or create global or static mutable state (`CODING_RULES §2`).
//!
//! ## Quickstart
//!
//! ```rust
//! use egui_layout::{Split, Size};
//!
//! # egui::__run_test_ctx(|ctx| {
//! # egui::CentralPanel::default().show(ctx, |ui| {
//! Split::horizontal()
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

pub mod size;
pub mod split;

pub use size::Size;
pub use split::Split;
