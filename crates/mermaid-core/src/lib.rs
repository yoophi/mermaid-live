//! What the desktop app and the terminal CLI must agree on.
//!
//! Chart detection, the raster size contract, and the render wire protocol live
//! here so neither side can drift from the other.

pub mod chart;
pub mod protocol;
pub mod raster;
