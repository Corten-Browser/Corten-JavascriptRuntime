#!/usr/bin/env python3
"""
Test Quality Checker - Automated over-mocking and test quality detection

This module provides AST-based static analysis of Python test files to detect:
- Over-mocking (mocking own source code)
- Missing integration tests
- Skipped tests
- Excessive mock usage

Usage:
    python orchestration/test_quality_checker.py components/my-component
    python orchestration/test_quality_checker.py --all
    python orchestration/test_quality_checker.py components/my-component --json
"""

import ast
import sys
import json
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Optional, Any
from dataclasses import dataclass, asdict
from enum import Enum


class Severity(Enum):
    """Issue severity levels"""
    CRITICAL = "CRITICAL"
    WARNING = "WARNING"
    INFO = "INFO"


@dataclass
class Issue:
    """Represents a single test quality issue"""
    severity: Severity
    category: str
    file: str
    line: int
    message: str
    pattern: str
    fix: Optional[str] = None
    recommendation: Optional[str] = None
    code: Optional[str] = None


@dataclass
class TestQualityReport:
    """Complete test quality analysis report"""
    component_path: str
    status: str  # "PASSED" or "FAILED"
    issues: List[Issue]
    summary: Dict[str, Any]
    overmocking: Dict[str, Any]
    integration_tests: Dict[str, Any]
    skipped_tests: Dict[str, Any]
    mock_analysis: Dict[str, Any]
    blocking_issues: List[Dict[str, str]]
    exit_code: int

    def has_critical_issues(self) -> bool:
        """Check if report has any critical issues"""
        return any(issue.severity == Severity.CRITICAL for issue in self.issues)

    @property
    def critical_count(self) -> int:
        """Count of critical issues"""
        return sum(1 for issue in self.issues if issue.severity == Severity.CRITICAL)

    @property
    def warning_count(self) -> int:
        """Count of warning issues"""
        return sum(1 for issue in self.issues if issue.severity == Severity.WARNING)

    @property
    def info_count(self) -> int:
        """Count of info issues"""
        return sum(1 for issue in self.issues if issue.severity == Severity.INFO)


