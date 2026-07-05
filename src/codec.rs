use crate::cursor::ByteCursor;
use crate::error::Result;

pub fn zigzag_decode(value: u32) -> i32 {
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

pub fn decode_delta_i16(data: &[u8], count: usize, base: i32, scale: i16) -> Result<Vec<i32>> {
    let mut cursor = ByteCursor::new(data);
    let mut out = Vec::with_capacity(count.min(4096));
    let mut current = base;
    while out.len() < count && !cursor.is_empty() {
        let delta = cursor.read_var_i32()?;
        current = current.wrapping_add(delta.wrapping_mul(scale as i32));
        out.push(current);
    }
    Ok(out)
}

pub fn decode_rle_i16(data: &[u8], expected: usize) -> Result<Vec<i16>> {
    let mut cursor = ByteCursor::new(data);
    let mut out = Vec::with_capacity(expected.min(65536));
    while !cursor.is_empty() && out.len() < expected.saturating_add(256) {
        let control = cursor.read_u8()?;
        if control & 0x80 == 0 {
            let run = (control as usize) + 1;
            let value = cursor.read_i16()?;
            for _ in 0..run {
                out.push(value);
            }
        } else {
            let run = ((control & 0x7f) as usize) + 1;
            for _ in 0..run {
                if cursor.remaining() < 2 {
                    break;
                }
                out.push(cursor.read_i16()?);
            }
        }
    }
    Ok(out)
}

pub fn decode_bitpacked_i16(data: &[u8], expected: usize, bias: i16) -> Vec<i16> {
    let mut reader = BitReader::new(data);
    let mut out = Vec::with_capacity(expected.min(65536));
    let width = (reader.read_bits(4).unwrap_or(8) as u8).clamp(1, 15);
    while out.len() < expected {
        match reader.read_bits(width) {
            Some(raw) => out.push((raw as i16).wrapping_add(bias)),
            None => break,
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct BitReader<'a> {
    data: &'a [u8],
    bit: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, bit: 0 }
    }

    pub fn remaining_bits(&self) -> usize {
        self.data.len() * 8usize - self.bit.min(self.data.len() * 8)
    }

    pub fn read_bits(&mut self, width: u8) -> Option<u32> {
        if width == 0 || width as usize > self.remaining_bits() {
            return None;
        }
        let mut value = 0u32;
        for idx in 0..width {
            let absolute = self.bit + idx as usize;
            let byte = self.data[absolute / 8];
            let bit = (byte >> (absolute % 8)) & 1;
            value |= (bit as u32) << idx;
        }
        self.bit += width as usize;
        Some(value)
    }
}

pub fn normalize_metric(value: i32, scale: i16, offset: i16) -> i32 {
    value.wrapping_mul(scale as i32).wrapping_add(offset as i32)
}

pub fn summarize_samples(samples: &[i32]) -> (i32, i32, i64) {
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    let mut sum = 0i64;
    for &sample in samples {
        min = min.min(sample);
        max = max.max(sample);
        sum += sample as i64;
    }
    if samples.is_empty() {
        (0, 0, 0)
    } else {
        (min, max, sum / samples.len() as i64)
    }
}
