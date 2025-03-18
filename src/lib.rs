#![allow(dead_code)]

mod blocking;
mod nonblocking;
mod types;

use pyo3::prelude::*;

pub const BASE_URL_ENV: &str = "ORPHEUS_BASE_URL";
pub const API_KEY_ENV: &str = "ORPHEUS_API_KEY";

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<blocking::Orpheus>()?;
    m.add_class::<nonblocking::AsyncOrpheus>()?;
    m.add_class::<types::message::Message>()?;
    m.add_class::<types::message::Conversation>()?;
    Ok(())
}