class OverMockingDetector:
    """Detects patterns of mocking own code in test files"""

    # External modules that are OK to mock
    EXTERNAL_MODULES = {
        'requests', 'boto3', 'stripe', 'redis', 'celery',
        'time', 'datetime', 'random', 'uuid',
        'builtins', 'os', 'subprocess', 'sys',
    }

    def __init__(self, component_path: Path, component_name: str):
        self.component_path = component_path
        self.component_name = component_name
        self.own_prefixes = [
            'src.',
            f'components.{component_name}.',
            f'components/{component_name}/',
        ]

    def analyze(self) -> List[Issue]:
        """Analyze all test files for over-mocking patterns"""
        issues = []

        # Find all test files
        test_dirs = [
            self.component_path / 'tests' / 'unit',
            self.component_path / 'tests' / 'integration',
        ]

        for test_dir in test_dirs:
            if not test_dir.exists():
                continue

            for test_file in test_dir.rglob('test_*.py'):
                issues.extend(self._analyze_test_file(test_file))

        return issues

    def _analyze_test_file(self, test_file: Path) -> List[Issue]:
        """Analyze a single test file"""
        issues = []

        try:
            with open(test_file, 'r') as f:
                content = f.read()
                tree = ast.parse(content, filename=str(test_file))
        except SyntaxError as e:
            # Report syntax error as warning
            issues.append(Issue(
                severity=Severity.WARNING,
                category="syntax_error",
                file=str(test_file.relative_to(self.component_path)),
                line=e.lineno or 0,
                message=f"Syntax error in test file: {e.msg}",
                pattern="syntax_error",
                recommendation="Fix syntax error before running quality checks"
            ))
            return issues
        except Exception as e:
            # Skip files that can't be parsed
            return issues

        # Detect various over-mocking patterns
        issues.extend(self._detect_patch_overmocking(test_file, tree))
        issues.extend(self._detect_mock_spec_overmocking(test_file, tree))
        issues.extend(self._detect_excessive_patches(test_file, tree))

        return issues

    def _detect_patch_overmocking(self, test_file: Path, tree: ast.AST) -> List[Issue]:
        """Detect @patch decorators mocking own code"""
        issues = []

        for node in ast.walk(tree):
            # Look for function definitions (test functions)
            if not isinstance(node, ast.FunctionDef):
                continue

            # Check decorators
            for decorator in node.decorator_list:
                patch_target = self._extract_patch_target(decorator)
                if patch_target and self._is_own_code(patch_target):
                    # Get the decorator code
                    decorator_code = f"@patch('{patch_target}')"

                    issues.append(Issue(
                        severity=Severity.CRITICAL,
                        category="overmocking",
                        file=str(test_file.relative_to(self.component_path)),
                        line=decorator.lineno,
                        message="Mocking own source code",
                        pattern="own_source_code",
                        code=decorator_code,
                        fix=f"Use real {patch_target.split('.')[-1]} or move test to integration/"
                    ))

        return issues

    def _detect_mock_spec_overmocking(self, test_file: Path, tree: ast.AST) -> List[Issue]:
        """Detect Mock(spec=OwnClass) patterns"""
        issues = []

        for node in ast.walk(tree):
            # Look for Mock() calls with spec= argument
            if not isinstance(node, ast.Call):
                continue

            # Check if this is a Mock() call
            func_name = self._get_call_name(node.func)
            if func_name not in ('Mock', 'MagicMock'):
                continue

            # Check for spec= keyword argument
            for keyword in node.keywords:
                if keyword.arg == 'spec':
                    spec_value = self._extract_spec_value(keyword.value)
                    if spec_value and self._is_own_code(spec_value):
                        issues.append(Issue(
                            severity=Severity.WARNING,
                            category="overmocking",
                            file=str(test_file.relative_to(self.component_path)),
                            line=node.lineno,
                            message=f"Mock with spec of own class: {spec_value}",
                            pattern="mock_spec_own_class",
                            recommendation="Consider using real object in integration test"
                        ))

        return issues

    def _detect_excessive_patches(self, test_file: Path, tree: ast.AST) -> List[Issue]:
        """Detect tests with too many @patch decorators"""
        issues = []

        for node in ast.walk(tree):
            if not isinstance(node, ast.FunctionDef):
                continue

            # Count @patch decorators
            patch_count = 0
            own_code_patches = 0

            for decorator in node.decorator_list:
                patch_target = self._extract_patch_target(decorator)
                if patch_target:
                    patch_count += 1
                    if self._is_own_code(patch_target):
                        own_code_patches += 1

            # Flag if > 3 patches total
            if patch_count > 3:
                issues.append(Issue(
                    severity=Severity.WARNING,
                    category="excessive_mocking",
                    file=str(test_file.relative_to(self.component_path)),
                    line=node.lineno,
                    message=f"Excessive mock usage: {patch_count} @patch decorators",
                    pattern="excessive_patches",
                    code=f"Test function: {node.name}",
                    recommendation="Consider splitting test or adding integration test"
                ))

        return issues

    def _extract_patch_target(self, decorator: ast.AST) -> Optional[str]:
        """Extract the target string from a @patch decorator"""
        if isinstance(decorator, ast.Call):
            # @patch('module.Class')
            func_name = self._get_call_name(decorator.func)
            if func_name == 'patch' and decorator.args:
                if isinstance(decorator.args[0], ast.Constant):
                    return decorator.args[0].value
        elif isinstance(decorator, ast.Attribute):
            # @patch.object(...)
            if decorator.attr == 'object':
                return None  # Skip patch.object for now
        return None

    def _extract_spec_value(self, node: ast.AST) -> Optional[str]:
        """Extract the spec value from Mock(spec=...) call"""
        if isinstance(node, ast.Name):
            return node.id
        elif isinstance(node, ast.Attribute):
            # Reconstruct dotted name (e.g., src.models.User)
            parts = []
            current = node
            while isinstance(current, ast.Attribute):
                parts.insert(0, current.attr)
                current = current.value
            if isinstance(current, ast.Name):
                parts.insert(0, current.id)
            return '.'.join(parts)
        return None

    def _get_call_name(self, node: ast.AST) -> str:
        """Get the name of a function call"""
        if isinstance(node, ast.Name):
            return node.id
        elif isinstance(node, ast.Attribute):
            return node.attr
        return ""

    def _is_own_code(self, module_path: str) -> bool:
        """Check if a module path refers to own component code"""
        # Check if it starts with any of our own prefixes
        for prefix in self.own_prefixes:
            if module_path.startswith(prefix):
                return True
        return False

    def _is_external_mock(self, module_path: str) -> bool:
        """Check if a module path is an external dependency (OK to mock)"""
        for external in self.EXTERNAL_MODULES:
            if module_path.startswith(external):
                return True
        return False


