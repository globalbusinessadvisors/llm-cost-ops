# Versioning Policy

LLM-CostOps follows [Semantic Versioning 2.0.0](https://semver.org/) (SemVer) for version numbers.

## Version Number Format

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]

Examples:
- 1.0.0         (Stable release)
- 1.2.3         (Stable release with patches)
- 2.0.0-alpha.1 (Pre-release)
- 1.5.0-rc.2    (Release candidate)
- 1.0.0+build.123 (Build metadata)
```

## Semantic Versioning Rules

### MAJOR Version (X.0.0)

Increment when you make **incompatible API changes**.

Examples of breaking changes:

- **API Changes**:
  - Removing or renaming public functions
  - Changing function signatures
  - Removing or renaming struct fields
  - Changing error types

- **Behavior Changes**:
  - Changing default behavior
  - Modifying output format
  - Altering calculation methods

- **Configuration Changes**:
  - Removing configuration options
  - Changing configuration file format
  - Modifying environment variable names

- **Database Changes**:
  - Breaking schema changes
  - Removing or renaming columns
  - Changing data types

- **CLI Changes**:
  - Removing commands or flags
  - Changing command behavior
  - Modifying output format

**Migration Required**: Users must update their code or configuration.

### MINOR Version (0.X.0)

Increment when you add **functionality in a backward-compatible manner**.

Examples of minor changes:

- **New Features**:
  - Adding new API endpoints
  - Adding new CLI commands
  - Adding new configuration options
  - Adding new LLM provider support

- **Enhancements**:
  - Adding optional parameters
  - Adding new fields to responses (backward compatible)
  - Performance improvements
  - New forecasting models

- **Deprecations**:
  - Marking features as deprecated (not removed)
  - Adding deprecation warnings

**No Migration Required**: Existing functionality continues to work.

### PATCH Version (0.0.X)

Increment when you make **backward-compatible bug fixes**.

Examples of patch changes:

- **Bug Fixes**:
  - Fixing incorrect calculations
  - Correcting validation logic
  - Resolving memory leaks
  - Fixing race conditions

- **Documentation**:
  - Documentation updates
  - Example fixes
  - Typo corrections

- **Internal Changes**:
  - Code refactoring (no behavior change)
  - Test improvements
  - Dependency updates (security or bug fixes)

**No Migration Required**: Drop-in replacement.

## Pre-release Versions

Pre-release versions indicate unstable releases:

### Alpha (X.Y.Z-alpha.N)

- **Purpose**: Early testing, API not finalized
- **Stability**: Unstable, breaking changes possible
- **Testing**: Internal testing
- **Example**: `2.0.0-alpha.1`

**Use for**: Major feature development, API experimentation

### Beta (X.Y.Z-beta.N)

- **Purpose**: Feature-complete, stabilization phase
- **Stability**: API frozen, bug fixes only
- **Testing**: Public testing
- **Example**: `2.0.0-beta.1`

**Use for**: Pre-release testing before RC

### Release Candidate (X.Y.Z-rc.N)

- **Purpose**: Final testing before release
- **Stability**: Production-ready unless critical bugs found
- **Testing**: Production-like testing
- **Example**: `2.0.0-rc.1`

**Use for**: Final validation before stable release

## Version Lifecycle

### Development Phase

```
0.1.0-alpha.1  →  0.1.0-alpha.2  →  0.1.0-beta.1  →  0.1.0-rc.1  →  0.1.0
```

### Stable Releases

```
1.0.0  →  1.0.1 (patch)  →  1.1.0 (minor)  →  2.0.0 (major)
```

### Support Timeline

```
Version    Release Date    Support Type           End of Support
1.0.x      2024-01-01     Bug fixes only         2025-01-01
1.1.x      2024-06-01     Full support           Active
2.0.x      2025-01-01     Full support           Active
```

## Version Support Policy

### Current Version

- **Support**: Full support (features + bug fixes + security)
- **Duration**: Until next major version + 6 months

### Previous Major Version

- **Support**: Bug fixes + security patches
- **Duration**: 12 months after new major version

### Older Versions

- **Support**: Security patches only (critical)
- **Duration**: 6 months after support ends for previous version

### Example Timeline

```
v2.0.0 Released (2025-01-01)
├─ v2.x.x: Full support
├─ v1.x.x: Bug fixes + security (until 2026-01-01)
└─ v0.x.x: Security only (until 2026-07-01)

