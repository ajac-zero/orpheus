#![allow(dead_code, clippy::too_many_arguments)]
#![deny(clippy::mod_module_files, clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

/// Contains definition of request clients.
pub mod client {
    pub(crate) mod core;
    pub(crate) mod mode;
    mod methods {
        mod respond;
    }

    pub use core::OrpheusCore;

    /// Alias for the OrpheusCore client in `Blocking` mode.
    pub type Orpheus = OrpheusCore<mode::Sync>;

    /// Alias for the OrpheusCore client in `Async` mode.
    pub type AsyncOrpheus = OrpheusCore<mode::Async>;
}

/// Types for building requests and handling responses.
pub mod models {
    pub mod ext;
    pub mod format;
    pub mod input;
    pub mod message;
    pub mod request;
    pub mod stream;
    pub mod tool;

    pub use ext::ResponseExt;
    pub use format::Format;
    pub use input::Input;
    pub use message::Message;
    pub use request::ResponseRequestBuilder;
    pub use stream::{ResponseEvent, ResponseStream};
    pub use tool::{Param, ParamType, Parameter, Tool, ToolFunctionBuilder};
}

/// Single import with most commonly used types
pub mod prelude {
    pub use crate::{
        client::{AsyncOrpheus, Orpheus},
        models::{Format, Input, Message, Param, ResponseExt, ResponseEvent, Tool},
    };
}

mod constants;
mod error;
mod integrations;

#[allow(unused_imports)]
pub use integrations::*;

/// Re-export the open-responses types for direct access.
pub use open_responses as responses;

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;
