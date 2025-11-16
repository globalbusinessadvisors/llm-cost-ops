/// Comprehensive compression tests
///
/// Tests all compression algorithms (Gzip, Brotli, Deflate)

use llm_cost_ops::compression::*;

const TEST_DATA: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
    Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.";

const SMALL_DATA: &[u8] = b"Hello";
const LARGE_DATA_SIZE: usize = 1_000_000;

// === Gzip Compression Tests ===

#[test]
fn test_gzip_compress_decompress() {
    let compressor = GzipCompressor::default();

    let compressed = compressor.compress(TEST_DATA, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, TEST_DATA);
    assert!(compressed.len() < TEST_DATA.len());
}

#[test]
fn test_gzip_compression_levels() {
    let compressor = GzipCompressor::default();

    let fast = compressor.compress(TEST_DATA, CompressionLevel::Fast).unwrap();
    let default = compressor.compress(TEST_DATA, CompressionLevel::Default).unwrap();
    let best = compressor.compress(TEST_DATA, CompressionLevel::Best).unwrap();

    // Best compression should produce smallest size
    assert!(best.len() <= default.len());
    assert!(default.len() <= fast.len());

    // All should decompress correctly
    assert_eq!(compressor.decompress(&fast).unwrap(), TEST_DATA);
    assert_eq!(compressor.decompress(&default).unwrap(), TEST_DATA);
    assert_eq!(compressor.decompress(&best).unwrap(), TEST_DATA);
}

#[test]
fn test_gzip_empty_data() {
    let compressor = GzipCompressor::default();
    let empty: &[u8] = &[];

    let compressed = compressor.compress(empty, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, empty);
}

#[test]
fn test_gzip_small_data() {
    let compressor = GzipCompressor::default();

    let compressed = compressor.compress(SMALL_DATA, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, SMALL_DATA);
    // Small data might not compress well
}

#[test]
fn test_gzip_large_data() {
    let compressor = GzipCompressor::default();
    let large_data = vec![b'A'; LARGE_DATA_SIZE];

    let compressed = compressor.compress(&large_data, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, large_data);
    assert!(compressed.len() < large_data.len() / 100); // Should compress very well (repetitive)
}

#[test]
fn test_gzip_invalid_data() {
    let compressor = GzipCompressor::default();
    let invalid_data = b"not compressed data";

    let result = compressor.decompress(invalid_data);
    assert!(result.is_err());
}

// === Brotli Compression Tests ===

#[test]
fn test_brotli_compress_decompress() {
    let compressor = BrotliCompressor::default();

    let compressed = compressor.compress(TEST_DATA, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, TEST_DATA);
    assert!(compressed.len() < TEST_DATA.len());
}

#[test]
fn test_brotli_compression_levels() {
    let compressor = BrotliCompressor::default();

    let fast = compressor.compress(TEST_DATA, CompressionLevel::Fast).unwrap();
    let default = compressor.compress(TEST_DATA, CompressionLevel::Default).unwrap();
    let best = compressor.compress(TEST_DATA, CompressionLevel::Best).unwrap();

    // Best compression should produce smallest size
    assert!(best.len() <= default.len());

    // All should decompress correctly
    assert_eq!(compressor.decompress(&fast).unwrap(), TEST_DATA);
    assert_eq!(compressor.decompress(&default).unwrap(), TEST_DATA);
    assert_eq!(compressor.decompress(&best).unwrap(), TEST_DATA);
}

#[test]
fn test_brotli_empty_data() {
    let compressor = BrotliCompressor::default();
    let empty: &[u8] = &[];

    let compressed = compressor.compress(empty, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, empty);
}

#[test]
fn test_brotli_large_data() {
    let compressor = BrotliCompressor::default();
    let large_data = vec![b'B'; LARGE_DATA_SIZE];

    let compressed = compressor.compress(&large_data, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, large_data);
    assert!(compressed.len() < large_data.len() / 100);
}

// === Deflate Compression Tests ===

#[test]
fn test_deflate_compress_decompress() {
    let compressor = DeflateCompressor::default();

    let compressed = compressor.compress(TEST_DATA, CompressionLevel::Default).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();

    assert_eq!(decompressed, TEST_DATA);
    assert!(compressed.len() < TEST_DATA.len());
}

#[test]
fn test_deflate_compression_levels() {
    let compressor = DeflateCompressor::default();

    let fast = compressor.compress(TEST_DATA, CompressionLevel::Fast).unwrap();
    let default = compressor.compress(TEST_DATA, CompressionLevel::Default).unwrap();
    let best = compressor.compress(TEST_DATA, CompressionLevel::Best).unwrap();

    // All should decompress correctly
    assert_eq!(compressor.decompress(&fast).unwrap(), TEST_DATA);
    assert_eq!(compressor.decompress(&default).unwrap(), TEST_DATA);
    assert_eq!(compressor.decompress(&best).unwrap(), TEST_DATA);
}

// === Compression Codec Tests ===

