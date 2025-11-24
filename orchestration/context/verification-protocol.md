# Verification Protocol

This file defines the verification steps that replace git hook-based enforcement. All verification is now instruction-based, meaning YOU (the LLM) must run these checks at the appropriate times.

## Why Instruction-Based Verification

**Previous approach (git hooks):**
- Hooks installed in `.git/hooks/` don't propagate to git clones
- Requires special setup on each new machine
- Pre-push hook blocked ALL pushes until 100% complete (violated Rule 8)

**Current approach (instruction-based):**
- Instructions travel with the repository in CLAUDE.md
- No special setup required on new clones
- Allows incremental commits/pushes after each task/phase
- Same verification logic, different trigger mechanism

## Verification Scripts

The verification scripts are located in `orchestration/hooks/` and remain available for use:

| Script | Purpose | When to Run |
|--------|---------|-------------|
| `pre_commit_naming.py` | Validates component names | Before commits |
| `pre_commit_enforcement.py` | Advisory checks for quality | Before commits |
| `pre_commit_completion_blocker.py` | Verifies completion claims | Before committing completion reports |
| `post_commit_enforcement.py` | Shows continuation status | After commits |

---

## Pre-Commit Verification

**Run BEFORE every `git commit`:**

```bash
# 1. Validate component naming (if components changed)
python orchestration/hooks/pre_commit_naming.py

# 2. Run advisory enforcement checks
python orchestration/hooks/pre_commit_enforcement.py
```

### What These Check

**Naming validation:**
- Component names follow `^[a-z][a-z0-9_]*$` pattern
- No hyphens (breaks Python imports)

**Advisory enforcement:**
- Task queue status (how many tasks remain)
- Rationalization language detection
- Stub/placeholder code detection

### Handling Results

- **Naming violations**: Fix before committing
- **Advisory warnings**: Note them, proceed with commit to preserve progress
- **Stub code warnings**: Acceptable during development, must be resolved before completion

---

## Post-Commit Verification

**Run AFTER every `git commit`:**

```bash
python orchestration/hooks/post_commit_enforcement.py
```

### What This Does

- Shows current task queue status
- Displays next task to work on
- Provides continuation guidance

### Expected Behavior

- If tasks remain: Shows "Continue with: [next task]"
- If queue empty but not verified: Shows "Run verification"
- If complete and verified: Shows success message

---

## Pre-Push Verification

**Run BEFORE every `git push`:**

```bash
# Verify current work is in good state
python orchestration/verification/verify_push_ready.py
```

### Push Policy (Rule 8 Compliant)

**You MAY push when:**
- Current task/phase is complete and committed
- Tests for completed work are passing
- No critical verification failures

**You are NOT blocked from pushing incomplete work** - the goal is progress preservation.

### Recommended Pre-Push Checklist

1. Current task status: Complete or well-documented WIP
2. Tests passing for completed portions
3. No broken imports or syntax errors
4. Commit messages are descriptive

---

## Phase Boundary Verification

**Run at every phase transition:**

```bash
python orchestration/gates/runner.py . {phase_number}
```

### Phase Gates

| Phase | Gate Purpose |
|-------|--------------|
| Phase 5 | Integration testing - 100% pass rate required |
| Phase 6 | Completion verification - All checks must pass |

### Gate Results

- **Exit 0**: Gate passed, may proceed to next phase
- **Exit 1**: Gate failed, fix issues and re-run

### After Running Gates

Include gate output in your response:
```
Running Phase 5 gate...
$ python orchestration/gates/runner.py . 5

[PASTE ACTUAL OUTPUT HERE]

Gate result: PASSED/FAILED
```

---

## Completion Verification

**Run BEFORE declaring any component or project complete:**

### Component Completion

```bash
python orchestration/verification/completion_verifier.py components/[name]
```

All 16 checks must pass.

### Project Completion

```bash
python orchestration/verification/run_full_verification.py
```

### Before Committing Completion Reports

If committing `COMPLETION-REPORT.md` or similar:

```bash
python orchestration/hooks/pre_commit_completion_blocker.py
```

This verifies:
- All phase gates passed
- Specification coverage is 100%
- Verification agent approved
- Smoke tests passed
- No stub components

---

## Task/Phase Commit-Push Workflow

### After Completing Each Task

```
1. Run pre-commit verification
   $ python orchestration/hooks/pre_commit_enforcement.py

2. Commit the work
   $ git add .
   $ git commit -m "feat(component): complete task description"

3. Run post-commit to see status
   $ python orchestration/hooks/post_commit_enforcement.py

4. Push to preserve progress (Rule 8)
   $ git push origin branch

5. Proceed to next task
```

### After Completing Each Phase

```
1. Run phase gate
   $ python orchestration/gates/runner.py . {phase}

2. If gate passes:
   - Commit phase completion
   - Push to remote
   - Proceed to next phase

3. If gate fails:
   - Review gate output
   - Fix identified issues
   - Re-run gate
   - Repeat until passing
```

---

## Quick Reference

### Before Commit
```bash
python orchestration/hooks/pre_commit_naming.py
python orchestration/hooks/pre_commit_enforcement.py
```

### After Commit
```bash
python orchestration/hooks/post_commit_enforcement.py
```

### Before Push
- Verify current task complete or well-documented
- No blocking issues

### At Phase Boundaries
```bash
python orchestration/gates/runner.py . {phase}
```

### Before Declaring Complete
```bash
python orchestration/verification/run_full_verification.py
```

---

## Reverting to Git Hooks (Optional)

If instruction-based verification proves insufficient, git hooks can be re-enabled:

```bash
# Install pre-commit framework
pip install pre-commit

# Configure hooks
python orchestration/setup/precommit_config.py .

# Install hooks
pre-commit install
```

This will restore automatic execution of verification on git events.
