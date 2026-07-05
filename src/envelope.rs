use crate::checksum;
use crate::cursor::ByteCursor;
use crate::dictionary;
use crate::error::{AquaError, Result};
use crate::journal;
use crate::ledger;
use crate::manifest;
use crate::model::{Bundle, Section};
use crate::plume;
use crate::script;
use crate::sensor;
use crate::topology;

pub const MAGIC: &[u8; 4] = b"AQWD";
pub const SECTION_DICTIONARY: u8 = 1;
pub const SECTION_MANIFEST: u8 = 2;
pub const SECTION_TOPOLOGY: u8 = 3;
pub const SECTION_SENSOR: u8 = 4;
pub const SECTION_PLUME: u8 = 5;
pub const SECTION_LEDGER: u8 = 6;
pub const SECTION_SCRIPT: u8 = 7;
pub const SECTION_JOURNAL: u8 = 8;

#[derive(Clone, Debug, Default)]
pub struct BundleDecoder {
    strict_checksums: bool,
}

impl BundleDecoder {
    pub fn new() -> Self {
        Self {
            strict_checksums: false,
        }
    }

    pub fn strict_checksums(mut self, strict: bool) -> Self {
        self.strict_checksums = strict;
        self
    }

    pub fn parse(&self, data: &[u8]) -> Result<Bundle> {
        let mut cursor = ByteCursor::new(data);
        if cursor.remaining() < 12 {
            return Err(AquaError::UnexpectedEof {
                needed: 12,
                remaining: cursor.remaining(),
            });
        }
        if cursor.read_exact(4)? != MAGIC {
            return Err(AquaError::InvalidMagic);
        }
        let version = cursor.read_u16()?;
        if version == 0 || version > 3 {
            return Err(AquaError::InvalidVersion(version));
        }
        let flags = cursor.read_u16()?;
        let section_count = cursor.read_u16()? as usize;
        let salt = cursor.read_u16()?;
        let mut bundle = Bundle {
            version,
            flags,
            salt,
            ..Bundle::default()
        };

        for ordinal in 0..section_count.min(256) {
            if cursor.remaining() < 12 {
                bundle.push_warning(format!("truncated section table at {ordinal}"));
                break;
            }
            let kind = cursor.read_u8()?;
            let section_flags = cursor.read_u8()?;
            let id = cursor.read_u16()?;
            let len = cursor.read_u32()? as usize;
            let expected = cursor.read_u32()?;
            if len > cursor.remaining() {
                return Err(AquaError::InvalidLength {
                    declared: len,
                    available: cursor.remaining(),
                });
            }
            let payload = cursor.read_exact(len)?.to_vec();
            let actual = checksum::crc32(&payload);
            if section_flags & 0x01 != 0 && expected != actual {
                if self.strict_checksums {
                    return Err(AquaError::ChecksumMismatch {
                        section: id,
                        expected,
                        actual,
                    });
                }
                bundle.push_warning(format!("section {id} checksum mismatch"));
            }
            let section = Section {
                kind,
                flags: section_flags,
                id,
                checksum: expected,
                payload,
            };
            self.apply_section(&mut bundle, &section)?;
            bundle.sections.push(section);
        }
        Ok(bundle)
    }

    fn apply_section(&self, bundle: &mut Bundle, section: &Section) -> Result<()> {
        match section.kind {
            SECTION_DICTIONARY => {
                let parsed = dictionary::parse_dictionary(&section.payload)?;
                for entry in parsed.entries {
                    bundle.dictionary.insert(entry);
                }
            }
            SECTION_MANIFEST => {
                bundle.manifest = manifest::parse_manifest(&section.payload)?;
            }
            SECTION_TOPOLOGY => {
                bundle.topology = topology::parse_topology(&section.payload, &bundle.dictionary)?;
            }
            SECTION_SENSOR => {
                let mut frames = sensor::parse_sensor_frames(&section.payload, &bundle.dictionary)?;
                bundle.sensor_frames.append(&mut frames);
            }
            SECTION_PLUME => {
                bundle
                    .plume_tiles
                    .push(plume::parse_plume_tile(&section.payload)?);
            }
            SECTION_LEDGER => {
                let mut ops = ledger::parse_ledger_ops(&section.payload)?;
                bundle.ledger_ops.append(&mut ops);
            }
            SECTION_SCRIPT => {
                bundle.scripts.push(script::parse_script(&section.payload)?);
            }
            SECTION_JOURNAL => {
                let mut entries = journal::parse_journal(&section.payload, &bundle.dictionary)?;
                bundle.journals.append(&mut entries);
            }
            other => {
                if other != 0 {
                    bundle.push_warning(format!("unknown section {other}"));
                }
            }
        }
        Ok(())
    }
}
