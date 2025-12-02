# Benchmark Command Documentation

The `cost-ops run` command executes performance benchmarks for the LLM Cost Operations platform.

## Usage

```bash
# Run all benchmarks with default settings
cost-ops run

# Or using cargo
cargo run -- run

# Specify custom output directory
cost-ops run --output ./my-benchmarks

# Skip summary report generation
cost-ops run --no-summary

# Filter benchmarks by name
cost-ops run --filter cost_calculation
cost-ops run --filter aggregation
```

## Command Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--output` | `-o` | `benchmarks/output` | Output directory for benchmark results |
| `--no-summary` | - | false | Skip generating summary.md report |
| `--filter` | `-f` | none | Filter benchmarks by name substring |

## Output Structure

The command generates the following output structure:

```
benchmarks/output/
├── raw/
│   └── benchmark-results-{timestamp}.json  # Detailed JSON results
└── summary.md                               # Human-readable summary
```

### JSON Output Format

The JSON file contains:
- **version**: LLM-CostOps version used
- **timestamp**: When benchmarks were run
- **results**: Array of individual benchmark results
  - `name`: Benchmark name
  - `iterations`: Number of iterations run
  - `total_duration_ms`: Total time taken
  - `avg_duration_us`: Average duration per operation in microseconds
  - `min_duration_us`: Fastest operation time
  - `max_duration_us`: Slowest operation time
  - `throughput_per_sec`: Operations per second
- **summary**: Overall statistics
  - `total_benchmarks`: Number of benchmarks run
  - `total_iterations`: Total operations performed
  - `total_duration_ms`: Total execution time
  - `fastest_benchmark`: Name of fastest benchmark
  - `slowest_benchmark`: Name of slowest benchmark

### Markdown Summary

The `summary.md` file provides:
- Overall statistics table
- Detailed results with min/max/avg metrics
- Performance analysis grouped by benchmark type
- Top performers highlighted

## Available Benchmarks

The following benchmarks are executed:

### Cost Calculation Benchmarks
- **single_cost_calculation**: Basic cost calculation for a single usage record
- **batch_cost_calculation_100**: Batch processing of 100 records
- **batch_cost_calculation_1000**: Batch processing of 1,000 records
- **batch_cost_calculation_10000**: Batch processing of 10,000 records
- **cached_token_calculation**: Cost calculation with prompt caching

### Aggregation Benchmarks
- **cost_aggregation_100**: Aggregating 100 cost records
- **cost_aggregation_1000**: Aggregating 1,000 cost records
- **cost_aggregation_10000**: Aggregating 10,000 cost records

### Validation Benchmarks
- **usage_validation**: Usage record validation

### Provider Benchmarks
- **multi_provider_openai**: OpenAI-specific calculation
- **multi_provider_anthropic**: Anthropic-specific calculation
- **multi_provider_googlevertexai**: Google Vertex AI calculation
- **multi_provider_azureopenai**: Azure OpenAI calculation

## Filtering Examples

```bash
# Run only cost calculation benchmarks
cost-ops run --filter cost_calculation

# Run only aggregation benchmarks
cost-ops run --filter aggregation

# Run only multi-provider benchmarks
cost-ops run --filter multi_provider

# Run only batch processing benchmarks
cost-ops run --filter batch
```

## Example Output

Console output during execution:

```
INFO Running benchmark: single_cost_calculation
INFO Running benchmark: batch_cost_calculation_100
INFO Running benchmark: batch_cost_calculation_1000
...

================================================================================
BENCHMARK RESULTS SUMMARY
================================================================================

Version: 0.1.0
Timestamp: 2025-12-02 10:30:45 UTC

Overall Statistics:
  Total Benchmarks: 12
  Total Iterations: 130000
  Total Duration: 1234.56 ms
  Fastest: single_cost_calculation
  Slowest: batch_cost_calculation_10000

Top 5 Fastest Operations:
  1. single_cost_calculation - 0.95 μs/op (1052631 ops/sec)
  2. usage_validation - 1.23 μs/op (813008 ops/sec)
  3. cached_token_calculation - 1.45 μs/op (689655 ops/sec)
  4. multi_provider_openai - 1.67 μs/op (598802 ops/sec)
  5. multi_provider_anthropic - 1.71 μs/op (584795 ops/sec)

================================================================================
```

## Performance Interpretation

- **μs/op (microseconds per operation)**: Lower is better. Measures average time per operation.
- **ops/sec (operations per second)**: Higher is better. Measures throughput.
- **Batch benchmarks**: Show scalability characteristics. Compare throughput across different batch sizes.

## Integration with CI/CD

The benchmark command can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run Benchmarks
  run: |
    cargo run -- run --output ./benchmark-results

- name: Upload Results
  uses: actions/upload-artifact@v3
  with:
    name: benchmark-results
    path: benchmark-results/
```

## Troubleshooting

### Common Issues

**Output directory not found:**
The command will automatically create the output directory if it doesn't exist.

**Permission errors:**
Ensure the user has write permissions to the specified output directory.

**Out of memory:**
For large-scale benchmarks, ensure sufficient system memory. The benchmarks are designed to be memory-efficient but may require 100-200MB for large batch operations.

## Performance Baselines

Expected performance on modern hardware (2024 laptop):
- Single cost calculation: ~1-2 μs/op
- Batch 1000 records: ~1000-2000 μs total
- Cost aggregation 1000 records: ~500-1000 μs

Actual performance will vary based on:
- CPU speed and architecture
- Memory speed
- System load
- Rust optimization level (debug vs release builds)

**Note**: For accurate performance measurements, always run benchmarks in `--release` mode:

```bash
cargo run --release -- run
```
