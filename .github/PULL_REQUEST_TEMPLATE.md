## Description

<!-- Provide a clear and concise description of your changes -->

### What does this PR do?

<!-- Describe what changes you're making and why -->

### Related Issues

<!-- Link to related issues using keywords: Fixes #123, Closes #456, Relates to #789 -->

Fixes #

## Type of Change

<!-- Mark the relevant option with an "x" -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring
- [ ] Test addition/improvement
- [ ] CI/CD improvement
- [ ] Dependency update
- [ ] Other (please describe):

## Changes Made

<!-- Provide a bullet-point list of changes -->

-
-
-

## Breaking Changes

<!-- If this is a breaking change, describe the impact and migration path -->

**Is this a breaking change?** No / Yes

<!-- If yes, complete the following: -->

### What breaks?

<!-- Describe what existing functionality will break -->

### Migration Path

<!-- Describe how users should migrate from the old to new behavior -->

```rust
// Before
old_api_call()

// After
new_api_call()
```

### Deprecation Timeline

<!-- When will the old behavior be removed? -->

## Testing

<!-- Describe the tests you've added or run -->

### Test Coverage

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Benchmarks added/updated
- [ ] Manual testing performed

### Test Details

<!-- Describe what you tested and how -->

```bash
# Commands used to test
cargo test
cargo test --test integration_tests
cargo bench
```

### Test Results

<!-- Paste relevant test output or describe results -->

```
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured
```

## Performance Impact

<!-- Describe any performance implications -->

- [ ] No performance impact
- [ ] Performance improvement (describe below)
- [ ] Performance regression (justify below)
- [ ] Performance neutral (different trade-offs)

<!-- If applicable, provide benchmark results -->

**Benchmarks:**

```
Before: 100ms
After:  50ms
Improvement: 50%
```

## Documentation

<!-- Ensure documentation is updated -->

- [ ] Code documentation (doc comments) updated
- [ ] README.md updated (if needed)
- [ ] Architecture documentation updated (if needed)
- [ ] API documentation updated (if needed)
- [ ] Examples updated/added (if needed)
- [ ] CHANGELOG.md updated
- [ ] Migration guide created (for breaking changes)

## Checklist

<!-- Ensure you've completed all items before requesting review -->

### Code Quality

- [ ] Code follows the project's style guidelines
- [ ] Code has been formatted (`cargo fmt`)
- [ ] Code passes linting (`cargo clippy -- -D warnings`)
- [ ] No new compiler warnings introduced
- [ ] Dead code and unused imports removed

### Testing

- [ ] All tests pass locally (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Existing tests updated for changed functionality
- [ ] Edge cases covered by tests
- [ ] Error cases tested

### Documentation

- [ ] Public APIs documented with doc comments
- [ ] Examples included in doc comments where appropriate
- [ ] README updated if user-facing changes
- [ ] CHANGELOG.md updated with notable changes

### Commits

- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] Commits are logical and atomic
- [ ] Commit history is clean (squashed/rebased if needed)

### Dependencies

- [ ] No unnecessary dependencies added
- [ ] New dependencies are well-maintained and secure
- [ ] Cargo.lock updated (if dependencies changed)

### Security

- [ ] No secrets or credentials in code
- [ ] Security implications considered
- [ ] Input validation added where needed
- [ ] Error messages don't leak sensitive information

### CI/CD

- [ ] CI pipeline passes
- [ ] No test failures
- [ ] No linting errors
- [ ] Code coverage maintained or improved

## Screenshots / Demos

<!-- If this includes UI changes, API changes, or new features, include screenshots or demos -->

<!-- For CLI changes: -->
```bash
# Example command output
$ llm-costops new-command
Output here...
```

<!-- For API changes: -->
```json
{
  "new_field": "value"
}
```

## Reviewer Notes

<!-- Any specific areas you'd like reviewers to focus on? -->

### Focus Areas

<!-- What should reviewers pay special attention to? -->

-
-

### Questions for Reviewers

<!-- Any specific questions or concerns? -->

-
-

## Database Migrations

<!-- If this includes database changes -->

- [ ] No database changes
- [ ] Database migration included
- [ ] Migration tested (up and down)
- [ ] Migration documented

<!-- If yes, describe the migration: -->

**Migration details:**

```sql
-- Migration SQL
ALTER TABLE usage_records ADD COLUMN new_field TEXT;
```

## Deployment Notes

<!-- Any special deployment considerations -->

- [ ] No special deployment steps required
- [ ] Requires configuration changes (describe below)
- [ ] Requires database migration
- [ ] Requires service restart
- [ ] Has backward compatibility considerations

<!-- If yes, provide details: -->

**Deployment steps:**

1.
2.
3.

## Additional Context

<!-- Any additional information that reviewers should know -->

## Post-Merge Tasks

<!-- Tasks to complete after merging (if any) -->

- [ ] Update production configuration
- [ ] Announce in Discord/community
- [ ] Create follow-up issues
- [ ] Update related documentation
- [ ] Monitor for issues in production

---

<!--
Thank you for contributing to LLM-CostOps!
Please ensure all checkboxes are complete before requesting review.
-->
