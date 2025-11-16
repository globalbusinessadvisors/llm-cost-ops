# CI/CD Documentation Index

Welcome to the CI/CD and DevOps automation documentation for LLM Cost Ops.

## Quick Navigation

### Getting Started
- [Quick Start Guide](QUICK-START-GUIDE.md) - Get up and running in 15 minutes
- [Integration Guide](../../.github/workflows/INTEGRATION.md) - Step-by-step integration
- [DevOps Automation Guide](DEVOPS_AUTOMATION_GUIDE.md) - Complete automation reference

### Architecture & Design
- [CI/CD Architecture](CI-CD-ARCHITECTURE.md) - System architecture and design
- [Workflow Reference](../../.github/workflows/README.md) - All workflows explained

### Security
- [Security Guide](SECURITY-GUIDE.md) - Security best practices and scanning

### Reports
- [DevOps Automation Report](../../DEVOPS_AUTOMATION_REPORT.md) - Implementation report
- [CI/CD Completion Report](CI-CD-COMPLETION-REPORT.md) - Project completion summary

## Documentation Structure

```
docs/ci-cd/
├── INDEX.md (this file)                    # Documentation index
├── QUICK-START-GUIDE.md                    # Quick start (15 min)
├── DEVOPS_AUTOMATION_GUIDE.md              # Complete guide
├── CI-CD-ARCHITECTURE.md                   # Architecture docs
├── CI-CD-COMPLETION-REPORT.md              # Completion report
└── SECURITY-GUIDE.md                       # Security guide

.github/workflows/
├── README.md                               # Workflow reference
├── INTEGRATION.md                          # Integration guide
├── sdk-release-orchestrator.yml            # Release automation
├── dependency-updates.yml                  # Dependency management
├── status-dashboard.yml                    # Health monitoring
└── reusable/                               # Reusable workflows
    ├── version-bump.yml
    ├── changelog-generator.yml
    ├── coverage-report.yml
    ├── security-scan.yml
    ├── docs-deploy.yml
    ├── test-matrix.yml
    ├── lint-quality.yml
    ├── publish-package.yml
    └── release-automation.yml
```

## By Topic

### Release Management
- [SDK Release Orchestrator](../../.github/workflows/sdk-release-orchestrator.yml)
- [Version Bump Workflow](../../.github/workflows/reusable/version-bump.yml)
- [Changelog Generator](../../.github/workflows/reusable/changelog-generator.yml)
- [Release Automation](../../.github/workflows/reusable/release-automation.yml)

### Testing & Quality
- [Test Matrix](../../.github/workflows/reusable/test-matrix.yml)
- [Lint & Quality](../../.github/workflows/reusable/lint-quality.yml)
- [Coverage Report](../../.github/workflows/reusable/coverage-report.yml)

### Security
- [Security Scan](../../.github/workflows/reusable/security-scan.yml)
- [Security Guide](SECURITY-GUIDE.md)
- [Dependency Updates](../../.github/workflows/dependency-updates.yml)

### Deployment
- [Publish Package](../../.github/workflows/reusable/publish-package.yml)
- [Docs Deploy](../../.github/workflows/reusable/docs-deploy.yml)

### Monitoring
- [Status Dashboard](../../.github/workflows/status-dashboard.yml)

## By Role

### Developers
Start here:
1. [Quick Start Guide](QUICK-START-GUIDE.md)
2. [Integration Guide](../../.github/workflows/INTEGRATION.md)
3. [Workflow Reference](../../.github/workflows/README.md)

### DevOps Engineers
Start here:
1. [CI/CD Architecture](CI-CD-ARCHITECTURE.md)
2. [DevOps Automation Guide](DEVOPS_AUTOMATION_GUIDE.md)
3. [Security Guide](SECURITY-GUIDE.md)

### Security Engineers
Start here:
1. [Security Guide](SECURITY-GUIDE.md)
2. [Security Scan Workflow](../../.github/workflows/reusable/security-scan.yml)

### Project Managers
Start here:
1. [DevOps Automation Report](../../DEVOPS_AUTOMATION_REPORT.md)
2. [CI/CD Completion Report](CI-CD-COMPLETION-REPORT.md)

## Common Tasks

### Release a New Version
```bash
# Automatic release (recommended)
git tag python-v1.2.3
git push origin python-v1.2.3

# Manual release
gh workflow run sdk-release-orchestrator.yml \
  -f sdk=python \
  -f version=1.2.3
```
See: [Integration Guide](../../.github/workflows/INTEGRATION.md#sdk-release-workflow)

### Update Dependencies
```bash
# Automatic (runs daily)
# Or manual trigger:
gh workflow run dependency-updates.yml -f sdk=all
```
See: [Integration Guide](../../.github/workflows/INTEGRATION.md#dependency-management)

### Check Project Health
```bash
# View status dashboard
gh workflow run status-dashboard.yml
```
See: [Status Dashboard](../../.github/workflows/status-dashboard.yml)

### Run Security Scan
```bash
# Part of every PR, or manual:
gh workflow run reusable/security-scan.yml
```
See: [Security Guide](SECURITY-GUIDE.md)

## Troubleshooting

### Common Issues
- [Workflow Troubleshooting](../../.github/workflows/README.md#troubleshooting)
- [Integration Issues](../../.github/workflows/INTEGRATION.md#troubleshooting)
- [Security Issues](SECURITY-GUIDE.md#troubleshooting)

### Getting Help
1. Check relevant documentation above
2. Review workflow logs in Actions tab
3. Search existing issues
4. Create new issue with `devops` label

## Contributing

To improve CI/CD documentation:
1. Make changes to relevant markdown files
2. Update this index if adding new docs
3. Follow documentation style guide
4. Submit pull request

## Version History

- **v1.0** (Nov 16, 2025) - Initial comprehensive automation system
  - 9 reusable workflows
  - 3 automation workflows
  - Complete documentation suite
  - Multi-language support (Python, TypeScript, Rust, Go, Java)

## License

Documentation licensed under CC-BY-4.0
Code licensed under Apache-2.0

---

**Last Updated**: November 16, 2025
**Maintained By**: DevOps Team
