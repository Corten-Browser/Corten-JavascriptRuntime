---
description: "Internal: Phase 5 integration testing (called by /orchestrate)"
---

# Internal: Phase 5 Integration Testing

**This is an internal command called by `/orchestrate` via subagent.**
**Do not invoke directly unless you understand the orchestration workflow.**

## Purpose

Run integration tests with absolute 100% pass rate and 100% execution rate requirements.
This command implements Phase 5 of the Level 3 orchestration workflow.

## Prerequisites

This command expects:
- Phase 4.5 (Contract Validation) passed
- Components exist in `components/` directory
- Integration tests exist in `tests/integration/`
- `orchestration/verification/` tools available

## Phase Skip Check (Resume Support)

```python
# If resuming and current_phase > 5, skip this phase
if 'resuming' in globals() and resuming and current_phase > 5:
    print("⏭️  Skipping Phase 5 (already completed)")
    # Report back and continue to Phase 6
```

---

## MANDATORY REQUIREMENTS

- Integration tests MUST achieve 100% pass rate
- Integration tests MUST achieve 100% execution rate (no "NOT RUN")
- Even ONE failure = STOP - DO NOT PROCEED
- Even ONE "NOT RUN" = STOP - FIX AND RE-TEST ALL
- No exceptions, no overrides, no justifications

**Why 100% Execution AND 100% Pass is Required:**
- Integration tests verify components can communicate
- One API mismatch can break entire system
- 99% pass rate can still mean 0% functional
- Historical failures: 79.5% pass = 0% functional, 37.5% execution = critical bugs undetected

---

## The Iterative Fix Loop

```
┌─────────────────────────────────────────────────────────────┐
│ START: Launch Integration Test Agent                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
            ┌────────────────────────┐
            │ Run ALL Integration    │
            │ Tests                  │
            └────────┬───────────────┘
                     │
                     ▼
         ┌───────────────────────┐
         │ Check Results:        │
         │ - Execution rate?     │
         │ - Pass rate?          │
         │ - Any NOT RUN?        │
         └───┬───────────────────┘
             │
    ┌────────┴────────┬──────────────────┬────────────────┐
    │                 │                  │                │
    ▼                 ▼                  ▼                ▼
OUTCOME 1:        OUTCOME 2:          OUTCOME 3:      OUTCOME 4:
100% execution    < 100% execution   Some tests      Tests exist
100% pass         Some PASS           FAILED          but CRASHED
                  Some NOT RUN

    │                 │                  │                │
    ▼                 ▼                  ▼                ▼
✅ PROCEED        🔁 FIX & RE-RUN    🔁 FIX & RE-RUN  🔁 FIX & RE-RUN
to Phase 6        ALL tests          ALL tests        ALL tests
                  (MANDATORY)        (MANDATORY)      (MANDATORY)
```

**Three Possible States**:
1. **PASS** (✅): Test executed and succeeded
2. **FAIL** (❌): Test executed but failed (API mismatch, contract violation, etc.)
3. **NOT RUN** (⏭️): Test blocked by earlier failure (UNACCEPTABLE)

**Only OUTCOME 1 allows progression.** All other outcomes require fixes and complete re-testing.

---

## Step 1: Validate Integration Tests

**BEFORE running integration tests**, validate that tests follow no-mocking policy:

```bash
echo "===== Validating Integration Tests ====="
python orchestration/verification/quality/integration_test_validator.py --strict

if [ $? -ne 0 ]; then
    echo "❌ Integration test validation FAILED"
    echo "Fix critical issues before running tests"
    exit 1
fi

echo "✅ Integration tests validated"
```

**Expected Output**:
```
✅ Integration test validation PASSED
   All tests use real components (no mocking detected)
```

**If Validation Fails**:
```
❌ Integration test validation FAILED
   Found 2 critical issues, 1 warnings

CRITICAL ISSUES (must fix before proceeding):
------------------------------------------------------------

[CRITICAL] test_cli_workflow.py:5
  Integration tests must NOT import unittest.mock.Mock
  Code: from unittest.mock import Mock
  See: docs/TESTING-STRATEGY.md - When Mocks Are FORBIDDEN

[CRITICAL] test_audio_loading.py:12
  Integration tests must NOT use @patch for components being tested
  Code: @patch('components.audio_loader.AudioLoader')
  See: docs/TESTING-STRATEGY.md - Cross-Component Integration Testing

============================================================
RESULT: BLOCKED - Fix critical issues before proceeding
```

