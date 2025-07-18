#![allow(dead_code, clippy::too_many_arguments)]

mod client;
mod constants;
mod error;
mod models;

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(feature = "mcp")]
pub mod mcp;

pub mod prelude {
    pub use crate::{
        client::{AsyncOrpheus, Orpheus},
        models::{
            chat::{Message, Param, ParsingEngine, Plugin, Role, Tool, ToolCall},
            common::provider::{MaxPrice, Provider, ProviderPreferences, Quantization, Sort},
        },
    };
}
