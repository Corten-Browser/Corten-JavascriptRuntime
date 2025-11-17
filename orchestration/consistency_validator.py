#!/usr/bin/env python3
"""
Cross-Component Consistency Validator

Ensures all components follow shared standards.
Part of v0.4.0 quality enhancement system.

Usage:
    # Validate single component
    python consistency_validator.py --component auth-service

    # Validate all components
    python consistency_validator.py --all

    # Generate detailed report
    python consistency_validator.py --all --report violations.json
"""

import ast
import re
import json
from pathlib import Path
from typing import List, Dict, Set, Optional, Tuple
from dataclasses import dataclass, asdict
import argparse
import sys


@dataclass
class ConsistencyViolation:
    """A standards violation."""
    file_path: str
    line_number: int
    violation_type: str
    description: str
    expected: str
    found: str
    severity: str = "error"  # error, warning

    def __str__(self) -> str:
        """Human-readable string representation."""
        return (
            f"{self.severity.upper()}: {self.file_path}:{self.line_number}\n"
            f"  {self.violation_type}: {self.description}\n"
            f"  Expected: {self.expected}\n"
            f"  Found: {self.found}"
        )


class PythonCodeAnalyzer(ast.NodeVisitor):
    """AST-based Python code analyzer."""

    def __init__(self, file_path: str):
        self.file_path = file_path
        self.violations: List[ConsistencyViolation] = []
        self.imports: Set[str] = set()
        self.error_codes_used: Set[str] = set()
        self.timeout_values: List[Tuple[int, int]] = []  # (line_number, timeout_value)
        self.response_classes_used: Set[str] = set()

    def visit_Import(self, node: ast.Import):
        """Track import statements."""
        for alias in node.names:
            self.imports.add(alias.name)
        self.generic_visit(node)

    def visit_ImportFrom(self, node: ast.ImportFrom):
        """Track from...import statements."""
        if node.module:
            for alias in node.names:
                self.imports.add(f"{node.module}.{alias.name}")
        self.generic_visit(node)

    def visit_Assign(self, node: ast.Assign):
        """Track assignments for error codes and timeouts."""
        # Check for hard-coded error codes
        if isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
            value = node.value.value
            # Check if looks like an error code
            if value.isupper() and '_' in value and any(
                keyword in value for keyword in ['ERROR', 'FAILED', 'INVALID', 'UNAUTHORIZED']
            ):
                self.error_codes_used.add(value)

        self.generic_visit(node)

    def visit_Call(self, node: ast.Call):
        """Track function calls for timeout parameters."""
        # Check for timeout arguments
        for keyword in node.keywords:
            if keyword.arg == 'timeout':
                if isinstance(keyword.value, ast.Constant):
                    if isinstance(keyword.value.value, (int, float)):
                        self.timeout_values.append((node.lineno, keyword.value.value))

        # Check for response class usage
        if isinstance(node.func, ast.Name):
            self.response_classes_used.add(node.func.id)

        self.generic_visit(node)


