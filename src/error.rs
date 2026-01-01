use thiserror::Error;

#[derive(Error, Debug)]
pub enum PcdError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid header at line {line}: {msg}")]
    InvalidHeader { line: usize, msg: String },

    #[error("Unsupported field type: {0}")]
    UnsupportedType(String),

    #[error("Unsupported data format: {0}")]
    UnsupportedDataFormat(String),

    #[error("Invalid data: {0}")]
    InvalidDataFormat(String),

    #[error("Decompression failed: {0}")]
    Decompression(String),

    #[error("Layout mismatch: expected {expected}, got {got}")]
    LayoutMismatch { expected: usize, got: usize },

    #[error("Buffer too small: expected {expected}, got {got}")]
    BufferTooSmall { expected: usize, got: usize },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, PcdError>;
