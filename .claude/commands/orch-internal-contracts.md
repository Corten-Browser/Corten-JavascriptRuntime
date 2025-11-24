---
description: "Internal: Phase 4.5 contract validation (called by /orchestrate)"
---

# Internal: Phase 4.5 Contract Validation

**This is an internal command called by `/orchestrate` via subagent.**
**Do not invoke directly unless you understand the orchestration workflow.**

## Purpose

Validate that each component implements its contract EXACTLY before integration testing.
This is a PRE-INTEGRATION GATE that catches API mismatches early.

## Prerequisites

This command expects:
- Phase 4 (Parallel Development) completed
- Components exist in `components/` directory
- Contract tests exist in `components/*/tests/contracts/`
- Contracts defined in `contracts/` directory

## Phase Skip Check (Resume Support)

```python
# Note: Phase 4.5 is critical validation - generally should not be skipped
# But if resuming from Phase 5+, we can skip
if 'resuming' in globals() and resuming and current_phase > 4.5:
    print("⏭️  Skipping Phase 4.5 (already completed)")
    # Report back and continue to Phase 5
```

---

## Why This Phase Exists

**Historical Lesson - Music Analyzer Failure:**
- Components had wrong method names
- Contract validation would have caught `scan()` vs `get_audio_files()` mismatch
- Prevents wasting time on integration tests when basic APIs are wrong
- Faster feedback loop than full integration testing

**Purpose**: Catch API mismatches BEFORE running integration tests

---

## Step 1: Run Contract Tests

**For each component, run contract tests:**

```bash
echo "===== Running Contract Tests ====="

for component_dir in components/*/; do
    component=$(basename "$component_dir")
    echo ""
    echo "Testing $component contracts..."

    if [ -d "${component_dir}tests/contracts" ]; then
        cd "$component_dir"
        pytest tests/contracts/ -v
        exit_code=$?
        cd - > /dev/null

        if [ $exit_code -eq 0 ]; then
            echo "✅ $component: Contract tests PASSED"
        else
            echo "❌ $component: Contract tests FAILED"
            # Record failure but continue checking other components
        fi
    else
        echo "⚠️  $component: No contract tests found (tests/contracts/ missing)"
    fi
done
```

**Required: 100% pass rate for ALL components.**

---

## Step 2: Contract Validation Tool

**Run automated validation across all components:**

```bash
echo "===== Running Contract Validator ====="
python orchestration/verification/contracts/contract_validator.py --all

if [ $? -ne 0 ]; then
    echo "❌ Contract validation FAILED"
    echo "Fix contract violations before proceeding to integration testing"
    exit 1
fi

echo "✅ All contracts validated"
```

---

## Step 3: Check for Common API Mismatches

**Common Mismatch Patterns to Check:**

| Mismatch Type | Example | Detection |
|---------------|---------|-----------|
| Method name | `scan()` vs `get_audio_files()` | Contract specifies one, implementation uses another |
| Parameter name | `directory` vs `path` | Callers use wrong parameter name |
| Singular/plural | `generate_playlist` vs `generate_playlists` | Inconsistent naming |
| Verb mismatch | `store` vs `save` | Different verbs for same operation |
| Return type | Returns `dict` vs `object` | Type mismatch in consumers |

**Manual Check Pattern:**
```python
# For each component, verify:
# 1. Contract file exists
# 2. All methods in contract are implemented
# 3. Method signatures match exactly
# 4. Return types match

for contract in contracts/*.yaml:
    component = extract_component_name(contract)
    implementation = f"components/{component}/src/"

    for method in contract.methods:
        # Verify method exists with exact signature
        assert method_exists(implementation, method.name)
        assert signature_matches(implementation, method)
        assert return_type_matches(implementation, method)
```

---

## Step 4: Active Contract Method Validation (Optional)

**PURPOSE**: Catch API mismatches BEFORE integration testing (defense-in-depth).

**OPTIONAL**: This check provides faster feedback but integration tests catch most issues.

**For each component**, validate that method calls match dependency contracts:

```bash
for component in cli_interface audio_loader rhythm_analyzer; do
    echo "Validating $component method calls..."
    python orchestration/validation/contract_method_validator.py --component $component

    if [ $? -ne 0 ]; then
        echo "❌ Contract violations found in $component"
        echo "Fix before proceeding to integration tests"
        exit 1
    fi
done
```

**Expected Output (Success)**:
```
Validating cli_interface method calls...
✅ Contract method validation PASSED
   All method calls match contract specifications
```

