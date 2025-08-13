#![allow(dead_code, clippy::too_many_arguments)]

mod client;
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
        models::{
            chat::{Message, Param, ParsingEngine, Plugin, Role, Tool, ToolCall},
            common::{
                provider::{MaxPrice, Provider, Quantization, Sort},
                reasoning::Effort,
            },
        },
    };
}
