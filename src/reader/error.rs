use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeliumError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid OpenVDB magic number. Found: {found:?}, Expected: {expected:?}")]
    InvalidMagic{
        expected: u64,
        found: u64
    },

    #[error("Unsupported Version {0}")]
    UnsupportedVersion(u32),
    
    #[error("Invalid UUID decoding")]
    InvalidUuidEncoding,
    
    #[error("Invalid Boolean in field {field:?}, value {value:?}")]
    InvalidBoolean {
        field: &'static str,
        value: u8,
    },
}