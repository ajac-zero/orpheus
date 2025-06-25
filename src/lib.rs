#![allow(dead_code, clippy::too_many_arguments)]

mod client;
mod constants;
mod exceptions;
pub mod models;

pub use client::{AsyncOrpheus, Orpheus};

pub type Error = exceptions::OrpheusError;
pub type Result<T, E = Error> = core::result::Result<T, E>;

// #[pymodule]
// fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_class::<models::chat::Message>()?;
//     m.add_class::<models::chat::ChatCompletion>()?;
//     m.add_class::<models::chat::ToolCall>()?;
//     m.add_class::<models::chat::Part>()?;
//     Ok(())
// }
