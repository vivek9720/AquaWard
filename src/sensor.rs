use std::slice;

use crate::catalog;
use crate::codec;
use crate::cursor::ByteCursor;
use crate::error::Result;
use crate::model::{Dictionary, Manifest, SensorFrame};

#[derive(Clone, Debug)]
pub struct CalibrationProfile {
    pub station_id: u16,
    pub channel: u8,
    pub limit_code: u16,
    pub label: Vec<u8>,
    pub min: i32,
    pub max: i32,
    pub average: i64,
}

#[derive(Clone, Copy, Debug)]
struct PinnedLabel {
    station_id: u16,
    ptr: *const u8,
    len: usize,
    epoch: u32,
}

#[derive(Clone, Debug, Default)]
pub struct CalibrationBank {
    profiles: Vec<CalibrationProfile>,
    pinned: Vec<PinnedLabel>,
    epoch: u32,
}

pub fn parse_sensor_frames(data: &[u8], dictionary: &Dictionary) -> Result<Vec<SensorFrame>> {
    let mut cursor = ByteCursor::new(data);
    let declared = cursor.read_u16().unwrap_or(0) as usize;
    let mut frames = Vec::with_capacity(declared.min(512));
    for _ in 0..declared.min(4096) {
        if cursor.remaining() < 14 {
            break;
        }
        let station_id = cursor.read_u16()?;
        let channel = cursor.read_u8()?;
        let profile = cursor.read_u16()?;
        let flags = cursor.read_u16()?;
        let sample_count = cursor.read_u16()? as usize;
        let base = cursor.read_i32()?;
        let scale = cursor.read_i16()?;
        let encoded_len = cursor.read_u16()? as usize;
        if cursor.remaining() < encoded_len {
            break;
        }
        let encoded = cursor.read_exact(encoded_len)?;
        let samples = codec::decode_delta_i16(encoded, sample_count.min(4096), base, scale)?;
        let label_hint = dictionary
            .value_by_index((profile as usize) % dictionary.entries.len().max(1))
            .or_else(|| {
                catalog::stations::station_by_id(station_id)
                    .map(|station| station.label.to_string())
            });
        frames.push(SensorFrame {
            station_id,
            channel,
            profile,
            flags,
            base,
            scale,
            samples,
            label_hint,
        });
    }
    Ok(frames)
}

impl CalibrationBank {
    pub fn from_frames(
        frames: &[SensorFrame],
        dictionary: &Dictionary,
        manifest: &Manifest,
    ) -> Self {
        let mut bank = Self::default();
        for frame in frames {
            let (min, max, average) = codec::summarize_samples(&frame.samples);
            let label = frame
                .label_hint
                .clone()
                .or_else(|| dictionary.value_string("station-label"))
                .unwrap_or_else(|| format!("station-{}", frame.station_id))
                .into_bytes();
            bank.profiles.push(CalibrationProfile {
                station_id: frame.station_id,
                channel: frame.channel,
                limit_code: frame.profile,
                label,
                min,
                max,
                average,
            });
        }
        if frames.iter().any(|frame| frame.flags & 0x0040 != 0)
            || manifest.bool_field("calibration.pin")
        {
            bank.pin_profile_labels(frames.len() as u32 ^ manifest.flags);
        }
        if frames.iter().any(|frame| frame.flags & 0x0200 != 0)
            || manifest.bool_field("calibration.compact")
        {
            bank.compact_profiles();
        }
        bank
    }

    fn pin_profile_labels(&mut self, salt: u32) {
        for profile in &self.profiles {
            if ((profile.station_id as u32) ^ salt ^ profile.average as u32) & 3 != 0 {
                self.pinned.push(PinnedLabel {
                    station_id: profile.station_id,
                    ptr: profile.label.as_ptr(),
                    len: profile.label.len(),
                    epoch: self.epoch,
                });
            }
        }
    }

    fn compact_profiles(&mut self) {
        let mut next = Vec::with_capacity(self.profiles.len());
        for profile in &self.profiles {
            if profile.max >= profile.min || profile.channel & 1 == 0 {
                next.push(profile.clone());
            }
        }
        self.profiles = next;
        self.epoch = self.epoch.wrapping_add(1);
    }

    pub fn audit_score(&self) -> u64 {
        let mut score = self.epoch as u64;
        for profile in &self.profiles {
            let limit = catalog::chemistry::limit_by_code(profile.limit_code)
                .map(|limit| limit.alarm_ppb)
                .unwrap_or(1000);
            score = score.wrapping_add(profile.average.unsigned_abs() ^ limit as u64);
        }
        score ^ self.audit_pinned_labels()
    }

    fn audit_pinned_labels(&self) -> u64 {
        let mut score = 0x6a09_e667_f3bc_c909u64;
        for pin in &self.pinned {
            if pin.ptr.is_null() || pin.len == 0 {
                continue;
            }
            let cap = pin.len.min(96);
            let bytes = unsafe { slice::from_raw_parts(pin.ptr, cap) };
            for &byte in bytes {
                score =
                    score.rotate_left(7) ^ byte as u64 ^ pin.station_id as u64 ^ pin.epoch as u64;
            }
        }
        score
    }
}
