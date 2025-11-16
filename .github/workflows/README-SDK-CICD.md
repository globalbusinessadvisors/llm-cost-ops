# TypeScript SDK CI/CD Workflows

This directory contains production-ready CI/CD workflows for the TypeScript SDK located in `/sdk/`.

## Overview

Three comprehensive workflows have been implemented:

1. **Testing Workflow** (`sdk-typescript-test.yml`)
2. **Release Workflow** (`sdk-typescript-release.yml`)
3. **Security Workflow** (`sdk-typescript-security.yml`)

---

## 1. Testing Workflow

**File:** `.github/workflows/sdk-typescript-test.yml`

### Triggers
- Push to `main` or `develop` branches (when SDK files change)
- Pull requests to `main` or `develop` branches
- Manual dispatch

### Features

#### Test Job
- **Matrix Testing:**
  - Node.js versions: 18, 20, 22
  - Operating Systems: Ubuntu, macOS, Windows
  - Total: 9 combinations

- **Test Coverage:**
  - Type checking with TypeScript
  - ESLint linting
  - Vitest unit tests with coverage
  - Coverage reports uploaded to Codecov
  - Coverage artifacts archived

#### Build Job
- **Build Validation:**
  - Builds package with tsup
  - Verifies CJS, ESM, and type declaration outputs
  - Tests both ESM and CJS imports
  - Checks bundle sizes (warns if ESM > 100KB)
  - Archives build artifacts

#### Integration Test Job
- **Package Installation Test:**
  - Packs the built package
  - Tests installation in a fresh directory
  - Validates package can be imported

#### All Tests Passed
- Final gate that ensures all previous jobs succeeded

### Caching
- npm dependencies cached based on `package-lock.json`
- Speeds up subsequent runs significantly

---

## 2. Release Workflow

**File:** `.github/workflows/sdk-typescript-release.yml`

### Triggers
- Git tags matching pattern `v*-typescript` (e.g., `v1.0.0-typescript`)
- Manual dispatch with version input

### Jobs

#### 1. Validate
- Extracts version from tag or input
- Validates version format (semver)
- Ensures `package.json` version matches release version

#### 2. Test & Build
- Runs full test suite
- Performs security audit
- Type checks, lints, and builds
- Verifies all build outputs
- Archives artifacts for release

#### 3. Security Scan
- Runs `npm audit` on production dependencies
- Fails if critical or high vulnerabilities found
- Checks license compliance
- Uses `license-checker` for validation

#### 4. Generate Documentation
- Installs TypeDoc
- Generates API documentation from TypeScript source
- Archives documentation artifacts

#### 5. Publish to npm
- **Provenance:** Uses npm provenance for supply chain security
- **Version Check:** Verifies version isn't already published
- **Public Access:** Publishes with `--access public`
- **Environment:** Uses `npm-production` environment for protection

#### 6. Create GitHub Release
- Downloads all artifacts
- Generates changelog from git commits
- Creates GitHub release with:
  - Version information
  - Changelog
  - Installation instructions
  - Links to npm package
  - Build artifacts attached
- Marks pre-releases (alpha, beta, rc) appropriately

### Required Secrets

#### NPM_TOKEN
- **Type:** Secret
- **Purpose:** Authenticate with npm registry
- **Scope:** Publish access to `@llm-cost-ops/sdk`
- **How to create:**
  1. Log in to npmjs.com
  2. Go to Access Tokens → Generate New Token
  3. Select "Automation" type
  4. Add to GitHub Secrets as `NPM_TOKEN`

#### CODECOV_TOKEN (optional)
- **Type:** Secret
- **Purpose:** Upload coverage reports to Codecov
- **How to create:**
  1. Log in to codecov.io
  2. Add repository
  3. Copy upload token
  4. Add to GitHub Secrets as `CODECOV_TOKEN`

#### GITHUB_TOKEN
- **Type:** Automatic
- **Purpose:** Create releases, upload artifacts
- **Scope:** Automatically provided by GitHub Actions

---

## 3. Security Workflow

**File:** `.github/workflows/sdk-typescript-security.yml`

### Triggers
- Weekly schedule (Mondays at 9:00 AM UTC)
- Manual dispatch
- Push to `main` when `package.json` or `package-lock.json` changes

### Jobs

#### 1. Dependency Audit
- Runs `npm audit` on production dependencies
- Runs `npm audit` on all dependencies (including dev)
- Generates JSON reports
- Fails if critical or high vulnerabilities in production deps
- Archives audit reports

#### 2. Dependency Review (push events only)
- Uses GitHub's Dependency Review Action
- Reviews dependency changes in commits
- Checks for vulnerabilities
- Validates licenses
- Uses config from `.github/dependency-review-config.yml`

#### 3. License Compliance
- Uses `license-checker` to validate all licenses
- **Allowed Licenses:**
  - MIT
  - Apache-2.0
  - BSD-2-Clause, BSD-3-Clause
  - ISC
  - 0BSD
  - CC0-1.0
  - Unlicense

- **Blocked Licenses:**
  - GPL, LGPL, AGPL (copyleft)
  - SSPL
  - CC-BY-NC (non-commercial)

