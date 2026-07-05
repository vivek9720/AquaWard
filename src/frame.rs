use crate::checksum;
use crate::cursor::ByteCursor;
use crate::error::Result;

#[derive(Clone, Debug)]
pub struct WireFrame {
    pub channel: u8,
    pub kind: u8,
    pub sequence: u16,
    pub fragment: u16,
    pub flags: u8,
    pub payload: Vec<u8>,
}

pub fn parse_frames(data: &[u8]) -> Result<Vec<WireFrame>> {
    let mut cursor = ByteCursor::new(data);
    let mut frames = Vec::new();
    while cursor.remaining() >= 12 {
        if cursor.peek_exact(3).map(|b| b != b"AQF").unwrap_or(true) {
            cursor.advance(1)?;
            continue;
        }
        cursor.advance(3)?;
        let channel = cursor.read_u8()?;
        let kind = cursor.read_u8()?;
        let sequence = cursor.read_u16()?;
        let fragment = cursor.read_u16()?;
        let flags = cursor.read_u8()?;
        let len = cursor.read_u16()? as usize;
        if cursor.remaining() < len + 4 {
            break;
        }
        let payload = cursor.read_exact(len)?.to_vec();
        let expected = cursor.read_u32()?;
        if flags & 1 != 0 && checksum::crc32(&payload) != expected {
            continue;
        }
        frames.push(WireFrame {
            channel,
            kind,
            sequence,
            fragment,
            flags,
            payload,
        });
    }
    Ok(frames)
}
