/// File I/O operations for benchmark results
///
/// Handles reading and writing benchmark results to disk in various formats.

use super::result::{BenchmarkResult, BenchmarkSummary};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

/// Error type for I/O operations
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkIoError {
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(String),

    #[error("Failed to write file: {0}")]
    FileWrite(String),

    #[error("Failed to read file: {0}")]
    FileRead(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type IoResult<T> = Result<T, BenchmarkIoError>;

/// Benchmark I/O handler
pub struct BenchmarkIo {
    output_dir: PathBuf,
    raw_dir: PathBuf,
}

impl BenchmarkIo {
    /// Create a new I/O handler with the specified output directory
    pub fn new<P: AsRef<Path>>(output_dir: P) -> IoResult<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();
        let raw_dir = output_dir.join("raw");

        // Create directories if they don't exist
        fs::create_dir_all(&output_dir)?;
        fs::create_dir_all(&raw_dir)?;

        Ok(Self {
            output_dir,
            raw_dir,
        })
    }

    /// Write a single benchmark result to a JSON file
    pub fn write_result(&self, result: &BenchmarkResult) -> IoResult<PathBuf> {
        let filename = format!("{}_{}.json",
            result.category.replace('/', "_"),
            result.id.replace('/', "_")
        );
        let path = self.raw_dir.join(filename);

        let json = serde_json::to_string_pretty(result)
            .map_err(|e| BenchmarkIoError::Serialization(e.to_string()))?;

        let mut file = fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;

        Ok(path)
    }

    /// Write all results to individual JSON files
    pub fn write_all_results(&self, results: &[BenchmarkResult]) -> IoResult<Vec<PathBuf>> {
        let mut paths = Vec::new();

        for result in results {
            let path = self.write_result(result)?;
            paths.push(path);
        }

        Ok(paths)
    }

    /// Write summary to JSON file
    pub fn write_summary(&self, summary: &BenchmarkSummary) -> IoResult<PathBuf> {
        let path = self.output_dir.join("summary.json");

        let json = serde_json::to_string_pretty(summary)
            .map_err(|e| BenchmarkIoError::Serialization(e.to_string()))?;

        let mut file = fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;

        Ok(path)
    }

    /// Read a benchmark result from a JSON file
    pub fn read_result<P: AsRef<Path>>(&self, path: P) -> IoResult<BenchmarkResult> {
        let content = fs::read_to_string(path)?;
        let result = serde_json::from_str(&content)
            .map_err(|e| BenchmarkIoError::Deserialization(e.to_string()))?;

        Ok(result)
    }

    /// Read all benchmark results from the raw directory
    pub fn read_all_results(&self) -> IoResult<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        for entry in fs::read_dir(&self.raw_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.read_result(&path) {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        eprintln!("Warning: Failed to read {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Read summary from JSON file
    pub fn read_summary(&self) -> IoResult<BenchmarkSummary> {
        let path = self.output_dir.join("summary.json");
        let content = fs::read_to_string(&path)?;
        let summary = serde_json::from_str(&content)
            .map_err(|e| BenchmarkIoError::Deserialization(e.to_string()))?;

        Ok(summary)
    }

    /// Clear all existing benchmark results
    pub fn clear_results(&self) -> IoResult<()> {
        if self.raw_dir.exists() {
            fs::remove_dir_all(&self.raw_dir)?;
            fs::create_dir_all(&self.raw_dir)?;
        }

        Ok(())
    }

    /// Get the output directory path
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Get the raw results directory path
    pub fn raw_dir(&self) -> &Path {
        &self.raw_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::time::Duration;

    #[test]
    fn test_create_io_handler() {
        let temp_dir = TempDir::new().unwrap();
        let io = BenchmarkIo::new(temp_dir.path()).unwrap();

        assert!(io.output_dir().exists());
        assert!(io.raw_dir().exists());
    }

    #[test]
    fn test_write_and_read_result() {
        let temp_dir = TempDir::new().unwrap();
        let io = BenchmarkIo::new(temp_dir.path()).unwrap();

        let result = BenchmarkResult::success(
            "test-1".to_string(),
            "Test Benchmark".to_string(),
            "engine".to_string(),
            Duration::from_secs(1),
            1000,
        );

        let path = io.write_result(&result).unwrap();
        assert!(path.exists());

        let read_result = io.read_result(&path).unwrap();
        assert_eq!(read_result.id, result.id);
        assert_eq!(read_result.name, result.name);
    }

    #[test]
    fn test_write_and_read_summary() {
        let temp_dir = TempDir::new().unwrap();
        let io = BenchmarkIo::new(temp_dir.path()).unwrap();

        let results = vec![
            BenchmarkResult::success(
                "1".to_string(),
                "Test 1".to_string(),
                "engine".to_string(),
                Duration::from_secs(1),
                100,
            ),
        ];

        let summary = BenchmarkSummary::from_results(results);
        let path = io.write_summary(&summary).unwrap();
        assert!(path.exists());

        let read_summary = io.read_summary().unwrap();
        assert_eq!(read_summary.total_count, summary.total_count);
    }

    #[test]
    fn test_clear_results() {
        let temp_dir = TempDir::new().unwrap();
        let io = BenchmarkIo::new(temp_dir.path()).unwrap();

        let result = BenchmarkResult::success(
            "test".to_string(),
            "Test".to_string(),
            "engine".to_string(),
            Duration::from_secs(1),
            100,
        );

        io.write_result(&result).unwrap();
        assert!(!io.read_all_results().unwrap().is_empty());

        io.clear_results().unwrap();
        assert!(io.read_all_results().unwrap().is_empty());
    }
}
