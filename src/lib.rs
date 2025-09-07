#![allow(dead_code, clippy::too_many_arguments)]
#![doc = include_str!("../README.md")]

pub mod client;
mod constants;
mod error;
mod integrations;
pub mod models;

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[allow(unused_imports)]
pub use integrations::*;

pub mod prelude {
    pub use crate::{
        client::{AsyncOrpheus, Orpheus},
        models::{Format, Message, Param, Parameter, Tool, ToolCall},
    };
}
