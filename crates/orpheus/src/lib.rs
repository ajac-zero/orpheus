#![allow(dead_code, clippy::too_many_arguments)]
#![deny(clippy::mod_module_files, clippy::unwrap_used)]
#![doc = include_str!("../../../README.md")]

/// Contains definition of request clients.
pub mod client {
    pub(crate) mod core;
    mod methods {
        mod respond;
    }

    pub use core::{AsyncOrpheus, Orpheus};
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
    pub use request::{AsyncResponseRequestBuilder, ResponseRequestBuilder};
    pub use stream::{AsyncResponseStream, ResponseEvent, ResponseStream};
    pub use tool::{Param, ParamType, Parameter, Tool, ToolFunctionBuilder};
}

/// Single import with most commonly used types
pub mod prelude {
    pub use crate::{
        client::{AsyncOrpheus, Orpheus},
        models::{Format, Input, Message, Param, ResponseEvent, ResponseExt, Tool},
    };
}

mod integrations;

#[allow(unused_imports)]
pub use integrations::*;

/// Re-export the open-responses types for direct access.
pub use open_responses as responses;

pub type Error = open_responses::client::Error;
pub type Result<T, E = Error> = core::result::Result<T, E>;
