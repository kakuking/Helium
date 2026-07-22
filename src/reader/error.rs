use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeliumError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid OpenVDB magic number at byte {0}")]
    InvalidMagic(usize),
}