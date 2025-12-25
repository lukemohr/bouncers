//! Core geometry and dynamics for 2D billiard systems.
//!
//! This crate should remain pure: no I/O, networking, or database logic.

pub mod dynamics;
pub mod geometry;

pub use geometry::table_spec::{PolylineSpec, TableSpec};
