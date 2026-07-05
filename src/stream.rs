use crate::envelope::{BundleDecoder, MAGIC};
use crate::error::{AquaError, Result};
use crate::frame;
use crate::model::Bundle;
use crate::reassembler::Reassembler;

#[derive(Clone, Debug, Default)]
pub struct StreamDecoder {
    direct: Vec<u8>,
    reassembler: Reassembler,
}

impl StreamDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, data: &[u8]) -> Result<()> {
        if data.starts_with(MAGIC) {
            self.direct.extend_from_slice(data);
            return Ok(());
        }
        for frame in frame::parse_frames(data)? {
            if frame.kind == 0 {
                self.direct.extend_from_slice(&frame.payload);
            } else {
                self.reassembler.push(frame);
            }
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<Bundle> {
        if self.direct.starts_with(MAGIC) {
            return BundleDecoder::new().parse(&self.direct);
        }
        let assembled = self.reassembler.assemble_primary();
        if assembled.starts_with(MAGIC) {
            return BundleDecoder::new().parse(&assembled);
        }
        Err(AquaError::InvalidMagic)
    }
}