class ConsistencyValidator:
    """Validates component compliance with shared standards."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self.components_dir = self.project_root / "components"
        self.shared_libs_dir = self.project_root / "shared-libs"

        # Standard error codes from shared_libs.standards
        self.standard_error_codes = {
            "VALIDATION_FAILED", "NOT_FOUND", "UNAUTHORIZED", "FORBIDDEN",
            "CONFLICT", "RATE_LIMIT_EXCEEDED", "BAD_REQUEST", "INVALID_INPUT",
            "INTERNAL_ERROR", "SERVICE_UNAVAILABLE", "TIMEOUT", "DEPENDENCY_FAILED",
            "DATABASE_ERROR", "EXTERNAL_API_ERROR"
        }

        # Standard timeout values
        self.standard_timeouts = {
            1, 2, 5, 10, 15, 30, 60, 120, 300, 900
        }

    def validate_component(self, component_path: Path) -> List[ConsistencyViolation]:
        """
        Validate entire component.

        Args:
            component_path: Path to component directory

        Returns:
            List of violations found
        """
        violations = []

        # Find all Python files
        python_files = list(component_path.rglob("*.py"))

        for py_file in python_files:
            # Skip __pycache__ and test files for some checks
            if "__pycache__" in str(py_file):
                continue

            violations.extend(self.validate_file(py_file))

        return violations

    def validate_file(self, file_path: Path) -> List[ConsistencyViolation]:
        """
        Validate a single Python file.

        Args:
            file_path: Path to Python file

        Returns:
            List of violations found
        """
        violations = []

        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()

            # Parse AST
            try:
                tree = ast.parse(content)
                analyzer = PythonCodeAnalyzer(str(file_path))
                analyzer.visit(tree)

                # Run various checks
                violations.extend(self.validate_error_codes(file_path, content, analyzer))
                violations.extend(self.validate_response_format(file_path, content, analyzer))
                violations.extend(self.validate_timeout_usage(file_path, content, analyzer))
                violations.extend(self.validate_datetime_format(file_path, content))
                violations.extend(self.validate_imports(file_path, analyzer))

            except SyntaxError as e:
                violations.append(ConsistencyViolation(
                    file_path=str(file_path),
                    line_number=e.lineno or 0,
                    violation_type="SYNTAX_ERROR",
                    description="File contains syntax errors",
                    expected="Valid Python syntax",
                    found=str(e),
                    severity="error"
                ))

        except Exception as e:
            violations.append(ConsistencyViolation(
                file_path=str(file_path),
                line_number=0,
                violation_type="FILE_READ_ERROR",
                description=f"Could not read file: {e}",
                expected="Readable file",
                found=str(e),
                severity="warning"
            ))

        return violations

    def validate_error_codes(
        self,
        file_path: Path,
        content: str,
        analyzer: PythonCodeAnalyzer
    ) -> List[ConsistencyViolation]:
        """
        Ensure only standard error codes are used.

        Args:
            file_path: Path to file
            content: File content
            analyzer: AST analyzer

        Returns:
            List of violations
        """
        violations = []

        # Check for hard-coded error codes
        for error_code in analyzer.error_codes_used:
            if error_code not in self.standard_error_codes:
                # Find line number
                for line_num, line in enumerate(content.split('\n'), 1):
                    if f'"{error_code}"' in line or f"'{error_code}'" in line:
                        violations.append(ConsistencyViolation(
                            file_path=str(file_path),
                            line_number=line_num,
                            violation_type="NON_STANDARD_ERROR_CODE",
                            description=f"Custom error code '{error_code}' used",
                            expected="Use ErrorCodes from shared_libs.standards",
                            found=f"Hard-coded string '{error_code}'",
                            severity="error"
                        ))
                        break

        # Check if file uses errors but doesn't import ErrorCodes
        has_error_handling = any(
            pattern in content for pattern in [
                'raise ', 'except ', 'error', 'Error'
            ]
        )

        if has_error_handling and not any(
            'shared_libs.standards' in imp and 'ErrorCodes' in imp
            for imp in analyzer.imports
        ):
            # Check if ErrorCodes is used anywhere
            if 'ErrorCodes' in content:
                violations.append(ConsistencyViolation(
                    file_path=str(file_path),
                    line_number=1,
                    violation_type="MISSING_ERROR_CODES_IMPORT",
                    description="File uses error handling but doesn't import ErrorCodes",
                    expected="from shared_libs.standards import ErrorCodes",
                    found="Missing import",
                    severity="warning"
                ))

        return violations

    def validate_response_format(
        self,
        file_path: Path,
        content: str,
        analyzer: PythonCodeAnalyzer
    ) -> List[ConsistencyViolation]:
        """
        Ensure all responses use ResponseEnvelope.

        Args:
            file_path: Path to file
            content: File content
            analyzer: AST analyzer

        Returns:
            List of violations
        """
        violations = []

        # Skip test files
        if 'test_' in file_path.name or file_path.name.endswith('_test.py'):
            return violations

        # Check for API endpoint patterns
        api_patterns = [
            r'@app\.route',
            r'@router\.',
            r'def.*\(.*request',
            r'return.*jsonify',
            r'return.*JSONResponse',
        ]

        has_api_endpoint = any(
            re.search(pattern, content, re.IGNORECASE)
            for pattern in api_patterns
        )

        if has_api_endpoint:
            # Check if ResponseEnvelope is imported
            has_response_envelope = any(
                'ResponseEnvelope' in imp
                for imp in analyzer.imports
            )

            if not has_response_envelope and 'ResponseEnvelope' not in content:
                violations.append(ConsistencyViolation(
                    file_path=str(file_path),
                    line_number=1,
                    violation_type="MISSING_RESPONSE_ENVELOPE",
                    description="API endpoint file doesn't use ResponseEnvelope",
                    expected="from shared_libs.standards import ResponseEnvelope",
                    found="Missing import and usage",
                    severity="warning"
                ))

        # Check for raw dict returns that look like API responses
        raw_response_pattern = r'return\s+\{[^}]*["\']success["\']'
        matches = re.finditer(raw_response_pattern, content)
        for match in matches:
            line_num = content[:match.start()].count('\n') + 1
            violations.append(ConsistencyViolation(
                file_path=str(file_path),
                line_number=line_num,
                violation_type="RAW_DICT_RESPONSE",
                description="Raw dict returned instead of ResponseEnvelope",
                expected="ResponseEnvelope(data=...)",
                found="return {'success': ...}",
                severity="error"
            ))

        return violations

    def validate_timeout_usage(
        self,
        file_path: Path,
        content: str,
        analyzer: PythonCodeAnalyzer
    ) -> List[ConsistencyViolation]:
        """
        Ensure all external calls use standard timeouts.

        Args:
            file_path: Path to file
            content: File content
            analyzer: AST analyzer

        Returns:
            List of violations
        """
        violations = []

        # Check for hard-coded timeout values
        for line_num, timeout_value in analyzer.timeout_values:
            if timeout_value not in self.standard_timeouts:
                violations.append(ConsistencyViolation(
                    file_path=str(file_path),
                    line_number=line_num,
                    violation_type="NON_STANDARD_TIMEOUT",
                    description=f"Non-standard timeout value {timeout_value}",
                    expected="Use TimeoutDefaults from shared_libs.standards",
                    found=f"timeout={timeout_value}",
                    severity="warning"
                ))

        # Check if file makes HTTP requests but doesn't import TimeoutDefaults
        has_http_calls = any(
            pattern in content for pattern in [
                'requests.', 'httpx.', 'urllib.', 'aiohttp.'
            ]
        )

        if has_http_calls and not any(
            'TimeoutDefaults' in imp
            for imp in analyzer.imports
        ):
            violations.append(ConsistencyViolation(
                file_path=str(file_path),
                line_number=1,
                violation_type="MISSING_TIMEOUT_DEFAULTS",
                description="File makes HTTP calls but doesn't import TimeoutDefaults",
                expected="from shared_libs.standards import TimeoutDefaults",
                found="Missing import",
                severity="warning"
            ))

        return violations

    def validate_datetime_format(
        self,
        file_path: Path,
        content: str
    ) -> List[ConsistencyViolation]:
        """
        Ensure ISO8601 format used.

        Args:
            file_path: Path to file
            content: File content

        Returns:
            List of violations
        """
        violations = []

        # Check for Unix timestamp usage in APIs
        unix_timestamp_patterns = [
            r'\.timestamp\(\)',
            r'time\.time\(\)',
            r'int\(.*\.timestamp',
        ]

        for pattern in unix_timestamp_patterns:
            matches = re.finditer(pattern, content)
            for match in matches:
                line_num = content[:match.start()].count('\n') + 1
                violations.append(ConsistencyViolation(
                    file_path=str(file_path),
                    line_number=line_num,
                    violation_type="UNIX_TIMESTAMP_USAGE",
                    description="Unix timestamp used instead of ISO8601",
                    expected="DateTimeFormats.now_iso8601() from shared_libs.standards",
                    found=match.group(),
                    severity="warning"
                ))

        # Check for strftime with custom formats
        custom_strftime_pattern = r'strftime\(["\'](?!%Y-%m-%dT%H:%M:%S)'
        matches = re.finditer(custom_strftime_pattern, content)
        for match in matches:
            line_num = content[:match.start()].count('\n') + 1
            violations.append(ConsistencyViolation(
                file_path=str(file_path),
                line_number=line_num,
                violation_type="CUSTOM_DATETIME_FORMAT",
                description="Custom datetime format used",
                expected="Use DateTimeFormats from shared_libs.standards",
                found=match.group(),
                severity="warning"
            ))

        return violations

    def validate_imports(
        self,
        file_path: Path,
        analyzer: PythonCodeAnalyzer
    ) -> List[ConsistencyViolation]:
        """
        Validate that shared standards are imported when needed.

        Args:
            file_path: Path to file
            analyzer: AST analyzer

        Returns:
            List of violations
        """
        violations = []

        # This is checked in other methods, but we could add more general checks here
        # For now, this is a placeholder for future import validation

        return violations

    def validate_all_components(self) -> Dict[str, List[ConsistencyViolation]]:
        """
        Validate all components in the project.

        Returns:
            Dict mapping component names to their violations
        """
        results = {}

        if not self.components_dir.exists():
            return results

        for component_dir in self.components_dir.iterdir():
            if component_dir.is_dir() and not component_dir.name.startswith('.'):
                violations = self.validate_component(component_dir)
                if violations:
                    results[component_dir.name] = violations

        return results

    def generate_report(
        self,
        violations_by_component: Dict[str, List[ConsistencyViolation]]
    ) -> str:
        """
        Generate formatted report.

        Args:
            violations_by_component: Dict mapping component names to violations

        Returns:
            Formatted report string
        """
        if not violations_by_component:
            return "✅ No consistency violations found!"

        report = []
        report.append("=" * 80)
        report.append("CONSISTENCY VALIDATION REPORT")
        report.append("=" * 80)
        report.append("")

        total_violations = sum(len(v) for v in violations_by_component.values())
        total_errors = sum(
            sum(1 for viol in v if viol.severity == "error")
            for v in violations_by_component.values()
        )
        total_warnings = total_violations - total_errors

        report.append(f"Total Components: {len(violations_by_component)}")
        report.append(f"Total Violations: {total_violations}")
        report.append(f"  Errors: {total_errors}")
        report.append(f"  Warnings: {total_warnings}")
        report.append("")

        for component, violations in sorted(violations_by_component.items()):
            report.append("-" * 80)
            report.append(f"Component: {component}")
            report.append(f"Violations: {len(violations)}")
            report.append("-" * 80)
            report.append("")

            # Group by violation type
            by_type: Dict[str, List[ConsistencyViolation]] = {}
            for violation in violations:
                if violation.violation_type not in by_type:
                    by_type[violation.violation_type] = []
                by_type[violation.violation_type].append(violation)

            for viol_type, viols in sorted(by_type.items()):
                report.append(f"{viol_type}: {len(viols)} occurrences")
                for violation in viols[:5]:  # Show first 5 of each type
                    report.append(f"  {violation.file_path}:{violation.line_number}")
                    report.append(f"    {violation.description}")
                if len(viols) > 5:
                    report.append(f"  ... and {len(viols) - 5} more")
                report.append("")

        report.append("=" * 80)

        return "\n".join(report)

    def generate_json_report(
        self,
        violations_by_component: Dict[str, List[ConsistencyViolation]]
    ) -> str:
        """
        Generate JSON report.

        Args:
            violations_by_component: Dict mapping component names to violations

        Returns:
            JSON string
        """
        report = {
            "summary": {
                "total_components": len(violations_by_component),
                "total_violations": sum(len(v) for v in violations_by_component.values()),
                "total_errors": sum(
                    sum(1 for viol in v if viol.severity == "error")
                    for v in violations_by_component.values()
                ),
                "total_warnings": sum(
                    sum(1 for viol in v if viol.severity == "warning")
                    for v in violations_by_component.values()
                )
            },
            "components": {}
        }

        for component, violations in violations_by_component.items():
            report["components"][component] = [
                asdict(v) for v in violations
            ]

        return json.dumps(report, indent=2)


def main():
    """CLI interface."""
    parser = argparse.ArgumentParser(
        description="Validate component compliance with shared standards"
    )
    parser.add_argument(
        "--component",
        help="Component name to validate"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Validate all components"
    )
    parser.add_argument(
        "--report",
        help="Output report to file (JSON format)"
    )
    parser.add_argument(
        "--project-root",
        default=".",
        help="Project root directory (default: current directory)"
    )

    args = parser.parse_args()

    # Determine project root
    project_root = Path(args.project_root).resolve()
    validator = ConsistencyValidator(project_root)

    # Validate
    if args.all:
        violations_by_component = validator.validate_all_components()
    elif args.component:
        component_path = project_root / "components" / args.component
        if not component_path.exists():
            print(f"Error: Component '{args.component}' not found", file=sys.stderr)
            sys.exit(1)
        violations = validator.validate_component(component_path)
        violations_by_component = {args.component: violations} if violations else {}
    else:
        print("Error: Must specify --component or --all", file=sys.stderr)
        parser.print_help()
        sys.exit(1)

    # Generate report
    if args.report:
        report_content = validator.generate_json_report(violations_by_component)
        with open(args.report, 'w') as f:
            f.write(report_content)
        print(f"Report written to {args.report}")
    else:
        report_content = validator.generate_report(violations_by_component)
        print(report_content)

    # Exit with error code if violations found
    total_errors = sum(
        sum(1 for v in violations if v.severity == "error")
        for violations in violations_by_component.values()
    )
    sys.exit(1 if total_errors > 0 else 0)


if __name__ == '__main__':
    main()