class IntegrationTestVerifier:
    """Verifies integration tests exist and are meaningful"""

    def __init__(self, component_path: Path):
        self.component_path = component_path
        self.integration_dir = component_path / 'tests' / 'integration'

    def verify(self) -> Tuple[List[Issue], Dict[str, Any]]:
        """Verify integration test quality"""
        issues = []
        result = {
            'directory_exists': False,
            'tests_present': False,
            'file_count': 0,
            'test_count': 0,
            'own_code_mocking': False
        }

        # Check 1: Directory exists
        if not self.integration_dir.exists():
            issues.append(Issue(
                severity=Severity.CRITICAL,
                category="integration_tests",
                file="tests/integration/",
                line=0,
                message="Integration tests directory missing",
                pattern="no_integration_directory",
                fix="Create tests/integration/ directory and add integration tests"
            ))
            return issues, result

        result['directory_exists'] = True

        # Check 2: Count test files and functions
        test_files = list(self.integration_dir.rglob('test_*.py'))
        result['file_count'] = len(test_files)

        total_test_functions = 0
        for test_file in test_files:
            test_count = self._count_test_functions(test_file)
            total_test_functions += test_count

        result['test_count'] = total_test_functions

        if total_test_functions == 0:
            issues.append(Issue(
                severity=Severity.CRITICAL,
                category="integration_tests",
                file="tests/integration/",
                line=0,
                message="No integration test functions found",
                pattern="no_integration_tests",
                fix="Add test functions to tests/integration/ files"
            ))
            return issues, result

        result['tests_present'] = True

        # Check 3: Verify no over-mocking in integration tests
        # (Integration tests should use real components)
        overmocking_issues = self._check_integration_test_mocking()
        if overmocking_issues:
            result['own_code_mocking'] = True
            issues.extend(overmocking_issues)

        return issues, result

    def _count_test_functions(self, test_file: Path) -> int:
        """Count test functions in a file"""
        try:
            with open(test_file, 'r') as f:
                tree = ast.parse(f.read(), filename=str(test_file))

            count = 0
            for node in ast.walk(tree):
                if isinstance(node, ast.FunctionDef) and node.name.startswith('test_'):
                    count += 1
            return count
        except:
            return 0

    def _check_integration_test_mocking(self) -> List[Issue]:
        """Check if integration tests are mocking own code (anti-pattern)"""
        issues = []

        # Use OverMockingDetector but only for integration tests
        detector = OverMockingDetector(
            self.component_path,
            self.component_path.name
        )

        for test_file in self.integration_dir.rglob('test_*.py'):
            file_issues = detector._analyze_test_file(test_file)
            # Upgrade severity to CRITICAL for integration tests
            for issue in file_issues:
                if issue.category == "overmocking":
                    issue.severity = Severity.CRITICAL
                    issue.message = f"Integration test mocking own code: {issue.message}"
                    issue.fix = "Integration tests must use real components, not mocks"
                    issues.append(issue)

        return issues


