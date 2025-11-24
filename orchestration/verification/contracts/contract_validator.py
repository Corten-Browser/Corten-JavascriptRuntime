#!/usr/bin/env python3
"""
Contract Validator - Pre-Integration Gate

Validates that all components implement their contracts exactly before
integration testing. Catches API mismatches early in the development cycle.

This tool prevents the "Music Analyzer Catastrophe" where components had
wrong method names (scan vs get_audio_files) that passed unit tests with
mocks but failed catastrophically in integration.

Usage:
    python orchestration/contract_validator.py --all          # Validate all components
    python orchestration/contract_validator.py auth-service   # Validate specific component
    python orchestration/contract_validator.py --verbose      # Show detailed output
"""

import subprocess
import sys
import os
from pathlib import Path
from typing import List, Tuple, Dict
import argparse
import json


class ContractValidator:
    """Validates component contract compliance before integration testing."""

    def __init__(self, project_root: Path = None, verbose: bool = False):
        self.project_root = project_root or Path.cwd()
        self.components_dir = self.project_root / "components"
        self.verbose = verbose
        self.results: Dict[str, dict] = {}

    def discover_components(self) -> List[str]:
        """Discover all components with contract tests."""
        if not self.components_dir.exists():
            return []

        components = []
        for component_path in self.components_dir.iterdir():
            if component_path.is_dir() and not component_path.name.startswith('.'):
                # Check if component has contract tests
                contract_tests = component_path / "tests" / "contracts"
                if contract_tests.exists() and any(contract_tests.glob("test_*.py")):
                    components.append(component_path.name)

        return sorted(components)

    def validate_component(self, component_name: str) -> Tuple[bool, dict]:
        """
        Validate a single component's contract compliance.

        Returns:
            Tuple of (success: bool, results: dict)
        """
        component_path = self.components_dir / component_name
        contract_tests_path = component_path / "tests" / "contracts"

        if not component_path.exists():
            return False, {
                "status": "error",
                "message": f"Component directory not found: {component_path}",
                "passed": 0,
                "failed": 0,
                "total": 0
            }

        if not contract_tests_path.exists():
            return False, {
                "status": "error",
                "message": f"Contract tests directory not found: {contract_tests_path}",
                "passed": 0,
                "failed": 0,
                "total": 0
            }

        # Count contract test files
        test_files = list(contract_tests_path.glob("test_*.py"))
        if not test_files:
            return False, {
                "status": "error",
                "message": "No contract test files found",
                "passed": 0,
                "failed": 0,
                "total": 0
            }

        # Run pytest on contract tests
        try:
            cmd = [
                "pytest",
                str(contract_tests_path),
                "-v",
                "--tb=short",
                "--no-header",
                "-q"
            ]

            if self.verbose:
                print(f"Running: {' '.join(cmd)}")

            result = subprocess.run(
                cmd,
                cwd=component_path,
                capture_output=True,
                text=True,
                timeout=60
            )

            # Parse pytest output
            output = result.stdout + result.stderr
            passed, failed, total = self._parse_pytest_output(output)

            success = (result.returncode == 0 and failed == 0)

            return success, {
                "status": "pass" if success else "fail",
                "message": output if not success or self.verbose else "All contract tests passed",
                "passed": passed,
                "failed": failed,
                "total": total,
                "returncode": result.returncode
            }

        except subprocess.TimeoutExpired:
            return False, {
                "status": "error",
                "message": "Contract tests timed out (>60s)",
                "passed": 0,
                "failed": 0,
                "total": 0
            }
        except Exception as e:
            return False, {
                "status": "error",
                "message": f"Failed to run contract tests: {str(e)}",
                "passed": 0,
                "failed": 0,
                "total": 0
            }

    def _parse_pytest_output(self, output: str) -> Tuple[int, int, int]:
        """
        Parse pytest output to extract test counts.

        Returns:
            Tuple of (passed, failed, total)
        """
        passed = 0
        failed = 0

        # Look for pytest summary line (e.g., "10 passed, 2 failed in 1.23s")
        for line in output.split('\n'):
            if 'passed' in line or 'failed' in line:
                if 'passed' in line:
                    try:
                        passed = int(line.split('passed')[0].strip().split()[-1])
                    except (ValueError, IndexError):
                        pass
                if 'failed' in line:
                    try:
                        failed = int(line.split('failed')[0].strip().split()[-1])
                    except (ValueError, IndexError):
                        pass

        total = passed + failed
        return passed, failed, total

    def validate_all(self, components: List[str] = None) -> bool:
        """
        Validate all components or specified list.

        Returns:
            True if all components pass, False otherwise
        """
        if components is None:
            components = self.discover_components()

        if not components:
            print("⚠️  No components with contract tests found")
            return False

        print(f"\n🔍 Contract Validation - Pre-Integration Gate")
        print(f"   Validating {len(components)} component(s)\n")

        all_passed = True

        for component_name in components:
            success, results = self.validate_component(component_name)
            self.results[component_name] = results

            status_icon = "✅" if success else "❌"
            status_text = f"{results['passed']}/{results['total']} pass" if results['total'] > 0 else "No tests"

            print(f"{status_icon} {component_name}: {status_text}")

            if not success:
                all_passed = False
                if self.verbose or results['status'] == 'error':
                    print(f"   {results['message']}\n")
                elif results['failed'] > 0:
                    print(f"   ⚠️  {results['failed']} contract test(s) failed")
                    print(f"   🛑 CRITICAL: Fix component API to match contract\n")

        print()

        if all_passed:
            print("✅ ALL COMPONENTS PASS CONTRACT VALIDATION")
            print("   Proceeding to integration testing is safe\n")
            return True
        else:
            print("❌ CONTRACT VALIDATION FAILED")
            print("   🛑 STOP - Do NOT proceed to integration testing")
            print("   📝 Fix component implementations (NOT contracts)")
            print("   🔄 Re-run contract validation until 100% pass\n")
            return False

    def print_summary(self):
        """Print detailed summary of validation results."""
        print("\n" + "=" * 70)
        print("CONTRACT VALIDATION SUMMARY")
        print("=" * 70 + "\n")

        total_components = len(self.results)
        passed_components = sum(1 for r in self.results.values() if r['status'] == 'pass')
        failed_components = total_components - passed_components

        total_tests = sum(r['total'] for r in self.results.values())
        passed_tests = sum(r['passed'] for r in self.results.values())
        failed_tests = sum(r['failed'] for r in self.results.values())

        print(f"Components: {passed_components}/{total_components} pass")
        print(f"Tests:      {passed_tests}/{total_tests} pass\n")

        if failed_components > 0:
            print("Failed Components:")
            for component, results in self.results.items():
                if results['status'] != 'pass':
                    print(f"  ❌ {component}: {results['failed']} test(s) failed")

        print()


