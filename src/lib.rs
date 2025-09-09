#![allow(dead_code, clippy::too_many_arguments)]
#![doc = include_str!("../README.md")]

/// Contains definition of request clients.
pub mod client;
mod constants;
mod error;
mod integrations;
/// Types to be used for specialized request features.
pub mod models;

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[allow(unused_imports)]
pub use integrations::*;

/// Single import with most commonly used types
pub mod prelude {
    pub use crate::{
        client::{AsyncOrpheus, Orpheus},
        models::{Format, Message, Param, Parameter, Tool, ToolCall},
    };
}
