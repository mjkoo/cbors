use std::collections::BTreeMap;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyByteArray, PyBytes, PyDict, PyList};
use pyo3::{wrap_pyfunction, AsPyPointer, FromPyObject, PyObject, ToPyObject};
use serde::{Deserialize, Serialize};
use serde_cbor::{ObjectKey, Value};

/// A simplified CBOR value containing only types useful for keys.
#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq, Deserialize, Serialize)]
struct CborObjectKey(ObjectKey);

/// An enum over all possible CBOR types.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct CborValue(Value);

impl<'source> FromPyObject<'source> for CborObjectKey {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if ob.as_ptr() == unsafe { pyo3::ffi::Py_None() } {
            Ok(CborObjectKey(ObjectKey::Null))
        } else if let Ok(b) = ob.extract::<bool>() {
            Ok(CborObjectKey(ObjectKey::Bool(b)))
        } else if let Ok(i) = ob.extract::<i64>() {
            Ok(CborObjectKey(ObjectKey::Integer(i)))
        } else if let Ok(s) = ob.extract::<String>() {
            Ok(CborObjectKey(ObjectKey::String(s)))
        } else if let Ok(b) = ob.downcast::<PyByteArray>() {
            Ok(CborObjectKey(ObjectKey::Bytes(b.to_vec())))
        } else if let Ok(b) = ob.downcast::<PyBytes>() {
            Ok(CborObjectKey(ObjectKey::Bytes(b.as_bytes().to_vec())))
        } else {
            Err(PyTypeError::new_err(format!(
                "Value not convertable to cbor object key: {}",
                ob.to_string()
            )))
        }
    }
}

impl<'source> FromPyObject<'source> for CborValue {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if ob.as_ptr() == unsafe { pyo3::ffi::Py_None() } {
            Ok(CborValue(Value::Null))
        } else if let Ok(b) = ob.extract::<bool>() {
            Ok(CborValue(Value::Bool(b)))
        } else if let Ok(u) = ob.extract::<u64>() {
            Ok(CborValue(Value::U64(u)))
        } else if let Ok(i) = ob.extract::<i64>() {
            Ok(CborValue(Value::I64(i)))
        } else if let Ok(f) = ob.extract::<f64>() {
            Ok(CborValue(Value::F64(f)))
        } else if let Ok(s) = ob.extract::<String>() {
            Ok(CborValue(Value::String(s)))
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
            Ok(CborValue(Value::Object(
                d.into_iter()
                    .map(|(k, v)| {
                        let ck: CborObjectKey = k.extract()?;
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

impl ToPyObject for CborObjectKey {
    fn to_object(&self, py: Python) -> PyObject {
        match &self.0 {
            ObjectKey::Null => py.None(),
            ObjectKey::Bool(b) => b.to_object(py),
            ObjectKey::Integer(i) => i.to_object(py),
            ObjectKey::String(s) => s.to_object(py),
            ObjectKey::Bytes(v) => PyBytes::new(py, &v).into(),
        }
    }
}

impl ToPyObject for CborValue {
    fn to_object(&self, py: Python) -> PyObject {
        match &self.0 {
            Value::Null => py.None(),
            Value::Bool(b) => b.to_object(py),
            Value::U64(u) => u.to_object(py),
            Value::I64(i) => i.to_object(py),
            Value::F64(f) => f.to_object(py),
            Value::String(s) => s.to_object(py),
            Value::Bytes(v) => PyBytes::new(py, &v).into(),
            Value::Array(a) => a
                .iter()
                .map(|x| CborValue(x.clone()))
                .collect::<Vec<_>>()
                .to_object(py),
            Value::Object(d) => d
                .iter()
                .map(|(k, v)| (CborObjectKey(k.clone()), CborValue(v.clone())))
                .collect::<BTreeMap<_, _>>()
                .to_object(py),
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
