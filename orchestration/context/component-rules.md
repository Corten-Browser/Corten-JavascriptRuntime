# Component Development Rules

This file contains the generic rules for all component development. Read this alongside your component's CLAUDE.md and component.yaml.

## Version Control Restrictions
**FORBIDDEN ACTIONS:**
- ❌ NEVER change project version to 1.0.0
- ❌ NEVER declare system "production ready"
- ❌ NEVER change lifecycle_state

**ALLOWED:**
- ✅ Report test coverage and quality metrics
- ✅ Complete your component work
- ✅ Suggest improvements

## Component Naming Convention

All component names MUST follow: `^[a-z][a-z0-9_]*$`
- Lowercase letters, numbers, underscores only
- No hyphens (breaks Python imports)
- Works across all programming languages

## MANDATORY: Test-Driven Development (TDD)

### TDD is Not Optional

**ALL code MUST be developed using TDD.** This is a strict requirement.

### TDD Workflow (Red-Green-Refactor)

**Follow this cycle for EVERY feature:**

1. **RED**: Write a failing test
   - Write the test FIRST before any implementation
   - Run and verify it FAILS
   - The test defines the behavior

2. **GREEN**: Make the test pass
   - Write MINIMUM code to pass
   - No extra features or "nice to have" code
   - Verify it PASSES

3. **REFACTOR**: Improve the code
   - Clean up duplication
   - Improve naming and structure
   - Maintain passing tests throughout

### TDD Commit Pattern

Your git history MUST show TDD practice:
```bash
# Example sequence
git commit -m "[component_name] test: Add failing test for endpoint"  # RED
git commit -m "[component_name] feat: Implement endpoint"              # GREEN
git commit -m "[component_name] refactor: Extract validation logic"    # REFACTOR
```

## Component Boundaries

### What You CAN Do:
- ✅ Import other components' PUBLIC APIs
- ✅ Use other components as dependencies
- ✅ Call public functions/classes from other components
- ✅ Read contracts from `../../contracts/`
- ✅ Read shared libraries from `../../shared-libs/`
- ✅ Work ONLY in your component directory

### What You CANNOT Do:
- ❌ Access other components' PRIVATE implementation
- ❌ Import from `_internal/` or `private/` subdirectories
- ❌ Modify files outside your component directory
- ❌ Depend on implementation details not in public API

## Component Manifest (component.yaml)

All components MUST declare their public API in `component.yaml`:

```yaml
name: component_name
type: backend  # backend, frontend, library
version: 0.1.0

user_facing_features:
  public_api:
    - class: MainClass
      module: components.component_name.src.module
      methods: [method1, method2]
      description: Main interface
```

**Why This Matters**: Feature coverage testing validates EVERY declared API.

## Context Window Management

### Size Limits
- **Optimal**: < 80,000 tokens (~8,000 lines)
- **Warning**: 80,000-100,000 tokens
- **Critical**: > 100,000 tokens - ALERT ORCHESTRATOR
- **NEVER EXCEED**: 120,000 tokens

### Your Responsibilities
1. Check size before starting work
2. Monitor file additions during development
3. Alert orchestrator if approaching limits

## Quality Checklist

Before marking ANY task complete:

### Code Quality
- [ ] TDD cycle visible in git history
- [ ] Test coverage ≥ 80%
- [ ] All tests pass - 100% pass rate
- [ ] Zero linting errors
- [ ] Code formatted
- [ ] Function complexity ≤ 10
- [ ] No hardcoded values

### Documentation
- [ ] Public functions have docstrings
- [ ] README.md updated if API changed
- [ ] Inline comments for non-obvious logic

### API Contract
- [ ] Endpoint matches contract exactly
- [ ] Request/response schemas match
- [ ] HTTP status codes match
- [ ] Error responses match format

### Security
- [ ] All user input validated
- [ ] No SQL injection (parameterized queries)
- [ ] No secrets in code
- [ ] Sensitive data not logged

### Performance
- [ ] No N+1 query problems
- [ ] Database indexes on queried fields
- [ ] Pagination for list endpoints

## Git Procedures

### You work in a SHARED repository

**Repository**: Single git repository at project root
**Your location**: `components/[your_component]/`

### Verification Before Git Operations

**IMPORTANT**: Run verification scripts before git operations. See `orchestration/context/verification-protocol.md` for full details.

**Before committing:**
```bash
# Validate component naming
python orchestration/hooks/pre_commit_naming.py

# Run advisory checks
python orchestration/hooks/pre_commit_enforcement.py
```

**After committing:**
```bash
# Check continuation status
python orchestration/hooks/post_commit_enforcement.py
```

### Committing Your Work

**Option 1: Using retry wrapper (recommended)**
```bash
python ../../orchestration/cli/git_retry.py "component_name" "feat: Description"
```

**Option 2: Direct git**
```bash
git add components/your_component/
git commit -m "[your_component] feat: Description"
```

### Commit Message Format
```
[component_name] <type>: <description>
```
Types: feat, fix, test, docs, refactor, chore

### IMPORTANT Git Rules
1. ✅ ONLY stage files in your component directory
2. ✅ ALWAYS use component name prefix in commits
3. ✅ Use retry wrapper to handle lock conflicts
4. ✅ Run verification scripts before commits
5. ❌ NEVER modify files outside your directory
6. ❌ NEVER run 'git add .' from root

## Development Workflow

### Before Writing Any Code
1. Read the API contract thoroughly
2. Understand requirements completely
3. Plan the implementation

### TDD Implementation (For Each Feature)
1. Write Integration Test (RED) → Expected: FAIL
2. Implement Minimum Code (GREEN) → Expected: PASS
3. Write Unit Tests (RED) → Expected: FAIL
4. Implement Business Logic (GREEN) → Expected: PASS
5. Refactor → All tests must still pass
6. Commit with retry wrapper

### Before Requesting Review
1. Run full test suite: `pytest`
2. Check coverage: `pytest --cov=src --cov-report=term-missing`
3. Run linter
4. Verify contract compliance

## Test Quality Standards

### What Makes a Good Test
- Tests behavior, not implementation
- One assertion concept per test
- Descriptive test names
- Proper setup/teardown

### Over-Mocking is FORBIDDEN
- ❌ DO NOT mock your own domain logic
- ❌ DO NOT mock validators, repositories, or transformers you own
- ✅ Mock external APIs and services
- ✅ Mock time/date functions
- ✅ Mock paid third-party services

### Integration Test Requirements
- Must use real components (no mocking internal code)
- 100% pass rate required
- 100% execution rate required (no "NOT RUN")

## Breaking Changes Policy

**Version < 1.0.0 = Breaking Changes ENCOURAGED**

**ALWAYS PREFER:**
- ✅ Clean code over backwards compatibility
- ✅ Removing deprecated code immediately
- ✅ Breaking changes that improve design
- ✅ Deleting unused code paths

**NEVER DO:**
- ❌ Add deprecation warnings for unreleased features
- ❌ Keep unused code "for backwards compatibility"
- ❌ Add compatibility layers during development

## Completion Verifier

Before declaring complete, run:
```bash
python orchestration/verification/completion_verifier.py components/[your_component]
```

All 16 checks must pass for component to be considered complete.