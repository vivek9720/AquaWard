#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = aquaward::analyze_plume_bytes(data);
});
