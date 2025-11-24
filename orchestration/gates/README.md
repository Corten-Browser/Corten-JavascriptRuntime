# Phase Gates

Programmatic enforcement system to prevent premature stopping.

**Version**: v0.14.0

---

## Purpose

Phase gates enforce completion criteria at phase transitions. They prevent the orchestrator from declaring completion before all requirements are met.

**Problem Solved**: Orchestrator stopping at 83.3% test pass rate and declaring "PROJECT COMPLETE" (CompletionFailureAssessment2.txt).

**Solution**: Binary pass/fail gates that block progression until criteria are met.

---

## How It Works

```
Phase 1 → [Gate 1] → Phase 2 → [Gate 2] → ... → Phase 5 → [Gate 5] → Phase 6
                ↓                                              ↓
              PASS                                    BLOCKS if <100% tests
                ↓                                              ↓
            Proceed                                    Must fix & re-run
```

### Gate Execution Flow

1. **Before transitioning** from Phase X to Phase X+1
2. **Run gate**: `python orchestration/gates/runner.py . X`
3. **Check result**:
   - Exit code 0: PASS → May proceed
   - Exit code 1: FAIL → CANNOT proceed
4. **If failed**: Fix issues, re-run gate until it passes
5. **If passed**: Proceed to next phase

---

## Available Gates

### Phase 5: Integration (CRITICAL)

**File**: `phase_5_integration.py`

**Enforces**: 100% integration test pass rate

**Blocks if**:
- Any integration test failing
- Test pass rate < 100%
- Integration tests timeout
- Integration tests cannot run

**Would have prevented**:
- Music Analyzer v1 (79.5% pass rate)
- Brain Music Analyzer v2 (83.3% pass rate)

**Usage**:
```bash
# Run Phase 5 gate
python orchestration/gates/runner.py . 5

# If fails: Fix all failing tests, then re-run
# Do not proceed to Phase 6 until this passes
```

### Phase 6: Verification

**File**: `phase_6_verification.py`

**Enforces**: All verification checks passing

**Blocks if**:
- completion_verifier not passing 13/13 checks
- Any component failing verification
- Blocking issues detected

---

## Usage

### Basic Usage

```bash
# Run gate for current phase
cd /path/to/project
python orchestration/gates/runner.py . 5

# Check orchestration state
python orchestration/gates/runner.py . --status
```

### Integration with Orchestration

**In your orchestration workflow**:

```python
# Before Phase 6 (after integration)
print("Running Phase 5 gate to validate integration...")
result = subprocess.run(
    ["python3", "orchestration/gates/runner.py", ".", "5"],
    cwd=project_root
)

if result.returncode != 0:
    print("❌ Phase 5 gate failed - cannot proceed to Phase 6")
    print("Fix all integration test failures and re-run gate")
    sys.exit(1)

print("✅ Phase 5 gate passed - proceeding to Phase 6")
```

---

## Gate Development

### Creating a New Gate

All gates follow a standard interface:

```python
#!/usr/bin/env python3
"""
Phase X Gate

Description of what this gate validates.

Exit Codes:
  0 - PASS
  1 - FAIL
"""

from pathlib import Path
import sys


class PhaseXGate:
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.failures = []

    def validate(self) -> bool:
        """Run validations."""
        self._check_requirement_1()
        self._check_requirement_2()
        return len(self.failures) == 0

    def _check_requirement_1(self):
        """Check specific requirement."""
        if not condition_met():
            self.failures.append("Requirement 1 not met")

    def report(self) -> str:
        """Generate human-readable report."""
        if not self.failures:
            return "✅ Phase X COMPLETE"
        else:
            return f"❌ Phase X INCOMPLETE: {self.failures}"


def main():
    if len(sys.argv) < 2:
        print("Usage: phase_X_gate.py <project_root>")
        sys.exit(1)

    project_root = Path(sys.argv[1]).resolve()
    gate = PhaseXGate(project_root)

    passed = gate.validate()
    print(gate.report())

    sys.exit(0 if passed else 1)


if __name__ == '__main__':
    main()
```

---

## State Tracking

Gate results are recorded in `orchestration/orchestration-state.json`:

```json
{
  "version": "0.14.0",
  "project_name": "my-project",
  "last_completed_phase": 5,
  "gate_results": {
    "phase_5": {
      "passed": true,
      "timestamp": "2025-11-13T18:00:00",
      "phase_name": "Integration"
    }
  }
}
```

---

## Troubleshooting

### Gate fails but I think it shouldn't

**Don't rationalize** - if the gate fails, the requirements aren't met.

Common rationalizations that are WRONG:
- ❌ "APIs are correct, just test issues" → Fix the tests
- ❌ "83.3% is close enough" → Must be 100%
- ❌ "These are minor failures" → All failures block

**Correct approach**:
1. Read gate output carefully
2. Fix ALL identified issues
3. Re-run gate
4. Repeat until gate passes

### How to bypass a gate (DISCOURAGED)

Gates are designed to be un-bypassable. If you absolutely must proceed:

1. **Don't** - seriously, fix the issues
2. If you must: Edit orchestration-state.json manually (not recommended)
3. Better: Fix the actual issues the gate identifies

### Gate says tests failing but they pass locally

Check:
- Are you running the same test command as the gate?
- Is pytest installed?
- Are you in the correct directory?
- Did you commit recent changes?

---

## Design Principles

1. **No Judgment Required**: All checks are binary (pass/fail)
2. **Cannot Be Rationalized**: Gate output is definitive
3. **Automatic Execution**: Run at phase boundaries
4. **Clear Feedback**: Explicit about what's blocking
5. **Audit Trail**: All results recorded in state file

---

## Related

- `orchestration/completion_verifier.py` - Runs 13 completion checks
- `CLAUDE.md` - Stopping criteria documentation
- `FailureAssessment/PROGRAMMATIC-ENFORCEMENT-PLAN.md` - Full design doc
