#!/usr/bin/env python3
"""
Integration Coverage Checker

Prevents Music Analyzer Failure #2 scenario where system was declared
complete with only 37.5% integration test execution (3/8 tests).

This tool verifies:
1. ALL integration tests executed (not just passed)
2. NO tests in "NOT RUN" status
3. 100% execution rate achieved
4. 100% pass rate achieved

Usage:
    python orchestration/integration_coverage_checker.py
    python orchestration/integration_coverage_checker.py --strict  # Exit non-zero on any issue

Exit codes:
    0 - All checks pass (100% execution, 100% pass rate, no NOT RUN)
    1 - Execution rate < 100% OR tests in NOT RUN status
    2 - Pass rate < 100% (failures exist)
    3 - TEST-RESULTS.md file not found

Author: AI Orchestration System v0.7.0
Date: 2025-11-12
"""

import re
import sys
from pathlib import Path
from typing import Dict, List, Optional


class IntegrationCoverageChecker:
    """Check integration test execution coverage and pass rates."""

    def __init__(self, project_root: Path):
        """Initialize checker with project root."""
        self.project_root = Path(project_root)
        self.test_results_path = self._find_test_results_file()

    def _find_test_results_file(self) -> Optional[Path]:
        """Find TEST-RESULTS.md file in standard locations."""
        search_paths = [
            self.project_root / "tests" / "integration" / "TEST-RESULTS.md",
            self.project_root / "TEST-RESULTS.md",
            self.project_root / "tests" / "TEST-RESULTS.md"
        ]

        for path in search_paths:
            if path.exists():
                return path

        return None

    def check_coverage(self) -> Dict:
        """
        Check integration test execution coverage.

        Returns:
            dict with keys:
                - total: Total tests planned
                - executed: Tests that ran
                - passed: Tests that passed
                - failed: Tests that failed
                - not_run: Tests in NOT RUN status
                - execution_rate: % of tests executed
                - pass_rate: % of executed tests that passed
                - overall_rate: % of total tests that passed
                - not_run_tests: List of test names in NOT RUN status
                - failed_tests: List of test names that failed
                - ready: True if 100% execution + 100% pass
                - file_found: True if TEST-RESULTS.md exists
        """
        if not self.test_results_path:
            return {
                'total': 0,
                'executed': 0,
                'passed': 0,
                'failed': 0,
                'not_run': 0,
                'execution_rate': 0.0,
                'pass_rate': 0.0,
                'overall_rate': 0.0,
                'not_run_tests': [],
                'failed_tests': [],
                'ready': False,
                'file_found': False,
                'error': 'TEST-RESULTS.md not found'
            }

        content = self.test_results_path.read_text()

        # Parse metrics
        total = self._extract_total(content)
        executed = self._extract_executed(content)
        passed = self._extract_passed(content)
        failed = self._extract_failed(content)

        # Calculate NOT RUN
        not_run = total - executed if total >= executed else 0

        # Calculate rates
        execution_rate = (executed / total * 100) if total > 0 else 0.0
        pass_rate = (passed / executed * 100) if executed > 0 else 0.0
        overall_rate = (passed / total * 100) if total > 0 else 0.0

        # Extract test names
        not_run_tests = self._extract_not_run_tests(content)
        failed_tests = self._extract_failed_tests(content)

        # Determine readiness
        ready = (
            execution_rate == 100.0 and
            pass_rate == 100.0 and
            len(not_run_tests) == 0 and
            len(failed_tests) == 0
        )

        return {
            'total': total,
            'executed': executed,
            'passed': passed,
            'failed': failed,
            'not_run': not_run,
            'execution_rate': execution_rate,
            'pass_rate': pass_rate,
            'overall_rate': overall_rate,
            'not_run_tests': not_run_tests,
            'failed_tests': failed_tests,
            'ready': ready,
            'file_found': True
        }

    def _extract_total(self, content: str) -> int:
        """Extract total test count from content."""
        # Pattern: "X out of Y planned"
        match = re.search(r'(\d+)\s+out\s+of\s+(\d+)\s+planned', content, re.IGNORECASE)
        if match:
            return int(match.group(2))

        # Fallback: Count test table rows
        test_rows = re.findall(r'^\|\s*\d+\s*\|', content, re.MULTILINE)
        return len(test_rows)

    def _extract_executed(self, content: str) -> int:
        """Extract executed test count from content."""
        # Pattern: "Total Tests Run: X"
        match = re.search(r'Total\s+Tests\s+Run:\s*(\d+)', content, re.IGNORECASE)
        if match:
            return int(match.group(1))

        # Fallback: Count non-NOT RUN entries
        not_run_count = len(re.findall(r'⏭️\s*NOT\s*RUN', content, re.IGNORECASE))
        total_rows = len(re.findall(r'^\|\s*\d+\s*\|', content, re.MULTILINE))
        return total_rows - not_run_count

    def _extract_passed(self, content: str) -> int:
        """Extract passed test count from content."""
        # Pattern: "Tests Passed: X"
        match = re.search(r'Tests\s+Passed:\s*(\d+)', content, re.IGNORECASE)
        if match:
            return int(match.group(1))

        # Fallback: Count ✅ entries
        return len(re.findall(r'✅\s*PASS', content, re.IGNORECASE))

    def _extract_failed(self, content: str) -> int:
        """Extract failed test count from content."""
        # Pattern: "Tests Failed: X"
        match = re.search(r'Tests\s+Failed:\s*(\d+)', content, re.IGNORECASE)
        if match:
            return int(match.group(1))

        # Fallback: Count ❌ entries
        return len(re.findall(r'❌\s*FAIL', content, re.IGNORECASE))

    def _extract_not_run_tests(self, content: str) -> List[str]:
        """Extract test names in NOT RUN status."""
        not_run = []

        # Pattern: | # | Test Name | ⏭️ NOT RUN |
        pattern = r'^\|\s*\d+\s*\|\s*(.+?)\s*\|\s*⏭️\s*NOT\s*RUN'
        for match in re.finditer(pattern, content, re.MULTILINE | re.IGNORECASE):
            test_name = match.group(1).strip()
            not_run.append(test_name)

        return not_run

    def _extract_failed_tests(self, content: str) -> List[str]:
        """Extract test names that failed."""
        failed = []

        # Pattern: | # | Test Name | ❌ FAIL |
        pattern = r'^\|\s*\d+\s*\|\s*(.+?)\s*\|\s*❌\s*FAIL'
        for match in re.finditer(pattern, content, re.MULTILINE | re.IGNORECASE):
            test_name = match.group(1).strip()
            failed.append(test_name)

        return failed


