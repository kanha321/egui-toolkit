//! # egui-nav-stack
//!
//! App-owned back-stack screen navigation system for egui, modeled on the philosophy of Android Navigation 3.
//!
//! ## State Ownership Contract
//!
//! The back stack (`NavStack<K>`) and transition state (`NavTransition<K>`) are plain values owned
//! and stored by the consuming application across frames (e.g. inside an application state struct)
//! and passed by `&` / `&mut` reference (`CODING_RULES §2`).
//!
//! ## Safe Immediate-Mode Mutation
//!
//! [`NavDisplay::show`](crate::NavDisplay::show) reads `&NavStack<K>` and returns [`NavResponse<K>`]
//! containing any requested [`NavAction<K>`], enabling the caller to mutate `&mut NavStack<K>` cleanly
//! after rendering without borrow-checker conflicts (`CODING_RULES §3`).
//!
//! ## Examples
//!
//! ```rust
//! use egui_nav_stack::{NavAction, NavDisplay, NavStack};
//!
//! #[derive(Clone, PartialEq)]
//! enum AppScreen {
//!     Home,
//!     Profile,
//! }
//!
//! # egui::__run_test_ctx(|ctx| {
//! # egui::CentralPanel::default().show(ctx, |ui| {
//! let mut stack = NavStack::new(AppScreen::Home);
//!
//! NavDisplay::new(&stack)
//!     .show(ui, |screen, ui| {
//!         match screen {
//!             AppScreen::Home => {
//!                 if ui.button("Go to Profile").clicked() {
//!                     Some(NavAction::Push(AppScreen::Profile))
//!                 } else {
//!                     None
//!                 }
//!             }
//!             AppScreen::Profile => {
//!                 if ui.button("Back").clicked() {
//!                     Some(NavAction::Pop)
//!                 } else {
//!                     None
//!                 }
//!             }
//!         }
//!     })
//!     .apply_to(&mut stack);
//! # });
//! # });
//! ```

pub mod display;
pub mod stack;
pub mod transition;

pub use display::{NavAction, NavDisplay, NavResponse};
pub use stack::NavStack;

#[cfg(feature = "animated-transitions")]
pub use egui_spring::MotionPhysics;
#[cfg(feature = "animated-transitions")]
pub use transition::{NavTransition, SlideDirection, TransitionKind};
