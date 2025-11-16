# Contributing to LLM-CostOps

Thank you for your interest in contributing to LLM-CostOps! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style and Standards](#code-style-and-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)
- [Getting Help](#getting-help)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: 1.75 or later (see [rust-toolchain.toml](rust-toolchain.toml))
- **PostgreSQL**: 14+ or **SQLite** for local development
- **Git**: For version control
- **Docker**: (Optional) For containerized testing

### Finding Issues to Work On

- Check the [issue tracker](https://github.com/yourusername/llm-cost-ops/issues)
- Look for issues labeled `good first issue` or `help wanted`
- Review the [roadmap](README.md#roadmap) for planned features
- Propose new features by opening a discussion first

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR-USERNAME/llm-cost-ops.git
cd llm-cost-ops

# Add upstream remote
git remote add upstream https://github.com/yourusername/llm-cost-ops.git
```

### 2. Install Dependencies

```bash
# Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update to project's Rust version
rustup show

# Install additional tools
cargo install sqlx-cli --no-default-features --features rustls,postgres,sqlite
cargo install cargo-watch
cargo install cargo-tarpaulin  # For code coverage
```

### 3. Database Setup

#### SQLite (Development)

```bash
# Create database and run migrations
sqlx database create --database-url sqlite:cost-ops.db
sqlx migrate run --database-url sqlite:cost-ops.db
```

#### PostgreSQL (Production-like)

```bash
# Start PostgreSQL with Docker
docker run -d \
  --name llm-costops-db \
  -e POSTGRES_DB=llm_costops \
  -e POSTGRES_USER=costops \
  -e POSTGRES_PASSWORD=devpassword \
  -p 5432:5432 \
  postgres:15

# Run migrations
export DATABASE_URL="postgresql://costops:devpassword@localhost/llm_costops"
sqlx database create
sqlx migrate run
```

### 4. Build and Test

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with watch mode for development
cargo watch -x test -x run
```

### 5. Environment Configuration

Create a `.env` file for local development:

```bash
DATABASE_URL=sqlite:cost-ops.db
RUST_LOG=info,llm_cost_ops=debug
```

## Development Workflow

### 1. Create a Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create a feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### Branch Naming Conventions

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions or updates
- `chore/` - Maintenance tasks

### 2. Make Changes

- Write clear, concise code
- Follow the [Code Style](#code-style-and-standards)
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Commit Your Changes

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```bash
# Format
<type>(<scope>): <subject>

# Examples
feat(pricing): add tiered pricing support for volume discounts
fix(auth): resolve JWT token expiration validation
docs(api): update REST API endpoint documentation
test(forecasting): add integration tests for ARIMA model
refactor(storage): optimize database query performance
chore(deps): update dependencies to latest versions
```

#### Commit Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semi-colons, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD changes
- `build`: Build system changes

#### Commit Scopes

- `api` - REST API
- `auth` - Authentication/Authorization
- `cli` - Command-line interface
- `pricing` - Pricing models
- `forecasting` - Forecasting engine
- `storage` - Database/storage layer
- `export` - Export and reporting
- `observability` - Metrics, tracing, logging
- `k8s` - Kubernetes deployment

### 4. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'

# Check code coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

### 5. Update Documentation

- Update relevant documentation in `docs/`
- Update README.md if adding new features
- Add inline code documentation
- Update API documentation if endpoints change

## Code Style and Standards

### Rust Style Guide

We follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) with these additions:

#### Formatting

```bash
# Format code before committing
cargo fmt

# Check formatting
cargo fmt -- --check
```

#### Linting

```bash
# Run Clippy for linting
cargo clippy -- -D warnings

# Run Clippy with all features
cargo clippy --all-features -- -D warnings
```

#### Code Organization

```rust
// File structure for modules
// 1. Module documentation
//! Module for cost calculation engine
//!
//! This module provides...

// 2. Imports
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// 3. Constants
const MAX_BATCH_SIZE: usize = 1000;

// 4. Type definitions
pub struct CostCalculator {
    // ...
}

// 5. Implementations
impl CostCalculator {
    // Public methods first
    pub fn new() -> Self {
        // ...
    }

    // Private methods
    fn internal_method(&self) {
        // ...
    }
}

// 6. Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

#### Documentation

```rust
/// Calculate the total cost for a usage record
///
/// # Arguments
///
/// * `usage` - The usage record to calculate costs for
/// * `pricing` - The pricing model to apply
///
/// # Returns
///
/// Returns `Result<Decimal, CostError>` with the total cost
///
/// # Examples
///
/// ```
/// use llm_cost_ops::engine::calculator::calculate_cost;
///
/// let cost = calculate_cost(&usage, &pricing)?;
/// assert!(cost > Decimal::ZERO);
/// ```
///
/// # Errors
///
/// Returns error if:
/// - Pricing model is invalid
/// - Token counts are negative
pub fn calculate_cost(
    usage: &UsageRecord,
    pricing: &PricingModel,
) -> Result<Decimal, CostError> {
    // Implementation
}
```

### Error Handling

```rust
// Use thiserror for error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CostError {
    #[error("Invalid pricing model: {0}")]
    InvalidPricing(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Calculation error: {0}")]
    Calculation(String),
}

// Prefer Result over unwrap/expect
pub fn process() -> Result<(), CostError> {
    let value = get_value()?;  // Use ? operator
    // ...
    Ok(())
}
```

### Performance Best Practices

- Avoid unnecessary allocations
- Use `&str` instead of `String` where possible
- Prefer iterators over collecting to vectors
- Use `Arc` for shared ownership
- Use `Cow` for clone-on-write scenarios

## Testing Requirements

### Test Coverage

- All new features must include tests
- Maintain minimum 80% code coverage
- Include unit tests, integration tests, and benchmarks where appropriate

### Test Types

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let calculator = CostCalculator::new();
        let cost = calculator.calculate(1000, 500);
        assert_eq!(cost, Decimal::from_str("0.025").unwrap());
    }

    #[test]
    #[should_panic(expected = "Invalid tokens")]
    fn test_negative_tokens() {
        let calculator = CostCalculator::new();
        calculator.calculate(-1, 500);
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs
use llm_cost_ops::*;

#[tokio::test]
async fn test_full_cost_flow() {
    let db = setup_test_db().await;
    let usage = create_test_usage();

    // Ingest
    ingest_usage(&db, &usage).await.unwrap();

    // Calculate
    let costs = calculate_costs(&db).await.unwrap();

    assert!(costs.len() > 0);
}
```

#### Benchmarks

```rust
// benches/cost_calculation.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_cost_calculation(c: &mut Criterion) {
    let calculator = CostCalculator::new();

    c.bench_function("calculate_cost", |b| {
        b.iter(|| calculator.calculate(black_box(1000), black_box(500)))
    });
}

criterion_group!(benches, bench_cost_calculation);
criterion_main!(benches);
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage

# Run benchmarks
cargo bench

# Run tests in release mode
cargo test --release
```

## Documentation

### Types of Documentation

1. **Code Documentation**: Inline comments and doc comments
2. **API Documentation**: Generated from doc comments
3. **User Documentation**: Guides and tutorials in `docs/`
4. **Architecture Documentation**: High-level design in `docs/`

### Generating Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate documentation with all features
cargo doc --all-features --no-deps --open
```

### Documentation Standards

- All public items must have documentation
- Include examples in doc comments
- Document error conditions
- Keep documentation up-to-date with code changes

## Submitting Changes

### Before Submitting

Ensure your changes:

1. Pass all tests: `cargo test`
2. Pass formatting check: `cargo fmt -- --check`
3. Pass linting: `cargo clippy -- -D warnings`
4. Are properly documented
5. Follow commit message conventions
6. Include relevant tests
7. Update CHANGELOG.md (for significant changes)

### Pull Request Process

1. **Update Your Branch**

```bash
git fetch upstream
git rebase upstream/main
```

2. **Push to Your Fork**

```bash
git push origin feature/your-feature-name
```

3. **Create Pull Request**

- Go to GitHub and create a pull request
- Fill out the pull request template completely
- Link related issues using keywords (e.g., "Fixes #123")
- Add relevant labels
- Request review from maintainers

4. **Pull Request Checklist**

- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Commit messages follow conventions
- [ ] No merge conflicts
- [ ] PR description is clear and complete

5. **Code Review**

- Address reviewer feedback promptly
- Make requested changes in new commits
- Engage in constructive discussion
- Be open to suggestions

6. **After Approval**

- Maintainers will merge your PR
- Your contribution will be included in the next release
- Delete your feature branch after merge

### Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Limit PR size to ~500 lines of code when possible
- Include screenshots for UI changes
- Update documentation in the same PR
- Ensure CI/CD passes before requesting review

## Release Process

Releases are managed by maintainers following [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Workflow

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create release commit: `chore: release v0.2.0`
4. Tag release: `git tag v0.2.0`
5. Push tag: `git push origin v0.2.0`
6. GitHub Actions creates release and publishes artifacts

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and general discussion
- **Discord**: Real-time chat and community support
- **Email**: For security issues, see [SECURITY.md](SECURITY.md)

### Resources

- [Documentation](https://docs.example.com/llm-cost-ops)
- [Architecture Guide](docs/ARCHITECTURE.md)
- [API Reference](https://docs.rs/llm-cost-ops)
- [Examples](examples/)

### Reporting Bugs

When reporting bugs, include:

- LLM-CostOps version
- Rust version
- Operating system
- Database type and version
- Steps to reproduce
- Expected behavior
- Actual behavior
- Relevant logs or error messages

### Suggesting Features

When suggesting features:

- Check if the feature is already planned
- Describe the use case and problem it solves
- Provide examples of desired behavior
- Discuss trade-offs and implementation approaches

## Recognition

Contributors are recognized in:

- CHANGELOG.md for significant contributions
- GitHub contributors page
- Release notes

Thank you for contributing to LLM-CostOps!
