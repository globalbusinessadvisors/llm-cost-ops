# DevOps Automation Integration Guide

## Quick Start

This guide helps you integrate the DevOps automation workflows into your development process.

## Table of Contents

1. [Initial Setup](#initial-setup)
2. [SDK Release Workflow](#sdk-release-workflow)
3. [Dependency Management](#dependency-management)
4. [Status Dashboard](#status-dashboard)
5. [Custom Integrations](#custom-integrations)

## Initial Setup

### 1. Configure Secrets

Add the following secrets to your repository (Settings → Secrets and variables → Actions):

#### Package Registry Tokens

```bash
# Python (PyPI)
PYPI_TOKEN=pypi-xxxxxxxxxxxx

# TypeScript (npm)
NPM_TOKEN=npm_xxxxxxxxxxxx

# Rust (crates.io)
CARGO_TOKEN=crates-io_xxxxxxxxxxxx
```

#### Cloud Deployment

```bash
# GitHub Pages (usually not needed, uses GITHUB_TOKEN)

# Netlify
NETLIFY_AUTH_TOKEN=xxxxxxxxxxxxx
NETLIFY_SITE_ID=xxxxxxxxxxxxx

# Vercel
VERCEL_TOKEN=xxxxxxxxxxxxx

# AWS S3
AWS_ACCESS_KEY_ID=AKIAXXXXXXXXXXXXX
AWS_SECRET_ACCESS_KEY=xxxxxxxxxxxxx
```

#### Security & Monitoring

```bash
# Code coverage
CODECOV_TOKEN=xxxxxxxxxxxxx

# Security scanning
SNYK_TOKEN=xxxxxxxxxxxxx

# Notifications
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/xxx/xxx/xxx
```

### 2. Enable GitHub Actions

1. Go to Settings → Actions → General
2. Set "Workflow permissions" to "Read and write permissions"
3. Enable "Allow GitHub Actions to create and approve pull requests"

### 3. Configure Branch Protection

```bash
# Required for main branch
Settings → Branches → Add rule

Branch name pattern: main

Require status checks before merging:
  ✓ Require status checks to pass before merging
  ✓ Require branches to be up to date before merging

  Status checks:
    - test
    - lint
    - security-scan
    - coverage

Require pull request reviews before merging:
  ✓ Required number of approvals: 1

Other settings:
  ✓ Require linear history
  ✓ Include administrators
```

## SDK Release Workflow

### Automatic Release (Recommended)

1. **Make your changes and commit**:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   git push origin main
   ```

2. **Create a release tag**:
   ```bash
   # Format: <sdk>-v<major>.<minor>.<patch>
   git tag python-v1.2.3
   git push origin python-v1.2.3
   ```

3. **Monitor the release**:
   - Go to Actions → SDK Release Orchestrator
   - Watch the workflow execution
   - Check job summaries

4. **Verify the release**:
   - Check GitHub Releases page
   - Verify package on registry (PyPI, npm, etc.)
   - Test documentation deployment

### Manual Release

1. **Navigate to Actions**:
   - Go to Actions tab
   - Select "SDK Release Orchestrator"

2. **Run workflow**:
   - Click "Run workflow"
   - Select inputs:
     - SDK: `python`
     - Version: `1.2.3`
     - Dry run: `false`
   - Click "Run workflow"

3. **Review and approve**:
   - Monitor workflow execution
   - Review generated changelog
   - Verify artifacts

### Release Configuration

Create `.github/release-config.yml`:

```yaml
# Release configuration
changelog:
  categories:
    - title: Breaking Changes
      labels: ['breaking']
    - title: New Features
      labels: ['feature', 'enhancement']
    - title: Bug Fixes
      labels: ['bug', 'fix']
    - title: Documentation
      labels: ['documentation']

exclude_labels:
  - skip-changelog
  - duplicate
  - invalid

release_notes:
  header: |
    ## What's Changed

  footer: |
    **Full Changelog**: https://github.com/${{ github.repository }}/compare/${{ previous_tag }}...${{ tag }}
```

## Dependency Management

### Automatic Updates

The system automatically checks for dependency updates daily at 2 AM UTC.

**What happens**:
1. Checks all SDKs for dependency updates
2. Runs security scans
3. Creates PRs for updates
4. Runs tests on PRs
5. Auto-merges patch updates (if configured)

### Manual Dependency Update

```bash
# Update all SDKs
gh workflow run dependency-updates.yml -f sdk=all

# Update specific SDK
gh workflow run dependency-updates.yml -f sdk=python

# Disable auto-merge
gh workflow run dependency-updates.yml -f sdk=python -f auto-merge-patch=false
```

### Reviewing Dependency PRs

When you receive a dependency update PR:

1. **Check the PR description**:
   - Update type (major, minor, patch)
   - Security fixes count
   - Breaking changes

2. **Review the changes**:
   ```bash
   gh pr view <PR-number>
   gh pr diff <PR-number>
   ```

3. **Check test results**:
   ```bash
   gh pr checks <PR-number>
   ```

4. **Merge or request changes**:
   ```bash
   # Auto-merge if tests pass
   gh pr merge <PR-number> --auto --squash

   # Or manually merge
   gh pr merge <PR-number> --squash
   ```

### Dependency Configuration

Create per-SDK configuration:

**Python** (`sdks/python/.bumpversion.cfg`):
```ini
[bumpversion]
current_version = 1.0.0
commit = True
tag = False

[bumpversion:file:pyproject.toml]
search = version = "{current_version}"
replace = version = "{new_version}"
```

**TypeScript** (`sdks/typescript/audit-ci.json`):
```json
{
  "moderate": true,
  "high": true,
  "critical": true,
  "allowlist": [],
  "skip-dev": false
}
```

## Status Dashboard

### Viewing the Dashboard

The dashboard automatically updates:
- Every 6 hours
- On push to main
- On PR open/close

**Access**:
1. Main README shows current status badges
2. Actions → Status Dashboard for detailed metrics
3. `.github/badges/` directory for raw badge files

### Dashboard Components

**Build Status Badges**:
- Python build status
- TypeScript build status
- Rust build status
- Go build status
- Java build status

**Version Badges**:
- Current version for each SDK

**Health Metrics**:
- Stale PRs count
- Open security issues
- Failing workflows

**Compatibility Matrix**:
- SDK versions
- API compatibility
- Language runtime versions
- Feature support

### Customizing the Dashboard

Edit `status-dashboard.yml` to customize:

```yaml
# Change badge colors
if (( $(echo "$COVERAGE >= 90" | bc -l) )); then
  COLOR="brightgreen"
elif (( $(echo "$COVERAGE >= 80" | bc -l) )); then
  COLOR="green"
# ... customize thresholds
```

## Custom Integrations

### Slack Notifications

1. **Create Slack webhook**:
   - Go to Slack App Directory
   - Create Incoming Webhook
   - Copy webhook URL

2. **Add to secrets**:
   ```bash
   gh secret set SLACK_WEBHOOK_URL --body "https://hooks.slack.com/services/xxx/xxx/xxx"
   ```

3. **Notification is sent on**:
   - Successful releases
   - Failed releases
   - Security issues

### Codecov Integration

1. **Sign up at codecov.io**
2. **Get token for repository**
3. **Add to secrets**:
   ```bash
   gh secret set CODECOV_TOKEN --body "your-token"
   ```

4. **Coverage is uploaded**:
   - On every PR
   - On push to main
   - During release process

### Custom Workflow Integration

Create `.github/workflows/custom.yml`:

```yaml
name: Custom Workflow

on:
  push:
    branches: [main]

jobs:
  # Use reusable workflows
  test:
    uses: ./.github/workflows/reusable/test-matrix.yml
    with:
      language: 'python'
      working-directory: './sdks/python'

  coverage:
    needs: test
    uses: ./.github/workflows/reusable/coverage-report.yml
    with:
      language: 'python'
      coverage-threshold: 85
    secrets:
      CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}

  custom-step:
    needs: [test, coverage]
    runs-on: ubuntu-latest
    steps:
      - name: Custom action
        run: echo "Custom step after tests and coverage"
```

## CI/CD Pipeline Examples

### Basic PR Pipeline

```yaml
name: PR Checks

on:
  pull_request:
    paths:
      - 'sdks/python/**'

jobs:
  lint:
    uses: ./.github/workflows/reusable/lint-quality.yml
    with:
      language: 'python'
      working-directory: './sdks/python'

  test:
    uses: ./.github/workflows/reusable/test-matrix.yml
    with:
      language: 'python'
      working-directory: './sdks/python'

  security:
    uses: ./.github/workflows/reusable/security-scan.yml
    with:
      language: 'python'
      working-directory: './sdks/python'
    secrets: inherit

  coverage:
    uses: ./.github/workflows/reusable/coverage-report.yml
    with:
      language: 'python'
      working-directory: './sdks/python'
      coverage-threshold: 80
    secrets: inherit
```

### Multi-SDK Release

```yaml
name: Multi-SDK Release

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., 1.2.3)'
        required: true

jobs:
  release-python:
    uses: ./.github/workflows/sdk-release-orchestrator.yml
    with:
      sdk: 'python'
      version: ${{ inputs.version }}
    secrets: inherit

  release-typescript:
    uses: ./.github/workflows/sdk-release-orchestrator.yml
    with:
      sdk: 'typescript'
      version: ${{ inputs.version }}
    secrets: inherit

  notify:
    needs: [release-python, release-typescript]
    runs-on: ubuntu-latest
    steps:
      - name: Notify completion
        run: echo "All SDKs released successfully"
```

### Scheduled Security Scan

```yaml
name: Weekly Security Audit

on:
  schedule:
    - cron: '0 0 * * 0'  # Every Sunday at midnight

jobs:
  security-audit:
    strategy:
      matrix:
        sdk: [python, typescript, rust]
    uses: ./.github/workflows/reusable/security-scan.yml
    with:
      language: ${{ matrix.sdk }}
      enable-codeql: true
      enable-sast: true
      enable-dependency-scan: true
      enable-secret-scan: true
    secrets: inherit

  create-issues:
    needs: security-audit
    runs-on: ubuntu-latest
    if: failure()
    steps:
      - name: Create security issue
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: 'Weekly Security Scan Failed',
              body: 'Security scan failed. Please review.',
              labels: ['security', 'urgent']
            })
```

## Troubleshooting

### Common Setup Issues

#### Missing Secrets

**Error**: `secret PYPI_TOKEN not found`

**Solution**:
```bash
gh secret set PYPI_TOKEN --body "your-token"
```

#### Permission Denied

**Error**: `Resource not accessible by integration`

**Solution**:
1. Go to Settings → Actions → General
2. Set "Workflow permissions" to "Read and write permissions"
3. Enable "Allow GitHub Actions to create and approve pull requests"

#### Tag Pattern Not Recognized

**Error**: `Tag format not recognized`

**Solution**:
Use the correct format: `<sdk>-v<version>`
```bash
git tag python-v1.2.3  # Correct
git tag v1.2.3         # Incorrect
```

### Getting Help

1. **Check workflow logs**:
   ```bash
   gh run list
   gh run view <run-id> --log
   ```

2. **Enable debug mode**:
   Add to workflow:
   ```yaml
   env:
     ACTIONS_STEP_DEBUG: true
   ```

3. **Test locally**:
   Use [act](https://github.com/nektos/act) to test workflows locally:
   ```bash
   act -j test
   ```

## Best Practices

1. **Always use dry-run first** for releases
2. **Review dependency PRs** before merging
3. **Monitor dashboard** regularly
4. **Keep secrets up to date**
5. **Test workflows** in feature branches
6. **Use conventional commits** for better changelogs
7. **Document custom workflows**
8. **Monitor security alerts**

## Next Steps

1. ✅ Configure all required secrets
2. ✅ Enable branch protection
3. ✅ Test a release with dry-run
4. ✅ Set up Slack notifications
5. ✅ Configure Codecov
6. ✅ Review and merge first dependency PR
7. ✅ Customize dashboard badges
8. ✅ Create custom workflows as needed

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow README](./README.md)
- [DevOps Automation Guide](../../docs/ci-cd/DEVOPS_AUTOMATION_GUIDE.md)
- [Repository Settings](../../settings)
