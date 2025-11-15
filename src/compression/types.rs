// Core types for compression

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    /// Gzip compression (RFC 1952)
    Gzip,
    /// Brotli compression (RFC 7932)
    Brotli,
    /// Deflate compression (RFC 1951)
    Deflate,
    /// No compression (identity)
    Identity,
}

impl CompressionAlgorithm {
    /// Get all supported algorithms
    pub fn all() -> Vec<Self> {
        vec![
            Self::Brotli,
            Self::Gzip,
            Self::Deflate,
            Self::Identity,
        ]
    }

    /// Get the content-encoding header value
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gzip => "gzip",
            Self::Brotli => "br",
            Self::Deflate => "deflate",
            Self::Identity => "identity",
        }
    }

    /// Get quality score for algorithm selection (higher is better)
    pub fn quality_score(&self) -> u8 {
        match self {
            Self::Brotli => 100,  // Best compression ratio
            Self::Gzip => 90,     // Good compression, widely supported
            Self::Deflate => 80,  // Legacy support
            Self::Identity => 0,  // No compression
        }
    }

    /// Check if algorithm requires compression
    pub fn is_compressed(&self) -> bool {
        !matches!(self, Self::Identity)
    }
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CompressionAlgorithm {
    type Err = super::CompressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gzip" | "x-gzip" => Ok(Self::Gzip),
            "br" | "brotli" => Ok(Self::Brotli),
            "deflate" => Ok(Self::Deflate),
            "identity" => Ok(Self::Identity),
            _ => Err(super::CompressionError::UnsupportedAlgorithm(
                s.to_string(),
            )),
        }
    }
}

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Fastest compression (level 1)
    Fastest,
    /// Fast compression (level 3)
    Fast,
    /// Balanced compression (level 6)
    Default,
    /// Best compression (level 9)
    Best,
    /// Custom level (0-11 for brotli, 0-9 for gzip/deflate)
    Custom(u32),
}

impl CompressionLevel {
    /// Get numeric level for gzip/deflate (0-9)
    pub fn gzip_level(&self) -> u32 {
        match self {
            Self::Fastest => 1,
            Self::Fast => 3,
            Self::Default => 6,
            Self::Best => 9,
            Self::Custom(level) => (*level).min(9),
        }
    }

    /// Get numeric level for brotli (0-11)
    pub fn brotli_level(&self) -> u32 {
        match self {
            Self::Fastest => 1,
            Self::Fast => 4,
            Self::Default => 6,
            Self::Best => 11,
            Self::Custom(level) => (*level).min(11),
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Default
    }
}

/// Content encoding with quality factor
#[derive(Debug, Clone, PartialEq)]
pub struct ContentEncoding {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
    /// Quality factor (0.0 - 1.0)
    pub quality: f32,
}

impl ContentEncoding {
    /// Create a new content encoding
    pub fn new(algorithm: CompressionAlgorithm, quality: f32) -> Self {
        Self {
            algorithm,
            quality: quality.clamp(0.0, 1.0),
        }
    }

    /// Create with default quality (1.0)
    pub fn with_algorithm(algorithm: CompressionAlgorithm) -> Self {
        Self::new(algorithm, 1.0)
    }

    /// Parse Accept-Encoding header value
    pub fn parse_accept_encoding(header: &str) -> Vec<Self> {
        let mut encodings = Vec::new();

        for part in header.split(',') {
            let part = part.trim();
            let (encoding, quality) = if let Some((enc, q)) = part.split_once(';') {
                let enc = enc.trim();
                let quality = q
                    .trim()
                    .strip_prefix("q=")
                    .and_then(|q| q.parse::<f32>().ok())
                    .unwrap_or(1.0);
                (enc, quality)
            } else {
                (part, 1.0)
            };

            if let Ok(algorithm) = CompressionAlgorithm::from_str(encoding) {
                encodings.push(Self::new(algorithm, quality));
            } else if encoding == "*" {
                // Wildcard - accept any encoding
                for algo in CompressionAlgorithm::all() {
                    encodings.push(Self::new(algo, quality * 0.5));
                }
            }
        }

        // Sort by quality (descending) and algorithm score
        encodings.sort_by(|a, b| {
            let quality_cmp = b.quality.partial_cmp(&a.quality).unwrap();
            if quality_cmp == std::cmp::Ordering::Equal {
                b.algorithm
                    .quality_score()
                    .cmp(&a.algorithm.quality_score())
            } else {
                quality_cmp
            }
        });

        encodings
    }

    /// Select best encoding from accepted encodings
    pub fn select_best(accept_encoding: &str, supported: &[CompressionAlgorithm]) -> Option<CompressionAlgorithm> {
        let encodings = Self::parse_accept_encoding(accept_encoding);

        for encoding in encodings {
            if encoding.quality > 0.0 && supported.contains(&encoding.algorithm) {
                return Some(encoding.algorithm);
            }
        }

        None
    }
}

/// Compression statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression ratio (compressed/original)
    pub compression_ratio: f64,
    /// Space saved in bytes
    pub bytes_saved: usize,
    /// Algorithm used
    pub algorithm: Option<CompressionAlgorithm>,
    /// Time taken in milliseconds
    pub duration_ms: f64,
}

