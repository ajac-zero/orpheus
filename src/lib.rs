#![allow(dead_code, clippy::too_many_arguments)]
#![deny(clippy::mod_module_files, clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

/// Contains definition of request clients.
pub mod client;
mod constants;
mod error;
mod integrations;
/// Types to be used for specialized request features.
pub mod models;
/// Single import with most commonly used types
pub mod prelude;
pub use integrations::*;

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;