v3.0.0 Released (2026-01-01)
├─ v3.x.x: Full support
├─ v2.x.x: Bug fixes + security (until 2027-01-01)
├─ v1.x.x: Security only (until 2027-07-01)
└─ v0.x.x: End of life
```

## Deprecation Policy

See [DEPRECATION.md](DEPRECATION.md) for full details.

### Deprecation Process

1. **Announce**: Mark feature as deprecated in documentation
2. **Warn**: Add deprecation warnings in code
3. **Wait**: Minimum one major version
4. **Remove**: Remove in next major version

### Example

```
v1.5.0: Feature X deprecated, warnings added
v1.6.0: Feature X still available (warnings continue)
v2.0.0: Feature X removed, alternative Feature Y available
```

## Breaking Changes Policy

See [BREAKING_CHANGES.md](BREAKING_CHANGES.md) for full details.

### Guidelines for Breaking Changes

1. **Minimize**: Avoid breaking changes when possible
2. **Document**: Clearly document all breaking changes
3. **Migrate**: Provide migration guides and tools
4. **Deprecate**: Deprecate before removing (when possible)
5. **Communicate**: Announce breaking changes in advance

### Breaking Change Checklist

- [ ] Document the breaking change in CHANGELOG.md
- [ ] Update migration guide
- [ ] Add deprecation warnings (if applicable)
- [ ] Update documentation
- [ ] Notify community via GitHub Discussions
- [ ] Update API version (if applicable)

## Version Numbering Examples

### Bug Fix

```
Before: 1.2.3
Change: Fix cost calculation rounding error
After:  1.2.4 (PATCH)
```

### New Feature (Backward Compatible)

```
Before: 1.2.4
Change: Add support for new LLM provider (Mistral)
After:  1.3.0 (MINOR)
```

### Breaking Change

```
Before: 1.5.2
Change: Redesign REST API with breaking changes
After:  2.0.0 (MAJOR)
```

### Pre-release

```
Before: 1.5.2
Change: Start work on major v2.0.0 rewrite
After:  2.0.0-alpha.1 (PRERELEASE)
```

## Rust Crate Versioning

### Cargo.toml

```toml
[package]
name = "llm-cost-ops"
version = "1.2.3"
```

### Cargo Compatibility

We maintain compatibility with Rust's [SemVer compatibility guidelines](https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility):

- **0.y.z**: Breaking changes allowed in MINOR
- **1.y.z+**: Breaking changes only in MAJOR

### Current Status

Since LLM-CostOps is pre-1.0 (currently 0.x.y):

- Minor version bumps MAY include breaking changes
- Patch version bumps MUST be backward compatible
- Breaking changes will be clearly documented

### Post-1.0 Commitment

After 1.0.0 release:

- MAJOR version: Breaking changes only
- MINOR version: New features, backward compatible
- PATCH version: Bug fixes, backward compatible

## Release Process

### Version Bump

```bash
# Update version in Cargo.toml
vim Cargo.toml

# Update CHANGELOG.md
vim CHANGELOG.md

# Commit version bump
git commit -am "chore: release v1.2.3"

# Tag release
git tag v1.2.3

# Push tag
git push origin v1.2.3
```

### Automated Versioning

We use conventional commits to determine version bumps:

```bash
# Install release tool
cargo install cargo-release

# Dry run
cargo release --dry-run

# Release patch version
cargo release patch --execute

# Release minor version
cargo release minor --execute

# Release major version
cargo release major --execute
```

## Version Queries

### CLI

```bash
# Get version
llm-costops --version
# Output: llm-cost-ops 1.2.3

# Get detailed version info
llm-costops version --verbose
# Output:
# llm-cost-ops 1.2.3
# Rust version: 1.75.0
# Build date: 2025-01-15
# Git commit: a1b2c3d
```

### API

```bash
# Version endpoint
curl https://api.example.com/version

{
  "version": "1.2.3",
  "api_version": "v1",
  "build_date": "2025-01-15T10:00:00Z",
  "git_commit": "a1b2c3d4e5f6"
}
```

### Rust Code

```rust
// Access version at compile time
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Runtime version check
use llm_cost_ops::version;
println!("Version: {}", version::VERSION);
```

## API Versioning

For REST API, we use URL-based versioning:

```
/api/v1/costs       (Stable)
/api/v2/costs       (New version)
/api/v1/forecasts   (Deprecated, will be removed)
```

### API Version Lifecycle

```
v1 Released (2024-01-01)
├─ v1: Stable
└─ v2 Released (2025-01-01)
    ├─ v2: Stable
    └─ v1: Deprecated (sunset: 2026-01-01)
```

## Version Compatibility Matrix

| LLM-CostOps | Rust    | PostgreSQL | Kubernetes |
|-------------|---------|------------|------------|
| 0.1.x       | 1.70+   | 14+        | 1.25+      |
| 0.2.x       | 1.75+   | 14+        | 1.27+      |
| 1.0.x       | 1.75+   | 15+        | 1.28+      |
| 2.0.x       | 1.77+   | 15+        | 1.29+      |

## Version History

See [CHANGELOG.md](../../CHANGELOG.md) for complete version history.

## Questions?

If you have questions about versioning:

- Read the [Semantic Versioning Spec](https://semver.org/)
- Check [GitHub Discussions](https://github.com/yourusername/llm-cost-ops/discussions)
- Ask in [Discord #development](https://discord.gg/example)