impl CompressionStats {
    /// Create new compression stats
    pub fn new(
        original_size: usize,
        compressed_size: usize,
        algorithm: CompressionAlgorithm,
        duration_ms: f64,
    ) -> Self {
        let compression_ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            1.0
        };

        let bytes_saved = original_size.saturating_sub(compressed_size);

        Self {
            original_size,
            compressed_size,
            compression_ratio,
            bytes_saved,
            algorithm: Some(algorithm),
            duration_ms,
        }
    }

    /// Get compression percentage
    pub fn compression_percentage(&self) -> f64 {
        (1.0 - self.compression_ratio) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_from_str() {
        assert_eq!(
            CompressionAlgorithm::from_str("gzip").unwrap(),
            CompressionAlgorithm::Gzip
        );
        assert_eq!(
            CompressionAlgorithm::from_str("br").unwrap(),
            CompressionAlgorithm::Brotli
        );
        assert_eq!(
            CompressionAlgorithm::from_str("deflate").unwrap(),
            CompressionAlgorithm::Deflate
        );
        assert_eq!(
            CompressionAlgorithm::from_str("identity").unwrap(),
            CompressionAlgorithm::Identity
        );
    }

    #[test]
    fn test_algorithm_as_str() {
        assert_eq!(CompressionAlgorithm::Gzip.as_str(), "gzip");
        assert_eq!(CompressionAlgorithm::Brotli.as_str(), "br");
        assert_eq!(CompressionAlgorithm::Deflate.as_str(), "deflate");
        assert_eq!(CompressionAlgorithm::Identity.as_str(), "identity");
    }

    #[test]
    fn test_compression_level_gzip() {
        assert_eq!(CompressionLevel::Fastest.gzip_level(), 1);
        assert_eq!(CompressionLevel::Fast.gzip_level(), 3);
        assert_eq!(CompressionLevel::Default.gzip_level(), 6);
        assert_eq!(CompressionLevel::Best.gzip_level(), 9);
        assert_eq!(CompressionLevel::Custom(5).gzip_level(), 5);
        assert_eq!(CompressionLevel::Custom(20).gzip_level(), 9); // Clamped
    }

    #[test]
    fn test_compression_level_brotli() {
        assert_eq!(CompressionLevel::Fastest.brotli_level(), 1);
        assert_eq!(CompressionLevel::Fast.brotli_level(), 4);
        assert_eq!(CompressionLevel::Default.brotli_level(), 6);
        assert_eq!(CompressionLevel::Best.brotli_level(), 11);
        assert_eq!(CompressionLevel::Custom(8).brotli_level(), 8);
        assert_eq!(CompressionLevel::Custom(20).brotli_level(), 11); // Clamped
    }

    #[test]
    fn test_parse_accept_encoding_simple() {
        let encodings = ContentEncoding::parse_accept_encoding("gzip, deflate, br");
        assert_eq!(encodings.len(), 3);

        // Should be sorted by quality score (brotli > gzip > deflate)
        assert_eq!(encodings[0].algorithm, CompressionAlgorithm::Brotli);
        assert_eq!(encodings[1].algorithm, CompressionAlgorithm::Gzip);
        assert_eq!(encodings[2].algorithm, CompressionAlgorithm::Deflate);
    }

    #[test]
    fn test_parse_accept_encoding_with_quality() {
        let encodings = ContentEncoding::parse_accept_encoding("gzip;q=0.8, br;q=1.0, deflate;q=0.5");
        assert_eq!(encodings.len(), 3);

        // Should be sorted by quality first
        assert_eq!(encodings[0].algorithm, CompressionAlgorithm::Brotli);
        assert_eq!(encodings[0].quality, 1.0);

        assert_eq!(encodings[1].algorithm, CompressionAlgorithm::Gzip);
        assert_eq!(encodings[1].quality, 0.8);

        assert_eq!(encodings[2].algorithm, CompressionAlgorithm::Deflate);
        assert_eq!(encodings[2].quality, 0.5);
    }

    #[test]
    fn test_select_best_encoding() {
        let supported = vec![CompressionAlgorithm::Gzip, CompressionAlgorithm::Brotli];

        let best = ContentEncoding::select_best("gzip, br", &supported);
        assert_eq!(best, Some(CompressionAlgorithm::Brotli)); // Higher quality score

        let best = ContentEncoding::select_best("gzip;q=1.0, br;q=0.5", &supported);
        assert_eq!(best, Some(CompressionAlgorithm::Gzip)); // Higher quality factor
    }

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats::new(1000, 300, CompressionAlgorithm::Gzip, 10.5);

        assert_eq!(stats.original_size, 1000);
        assert_eq!(stats.compressed_size, 300);
        assert_eq!(stats.compression_ratio, 0.3);
        assert_eq!(stats.bytes_saved, 700);
        assert_eq!(stats.compression_percentage(), 70.0);
        assert_eq!(stats.duration_ms, 10.5);
    }
}
