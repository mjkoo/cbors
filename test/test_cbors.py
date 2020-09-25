import math
from string import printable

import cbors
import pytest

from hypothesis import given, settings, HealthCheck
from hypothesis.strategies import (
    binary,
    booleans,
    dictionaries,
    floats,
    integers,
    lists,
    none,
    recursive,
    text,
)

i64 = integers(min_value=-9223372036854775808, max_value=9223372036854775807)
u64 = integers(min_value=0, max_value=18446744073709551615)
bytearrays = binary().map(bytearray)
cbor_keys = none() | booleans() | i64 | text(printable) | binary()
cbor_values = recursive(
    none()
    | booleans()
    | u64
    | i64
    | floats()
    | text(printable)
    | binary()
    | bytearrays,
    lambda children: lists(children, min_size=1)
    | dictionaries(cbor_keys, children, min_size=1),
)


def assert_equal(expected, actual):
    if expected is None:
        assert actual is None
    elif isinstance(expected, list):
        assert isinstance(actual, list)
        assert len(expected) == len(actual)
        for e, a in zip(expected, actual):
            assert_equal(e, a)
    elif isinstance(expected, dict):
        assert isinstance(actual, dict)
        assert len(expected) == len(actual)
        keys = set().union(expected, actual)
        for k in keys:
            assert k in expected
            assert k in actual
            assert_equal(expected[k], actual[k])
    elif isinstance(expected, float):
        if math.isnan(expected):
            assert math.isnan(actual)
        else:
            assert math.isclose(expected, actual)
    else:
        assert expected == actual


@given(cbor_values)
@settings(suppress_health_check=(HealthCheck.too_slow,))
def test_decode_inverts_encode(v):
    assert_equal(v, cbors.loadb(cbors.dumpb(v)))


def test_invalid():
    with pytest.raises(TypeError):
        cbors.loadb(1)
    with pytest.raises(ValueError):
        cbors.loadb(b"foo")

    class foo:
        pass

    with pytest.raises(TypeError):
        cbors.dumpb(foo())
    with pytest.raises(TypeError):
        cbors.dumpb({"foo": foo()})


def test_input_types():
    cbors.loadb(b"\x01")
    cbors.loadb(bytearray(b"\x01"))
    with pytest.raises(TypeError):
        cbors.loadb(1)


def test_output_type():
    assert isinstance(cbors.dumpb(1), bytes)


def test_rfc():
    examples = [
        (0, 0x00),
        (1, 0x01),
        (10, 0x0A),
        (23, 0x17),
        (24, 0x1818),
        (25, 0x1819),
        (100, 0x1864),
        (1000, 0x1903E8),
        (1000000, 0x1A000F4240),
        (1000000000000, 0x1B000000E8D4A51000),
        (18446744073709551615, 0x1BFFFFFFFFFFFFFFFF),
        (-1, 0x20),
        (-10, 0x29),
        (-100, 0x3863),
        (-1000, 0x3903E7),
        (0.0, 0xF90000),
        (-0.0, 0xF98000),
        (1.0, 0xF93C00),
        (1.1, 0xFB3FF199999999999A),
        (1.5, 0xF93E00),
        (65504.0, 0xF97BFF),
        (100000.0, 0xFA47C35000),
        (3.4028234663852886e38, 0xFA7F7FFFFF),
        (1.0e300, 0xFB7E37E43C8800759C),
        (5.960464477539063e-8, 0xF90001),
        (0.00006103515625, 0xF90400),
        (-4.0, 0xF9C400),
        (-4.1, 0xFBC010666666666666),
        (float("inf"), 0xF97C00),
        (float("nan"), 0xF97E00),
        (float("-inf"), 0xF9FC00),
        (False, 0xF4),
        (True, 0xF5),
        (None, 0xF6),
        ("", 0x60),
        ("a", 0x6161),
        ("IETF", 0x6449455446),
        ('"\\', 0x62225C),
        ("\u00fc", 0x62C3BC),
        ("\u6c34", 0x63E6B0B4),
        ([], 0x80),
        ([1, 2, 3], 0x83010203),
        ([1, [2, 3], [4, 5]], 0x8301820203820405),
        (
            list(range(1, 26)),
            0x98190102030405060708090A0B0C0D0E0F101112131415161718181819,
        ),
        ({}, 0xA0),
        ({1: 2, 3: 4}, 0xA201020304),
        ({"a": 1, "b": [2, 3]}, 0xA26161016162820203),
        (["a", {"b": "c"}], 0x826161A161626163),
        ({c: c.upper() for c in "abcde"}, 0xA56161614161626142616361436164614461656145),
    ]

    for v, h in examples:
        e = bytes.fromhex("{:02x}".format(h))
        assert cbors.dumpb(v) == e
        assert_equal(v, cbors.loadb(e))
