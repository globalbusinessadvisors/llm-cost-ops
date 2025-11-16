// Compression/decompression codec implementation

use super::{CompressionAlgorithm, CompressionError, CompressionLevel, CompressionResult, CompressionStats};
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use std::io::{Read, Write};
use std::time::Instant;

/// Compressor trait for different algorithms
pub trait Compressor: Send + Sync {
    /// Compress data
    fn compress(&self, data: &[u8], level: CompressionLevel) -> CompressionResult<Vec<u8>>;

    /// Decompress data
    fn decompress(&self, data: &[u8]) -> CompressionResult<Vec<u8>>;

    /// Get algorithm
    fn algorithm(&self) -> CompressionAlgorithm;
}

/// Gzip compressor
#[derive(Debug, Clone, Default)]
pub struct GzipCompressor;

impl Compressor for GzipCompressor {
    fn compress(&self, data: &[u8], level: CompressionLevel) -> CompressionResult<Vec<u8>> {
        let compression_level = flate2::Compression::new(level.gzip_level());
        let mut encoder = GzEncoder::new(Vec::new(), compression_level);

        encoder
            .write_all(data)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
    }

    fn decompress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

        Ok(decompressed)
    }

    fn algorithm(&self) -> CompressionAlgorithm {
        CompressionAlgorithm::Gzip
    }
}

/// Brotli compressor
#[derive(Debug, Clone, Default)]
pub struct BrotliCompressor;

impl Compressor for BrotliCompressor {
    fn compress(&self, data: &[u8], level: CompressionLevel) -> CompressionResult<Vec<u8>> {
        let quality = level.brotli_level() as i32;
        let mut output = Vec::new();

        brotli::BrotliCompress(
            &mut std::io::Cursor::new(data),
            &mut output,
            &brotli::enc::BrotliEncoderParams {
                quality,
                ..Default::default()
            },
        )
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

        Ok(output)
    }

    fn decompress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        let mut output = Vec::new();

        brotli::BrotliDecompress(&mut std::io::Cursor::new(data), &mut output)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

        Ok(output)
    }

    fn algorithm(&self) -> CompressionAlgorithm {
        CompressionAlgorithm::Brotli
    }
}

/// Deflate compressor
#[derive(Debug, Clone, Default)]
pub struct DeflateCompressor;

impl Compressor for DeflateCompressor {
    fn compress(&self, data: &[u8], level: CompressionLevel) -> CompressionResult<Vec<u8>> {
        let compression_level = flate2::Compression::new(level.gzip_level());
        let mut encoder = ZlibEncoder::new(Vec::new(), compression_level);

        encoder
            .write_all(data)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
    }

    fn decompress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

        Ok(decompressed)
    }

    fn algorithm(&self) -> CompressionAlgorithm {
        CompressionAlgorithm::Deflate
    }
}

/// Get compressor for algorithm
pub fn get_compressor(algorithm: CompressionAlgorithm) -> CompressionResult<Box<dyn Compressor>> {
    match algorithm {
        CompressionAlgorithm::Gzip => Ok(Box::new(GzipCompressor)),
        CompressionAlgorithm::Brotli => Ok(Box::new(BrotliCompressor)),
        CompressionAlgorithm::Deflate => Ok(Box::new(DeflateCompressor)),
        CompressionAlgorithm::Identity => Err(CompressionError::UnsupportedAlgorithm(
            "Identity encoding does not require compression".to_string(),
        )),
    }
}

/// Compress data with specified algorithm and level
pub fn compress(
    data: &[u8],
    algorithm: CompressionAlgorithm,
    level: CompressionLevel,
) -> CompressionResult<(Vec<u8>, CompressionStats)> {
    if algorithm == CompressionAlgorithm::Identity {
        return Ok((
            data.to_vec(),
            CompressionStats::new(data.len(), data.len(), algorithm, 0.0),
        ));
    }

    let start = Instant::now();
    let compressor = get_compressor(algorithm)?;
    let compressed = compressor.compress(data, level)?;
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    let stats = CompressionStats::new(data.len(), compressed.len(), algorithm, duration_ms);

    Ok((compressed, stats))
}

/// Decompress data with specified algorithm
pub fn decompress(
    data: &[u8],
    algorithm: CompressionAlgorithm,
) -> CompressionResult<(Vec<u8>, CompressionStats)> {
    if algorithm == CompressionAlgorithm::Identity {
        return Ok((
            data.to_vec(),
            CompressionStats::new(data.len(), data.len(), algorithm, 0.0),
        ));
    }

    let start = Instant::now();
    let compressor = get_compressor(algorithm)?;
    let decompressed = compressor.decompress(data)?;
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    let stats = CompressionStats::new(decompressed.len(), data.len(), algorithm, duration_ms);

    Ok((decompressed, stats))
}

