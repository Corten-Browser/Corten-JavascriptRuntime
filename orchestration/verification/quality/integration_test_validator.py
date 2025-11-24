#!/usr/bin/env python3
"""
Integration Test Validator

Validates that integration tests use real components, not mocks.

Part of the adaptive orchestration system (v0.10.0).
Prevents Brain Music Analyzer-type failures.
"""

from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass
from enum import Enum
import re


class Severity(Enum):
    """Issue severity levels."""
    CRITICAL = "CRITICAL"  # Blocks progression
    WARNING = "WARNING"    # Should be reviewed
    INFO = "INFO"          # Informational


@dataclass
class ValidationIssue:
    """Represents a validation issue found in tests."""
    severity: Severity
    file: str
    line: Optional[int]
    message: str
    reference: str  # Link to docs
    code_snippet: Optional[str] = None


class IntegrationTestValidator:
    """Validates integration tests follow no-mocking policy."""

    def __init__(self, project_root: Path):
        """
        Initialize validator.

        Args:
            project_root: Path to project root directory
        """
        self.project_root = Path(project_root).resolve()
        self.integration_test_dir = self.project_root / "tests" / "integration"

    def validate(self) -> List[ValidationIssue]:
        """
        Run all validation checks.

        Returns:
            List of validation issues found
        """
        issues = []

        if not self.integration_test_dir.exists():
            return []  # No integration tests yet

        for test_file in self.integration_test_dir.glob("test_*.py"):
            issues.extend(self._validate_file(test_file))

        return issues

    def _validate_file(self, test_file: Path) -> List[ValidationIssue]:
        """Validate a single test file."""
        issues = []

        try:
            content = test_file.read_text()
            lines = content.split('\n')

            # Check for forbidden imports
            issues.extend(self._check_mock_imports(test_file, content, lines))

            # Check for @patch decorators
            issues.extend(self._check_patch_decorators(test_file, content, lines))

            # Check for real component imports (should have them)
            issues.extend(self._check_real_imports(test_file, content, lines))

        except Exception as e:
            issues.append(ValidationIssue(
                severity=Severity.WARNING,
                file=test_file.name,
                line=None,
                message=f"Could not validate file: {str(e)}",
                reference=""
            ))

        return issues

    def _check_mock_imports(self, file: Path, content: str, lines: List[str]) -> List[ValidationIssue]:
        """Check for forbidden mock imports."""
        issues = []

        forbidden_patterns = [
            (r'from unittest\.mock import.*Mock', "unittest.mock.Mock"),
            (r'from mock import', "mock library"),
            (r'import unittest\.mock', "unittest.mock module"),
            (r'from unittest import.*mock', "unittest.mock"),
        ]

        for pattern, description in forbidden_patterns:
            matches = list(re.finditer(pattern, content, re.IGNORECASE))
            for match in matches:
                line_num = self._get_line_number(content, match.start())
                line_content = lines[line_num - 1] if line_num <= len(lines) else ""

                issues.append(ValidationIssue(
                    severity=Severity.CRITICAL,
                    file=file.name,
                    line=line_num,
                    message=f"Integration tests must NOT import {description}",
                    reference="docs/TESTING-STRATEGY.md - When Mocks Are FORBIDDEN",
                    code_snippet=line_content.strip()
                ))

        return issues

    def _check_patch_decorators(self, file: Path, content: str, lines: List[str]) -> List[ValidationIssue]:
        """Check for @patch decorators on cross-component tests."""
        issues = []

        # Look for @patch('components.
        pattern = r'@patch\([\'"]components\.'
        matches = list(re.finditer(pattern, content))

        for match in matches:
            line_num = self._get_line_number(content, match.start())
            line_content = lines[line_num - 1] if line_num <= len(lines) else ""

            issues.append(ValidationIssue(
                severity=Severity.CRITICAL,
                file=file.name,
                line=line_num,
                message="Integration tests must NOT use @patch for components being tested",
                reference="docs/TESTING-STRATEGY.md - Cross-Component Integration Testing",
                code_snippet=line_content.strip()
            ))

        return issues

    def _check_real_imports(self, file: Path, content: str, lines: List[str]) -> List[ValidationIssue]:
        """Check that file imports real components."""
        issues = []

        # Look for "from components.XXX import"
        has_real_imports = bool(re.search(r'from components\.[\w_]+\.src', content))

        if not has_real_imports:
            issues.append(ValidationIssue(
                severity=Severity.WARNING,
                file=file.name,
                line=None,
                message="No real component imports detected - verify this file tests cross-component integration",
                reference="Integration tests should import from components/*/src/"
            ))

        return issues

    def _get_line_number(self, content: str, position: int) -> int:
        """Get line number for a character position in content."""
        return content[:position].count('\n') + 1

    def print_report(self, issues: List[ValidationIssue]) -> None:
        """Print validation report."""
        if not issues:
            print("✅ Integration test validation PASSED")
            print("   All tests use real components (no mocking detected)")
            return

        # Group by severity
        critical = [i for i in issues if i.severity == Severity.CRITICAL]
        warnings = [i for i in issues if i.severity == Severity.WARNING]

        print("❌ Integration test validation FAILED")
        print(f"   Found {len(critical)} critical issues, {len(warnings)} warnings")
        print()

        if critical:
            print("CRITICAL ISSUES (must fix before proceeding):")
            print("-" * 60)
            for issue in critical:
                self._print_issue(issue)

        if warnings:
            print("\nWARNINGS (review recommended):")
            print("-" * 60)
            for issue in warnings:
                self._print_issue(issue)

        print("\n" + "=" * 60)
        if critical:
            print("RESULT: BLOCKED - Fix critical issues before proceeding")
        else:
            print("RESULT: WARNING - Review issues before proceeding")

    def _print_issue(self, issue: ValidationIssue) -> None:
        """Print a single issue."""
        location = f"{issue.file}"
        if issue.line:
            location += f":{issue.line}"

        print(f"\n[{issue.severity.value}] {location}")
        print(f"  {issue.message}")

        if issue.code_snippet:
            print(f"  Code: {issue.code_snippet}")

        if issue.reference:
            print(f"  See: {issue.reference}")


def main():
    """CLI interface for integration test validation."""
    import sys
    import argparse

    parser = argparse.ArgumentParser(
        description='Validate integration tests follow no-mocking policy'
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
        help='Treat warnings as errors'
    )

    args = parser.parse_args()

    # Run validation
    validator = IntegrationTestValidator(args.project_root)
    issues = validator.validate()

    # Print report
    validator.print_report(issues)

    # Exit code
    critical_count = sum(1 for i in issues if i.severity == Severity.CRITICAL)
    warning_count = sum(1 for i in issues if i.severity == Severity.WARNING)

    if critical_count > 0:
        sys.exit(1)  # Critical issues
    elif args.strict and warning_count > 0:
        sys.exit(2)  # Warnings in strict mode
    else:
        sys.exit(0)  # Success


if __name__ == '__main__':
    main()
