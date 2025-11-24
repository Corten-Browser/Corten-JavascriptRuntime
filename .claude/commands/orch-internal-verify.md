---
description: "Internal: Phase 6 completion verification (called by /orchestrate)"
---

# Internal: Phase 6 Completion Verification

**This is an internal command called by `/orchestrate` via subagent.**
**Do not invoke directly unless you understand the orchestration workflow.**

## Purpose

Verify project completion with zero tolerance before declaring a project complete.
This command implements Phase 6 of the Level 3 orchestration workflow.

## Prerequisites

This command expects:
- All previous phases (1-5) completed
- Components exist in `components/` directory
- Integration tests have passed (100% execution, 100% pass rate)
- `orchestration/` directory with verification tools

## Phase Skip Check (Resume Support)

```python
# If resuming and current_phase > 6, skip this phase
# (Though if current_phase > 6, project is complete)
if 'resuming' in globals() and resuming and current_phase > 6:
    print("⏭️  Skipping Phase 6 (already completed)")
    print("✅ Project complete!")
    # Report back immediately
```

---

## Step 1: Run Automated Verification

**Use completion verifier for each component:**
```bash
for component in components/*/; do
  python orchestration/verification/completion/completion_verifier.py "$component"
done
```

**All 11 checks must pass:**
- [ ] Tests pass (100%)
- [ ] Imports resolve
- [ ] No stubs
- [ ] No TODOs
- [ ] Documentation complete
- [ ] No remaining work markers
- [ ] Test coverage ≥ 80%
- [ ] Manifest complete
- [ ] Test quality verified
- [ ] **User acceptance verified**
- [ ] **Integration test execution (100%)**

---

## Step 2: Project Type Detection

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

## Step 3: Type-Specific User Acceptance Testing

### Project-Type-Specific UAT Patterns

**Quick Reference: What to test for each project type**

#### CLI Application Pattern
- Generate test input files (audio, images, CSV)
- Run primary command with test data
- Verify output files created
- Verify output contains expected data
- Test --help shows usage
- Test error handling (invalid input)

#### REST API Pattern
- Start service in background
- Verify health endpoint responds
- Test primary endpoints with sample data
- Verify response format matches API contract
- Test error responses (400, 401, 404, 500)
- Stop service cleanly

#### Web Application Pattern
- Start web server
- Verify homepage loads without errors
- Test primary user flow (login, action, logout)
- Verify JavaScript loads without console errors
- Stop server cleanly

#### Library/SDK Pattern
- Import library successfully
- Run primary use case with real data
- Verify output matches expected format
- Test error handling (invalid inputs)
- Verify documentation examples work

#### Data Pipeline Pattern
- Generate test input data (CSV, JSON, etc.)
- Run pipeline with test data
- Verify output data created
- Verify transformations applied correctly
- Test error handling (malformed data)

**Choose the pattern matching your project type and execute ALL items in the detailed sections below.**

---

### For CLI Applications - EXECUTABLE UAT

**CRITICAL**: UAT is executable, not checkbox-based. You must actually run the primary user workflow and paste the output.

#### Step 1: Use Test Data Generator (Created in Phase 5)

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
- Generator runs without errors
- Test data created successfully
- Data is in expected format

#### Step 2: Execute Primary User Workflow

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

#### Step 3: Document Execution Results

**REQUIRED**: Copy-paste the ACTUAL terminal output.

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

#### Step 4: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- Test data generator ran successfully
- Test data created in expected location
- Primary user workflow executed without exceptions
- Output files/responses created as expected
- Output contains valid data (not empty)
- **Actual terminal output pasted above** (REQUIRED)

**If ANY item is false**:
- DO NOT declare project complete
- DO NOT proceed to final documentation
- Fix the issues and re-run smoke test
- Only proceed after smoke test passes

---

### For Libraries/Packages - EXECUTABLE UAT

#### Step 1: Use Test Data Generator (If Applicable)

If the library processes data files or requires test data, use the test data generator created in Phase 5:

```bash
# Example for data processing library:
python tests/utilities/generate_test_data.py
```

**Skip this step if library is pure logic (no data files needed).**

#### Step 2: Execute Library Smoke Tests

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

#### Step 3: Document Execution Results

**REQUIRED**: Copy-paste the ACTUAL terminal output.

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

