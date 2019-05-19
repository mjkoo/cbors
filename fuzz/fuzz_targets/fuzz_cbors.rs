#![no_main]
use libfuzzer_sys::fuzz_target;

use cbors::cbors;
use pyo3::prelude::*;
use pyo3::{wrap_pymodule};

fuzz_target!(|data: &[u8]| {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let module = wrap_pymodule!(cbors)(py);
});