#[test]
fn test_codec_factory() {
    let gzip = CompressionCodec::new(CompressionAlgorithm::Gzip);
    let brotli = CompressionCodec::new(CompressionAlgorithm::Brotli);
    let deflate = CompressionCodec::new(CompressionAlgorithm::Deflate);

    assert_eq!(gzip.algorithm(), CompressionAlgorithm::Gzip);
    assert_eq!(brotli.algorithm(), CompressionAlgorithm::Brotli);
    assert_eq!(deflate.algorithm(), CompressionAlgorithm::Deflate);
}

#[test]
fn test_codec_compress_with_stats() {
    let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);

    let (compressed, stats) = codec.compress_with_stats(TEST_DATA, CompressionLevel::Default).unwrap();

    assert_eq!(stats.original_size, TEST_DATA.len());
    assert_eq!(stats.compressed_size, compressed.len());
    assert_eq!(stats.algorithm, CompressionAlgorithm::Gzip);
    assert!(stats.compression_time_ms > 0);

    let ratio = stats.compression_ratio();
    assert!(ratio > 0.0 && ratio < 1.0);
}

#[test]
fn test_compression_stats_ratio() {
    let stats = CompressionStats {
        algorithm: CompressionAlgorithm::Gzip,
        original_size: 1000,
        compressed_size: 300,
        compression_time_ms: 10,
        decompression_time_ms: 5,
    };

    let ratio = stats.compression_ratio();
    assert_eq!(ratio, 0.3);

    let space_savings = stats.space_savings_percent();
    assert_eq!(space_savings, 70.0);
}

// === Algorithm Comparison Tests ===

#[test]
fn test_algorithm_comparison() {
    let algorithms = vec![
        CompressionAlgorithm::Gzip,
        CompressionAlgorithm::Brotli,
        CompressionAlgorithm::Deflate,
    ];

    let data = vec![b'X'; 10000];

    for algo in algorithms {
        let codec = CompressionCodec::new(algo);
        let compressed = codec.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = codec.decompress(&compressed).unwrap();

        assert_eq!(decompressed, data);
        assert!(compressed.len() < data.len());
    }
}

#[test]
fn test_best_compression_ratio() {
    let data = TEST_DATA.repeat(100); // Make it larger

    let gzip_codec = CompressionCodec::new(CompressionAlgorithm::Gzip);
    let brotli_codec = CompressionCodec::new(CompressionAlgorithm::Brotli);

    let gzip_compressed = gzip_codec.compress(&data, CompressionLevel::Best).unwrap();
    let brotli_compressed = brotli_codec.compress(&data, CompressionLevel::Best).unwrap();

    // Brotli typically achieves better compression
    // Note: This might not always be true for all data patterns
    assert!(brotli_compressed.len() <= gzip_compressed.len() * 11 / 10); // Within 10%
}

// === Performance Tests ===

#[test]
fn test_compression_speed_fast_level() {
    let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);
    let data = vec![b'F'; 100_000];

    let start = std::time::Instant::now();
    let _ = codec.compress(&data, CompressionLevel::Fast).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "Fast compression too slow: {:?}", elapsed);
}

#[test]
fn test_decompression_speed() {
    let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);
    let data = vec![b'D'; 100_000];

    let compressed = codec.compress(&data, CompressionLevel::Default).unwrap();

    let start = std::time::Instant::now();
    let _ = codec.decompress(&compressed).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 50, "Decompression too slow: {:?}", elapsed);
}

#[test]
fn test_bulk_compression() {
    let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);

    let start = std::time::Instant::now();

    for i in 0..100 {
        let data = format!("Test data {}", i).as_bytes().to_vec();
        let compressed = codec.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = codec.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "Bulk compression too slow: {:?}", elapsed);
}

// === Edge Cases ===

#[test]
fn test_incompressible_data() {
    let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);

    // Random-like data that doesn't compress well
    let data: Vec<u8> = (0..1000).map(|i| (i * 37 % 256) as u8).collect();

    let compressed = codec.compress(&data, CompressionLevel::Best).unwrap();
    let decompressed = codec.decompress(&compressed).unwrap();

    assert_eq!(decompressed, data);
    // Compressed might be larger due to overhead
}

#[test]
fn test_highly_compressible_data() {
    let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);

    let data = vec![0u8; 100_000]; // All zeros

    let compressed = codec.compress(&data, CompressionLevel::Best).unwrap();
    let decompressed = codec.decompress(&compressed).unwrap();

    assert_eq!(decompressed, data);
    assert!(compressed.len() < data.len() / 100); // Should compress to < 1%
}

#[test]
fn test_compression_level_enum() {
    assert_eq!(CompressionLevel::Fast.gzip_level(), 1);
    assert_eq!(CompressionLevel::Default.gzip_level(), 6);
    assert_eq!(CompressionLevel::Best.gzip_level(), 9);

    assert_eq!(CompressionLevel::Fast.brotli_level(), 1);
    assert_eq!(CompressionLevel::Default.brotli_level(), 6);
    assert_eq!(CompressionLevel::Best.brotli_level(), 11);
}