def main():
    parser = argparse.ArgumentParser(
        description="Validate component contract compliance before integration testing",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Validate all components
  python orchestration/contract_validator.py --all

  # Validate specific component
  python orchestration/contract_validator.py auth-service

  # Validate multiple components
  python orchestration/contract_validator.py auth-service user-service

  # Verbose output
  python orchestration/contract_validator.py --all --verbose

Exit Codes:
  0 - All contract tests passed (100%)
  1 - One or more contract tests failed
  2 - Error running validation
        """
    )

    parser.add_argument(
        'components',
        nargs='*',
        help='Component names to validate (omit to use --all)'
    )
    parser.add_argument(
        '--all',
        action='store_true',
        help='Validate all components with contract tests'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Show detailed output including test output'
    )
    parser.add_argument(
        '--summary',
        action='store_true',
        help='Print detailed summary at end'
    )

    args = parser.parse_args()

    # Validation
    if not args.all and not args.components:
        parser.error("Must specify component names or use --all")

    # Initialize validator
    validator = ContractValidator(verbose=args.verbose)

    # Validate
    if args.all:
        success = validator.validate_all()
    else:
        success = validator.validate_all(components=args.components)

    # Print summary if requested
    if args.summary:
        validator.print_summary()

    # Exit with appropriate code
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