def print_report(coverage: Dict) -> None:
    """Print formatted coverage report."""
    print("╔══════════════════════════════════════════════════════════╗")
    print("║      Integration Test Execution Coverage Report         ║")
    print("╚══════════════════════════════════════════════════════════╝")
    print()

    if not coverage.get('file_found'):
        print("❌ TEST-RESULTS.md not found")
        print()
        print("Expected locations:")
        print("  - tests/integration/TEST-RESULTS.md")
        print("  - TEST-RESULTS.md")
        print("  - tests/TEST-RESULTS.md")
        print()
        print("Integration tests may not have been run yet.")
        return

    # Metrics summary
    print(f"Total Tests Planned:  {coverage['total']}")
    print(f"Tests Executed:       {coverage['executed']}")
    print(f"Tests Passed:         {coverage['passed']}")
    print(f"Tests Failed:         {coverage['failed']}")
    print(f"Tests NOT RUN:        {coverage['not_run']}")
    print()

    # Rates
    print(f"Execution Rate:       {coverage['execution_rate']:.1f}%")
    print(f"Pass Rate:            {coverage['pass_rate']:.1f}%")
    print(f"Overall Rate:         {coverage['overall_rate']:.1f}%")
    print()

    # Status
    print("─" * 60)
    if coverage['ready']:
        print("✅ SYSTEM READY FOR COMPLETION")
        print("   - 100% execution rate (all tests ran)")
        print("   - 100% pass rate (all tests passed)")
        print("   - Zero tests in NOT RUN status")
    else:
        print("❌ SYSTEM NOT READY FOR COMPLETION")
        print()

        if coverage['execution_rate'] < 100.0:
            print(f"  ❌ Execution rate: {coverage['execution_rate']:.1f}% (need 100%)")
            print(f"     {coverage['not_run']} test(s) did not execute")

        if coverage['not_run'] > 0:
            print(f"  ❌ {coverage['not_run']} test(s) in NOT RUN status:")
            for i, test_name in enumerate(coverage['not_run_tests'], 1):
                print(f"     {i}. {test_name}")

        if coverage['pass_rate'] < 100.0 and coverage['executed'] > 0:
            print(f"  ❌ Pass rate: {coverage['pass_rate']:.1f}% (need 100%)")
            print(f"     {coverage['failed']} test(s) failed")

        if len(coverage['failed_tests']) > 0:
            print(f"  ❌ Failed tests:")
            for i, test_name in enumerate(coverage['failed_tests'], 1):
                print(f"     {i}. {test_name}")

        print()
        print("REQUIRED ACTIONS:")
        print("1. Fix ALL failing tests")
        print("2. Re-run ENTIRE integration test suite")
        print("3. Verify 100% execution + 100% pass")
        print("4. Re-run this checker to confirm")

    print("─" * 60)


def get_exit_code(coverage: Dict) -> int:
    """
    Determine exit code based on coverage results.

    Exit codes:
        0 - All checks pass (100% execution, 100% pass rate)
        1 - Execution rate < 100% OR tests in NOT RUN status
        2 - Pass rate < 100% (failures exist)
        3 - TEST-RESULTS.md not found
    """
    if not coverage.get('file_found'):
        return 3

    if coverage['ready']:
        return 0

    # Check execution first (more critical)
    if coverage['execution_rate'] < 100.0 or coverage['not_run'] > 0:
        return 1

    # Then check pass rate
    if coverage['pass_rate'] < 100.0:
        return 2

    return 1  # Fallback


def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Check integration test execution coverage and prevent Failure #2 scenario'
    )
    parser.add_argument(
        '--project-root',
        type=Path,
        default=Path.cwd(),
        help='Project root directory (default: current directory)'
    )
    parser.add_argument(
        '--strict',
        action='store_true',
        help='Exit with non-zero code if any issues found'
    )

    args = parser.parse_args()

    # Check coverage
    checker = IntegrationCoverageChecker(args.project_root)
    coverage = checker.check_coverage()

    # Print report
    print_report(coverage)

    # Exit with appropriate code if strict mode
    if args.strict:
        exit_code = get_exit_code(coverage)
        sys.exit(exit_code)


if __name__ == '__main__':
    main()
