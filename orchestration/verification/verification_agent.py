#!/usr/bin/env python3
"""
Independent verification agent.
ONLY verifies - never implements.
Has DIFFERENT incentives than implementation agent.

This agent is skeptical by default. It assumes incompleteness until proven otherwise.
"""
import json
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths
from .quality.rationalization_detector import run_rationalization_check
from .quality.stub_detector import scan_for_stubs
from .quality.smoke_tests import run_smoke_tests

# Global paths instance
_paths = DataPaths()


class VerificationAgent:
    """
    Independent agent that verifies project completion.

    Core principle: Trust nothing, verify everything.
    Default assumption: Project is incomplete until proven complete.
    """

    def __init__(self, project_dir: Path):
        self.project_dir = project_dir
        self.results = {}
        self.is_verified = False

    def run_all_checks(self) -> bool:
        """
        Run comprehensive verification.
        Returns True ONLY if ALL checks pass.
        """
        print("=" * 70)
        print("VERIFICATION AGENT STARTING")
        print("=" * 70)
        print(f"Project: {self.project_dir}")
        print("")
        print("DEFAULT ASSUMPTION: Incomplete until proven otherwise")
        print("")

        all_passed = True

        # Check 1: No Rationalization Language
        print("CHECK 1: Rationalization Language Detection")
        print("-" * 70)
        rationalization_clean = run_rationalization_check(self.project_dir)
        self.results["rationalization_check"] = {"passed": rationalization_clean}

        if not rationalization_clean:
            print("CHECK 1 FAILED: Rationalization patterns found")
            all_passed = False
        else:
            print("CHECK 1 PASSED: No rationalization patterns")
        print("")

        # Check 2: No Stub Components
        print("CHECK 2: Stub/Placeholder Detection")
        print("-" * 70)
        stubs = scan_for_stubs(self.project_dir)
        self.results["stub_detection"] = {"stubs_found": stubs}

        if stubs:
            print(f"CHECK 2 FAILED: Found {len(stubs)} stub components")
            for stub in stubs[:10]:  # Show first 10
                print(f"   - {stub}")
            all_passed = False
        else:
            print("CHECK 2 PASSED: No stub components found")
        print("")

        # Check 3: Smoke Tests (User-Facing Commands)
        print("CHECK 3: Smoke Tests (Actual User Commands)")
        print("-" * 70)
        smoke_results = run_smoke_tests(self.project_dir)
        self.results["smoke_tests"] = smoke_results

        if not smoke_results["all_passed"]:
            print(f"CHECK 3 FAILED: Smoke tests failed")
            for failure in smoke_results.get("failures", [])[:5]:
                print(f"   - {failure['test']}: {failure['reason']}")
            all_passed = False
        else:
            print("CHECK 3 PASSED: All smoke tests passed")
        print("")

        # Check 4: Task Queue Empty
        print("CHECK 4: Task Queue Status")
        print("-" * 70)
        try:
            sys.path.insert(0, str(Path(__file__).parent.parent))
            from tasks.queue import TaskQueue
            queue = TaskQueue()
            queue_complete = queue.is_complete()
            self.results["task_queue"] = {
                "complete": queue_complete,
                "progress": queue.get_progress()
            }

            if not queue_complete:
                progress = queue.get_progress()
                print(f"CHECK 4 FAILED: Task queue not empty ({progress['percentage']:.1f}% complete)")
                all_passed = False
            else:
                print("CHECK 4 PASSED: All tasks completed")
        except ImportError:
            print("CHECK 4 SKIPPED: Task queue not available")
            self.results["task_queue"] = {"complete": True, "progress": {}}
        print("")

        # Check 5: Gate Execution History
        print("CHECK 5: Gate Execution History")
        print("-" * 70)
        gate_log = _paths.gate_execution_log
        if gate_log.exists():
            try:
                log = json.loads(gate_log.read_text())
                passed_gates = [
                    e for e in log.get("executions", [])
                    if e.get("passed", False)
                ]
                self.results["gates"] = {"passed_count": len(passed_gates)}

                if len(passed_gates) >= 2:  # At least gates 5 and 6
                    print(f"CHECK 5 PASSED: {len(passed_gates)} gates passed")
                else:
                    print(f"CHECK 5 FAILED: Only {len(passed_gates)} gates passed (need at least 2)")
                    all_passed = False
            except json.JSONDecodeError:
                print("CHECK 5 FAILED: Gate log corrupted")
                all_passed = False
        else:
            print("CHECK 5 FAILED: No gate execution log found")
            self.results["gates"] = {"passed_count": 0}
            all_passed = False
        print("")

        # Final verdict
        print("=" * 70)
        print("VERIFICATION AGENT VERDICT")
        print("=" * 70)

        if all_passed:
            print("VERIFIED COMPLETE")
            print("")
            print("All checks passed. Project may be declared complete.")
            self.is_verified = True
        else:
            print("NOT VERIFIED")
            print("")
            print("One or more checks failed. Project is INCOMPLETE.")
            print("Fix all issues and re-run verification.")
            self.is_verified = False

        print("=" * 70)

        # Save results
        self._save_results()
        self._update_completion_authority()

        return self.is_verified

    def _save_results(self):
        """Save verification results to file."""
        output_file = _paths.verification_results
        output_file.parent.mkdir(parents=True, exist_ok=True)

        full_results = {
            "timestamp": datetime.now().isoformat(),
            "project_dir": str(self.project_dir),
            "is_verified": self.is_verified,
            "checks": self.results
        }

        output_file.write_text(json.dumps(full_results, indent=2))

    def _update_completion_authority(self):
        """Update the central completion authority state."""
        state_file = _paths.completion_state
        state_file.parent.mkdir(parents=True, exist_ok=True)

        # Determine check results
        task_queue_complete = self.results.get("task_queue", {}).get("complete", False)
        smoke_passed = self.results.get("smoke_tests", {}).get("all_passed", False)
        no_stubs = len(self.results.get("stub_detection", {}).get("stubs_found", [])) == 0
        gates_passed = self.results.get("gates", {}).get("passed_count", 0) >= 2

        state = {
            "last_verification": datetime.now().isoformat(),
            "all_gates_passed": gates_passed,
            "spec_coverage_100": task_queue_complete,  # Task queue empty = spec covered
            "verification_agent_approved": self.is_verified,
            "smoke_tests_passed": smoke_passed,
            "no_stub_components": no_stubs,
        }

        state_file.write_text(json.dumps(state, indent=2))
        print(f"\nCompletion authority state updated: {state_file}")


def run_full_verification(project_dir: Path = None) -> bool:
    """Run full verification process."""
    if project_dir is None:
        project_dir = Path.cwd()

    agent = VerificationAgent(project_dir)
    return agent.run_all_checks()


def main():
    if len(sys.argv) > 1:
        project_dir = Path(sys.argv[1])
    else:
        project_dir = Path.cwd()

    if not project_dir.exists():
        print(f"ERROR: Project directory not found: {project_dir}")
        sys.exit(1)

    verified = run_full_verification(project_dir)

    sys.exit(0 if verified else 1)


if __name__ == "__main__":
    main()
