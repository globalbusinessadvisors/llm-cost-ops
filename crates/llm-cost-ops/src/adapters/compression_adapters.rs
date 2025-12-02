/// Benchmark adapters for compression operations
///
/// Provides benchmark targets for compression and decompression using different algorithms.

use super::{BenchTarget, calculate_stats, run_iterations};
use crate::benchmarks::result::BenchmarkResult;
use crate::compression::{Compressor, CompressionAlgorithm, CompressionLevel};

/// Benchmark: Gzip compression
pub struct GzipCompression {
    level: CompressionLevel,
    data_size: usize,
}

impl GzipCompression {
    pub fn new(level: CompressionLevel, data_size: usize) -> Self {
        Self { level, data_size }
    }

    fn generate_test_data(&self) -> Vec<u8> {
        // Generate compressible test data
        let base_pattern = b"The quick brown fox jumps over the lazy dog. ";
        let mut data = Vec::with_capacity(self.data_size);

        while data.len() < self.data_size {
            data.extend_from_slice(base_pattern);
        }

        data.truncate(self.data_size);
        data
    }
}

impl BenchTarget for GzipCompression {
    fn id(&self) -> String {
        format!("compression/gzip_{:?}_{}", self.level, self.data_size)
    }

    fn name(&self) -> String {
        format!("Gzip Compression ({:?}, {} bytes)", self.level, self.data_size)
    }

    fn category(&self) -> String {
        "compression".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let compressor = Compressor::new(CompressionAlgorithm::Gzip, self.level);
        let test_data = self.generate_test_data();

        let iterations = if self.data_size > 100_000 { 100 } else { 1000 };

        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = compressor.compress(&test_data);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        // Calculate compression metadata
        let compressed = compressor.compress(&test_data).unwrap_or_default();
        let compression_ratio = if !compressed.is_empty() {
            test_data.len() as f64 / compressed.len() as f64
        } else {
            0.0
        };

        let metadata = serde_json::json!({
            "original_size": test_data.len(),
            "compressed_size": compressed.len(),
            "compression_ratio": compression_ratio,
            "algorithm": "gzip",
            "level": format!("{:?}", self.level),
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Benchmark: Brotli compression
pub struct BrotliCompression {
    level: CompressionLevel,
    data_size: usize,
}

impl BrotliCompression {
    pub fn new(level: CompressionLevel, data_size: usize) -> Self {
        Self { level, data_size }
    }

    fn generate_test_data(&self) -> Vec<u8> {
        let base_pattern = b"The quick brown fox jumps over the lazy dog. ";
        let mut data = Vec::with_capacity(self.data_size);

        while data.len() < self.data_size {
            data.extend_from_slice(base_pattern);
        }

        data.truncate(self.data_size);
        data
    }
}

impl BenchTarget for BrotliCompression {
    fn id(&self) -> String {
        format!("compression/brotli_{:?}_{}", self.level, self.data_size)
    }

    fn name(&self) -> String {
        format!("Brotli Compression ({:?}, {} bytes)", self.level, self.data_size)
    }

    fn category(&self) -> String {
        "compression".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let compressor = Compressor::new(CompressionAlgorithm::Brotli, self.level);
        let test_data = self.generate_test_data();

        let iterations = if self.data_size > 100_000 { 100 } else { 1000 };

        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = compressor.compress(&test_data);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        let compressed = compressor.compress(&test_data).unwrap_or_default();
        let compression_ratio = if !compressed.is_empty() {
            test_data.len() as f64 / compressed.len() as f64
        } else {
            0.0
        };

        let metadata = serde_json::json!({
            "original_size": test_data.len(),
            "compressed_size": compressed.len(),
            "compression_ratio": compression_ratio,
            "algorithm": "brotli",
            "level": format!("{:?}", self.level),
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Benchmark: Gzip decompression
pub struct GzipDecompression {
    data_size: usize,
}

impl GzipDecompression {
    pub fn new(data_size: usize) -> Self {
        Self { data_size }
    }

    fn generate_test_data(&self) -> Vec<u8> {
        let base_pattern = b"The quick brown fox jumps over the lazy dog. ";
        let mut data = Vec::with_capacity(self.data_size);

        while data.len() < self.data_size {
            data.extend_from_slice(base_pattern);
        }

        data.truncate(self.data_size);
        data
    }
}

impl BenchTarget for GzipDecompression {
    fn id(&self) -> String {
        format!("compression/gzip_decompress_{}", self.data_size)
    }

    fn name(&self) -> String {
        format!("Gzip Decompression ({} bytes)", self.data_size)
    }

    fn category(&self) -> String {
        "compression".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let compressor = Compressor::new(CompressionAlgorithm::Gzip, CompressionLevel::Default);
        let test_data = self.generate_test_data();

        // Pre-compress the data
        let compressed = match compressor.compress(&test_data) {
            Ok(data) => data,
            Err(e) => {
                return BenchmarkResult::failure(
                    self.id(),
                    self.name(),
                    self.category(),
                    format!("Failed to compress test data: {}", e),
                );
            }
        };

        let iterations = if self.data_size > 100_000 { 100 } else { 1000 };

        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = compressor.decompress(&compressed);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        let metadata = serde_json::json!({
            "original_size": test_data.len(),
            "compressed_size": compressed.len(),
            "algorithm": "gzip",
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Create all compression benchmark targets
pub fn create_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        // Gzip compression at different levels and sizes
        Box::new(GzipCompression::new(CompressionLevel::Fastest, 1_024)),
        Box::new(GzipCompression::new(CompressionLevel::Default, 1_024)),
        Box::new(GzipCompression::new(CompressionLevel::Best, 1_024)),
        Box::new(GzipCompression::new(CompressionLevel::Default, 10_240)),
        Box::new(GzipCompression::new(CompressionLevel::Default, 102_400)),

        // Brotli compression at different levels and sizes
        Box::new(BrotliCompression::new(CompressionLevel::Fastest, 1_024)),
        Box::new(BrotliCompression::new(CompressionLevel::Default, 1_024)),
        Box::new(BrotliCompression::new(CompressionLevel::Best, 1_024)),
        Box::new(BrotliCompression::new(CompressionLevel::Default, 10_240)),
        Box::new(BrotliCompression::new(CompressionLevel::Default, 102_400)),

        // Decompression benchmarks
        Box::new(GzipDecompression::new(1_024)),
        Box::new(GzipDecompression::new(10_240)),
        Box::new(GzipDecompression::new(102_400)),
    ]
}
