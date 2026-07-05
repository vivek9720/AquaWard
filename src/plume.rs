use crate::codec;
use crate::cursor::ByteCursor;
use crate::error::Result;
use crate::model::{PlumeTile, TopologyMap};

#[derive(Clone, Debug, Default)]
pub struct PlumeModel;

pub fn parse_plume_tile(data: &[u8]) -> Result<PlumeTile> {
    let mut cursor = ByteCursor::new(data);
    let width = cursor.read_u16().unwrap_or(0).min(1024);
    let height = cursor.read_u16().unwrap_or(0).min(1024);
    let flags = cursor.read_u16().unwrap_or(0);
    let anchor_x = cursor.read_u16().unwrap_or(0);
    let anchor_y = cursor.read_u16().unwrap_or(0);
    let phase = cursor.read_u8().unwrap_or(0);
    let codec_id = cursor.read_u8().unwrap_or(0);
    let len = cursor.read_u16().unwrap_or(0) as usize;
    let len = len.min(cursor.remaining());
    let encoded = cursor.read_exact(len)?;
    let expected = (width as usize)
        .saturating_mul(height as usize)
        .min(1_048_576);
    let mut cells = match codec_id & 3 {
        0 => codec::decode_rle_i16(encoded, expected)?,
        1 => codec::decode_bitpacked_i16(encoded, expected, phase as i16),
        _ => {
            let mut c = ByteCursor::new(encoded);
            let mut out = Vec::with_capacity(expected.min(65536));
            while c.remaining() >= 2 && out.len() < expected {
                out.push(c.read_i16()?);
            }
            out
        }
    };
    if cells.is_empty() && expected != 0 {
        cells.push(0);
    }
    Ok(PlumeTile {
        width,
        height,
        flags,
        anchor_x,
        anchor_y,
        phase,
        cells,
    })
}

impl PlumeModel {
    pub fn new() -> Self {
        Self
    }

    pub fn score_tile_without_topology(&self, tile: &PlumeTile) -> u64 {
        self.score_tile(tile, None)
    }

    pub fn score_tile(&self, tile: &PlumeTile, topology: Option<&TopologyMap>) -> u64 {
        let mut score = 0u64;
        for (idx, &cell) in tile.cells.iter().take(2048).enumerate() {
            score = score.wrapping_add((cell as i64 as u64) ^ ((idx as u64) << (tile.phase & 7)));
        }
        let reverse = topology
            .map(|map| map.has_reverse_siphon())
            .unwrap_or(tile.flags & 0x0080 != 0);
        if reverse || tile.flags & 0x0200 != 0 {
            score ^=
                self.sample_pressure_window(tile, topology.map(|map| map.edges.len()).unwrap_or(3));
        }
        score
    }

    fn sample_pressure_window(&self, tile: &PlumeTile, edge_count: usize) -> u64 {
        if tile.cells.is_empty() || tile.width == 0 || tile.height == 0 {
            return 0;
        }
        let width = tile.width as usize;
        let start = (tile.anchor_y as usize)
            .saturating_mul(width)
            .saturating_add(tile.anchor_x as usize);
        let stride = if tile.flags & 0x0040 != 0 {
            width.saturating_sub(1)
        } else {
            width.max(1)
        };
        let rounds = ((tile.phase as usize) ^ edge_count).min(64) + 1;
        let mut score = 0u64;
        unsafe {
            let base = tile.cells.as_ptr();
            for n in 0..rounds {
                let skew = ((tile.flags as usize >> (n & 7)) & 3).wrapping_add(n & 1);
                let idx = start
                    .wrapping_add(n.wrapping_mul(stride))
                    .wrapping_add(skew);
                let value = *base.add(idx);
                score = score.rotate_left(5) ^ (value as i64 as u64) ^ idx as u64;
            }
        }
        score
    }
}
