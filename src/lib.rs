#![allow(dead_code, clippy::too_many_arguments)]

mod client;
mod constants;
mod error;
mod models;

pub use client::{AsyncOrpheus, Orpheus};
pub use models::*;

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(feature = "mcp")]
pub mod mcp;
