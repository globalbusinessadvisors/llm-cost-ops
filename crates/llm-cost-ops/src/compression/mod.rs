// Compression module for supporting gzip/brotli encoding

pub mod types;
pub mod codec;
pub mod middleware;
pub mod config;
pub mod metrics;

pub use types::{CompressionAlgorithm, CompressionLevel, ContentEncoding, CompressionStats};
pub use codec::{Compressor, compress, decompress};
pub use middleware::{CompressionLayer, compression_layer};
pub use config::CompressionConfig;
pub use metrics::CompressionMetrics;

/// Compression error types
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("Invalid compression level: {0}")]
    InvalidLevel(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    #[error("Size limit exceeded: {0}")]
    SizeLimitExceeded(String),
}

impl From<CompressionError> for crate::domain::CostOpsError {
    fn from(err: CompressionError) -> Self {
        crate::domain::CostOpsError::Internal(err.to_string())
    }
}

pub type CompressionResult<T> = Result<T, CompressionError>;