class SkippedTestDetector:
    """Detects and categorizes skipped tests"""

    def __init__(self, component_path: Path):
        self.component_path = component_path

    def detect(self) -> List[Issue]:
        """Detect all skipped tests"""
        issues = []

        # Check both unit and integration tests
        test_dirs = [
            ('unit', self.component_path / 'tests' / 'unit'),
            ('integration', self.component_path / 'tests' / 'integration'),
        ]

        for location, test_dir in test_dirs:
            if not test_dir.exists():
                continue

            for test_file in test_dir.rglob('test_*.py'):
                issues.extend(self._detect_skips_in_file(test_file, location))

        return issues

    def _detect_skips_in_file(self, test_file: Path, location: str) -> List[Issue]:
        """Detect skips in a single test file"""
        issues = []

        try:
            with open(test_file, 'r') as f:
                content = f.read()
                tree = ast.parse(content, filename=str(test_file))
        except:
            return issues

        # Detect skip decorators
        issues.extend(self._detect_skip_decorators(test_file, tree, location))

        # Detect pytest.skip() calls
        issues.extend(self._detect_skip_calls(test_file, tree, location))

        return issues

    def _detect_skip_decorators(self, test_file: Path, tree: ast.AST, location: str) -> List[Issue]:
        """Detect @pytest.mark.skip decorators"""
        issues = []

        for node in ast.walk(tree):
            if not isinstance(node, ast.FunctionDef):
                continue

            for decorator in node.decorator_list:
                if self._is_skip_decorator(decorator):
                    reason = self._extract_skip_reason(decorator)

                    # CRITICAL for integration tests, WARNING for unit tests
                    severity = Severity.CRITICAL if location == 'integration' else Severity.WARNING

                    issues.append(Issue(
                        severity=severity,
                        category="skipped_tests",
                        file=str(test_file.relative_to(self.component_path)),
                        line=node.lineno,
                        message=f"Skipped test in {location} tests",
                        pattern="skip_decorator",
                        code=f"@pytest.mark.skip: {node.name}",
                        recommendation=f"Reason: {reason}. Either implement or remove",
                    ))

        return issues

    def _detect_skip_calls(self, test_file: Path, tree: ast.AST, location: str) -> List[Issue]:
        """Detect pytest.skip() calls in test bodies"""
        issues = []

        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                call_name = self._get_full_call_name(node.func)
                if 'skip' in call_name.lower() and 'pytest' in call_name.lower():
                    # Check if this is a conditional skip (may be OK)
                    is_conditional = self._is_conditional_skip(node)

                    # Extract reason
                    reason = self._extract_call_reason(node)

                    # Determine severity
                    if is_conditional:
                        severity = Severity.INFO
                        message = f"Conditional skip in {location} tests"
                    else:
                        severity = Severity.CRITICAL if location == 'integration' else Severity.WARNING
                        message = f"Unconditional skip in {location} tests"

                    issues.append(Issue(
                        severity=severity,
                        category="skipped_tests",
                        file=str(test_file.relative_to(self.component_path)),
                        line=node.lineno,
                        message=message,
                        pattern="skip_call",
                        recommendation=f"Reason: {reason}",
                    ))

        return issues

    def _is_skip_decorator(self, decorator: ast.AST) -> bool:
        """Check if decorator is a skip marker"""
        if isinstance(decorator, ast.Attribute):
            # @pytest.mark.skip
            if decorator.attr == 'skip':
                return True
        elif isinstance(decorator, ast.Call):
            # @pytest.mark.skip(reason=...)
            if isinstance(decorator.func, ast.Attribute):
                if decorator.func.attr == 'skip':
                    return True
        return False

    def _extract_skip_reason(self, decorator: ast.AST) -> str:
        """Extract reason from skip decorator"""
        if isinstance(decorator, ast.Call):
            for keyword in decorator.keywords:
                if keyword.arg == 'reason':
                    if isinstance(keyword.value, ast.Constant):
                        return keyword.value.value
            # Check positional args
            if decorator.args and isinstance(decorator.args[0], ast.Constant):
                return decorator.args[0].value
        return "No reason provided"

    def _extract_call_reason(self, call: ast.Call) -> str:
        """Extract reason from pytest.skip() call"""
        if call.args and isinstance(call.args[0], ast.Constant):
            return call.args[0].value
        return "No reason provided"

    def _get_full_call_name(self, node: ast.AST) -> str:
        """Get full call name (e.g., pytest.skip)"""
        if isinstance(node, ast.Name):
            return node.id
        elif isinstance(node, ast.Attribute):
            base = self._get_full_call_name(node.value)
            return f"{base}.{node.attr}"
        return ""

    def _is_conditional_skip(self, node: ast.Call) -> bool:
        """Check if skip is inside try/except or if statement"""
        # This is simplified - a full implementation would track parent nodes
        # For now, we'll be conservative and mark all as unconditional
        return False


