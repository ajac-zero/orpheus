#![allow(dead_code, clippy::too_many_arguments)]
#![doc = include_str!("../README.md")]

/// Contains definition of request clients.
pub mod client {
    mod core;
    mod handler;
    mod mode;
    mod methods {
        mod chat;
        mod completion;
        mod keys;
    }

    pub use core::OrpheusCore;

    pub(crate) use handler::{AsyncExecutor, Executor, Handler};
    pub(crate) use mode::{Async, Mode, Sync};

    /// Alias for the OrpheusCore client in `Blocking` mode.
    pub type Orpheus = OrpheusCore<Sync>;

    /// Alias for the OrpheusCore client in `Async` mode.
    pub type AsyncOrpheus = OrpheusCore<Async>;
}
mod constants;
mod error;

/// Types to be used for specialized request features.
pub mod models {
    pub mod chat {
        mod handler;
        mod request {
            pub mod body;
            pub mod content;
            pub mod message;
            pub mod plugins;
            pub mod structured;
            pub mod tool;
        }
        mod response {
            pub mod completion;
            pub mod stream {
                pub mod blocking;
                pub mod common;
                pub mod nonblocking;
            }
            pub mod usage;
        }

        pub(crate) use handler::ChatHandler;
        pub(crate) use request::body::ChatRequest;
        pub use request::{
            body::ChatRequestBuilder,
            content::{CacheControl, Content, Part},
            message::{History, Message, Role, ToolCall},
            plugins::{ParsingEngine, Plugin},
            structured::Format,
            tool::{Param, ParamType, Parameter, Tool, ToolFunctionBuilder},
        };
        pub use response::{
            completion::{ChatChoice, ChatCompletion},
            stream::{
                blocking::ChatStream,
                common::{ChatStreamChoice, ChatStreamChunk},
                nonblocking::AsyncStream,
            },
            usage::ChatUsage,
        };
    }
    pub mod common {
        mod provider;
        mod reasoning;
        mod transforms;
        mod usage;

        pub use provider::{DataCollection, MaxPrice, Preferences, Provider, Quantization, Sort};
        pub(crate) use provider::{PreferencesBuilder, preferences_builder};
        pub use reasoning::{Effort, Reasoning};
        pub(crate) use reasoning::{ReasoningBuilder, reasoning_builder};
        pub use transforms::Transform;
        pub use usage::Usage;
    }
    pub mod completion {
        mod handler;
        mod request;
        mod response;

        pub(crate) use handler::CompletionHandler;
        pub(crate) use request::CompletionRequest;
        pub use request::CompletionRequestBuilder;
        pub use response::{CompletionChoice, CompletionResponse};
    }
    pub mod keys {
        mod handler;
        mod request;
        mod response;

        pub(crate) use handler::ProvisionHandler;
        pub(crate) use request::KeyProvisioningRequest;
        pub use request::KeyProvisioningRequestBuilder;
        pub use response::{ApiKey, CreateKeyResult, DeleteKeyResult, ListKeysResult};
    }

    pub use chat::{
        Format, History, Message, Param, Parameter, ParsingEngine, Plugin, Tool, ToolCall,
    };
    pub use common::{
        DataCollection, Effort, MaxPrice, Preferences, Provider, Quantization, Reasoning, Sort,
        Transform, Usage,
    };
}

pub type Error = error::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;

mod integrations {
    #[cfg(feature = "langfuse")]
    pub mod langfuse;

    #[cfg(feature = "mcp")]
    pub mod mcp {
        mod context;
        mod tools;

        pub use context::Mcp;
    }

    #[cfg(feature = "otel")]
    pub mod otel;
}
#[allow(unused_imports)]
pub use integrations::*;

/// Single import with most commonly used types
pub mod prelude {
    pub use crate::{
        client::{AsyncOrpheus, Orpheus},
        models::{Format, Message, Param, Parameter, Tool, ToolCall},
    };
}
