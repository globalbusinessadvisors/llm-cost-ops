// Compression configuration

use super::{CompressionAlgorithm, CompressionLevel};
use serde::{Deserialize, Serialize};

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,

    /// Default compression level
    pub level: CompressionLevel,

    /// Supported algorithms (in order of preference)
    pub algorithms: Vec<CompressionAlgorithm>,

    /// Minimum size in bytes to compress (don't compress small responses)
    pub min_size: usize,

    /// Maximum size in bytes to compress (avoid compressing huge responses)
    pub max_size: Option<usize>,

    /// MIME types to compress
    pub mime_types: Vec<String>,

    /// Enable compression for requests
    pub compress_requests: bool,

    /// Enable compression for responses
    pub compress_responses: bool,

    /// Buffer size for streaming compression
    pub buffer_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: CompressionLevel::Default,
            algorithms: vec![
                CompressionAlgorithm::Brotli,
                CompressionAlgorithm::Gzip,
                CompressionAlgorithm::Deflate,
            ],
            min_size: 1024, // 1 KB
            max_size: Some(10 * 1024 * 1024), // 10 MB
            mime_types: vec![
                "text/*".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
                "application/javascript".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ],
            compress_requests: false,
            compress_responses: true,
            buffer_size: 8192, // 8 KB
        }
    }
}

impl CompressionConfig {
    /// Create a production configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            level: CompressionLevel::Default,
            algorithms: vec![
                CompressionAlgorithm::Brotli,
                CompressionAlgorithm::Gzip,
            ],
            min_size: 1024, // 1 KB
            max_size: Some(50 * 1024 * 1024), // 50 MB
            mime_types: vec![
                "text/*".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
                "application/javascript".to_string(),
            ],
            compress_requests: false,
            compress_responses: true,
            buffer_size: 16384, // 16 KB
        }
    }

    /// Create a development configuration
    pub fn development() -> Self {
        Self {
            enabled: true,
            level: CompressionLevel::Fastest,
            algorithms: vec![CompressionAlgorithm::Gzip],
            min_size: 512, // 512 bytes
            max_size: Some(5 * 1024 * 1024), // 5 MB
            mime_types: vec!["text/*".to_string(), "application/json".to_string()],
            compress_requests: false,
            compress_responses: true,
            buffer_size: 4096, // 4 KB
        }
    }

    /// Create a disabled configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create a high compression configuration
    pub fn high_compression() -> Self {
        Self {
            enabled: true,
            level: CompressionLevel::Best,
            algorithms: vec![CompressionAlgorithm::Brotli],
            min_size: 256, // 256 bytes
            max_size: None, // No limit
            mime_types: vec![
                "text/*".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
                "application/javascript".to_string(),
                "application/octet-stream".to_string(),
            ],
            compress_requests: true,
            compress_responses: true,
            buffer_size: 32768, // 32 KB
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.algorithms.is_empty() {
            return Err("At least one compression algorithm must be specified".to_string());
        }

        if self.min_size == 0 {
            return Err("Minimum size must be greater than 0".to_string());
        }

        if let Some(max_size) = self.max_size {
            if max_size < self.min_size {
                return Err("Maximum size must be greater than minimum size".to_string());
            }
        }

        if self.buffer_size == 0 {
            return Err("Buffer size must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Check if content type should be compressed
    pub fn should_compress_mime_type(&self, content_type: &str) -> bool {
        if !self.enabled {
            return false;
        }

        for pattern in &self.mime_types {
            if pattern.ends_with("/*") {
                let prefix = pattern.trim_end_matches("/*");
                if content_type.starts_with(prefix) {
                    return true;
                }
            } else if content_type.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Check if size should be compressed
    pub fn should_compress_size(&self, size: usize) -> bool {
        if !self.enabled {
            return false;
        }

        if size < self.min_size {
            return false;
        }

        if let Some(max_size) = self.max_size {
            if size > max_size {
                return false;
            }
        }

        true
    }

    /// Select best algorithm from Accept-Encoding header
    pub fn select_algorithm(&self, accept_encoding: Option<&str>) -> Option<CompressionAlgorithm> {
        if !self.enabled {
            return None;
        }

        let accept_encoding = accept_encoding?;

        super::types::ContentEncoding::select_best(accept_encoding, &self.algorithms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CompressionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_size, 1024);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_production_config() {
        let config = CompressionConfig::production();
        assert!(config.enabled);
        assert_eq!(config.algorithms.len(), 2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_development_config() {
        let config = CompressionConfig::development();
        assert!(config.enabled);
        assert_eq!(config.level, CompressionLevel::Fastest);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_disabled_config() {
        let config = CompressionConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_high_compression_config() {
        let config = CompressionConfig::high_compression();
        assert_eq!(config.level, CompressionLevel::Best);
        assert!(config.compress_requests);
        assert!(config.compress_responses);
    }

    #[test]
    fn test_should_compress_mime_type() {
        let config = CompressionConfig::default();

        assert!(config.should_compress_mime_type("text/html"));
        assert!(config.should_compress_mime_type("text/plain"));
        assert!(config.should_compress_mime_type("application/json"));
        assert!(!config.should_compress_mime_type("image/png"));
        assert!(!config.should_compress_mime_type("video/mp4"));
    }

    #[test]
    fn test_should_compress_size() {
        let config = CompressionConfig::default();

        assert!(!config.should_compress_size(512)); // Too small
        assert!(config.should_compress_size(2048)); // Just right
        assert!(!config.should_compress_size(20 * 1024 * 1024)); // Too large
    }

    #[test]
    fn test_select_algorithm() {
        let config = CompressionConfig::default();

        let algo = config.select_algorithm(Some("gzip, deflate, br"));
        assert_eq!(algo, Some(CompressionAlgorithm::Brotli));

        let algo = config.select_algorithm(Some("gzip"));
        assert_eq!(algo, Some(CompressionAlgorithm::Gzip));

        let algo = config.select_algorithm(None);
        assert_eq!(algo, None);
    }

    #[test]
    fn test_validate_config() {
        let mut config = CompressionConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Empty algorithms
        config.algorithms.clear();
        assert!(config.validate().is_err());

        // Reset
        config = CompressionConfig::default();

        // min_size = 0
        config.min_size = 0;
        assert!(config.validate().is_err());

        // Reset
        config = CompressionConfig::default();

        // max_size < min_size
        config.max_size = Some(100);
        config.min_size = 1000;
        assert!(config.validate().is_err());
    }
}