**Recovery Process for Validation Failures**:
1. STOP - Do not run integration tests yet
2. Review validation failures
3. Fix issues (remove mocks, use real components)
4. Re-run validator
5. Only proceed when validation passes

---

## Step 2: Run Integration Tests

**Launch Integration Test Agent:**
```python
Task(
    description="Create and run cross-component integration tests",
    prompt="""Read claude-orchestration-system/templates/integration-test-agent.md
    for complete instructions. Create integration tests in tests/integration/,
    E2E tests in tests/e2e/, run pytest, report results in TEST-RESULTS.md

    CRITICAL: Report EXACT pass rate. Even 99% = FAILURE.""",
    subagent_type="general-purpose"
)
```

**Or run directly:**
```bash
cd tests/integration
pytest -v > TEST-RESULTS.md 2>&1
echo "Exit code: $?"
```

---

## Step 3: Analyze Results

**Failure Interpretation Rules:**
1. ANY AttributeError → Component API mismatch → CRITICAL
2. ANY TypeError → Signature mismatch → CRITICAL
3. ANY ImportError → Component structure issue → CRITICAL
4. NEVER classify integration failures as "test issues"
5. NEVER proceed with even one failure

**Integration Test Interpretation Rules - ZERO TOLERANCE:**

| Error Pattern | Meaning | Severity | Action |
|--------------|---------|----------|---------|
| `AttributeError: 'X' object has no attribute 'Y'` | API mismatch | **CRITICAL** | Fix component API NOW |
| `TypeError: X() takes Y arguments but Z were given` | Signature mismatch | **CRITICAL** | Fix method signature NOW |
| `ImportError: cannot import name 'X'` | Missing implementation | **CRITICAL** | Implement missing class NOW |
| `KeyError` in integration test | Contract violation | **CRITICAL** | Fix data structure NOW |

---

## Step 4: Fix Failures (Iterative)

**Recovery Process for ANY Failure:**
1. STOP all forward progress immediately
2. Treat EVERY failure as CRITICAL
3. Review TEST-RESULTS.md for error patterns
4. Identify failing component pairs:
   - AttributeError: 'X' object has no attribute 'Y' = API mismatch
   - TypeError: Wrong arguments = Signature mismatch
   - ImportError: Cannot import = Structure issue
5. Launch sub-agents to fix ALL issues
6. Re-run entire integration suite
7. Only proceed with 100% pass rate
8. If cannot achieve 100%: Reduce scope, not quality

**Mandatory Re-Test Protocol - CRITICAL:**

**THE RULE**: After fixing ANY integration test failure, you MUST re-run the ENTIRE integration test suite.

**Why This Is Critical**:
- Historical failure: Fixed test #3, but NEVER re-ran tests #4-8
- Result: 5 tests left in "NOT RUN" status (37.5% execution)
- Critical bug (method name mismatch) went undetected
- System declared complete with only 67% pass rate

**Required Workflow After ANY Fix**:
```
1. Fix identified issue
2. Run affected component tests (verify fix works)
3. MANDATORY: Re-run ENTIRE integration suite
   cd tests/integration
   pytest -v > TEST-RESULTS.md 2>&1
4. Verify results:
   - 100% execution rate (all tests ran)
   - 0 tests in "NOT RUN" status
   - 100% pass rate (all tests passed)
5. If ANY test still fails or NOT RUN → Return to step 1
6. Only proceed when ALL tests execute AND pass
```

**NEVER Do This** (historical mistake):
```
❌ Fix test #3 → Tests 3/8 pass → Declare complete  [WRONG]
❌ "5 tests didn't run because of earlier failure"   [WRONG]
❌ "We'll fix those later"                           [WRONG]
```

**ALWAYS Do This**:
```
✅ Fix test #3 → Re-run ALL 8 tests → Verify 100% execution → Verify 100% pass
✅ Fix blocking failure → Re-test everything → Confirm nothing else broken
✅ After ANY change → Full integration suite → Complete verification
```

---

## Step 5: Verify 100% Requirements

**Checking Execution Coverage**:
```bash
# Use integration coverage checker to verify
python orchestration/verification/quality/integration_coverage_checker.py --strict

# Must show:
# ✅ Execution Rate: 100%
# ✅ Tests NOT RUN: 0
# ✅ Pass Rate: 100%
```

