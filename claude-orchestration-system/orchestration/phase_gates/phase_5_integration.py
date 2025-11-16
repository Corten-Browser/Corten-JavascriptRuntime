#!/usr/bin/env python3
"""
Phase 5 Integration Gate

CRITICAL: This gate enforces 100% integration test pass rate.

Exit Codes:
  0 - PASS: All integration tests passing (100%)
  1 - FAIL: Any integration tests failing

NO EXCEPTIONS. NO RATIONALIZATIONS.
If ANY test fails, gate fails.

This gate would have prevented:
- Music Analyzer v1 failure (79.5% pass rate)
- Brain Music Analyzer v2 failure (83.3% pass rate)
"""

import subprocess
import re
from pathlib import Path
import sys


class Phase5IntegrationGate:
    """Phase 5 exit gate: 100% integration tests passing."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.failures = []
        self.test_results = None
        self.passed_count = 0
        self.failed_count = 0

    def validate(self) -> bool:
        """Run Phase 5 validations."""
        self._check_integration_tests_exist()
        if not self.failures:
            self._check_integration_tests_run()
        if not self.failures:
            self._check_integration_tests_pass_100_percent()

        return len(self.failures) == 0

    def _check_integration_tests_exist(self):
        """Verify integration tests exist."""
        test_dirs = [
            self.project_root / "tests" / "integration",
            self.project_root / "integration_tests",
        ]

        found = False
        for test_dir in test_dirs:
            if test_dir.exists():
                # Check for Python test files
                test_files = list(test_dir.glob("test_*.py")) + list(test_dir.glob("*_test.py"))
                if test_files:
                    found = True
                    break

        if not found:
            self.failures.append(
                "No integration tests found in tests/integration/ or integration_tests/\n"
                "  Integration tests are required for Phase 5 completion"
            )

    def _check_integration_tests_run(self):
        """Verify integration tests can run."""
        test_paths = [
            self.project_root / "tests" / "integration",
            self.project_root / "integration_tests",
        ]

        test_path = None
        for path in test_paths:
            if path.exists():
                test_path = path
                break

        if not test_path:
            return  # Already caught by existence check

        try:
            result = subprocess.run(
                ["pytest", str(test_path), "-v", "--collect-only"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30
            )

            if result.returncode != 0:
                self.failures.append(
                    f"Integration tests cannot be collected\n"
                    f"  Error: {result.stderr[:200]}"
                )

        except FileNotFoundError:
            self.failures.append(
                "pytest not installed (required for integration tests)\n"
                "  Install with: pip install pytest"
            )
        except subprocess.TimeoutExpired:
            self.failures.append("Integration test collection timed out")
        except Exception as e:
            self.failures.append(f"Error collecting integration tests: {str(e)}")

    def _check_integration_tests_pass_100_percent(self):
        """
        CRITICAL CHECK: Verify 100% integration test pass rate.

        This is the check that would have prevented both Music Analyzer failures.

        NO EXCEPTIONS:
        - 99% pass rate = FAIL
        - 83.3% pass rate = FAIL (Brain Music Analyzer)
        - 79.5% pass rate = FAIL (Music Analyzer v1)
        - "Minor test issues" = FAIL
        - "APIs are correct" = IRRELEVANT, still FAIL
        - "Just parameter issues" = IRRELEVANT, still FAIL

        Only 100% pass rate = PASS
        """
        test_paths = [
            self.project_root / "tests" / "integration",
            self.project_root / "integration_tests",
        ]

        test_path = None
        for path in test_paths:
            if path.exists():
                test_path = path
                break

        if not test_path:
            return  # Already caught by existence check

        try:
            result = subprocess.run(
                ["pytest", str(test_path), "-v", "--tb=short"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=600  # 10 minutes max
            )

            self.test_results = result.stdout + result.stderr

            # Parse test results
            passed_match = re.search(r'(\d+) passed', self.test_results)
            failed_match = re.search(r'(\d+) failed', self.test_results)
            skipped_match = re.search(r'(\d+) skipped', self.test_results)

            self.passed_count = int(passed_match.group(1)) if passed_match else 0
            self.failed_count = int(failed_match.group(1)) if failed_match else 0
            skipped_count = int(skipped_match.group(1)) if skipped_match else 0

            total_count = self.passed_count + self.failed_count

            if result.returncode != 0:
                # Tests failed
                if total_count > 0:
                    pass_rate = (self.passed_count / total_count * 100)
                else:
                    pass_rate = 0.0

                self.failures.append(
                    f"Integration tests FAILING: {self.failed_count} failed, "
                    f"{self.passed_count} passed ({pass_rate:.1f}% pass rate)\n"
                    f"  REQUIRED: 100% pass rate (0 failures)\n"
                    f"  ACTUAL: {pass_rate:.1f}% pass rate ({self.failed_count} failures)\n"
                    f"  \n"
                    f"  🛑 YOU MUST FIX ALL {self.failed_count} FAILING TESTS BEFORE PROCEEDING\n"
                    f"  🛑 NO EXCEPTIONS - even 1 failing test blocks Phase 6\n"
                    f"  🛑 'APIs are correct' is NOT a valid reason to bypass this gate"
                )

            elif self.failed_count > 0:
                # Edge case: pytest returned 0 but failures detected in output
                self.failures.append(
                    f"Integration tests show {self.failed_count} failures despite exit code 0\n"
                    f"  YOU MUST FIX ALL {self.failed_count} FAILING TESTS BEFORE PROCEEDING"
                )

            if skipped_count > 0:
                # Warn about skipped tests (not blocking, but noteworthy)
                print(f"⚠️  WARNING: {skipped_count} integration tests skipped")

        except subprocess.TimeoutExpired:
            self.failures.append(
                "Integration tests timed out after 10 minutes\n"
                "  This may indicate hanging tests or infinite loops\n"
                "  YOU MUST FIX TIMEOUT ISSUES BEFORE PROCEEDING"
            )
        except Exception as e:
            self.failures.append(f"Error running integration tests: {str(e)}")

    def report(self) -> str:
        """Generate report."""
        if not self.failures:
            return (
                "✅ PHASE 5 INTEGRATION - COMPLETE\n"
                f"   All {self.passed_count} integration tests passing (100%)\n"
                "   May proceed to Phase 6 (Verification)\n"
            )
        else:
            report = (
                "❌ PHASE 5 INTEGRATION - INCOMPLETE\n"
                f"   {len(self.failures)} blocking issue(s):\n\n"
            )
            for i, failure in enumerate(self.failures, 1):
                # Indent multi-line failures
                indented_failure = failure.replace('\n', '\n   ')
                report += f"   {i}. {indented_failure}\n"

            report += (
                "\n"
                "   🛑 CANNOT PROCEED TO PHASE 6\n"
                "   🛑 FIX ALL ISSUES ABOVE BEFORE CONTINUING\n"
                "   🛑 NO EXCEPTIONS - 100% PASS RATE REQUIRED\n"
                "   🛑 DO NOT RATIONALIZE - FIX THE TESTS\n"
            )

            if self.test_results and self.failed_count > 0:
                # Extract failure details
                failure_section_match = re.search(
                    r'FAILED.*?(?=\n=|\nshort test summary|\Z)',
                    self.test_results,
                    re.DOTALL
                )
                if failure_section_match:
                    report += "\n   Recent test failures:\n"
                    failure_lines = failure_section_match.group(0).split('\n')[:10]
                    for line in failure_lines:
                        report += f"   {line}\n"

            return report


def main():
    """CLI entry point."""
    if len(sys.argv) < 2:
        print("Usage: phase_5_integration.py <project_root>")
        print("\nValidates Phase 5 (Integration) is complete.")
        print("Exit code 0 = PASS, 1 = FAIL")
        sys.exit(1)

    project_root = Path(sys.argv[1]).resolve()
    gate = Phase5IntegrationGate(project_root)

    print("🔍 Running Phase 5 Integration Gate...")
    print(f"   Project: {project_root}")
    print()

    passed = gate.validate()
    print(gate.report())

    sys.exit(0 if passed else 1)


if __name__ == '__main__':
    main()