**If Violations Found**:
```
❌ Contract method validation FAILED
   Found 2 contract violations

[cli_interface] api.py:42
  Called: audio_loader.load()
  Problem: Method not in audio_loader contract
  Available methods: load_audio, get_metadata, close
  💡 Did you mean: load_audio()?

[cli_interface] api.py:67
  Called: data_manager.export_to_spreadsheet()
  Problem: Method not in data_manager contract
  Available methods: export_to_excel, save_results
  💡 Did you mean: export_to_excel()?

============================================================
RESULT: BLOCKED - Fix contract violations before proceeding
```

**Why This Helps**:
- Catches method name mismatches early
- Faster feedback than waiting for integration tests
- Suggests correct method names
- Prevents wasted time on integration when APIs are wrong

---

## Failure Response

**If ANY Contract Test Fails:**

```
❌ file_scanner: Contract tests FAILED (7/8 pass)
  - CRITICAL: test_exact_api_compliance FAILED
  - Expected method: scan()
  - Found method: get_audio_files()

🛑 STOP - Fix file_scanner API before integration testing
```

**Recovery Process:**
1. STOP immediately - do not proceed to Phase 5
2. Identify the mismatch (contract vs implementation)
3. **Fix the component implementation** (NOT the contract)
4. Re-run contract tests until 100% pass
5. Only proceed when ALL components pass contract validation

**CRITICAL**: Fix the implementation to match the contract. Do NOT change the contract to match a wrong implementation.

---

## Expected Output (Success)

```
===== Running Contract Tests =====

Testing auth_service contracts...
✅ auth_service: Contract tests 100% pass (12/12)

Testing file_scanner contracts...
✅ file_scanner: Contract tests 100% pass (8/8)

Testing playlist_generator contracts...
✅ playlist_generator: Contract tests 100% pass (10/10)

Testing database_manager contracts...
✅ database_manager: Contract tests 100% pass (6/6)

===== Running Contract Validator =====
✅ All contracts validated

✅ ALL COMPONENTS PASS CONTRACT VALIDATION - Proceeding to integration testing
```

---

## Checkpoint Integration

**Save Phase 4.5 completion:**
```python
orchestrator.complete_phase(
    phase_number=4.5,
    outputs={
        "phase_name": "Contract Validation",
        "contract_tests_passed": True,
        "api_compliance_verified": True
    }
)
```

---

## Reporting Back to Orchestrator

**After completing all steps, report back with this structure:**

**Success case (proceed to Phase 5):**
```json
{
    "phase": 4.5,
    "phase_name": "Contract Validation",
    "validation_passed": true,
    "results_by_component": {
        "auth_service": {"status": "pass", "tests": "12/12"},
        "file_scanner": {"status": "pass", "tests": "8/8"},
        "playlist_generator": {"status": "pass", "tests": "10/10"},
        "database_manager": {"status": "pass", "tests": "6/6"}
    },
    "total_components": 4,
    "components_passed": 4,
    "components_failed": 0,
    "violations": [],
    "can_proceed": true
}
```

**Failure case (fix required):**
```json
{
    "phase": 4.5,
    "phase_name": "Contract Validation",
    "validation_passed": false,
    "results_by_component": {
        "auth_service": {"status": "pass", "tests": "12/12"},
        "file_scanner": {"status": "fail", "tests": "7/8"},
        "playlist_generator": {"status": "pass", "tests": "10/10"},
        "database_manager": {"status": "pass", "tests": "6/6"}
    },
    "total_components": 4,
    "components_passed": 3,
    "components_failed": 1,
    "violations": [
        {
            "component": "file_scanner",
            "test": "test_exact_api_compliance",
            "expected": "scan(directory: str) -> list[Path]",
            "found": "get_audio_files(path: str) -> list[str]",
            "fix_required": "Rename method to 'scan', change parameter to 'directory', return list[Path]"
        }
    ],
    "can_proceed": false,
    "next_action": "Fix file_scanner to implement scan() method per contract"
}
```

**No contract tests found:**
```json
{
    "phase": 4.5,
    "phase_name": "Contract Validation",
    "validation_passed": false,
    "results_by_component": {
        "auth_service": {"status": "no_tests", "tests": "0/0"},
        "file_scanner": {"status": "no_tests", "tests": "0/0"}
    },
    "total_components": 2,
    "components_passed": 0,
    "components_failed": 0,
    "components_no_tests": 2,
    "violations": [],
    "can_proceed": false,
    "next_action": "Create contract tests in components/*/tests/contracts/ for each component"
}
```

**CRITICAL**:
- Do NOT proceed to Phase 5 if any contract validation fails
- Always fix implementation to match contract (not vice versa)
- Include specific fix instructions in violations list
