use std::collections::BTreeMap;

use crate::frame::WireFrame;

#[derive(Clone, Debug, Default)]
pub struct Reassembler {
    channels: BTreeMap<u8, Vec<WireFrame>>,
}

impl Reassembler {
    pub fn push(&mut self, frame: WireFrame) {
        self.channels.entry(frame.channel).or_default().push(frame);
    }

    pub fn assemble_primary(&mut self) -> Vec<u8> {
        let channel = self.channels.keys().next().copied().unwrap_or(0);
        let mut frames = self.channels.remove(&channel).unwrap_or_default();
        frames.sort_by_key(|frame| (frame.sequence, frame.fragment));
        let mut out = Vec::new();
        for frame in frames {
            out.extend_from_slice(&frame.payload);
            if frame.flags & 0x80 != 0 {
                break;
            }
        }
        out
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}
