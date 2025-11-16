# Testing Quick Reference Guide

## Running Tests

### All Tests
```bash
# Run all tests (unit + integration)
cargo test --all-features

# Run with output
cargo test --all-features -- --nocapture

# Run specific test
cargo test test_cost_calculation --all-features

# Run tests in specific file
cargo test --test integration_tests
```

### Unit Tests Only
```bash
# Run library unit tests
cargo test --lib --all-features

# Run specific module
cargo test domain:: --lib
```

### Integration Tests
```bash
# All integration tests
cargo test --test '*'

# Specific integration test file
cargo test --test integration_tests
cargo test --test property_tests
cargo test --test mock_server
```

### Property-Based Tests
```bash
# Run with default 100 cases
cargo test --test property_tests

# Run with more cases for thorough testing
PROPTEST_CASES=10000 cargo test --test property_tests
```

### Security Tests
```bash
# Run security test suite
cargo test --test security_tests
cargo test --test security_comprehensive_tests

# Run auth-specific tests
cargo test auth:: --lib
```

## Code Coverage

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage (HTML + XML)
cargo tarpaulin \
    --all-features \
    --out Html \
    --out Xml \
    --output-dir ./coverage \
    --exclude-files "tests/*" "benches/*"

# Open HTML report
open coverage/index.html
```

### Coverage by Module
```bash
# Domain coverage
cargo tarpaulin --lib --packages llm-cost-ops -- domain::

# Engine coverage
cargo tarpaulin --lib --packages llm-cost-ops -- engine::
```

## Benchmarks

### Run All Benchmarks
```bash
# Run all performance benchmarks
cargo bench

# Run specific benchmark
cargo bench cost_calculation

# Save baseline for comparison
cargo bench -- --save-baseline main
```

### Compare Performance
```bash
# Save current as baseline
cargo bench -- --save-baseline pr-123

# Compare against baseline
cargo bench -- --baseline pr-123
```

### View Results
```bash
# Open criterion HTML report
open target/criterion/report/index.html
```

## Code Quality

### Linting
```bash
# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix where possible
cargo clippy --fix --all-targets --all-features
```

### Formatting
```bash
# Check formatting
cargo fmt --all -- --check

# Auto-format
cargo fmt --all
```

### Documentation
```bash
# Build and open docs
cargo doc --no-deps --all-features --open

# Check for broken links
cargo doc --no-deps --all-features
```

## Security

### Dependency Audit
```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit

# Fix vulnerabilities
cargo audit fix
```

### Dependency Policy Check
```bash
# Install cargo-deny
cargo install cargo-deny

# Check dependencies
cargo deny check
```

## Database Tests

### SQLite Tests
```bash
# Run with in-memory SQLite
cargo test --features sqlite --test integration_tests
```

### PostgreSQL Tests
```bash
# Start PostgreSQL (Docker)
docker run -d \
    --name postgres-test \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB=testdb \
    -p 5432:5432 \
    postgres:15

# Run tests
DATABASE_URL=postgres://postgres:postgres@localhost:5432/testdb \
    cargo test --features postgres --test integration_tests

# Cleanup
docker stop postgres-test && docker rm postgres-test
```

## Continuous Integration

### Run CI Checks Locally
```bash
# Simulate CI pipeline
./scripts/ci-check.sh

# Or manually:
cargo fmt --all -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all-features && \
cargo bench --no-run
```

### Pre-commit Hook
```bash
# Install pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
EOF

chmod +x .git/hooks/pre-commit
```

## Debugging Tests

### Run Single Test with Debugging
```bash
# With full output
cargo test test_name -- --nocapture --test-threads=1

# With environment variables
RUST_LOG=debug cargo test test_name -- --nocapture

# With backtraces
RUST_BACKTRACE=full cargo test test_name
```

### Debug Failing Test
```bash
# Run test with debugger
rust-lldb -- cargo test test_name --no-run
# Then in lldb:
# > run --test-threads=1
```

## Test Data

### Generate Test Data
```bash
# Run test data generator
cargo run --example generate_test_data

