//! # spring-core
//!
//! Pure math, analytical closed-form 1D spring physics solver with zero GUI dependencies.
//!
//! ## State Ownership Contract
//!
//! `Spring` is a plain value struct owned and stored by the consuming application across frames.
//! There is no global, singleton, or hidden static state (`CODING_RULES §2`).
//!
//! ## Quickstart
//!
//! ```rust
//! use spring_core::{Spring, SpringParams};
//!
//! let mut spring = Spring::new(0.0, SpringParams::snappy());
//! spring.set_target(100.0);
//!
//! while !spring.is_settled() {
//!     spring.update(1.0 / 60.0); // 60 FPS frame delta
//! }
//!
//! assert_eq!(spring.value(), 100.0);
//! ```

pub mod params;
pub mod solver;
pub mod spring;

pub use params::{MotionPhysics, SpringParams};
pub use solver::{solve_step, SolverState};
pub use spring::Spring;
