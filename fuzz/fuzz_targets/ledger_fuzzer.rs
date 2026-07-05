#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = aquaward::replay_ledger_bytes(data);
});
