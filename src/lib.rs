#![allow(dead_code)]

mod client;
mod types;

use pyo3::prelude::*;

#[pymodule]
fn orpheus(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<client::Orpheus>()?;
    m.add_class::<client::AsyncOrpheus>()?;
    Ok(())
}