# Use in tests
cargo test --test integration_tests -- --ignored
```

## Mock Server

### Start Mock Server
```bash
# Start standalone mock server
cargo test --test mock_server -- --ignored test_mock_server_starts
```

### Configure Mock Responses
```rust
// In your test
let (state, addr) = start_mock_server().await;
state.set_response("/endpoint", MockResponse {
    status_code: 200,
    body: json!({"data": "test"}),
    headers: HashMap::new(),
});
```

## Common Test Patterns

### Testing Async Functions
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Testing Errors
```rust
#[test]
fn test_error_case() {
    let result = function_that_fails();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Expected error message"
    );
}
```

### Testing Panics
```rust
#[test]
#[should_panic(expected = "panic message")]
fn test_panic() {
    panic!("panic message");
}
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(x in 0u64..1000) {
        prop_assert!(my_function(x) >= 0);
    }
}
```

## Performance Profiling

### CPU Profiling
```bash
# Install flamegraph
cargo install flamegraph

# Profile tests
cargo flamegraph --test integration_tests

# Open flamegraph.svg
```

### Memory Profiling
```bash
# Install heaptrack
# Ubuntu: sudo apt-get install heaptrack

# Profile memory
heaptrack cargo test --test integration_tests

# Analyze
heaptrack_gui heaptrack.cargo.*.gz
```

## Test Organization

### Test File Structure
```
tests/
├── integration_tests.rs      # End-to-end workflows
├── domain_tests.rs           # Domain model unit tests
├── engine_tests.rs           # Engine unit tests
├── storage_tests.rs          # Database tests
├── security_tests.rs         # Auth/security tests
├── security_comprehensive_tests.rs  # Additional security
├── property_tests.rs         # Property-based tests
├── mock_server.rs           # Mock server for testing
└── common/                  # Shared test utilities
    ├── mod.rs
    ├── fixtures.rs
    └── helpers.rs
```

### Test Naming Convention
```rust
// Format: test_<component>_<scenario>_<expected>

#[test]
fn test_calculator_with_cache_returns_discounted_cost() {
    // ...
}

#[test]
fn test_validator_with_zero_tokens_returns_error() {
    // ...
}
```

## Troubleshooting

### Tests Timing Out
```bash
# Increase timeout
cargo test -- --test-threads=1 --timeout=300
```

### Database Locked
```bash
# Use separate database per test
# In test setup:
let pool = SqlitePool::connect(":memory:").await?;
```

### Flaky Tests
```bash
# Run test multiple times
for i in {1..100}; do
    cargo test flaky_test || break
done
```

### Missing sqlx Metadata
```bash
# Prepare sqlx metadata
export DATABASE_URL=sqlite:test.db
cargo sqlx prepare

# Check .sqlx directory was created
ls -la .sqlx/
```

## Metrics

### Test Metrics to Track
- **Total Tests:** Should increase over time
- **Coverage:** Target 90%+ for critical paths
- **Test Duration:** Full suite < 5 minutes
- **Flake Rate:** < 1%
- **Mutation Score:** 80%+

### Generate Test Report
```bash
# Install cargo-nextest
cargo install cargo-nextest

# Run with junit output
cargo nextest run --profile ci --junit output.xml

# View in CI/CD dashboard
```

## Best Practices

### DO:
- ✅ Write tests before fixing bugs
- ✅ Use descriptive test names
- ✅ Test one thing per test
- ✅ Use test fixtures/builders
- ✅ Mock external dependencies
- ✅ Clean up resources in tests

### DON'T:
- ❌ Test implementation details
- ❌ Share mutable state between tests
- ❌ Use sleep() for timing
- ❌ Ignore flaky tests
- ❌ Skip writing tests for "simple" code
- ❌ Commit commented-out tests

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://docs.rs/tokio/latest/tokio/attr.test.html)
- [Proptest Book](https://altsysrq.github.io/proptest-book/)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [SQLx Testing](https://docs.rs/sqlx/latest/sqlx/)

## Quick Commands Cheatsheet

```bash
# Full test suite
cargo test --all-features

# Fast check (no tests)
cargo check --all-features

# Coverage
cargo tarpaulin --all-features --out Html

# Benchmarks
cargo bench

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt --all

# Security
cargo audit && cargo deny check

# CI simulation
cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Watch mode (requires cargo-watch)
cargo watch -x test
```

---

**Last Updated:** 2025-11-15
**Maintained By:** QA Team