**Expected Output for Success**:
```
╔══════════════════════════════════════════════════════════╗
║      Integration Test Execution Coverage Report         ║
╚══════════════════════════════════════════════════════════╝

Total Tests Planned:  12
Tests Executed:       12
Tests Passed:         12
Tests Failed:         0
Tests NOT RUN:        0

Execution Rate:       100.0%
Pass Rate:            100.0%
Overall Rate:         100.0%

✅ SYSTEM READY FOR PHASE 6
```

---

## Premature Stopping Prevention

**CRITICAL**: Do NOT stop at integration test failures and report status. FIX THEM.

**Integration Test Failures Are NOT Stopping Points:**

| If You See... | Don't Do This... | Do This Instead... |
|---------------|------------------|--------------------|
| 0/4 tests pass | "Report 95% complete" | "Fix failures, continue" |
| API mismatch errors | "Ask user for direction" | "Launch agent to fix API" |
| Type conversion issues | "Wait for review" | "Fix types, re-run tests" |
| Missing parameters | "Request clarification" | "Add parameters, re-test" |

**Valid Stopping Reasons in Phase 5:**
1. ✅ 100% integration test pass rate achieved → Proceed to Phase 6
2. ✅ Specification contradiction discovered → Ask user for clarification
3. ✅ External API permanently down → Report blocking issue

**Invalid Stopping Reasons in Phase 5:**
1. ❌ "Some tests failing" → Fix them
2. ❌ "95% complete" → Finish the remaining 5%
3. ❌ "API mismatches found" → Fix them autonomously
4. ❌ "Should I continue?" → Yes, always
5. ❌ "User might want to review" → User wants 100% complete

**Remember**: The goal is 100% functional and deployment-ready, not "mostly done."

---

## Phase 5 Completion Gate

Before proceeding to Phase 6, verify:
- [ ] Integration tests: 100% execution rate
- [ ] Integration tests: 100% pass rate
- [ ] Integration tests: 0 "NOT RUN" status
- [ ] No API mismatches remain
- [ ] No blocking errors remain

**If ANY item is unchecked, DO NOT proceed to Phase 6. FIX IT FIRST.**

**Do NOT**:
- Create status report with "X% complete"
- Stop and wait for user approval
- Ask "should I continue?"

**Correct behavior**: Continue fixing until ALL items are checked, then proceed to Phase 6 automatically.

---

## Checkpoint Integration

**Save Phase 5 completion:**
```python
orchestrator.complete_phase(
    phase_number=5,
    outputs={
        "phase_name": "Integration Testing",
        "integration_test_pass_rate": "100%",
        "integration_test_execution_rate": "100%"
    }
)
```

---

## Reporting Back to Orchestrator

**After completing all steps, report back with this structure:**

**Success case (proceed to Phase 6):**
```json
{
    "phase": 5,
    "phase_name": "Integration Testing",
    "execution_rate": "100%",
    "pass_rate": "100%",
    "not_run_count": 0,
    "total_tests": 12,
    "tests_passed": 12,
    "tests_failed": 0,
    "failures": [],
    "test_results_summary": "All 12 integration tests passed",
    "iterations_required": 1,
    "can_proceed": true
}
```

**Failure case (still working):**
```json
{
    "phase": 5,
    "phase_name": "Integration Testing",
    "execution_rate": "75%",
    "pass_rate": "67%",
    "not_run_count": 3,
    "total_tests": 12,
    "tests_passed": 8,
    "tests_failed": 1,
    "failures": [
        {
            "test": "test_cli_workflow.py::test_analyze_command",
            "error": "AttributeError: 'AudioLoader' object has no attribute 'load_audio'",
            "component": "audio_loader",
            "fix_required": "Rename method from 'load' to 'load_audio' in audio_loader component"
        }
    ],
    "test_results_summary": "8/12 passed, 1 failed, 3 not run due to blocking failure",
    "iterations_required": 2,
    "can_proceed": false,
    "next_action": "Fix AudioLoader.load_audio method, re-run ENTIRE suite"
}
```

**Blocked case (validation failed):**
```json
{
    "phase": 5,
    "phase_name": "Integration Testing",
    "validation_passed": false,
    "validation_errors": [
        "test_cli_workflow.py uses unittest.mock.Mock (forbidden)",
        "test_audio_loading.py uses @patch decorator (forbidden)"
    ],
    "execution_rate": null,
    "pass_rate": null,
    "can_proceed": false,
    "next_action": "Remove mocks from integration tests, use real components"
}
```

**CRITICAL**:
- Do NOT report partial success as completion
- Keep iterating until 100% execution AND 100% pass
- Include specific failure details for debugging
- Always include `next_action` if `can_proceed` is false
