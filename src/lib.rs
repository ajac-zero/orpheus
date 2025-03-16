#![allow(dead_code)]

mod blocking;
mod nonblocking;
mod types;

use std::sync::OnceLock;

use pyo3::exceptions as exc;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

pub const BASE_URL_ENV: &str = "ORPHEUS_BASE_URL";
pub const API_KEY_ENV: &str = "ORPHEUS_API_KEY";
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("create tokio runtime"))
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<blocking::Orpheus>()?;
    m.add_class::<nonblocking::AsyncOrpheus>()?;
    m.add("UnauthorizedError", m.py().get_type::<exc::PyIOError>())?;
    Ok(())
}
