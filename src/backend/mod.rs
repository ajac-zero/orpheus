mod open_responses;
mod traits;

#[cfg(feature = "gemini")]
pub mod gemini;

pub use open_responses::{AsyncOpenResponsesBackend, OpenResponsesBackend};
pub use traits::{
    AsyncRequestBuilder, Backend, Mode, RequestBuilder, Response, StreamResponse,
    SyncRequestBuilder,
};

#[cfg(feature = "gemini")]
pub use gemini::GeminiBackend;
