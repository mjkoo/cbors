# cbo-rs
[![Build Status](https://travis-ci.org/mjkoo/cbors.svg?branch=master)](https://travis-ci.org/mjkoo/cbors)
[![codecov](https://codecov.io/gh/mjkoo/cbors/branch/master/graph/badge.svg)](https://codecov.io/gh/mjkoo/cbors)
[![PyPI](https://img.shields.io/pypi/v/cbors.svg)](https://pypi.org/project/cbors/)

A Python CBOR (de)serialization module, powered by Rust.

Wraps the excellent [serde_cbor](https://github.com/pyfisch/cbor) crate and provides a pythonic interface via [pyo3](https://github.com/PyO3/pyo3).

## Installation

Python>=3.5 is required due to the requirements of pyo3.

Recommended to install from [PyPI](https://pypi.org/project/cbors/), e.g.

```
pip install cbors
```

To install from source, use [maturin](https://github.com/PyO3/maturin) to build a wheel from repository root.

```
maturin build -i python3
pip install target/wheels/*.whl
```

## Usage

Serialize data via `cbors.dumpb`, deserialize via `cbors.loadb`.

Interface is similar to the standard library's `json` module.

```python
import cbors

b = cbors.dumpb("foo")
assert(b == b"cfoo")

s = cbors.loadb(b)
assert(s = "foo")
```

## Limitations

As this uses `serde_cbor` under the hood, the same limitations apply here.

Notably, tags are not currently supported (see [pyfisch/cbor#3](https://github.com/pyfisch/cbor/issues/3)).

If this functionality is important to you, [cbor2](https://pypi.org/project/cbor2/) might be a better choice.

## Development

For local development, it is recommended to create a virtual environment, and build the module via `maturin develop`.

A Dockerfile is provided which will build and install the module and run the test suite.

If you do not want to use docker, it is recommended to use `tox` for testing.

Pull requests welcome!
