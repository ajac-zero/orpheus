#![allow(dead_code, clippy::too_many_arguments)]
#![deny(clippy::mod_module_files, clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

/// Contains definition of request clients.
pub mod client {
    pub(crate) mod core;
    pub(crate) mod handler;
    pub(crate) mod mode;
    mod methods {
        mod chat;
        mod completion;
        mod keys;
    }

    pub use core::OrpheusCore;

    /// Alias for the OrpheusCore client in `Blocking` mode.
    pub type Orpheus = OrpheusCore<mode::Sync>;

    /// Alias for the OrpheusCore client in `Async` mode.
    pub type AsyncOrpheus = OrpheusCore<mode::Async>;
}

/// Types to be used for specialized request features.
pub mod models {
    pub mod chat;
    pub mod common;
    pub mod completion;
    pub mod keys;

    pub use chat::{
        Format, History, Message, Param, Parameter, ParsingEngine, Plugin, Tool, ToolCall,
    };
    pub use common::{
        DataCollection, Effort, MaxPrice, Preferences, Provider, Quantization, Reasoning, Sort,
        Transform, Usage,
    };
}
/// Single import with most commonly used types
pub mod prelude {
    pub use crate::{
        client::{AsyncOrpheus, Orpheus},
        models::{Format, Message, Param, Parameter, Tool, ToolCall},
    };
}

mod constants;
mod error;
mod integrations;

#[allow(unused_imports)]
pub use integrations::*;

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;
