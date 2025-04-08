pub mod chat;
pub mod embed;

use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// This object accepts arbitrary python dictionaries,
/// as long as the can be serialized to a JSON string.
/// It is basically to be used as a wildcard for unknown dicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitraryDict(Value);

impl<'py> FromPyObject<'py> for ArbitraryDict {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        depythonize::<Value>(ob).map(Self).map_err(|e| e.into())
    }
}

impl<'py> IntoPyObject<'py> for ArbitraryDict {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        pythonize(py, &self.0).map_err(|e| e.into())
    }
}
