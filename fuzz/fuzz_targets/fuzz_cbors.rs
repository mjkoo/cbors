#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate cbors;

fuzz_target!(|data: &[u8]| {

});
