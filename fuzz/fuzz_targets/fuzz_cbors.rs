#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_cbor::Value;

fuzz_target!(|data: &[u8]| {
    let _: Result<Value, _> = serde_cbor::from_slice(data);
});
