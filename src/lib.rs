#![allow(dead_code)]

mod client;
mod types;

use pyo3::exceptions as exc;
use pyo3::prelude::*;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<client::Orpheus>()?;
    m.add_class::<client::AsyncOrpheus>()?;
    m.add("UnauthorizedError", m.py().get_type::<exc::PyIOError>())?;
    Ok(())
}
