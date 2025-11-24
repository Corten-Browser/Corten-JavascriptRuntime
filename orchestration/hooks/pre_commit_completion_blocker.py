#!/usr/bin/env python3
"""
Pre-commit hook that blocks COMPLETION-REPORT.md commits
until all verification checks pass.

This is TECHNICAL ENFORCEMENT - cannot be bypassed by rationalization.
"""
import subprocess
import sys
import json
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()

# Files that indicate completion intent
COMPLETION_FILES = [
    "COMPLETION-REPORT.md",
    "PROJECT-COMPLETION-REPORT.md",
    "FINAL-REPORT.md",
]


def get_staged_files() -> list[str]:
    """Get list of files staged for commit."""
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-only"],
        capture_output=True,
        text=True
    )
    return [f for f in result.stdout.strip().split('\n') if f]


def check_completion_authorized() -> tuple[bool, str]:
    """Check if completion is authorized by verification system."""
    state_file = _paths.completion_state

    if not state_file.exists():
        return False, "No verification state found. Run verification first:\n  python orchestration/verification/run_full_verification.py"

    try:
        with open(state_file) as f:
            state = json.load(f)
    except json.JSONDecodeError:
        return False, "Verification state file is corrupted. Re-run verification."

    checks = [
        ("all_gates_passed", "All phase gates must pass"),
        ("spec_coverage_100", "Specification coverage must be 100%"),
        ("verification_agent_approved", "Verification agent must approve"),
        ("smoke_tests_passed", "All smoke tests must pass"),
        ("no_stub_components", "No stub/placeholder components allowed"),
    ]

    failures = []
    for check_name, description in checks:
        if not state.get(check_name, False):
            failures.append(f"  {description}")

    if failures:
        failure_msg = "\n".join(failures)
        return False, f"Failed checks:\n{failure_msg}"

    return True, "All verification checks passed"


def main():
    staged_files = get_staged_files()

    if not staged_files:
        sys.exit(0)  # No files staged

    # Check if any completion files are being committed
    completion_files_staged = [
        f for f in staged_files
        if any(cf in f for cf in COMPLETION_FILES)
    ]

    if not completion_files_staged:
        sys.exit(0)  # No completion files, allow commit

    print("=" * 60)
    print("COMPLETION REPORT COMMIT DETECTED")
    print("=" * 60)
    print(f"Files: {', '.join(completion_files_staged)}")
    print("")
    print("Running automated verification checks...")
    print("")

    authorized, message = check_completion_authorized()

    if not authorized:
        print("COMMIT BLOCKED - COMPLETION NOT VERIFIED")
        print("")
        print(message)
        print("")
        print("To resolve:")
        print("1. Run: python orchestration/verification/run_full_verification.py")
        print("2. Fix all identified issues")
        print("3. Re-run verification until all checks pass")
        print("4. Then retry commit")
        print("")
        print("=" * 60)
        sys.exit(1)

    print("COMPLETION VERIFIED - Commit allowed")
    print(message)
    sys.exit(0)


if __name__ == "__main__":
    main()
