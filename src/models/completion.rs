mod request;
mod response;

pub(crate) use request::CompletionRequest;
pub use request::CompletionRequestBuilder;
pub use response::{CompletionChoice, CompletionResponse};
