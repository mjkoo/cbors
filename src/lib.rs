use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes};
use pyo3::{wrap_pyfunction, FromPyObject, PyObject, ToPyObject};

use serde_cbor::Value;
pub struct CborValue(Value);

impl<'source> FromPyObject<'source> for CborValue {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(CborValue(Value::String("hello".to_owned())))
    }
}

impl ToPyObject for CborValue {
    fn to_object(&self, py: Python) -> PyObject {
        match &self.0 {
            Value::U64(u) => u.to_object(py),
            Value::I64(i) => i.to_object(py),
            Value::Bytes(v) => PyBytes::new(py, &v).into(),
            Value::String(s) => s.to_object(py),
            //Value::Array(v) => v.iter().map(|x| x.into()).collect().into(),
            //Value::Object(o) => ...,
            Value::F64(f) => f.to_object(py),
            Value::Bool(b) => b.to_object(py),
            Value::Null => py.None(),
            _ => "foo".to_owned().to_object(py),
        }
    }
}

#[pyfunction]
fn loadb(py: Python, b: &PyBytes) -> PyResult<PyObject> {
    let value = CborValue(
        serde_cbor::from_slice(b.as_bytes()).map_err(|e| ValueError::py_err(format!("{}", e)))?,
    );
    Ok(value.to_object(py))
}

#[pymodule]
fn cbors(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(loadb))?;

    Ok(())
}
