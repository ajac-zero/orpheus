#![allow(dead_code, clippy::too_many_arguments)]

mod blocking;
mod constants;
mod models;
mod nonblocking;
mod types;
mod utils;

use pyo3::prelude::*;

#[pymodule]
fn orpheus_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<blocking::OrpheusCore>()?;
    m.add_class::<nonblocking::AsyncOrpheusCore>()?;
    m.add_class::<models::chat::message::Message>()?;
    m.add_class::<models::chat::ChatCompletion>()?;
    m.add_class::<models::embed::EmbeddingResponse>()?;
    Ok(())
}
