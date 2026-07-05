use core::fmt;

pub type Result<T> = core::result::Result<T, AquaError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AquaError {
    UnexpectedEof {
        needed: usize,
        remaining: usize,
    },
    InvalidMagic,
    InvalidVersion(u16),
    InvalidSection(u8),
    InvalidLength {
        declared: usize,
        available: usize,
    },
    ChecksumMismatch {
        section: u16,
        expected: u32,
        actual: u32,
    },
    VarintTooLong,
    Utf8Field,
    Semantic(String),
}

impl fmt::Display for AquaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AquaError::UnexpectedEof { needed, remaining } => {
                write!(f, "unexpected eof: needed {needed}, remaining {remaining}")
            }
            AquaError::InvalidMagic => write!(f, "invalid AquaWard magic"),
            AquaError::InvalidVersion(version) => write!(f, "unsupported version {version}"),
            AquaError::InvalidSection(section) => write!(f, "invalid section {section}"),
            AquaError::InvalidLength {
                declared,
                available,
            } => {
                write!(f, "invalid length {declared}, available {available}")
            }
            AquaError::ChecksumMismatch {
                section,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "checksum mismatch in section {section}: {expected:#x} != {actual:#x}"
                )
            }
            AquaError::VarintTooLong => write!(f, "varint is too long"),
            AquaError::Utf8Field => write!(f, "text field is not utf-8"),
            AquaError::Semantic(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AquaError {}
