# DevOps Automation Guide

## Overview

This guide covers the comprehensive DevOps automation system for LLM Cost Ops, including CI/CD pipelines, release orchestration, and infrastructure management.

## Table of Contents

1. [Architecture](#architecture)
2. [Reusable Workflows](#reusable-workflows)
3. [Release Process](#release-process)
4. [Dependency Management](#dependency-management)
5. [Monitoring & Dashboards](#monitoring--dashboards)
6. [Security Automation](#security-automation)
7. [Best Practices](#best-practices)

## Architecture

### Workflow Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│           SDK Release Orchestrator                      │
│                  (Main Controller)                       │
└────────────┬────────────────────────────────────────────┘
             │
             ├─── Detect SDK & Version
             │
             ├─── Version Validation
             │
             ├─── Changelog Generation
             │
             ├─── Test Matrix
             │
             ├─── Security Scan
             │
             ├─── Coverage Report
             │
             ├─── Build Artifacts
             │
             ├─── Create Release
             │
             ├─── Publish Package
             │
             └─── Deploy Documentation
```

### Reusable Workflow Components

```
┌────────────────────────────────────────────────────────┐
│                 Reusable Workflows                     │
├────────────────────────────────────────────────────────┤
│ • version-bump.yml         • test-matrix.yml          │
│ • changelog-generator.yml  • lint-quality.yml         │
│ • coverage-report.yml      • publish-package.yml      │
│ • security-scan.yml        • release-automation.yml   │
│ • docs-deploy.yml                                      │
└────────────────────────────────────────────────────────┘
```

## Reusable Workflows

### 1. Version Bump

**Purpose**: Automatically manage version numbers across different languages.

**Supported Languages**: Python, TypeScript, Rust, Go, Java

**Features**:
- Auto-detect version bump type from conventional commits
- Manual override support
- Dry-run mode
- Cross-language support

**Example Usage**:

```yaml
jobs:
  bump:
    uses: ./.github/workflows/reusable/version-bump.yml
    with:
      language: 'python'
      working-directory: './sdks/python'
      bump-type: 'auto'
      dry-run: false
```

**Outputs**:
- `new-version`: The bumped version
- `old-version`: Previous version
- `version-changed`: Boolean indicating if version changed

### 2. Changelog Generator

**Purpose**: Generate changelogs from git commit history.

**Formats**:
- Keep a Changelog
- Conventional Commits
- Simple

**Features**:
- Automatic commit categorization
- git-cliff integration
- Multi-language support
- Auto-commit option

**Example Usage**:

```yaml
jobs:
  changelog:
    uses: ./.github/workflows/reusable/changelog-generator.yml
    with:
      language: 'typescript'
      version: 'v1.2.3'
      format: 'conventional'
      commit-changelog: true
```

### 3. Coverage Report

**Purpose**: Generate and enforce code coverage standards.

**Tools by Language**:
- Python: pytest-cov, coverage.py
- TypeScript: Jest, nyc
- Rust: cargo-llvm-cov
- Go: go tool cover
- Java: JaCoCo

**Features**:
- Coverage badge generation
- Codecov integration
- Threshold enforcement
- Multiple report formats

**Example Usage**:

```yaml
jobs:
  coverage:
    uses: ./.github/workflows/reusable/coverage-report.yml
    with:
      language: 'rust'
      coverage-threshold: 80
      fail-on-threshold: true
    secrets:
      CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}
```

### 4. Security Scan

**Purpose**: Comprehensive security analysis.

**Scan Types**:
- CodeQL (static analysis)
- Semgrep (SAST)
- Dependency scanning
- Secret detection
- Container scanning
- SBOM generation
- OSSF Scorecard

**Example Usage**:

```yaml
jobs:
  security:
    uses: ./.github/workflows/reusable/security-scan.yml
    with:
      language: 'python'
      enable-codeql: true
      enable-sast: true
      enable-dependency-scan: true
      enable-secret-scan: true
    secrets:
      SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
```

### 5. Documentation Deployment

**Purpose**: Build and deploy documentation.

**Supported Generators**:
- Python: Sphinx, MkDocs
- TypeScript: TypeDoc, JSDoc
- Rust: rustdoc
- Go: godoc
- Java: Javadoc

**Deployment Targets**:
- GitHub Pages
- Netlify
- Vercel
- AWS S3

**Example Usage**:

```yaml
jobs:
  docs:
    uses: ./.github/workflows/reusable/docs-deploy.yml
    with:
      language: 'python'
      deploy-target: 'github-pages'
      generate-api-docs: true
```

## Release Process

### Automatic Release (Tag-Based)

1. **Create a tag following the pattern**:
   ```bash
   git tag <sdk>-v<version>
   # Examples:
   git tag python-v1.2.3
   git tag typescript-v2.0.0
   git tag rust-v0.5.1
   ```

2. **Push the tag**:
   ```bash
   git push origin <sdk>-v<version>
   ```

3. **Orchestrator executes**:
   - Detects SDK and version from tag
   - Validates version consistency
   - Generates changelog
   - Runs full test suite
   - Performs security scans
   - Generates coverage reports
   - Builds release artifacts
   - Creates GitHub release
   - Publishes to package registry
   - Deploys documentation
   - Sends notifications

### Manual Release

1. **Trigger via GitHub Actions UI**:
   - Go to Actions → SDK Release Orchestrator
   - Click "Run workflow"
   - Select SDK and version
   - Choose dry-run mode if testing

2. **Monitor progress**:
   - Check workflow run in Actions tab
   - Review job summaries
   - Verify artifact uploads

### Release Checklist

- [ ] All tests passing
- [ ] Security scans clean
- [ ] Coverage meets threshold
- [ ] CHANGELOG updated
- [ ] Version bumped in all files
- [ ] Documentation generated
- [ ] Package published
- [ ] GitHub release created

## Dependency Management

### Automatic Updates

The `dependency-updates.yml` workflow runs daily to:

1. Check for dependency updates
2. Run security scans
3. Create PRs for updates
4. Execute tests on PRs
5. Auto-merge patch updates (optional)

### Per-Language Tools

| Language | Update Tool | Security Scanner |
|----------|-------------|------------------|
| Python | pip-tools | pip-audit, safety |
| TypeScript | npm-check-updates | npm audit |
| Rust | cargo-edit | cargo-audit |
| Go | go get -u | govulncheck |
| Java | versions plugin | OWASP Dependency Check |

### Manual Dependency Update

```bash
# Trigger for specific SDK
gh workflow run dependency-updates.yml \
  -f sdk=python \
  -f auto-merge-patch=true
```

### Reviewing Dependency PRs

1. **Check PR description** for update details
2. **Review security fixes** count
3. **Verify test results**
4. **Check for breaking changes**
5. **Merge or request changes**

## Monitoring & Dashboards

### Status Dashboard

The `status-dashboard.yml` workflow maintains project health metrics:

**Features**:
- Build status badges
- Version badges
- SDK compatibility matrix
- Health reports
- Automated README updates

**Schedule**: Every 6 hours + on push/PR events

**Generated Artifacts**:
```
.github/badges/
├── python-build.svg
├── python-version.svg
├── typescript-build.svg
├── typescript-version.svg
├── rust-build.svg
└── rust-version.svg
```

### Health Metrics

The dashboard tracks:
- Stale PRs (>30 days)
- Open security issues
- Failing workflows
- SDK build status
- Test coverage
- Dependency vulnerabilities

### Compatibility Matrix

Auto-generated matrix showing:
- SDK versions
- API compatibility
- Language runtime versions
- Feature support status

## Security Automation

### Continuous Security Scanning

All PRs and commits trigger:
1. CodeQL analysis
2. Semgrep SAST
3. Dependency scanning
4. Secret detection

### Security Scan Outputs

- SARIF reports → GitHub Security tab
- JSON artifacts → 90-day retention
- SBOM → Dependency graph
- OSSF Scorecard → Security insights

### Responding to Security Issues

1. **Review Security tab** for findings
2. **Check scan artifacts** for details
3. **Create fix PR** with security label
4. **Re-run scans** to verify fix
5. **Document in changelog**

### SBOM Generation

Software Bill of Materials generated for:
- Dependencies
- Licenses
- Vulnerabilities
- Supply chain security

Formats: SPDX, CycloneDX

## Best Practices

### 1. Commit Messages

Use conventional commits for automatic changelog generation:

```
feat: add new feature
fix: resolve bug
docs: update documentation
perf: improve performance
refactor: restructure code
test: add tests
chore: maintenance tasks
```

### 2. Version Tagging

Always use the SDK-specific tag format:
```bash
<sdk>-v<major>.<minor>.<patch>
```

### 3. Release Cadence

- **Major releases**: Quarterly or for breaking changes
- **Minor releases**: Monthly for new features
- **Patch releases**: As needed for bug fixes

### 4. Testing Strategy

- Run full test matrix before releases
- Maintain coverage above threshold
- Test cross-SDK compatibility
- Verify in staging environment

### 5. Documentation

- Keep docs in sync with code
- Generate API docs automatically
- Update compatibility matrix
- Maintain changelog

### 6. Security

- Enable all security scans
- Review dependency updates weekly
- Respond to security issues within 24h
- Keep dependencies up to date

### 7. Monitoring

- Check dashboard regularly
- Review health metrics
- Address stale PRs
- Monitor build status

## Troubleshooting

### Common Issues

#### Release Failed

**Symptom**: Release workflow fails

**Solutions**:
1. Check version consistency across files
2. Ensure all tests pass
3. Verify no security vulnerabilities
4. Check for merge conflicts
5. Review workflow logs

#### Coverage Below Threshold

**Symptom**: Coverage job fails

**Solutions**:
1. Add more tests
2. Remove untested code
3. Temporarily adjust threshold
4. Use fail-on-threshold: false

#### Dependency Update Conflicts

**Symptom**: Update PR has conflicts

**Solutions**:
1. Manually resolve conflicts
2. Update base branch
3. Re-run dependency update
4. Review breaking changes

#### Documentation Deployment Failed

**Symptom**: Docs not deploying

**Solutions**:
1. Check deployment target configuration
2. Verify secrets are set
3. Validate documentation build
4. Check deployment permissions

### Debug Mode

Enable debug logging:

```yaml
env:
  ACTIONS_STEP_DEBUG: true
  ACTIONS_RUNNER_DEBUG: true
```

### Workflow Logs

Access detailed logs:
1. Go to Actions tab
2. Select workflow run
3. Click on failed job
4. Expand step for logs
5. Download log archive if needed

## Advanced Usage

### Custom Workflows

Create custom workflows using reusable components:

```yaml
name: Custom Release

on:
  workflow_dispatch:

jobs:
  test:
    uses: ./.github/workflows/reusable/test-matrix.yml
    with:
      language: 'python'

  security:
    needs: test
    uses: ./.github/workflows/reusable/security-scan.yml
    with:
      language: 'python'

  release:
    needs: [test, security]
    uses: ./.github/workflows/reusable/release-automation.yml
    with:
      language: 'python'
      version: '1.0.0'
```

### Matrix Strategies

Test across multiple configurations:

```yaml
strategy:
  matrix:
    python-version: ['3.9', '3.10', '3.11', '3.12']
    os: [ubuntu-latest, macos-latest, windows-latest]
```

### Conditional Execution

Run jobs conditionally:

```yaml
jobs:
  deploy:
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    runs-on: ubuntu-latest
```

## Integration Guide

### CI/CD Pipeline Integration

Integrate with existing pipelines:

1. **Pre-commit hooks**: Run linters and formatters
2. **PR checks**: Execute tests and security scans
3. **Merge gates**: Require approval and passing tests
4. **Post-merge**: Deploy to staging
5. **Release**: Automated release process

### Third-Party Integrations

- **Slack**: Release notifications
- **Codecov**: Coverage reports
- **Snyk**: Security scanning
- **Dependabot**: Alternative dependency updates

### Monitoring Integrations

- **Datadog**: Metrics and logs
- **Sentry**: Error tracking
- **PagerDuty**: Incident management

## Support & Resources

### Documentation

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Workflow README](../../.github/workflows/README.md)
- [SDK Documentation](../sdk/)

### Getting Help

- Create an issue with `devops` label
- Check workflow run logs
- Review step summaries
- Consult team documentation

### Contributing

To improve DevOps automation:

1. Fork the repository
2. Create a feature branch
3. Add/modify workflows
4. Test thoroughly
5. Submit pull request
6. Update documentation

## Appendix

### Workflow File Reference

| File | Purpose | Trigger |
|------|---------|---------|
| `sdk-release-orchestrator.yml` | Main release controller | Tag push, manual |
| `dependency-updates.yml` | Automated dependency updates | Daily, manual |
| `status-dashboard.yml` | Health monitoring | Every 6h, push, PR |
| `version-bump.yml` | Version management | Called by other workflows |
| `changelog-generator.yml` | Changelog generation | Called by other workflows |
| `coverage-report.yml` | Coverage reporting | Called by other workflows |
| `security-scan.yml` | Security analysis | Called by other workflows |
| `docs-deploy.yml` | Documentation deployment | Called by other workflows |

### Environment Setup

Required repository secrets:
- Package registry tokens
- Cloud provider credentials
- Service API tokens
- Notification webhooks

### Performance Optimization

- Use caching for dependencies
- Parallelize independent jobs
- Optimize test execution
- Minimize artifact sizes
- Use matrix strategies efficiently