/// Compress data with automatic algorithm selection
pub fn compress_auto(
    data: &[u8],
    level: CompressionLevel,
) -> CompressionResult<(Vec<u8>, CompressionAlgorithm, CompressionStats)> {
    // Try brotli first (best compression ratio)
    match compress(data, CompressionAlgorithm::Brotli, level) {
        Ok((compressed, stats)) => Ok((compressed, CompressionAlgorithm::Brotli, stats)),
        Err(_) => {
            // Fallback to gzip
            let (compressed, stats) = compress(data, CompressionAlgorithm::Gzip, level)?;
            Ok((compressed, CompressionAlgorithm::Gzip, stats))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATA: &str = "Hello, World! This is a test string that should compress well because it has repetition. Hello, World! This is a test string that should compress well because it has repetition.";

    #[test]
    fn test_gzip_compress_decompress() {
        let compressor = GzipCompressor;
        let data = TEST_DATA.as_bytes();

        let compressed = compressor.compress(data, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_brotli_compress_decompress() {
        let compressor = BrotliCompressor;
        let data = TEST_DATA.as_bytes();

        let compressed = compressor.compress(data, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_deflate_compress_decompress() {
        let compressor = DeflateCompressor;
        let data = TEST_DATA.as_bytes();

        let compressed = compressor.compress(data, CompressionLevel::Default).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_function() {
        let data = TEST_DATA.as_bytes();

        let (compressed, stats) =
            compress(data, CompressionAlgorithm::Gzip, CompressionLevel::Default).unwrap();

        assert!(compressed.len() < data.len());
        assert_eq!(stats.original_size, data.len());
        assert_eq!(stats.compressed_size, compressed.len());
        assert!(stats.compression_ratio < 1.0);
        assert_eq!(stats.algorithm, Some(CompressionAlgorithm::Gzip));
    }

    #[test]
    fn test_decompress_function() {
        let data = TEST_DATA.as_bytes();

        let (compressed, _) =
            compress(data, CompressionAlgorithm::Gzip, CompressionLevel::Default).unwrap();

        let (decompressed, stats) = decompress(&compressed, CompressionAlgorithm::Gzip).unwrap();

        assert_eq!(decompressed, data);
        assert_eq!(stats.original_size, data.len());
        assert_eq!(stats.compressed_size, compressed.len());
    }

    #[test]
    fn test_compression_levels() {
        // Use larger data to ensure compression overhead doesn't skew results
        let large_data = TEST_DATA.repeat(100);

        let (fast, _) =
            compress(large_data.as_bytes(), CompressionAlgorithm::Gzip, CompressionLevel::Fastest).unwrap();
        let (best, _) =
            compress(large_data.as_bytes(), CompressionAlgorithm::Gzip, CompressionLevel::Best).unwrap();

        // Best compression should produce smaller or equal output for large data
        // For small data, compression overhead can make "best" larger than "fastest"
        assert!(best.len() <= fast.len());
    }

    #[test]
    fn test_identity_compression() {
        let data = TEST_DATA.as_bytes();

        let (compressed, stats) =
            compress(data, CompressionAlgorithm::Identity, CompressionLevel::Default).unwrap();

        assert_eq!(compressed, data);
        assert_eq!(stats.compression_ratio, 1.0);
    }

    #[test]
    fn test_compress_auto() {
        let data = TEST_DATA.as_bytes();

        let (compressed, algorithm, stats) = compress_auto(data, CompressionLevel::Default).unwrap();

        assert!(compressed.len() < data.len());
        assert_eq!(algorithm, CompressionAlgorithm::Brotli);
        assert!(stats.compression_ratio < 1.0);
    }

    #[test]
    fn test_brotli_better_than_gzip() {
        let data = TEST_DATA.repeat(10).as_bytes().to_vec();

        let (gzip_compressed, _) =
            compress(&data, CompressionAlgorithm::Gzip, CompressionLevel::Best).unwrap();
        let (brotli_compressed, _) =
            compress(&data, CompressionAlgorithm::Brotli, CompressionLevel::Best).unwrap();

        // Brotli should generally produce smaller output
        assert!(brotli_compressed.len() <= gzip_compressed.len());
    }

    #[test]
    fn test_round_trip_all_algorithms() {
        let data = TEST_DATA.as_bytes();

        for algorithm in [
            CompressionAlgorithm::Gzip,
            CompressionAlgorithm::Brotli,
            CompressionAlgorithm::Deflate,
        ] {
            let (compressed, _) = compress(data, algorithm, CompressionLevel::Default).unwrap();
            let (decompressed, _) = decompress(&compressed, algorithm).unwrap();
            assert_eq!(decompressed, data, "Failed for algorithm: {:?}", algorithm);
        }
    }
}
