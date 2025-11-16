# Deprecation Policy

This document describes how features, APIs, and configurations are deprecated and eventually removed in LLM-CostOps.

## Deprecation Philosophy

We strive to:

1. **Minimize Breaking Changes**: Avoid deprecations when possible
2. **Provide Alternatives**: Always offer a better replacement
3. **Give Advance Notice**: Announce deprecations early
4. **Support Migration**: Provide tools and guides
5. **Maintain Stability**: Keep deprecated features working until removal

## Deprecation Timeline

### Standard Timeline

```
Version N: Feature deprecated, warnings added
Version N+1: Feature still available, warnings continue
Version N+2 (MAJOR): Feature removed

Minimum Duration: 6 months or one major version, whichever is longer
```

### Example

```
v1.5.0 (2024-06-01)
├─ /api/v1/costs/legacy endpoint deprecated
├─ Deprecation warning added
└─ Migration guide published

v1.6.0 (2024-09-01)
├─ Legacy endpoint still available
└─ Warnings continue

v1.7.0 (2024-12-01)
├─ Legacy endpoint still available
└─ Warnings continue

v2.0.0 (2025-01-01)
└─ Legacy endpoint removed
   Alternative: /api/v2/costs
```

## Deprecation Process

### 1. Decision Phase

Before deprecating a feature:

- [ ] Identify reason for deprecation
- [ ] Design replacement or alternative
- [ ] Assess user impact
- [ ] Plan migration path
- [ ] Get maintainer approval

### 2. Announcement Phase

- [ ] Create GitHub discussion
- [ ] Update documentation
- [ ] Add deprecation notice to CHANGELOG
- [ ] Announce in community channels (Discord, Twitter)
- [ ] Add to deprecation tracking issue

### 3. Warning Phase

- [ ] Add deprecation warnings in code
- [ ] Update function/type documentation
- [ ] Add migration examples
- [ ] Create migration guide
- [ ] Update API documentation

### 4. Sunset Phase

- [ ] Verify alternative is stable
- [ ] Confirm adequate migration period
- [ ] Final reminder in release notes
- [ ] Remove deprecated feature
- [ ] Update documentation

## Types of Deprecations

### API Deprecation

#### REST Endpoints

```rust
// Old endpoint (deprecated)
#[deprecated(
    since = "1.5.0",
    note = "Use /api/v2/costs instead. See migration guide: docs/migration/v1-to-v2.md"
)]
async fn legacy_costs_endpoint() -> Result<Response> {
    // Log deprecation warning
    warn!("Deprecated endpoint /api/v1/costs/legacy called. Use /api/v2/costs instead.");

    // Still functional
    Ok(legacy_response())
}
```

#### Public Functions

```rust
#[deprecated(
    since = "1.5.0",
    note = "Use calculate_cost_v2() instead. The new function supports cached tokens."
)]
pub fn calculate_cost(
    tokens: u64,
    price_per_million: Decimal,
) -> Decimal {
    calculate_cost_v2(tokens, price_per_million, None)
}

pub fn calculate_cost_v2(
    tokens: u64,
    price_per_million: Decimal,
    cached_tokens: Option<u64>,
) -> Decimal {
    // New implementation
}
```

### Configuration Deprecation

```yaml
# config.yaml (deprecated format)
database_url: "postgresql://localhost/db"  # Deprecated in v1.5.0

# config.yaml (new format)
database:
  url: "postgresql://localhost/db"
  pool_size: 20
```

Handling in code:

```rust
fn load_config(config: &Config) -> Result<DatabaseConfig> {
    // Check for deprecated config
    if let Some(old_url) = config.get::<String>("database_url").ok() {
        warn!(
            "Configuration key 'database_url' is deprecated since v1.5.0. \
             Use 'database.url' instead. See migration guide: docs/migration/config-v2.md"
        );

        // Still support old format
        return Ok(DatabaseConfig { url: old_url, ..Default::default() });
    }

    // Load new format
    config.get::<DatabaseConfig>("database")
}
```

### CLI Command Deprecation

```rust
// Old command (deprecated)
#[deprecated(
    since = "1.5.0",
    note = "Use 'llm-costops export' instead"
)]
fn cmd_dump() -> Result<()> {
    eprintln!("WARNING: 'dump' command is deprecated since v1.5.0.");
    eprintln!("Use 'export' command instead: llm-costops export --format json");
    eprintln!("The 'dump' command will be removed in v2.0.0.");

    // Still functional
    do_dump()
}
```

### Environment Variable Deprecation

```rust
fn get_database_url() -> String {
    // Try new variable first
    if let Ok(url) = env::var("COSTOPS_DATABASE_URL") {
        return url;
    }

    // Fall back to deprecated variable
    if let Ok(url) = env::var("DATABASE_URL") {
        warn!(
            "Environment variable DATABASE_URL is deprecated since v1.5.0. \
             Use COSTOPS_DATABASE_URL instead."
        );
        return url;
    }

    panic!("Database URL not configured");
}
```

## Deprecation Categories

### Category 1: Critical Security/Data Integrity

**Accelerated Timeline**: May be removed in next PATCH or MINOR version

Examples:
- Insecure authentication methods
- Data corruption risks
- Critical security vulnerabilities

**Process**:
1. Immediate deprecation warning
2. Security advisory published
3. Removal in next release if critical

### Category 2: High-Impact Features

**Standard Timeline**: Minimum one major version (6-12 months)

Examples:
- Major API endpoints
- Core CLI commands
- Database schema changes

**Process**:
1. Announce in release notes
2. Add warnings
3. Provide migration guide
4. Remove in next major version

### Category 3: Low-Impact Features

**Extended Timeline**: May extend beyond one major version

Examples:
- Experimental features
- Rarely-used utilities
- Internal APIs

