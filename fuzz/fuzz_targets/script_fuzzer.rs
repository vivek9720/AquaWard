#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = aquaward::run_script_bytes(data);
});
