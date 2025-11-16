# TypeScript SDK - CI/CD Quick Start Guide

## Overview

This SDK has production-ready CI/CD workflows configured for testing, releasing, and security scanning.

## Workflows

### 1. Testing (Automated)
**Triggers:** Push/PR to main or develop

Runs on every push/PR:
- Type checking
- Linting
- Unit tests with coverage
- Build validation
- Package installation test

**Matrix:** Node 18, 20, 22 × Ubuntu, macOS, Windows (9 combinations)

### 2. Release (Tag-based)
**Triggers:** Tags matching `v*-typescript`

Steps:
1. Validate version
2. Run full test suite
3. Security audit
4. Build package
5. Generate docs
6. Publish to npm (with provenance)
7. Create GitHub release

### 3. Security (Weekly + On-demand)
**Triggers:** Weekly (Mondays) + package.json changes

Scans for:
- Dependency vulnerabilities
- License compliance
- Supply chain risks
- Outdated packages
- Code security issues (CodeQL)

---

## Quick Commands

### Local Development

```bash
# Install dependencies
npm install

# Type check
npm run typecheck

# Lint
npm run lint

# Run tests
npm test

# Run tests with coverage
npm run test:coverage

# Build package
npm run build

# Watch mode for development
npm run build:watch
npm run test:watch

# Verify everything before commit
npm run verify
```

### Release Process

#### Step 1: Update Version

```bash
# Bump version (choose one)
npm version patch  # 1.0.0 → 1.0.1
npm version minor  # 1.0.0 → 1.1.0
npm version major  # 1.0.0 → 2.0.0

# Or manually edit package.json
```

#### Step 2: Commit Changes

```bash
git add package.json package-lock.json
git commit -m "chore: bump version to 1.2.3"
git push origin main
```

#### Step 3: Create Release Tag

```bash
# Get the version from package.json
VERSION=$(node -p "require('./package.json').version")

# Create and push tag
git tag "v${VERSION}-typescript"
git push origin "v${VERSION}-typescript"
```

#### Step 4: Monitor Release

1. Go to GitHub Actions
2. Watch "TypeScript SDK - Release" workflow
3. Verify npm publication
4. Check GitHub release created

---

## Required Secrets

### NPM_TOKEN
**Required for:** Publishing to npm

**Setup:**
1. Log in to [npmjs.com](https://npmjs.com)
2. Settings → Access Tokens → Generate New Token
3. Select "Automation" type
4. Copy token
5. GitHub repo → Settings → Secrets → New repository secret
6. Name: `NPM_TOKEN`
7. Paste token value

### CODECOV_TOKEN (Optional)
**Required for:** Coverage reports

**Setup:**
1. Log in to [codecov.io](https://codecov.io)
2. Add your repository
3. Copy upload token
4. Add to GitHub Secrets as `CODECOV_TOKEN`

---

## Pre-release Versions

For alpha, beta, or RC releases:

```bash
# Alpha
npm version 1.0.0-alpha.1

# Beta
npm version 1.0.0-beta.1

# Release Candidate
npm version 1.0.0-rc.1

# Then tag and push
git tag "v1.0.0-alpha.1-typescript"
git push origin "v1.0.0-alpha.1-typescript"
```

Pre-releases are automatically marked in GitHub releases.

---

## Troubleshooting

### Tests Failing?

```bash
# Run tests in watch mode
npm run test:watch

# Check type errors
npm run typecheck

# Fix lint errors
npm run lint:fix
```

### Build Failing?

```bash
# Clean and rebuild
npm run clean
npm run build

# Check for errors
npm run verify
```

### Security Issues?

```bash
# Check vulnerabilities
npm audit

# Fix automatically
npm audit fix

# Fix with breaking changes (use carefully)
npm audit fix --force
```

### Release Failed?

**Common issues:**
1. **Version mismatch:** Ensure package.json version matches tag
2. **Already published:** Check if version exists on npm
3. **NPM_TOKEN invalid:** Regenerate and update secret
4. **Tests failing:** Fix issues before tagging

---

## Best Practices

### Before Every Commit
- [ ] Run `npm run verify`
- [ ] Tests passing
- [ ] No lint errors
- [ ] Types check

### Before Every Release
- [ ] Update CHANGELOG.md
- [ ] Version bump in package.json
- [ ] All tests passing
- [ ] Security audit clean
- [ ] Build artifacts verified

### Code Quality
- Maintain >80% test coverage
- Zero lint errors
- Strict type checking
- No `any` types
- Document public APIs

---

## Monitoring

### GitHub Actions
- **Location:** Repository → Actions tab
- **Test runs:** Every push/PR
- **Security scans:** Weekly + on package changes
- **Releases:** On version tags

### npm Package
- **Location:** https://www.npmjs.com/package/@llm-cost-ops/sdk
- **Stats:** Downloads, versions, dependencies
- **Status:** Published versions

### Security
- **Location:** Repository → Security tab
- **Alerts:** Dependabot, CodeQL
- **Reports:** Weekly security workflow

---

## Version History

| Version | Type | Changes |
|---------|------|---------|
| 1.0.0 | Major | Initial release |

---

## Semantic Versioning Guide

```
MAJOR.MINOR.PATCH

1.2.3
│ │ │
│ │ └─ Bug fixes (backward compatible)
│ └─── New features (backward compatible)
└───── Breaking changes
```

**Examples:**
- Bug fix: 1.0.0 → 1.0.1
- New feature: 1.0.1 → 1.1.0
- Breaking change: 1.1.0 → 2.0.0

---

## Getting Help

**Documentation:**
- [Full CI/CD Guide](../.github/workflows/README-SDK-CICD.md)
- [SDK README](./README.md)

**Issues:**
- Check GitHub Actions logs
- Review error messages
- Open issue in repository

**Support:**
- LLM DevOps Team
- GitHub Discussions

---

## Workflow Status Badges

Add these to README.md:

```markdown
![Tests](https://github.com/llm-devops/llm-cost-ops/workflows/TypeScript%20SDK%20-%20Test/badge.svg)
![Security](https://github.com/llm-devops/llm-cost-ops/workflows/TypeScript%20SDK%20-%20Security/badge.svg)
![npm version](https://img.shields.io/npm/v/@llm-cost-ops/sdk)
![npm downloads](https://img.shields.io/npm/dm/@llm-cost-ops/sdk)
```

---

**Last Updated:** 2025-11-16
**Maintained By:** LLM DevOps Team