**Process**:
1. Soft deprecation (documentation only)
2. Add warnings in next minor
3. Remove when appropriate

## Deprecation Documentation

### CHANGELOG Entry

```markdown
## [1.5.0] - 2024-06-01

### Deprecated

- **API**: `/api/v1/costs/legacy` endpoint is deprecated. Use `/api/v2/costs` instead. The legacy endpoint will be removed in v2.0.0. See [migration guide](docs/migration/v1-to-v2.md).

- **Config**: `database_url` configuration key is deprecated. Use `database.url` instead. The old format will be removed in v2.0.0.

- **CLI**: `dump` command is deprecated. Use `export` instead. The dump command will be removed in v2.0.0.

- **Function**: `calculate_cost()` is deprecated. Use `calculate_cost_v2()` for cached token support. The old function will be removed in v2.0.0.
```

### Migration Guide

Create `docs/migration/v1-to-v2.md`:

```markdown
# Migration Guide: v1.x to v2.0

## Deprecated Features Removed

### API Changes

#### Cost Query Endpoint

**Before (v1.x)**:
```bash
GET /api/v1/costs/legacy?start=2024-01-01&end=2024-01-31
```

**After (v2.0+)**:
```bash
POST /api/v2/costs/query
Content-Type: application/json

{
  "period": {
    "start": "2024-01-01",
    "end": "2024-01-31"
  }
}
```

### Configuration Changes

**Before (v1.x)**:
```yaml
database_url: postgresql://localhost/db
log_level: info
```

**After (v2.0+)**:
```yaml
database:
  url: postgresql://localhost/db
  pool_size: 20

logging:
  level: info
  format: json
```

### CLI Changes

**Before (v1.x)**:
```bash
llm-costops dump --output data.json
```

**After (v2.0+)**:
```bash
llm-costops export --format json --output data.json
```

### Code Changes

**Before (v1.x)**:
```rust
use llm_cost_ops::calculate_cost;

let cost = calculate_cost(1000, Decimal::from_str("10.0").unwrap());
```

**After (v2.0+)**:
```rust
use llm_cost_ops::calculate_cost_v2;

let cost = calculate_cost_v2(
    1000,
    Decimal::from_str("10.0").unwrap(),
    None  // cached_tokens
);
```
```

### Deprecation Tracking

Maintain a deprecation tracking issue on GitHub:

```markdown
# Deprecation Tracking: v2.0.0

Target Release: 2025-01-01

## APIs

- [ ] #123 Remove `/api/v1/costs/legacy` (deprecated in v1.5.0)
- [ ] #124 Remove `/api/v1/forecasts/old` (deprecated in v1.6.0)

## Configuration

- [ ] #125 Remove `database_url` key (deprecated in v1.5.0)
- [ ] #126 Remove `log_level` key (deprecated in v1.6.0)

## CLI

- [ ] #127 Remove `dump` command (deprecated in v1.5.0)
- [ ] #128 Remove `--legacy-format` flag (deprecated in v1.6.0)

## Code

- [ ] #129 Remove `calculate_cost()` (deprecated in v1.5.0)
- [ ] #130 Remove `CostRecord::from_legacy()` (deprecated in v1.6.0)

## Documentation

- [ ] Update migration guide
- [ ] Update examples
- [ ] Remove deprecated feature docs
- [ ] Update API reference
```

## Communication Channels

### Announcement Channels

1. **GitHub Release Notes**: All deprecations listed
2. **CHANGELOG.md**: Deprecation section in each release
3. **GitHub Discussions**: Dedicated deprecation announcements
4. **Discord**: #announcements channel
5. **Documentation**: Deprecation notices in docs
6. **API Warnings**: Runtime warnings in application logs

### Advance Notice

- **Major Deprecations**: 90 days before first version with warnings
- **Minor Deprecations**: 30 days before first version with warnings
- **Critical Security**: Immediate notification

## Deprecation Warning Levels

### Level 1: Documentation Only

```rust
// Soft deprecation - no code warnings yet
/// This function is deprecated and will be removed in v2.0.0.
/// Use `new_function()` instead.
pub fn old_function() {
    // Still works without warnings
}
```

### Level 2: Compiler Warnings

```rust
#[deprecated(since = "1.5.0", note = "Use new_function() instead")]
pub fn old_function() {
    // Compiler warning when used
}
```

### Level 3: Runtime Warnings

```rust
#[deprecated(since = "1.5.0", note = "Use new_function() instead")]
pub fn old_function() {
    warn!("old_function() is deprecated. Use new_function() instead.");
    // Runtime warning logged
}
```

### Level 4: Loud Warnings

```rust
#[deprecated(since = "1.5.0", note = "Use new_function() instead")]
pub fn old_function() {
    eprintln!("⚠️  WARNING: old_function() is deprecated and will be removed in v2.0.0!");
    eprintln!("    Use new_function() instead. See: docs/migration/v2.md");
    warn!("old_function() called (deprecated)");
    // Stderr warning + log warning
}
```

## Exceptions

Deprecation timeline may be shortened for:

1. **Security Issues**: Immediate removal if critical
2. **Data Corruption**: Removal in next patch if data integrity at risk
3. **Legal/Compliance**: Removal required by legal obligations
4. **Unmaintainable Code**: If cost of maintenance is prohibitive

In these cases:
- Clearly communicate urgency
- Provide automated migration tools
- Offer support for urgent migrations
- Document exception reasoning

## Questions?

For questions about deprecations:

- Check the [migration guides](../migration/)
- Read [CHANGELOG.md](../../CHANGELOG.md)
- Ask in [GitHub Discussions](https://github.com/yourusername/llm-cost-ops/discussions)
- Join [Discord #help](https://discord.gg/example)
