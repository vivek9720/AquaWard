pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

pub fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

pub fn fold_u64(mut value: u64) -> u32 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51afd7ed558ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ceb9fe1a85ec53);
    (value ^ (value >> 33)) as u32
}

pub fn rolling_tag(data: &[u8], salt: u32) -> u32 {
    let mut acc = salt ^ 0x9e37_79b9;
    for (idx, &byte) in data.iter().enumerate() {
        let lane = ((idx as u32) << 8) ^ byte as u32;
        acc = acc.rotate_left(5) ^ lane.wrapping_mul(0x45d9_f3b);
    }
    acc
}