#### Step 4: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- Library imports without ImportError
- README example code executes successfully
- Package configuration (setup.py/pyproject.toml) exists
- Clean venv installation works
- **Actual terminal output pasted above** (REQUIRED)

---

### For Web Servers/APIs - EXECUTABLE UAT

#### Step 1: Use Test Data Generator (Created in Phase 5)

```bash
# Example for REST API:
python tests/utilities/generate_test_data.py

# Example for data processing API:
python tests/utilities/generate_test_payloads.py
```

**Verify**:
- Generator runs without errors
- Test payloads/data created successfully
- Data is in expected format (JSON, XML, etc.)

#### Step 2: Execute API Smoke Tests

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

#### Step 3: Document Execution Results

**REQUIRED**: Copy-paste the ACTUAL terminal output.

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

#### Step 4: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- Server starts without crashes
- Health endpoint returns HTTP 200
- Primary API endpoint responds with valid data
- Docker build/run successful (if Dockerfile exists)
- **Actual terminal output pasted above** (REQUIRED)

---

### For GUI Applications - EXECUTABLE UAT

#### Step 1: Use Test Data Generator (If Applicable)

If the GUI processes data files, use the test data generator created in Phase 5:

```bash
# Example for image editor:
python tests/utilities/generate_test_images.py

# Example for data visualization:
python tests/utilities/generate_test_data.py
```

**Skip this step if GUI is purely interactive (no data files needed).**

#### Step 2: Execute Automated GUI Smoke Tests

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

#### Step 3: Execute Manual GUI Smoke Test

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

#### Step 4: Document Execution Results

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

#### Step 5: Verify Smoke Test Passed

**Only proceed when ALL are true**:
- GUI launches without immediate crash
- No critical errors in startup logs
- Main window appears and renders correctly (manual verification)
- Primary workflow executes without crashes (manual verification)
- Application closes cleanly (manual verification)
- **Actual terminal output pasted above** (REQUIRED)
- **Manual test results documented above** (REQUIRED)

---

## Step 4: Verify Test Pass Rates

**ABSOLUTE REQUIREMENTS (ALL must be 100%):**

- [ ] Unit tests: 100% pass rate (NO EXCEPTIONS)
- [ ] Integration tests: 100% pass rate (NO EXCEPTIONS)
- [ ] Integration tests: 100% execution rate (NO EXCEPTIONS)
- [ ] Contract tests: 100% pass rate (NO EXCEPTIONS)
- [ ] E2E tests: 100% pass rate (NO EXCEPTIONS)
- [ ] Zero AttributeError in any test
- [ ] Zero TypeError in any test
- [ ] Zero ImportError in any test
- [ ] Zero tests in "NOT RUN" status

**Why 100% for ALL Tests:**
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
- 99% pass = SYSTEM BROKEN - Return to Phase 5
- 95% pass = SYSTEM BROKEN - Return to Phase 5
- 79.5% pass = CATASTROPHIC FAILURE - Return to Phase 5
- 100% pass = Minimum requirement to proceed

**Integration Test Execution Rate Requirement:**
- 99% execution = SYSTEM BROKEN - Return to Phase 5
- ANY "NOT RUN" status = BLOCKING - Return to Phase 5
- 100% execution = Minimum requirement to proceed

**Verify Integration Execution Coverage:**
```bash
# MANDATORY: Run integration coverage checker
python orchestration/verification/quality/integration_coverage_checker.py --strict

# Must show:
# ✅ Execution Rate: 100.0%
# ✅ Tests NOT RUN: 0
# ✅ Pass Rate: 100.0%
# ✅ SYSTEM READY FOR COMPLETION
```

**If ANY test fails:**
- STOP - Do not proceed
- Fix the failures
- Re-run verification
- Only proceed with 100% pass rate

---

## Step 5: Final Acceptance Gate

**Before declaring complete, verify:**

- [ ] completion_verifier.py passes for ALL components (11/11 checks)
- [ ] Project-type-specific UAT passed (all checks above)
- [ ] All tests pass: 100% (unit, integration, contract, E2E)
- [ ] **All integration tests executed: 100% (no "NOT RUN")**
- [ ] Test coverage ≥ 80% for all components
- [ ] README examples verified (actually work when copy-pasted)
- [ ] Documented commands work exactly as written
- [ ] Users can actually run/install/use the product
- [ ] **Integration coverage checker shows READY**

