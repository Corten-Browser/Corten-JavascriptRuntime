---
description: "Autonomously orchestrate complete project development with detailed planning (full version)"
---

# Autonomous Project Orchestration - Detailed Execution

## ⚠️ CRITICAL VERSION CONTROL RESTRICTIONS ⚠️
**ABSOLUTELY FORBIDDEN:**
- ❌ NEVER change version to 1.0.0
- ❌ NEVER declare system "production ready"
- ❌ NEVER change lifecycle_state from "pre-release"

**ALLOWED:**
- ✅ Minor/patch versions (0.2.0, 0.1.1)
- ✅ Quality assessment reports
- ✅ Readiness documentation

## 🎯 MODEL STRATEGY
**Your Model**: Sonnet (default) or Opus (user's choice)
**Sub-Agent Model**: ALWAYS Sonnet (you must enforce)

**CRITICAL**: Always specify `model="sonnet"` when launching Task tools.
Never let sub-agents inherit Opus (no coding benefit at 5x cost).

## 🧠 THINKING STRATEGY

**Use extended thinking for:**
- Phase 1: Architecture planning (16K budget - "think hard")
- Component split decisions (16K budget - "think hard")
- Complex sub-agent tasks (8K budget - "think" in prompt)

**Disable thinking for:**
- Routine agent coordination
- Status monitoring
- Documentation generation
- Straightforward implementations

PROJECT ORCHESTRATION - AUTONOMOUS EXECUTION MODE

Specifications are in root directory markdown files.

Execute complete autonomous project development following this workflow:

## PHASE 1: ANALYSIS & ARCHITECTURE (Think hard - this is critical)

Think hard about the optimal architecture considering:
- Component boundaries and token budgets
- Dependency relationships
- Contract interfaces
- Scalability and maintainability

- **IMMEDIATE: Check all existing component sizes**
  * If ANY component > 100,000 tokens: Schedule for splitting
  * If ANY component > 120,000 tokens: STOP - split BEFORE proceeding
- Read all specification documents
- Analyze requirements, features, architecture
- Design multi-component architecture with:
  * Component boundaries (keep each <80k tokens optimal, NEVER >120k)
  * Technology stack per component
  * Inter-component contracts
  * Shared libraries
  * Dependency graph
- Document the architecture plan
- PROCEED IMMEDIATELY to Phase 2 (do not wait for approval)

## PHASE 2: COMPONENT CREATION (Execute immediately)
- Create all necessary components using dynamic component creation workflow
- For each component:
  * Create directory structure
  * Generate CLAUDE.md from template with component-specific instructions
  * Create README.md
  * Component files tracked in single project git repository
- After ALL components created, display:
  ```
  ✅ Components created: [list component names]

  Proceeding immediately to Phase 3 (contracts & setup)...
  ```
- PROCEED IMMEDIATELY to Phase 3 (no restart, no pauses)

## PHASE 3: CONTRACTS & SETUP (Execute immediately)
- Create all API contracts in contracts/ directory
- Define shared libraries in shared-libs/
- Validate all component configurations

## PHASE 4: PARALLEL DEVELOPMENT (Execute immediately)
- **PRE-FLIGHT CHECKS (MANDATORY before launching EACH agent):**
  * Verify component size < 90,000 tokens (adjusted for ~10,400 token CLAUDE.md overhead)
  * If component > 90,000 tokens: Split FIRST, then launch agent
  * Estimate task will not exceed 110,000 tokens
  * Ensure total context needed < 130,000 tokens (emergency limit)
- **Read agent limits from orchestration/orchestration-config.json:**
  * max_parallel_agents: 5 (default maximum)
  * warn_above: 7 (show info message)
  * recommended_max: 10 (performance sweet spot)
  * absolute_max: 15 (hard cap - error if exceeded)
- **Validate agent count before launching:**
  * Use: `from orchestration.context_manager import ContextWindowManager`
  * Call: `manager.validate_concurrent_agents(planned_agent_count)`
  * Check result['valid'] - if False, reduce count or queue work
  * Display result['message'] and result['recommendation'] to user
- Use Task tool to launch component agents in parallel
- Respect validated concurrency limit (queue overflow work if needed)
- **Each Task tool invocation MUST include:**
  * `model="sonnet"` parameter (REQUIRED - forces Sonnet for coding)
  * Component-specific prompt (use template from CLAUDE.md)
  * Instruction to read components/X/CLAUDE.md
  * Directory isolation enforcement (work ONLY in components/X/)
  * Complete task requirements per specifications
- **Example Task invocation (complex logic - thinking enabled):**
  ```python
  Task(
      description="Implement auth-service component",
      prompt="""Read components/auth-service/CLAUDE.md.

      Think about security implications, token management, and session handling.

      Implement authentication system with JWT, refresh tokens, and session management.
      Follow TDD, achieve 80%+ coverage.""",
      subagent_type="general-purpose",
      model="sonnet"
  )
  ```

- **Example Task invocation (routine work - thinking disabled):**
  ```python
  Task(
      description="Implement user CRUD operations",
      prompt="""Read components/user-api/CLAUDE.md.

      Implement standard CRUD endpoints for user management.
      Follow existing repository patterns, write tests.""",
      subagent_type="general-purpose",
      model="sonnet"
  )
  ```
- Each component agent:
  * Implements full feature set per specs
  * Follows TDD (tests before code)
  * Achieves 80%+ test coverage
  * Maintains quality standards
  * Commits work via retry wrapper: python orchestration/git_retry.py "component-name" "message"
- Monitor agent progress and collect results
- Verify agents stayed within boundaries
- Run quality verification on completed work
- **CONTINUOUS MONITORING:**
  * If any component approaches 100,000: Pause, split, resume
  * NEVER allow component to exceed 120,000 tokens
  * Abort operations that would exceed context limits

**Phase 4 Completion Gate [v0.8.0]:**

Before proceeding to Phase 4.5, verify:
- ✅ All components implemented (100% of planned components)
- ✅ All component tests passing (100% pass rate per component)
- ✅ All agents completed and reported success
- ✅ No components exceed size limits (< 120K tokens)
- ✅ Quality verification passed for all components

**If ANY item is ✗, DO NOT proceed. FIX IT FIRST.**

**Do NOT**:
- ❌ Create status report with "X% complete"
- ❌ Stop and wait for user approval
- ❌ Ask "should I continue to contract validation?"

**Correct behavior**: Continue fixing until ALL items are ✅, then proceed to Phase 4.5 automatically.

## PHASE 4.5: CONTRACT VALIDATION - PRE-INTEGRATION GATE (Execute immediately)

**PURPOSE**: Catch API mismatches BEFORE running integration tests

**CRITICAL**: Validate each component implements its contract EXACTLY before integration testing.

**Validation Steps**:
1. **Run Contract Tests** (100% pass required):
   ```bash
   # For each component
   cd components/[component-name]
   pytest tests/contracts/ -v
   ```

2. **Contract Validation Tool**:
   ```bash
   # Automated validation across all components
   python orchestration/contract_validator.py --all
   ```

3. **Check for Common API Mismatches**:
   - Method name mismatches (scan vs get_audio_files)
   - Parameter name mismatches (directory vs path)
   - Singular/plural mismatches (generate_playlist vs generate_playlists)
   - Verb mismatches (store vs save)
   - Return type mismatches

**Failure Response**:
- ❌ ANY contract test failure = STOP immediately
- Fix the component implementation (NOT the contract)
- Re-run contract tests until 100% pass
- Only proceed when ALL components pass contract validation

**Why This Phase Exists**:
- Music Analyzer: Components had wrong method names
- Contract validation would have caught `scan()` vs `get_audio_files()` mismatch
- Prevents wasting time on integration tests when basic APIs are wrong
- Faster feedback loop than full integration testing

**Expected Output**:
```
✅ auth-service: Contract tests 100% pass (12/12)
✅ file-scanner: Contract tests 100% pass (8/8)
✅ playlist-generator: Contract tests 100% pass (10/10)
✅ database-manager: Contract tests 100% pass (6/6)

✅ ALL COMPONENTS PASS CONTRACT VALIDATION - Proceeding to integration testing
```

---

### Active Contract Method Validation [v0.10.0 - OPTIONAL]

**PURPOSE**: Catch API mismatches BEFORE integration testing (defense-in-depth).

**OPTIONAL**: This check is optional but recommended. Integration test validation (Phase 5) catches most issues, but this provides faster feedback.

**For each component**, validate that method calls match dependency contracts:

```bash
for component in cli-interface audio-loader rhythm-analyzer; do
    echo "Validating $component method calls..."
    python orchestration/contract_method_validator.py --component $component

    if [ $? -ne 0 ]; then
        echo "❌ Contract violations found in $component"
        echo "Fix before proceeding to integration tests"
        exit 1
    fi
done
```

**Expected Output**:
```
Validating cli-interface method calls...
✅ Contract method validation PASSED
   All method calls match contract specifications
```

**If Violations Found**:
```
❌ Contract method validation FAILED
   Found 2 contract violations

[cli-interface] api.py:42
  Called: audio_loader.load()
  Problem: Method not in audio_loader contract
  Available methods: load_audio, get_metadata, close
  💡 Did you mean: load_audio()?

[cli-interface] api.py:67
  Called: data_manager.export_to_spreadsheet()
  Problem: Method not in data_manager contract
  Available methods: export_to_excel, save_results
  💡 Did you mean: export_to_excel()?

============================================================
RESULT: BLOCKED - Fix contract violations before proceeding
```

**Why This Helps**:
- Catches Brain Music Analyzer failure mode (load vs load_audio)
- Faster feedback than waiting for integration tests
- Suggests correct method names
- Prevents wasted time on integration tests when basic APIs are wrong

**Note**: This is optional if implementation complexity is high. Integration test validation (Phase 5) catches most issues. This adds defense-in-depth.

---

**If Failures Occur**:
```
❌ file-scanner: Contract tests FAILED (7/8 pass)
  - CRITICAL: test_exact_api_compliance FAILED
  - Expected method: scan()
  - Found method: get_audio_files()

🛑 STOP - Fix file-scanner API before integration testing
```

## PHASE 5: INTEGRATION TESTING - ABSOLUTE GATE (Execute immediately)

**MANDATORY REQUIREMENT:**
- Integration tests MUST achieve 100% pass rate
- Integration tests MUST achieve 100% execution rate (no "NOT RUN")
- Even ONE failure = STOP - DO NOT PROCEED
- Even ONE "NOT RUN" = STOP - FIX AND RE-TEST ALL
- No exceptions, no overrides, no justifications

**Why 100% Execution AND 100% Pass is Required:**
- Integration tests verify components can communicate
- One API mismatch can break entire system
- 99% pass rate can still mean 0% functional
- Music Analyzer Failure #1: 79.5% pass = 0% functional
- Music Analyzer Failure #2: 37.5% execution = critical bugs undetected

**Phase 5 Is Iterative - Expect Multiple Cycles:**

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

**Only OUTCOME 1 allows progression**. All other outcomes require fixes and complete re-testing.

**Failure Interpretation Rules:**
1. ANY AttributeError → Component API mismatch → CRITICAL
2. ANY TypeError → Signature mismatch → CRITICAL
3. ANY ImportError → Component structure issue → CRITICAL
4. NEVER classify integration failures as "test issues"
5. NEVER proceed with even one failure

**Launch Integration Test Agent:**
```python
Task(
    description="Create and run cross-component integration tests",
    prompt="""Read claude-orchestration-system/templates/integration-test-agent.md
    for complete instructions. Create integration tests in tests/integration/,
    E2E tests in tests/e2e/, run pytest, report results in TEST-RESULTS.md

    CRITICAL: Report EXACT pass rate. Even 99% = FAILURE.""",
    subagent_type="general-purpose",
    model="sonnet"
)
```

---

### Integration Test Validation [v0.10.0]

**BEFORE running integration tests**, validate that tests follow no-mocking policy:

```bash
echo "===== Validating Integration Tests ====="
python orchestration/integration_test_validator.py --strict

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
4. Re-run validator: `python orchestration/integration_test_validator.py --strict`
5. Only proceed when validation passes

**Why This Check**:
- Catches mocking anti-pattern BEFORE tests run
- Prevents Brain Music Analyzer failure mode (100% mocked tests passing)
- Forces tests to use real components (API mismatches will be caught)
- Integration tests must test REAL integration, not mocked behavior

---

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

**Mandatory Re-Test Protocol - CRITICAL [v0.7.0]:**

**THE RULE**: After fixing ANY integration test failure, you MUST re-run the ENTIRE integration test suite.

**Why This Is Critical**:
- Music Analyzer Failure #2: Fixed test #3, but NEVER re-ran tests #4-8
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

**Checking Execution Coverage**:
```bash
# Use integration coverage checker to verify
python orchestration/integration_coverage_checker.py --strict

# Must show:
# ✅ Execution Rate: 100%
# ✅ Tests NOT RUN: 0
# ✅ Pass Rate: 100%
```

**NEVER Do This** (Music Analyzer mistake):
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

### Premature Stopping Anti-Pattern [v0.8.0]

**CRITICAL**: Do NOT stop at integration test failures and report status. FIX THEM.

**What Music Analyzer Failure #3 Taught Us:**

The orchestrator stopped at Phase 5 with 0/4 integration tests passing and reported:
> "Project 95% complete with only integration fixes remaining before full functionality."

**This violated continuous execution.** The orchestrator should have:
1. Analyzed the failures (API mismatches)
2. Launched agents to fix the mismatches
3. Re-run tests until 100% pass rate
4. Proceeded to Phase 6
5. Generated final documentation
6. THEN reported 100% completion

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

**Integration Test Interpretation Rules - ZERO TOLERANCE:**

| Error Pattern | Meaning | Severity | Action |
|--------------|---------|----------|---------|
| `AttributeError: 'X' object has no attribute 'Y'` | API mismatch | **CRITICAL** | Fix component API NOW |
| `TypeError: X() takes Y arguments but Z were given` | Signature mismatch | **CRITICAL** | Fix method signature NOW |
| `ImportError: cannot import name 'X'` | Missing implementation | **CRITICAL** | Implement missing class NOW |
| `KeyError` in integration test | Contract violation | **CRITICAL** | Fix data structure NOW |

**ABSOLUTE GATE**: Do not proceed to PHASE 6 until 100% integration pass rate achieved

**Phase 5 Completion Gate [v0.8.0]:**

Before proceeding to Phase 6, verify:
- ✅ Integration tests: 100% execution rate
- ✅ Integration tests: 100% pass rate
- ✅ Integration tests: 0 "NOT RUN" status
- ✅ No API mismatches remain
- ✅ No blocking errors remain

**If ANY item is ✗, DO NOT proceed to Phase 6. FIX IT FIRST.**

**Do NOT**:
- ❌ Create status report with "X% complete"
- ❌ Stop and wait for user approval
- ❌ Ask "should I continue?"

**Correct behavior**: Continue fixing until ALL items are ✅, then proceed to Phase 6 automatically.

## PHASE 6: COMPLETION VERIFICATION - ZERO TOLERANCE (Execute immediately)

### Step 1: Run Automated Verification

**Use completion verifier for each component:**
```bash
for component in components/*/; do
  python orchestration/completion_verifier.py "$component"
done
```

**All 11 checks must pass (v0.7.0):**
□ Tests pass (100%)
□ Imports resolve
□ No stubs
□ No TODOs
□ Documentation complete
□ No remaining work markers
□ Test coverage ≥ 80%
□ Manifest complete
□ Test quality verified
□ **User acceptance verified** [v0.6.0]
□ **Integration test execution (100%)** ← NEW CHECK [v0.7.0]

### Step 2: Project Type Detection

**Identify project type from primary application component:**

```bash
# Read component.yaml of main application component
cat components/<main-app>/component.yaml | grep "type:"
```

**Project types:**
- `cli` or `application`: Command-line interface
- `library` or `package`: Importable library
- `api`, `web`, `server`: Web server/API
- `gui`, `desktop`: GUI application

---

### Project-Type-Specific UAT Patterns [v0.10.0]

**Quick Reference: What to test for each project type**

#### CLI Application Pattern
- ✅ Generate test input files (audio, images, CSV)
- ✅ Run primary command with test data
- ✅ Verify output files created
- ✅ Verify output contains expected data
- ✅ Test --help shows usage
- ✅ Test error handling (invalid input)

#### REST API Pattern
- ✅ Start service in background
- ✅ Verify health endpoint responds
- ✅ Test primary endpoints with sample data
- ✅ Verify response format matches API contract
- ✅ Test error responses (400, 401, 404, 500)
- ✅ Stop service cleanly

#### Web Application Pattern
- ✅ Start web server
- ✅ Verify homepage loads without errors
- ✅ Test primary user flow (login, action, logout)
- ✅ Verify JavaScript loads without console errors
- ✅ Stop server cleanly

#### Library/SDK Pattern
- ✅ Import library successfully
- ✅ Run primary use case with real data
- ✅ Verify output matches expected format
- ✅ Test error handling (invalid inputs)
- ✅ Verify documentation examples work

#### Data Pipeline Pattern
- ✅ Generate test input data (CSV, JSON, etc.)
- ✅ Run pipeline with test data
- ✅ Verify output data created
- ✅ Verify transformations applied correctly
- ✅ Test error handling (malformed data)

**Choose the pattern matching your project type and execute ALL items in the detailed sections below.**

---

### Step 3: Type-Specific User Acceptance Verification

#### For CLI Applications [v0.10.0 - EXECUTABLE UAT]

**CRITICAL CHANGE**: UAT is now executable, not checkbox-based. You must actually run the primary user workflow and paste the output.

##### Step 1: Use Test Data Generator (Created in Phase 5)

The Integration Test Agent created a test data generator in Phase 5. Use it to create test data for smoke testing.

```bash
# Example for music-analyzer or audio-processor:
python tests/utilities/generate_test_audio.py

# Example for image-processor:
python tests/utilities/generate_test_images.py

# Example for data-pipeline:
python tests/utilities/generate_test_csv.py
```

**Verify**:
- ✅ Generator runs without errors
- ✅ Test data created successfully
- ✅ Data is in expected format

##### Step 2: Execute Primary User Workflow

**MANDATORY**: Actually run the primary user workflow with the generated test data.

```bash
echo "===== SMOKE TEST: Primary CLI Command ====="

# Run the actual CLI command (replace with your command)
<your-cli-command> <action> test_data/ --output smoke_test_results.xlsx

# Check exit code
if [ $? -ne 0 ]; then
    echo "❌ SMOKE TEST FAILED - Command crashed"
    echo "Cannot declare project complete"
    exit 1
fi

# Verify output
echo "===== Verifying Output ====="
python -c "
# Verify output file exists and contains data
import pandas as pd
df = pd.read_excel('smoke_test_results.xlsx')
assert len(df) > 0, 'Results file is empty'
assert 'filename' in df.columns, 'Missing expected column'  # Adjust for your schema
print(f'✅ Smoke test passed: {len(df)} records in results')
"

echo "✅ SMOKE TEST PASSED"
```

##### Step 3: Document Execution Results

**REQUIRED**: Copy-paste the ACTUAL terminal output below.

**Smoke Test Output**:
```
[PASTE ACTUAL TERMINAL OUTPUT HERE - REQUIRED FOR VERIFICATION]

Example:
===== SMOKE TEST: Primary CLI Command =====
Processing test_audio/sample_440hz.wav...
Analysis complete: 1 file processed
Results saved to: smoke_test_results.xlsx
===== Verifying Output =====
✅ Smoke test passed: 1 records in results
✅ SMOKE TEST PASSED
```

**DO NOT write**: "Smoke test passed" without actual output.
**DO write**: Copy-paste the actual terminal output showing execution.

##### Step 4: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- ✅ Test data generator ran successfully
- ✅ Test data created in expected location
- ✅ Primary user workflow executed without exceptions
- ✅ Output files/responses created as expected
- ✅ Output contains valid data (not empty)
- ✅ **Actual terminal output pasted above** (REQUIRED)

**If ANY item is false**:
- ❌ DO NOT declare project complete
- ❌ DO NOT proceed to final documentation
- ✅ Fix the issues and re-run smoke test
- ✅ Only proceed after smoke test passes

##### Why This Matters

**Brain Music Analyzer Lesson**: System was declared "complete" with:
- ✅ All tests passing (100%)
- ✅ UAT checklist marked complete
- ❌ **But nobody actually ran it**

Result: Crashed immediately on first user attempt with 100% failure rate.

**The Fix**: Actually execute the primary workflow before declaring complete. If it crashes, you find out BEFORE the user does.

#### For Libraries/Packages [v0.10.0 - EXECUTABLE UAT]

**CRITICAL CHANGE**: UAT is now executable, not checkbox-based.

##### Step 1: Use Test Data Generator (If Applicable)

If the library processes data files or requires test data, use the test data generator created in Phase 5:

```bash
# Example for data processing library:
python tests/utilities/generate_test_data.py
```

**Skip this step if library is pure logic (no data files needed).**

##### Step 2: Execute Library Smoke Tests

**MANDATORY**: Actually execute the library's primary API with real usage.

```bash
echo "===== SMOKE TEST: Library Import and Usage ====="

# Test 1: Import works as documented
python -c "
from <library_name> import <ClassName>
obj = <ClassName>()
print('✅ Import successful: <ClassName>')
"

# Test 2: Execute README example
echo "===== Testing README Example ====="
python -c "
# PASTE EXACT CODE FROM README.md USAGE SECTION HERE
# Example:
# from my_library import DataProcessor
# processor = DataProcessor()
# result = processor.process_data('test_data/sample.csv')
# assert result is not None, 'Processing failed'
# print(f'✅ README example passed: {result}')
"

# Test 3: Verify packaging
echo "===== Verifying Package Installation ====="
if [ -f setup.py ] || [ -f pyproject.toml ]; then
    echo "✅ Package configuration found"
else
    echo "❌ No setup.py or pyproject.toml found"
    exit 1
fi

# Test 4: Install in clean venv and test import
python -m venv /tmp/uat_test_venv
/tmp/uat_test_venv/bin/pip install -e . > /dev/null 2>&1
/tmp/uat_test_venv/bin/python -c "import <library_name>; print('✅ Installation test passed')"
rm -rf /tmp/uat_test_venv

echo "✅ LIBRARY SMOKE TEST PASSED"
```

##### Step 3: Document Execution Results

**REQUIRED**: Copy-paste the ACTUAL terminal output below.

**Smoke Test Output**:
```
[PASTE ACTUAL TERMINAL OUTPUT HERE - REQUIRED FOR VERIFICATION]

Example:
===== SMOKE TEST: Library Import and Usage =====
✅ Import successful: DataProcessor
===== Testing README Example =====
✅ README example passed: {'processed': 100, 'valid': 95}
===== Verifying Package Installation =====
✅ Package configuration found
✅ Installation test passed
✅ LIBRARY SMOKE TEST PASSED
```

**DO NOT write**: "Library works" without actual output.
**DO write**: Copy-paste the actual terminal output showing execution.

##### Step 4: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- ✅ Library imports without ImportError
- ✅ README example code executes successfully
- ✅ Package configuration (setup.py/pyproject.toml) exists
- ✅ Clean venv installation works
- ✅ **Actual terminal output pasted above** (REQUIRED)

**If ANY item is false**:
- ❌ DO NOT declare project complete
- ✅ Fix the issues and re-run smoke test
- ✅ Only proceed after smoke test passes

#### For Web Servers/APIs [v0.10.0 - EXECUTABLE UAT]

**CRITICAL CHANGE**: UAT is now executable, not checkbox-based.

##### Step 1: Use Test Data Generator (Created in Phase 5)

The Integration Test Agent created a test data generator in Phase 5. Use it to create test payloads for smoke testing.

```bash
# Example for REST API:
python tests/utilities/generate_test_data.py

# Example for data processing API:
python tests/utilities/generate_test_payloads.py
```

**Verify**:
- ✅ Generator runs without errors
- ✅ Test payloads/data created successfully
- ✅ Data is in expected format (JSON, XML, etc.)

##### Step 2: Execute API Smoke Tests

**MANDATORY**: Actually start the server and test the primary endpoints.

```bash
echo "===== SMOKE TEST: API Server ====="

# Test 1: Server starts
echo "Starting server..."
python -m <server_module> > /tmp/server.log 2>&1 &
SERVER_PID=$!
sleep 3

if ps -p $SERVER_PID > /dev/null; then
    echo "✅ Server started (PID: $SERVER_PID)"
else
    echo "❌ Server crashed on startup"
    cat /tmp/server.log
    exit 1
fi

# Test 2: Health endpoint responds
echo "===== Testing Health Endpoint ====="
HEALTH_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/health)
if [ "$HEALTH_RESPONSE" = "200" ]; then
    echo "✅ Health endpoint: HTTP 200"
else
    echo "❌ Health endpoint: HTTP $HEALTH_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

# Test 3: Primary API endpoint with test data
echo "===== Testing Primary API Endpoint ====="
API_RESPONSE=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d @test_data/sample_payload.json \
    http://localhost:8000/api/<primary-endpoint>)

if echo "$API_RESPONSE" | jq . > /dev/null 2>&1; then
    echo "✅ API responded with valid JSON"
    echo "Response: $API_RESPONSE"
else
    echo "❌ API responded with invalid JSON or error"
    echo "Response: $API_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

# Test 4: Docker (if applicable)
if [ -f Dockerfile ]; then
    echo "===== Testing Docker Build and Run ====="
    docker build -t smoke-test-api . > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ Docker build succeeded"

        # Stop existing server
        kill $SERVER_PID
        sleep 2

        # Run Docker container
        docker run -d -p 8000:8000 --name smoke-test-container smoke-test-api
        sleep 5

        DOCKER_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/health)
        if [ "$DOCKER_HEALTH" = "200" ]; then
            echo "✅ Docker container health check: HTTP 200"
        else
            echo "❌ Docker container health check failed: HTTP $DOCKER_HEALTH"
            docker logs smoke-test-container
            docker stop smoke-test-container > /dev/null
            docker rm smoke-test-container > /dev/null
            exit 1
        fi

        # Cleanup
        docker stop smoke-test-container > /dev/null
        docker rm smoke-test-container > /dev/null
    else
        echo "❌ Docker build failed"
        kill $SERVER_PID
        exit 1
    fi
else
    echo "ℹ️  No Dockerfile found, skipping Docker test"
    kill $SERVER_PID
fi

echo "✅ API SMOKE TEST PASSED"
```

##### Step 3: Document Execution Results

**REQUIRED**: Copy-paste the ACTUAL terminal output below.

**Smoke Test Output**:
```
[PASTE ACTUAL TERMINAL OUTPUT HERE - REQUIRED FOR VERIFICATION]

Example:
===== SMOKE TEST: API Server =====
Starting server...
✅ Server started (PID: 12345)
===== Testing Health Endpoint =====
✅ Health endpoint: HTTP 200
===== Testing Primary API Endpoint =====
✅ API responded with valid JSON
Response: {"status": "success", "data": {"processed": 42}}
===== Testing Docker Build and Run =====
✅ Docker build succeeded
✅ Docker container health check: HTTP 200
✅ API SMOKE TEST PASSED
```

**DO NOT write**: "API works" without actual output.
**DO write**: Copy-paste the actual terminal output showing execution.

##### Step 4: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- ✅ Server starts without crashes
- ✅ Health endpoint returns HTTP 200
- ✅ Primary API endpoint responds with valid data
- ✅ Docker build/run successful (if Dockerfile exists)
- ✅ **Actual terminal output pasted above** (REQUIRED)

**If ANY item is false**:
- ❌ DO NOT declare project complete
- ❌ DO NOT proceed to final documentation
- ✅ Fix the issues and re-run smoke test
- ✅ Only proceed after smoke test passes

##### Why This Matters

**Integration Failure Pattern**: APIs can pass all tests but fail on:
- Server startup crashes under real conditions
- Endpoint routing misconfiguration (404s)
- JSON serialization errors with real data
- Docker configuration issues

**The Fix**: Actually start the server and make real requests before declaring complete.

#### For GUI Applications [v0.10.0 - EXECUTABLE UAT]

**CRITICAL CHANGE**: UAT is now executable with automated and manual components.

##### Step 1: Use Test Data Generator (If Applicable)

If the GUI processes data files, use the test data generator created in Phase 5:

```bash
# Example for image editor:
python tests/utilities/generate_test_images.py

# Example for data visualization:
python tests/utilities/generate_test_data.py
```

**Skip this step if GUI is purely interactive (no data files needed).**

##### Step 2: Execute Automated GUI Smoke Tests

**MANDATORY**: Actually launch the GUI and verify it doesn't crash.

```bash
echo "===== SMOKE TEST: GUI Application ====="

# Test 1: GUI launches without immediate crash
echo "Launching GUI..."
timeout 5s python -m <gui_module> > /tmp/gui.log 2>&1 &
GUI_PID=$!
sleep 2

if ps -p $GUI_PID > /dev/null 2>&1; then
    echo "✅ GUI launched without crash (PID: $GUI_PID)"
    kill $GUI_PID 2>/dev/null
elif [ $? -eq 124 ]; then
    echo "✅ GUI launched and running (timeout reached)"
else
    echo "❌ GUI crashed on startup"
    cat /tmp/gui.log
    exit 1
fi

# Test 2: Check for critical startup errors in logs
if grep -i "error\|exception\|traceback" /tmp/gui.log > /dev/null; then
    echo "⚠️  Warning: Found errors in GUI startup logs:"
    grep -i "error\|exception\|traceback" /tmp/gui.log
    echo "Manual verification required."
else
    echo "✅ No critical errors in startup logs"
fi

echo "===== Automated Checks Complete ====="
```

##### Step 3: Execute Manual GUI Smoke Test

**MANDATORY**: Manually launch and test the primary workflow.

**Manual Test Procedure**:
1. Launch application: `python -m <gui_module>`
2. Verify main window appears with all UI elements
3. Execute primary user workflow (e.g., load data → process → view results)
4. Verify no crashes or error dialogs
5. Close application cleanly (no forced kills)

**Document Results**:
```
[DOCUMENT MANUAL TEST RESULTS HERE - REQUIRED]

Example:
Manual GUI Test Results:
1. ✅ Main window appeared after 2 seconds
2. ✅ Loaded test_data/sample.csv successfully
3. ✅ Processed data without errors
4. ✅ Results displayed in chart view
5. ✅ Application closed cleanly via File → Exit

Screenshots (if applicable):
- [Attach or reference screenshots if needed]

Manual Tester: [Your Name]
Date: [Test Date]
```

##### Step 4: Document Execution Results

**REQUIRED**: Copy-paste the ACTUAL terminal output from automated tests.

**Automated Smoke Test Output**:
```
[PASTE ACTUAL TERMINAL OUTPUT HERE - REQUIRED FOR VERIFICATION]

Example:
===== SMOKE TEST: GUI Application =====
Launching GUI...
✅ GUI launched without crash (PID: 12345)
✅ No critical errors in startup logs
===== Automated Checks Complete =====
```

##### Step 5: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- ✅ GUI launches without immediate crash
- ✅ No critical errors in startup logs
- ✅ Main window appears and renders correctly (manual verification)
- ✅ Primary workflow executes without crashes (manual verification)
- ✅ Application closes cleanly (manual verification)
- ✅ **Actual terminal output pasted above** (REQUIRED)
- ✅ **Manual test results documented above** (REQUIRED)

**If ANY item is false**:
- ❌ DO NOT declare project complete
- ✅ Fix the issues and re-run smoke test
- ✅ Only proceed after smoke test passes

##### Why This Matters

**GUI-Specific Risks**:
- GUI frameworks can crash on startup due to missing dependencies
- UI components may fail to render (blank windows, missing buttons)
- Event handlers may have uncaught exceptions
- File dialogs may crash when accessing real filesystem

**The Fix**: Actually launch the GUI and use it manually before declaring complete. Automated tests can't catch all UI issues.

**Note**: GUI testing is inherently more manual than CLI/API testing. Both automated checks and manual verification are required.

### Step 4: Verify Test Pass Rates

**ABSOLUTE REQUIREMENTS (ALL must be 100%):**

□ Unit tests: 100% pass rate (NO EXCEPTIONS)
□ Integration tests: 100% pass rate (NO EXCEPTIONS)
□ Integration tests: 100% execution rate (NO EXCEPTIONS) [v0.7.0]
□ Contract tests: 100% pass rate (NO EXCEPTIONS)
□ E2E tests: 100% pass rate (NO EXCEPTIONS)
□ Zero AttributeError in any test
□ Zero TypeError in any test
□ Zero ImportError in any test
□ Zero tests in "NOT RUN" status [v0.7.0]

**Why 100% for ALL Tests:**
- Music Analyzer lesson: "Don't dismiss test failures" applies to ALL tests
- Broken unit tests = broken internal logic = unstable system
- Integration tests depend on unit-tested components
- Zero tolerance means ZERO exceptions (not 90%, not 99%, not 99.9%)

**Handling Edge Cases:**
- Flaky tests → Fix them (make deterministic) or explicitly skip with tracking
- Deprecated code → Delete code AND tests together (don't deliver with failures)
- External dependencies → Mock them (make tests deterministic)
- If test fails → Either fix the bug, skip with reason, or delete the test
- NEVER lower the pass rate threshold to accommodate failures

**Integration Test Pass Rate Requirement:**
- ❌ 99% pass = SYSTEM BROKEN - Return to Phase 5
- ❌ 95% pass = SYSTEM BROKEN - Return to Phase 5
- ❌ 79.5% pass = CATASTROPHIC FAILURE (Music Analyzer #1) - Return to Phase 5
- ✅ 100% pass = Minimum requirement to proceed

**Integration Test Execution Rate Requirement [v0.7.0]:**
- ❌ 99% execution = SYSTEM BROKEN - Return to Phase 5
- ❌ 95% execution = SYSTEM BROKEN - Return to Phase 5
- ❌ 67% execution = CATASTROPHIC FAILURE - Return to Phase 5
- ❌ 37.5% execution = CATASTROPHIC FAILURE (Music Analyzer #2) - Return to Phase 5
- ❌ ANY "NOT RUN" status = BLOCKING - Return to Phase 5
- ✅ 100% execution = Minimum requirement to proceed

**Verify Integration Execution Coverage:**
```bash
# MANDATORY: Run integration coverage checker
python orchestration/integration_coverage_checker.py --strict

# Must show:
# ✅ Execution Rate: 100.0%
# ✅ Tests NOT RUN: 0
# ✅ Pass Rate: 100.0%
# ✅ SYSTEM READY FOR COMPLETION

# If NOT READY:
# - Fix ALL issues
# - Re-run ENTIRE integration suite
# - Re-run this checker
# - Only proceed when READY
```

**If ANY test fails:**
- STOP - Do not proceed
- Fix the failures
- Re-run verification
- Only proceed with 100% pass rate

### Step 5: Final Acceptance Gate

**Before declaring complete, verify:**

□ completion_verifier.py passes for ALL components (11/11 checks) [v0.7.0]
□ Project-type-specific UAT passed (all checks above)
□ All tests pass: 100% (unit, integration, contract, E2E)
□ **All integration tests executed: 100% (no "NOT RUN")** [v0.7.0]
□ Test coverage ≥ 80% for all components
□ README examples verified (actually work when copy-pasted)
□ Documented commands work exactly as written
□ Users can actually run/install/use the product
□ **Integration coverage checker shows READY** [v0.7.0]

**Integration Execution Verification [v0.7.0]:**
```bash
# Final check before declaring complete
python orchestration/integration_coverage_checker.py --strict

# Expected output:
# ╔══════════════════════════════════════════════════════════╗
# ║      Integration Test Execution Coverage Report         ║
# ╚══════════════════════════════════════════════════════════╝
#
# Total Tests Planned:  X
# Tests Executed:       X
# Tests Passed:         X
# Tests Failed:         0
# Tests NOT RUN:        0
#
# Execution Rate:       100.0%
# Pass Rate:            100.0%
# Overall Rate:         100.0%
#
# ✅ SYSTEM READY FOR COMPLETION
#    - 100% execution rate (all tests ran)
#    - 100% pass rate (all tests passed)
#    - Zero tests in NOT RUN status

# Exit code: 0
```

**If all checks pass:**
✅ Project is COMPLETE and ready for delivery

**If any check fails:**
❌ Project is INCOMPLETE - return to previous phase

**Common Failure Scenarios [v0.7.0]:**
- completion_verifier shows "Integration Test Execution: FAIL" → Re-run integration suite
- integration_coverage_checker shows "NOT READY" → Fix and re-run all tests
- Tests show "NOT RUN" status → Fix blocking failures, re-run ENTIRE suite
- Execution rate < 100% → Find cause, fix, re-run ENTIRE suite

### Step 6: Generate Completion Report

**After ALL Checks Pass:**
- Generate project documentation
- Create deployment guides
- Provide final status report with current version (e.g., "v0.2.0")
- Create quality assessment report (NOT production declaration)
- DO NOT change version to 1.0.0
- DO NOT declare "production ready"

**Completion Report Template:**
```markdown
# Project Completion Report

## Verification Results
- ✅ All components verified (11/11 checks) [v0.7.0]
- ✅ UAT passed (CLI/library/web/GUI)
- ✅ All tests passing (100% pass rate)
- ✅ All integration tests executed (100% execution rate) [v0.7.0]
- ✅ Test coverage: XX%
- ✅ User acceptance verified
- ✅ Integration execution verified (0 NOT RUN) [v0.7.0]

## Deliverables
- Source code: <location>
- Documentation: <location>
- Installation guide: <location>
- User guide: <location>

## Project Type: <CLI/Library/Web/GUI>
## Version: <current version>
## Status: Complete (pre-release)
```

**Phase 6 Completion Gate [v0.8.0]:**

Before generating completion report, verify:
- ✅ All 11 completion checks passing (completion_verifier.py)
- ✅ Project-type-specific UAT passed
- ✅ All tests passing: 100% (unit, integration, contract, E2E)
- ✅ All integration tests executed: 100% (no "NOT RUN")
- ✅ Test coverage ≥ 80% for all components
- ✅ README examples verified (work when copy-pasted)
- ✅ Documented commands work exactly as written
- ✅ Users can actually run/install/use the product
- ✅ Integration coverage checker shows READY

**If ANY item is ✗, DO NOT generate completion report. FIX IT FIRST.**

**Do NOT**:
- ❌ Generate partial completion report with "remaining work" section
- ❌ Report "X% complete"
- ❌ Ask "ready to finalize?"

**Correct behavior**: Continue fixing until ALL items are ✅, then generate completion report showing 100% complete.

## EXECUTION RULES (CRITICAL - READ CAREFULLY):

### Continuous Execution - NO PREMATURE STOPS

**THE IRON LAW**: Execute continuously from start to finish. Do NOT stop until 100% complete.

**ONLY Stop When**:
1. ALL phases complete (Phase 1-6, 100% passing tests, docs generated)
2. Specification contains unresolvable contradiction
3. User explicitly requests pause ("stop and wait for instructions")
4. External system permanently unavailable AND blocking

**NEVER Stop When**:
❌ Integration tests failing (fix them)
❌ API mismatches found (fix them)
❌ 95% complete (finish the remaining 5%)
❌ "User might want to know" (they want 100% complete)
❌ "Should I continue?" (YES, always continue)
❌ Tempted to create "status report" before 100% (DON'T)

### The 95% Completion Anti-Pattern (AVOID)

**Music Analyzer Failure #3 Scenario**:
```
Orchestrator reached Phase 5, integration tests failing (0/4 passing)
→ Created status report: "95% complete, integration fixes remaining"
→ STOPPED and waited for user input
→ WRONG - Should have fixed failures and continued to 100%
```

**What Should Have Happened**:
```
Orchestrator reached Phase 5, integration tests failing (0/4 passing)
→ Analyzed failures: API mismatches (RhythmAnalyzer, BenefitScorer)
→ Launched agents to fix API mismatches
→ Re-ran integration tests (100% passing)
→ Proceeded to Phase 6 (completion verification)
→ Generated final documentation
→ THEN reported: "Project 100% complete and deployment-ready"
```

### Fix vs Ask Decision Framework

**Fix Autonomously** (99% of cases):
- API signature mismatches
- Type errors (str vs int, dict vs object)
- Missing optional parameters
- Component communication errors
- Integration test failures
- Import errors
- Configuration issues
- Performance optimizations

**Ask User** (1% of cases):
- Specification contradicts itself fundamentally
- Business logic has multiple valid interpretations
- Security/privacy policy decisions
- User preference required (UI colors, workflow order)
- External API credentials needed

**Rule of Thumb**: If the error message tells you what's wrong, FIX IT. Don't ask.

### Examples of Autonomous Fixing

**Scenario 1: API Mismatch**
```
ERROR: RhythmAnalyzer.analyze() missing required argument 'config'
ERROR: BenefitScorer expected dict, got RhythmAnalysisResult object

❌ WRONG Response:
"Integration tests failing due to API mismatches. Should I fix these or wait for your review?"

✅ CORRECT Response:
"Integration tests failing - API mismatches detected. Fixing:
1. Adding optional config parameter to RhythmAnalyzer.analyze()
2. Updating BenefitScorer to accept dict or convert from object
3. Re-running integration suite..."
[Continues working until 100% pass rate achieved]
```

**Scenario 2: Import Error**
```
ERROR: ImportError: cannot import name 'AudioProcessor'

❌ WRONG Response:
"Component import failing. Please advise on module structure."

✅ CORRECT Response:
"Import error detected - fixing module exports and re-testing..."
[Fixes import, continues working]
```

**Scenario 3: 95% Complete**
```
Status: Phases 1-4 complete, Phase 5 has 2 failing integration tests

❌ WRONG Response:
"Project 95% complete. Integration fixes remaining. Ready for your review."
[STOPS]

✅ CORRECT Response:
"Phase 5 integration tests failing - analyzing and fixing..."
[Fixes tests, completes Phase 6, generates docs, THEN reports 100%]
```

### Trust Your Judgment

From CLAUDE.md:
> "Make ALL architectural and implementation decisions autonomously based on specifications"
> "Trust your architectural judgment"
> "Break and improve code freely (0.x.x pre-release policy)"

**This means**:
- You CAN fix API mismatches
- You CAN modify component interfaces
- You CAN refactor to make tests pass
- You CAN make implementation decisions
- You DON'T need permission to fix technical issues

**API mismatches are implementation details, not specification ambiguities.**

### Additional Execution Guidelines

- Make ALL architectural and implementation decisions autonomously based on specifications
- Do NOT ask for plan approval - create plan and execute immediately
- Do NOT ask for implementation approval - implement based on specs
- Use Task tool for parallel component agent execution
- ONLY ask questions if specifications are genuinely ambiguous or contradictory
- Break and improve code freely (0.x.x pre-release policy)
- Prefer clean code over backwards compatibility
- Trust your architectural judgment

BEGIN EXECUTION NOW with Phase 1.
