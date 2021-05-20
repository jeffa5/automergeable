#![no_main]

use automergeable::automerge::Backend;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: Vec<u8>| {
    let _ = Backend::load(bytes);
});
