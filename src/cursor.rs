use crate::checksum;
use crate::error::{AquaError, Result};

#[derive(Clone, Debug)]
pub struct ByteCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteCursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn rewind(&mut self, amount: usize) {
        self.pos = self.pos.saturating_sub(amount);
    }

    pub fn advance(&mut self, amount: usize) -> Result<()> {
        if self.remaining() < amount {
            return Err(AquaError::UnexpectedEof {
                needed: amount,
                remaining: self.remaining(),
            });
        }
        self.pos += amount;
        Ok(())
    }

    pub fn read_exact(&mut self, amount: usize) -> Result<&'a [u8]> {
        if self.remaining() < amount {
            return Err(AquaError::UnexpectedEof {
                needed: amount,
                remaining: self.remaining(),
            });
        }
        let out = &self.data[self.pos..self.pos + amount];
        self.pos += amount;
        Ok(out)
    }

    pub fn peek_exact(&self, amount: usize) -> Result<&'a [u8]> {
        if self.remaining() < amount {
            return Err(AquaError::UnexpectedEof {
                needed: amount,
                remaining: self.remaining(),
            });
        }
        Ok(&self.data[self.pos..self.pos + amount])
    }

    pub fn rest(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1)?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let b = self.read_exact(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let b = self.read_exact(2)?;
        Ok(i16::from_le_bytes([b[0], b[1]]))
    }

    pub fn read_u24(&mut self) -> Result<u32> {
        let b = self.read_exact(3)?;
        Ok((b[0] as u32) | ((b[1] as u32) << 8) | ((b[2] as u32) << 16))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let b = self.read_exact(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let b = self.read_exact(4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        let b = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub fn read_var_u32(&mut self) -> Result<u32> {
        let mut shift = 0;
        let mut value = 0u32;
        for _ in 0..5 {
            let byte = self.read_u8()?;
            value |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
        }
        Err(AquaError::VarintTooLong)
    }

    pub fn read_var_i32(&mut self) -> Result<i32> {
        let raw = self.read_var_u32()?;
        Ok(((raw >> 1) as i32) ^ (-((raw & 1) as i32)))
    }

    pub fn read_len_u8_bytes(&mut self) -> Result<&'a [u8]> {
        let len = self.read_u8()? as usize;
        self.read_exact(len)
    }

    pub fn read_len_u16_bytes(&mut self, max: usize) -> Result<&'a [u8]> {
        let len = self.read_u16()? as usize;
        if len > max {
            return Err(AquaError::InvalidLength {
                declared: len,
                available: max,
            });
        }
        self.read_exact(len)
    }

    pub fn read_lossy_string(&mut self, len: usize) -> Result<String> {
        let bytes = self.read_exact(len)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn fork(&self, len: usize) -> Result<ByteCursor<'a>> {
        if self.remaining() < len {
            return Err(AquaError::UnexpectedEof {
                needed: len,
                remaining: self.remaining(),
            });
        }
        Ok(ByteCursor::new(&self.data[self.pos..self.pos + len]))
    }

    pub fn checksum_from_here(&self, len: usize) -> Result<u32> {
        if self.remaining() < len {
            return Err(AquaError::UnexpectedEof {
                needed: len,
                remaining: self.remaining(),
            });
        }
        Ok(checksum::crc32(&self.data[self.pos..self.pos + len]))
    }
}

pub fn capped_len(declared: usize, available: usize, cap: usize) -> Result<usize> {
    if declared > available {
        return Err(AquaError::InvalidLength {
            declared,
            available,
        });
    }
    Ok(declared.min(cap))
}
