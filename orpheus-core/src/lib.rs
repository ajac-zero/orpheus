#![allow(dead_code, clippy::too_many_arguments)]

mod blocking;
mod nonblocking;
mod types;

use pyo3::prelude::*;

pub const BASE_URL_ENVS: [&str; 2] = ["ORPHEUS_BASE_URL", "OPENAI_BASE_URL"];
pub const API_KEY_ENVS: [&str; 2] = ["ORPHEUS_API_KEY", "OPENAI_API_KEY"];

#[pymodule]
fn orpheus_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<blocking::OrpheusCore>()?;
    m.add_class::<nonblocking::AsyncOrpheusCore>()?;
    m.add_class::<types::chat::message::Message>()?;
    m.add_class::<types::chat::ChatCompletion>()?;
    m.add_class::<types::embed::EmbeddingResponse>()?;
    Ok(())
}
