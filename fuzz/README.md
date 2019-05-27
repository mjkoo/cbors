Harness for fuzzing via `cargo-fuzz`

A similar harness is provided upstream in `serde_cbor`, however outstanding tickets seem to indicate [some issues with it](https://github.com/pyfisch/cbor/issues/102).

Run via `cargo fuzz run fuzz_cbors` from the top directory.

An initial corpus is provided via the seeds from `serde_cbor`'s harness