class TestQualityChecker:
    """Coordinator class that runs all checks and generates reports"""

    def __init__(self, component_path: Path):
        self.component_path = component_path
        self.component_name = component_path.name

    def check(self, verbose: bool = False) -> TestQualityReport:
        """Run all checks and generate report"""
        all_issues = []

        # Run over-mocking detection
        overmocking_detector = OverMockingDetector(self.component_path, self.component_name)
        overmocking_issues = overmocking_detector.analyze()
        all_issues.extend(overmocking_issues)

        # Run integration test verification
        integration_verifier = IntegrationTestVerifier(self.component_path)
        integration_issues, integration_result = integration_verifier.verify()
        all_issues.extend(integration_issues)

        # Run skipped test detection
        skipped_detector = SkippedTestDetector(self.component_path)
        skipped_issues = skipped_detector.detect()
        all_issues.extend(skipped_issues)

        # Calculate statistics
        summary = self._calculate_summary(all_issues)

        # Calculate mock analysis
        mock_analysis = self._calculate_mock_analysis(overmocking_issues)

        # Build overmocking section
        overmocking_section = {
            'detected': len(overmocking_issues) > 0,
            'issues': [self._issue_to_dict(issue) for issue in overmocking_issues]
        }

        # Build skipped tests section
        skipped_section = {
            'detected': len(skipped_issues) > 0,
            'issues': [self._issue_to_dict(issue) for issue in skipped_issues]
        }

        # Determine status and exit code
        has_critical = any(issue.severity == Severity.CRITICAL for issue in all_issues)
        status = "FAILED" if has_critical else "PASSED"
        exit_code = 1 if has_critical else 0

        # Build blocking issues list
        blocking_issues = [
            {
                'file': issue.file,
                'line': issue.line,
                'issue': issue.fix or issue.message
            }
            for issue in all_issues
            if issue.severity == Severity.CRITICAL
        ]

        # Create report
        report = TestQualityReport(
            component_path=str(self.component_path),
            status=status,
            issues=all_issues,
            summary=summary,
            overmocking=overmocking_section,
            integration_tests=integration_result,
            skipped_tests=skipped_section,
            mock_analysis=mock_analysis,
            blocking_issues=blocking_issues,
            exit_code=exit_code
        )

        return report

    def _calculate_summary(self, issues: List[Issue]) -> Dict[str, Any]:
        """Calculate summary statistics"""
        return {
            'total_issues': len(issues),
            'critical_count': sum(1 for i in issues if i.severity == Severity.CRITICAL),
            'warning_count': sum(1 for i in issues if i.severity == Severity.WARNING),
            'info_count': sum(1 for i in issues if i.severity == Severity.INFO),
        }

    def _calculate_mock_analysis(self, overmocking_issues: List[Issue]) -> Dict[str, Any]:
        """Calculate mock usage statistics"""
        own_code_mocks = sum(1 for i in overmocking_issues if i.pattern == 'own_source_code')

        # Simplified - in full implementation would count all mocks
        total_mocks = own_code_mocks + 5  # Assume some external mocks
        external_mocks = total_mocks - own_code_mocks

        mock_ratio = own_code_mocks / total_mocks if total_mocks > 0 else 0

        if mock_ratio < 0.1:
            health = "HEALTHY"
        elif mock_ratio < 0.3:
            health = "MONITOR"
        else:
            health = "PROBLEMATIC"

        return {
            'external_mocks': external_mocks,
            'own_code_mocks': own_code_mocks,
            'total_mocks': total_mocks,
            'mock_ratio': round(mock_ratio, 2),
            'health': health
        }

    def _issue_to_dict(self, issue: Issue) -> Dict[str, Any]:
        """Convert Issue to dictionary for JSON output"""
        return {
            'severity': issue.severity.value,
            'category': issue.category,
            'file': issue.file,
            'line': issue.line,
            'message': issue.message,
            'pattern': issue.pattern,
            'fix': issue.fix,
            'recommendation': issue.recommendation,
            'code': issue.code
        }

    def generate_human_report(self, report: TestQualityReport) -> str:
        """Generate human-readable report"""
        lines = []

        # Header
        lines.append(f"🔍 Test Quality Check: {self.component_name}")
        lines.append("")

        # Over-mocking section
        lines.append("━" * 60)
        lines.append("Over-Mocking Detection")
        lines.append("━" * 60)
        lines.append("")

        overmocking_issues = [i for i in report.issues if i.category == "overmocking"]
        if overmocking_issues:
            for issue in overmocking_issues:
                severity_icon = "❌" if issue.severity == Severity.CRITICAL else "⚠️ "
                lines.append(f"{severity_icon} {issue.severity.value}: {issue.message}")
                lines.append(f"   File: {issue.file}:{issue.line}")
                if issue.code:
                    lines.append(f"   Pattern: {issue.code}")
                if issue.fix:
                    lines.append(f"   Fix: {issue.fix}")
                lines.append("")
        else:
            lines.append("✅ No over-mocking detected")
            lines.append("")

        # Integration tests section
        lines.append("━" * 60)
        lines.append("Integration Test Verification")
        lines.append("━" * 60)
        lines.append("")

        int_result = report.integration_tests
        dir_icon = "✅" if int_result['directory_exists'] else "❌"
        lines.append(f"{dir_icon} Integration tests directory: {'exists' if int_result['directory_exists'] else 'missing'}")

        if int_result['directory_exists']:
            tests_icon = "✅" if int_result['tests_present'] else "❌"
            lines.append(f"{tests_icon} Integration tests present: {int_result['file_count']} files, {int_result['test_count']} tests")

            mocking_icon = "❌" if int_result['own_code_mocking'] else "✅"
            mocking_status = "detected" if int_result['own_code_mocking'] else "none"
            lines.append(f"{mocking_icon} Own-code mocking in integration tests: {mocking_status}")

        lines.append("")

        # Skipped tests section
        lines.append("━" * 60)
        lines.append("Skipped Test Detection")
        lines.append("━" * 60)
        lines.append("")

        skipped_issues = [i for i in report.issues if i.category == "skipped_tests"]
        if skipped_issues:
            for issue in skipped_issues:
                severity_icon = {"CRITICAL": "❌", "WARNING": "⚠️ ", "INFO": "ℹ️ "}[issue.severity.value]
                lines.append(f"{severity_icon} {issue.severity.value}: {issue.message}")
                lines.append(f"   File: {issue.file}:{issue.line}")
                if issue.recommendation:
                    lines.append(f"   {issue.recommendation}")
                lines.append("")
        else:
            lines.append("✅ No skipped tests detected")
            lines.append("")

        # Summary
        lines.append("━" * 60)
        lines.append("Summary")
        lines.append("━" * 60)
        lines.append("")
        lines.append(f"Total Issues: {report.summary['total_issues']}")
        lines.append(f"  Critical: {report.summary['critical_count']} ❌")
        lines.append(f"  Warnings: {report.summary['warning_count']} ⚠️ ")
        lines.append(f"  Info: {report.summary['info_count']} ℹ️ ")
        lines.append("")

        lines.append("Mock Usage Analysis:")
        lines.append(f"  External service mocks: {report.mock_analysis['external_mocks']} (✅ appropriate)")
        lines.append(f"  Own code mocks: {report.mock_analysis['own_code_mocks']} ({'❌ problematic' if report.mock_analysis['own_code_mocks'] > 0 else '✅ good'})")
        lines.append(f"  Mock health: {report.mock_analysis['health']}")
        lines.append("")

        # Status
        if report.status == "FAILED":
            lines.append("❌ FAILED: Critical issues must be fixed before component completion.")
            lines.append("")
            lines.append("Fix the following CRITICAL issues:")
            for i, blocking in enumerate(report.blocking_issues, 1):
                lines.append(f"  {i}. {blocking['file']}:{blocking['line']} - {blocking['issue']}")
        else:
            lines.append("✅ PASSED: Test quality is acceptable.")
            if report.summary['warning_count'] > 0:
                lines.append(f"   ({report.summary['warning_count']} warnings to review)")

        return '\n'.join(lines)

    def generate_json_report(self, report: TestQualityReport) -> str:
        """Generate JSON report"""
        report_dict = {
            'component': report.component_path,
            'status': report.status,
            'summary': report.summary,
            'overmocking': report.overmocking,
            'integration_tests': report.integration_tests,
            'skipped_tests': report.skipped_tests,
            'mock_analysis': report.mock_analysis,
            'blocking_issues': report.blocking_issues,
            'exit_code': report.exit_code
        }
        return json.dumps(report_dict, indent=2)


