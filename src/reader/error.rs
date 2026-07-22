use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeliumError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "Invalid OpenVDB magic number. Found: {found:#018X}, \
         expected: {expected:#018X}"
    )]
    InvalidMagic {
        expected: u64,
        found: u64,
    },

    #[error("Unsupported OpenVDB file version {0}")]
    UnsupportedVersion(u32),

    #[error("Invalid UUID encoding")]
    InvalidUuidEncoding,

    #[error(
        "Invalid boolean in field {field:?}: expected 0 or 1, found {value}"
    )]
    InvalidBoolean {
        field: &'static str,
        value: u8,
    },

    #[error("Invalid UTF-8 string: {0}")]
    InvalidStringEncoding(#[from] FromUtf8Error),

    #[error("String is unreasonably large: {0} bytes")]
    StringTooLarge(usize),

    #[error("Invalid metadata entry count: {0}")]
    InvalidMetadataCount(u32),

    #[error(
        "Metadata payload {name:?} is unreasonably large: {size} bytes"
    )]
    MetadataPayloadTooLarge {
        name: String,
        size: usize,
    },

    #[error("Invalid grid count: {0}")]
    InvalidGridCount(i32),

    #[error(
        "This reader currently requires a VDB file containing grid offsets"
    )]
    GridOffsetsRequired,

    #[error(
        "Invalid offsets for grid {grid_index}: \
         grid={grid_position}, block={block_position}, end={end_position}"
    )]
    InvalidGridOffsets {
        grid_index: usize,
        grid_position: i64,
        block_position: i64,
        end_position: i64,
    },

    #[error(
        "Grid {grid_index} ends at byte {end_position}, \
         but the file is only {file_length} bytes long"
    )]
    GridOffsetPastEnd {
        grid_index: usize,
        end_position: u64,
        file_length: u64,
    },

    #[error(
        "Grid {grid_index} contains negative offsets: \
        grid={grid_position}, block={block_position}, end={end_position}"
    )]
    NegativeGridOffsets {
        grid_index: usize,
        grid_position: i64,
        block_position: i64,
        end_position: i64,
    },

    #[error("Negative stream position in {field}: {value}")]
    NegativeStreamPosition {
        field: &'static str,
        value: i64,
    },

    #[error(
        "Unknown compression flags for grid {grid_index}: {flags:#010X}"
    )]
    UnknownCompressionFlags {
        grid_index: usize,
        flags: u32,
    },

    #[error("Unsupported OpenVDB transform type {0:?}")]
    UnsupportedTransform(String),

    #[error(
        "Grid {grid_index} transform ends at {topology_start}, \
        beyond block position {block_position}"
    )]
    GridSectionOverlap {
        grid_index: usize,
        topology_start: u64,
        block_position: u64,
    },

    #[error("Invalid byte range {start}..{end}")]
    InvalidByteRange {
        start: u64,
        end: u64,
    },

    #[error("Byte range {start}..{end} is too large")]
    ByteRangeTooLarge {
        start: u64,
        end: u64,
    },
}