**Integration Execution Verification:**
```bash
# Final check before declaring complete
python orchestration/verification/quality/integration_coverage_checker.py --strict

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
```

**If all checks pass:**
Project is COMPLETE and ready for delivery

**If any check fails:**
Project is INCOMPLETE - return to previous phase

**Common Failure Scenarios:**
- completion_verifier shows "Integration Test Execution: FAIL" → Re-run integration suite
- integration_coverage_checker shows "NOT READY" → Fix and re-run all tests
- Tests show "NOT RUN" status → Fix blocking failures, re-run ENTIRE suite
- Execution rate < 100% → Find cause, fix, re-run ENTIRE suite

---

## Step 6: Generate Completion Report

**After ALL Checks Pass:**
- Generate project documentation
- Create deployment guides
- Provide final status report with current version
- Create quality assessment report
- DO NOT change version to 1.0.0
- DO NOT declare "production ready"

**Completion Report Template:**
```markdown
# Project Completion Report

## Verification Results
- ✅ All components verified (11/11 checks)
- ✅ UAT passed (CLI/library/web/GUI)
- ✅ All tests passing (100% pass rate)
- ✅ All integration tests executed (100% execution rate)
- ✅ Test coverage: XX%
- ✅ User acceptance verified
- ✅ Integration execution verified (0 NOT RUN)

## Deliverables
- Source code: <location>
- Documentation: <location>
- Installation guide: <location>
- User guide: <location>

## Project Type: <CLI/Library/Web/GUI>
## Version: <current version>
## Status: Complete (pre-release)
```

**Phase 6 Completion Gate:**

Before generating completion report, verify:
- All 11 completion checks passing (completion_verifier.py)
- Project-type-specific UAT passed
- All tests passing: 100% (unit, integration, contract, E2E)
- All integration tests executed: 100% (no "NOT RUN")
- Test coverage ≥ 80% for all components
- README examples verified (work when copy-pasted)
- Documented commands work exactly as written
- Users can actually run/install/use the product
- Integration coverage checker shows READY

**If ANY item fails, DO NOT generate completion report. FIX IT FIRST.**

---

## Checkpoint Integration

**Save Phase 6 completion (FINAL CHECKPOINT):**
```python
orchestrator.complete_phase(
    phase_number=6,
    outputs={
        "phase_name": "Completion Verification",
        "all_checks_passed": True,
        "project_complete": True,
        "test_pass_rate": "100%",
        "test_coverage": coverage_percentage
    }
)

print("✅ All phases complete!")
print("✅ Checkpoint saved - orchestration can be safely resumed if interrupted")
```

---

## Reporting Back to Orchestrator

**After completing all steps, report back with this structure:**

```json
{
    "phase": 6,
    "phase_name": "Completion Verification",
    "verification_passed": true,
    "checks": {
        "completion_verifier": "11/11 for all components",
        "uat_type": "CLI|Library|API|GUI",
        "uat_passed": true,
        "test_pass_rate": "100%",
        "test_execution_rate": "100%",
        "test_coverage": "XX%",
        "readme_verified": true,
        "integration_coverage_ready": true
    },
    "failing_checks": [],
    "uat_output": "[ACTUAL TERMINAL OUTPUT FROM SMOKE TEST]",
    "completion_report": "[MARKDOWN CONTENT OF COMPLETION REPORT]",
    "can_proceed": true,
    "project_complete": true
}
```

**If verification failed:**
```json
{
    "phase": 6,
    "phase_name": "Completion Verification",
    "verification_passed": false,
    "checks": {
        "completion_verifier": "10/11 - missing user acceptance",
        "uat_type": "CLI",
        "uat_passed": false,
        "test_pass_rate": "98%",
        "test_execution_rate": "100%",
        "test_coverage": "75%",
        "readme_verified": false,
        "integration_coverage_ready": false
    },
    "failing_checks": [
        "UAT smoke test crashed with ImportError",
        "Test coverage below 80% threshold",
        "README example does not execute"
    ],
    "uat_output": "[ACTUAL ERROR OUTPUT]",
    "completion_report": null,
    "can_proceed": false,
    "project_complete": false,
    "recommended_action": "Return to Phase 5 to fix integration issues"
}
```

**CRITICAL**: Always include actual terminal output in `uat_output` field. Do not summarize.