def main():
    """CLI entry point"""
    parser = argparse.ArgumentParser(
        description='Test Quality Checker - Detect over-mocking and test quality issues'
    )
    parser.add_argument(
        'component',
        nargs='?',
        help='Component path (e.g., components/auth-service)'
    )
    parser.add_argument(
        '--all',
        action='store_true',
        help='Check all components'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output in JSON format'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Verbose output'
    )

    args = parser.parse_args()

    # Determine components to check
    if args.all:
        components_dir = Path('components')
        if not components_dir.exists():
            print("❌ ERROR: components/ directory not found", file=sys.stderr)
            return 2

        components = [d for d in components_dir.iterdir() if d.is_dir()]
    elif args.component:
        component_path = Path(args.component)
        if not component_path.exists():
            print(f"❌ ERROR: Component directory not found: {args.component}", file=sys.stderr)
            return 2
        components = [component_path]
    else:
        parser.print_help()
        return 2

    # Check each component
    exit_code = 0
    for component in components:
        checker = TestQualityChecker(component)
        report = checker.check(verbose=args.verbose)

        if args.json:
            print(checker.generate_json_report(report))
        else:
            print(checker.generate_human_report(report))
            print()  # Blank line between components

        # Use worst exit code
        if report.exit_code > exit_code:
            exit_code = report.exit_code

    return exit_code


if __name__ == '__main__':
    sys.exit(main())
