import cbors
import pytest

def test_invalid():
    with pytest.raises(TypeError):
        cbors.loadb(1)
    with pytest.raises(ValueError):
        cbors.loadb(b'foo')

    class foo:
        pass

    with pytest.raises(TypeError):
        cbors.dumpb(foo())
    with pytest.raises(TypeError):
        cbors.dumpb({'foo': foo()})

def test_bytestring():
    test = b'\x5f\x44\xaa\xbb\xcc\xdd\x43\xee\xff\x99\xff'
    expected = b'\xaa\xbb\xcc\xdd\xee\xff\x99'

    assert cbors.loadb(test) == expected
    print(cbors.dumpb(expected))

