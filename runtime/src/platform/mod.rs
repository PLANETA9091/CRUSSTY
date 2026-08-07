#![allow(unused_imports)] // Re-exports below are for downstream modules.

//! crussty platform primitives.
//!
//! Ten bricks every module builds on. Each file is owned by exactly one
//! implementation agent; files must NOT depend on each other's internals —
//! the public API of every brick lives in this `mod.rs` so cross-brick use
//! goes through the stable surface below.

pub mod barriers;
pub mod events;
pub mod hot_reload;
pub mod network;
pub mod save_events;
pub mod scheduler;
pub mod side_table;
pub mod signals;
pub mod storage;
pub mod telemetry;
pub mod threads;
pub mod transform;

pub use barriers::*;
pub use events::*;
pub use hot_reload::*;
pub use network::*;
pub use save_events::*;
pub use scheduler::*;
pub use side_table::*;
pub use signals::*;
pub use storage::*;
pub use telemetry::*;
pub use threads::*;
pub use transform::*;
