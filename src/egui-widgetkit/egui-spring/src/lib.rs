//! # egui-spring
//!
//! Spring-animated selection highlights and Bézier border deformation for egui.
//!
//! ## State Ownership Contract
//!
//! `SpringRect` and `CornerSprings` are plain value structs owned by the consuming
//! application across frames (`CODING_RULES §2`).
//!
//! ## Rendering Standards
//!
//! - **`egui::Shape` Only**: All visual output is emitted via `egui::Shape` primitives (`CODING_RULES §4`).
//! - **Continuous Motion Repaint**: Calls `ctx.request_repaint()` every frame while moving.
//! - **Corner Rounding Invariant**: The 4 rounded corners stay intact across all window sizes.

pub mod bezier;
pub mod config;
pub mod corner_springs;
pub mod group;
pub mod spring_rect;

pub use bezier::{build_bezier_boundary, BEZIER_KAPPA};
pub use config::HighlightConfig;
pub use corner_springs::{CornerSprings, SpringPoint};
pub use group::HighlightGroup;
pub use spring_core::{MotionPhysics, SpringParams};
pub use spring_rect::SpringRect;