- Generates license report in Markdown
- Archives license compliance reports

#### 4. Supply Chain Security
- Verifies `package-lock.json` integrity
- Checks for deprecated packages
- Validates package checksums
- Detects potential typosquatting attempts

#### 5. Outdated Dependencies
- Reports outdated dependencies
- Identifies available major version updates
- Provides update recommendations

#### 6. CodeQL Analysis
- Runs GitHub's CodeQL security scanner
- Scans for security vulnerabilities and code quality issues
- Uses extended security queries
- Results available in Security tab

#### 7. Security Summary
- Aggregates all security job results
- Generates comprehensive security report
- Provides recommendations

---

## Configuration Files

### `.github/dependency-review-config.yml`
Configures the Dependency Review Action:
- Severity thresholds
- License allow/deny lists
- Package restrictions
- PR commenting behavior

---

## Usage Examples

### Running Tests Locally

```bash
cd sdk
npm install
npm run typecheck
npm run lint
npm test
npm run build
```

### Creating a Release

#### Option 1: Tag-based (Recommended)

```bash
# Update version in package.json
cd sdk
npm version 1.2.3 --no-git-tag-version

# Commit the change
git add package.json package-lock.json
git commit -m "chore: bump version to 1.2.3"

# Create and push tag
git tag v1.2.3-typescript
git push origin main --tags
```

#### Option 2: Manual Dispatch

1. Go to Actions → TypeScript SDK - Release
2. Click "Run workflow"
3. Enter version (e.g., `1.2.3`)
4. Optionally enable dry-run for testing

### Checking Security Status

```bash
cd sdk

# Run security audit
npm audit

# Check licenses
npx license-checker --production --summary

# Check for outdated packages
npm outdated
```

---

## Workflow Optimization

### Caching Strategy
All workflows use npm caching based on `package-lock.json`:
```yaml
- uses: actions/setup-node@v4
  with:
    cache: 'npm'
    cache-dependency-path: sdk/package-lock.json
```

### Concurrency Control
Test workflow prevents concurrent runs on the same ref:
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

### Timeout Protection
All jobs have reasonable timeouts to prevent hanging:
- Test jobs: 15 minutes
- Build jobs: 10 minutes
- Security jobs: 10-15 minutes

---

## Bundle Size Monitoring

The test workflow includes bundle size checks:
- Measures uncompressed ESM bundle size
- Warns if bundle exceeds 100KB
- Helps maintain small package size

To check locally:
```bash
cd sdk
npm run build
du -h dist/index.mjs
```

---

## Troubleshooting

### Test Failures

**Type Errors:**
```bash
npm run typecheck
```

**Lint Errors:**
```bash
npm run lint:fix
```

**Test Failures:**
```bash
npm run test:watch
```

### Release Failures

**Version Mismatch:**
- Ensure `package.json` version matches release tag
- Tag format: `v1.2.3-typescript`

**npm Publish Fails:**
- Verify `NPM_TOKEN` secret is set
- Check token has publish permissions
- Ensure version isn't already published

**Security Audit Fails:**
- Run `npm audit fix` to auto-fix vulnerabilities
- For breaking changes, evaluate manually

### Security Failures

**Vulnerable Dependencies:**
```bash
npm audit
npm audit fix
# Or for breaking changes:
npm audit fix --force
```

**License Violations:**
- Review dependency licenses
- Replace dependencies with problematic licenses
- Update allowed licenses if appropriate

---

## Best Practices

### Before Committing
1. Run tests locally: `npm test`
2. Check types: `npm run typecheck`
3. Lint code: `npm run lint`
4. Build package: `npm run build`

### Before Releasing
1. Update version in `package.json`
2. Update CHANGELOG.md
3. Run full test suite
4. Check security: `npm audit`
5. Test package locally: `npm pack && npm install -g llm-cost-ops-sdk-*.tgz`

### Semantic Versioning
- **Patch** (1.0.x): Bug fixes, no breaking changes
- **Minor** (1.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes

### Pre-release Versions
- Alpha: `1.0.0-alpha.1`
- Beta: `1.0.0-beta.1`
- RC: `1.0.0-rc.1`

These are automatically marked as pre-releases on GitHub.

---

## Monitoring

### GitHub Actions
- View workflow runs: Repository → Actions
- Download artifacts from completed runs
- Review security scanning results

### npm Registry
- Package page: https://www.npmjs.com/package/@llm-cost-ops/sdk
- Download stats, version history

### Security Alerts
- Repository → Security → Dependabot alerts
- CodeQL scanning results
- Weekly security workflow reports

---

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [npm Publishing Guide](https://docs.npmjs.com/packages-and-modules/contributing-packages-to-the-registry)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [tsup Documentation](https://tsup.egoist.dev/)
- [Vitest Documentation](https://vitest.dev/)
- [CodeQL Documentation](https://codeql.github.com/docs/)

---

## Support

For issues or questions:
1. Check workflow logs in GitHub Actions
2. Review this documentation
3. Open an issue in the repository
4. Contact the LLM DevOps Team

---

**Last Updated:** 2025-11-16
**Maintained By:** LLM DevOps Team
