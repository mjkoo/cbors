use std::collections::BTreeMap;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyByteArray, PyBytes, PyDict, PyList};
use pyo3::{wrap_pyfunction, AsPyPointer, FromPyObject, PyObject, ToPyObject};
use serde::{Deserialize, Serialize};
use serde_cbor::value::Value;

/// An enum over all possible CBOR types.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct CborValue(Value);

impl<'source> FromPyObject<'source> for CborValue {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if ob.as_ptr() == unsafe { pyo3::ffi::Py_None() } {
            Ok(CborValue(Value::Null))
        } else if let Ok(b) = ob.extract::<bool>() {
            Ok(CborValue(Value::Bool(b)))
        } else if let Ok(i) = ob.extract::<i128>() {
            Ok(CborValue(Value::Integer(i)))
        } else if let Ok(f) = ob.extract::<f64>() {
            Ok(CborValue(Value::Float(f)))
        } else if let Ok(s) = ob.extract::<String>() {
            Ok(CborValue(Value::Text(s)))
        } else if let Ok(b) = ob.downcast::<PyByteArray>() {
            Ok(CborValue(Value::Bytes(b.to_vec())))
        } else if let Ok(b) = ob.downcast::<PyBytes>() {
            Ok(CborValue(Value::Bytes(b.as_bytes().to_vec())))
        } else if let Ok(a) = ob.downcast::<PyList>() {
            Ok(CborValue(Value::Array(
                a.into_iter()
                    .map(|x| {
                        let cv: CborValue = x.extract()?;
                        Ok(cv.0)
                    })
                    .collect::<PyResult<Vec<_>>>()?,
            )))
        } else if let Ok(d) = ob.downcast::<PyDict>() {
            Ok(CborValue(Value::Map(
                d.into_iter()
                    .map(|(k, v)| {
                        let ck: CborValue = k.extract()?;
                        let cv: CborValue = v.extract()?;
                        Ok((ck.0, cv.0))
                    })
                    .collect::<PyResult<BTreeMap<_, _>>>()?,
            )))
        } else {
            Err(PyTypeError::new_err(format!(
                "Value not convertable to cbor value: {}",
                ob.to_string()
            )))
        }
    }
}

impl ToPyObject for CborValue {
    fn to_object(&self, py: Python) -> PyObject {
        match &self.0 {
            Value::Null => py.None(),
            Value::Bool(b) => b.to_object(py),
            Value::Integer(i) => i.to_object(py),
            Value::Float(f) => f.to_object(py),
            Value::Text(s) => s.to_object(py),
            Value::Bytes(v) => PyBytes::new(py, v).into(),
            Value::Array(a) => a
                .iter()
                .map(|x| CborValue(x.clone()))
                .collect::<Vec<_>>()
                .to_object(py),
            Value::Map(d) => d
                .iter()
                .map(|(k, v)| (CborValue(k.clone()), CborValue(v.clone())))
                .collect::<BTreeMap<_, _>>()
                .to_object(py),
            _ => py.None(),
        }
    }
}

/// loadb(b: ByteString, /) -> Any
/// --
///
/// This function deserializes CBOR from a bytes or bytearray into an object.
#[pyfunction]
fn loadb(py: Python, b: &PyAny) -> PyResult<PyObject> {
    let b = if let Ok(b) = b.downcast::<PyByteArray>() {
        Ok(b.to_vec())
    } else if let Ok(b) = b.downcast::<PyBytes>() {
        Ok(b.as_bytes().to_vec())
    } else {
        Err(PyTypeError::new_err(
            "cbor input must be bytes or bytearray".to_owned(),
        ))
    }?;

    let value =
        CborValue(serde_cbor::from_slice(&b).map_err(|e| PyValueError::new_err(format!("{}", e)))?);
    Ok(value.to_object(py))
}

/// dumpb(a: Any, /) -> bytes
/// --
///
/// This function serializes an object into CBOR-encoded bytes.
#[pyfunction]
fn dumpb(py: Python, a: &PyAny) -> PyResult<PyObject> {
    let bytes = PyBytes::new(
        py,
        &serde_cbor::to_vec(&a.extract::<CborValue>()?)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
    );
    Ok(bytes.to_object(py))
}

/// A Python CBOR (de)serialization module powered by Rust.
#[pymodule]
fn cbors(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(loadb))?;
    m.add_wrapped(wrap_pyfunction!(dumpb))?;

    Ok(())
}
