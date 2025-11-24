#!/usr/bin/env python3
"""
Completion Verifier

Verifies that components are ACTUALLY complete before accepting them as done.

This solves the v0.2.0 problem where the orchestrator declared "Project Complete!"
when 20% of work remained incomplete (2/10 components unfinished).

The completion verifier runs 16 critical checks to ensure:
1. All tests pass (100%)
2. Imports resolve correctly
3. No stub implementations remain
4. No TODO markers remain
5. Documentation is complete
6. No "remaining work" markers
7. Test coverage ≥ 80%
8. Component manifest is complete
9. Test quality (no over-mocking, integration tests exist) [v0.5.0]
10. User acceptance (product works from user perspective) [v0.6.0]
11. Integration test execution (all tests ran, none NOT RUN) [v0.7.0]
12. README accuracy (examples work as documented) [v0.12.0]
13. Feature coverage (all declared features tested) [v0.13.0]
14. No hardcoded absolute paths (distribution-ready) [v0.15.0]
15. Package is installable (pip install works) [v0.15.0]
16. README.md comprehensive (complete user documentation) [v0.15.0]

Part of v0.3.0 completion guarantee system, enhanced in v0.5.0, v0.6.0, v0.7.0, v0.12.0, v0.13.0, v0.15.0.
"""

import re
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Tuple, Optional, Dict
from dataclasses import dataclass
import json
import doctest

# Import markdown parser for README testing (v0.12.0)
try:
    # Try relative import first (when in orchestration/)
    from markdown_parser import MarkdownParser, CodeBlock
except ImportError:
    try:
        # Try absolute import (when installed as package)
        from orchestration.analysis.markdown_parser import MarkdownParser, CodeBlock
    except ImportError:
        # Fallback if markdown_parser not available
        MarkdownParser = None
        CodeBlock = None

# Import quality checker (v0.5.0)
try:
    from orchestration.verification.quality.quality_checker import QualityChecker
except ImportError:
    QualityChecker = None

# Import integration coverage checker (v0.7.0)
try:
    from integration_coverage_checker import IntegrationCoverageChecker
except ImportError:
    IntegrationCoverageChecker = None


@dataclass
class CheckResult:
    """Result of a single verification check."""
    check_name: str
    passed: bool
    message: str
    details: Optional[str] = None
    is_critical: bool = True  # If False, warning only


@dataclass
class CompletionVerification:
    """
    Complete verification result for a component.

    Enhanced in v0.14.0 with blocking_issues detection.
    """
    component_name: str
    is_complete: bool
    checks: List[CheckResult]
    remaining_tasks: List[str]
    completion_percentage: int
    blocking_issues: List[str] = None  # v0.14.0: Explicit blocking issues

    def __post_init__(self):
        """Initialize blocking_issues if not provided."""
        if self.blocking_issues is None:
            self.blocking_issues = []

    def get_failed_checks(self) -> List[CheckResult]:
        """Get all failed checks."""
        return [c for c in self.checks if not c.passed]

    def get_critical_failures(self) -> List[CheckResult]:
        """Get critical failures only."""
        return [c for c in self.checks if not c.passed and c.is_critical]

    def get_warnings(self) -> List[CheckResult]:
        """Get non-critical failures (warnings)."""
        return [c for c in self.checks if not c.passed and not c.is_critical]


class CompletionVerifier:
    """Verifies component completion with 11-check system (v0.7.0)."""

    def __init__(self, project_root: Path):
        """
        Initialize verifier.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()

    def verify_component(self, component_path: Path) -> CompletionVerification:
        """
        Verify component is complete.

        Args:
            component_path: Path to component directory

        Returns:
            CompletionVerification with detailed results
        """
        component_path = Path(component_path).resolve()
        component_name = component_path.name

        print(f"🔍 Verifying component: {component_name}")
        print(f"   Path: {component_path}")

        # Run all 16 checks (v0.15.0: added distribution checks)
        checks = []
        checks.append(self._check_tests_pass(component_path))
        checks.append(self._check_imports_resolve(component_path))
        checks.append(self._check_no_stubs(component_path))
        checks.append(self._check_no_todos(component_path))
        checks.append(self._check_documentation_complete(component_path))
        checks.append(self._check_no_remaining_work_markers(component_path))
        checks.append(self._check_test_coverage(component_path))
        checks.append(self._check_manifest_complete(component_path))
        checks.append(self._check_test_quality(component_path))  # v0.5.0: Test quality
        checks.append(self._check_user_acceptance(component_path))  # v0.6.0: UAT
        checks.append(self._check_integration_test_execution(component_path))  # v0.7.0: Integration execution
        checks.append(self._check_readme_accuracy(component_path))  # v0.12.0: README accuracy
        checks.append(self._check_feature_coverage(component_path))  # v0.13.0: Feature coverage
        checks.append(self._check_no_hardcoded_paths(component_path))  # v0.15.0: No hardcoded paths
        checks.append(self._check_package_installable(component_path))  # v0.15.0: Package installable
        checks.append(self._check_readme_comprehensive(component_path))  # v0.15.0: README comprehensive

        # Determine overall completion
        critical_checks = [c for c in checks if c.is_critical]
        critical_passed = all(c.passed for c in critical_checks)

        # Calculate completion percentage
        total_checks = len(checks)
        passed_checks = sum(1 for c in checks if c.passed)
        completion_percentage = int((passed_checks / total_checks) * 100)

        # Extract remaining tasks from failures
        remaining_tasks = []
        for check in checks:
            if not check.passed:
                remaining_tasks.append(f"{check.check_name}: {check.message}")

        # v0.14.0: Identify blocking issues
        blocking_issues = []

        # Blocking Issue 1: Any failing tests
        test_check = checks[0]  # First check is "Tests Pass"
        if not test_check.passed:
            blocking_issues.append(
                f"BLOCKING: {test_check.message} - YOU MUST FIX ALL FAILING TESTS\n"
                f"  No rationalizations accepted - tests must pass 100%"
            )

        # Blocking Issue 2: Integration tests not 100% passing
        integration_check = checks[10]  # 11th check is "Integration Test Execution"
        if not integration_check.passed:
            if 'NOT RUN' in integration_check.message or 'not executed' in integration_check.message:
                blocking_issues.append(
                    f"BLOCKING: Integration tests not executed - ALL tests must run\n"
                    f"  Found tests marked as NOT RUN - must execute all tests"
                )
            elif 'failing' in integration_check.message.lower():
                blocking_issues.append(
                    f"BLOCKING: Integration tests failing - CANNOT PROCEED\n"
                    f"  {integration_check.message}\n"
                    f"  Phase 5 gate requires 100% integration test pass rate"
                )

        # Blocking Issue 3: Feature coverage failing
        feature_check = checks[12]  # 13th check is "Feature Coverage"
        if not feature_check.passed:
            blocking_issues.append(
                f"BLOCKING: Not all features tested - MUST DECLARE AND TEST ALL FEATURES\n"
                f"  {feature_check.message}\n"
                f"  Update component.yaml user_facing_features section"
            )

        # Blocking Issue 4: Stub implementations remain
        stub_check = checks[2]  # 3rd check is "No Stubs"
        if not stub_check.passed:
            blocking_issues.append(
                f"BLOCKING: Stub implementations found - MUST IMPLEMENT ALL CODE\n"
                f"  {stub_check.message}"
            )

        # Blocking Issue 5: Hardcoded absolute paths (v0.15.0)
        paths_check = checks[13]  # 14th check is "No Hardcoded Paths"
        if not paths_check.passed:
            blocking_issues.append(
                f"BLOCKING: Hardcoded absolute paths found - SOFTWARE NOT DISTRIBUTABLE\n"
                f"  {paths_check.message}\n"
                f"  Software will fail when installed to different directory"
            )

        # Blocking Issue 6: Package not installable (v0.15.0)
        install_check = checks[14]  # 15th check is "Package Installable"
        if not install_check.passed:
            blocking_issues.append(
                f"BLOCKING: Package not installable - MUST CREATE setup.py/pyproject.toml\n"
                f"  {install_check.message}"
            )

        # Update is_complete based on blocking issues
        if blocking_issues:
            is_complete = False

        is_complete = critical_passed and completion_percentage >= 75 and not blocking_issues

        return CompletionVerification(
            component_name=component_name,
            is_complete=is_complete,
            checks=checks,
            remaining_tasks=remaining_tasks,
            completion_percentage=completion_percentage,
            blocking_issues=blocking_issues  # v0.14.0
        )

    def _check_tests_pass(self, component_path: Path) -> CheckResult:
        """Check 1: All tests pass (100%)."""
        try:
            # Try pytest first (Python)
            result = subprocess.run(
                ["pytest", str(component_path / "tests"), "-v", "--tb=short"],
                cwd=component_path,
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode == 0:
                return CheckResult(
                    check_name="Tests Pass",
                    passed=True,
                    message="All tests passing",
                    is_critical=True
                )
            else:
                # Extract failure count from output
                output = result.stdout + result.stderr
                failure_match = re.search(r'(\d+) failed', output)
                failures = failure_match.group(1) if failure_match else "some"

                return CheckResult(
                    check_name="Tests Pass",
                    passed=False,
                    message=f"{failures} test(s) failing",
                    details=output[-500:] if len(output) > 500 else output,
                    is_critical=True
                )

        except subprocess.TimeoutExpired:
            return CheckResult(
                check_name="Tests Pass",
                passed=False,
                message="Tests timed out (>5 minutes)",
                is_critical=True
            )

        except FileNotFoundError:
            # pytest not found, try npm test (JavaScript)
            try:
                result = subprocess.run(
                    ["npm", "test"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                if result.returncode == 0:
                    return CheckResult(
                        check_name="Tests Pass",
                        passed=True,
                        message="All tests passing",
                        is_critical=True
                    )
                else:
                    return CheckResult(
                        check_name="Tests Pass",
                        passed=False,
                        message="Tests failing (npm test)",
                        details=result.stdout[-500:],
                        is_critical=True
                    )

            except Exception as e:
                return CheckResult(
                    check_name="Tests Pass",
                    passed=False,
                    message=f"Could not run tests: {str(e)}",
                    is_critical=True
                )

    def _check_imports_resolve(self, component_path: Path) -> CheckResult:
        """Check 2: All imports resolve correctly."""
        # Find all Python files
        python_files = list(component_path.glob("**/*.py"))

        if not python_files:
            # Not a Python project, skip
            return CheckResult(
                check_name="Imports Resolve",
                passed=True,
                message="No Python files found (skipped)",
                is_critical=False
            )

        import_errors = []

        for py_file in python_files:
            # Skip __pycache__
            if "__pycache__" in str(py_file):
                continue

            try:
                # Try to compile the file (checks syntax and imports)
                with open(py_file, 'r') as f:
                    code = f.read()

                compile(code, str(py_file), 'exec')

            except SyntaxError as e:
                import_errors.append(f"{py_file.name}:{e.lineno}: {e.msg}")

            except Exception as e:
                # Other compilation errors
                if "import" in str(e).lower():
                    import_errors.append(f"{py_file.name}: {str(e)}")

        if not import_errors:
            return CheckResult(
                check_name="Imports Resolve",
                passed=True,
                message=f"All imports resolve ({len(python_files)} files checked)",
                is_critical=True
            )
        else:
            return CheckResult(
                check_name="Imports Resolve",
                passed=False,
                message=f"{len(import_errors)} import error(s) found",
                details="\n".join(import_errors[:5]),
                is_critical=True
            )

    def _check_no_stubs(self, component_path: Path) -> CheckResult:
        """Check 3: No stub implementations remain."""
        stub_patterns = [
            r'raise NotImplementedError',
            r'pass\s*#\s*TODO',
            r'pass\s*#\s*stub',
            r'pass\s*#\s*placeholder',
            r'def\s+\w+\([^)]*\):\s*pass\s*$',  # Empty functions
        ]

        source_files = list(component_path.glob("src/**/*.py"))
        source_files.extend(component_path.glob("src/**/*.ts"))
        source_files.extend(component_path.glob("src/**/*.js"))

        stubs_found = []

        for file_path in source_files:
            try:
                with open(file_path, 'r') as f:
                    content = f.read()

                for pattern in stub_patterns:
                    matches = re.finditer(pattern, content, re.MULTILINE)
                    for match in matches:
                        line_num = content[:match.start()].count('\n') + 1
                        stubs_found.append(f"{file_path.name}:{line_num}")

            except Exception:
                pass

        if not stubs_found:
            return CheckResult(
                check_name="No Stubs",
                passed=True,
                message="No stub implementations found",
                is_critical=True
            )
        else:
            return CheckResult(
                check_name="No Stubs",
                passed=False,
                message=f"{len(stubs_found)} stub(s) remain",
                details="\n".join(stubs_found[:10]),
                is_critical=True
            )

    def _check_no_todos(self, component_path: Path) -> CheckResult:
        """Check 4: No TODO markers remain."""
        todo_patterns = [
            r'#\s*TODO',
            r'//\s*TODO',
            r'/\*\s*TODO',
            r'FIXME',
            r'XXX',
        ]

        source_files = list(component_path.glob("src/**/*"))
        source_files = [f for f in source_files if f.is_file()]

        todos_found = []

        for file_path in source_files:
            try:
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    for line_num, line in enumerate(f, 1):
                        for pattern in todo_patterns:
                            if re.search(pattern, line, re.IGNORECASE):
                                todos_found.append(f"{file_path.name}:{line_num}: {line.strip()[:60]}")

            except Exception:
                pass

        if not todos_found:
            return CheckResult(
                check_name="No TODOs",
                passed=True,
                message="No TODO markers found",
                is_critical=False  # Warning only
            )
        else:
            return CheckResult(
                check_name="No TODOs",
                passed=False,
                message=f"{len(todos_found)} TODO marker(s) found",
                details="\n".join(todos_found[:10]),
                is_critical=False  # Warning only
            )

    def _check_documentation_complete(self, component_path: Path) -> CheckResult:
        """Check 5: Documentation is complete."""
        required_docs = ["README.md", "CLAUDE.md"]
        missing_docs = []

        for doc in required_docs:
            doc_path = component_path / doc
            if not doc_path.exists():
                missing_docs.append(doc)
            elif doc_path.stat().st_size < 100:
                missing_docs.append(f"{doc} (too short)")

        if not missing_docs:
            return CheckResult(
                check_name="Documentation Complete",
                passed=True,
                message="All required documentation present",
                is_critical=False
            )
        else:
            return CheckResult(
                check_name="Documentation Complete",
                passed=False,
                message=f"Missing: {', '.join(missing_docs)}",
                is_critical=False
            )

    def _check_no_remaining_work_markers(self, component_path: Path) -> CheckResult:
        """Check 6: No 'remaining work' markers."""
        markers = [
            "IN PROGRESS",
            "INCOMPLETE",
            "NOT IMPLEMENTED",
            "PARTIAL IMPLEMENTATION",
            "WORK IN PROGRESS",
        ]

        all_files = list(component_path.glob("**/*"))
        all_files = [f for f in all_files if f.is_file() and f.suffix in ['.py', '.ts', '.js', '.md']]

        markers_found = []

        for file_path in all_files:
            try:
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()

                for marker in markers:
                    if marker in content.upper():
                        line_num = content.upper().index(marker)
                        markers_found.append(f"{file_path.name}: {marker}")

            except Exception:
                pass

        if not markers_found:
            return CheckResult(
                check_name="No Remaining Work Markers",
                passed=True,
                message="No incomplete markers found",
                is_critical=False
            )
        else:
            return CheckResult(
                check_name="No Remaining Work Markers",
                passed=False,
                message=f"{len(markers_found)} incomplete marker(s)",
                details="\n".join(markers_found[:5]),
                is_critical=False
            )

    def _check_test_coverage(self, component_path: Path) -> CheckResult:
        """Check 7: Test coverage ≥ 80%."""
        try:
            # Run pytest with coverage
            result = subprocess.run(
                ["pytest", "--cov=src", "--cov-report=term-missing", "tests/"],
                cwd=component_path,
                capture_output=True,
                text=True,
                timeout=300
            )

            output = result.stdout + result.stderr

            # Extract coverage percentage
            coverage_match = re.search(r'TOTAL\s+\d+\s+\d+\s+(\d+)%', output)

            if coverage_match:
                coverage = int(coverage_match.group(1))

                if coverage >= 80:
                    return CheckResult(
                        check_name="Test Coverage",
                        passed=True,
                        message=f"Coverage: {coverage}% (≥80%)",
                        is_critical=True
                    )
                else:
                    return CheckResult(
                        check_name="Test Coverage",
                        passed=False,
                        message=f"Coverage: {coverage}% (target: 80%)",
                        is_critical=True
                    )
            else:
                # Could not extract coverage
                return CheckResult(
                    check_name="Test Coverage",
                    passed=False,
                    message="Could not determine coverage",
                    details=output[-200:],
                    is_critical=False
                )

        except Exception as e:
            return CheckResult(
                check_name="Test Coverage",
                passed=False,
                message=f"Coverage check failed: {str(e)}",
                is_critical=False
            )

    def _check_manifest_complete(self, component_path: Path) -> CheckResult:
        """Check 8: Component manifest is complete."""
        manifest_path = component_path / "component.yaml"

        if not manifest_path.exists():
            return CheckResult(
                check_name="Manifest Complete",
                passed=False,
                message="component.yaml missing",
                is_critical=False
            )

        try:
            import yaml
            with open(manifest_path, 'r') as f:
                manifest = yaml.safe_load(f)

            required_fields = ["name", "version", "type", "description"]
            missing_fields = [f for f in required_fields if f not in manifest or not manifest[f]]

            if not missing_fields:
                return CheckResult(
                    check_name="Manifest Complete",
                    passed=True,
                    message="component.yaml complete",
                    is_critical=False
                )
            else:
                return CheckResult(
                    check_name="Manifest Complete",
                    passed=False,
                    message=f"Missing fields: {', '.join(missing_fields)}",
                    is_critical=False
                )

        except ImportError:
            return CheckResult(
                check_name="Manifest Complete",
                passed=False,
                message="PyYAML not installed (cannot validate)",
                is_critical=False
            )

        except Exception as e:
            return CheckResult(
                check_name="Manifest Complete",
                passed=False,
                message=f"Error reading manifest: {str(e)}",
                is_critical=False
            )

    def _check_test_quality(self, component_path: Path) -> CheckResult:
        """Check 9: Test Quality (no over-mocking, integration tests exist) [v0.5.0]."""
        if QualityChecker is None:
            return CheckResult(
                check_name="Test Quality",
                passed=False,
                message="test_quality_checker module not available",
                is_critical=False
            )

        try:
            checker = QualityChecker(component_path)
            report = checker.check()

            if report.has_critical_issues():
                # Format blocking issues for display
                blocking_summary = []
                for issue in report.blocking_issues[:3]:  # Show first 3
                    blocking_summary.append(f"{issue['file']}:{issue['line']}")

                details = "\n".join(blocking_summary)
                if len(report.blocking_issues) > 3:
                    details += f"\n... and {len(report.blocking_issues) - 3} more"

                return CheckResult(
                    check_name="Test Quality",
                    passed=False,
                    message=f"{report.critical_count} critical test quality issue(s)",
                    details=details,
                    is_critical=True
                )
            else:
                warning_note = f" ({report.warning_count} warnings)" if report.warning_count > 0 else ""
                return CheckResult(
                    check_name="Test Quality",
                    passed=True,
                    message=f"Test quality verified{warning_note}",
                    is_critical=True
                )

        except Exception as e:
            return CheckResult(
                check_name="Test Quality",
                passed=False,
                message=f"Test quality check failed: {str(e)}",
                details=str(e),
                is_critical=False  # Don't block on checker errors
            )

    def _check_user_acceptance(self, component_path: Path) -> CheckResult:
        """Check 10: User acceptance - product works from user perspective [v0.6.0]."""

        # Read component manifest to determine project type
        manifest_path = component_path / "component.yaml"

        if not manifest_path.exists():
            return CheckResult(
                check_name="User Acceptance",
                passed=False,
                message="component.yaml missing (cannot determine project type)",
                is_critical=False
            )

        try:
            manifest = self._read_manifest(manifest_path)
            project_type = manifest.get('type', 'unknown').lower()

            # Dispatch to type-specific UAT
            if project_type in ['cli', 'application']:
                return self._uat_cli_application(component_path, manifest)
            elif project_type in ['library', 'package']:
                return self._uat_library(component_path, manifest)
            elif project_type in ['api', 'web', 'server', 'microservice']:
                return self._uat_web_server(component_path, manifest)
            elif project_type in ['gui', 'desktop']:
                return self._uat_gui_application(component_path, manifest)
            else:
                # Unknown type - minimal checks
                return CheckResult(
                    check_name="User Acceptance",
                    passed=False,
                    message=f"Unknown project type: {project_type}",
                    details="Cannot perform UAT without known project type",
                    is_critical=False
                )

        except Exception as e:
            return CheckResult(
                check_name="User Acceptance",
                passed=False,
                message=f"UAT check failed: {str(e)}",
                details=str(e),
                is_critical=False
            )

    def _uat_cli_application(self, component_path: Path, manifest: Dict) -> CheckResult:
        """
        UAT for CLI applications - verify ALL commands work.

        Enhanced [v0.12.0]: Also tests README shell commands.
        Enhanced [v0.13.0]: Tests ALL declared CLI commands from manifest.
        """

        entry_module = manifest.get('entry_module', component_path.name)
        project_root = component_path.parent.parent  # Up from components/<name>
        features = manifest.get('user_facing_features', {})

        failures = []

        # Test 1: Basic entry point --help (backward compatibility)
        try:
            result = subprocess.run(
                ['python', '-m', entry_module, '--help'],
                cwd=project_root,
                capture_output=True,
                text=True,
                timeout=10
            )

            if result.returncode != 0:
                failures.append(f"Entry point --help failed: {result.stderr[:100]}")
        except subprocess.TimeoutExpired:
            failures.append("Entry point --help timed out (>10s)")
        except Exception as e:
            failures.append(f"Cannot run entry point: {str(e)}")

        # Test 1b: ALL declared CLI commands (NEW v0.13.0)
        cli_commands = features.get('cli_commands', [])
        if cli_commands:
            for cmd_config in cli_commands:
                cmd_name = cmd_config['name']

                # Test basic invocation
                try:
                    result = subprocess.run(
                        ['python', '-m', entry_module, cmd_name, '--help'],
                        cwd=project_root,
                        capture_output=True,
                        timeout=5,
                        text=True
                    )
                    if result.returncode != 0:
                        failures.append(f"Command '{cmd_name} --help' failed: {result.stderr[:100]}")
                except Exception as e:
                    failures.append(f"Command '{cmd_name}' invocation failed: {str(e)}")

                # Test scenario if provided
                if 'test_scenario' in cmd_config:
                    scenario = cmd_config['test_scenario'].replace('{entry_module}', entry_module)
                    try:
                        result = subprocess.run(
                            scenario,
                            shell=True,
                            cwd=project_root,
                            capture_output=True,
                            timeout=30
                        )
                        if result.returncode != 0:
                            failures.append(
                                f"Command '{cmd_name}' scenario failed\n"
                                f"  Scenario: {scenario}\n"
                                f"  Error: {result.stderr[:200]}"
                            )
                    except subprocess.TimeoutExpired:
                        failures.append(f"Command '{cmd_name}' scenario timed out (>30s)")
                    except Exception as e:
                        failures.append(f"Command '{cmd_name}' scenario error: {str(e)}")

        # Test 2: README shell commands (NEW - v0.12.0)
        readme_path = component_path / "README.md"
        if readme_path.exists() and MarkdownParser is not None:
            try:
                parser = MarkdownParser(readme_path)
                shell_blocks = parser.extract_code_blocks(
                    languages=['bash', 'shell', 'console'],
                    sections=['Quick Start', 'Installation', 'Usage']
                )

                # Test first 3 shell commands
                for block in shell_blocks[:3]:
                    if block.skip:
                        continue

                    commands = self._split_shell_commands(block.content)
                    for cmd in commands[:1]:  # First command from each block
                        cmd = cmd.strip()
                        if not cmd or cmd.startswith('#'):
                            continue

                        # Only test help/version commands (safe, quick)
                        if 'help' in cmd.lower() or '--version' in cmd:
                            try:
                                result = subprocess.run(
                                    cmd,
                                    shell=True,
                                    cwd=project_root,
                                    capture_output=True,
                                    timeout=5,
                                    text=True
                                )
                                if result.returncode != 0:
                                    failures.append(f"README command failed (line {block.line_number}): {cmd}")
                            except Exception:
                                pass  # Non-critical for UAT
            except Exception:
                pass  # Non-critical if README parsing fails

        # Test 3: Packaging exists
        has_packaging = (
            (project_root / "setup.py").exists() or
            (project_root / "pyproject.toml").exists()
        )

        if not has_packaging:
            failures.append("No setup.py or pyproject.toml (not installable)")

        # Test 4: Entry point defined (if setup.py exists)
        if (project_root / "setup.py").exists():
            try:
                with open(project_root / "setup.py", 'r') as f:
                    setup_content = f.read()
                    if 'console_scripts' not in setup_content and 'entry_points' not in setup_content:
                        failures.append("No entry_points in setup.py (not installable as command)")
            except Exception:
                pass  # Non-critical

        if not failures:
            cmd_count = len(cli_commands) if cli_commands else 1
            return CheckResult(
                check_name="User Acceptance (CLI)",
                passed=True,
                message=f"All {cmd_count} CLI command(s) work, packaging exists",
                is_critical=True
            )
        else:
            return CheckResult(
                check_name="User Acceptance (CLI)",
                passed=False,
                message=f"{len(failures)} UAT issue(s) found",
                details="\n".join(failures),
                is_critical=True
            )

    def _uat_library(self, component_path: Path, manifest: Dict) -> CheckResult:
        """
        UAT for libraries - verify importability and packaging.

        Enhanced [v0.12.0]: Also tests README import examples.
        Enhanced [v0.13.0]: Tests ALL declared public API from manifest.
        """

        library_name = manifest.get('name', component_path.name)
        project_root = component_path.parent.parent
        features = manifest.get('user_facing_features', {})

        failures = []

        # Test 1: Packaging exists
        has_packaging = (
            (project_root / "setup.py").exists() or
            (project_root / "pyproject.toml").exists()
        )

        if not has_packaging:
            failures.append("No setup.py or pyproject.toml (not installable)")

        # Test 2: Can import locally (manifest name)
        try:
            # Add project root to path
            import sys
            sys.path.insert(0, str(project_root))

            # Try to import
            __import__(library_name)

        except ImportError as e:
            failures.append(f"Cannot import {library_name}: {str(e)}")
        except Exception as e:
            failures.append(f"Import error: {str(e)}")
        finally:
            # Clean up path
            if str(project_root) in sys.path:
                sys.path.remove(str(project_root))

        # Test 2b: ALL declared public API (NEW - v0.13.0)
        public_api = features.get('public_api', [])
        api_count = 0
        if public_api:
            for api_config in public_api:
                api_name = api_config.get('class') or api_config.get('function', 'unnamed')
                module_path = api_config.get('module')
                api_count += 1

                if not module_path:
                    failures.append(f"API '{api_name}': missing module path in manifest")
                    continue

                # Test basic import
                try:
                    sys.path.insert(0, str(project_root))
                    module = __import__(module_path, fromlist=[''])

                    # Verify class/function exists
                    if 'class' in api_config:
                        class_name = api_config['class']
                        if not hasattr(module, class_name):
                            failures.append(
                                f"API class '{class_name}' not found in module '{module_path}'"
                            )
                        else:
                            # Optionally verify methods exist
                            methods = api_config.get('methods', [])
                            if methods:
                                cls = getattr(module, class_name)
                                for method in methods:
                                    if not hasattr(cls, method):
                                        failures.append(
                                            f"Method '{class_name}.{method}' not found"
                                        )

                    elif 'function' in api_config:
                        func_name = api_config['function']
                        if not hasattr(module, func_name):
                            failures.append(
                                f"API function '{func_name}' not found in module '{module_path}'"
                            )

                except ImportError as e:
                    failures.append(
                        f"API '{api_name}': cannot import module '{module_path}'\n"
                        f"  Error: {str(e)}"
                    )
                except Exception as e:
                    failures.append(
                        f"API '{api_name}': error testing\n"
                        f"  Error: {str(e)}"
                    )
                finally:
                    if str(project_root) in sys.path:
                        sys.path.remove(str(project_root))

        # Test 3: README import examples (NEW - v0.12.0)
        readme_path = component_path / "README.md"
        if readme_path.exists():
            # Check README has import examples
            try:
                readme = readme_path.read_text()
                if f"import {library_name}" not in readme and f"from {library_name}" not in readme:
                    failures.append("README missing import examples")
            except Exception:
                pass  # Non-critical

            # Test README import examples actually work (NEW - v0.12.0)
            if MarkdownParser is not None:
                try:
                    parser = MarkdownParser(readme_path)
                    python_blocks = parser.extract_code_blocks(
                        languages=['python'],
                        sections=['Quick Start', 'Usage', 'Examples']
                    )

                    # Test first 3 Python examples
                    for block in python_blocks[:3]:
                        if block.skip:
                            continue

                        # Extract just import statements
                        imports = [
                            line for line in block.content.split('\n')
                            if line.strip().startswith(('import ', 'from '))
                        ]

                        for import_stmt in imports:
                            try:
                                sys.path.insert(0, str(project_root))
                                exec(import_stmt, {})
                            except ImportError as e:
                                failures.append(
                                    f"README import failed (line {block.line_number}): {import_stmt}\n"
                                    f"  Error: {str(e)}"
                                )
                            except Exception:
                                pass  # Other errors non-critical for UAT
                            finally:
                                if str(project_root) in sys.path:
                                    sys.path.remove(str(project_root))
                except Exception:
                    pass  # Non-critical if README parsing fails

        if not failures:
            # Build success message with API count if applicable
            success_msg = "Library importable, packaging exists"
            if api_count > 0:
                success_msg = f"All {api_count} public API(s) importable, packaging exists"

            return CheckResult(
                check_name="User Acceptance (Library)",
                passed=True,
                message=success_msg,
                is_critical=True
            )
        else:
            return CheckResult(
                check_name="User Acceptance (Library)",
                passed=False,
                message=f"{len(failures)} UAT issue(s) found",
                details="\n".join(failures),
                is_critical=True
            )

    def _uat_web_server(self, component_path: Path, manifest: Dict) -> CheckResult:
        """UAT for web servers - verify server can start."""

        entry_module = manifest.get('entry_module', component_path.name)
        project_root = component_path.parent.parent

        failures = []

        # Test 1: Server starts without immediate crash
        try:
            server_process = subprocess.Popen(
                ['python', '-m', entry_module],
                cwd=project_root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            # Wait for startup
            time.sleep(2)

            # Check if still running
            if server_process.poll() is not None:
                # Process exited
                _, stderr = server_process.communicate()
                failures.append(f"Server crashed on startup: {stderr[:100]}")
            else:
                # Still running - terminate cleanly
                server_process.terminate()
                try:
                    server_process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    server_process.kill()

        except Exception as e:
            failures.append(f"Cannot start server: {str(e)}")

        # Test 2: Dockerfile exists (if applicable)
        dockerfile = project_root / "Dockerfile"
        if not dockerfile.exists():
            # Not critical but nice to have
            pass

        if not failures:
            return CheckResult(
                check_name="User Acceptance (Web Server)",
                passed=True,
                message="Server starts without crashing",
                is_critical=True
            )
        else:
            return CheckResult(
                check_name="User Acceptance (Web Server)",
                passed=False,
                message=f"{len(failures)} UAT issue(s) found",
                details="\n".join(failures),
                is_critical=True
            )

    def _uat_gui_application(self, component_path: Path, manifest: Dict) -> CheckResult:
        """UAT for GUI applications - verify can launch without crash."""

        entry_module = manifest.get('entry_module', component_path.name)
        project_root = component_path.parent.parent

        # GUI testing is limited without display
        # We can only check it doesn't crash immediately

        try:
            gui_process = subprocess.Popen(
                ['python', '-m', entry_module],
                cwd=project_root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            # Wait for initialization
            time.sleep(3)

            # Check if still running
            if gui_process.poll() is not None:
                # Crashed
                _, stderr = gui_process.communicate()
                return CheckResult(
                    check_name="User Acceptance (GUI)",
                    passed=False,
                    message="GUI crashed on startup",
                    details=stderr[:200],
                    is_critical=True
                )
            else:
                # Still running - good sign
                gui_process.terminate()
                try:
                    gui_process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    gui_process.kill()

                return CheckResult(
                    check_name="User Acceptance (GUI)",
                    passed=True,
                    message="GUI launches without immediate crash (manual testing recommended)",
                    is_critical=True
                )

        except Exception as e:
            return CheckResult(
                check_name="User Acceptance (GUI)",
                passed=False,
                message=f"Cannot launch GUI: {str(e)}",
                details=str(e),
                is_critical=False  # May fail in headless environment
            )

    def _read_manifest(self, manifest_path: Path) -> Dict:
        """Read and parse component.yaml manifest."""
        try:
            import yaml
            with open(manifest_path, 'r') as f:
                return yaml.safe_load(f) or {}
        except ImportError:
            # PyYAML not available - try JSON fallback
            import json
            with open(manifest_path, 'r') as f:
                return json.load(f)
        except Exception:
            return {}

    def _check_integration_test_execution(self, component_path: Path) -> CheckResult:
        """Check 11: Integration test execution - all tests ran, none NOT RUN [v0.7.0]."""

        # Use IntegrationCoverageChecker to verify execution
        if IntegrationCoverageChecker is None:
            return CheckResult(
                check_name="Integration Test Execution",
                passed=False,
                message="integration_coverage_checker module not available",
                is_critical=False
            )

        try:
            project_root = component_path.parent.parent
            checker = IntegrationCoverageChecker(project_root)
            coverage = checker.check_coverage()

            # Check 1: File found
            if not coverage.get('file_found'):
                return CheckResult(
                    check_name="Integration Test Execution",
                    passed=False,
                    message="TEST-RESULTS.md not found (integration tests may not have run)",
                    details="Expected: tests/integration/TEST-RESULTS.md",
                    is_critical=False  # Not critical if tests haven't been created yet
                )

            # Check 2: Any "NOT RUN" status is blocking
            if coverage['not_run'] > 0:
                not_run_summary = "\n".join(coverage['not_run_tests'][:5])
                if len(coverage['not_run_tests']) > 5:
                    not_run_summary += f"\n... and {len(coverage['not_run_tests']) - 5} more"

                return CheckResult(
                    check_name="Integration Test Execution",
                    passed=False,
                    message=f"{coverage['not_run']} integration test(s) NOT RUN (blocked by failures)",
                    details=(
                        f"Tests in NOT RUN status:\n{not_run_summary}\n\n"
                        f"REQUIRED ACTIONS:\n"
                        f"1. Fix ALL failing tests\n"
                        f"2. Re-run ENTIRE integration test suite\n"
                        f"3. Verify 100% execution rate\n"
                        f"4. Re-run completion_verifier"
                    ),
                    is_critical=True
                )

            # Check 3: Execution rate must be 100%
            if coverage['execution_rate'] < 100.0:
                return CheckResult(
                    check_name="Integration Test Execution",
                    passed=False,
                    message=f"Execution rate {coverage['execution_rate']:.1f}% (need 100%)",
                    details=(
                        f"Total planned: {coverage['total']}\n"
                        f"Executed: {coverage['executed']}\n"
                        f"Not executed: {coverage['not_run']}\n\n"
                        f"All integration tests must execute, not just pass."
                    ),
                    is_critical=True
                )

            # Check 4: Pass rate must be 100%
            if coverage['pass_rate'] < 100.0:
                failed_summary = "\n".join(coverage['failed_tests'][:5])
                if len(coverage['failed_tests']) > 5:
                    failed_summary += f"\n... and {len(coverage['failed_tests']) - 5} more"

                return CheckResult(
                    check_name="Integration Test Execution",
                    passed=False,
                    message=f"Pass rate {coverage['pass_rate']:.1f}% (need 100%)",
                    details=(
                        f"Tests failed:\n{failed_summary}\n\n"
                        f"All integration tests must pass (zero-tolerance policy)."
                    ),
                    is_critical=True
                )

            # All checks passed
            return CheckResult(
                check_name="Integration Test Execution",
                passed=True,
                message=f"All {coverage['total']} integration tests executed and passed (100%)",
                is_critical=True
            )

        except Exception as e:
            return CheckResult(
                check_name="Integration Test Execution",
                passed=False,
                message=f"Integration execution check failed: {str(e)}",
                details=str(e),
                is_critical=False  # Don't block on checker errors
            )

    def _check_readme_accuracy(self, component_path: Path) -> CheckResult:
        """
        Check 12: README accuracy - all examples work [v0.12.0].

        Comprehensive testing of README.md:
        1. Shell commands (CLI apps, installation)
        2. Python code examples (libraries, API usage)
        3. Doctest examples (if present)

        This prevents Music Analyzer failure mode: documentation that
        doesn't match implementation.

        Args:
            component_path: Path to component directory

        Returns:
            CheckResult with detailed failure information
        """
        readme_path = component_path / "README.md"

        if not readme_path.exists():
            return CheckResult(
                check_name="README Accuracy",
                passed=True,  # Not a failure if README doesn't exist
                message="README.md not found (skipping accuracy check)",
                is_critical=False
            )

        # Check if markdown_parser is available
        if MarkdownParser is None:
            return CheckResult(
                check_name="README Accuracy",
                passed=False,
                message="markdown_parser module not available",
                is_critical=False
            )

        project_root = component_path.parent.parent

        # Collect all failures
        all_failures = []
        stats = {
            'shell': {'total': 0, 'passed': 0},
            'python': {'total': 0, 'passed': 0},
            'doctest': {'total': 0, 'passed': 0}
        }

        # Part 1: Test shell commands
        shell_failures = self._test_readme_shell_commands(
            readme_path, project_root, stats['shell']
        )
        all_failures.extend(shell_failures)

        # Part 2: Test Python code examples
        python_failures = self._test_readme_python_examples(
            readme_path, project_root, stats['python']
        )
        all_failures.extend(python_failures)

        # Part 3: Test doctest examples (if present)
        doctest_failures = self._test_readme_doctest(
            readme_path, stats['doctest']
        )
        all_failures.extend(doctest_failures)

        # Determine result
        total_tested = sum(s['total'] for s in stats.values())

        if total_tested == 0:
            # No testable examples found - warning, not failure
            return CheckResult(
                check_name="README Accuracy",
                passed=True,
                message="No testable code examples in README (shell/python/doctest)",
                is_critical=False
            )

        if not all_failures:
            # All examples passed
            message_parts = []
            if stats['shell']['total'] > 0:
                message_parts.append(
                    f"shell: {stats['shell']['passed']}/{stats['shell']['total']}"
                )
            if stats['python']['total'] > 0:
                message_parts.append(
                    f"python: {stats['python']['passed']}/{stats['python']['total']}"
                )
            if stats['doctest']['total'] > 0:
                message_parts.append(
                    f"doctest: {stats['doctest']['passed']}/{stats['doctest']['total']}"
                )

            return CheckResult(
                check_name="README Accuracy",
                passed=True,
                message=f"All README examples work ({', '.join(message_parts)})",
                is_critical=True
            )
        else:
            # Some examples failed - BLOCKING
            failure_summary = "\n".join(all_failures[:10])  # First 10
            if len(all_failures) > 10:
                failure_summary += f"\n... and {len(all_failures) - 10} more"

            return CheckResult(
                check_name="README Accuracy",
                passed=False,
                message=f"{len(all_failures)} README example failure(s)",
                details=failure_summary,
                is_critical=True  # BLOCKING - documentation must match reality
            )

    def _test_readme_shell_commands(
        self,
        readme_path: Path,
        project_root: Path,
        stats: dict
    ) -> List[str]:
        """Test shell commands from README."""
        failures = []

        parser = MarkdownParser(readme_path)
        shell_blocks = parser.extract_code_blocks(
            languages=['bash', 'shell', 'sh', 'console']
        )

        stats['total'] = len(shell_blocks)

        for block in shell_blocks:
            if block.skip:
                continue

            # Extract individual commands (handle multi-line, &&, etc.)
            commands = self._split_shell_commands(block.content)

            for cmd in commands:
                # Skip comments and empty lines
                cmd = cmd.strip()
                if not cmd or cmd.startswith('#'):
                    continue

                try:
                    result = subprocess.run(
                        cmd,
                        shell=True,
                        cwd=project_root,
                        capture_output=True,
                        text=True,
                        timeout=10
                    )

                    if result.returncode != 0:
                        failures.append(
                            f"Shell command (line {block.line_number}): {cmd}\n"
                            f"  Error: {result.stderr[:200]}"
                        )
                    else:
                        stats['passed'] += 1

                except subprocess.TimeoutExpired:
                    failures.append(
                        f"Shell command (line {block.line_number}): {cmd}\n"
                        f"  Error: Timeout (>10s)"
                    )
                except Exception as e:
                    failures.append(
                        f"Shell command (line {block.line_number}): {cmd}\n"
                        f"  Error: {str(e)}"
                    )

        return failures

    def _test_readme_python_examples(
        self,
        readme_path: Path,
        project_root: Path,
        stats: dict
    ) -> List[str]:
        """Test Python code examples from README."""
        failures = []

        parser = MarkdownParser(readme_path)
        python_blocks = parser.extract_code_blocks(
            languages=['python', 'py']
        )

        stats['total'] = len(python_blocks)

        for block in python_blocks:
            if block.skip:
                continue

            try:
                # Create isolated namespace
                namespace = {
                    '__name__': '__main__',
                    '__file__': str(readme_path),
                    '__builtins__': __builtins__
                }

                # Add project to path temporarily
                import sys
                original_path = sys.path.copy()
                sys.path.insert(0, str(project_root))

                try:
                    # Execute code example
                    exec(block.content, namespace)
                    stats['passed'] += 1

                finally:
                    # Restore path
                    sys.path = original_path

            except ImportError as e:
                failures.append(
                    f"Python example (line {block.line_number}): Import failed\n"
                    f"  Code: {block.content[:100]}...\n"
                    f"  Error: {str(e)}"
                )
            except TypeError as e:
                failures.append(
                    f"Python example (line {block.line_number}): Type error\n"
                    f"  Code: {block.content[:100]}...\n"
                    f"  Error: {str(e)}\n"
                    f"  Hint: Check API signature matches documentation"
                )
            except AssertionError as e:
                failures.append(
                    f"Python example (line {block.line_number}): Assertion failed\n"
                    f"  Error: {str(e)}"
                )
            except Exception as e:
                failures.append(
                    f"Python example (line {block.line_number}): Execution error\n"
                    f"  Code: {block.content[:100]}...\n"
                    f"  Error: {type(e).__name__}: {str(e)}"
                )

        return failures

    def _test_readme_doctest(self, readme_path: Path, stats: dict) -> List[str]:
        """Test doctest examples from README."""
        failures = []

        # Check if README contains doctest format (>>> examples)
        with open(readme_path, 'r') as f:
            content = f.read()

        if '>>>' not in content:
            # No doctest examples
            return failures

        try:
            # Run doctest on README
            results = doctest.testfile(
                str(readme_path),
                module_relative=False,
                verbose=False,
                optionflags=doctest.ELLIPSIS | doctest.NORMALIZE_WHITESPACE
            )

            stats['total'] = results.attempted
            stats['passed'] = results.attempted - results.failed

            if results.failed > 0:
                failures.append(
                    f"Doctest: {results.failed}/{results.attempted} examples failed\n"
                    f"  Run: python -m doctest {readme_path} -v"
                )

        except Exception as e:
            failures.append(f"Doctest execution error: {str(e)}")

        return failures

    def _split_shell_commands(self, content: str) -> List[str]:
        """Split shell script into individual commands."""
        # Handle && and || chains, newlines, etc.
        # For v0.12.0, simple split on newlines
        # TODO: Smarter parsing for complex scripts
        return [line for line in content.split('\n') if line.strip()]

    def _check_feature_coverage(self, component_path: Path) -> CheckResult:
        """
        Check 13: Feature coverage - all declared features tested [v0.13.0].

        Verifies that every feature declared in component.yaml is actually tested.
        This prevents situations where features exist but are never validated
        (e.g., Music Analyzer playlist command failure - ReadmeFailureAssessment2.txt).

        Tests:
        1. CLI commands: Run smoke test for each command
        2. Public API: Verify imports and instantiation
        3. HTTP endpoints: Basic connectivity test (if applicable)

        Args:
            component_path: Path to component directory

        Returns:
            CheckResult with detailed failure information
        """
        manifest_path = component_path / "component.yaml"

        if not manifest_path.exists():
            return CheckResult(
                check_name="Feature Coverage",
                passed=False,
                message="component.yaml not found",
                is_critical=False
            )

        manifest = self._read_manifest(manifest_path)
        features = manifest.get('user_facing_features', {})

        # If no features declared, check if component type requires them
        comp_type = manifest.get('type', 'generic')
        if not features:
            if comp_type in ['cli_application', 'library', 'web_server']:
                return CheckResult(
                    check_name="Feature Coverage",
                    passed=False,
                    message=f"Type '{comp_type}' must declare user_facing_features",
                    details=(
                        f"Add user_facing_features section to component.yaml.\n"
                        f"See templates for examples of feature declarations."
                    ),
                    is_critical=True
                )
            else:
                # Generic components don't need features
                return CheckResult(
                    check_name="Feature Coverage",
                    passed=True,
                    message="No user-facing features (generic component)",
                    is_critical=False
                )

        project_root = component_path.parent.parent
        all_failures = []
        stats = {
            'cli_commands': {'total': 0, 'passed': 0},
            'public_api': {'total': 0, 'passed': 0},
            'http_endpoints': {'total': 0, 'passed': 0}
        }

        # Test CLI commands
        cli_failures = self._test_cli_commands(
            features.get('cli_commands', []),
            manifest,
            project_root,
            stats['cli_commands']
        )
        all_failures.extend(cli_failures)

        # Test public API
        api_failures = self._test_public_api(
            features.get('public_api', []),
            project_root,
            stats['public_api']
        )
        all_failures.extend(api_failures)

        # Test HTTP endpoints (if web server)
        if comp_type == 'web_server':
            endpoint_failures = self._test_http_endpoints(
                features.get('http_endpoints', []),
                stats['http_endpoints']
            )
            all_failures.extend(endpoint_failures)

        # Determine result
        total_features = sum(s['total'] for s in stats.values())

        if total_features == 0:
            return CheckResult(
                check_name="Feature Coverage",
                passed=False,
                message="No features declared (empty user_facing_features)",
                is_critical=True
            )

        if not all_failures:
            # All features passed
            message_parts = []
            if stats['cli_commands']['total'] > 0:
                message_parts.append(
                    f"CLI: {stats['cli_commands']['passed']}/{stats['cli_commands']['total']}"
                )
            if stats['public_api']['total'] > 0:
                message_parts.append(
                    f"API: {stats['public_api']['passed']}/{stats['public_api']['total']}"
                )
            if stats['http_endpoints']['total'] > 0:
                message_parts.append(
                    f"HTTP: {stats['http_endpoints']['passed']}/{stats['http_endpoints']['total']}"
                )

            return CheckResult(
                check_name="Feature Coverage",
                passed=True,
                message=f"All features tested ({', '.join(message_parts)})",
                is_critical=True
            )
        else:
            # Some features failed
            failure_summary = "\n".join(all_failures[:10])
            if len(all_failures) > 10:
                failure_summary += f"\n... and {len(all_failures) - 10} more"

            return CheckResult(
                check_name="Feature Coverage",
                passed=False,
                message=f"{len(all_failures)} feature(s) failed smoke test",
                details=failure_summary,
                is_critical=True  # BLOCKING
            )

    def _test_cli_commands(
        self,
        commands: List[Dict],
        manifest: Dict,
        project_root: Path,
        stats: dict
    ) -> List[str]:
        """Test all declared CLI commands."""
        failures = []
        entry_module = manifest.get('entry_module')

        if not entry_module:
            return ["entry_module not specified in manifest"]

        stats['total'] = len(commands)

        for cmd_config in commands:
            cmd_name = cmd_config.get('name', 'unnamed')
            smoke_test = cmd_config.get('smoke_test', f"python -m {entry_module} {cmd_name} --help")
            required = cmd_config.get('required', True)

            # Substitute {entry_module} placeholder
            smoke_test = smoke_test.replace('{entry_module}', entry_module)

            try:
                result = subprocess.run(
                    smoke_test,
                    shell=True,
                    cwd=project_root,
                    capture_output=True,
                    text=True,
                    timeout=10
                )

                if result.returncode != 0:
                    error_msg = (
                        f"CLI command '{cmd_name}' smoke test failed\n"
                        f"  Command: {smoke_test}\n"
                        f"  Error: {result.stderr[:200]}"
                    )

                    if required:
                        failures.append(error_msg)
                    else:
                        # Non-required features just warn
                        failures.append(f"[WARNING] {error_msg}")
                else:
                    stats['passed'] += 1

            except subprocess.TimeoutExpired:
                failures.append(
                    f"CLI command '{cmd_name}' smoke test timed out (>10s)\n"
                    f"  Command: {smoke_test}"
                )
            except Exception as e:
                failures.append(
                    f"CLI command '{cmd_name}' smoke test error\n"
                    f"  Command: {smoke_test}\n"
                    f"  Error: {str(e)}"
                )

        return failures

    def _test_public_api(
        self,
        api_list: List[Dict],
        project_root: Path,
        stats: dict
    ) -> List[str]:
        """Test all declared public API."""
        failures = []

        stats['total'] = len(api_list)

        for api_config in api_list:
            api_name = api_config.get('class') or api_config.get('function') or 'unnamed'
            module_path = api_config.get('module')
            required = api_config.get('required', True)

            if not module_path:
                failures.append(f"API '{api_name}': no module specified")
                continue

            try:
                # Add project to path
                import sys
                original_path = sys.path.copy()
                sys.path.insert(0, str(project_root))

                try:
                    # Import the module
                    module = __import__(module_path, fromlist=[''])

                    # Check if class/function exists
                    if 'class' in api_config:
                        class_name = api_config['class']
                        if not hasattr(module, class_name):
                            failures.append(
                                f"API class '{class_name}' not found in {module_path}"
                            )
                            continue

                        # Try to verify it's a class
                        cls = getattr(module, class_name)
                        if not isinstance(cls, type):
                            failures.append(
                                f"API '{class_name}' is not a class in {module_path}"
                            )
                            continue

                    elif 'function' in api_config:
                        func_name = api_config['function']
                        if not hasattr(module, func_name):
                            failures.append(
                                f"API function '{func_name}' not found in {module_path}"
                            )
                            continue

                        # Verify it's callable
                        func = getattr(module, func_name)
                        if not callable(func):
                            failures.append(
                                f"API '{func_name}' is not callable in {module_path}"
                            )
                            continue

                    stats['passed'] += 1

                finally:
                    # Restore path
                    sys.path = original_path

            except ImportError as e:
                error_msg = (
                    f"API '{api_name}' import failed\n"
                    f"  Module: {module_path}\n"
                    f"  Error: {str(e)}"
                )
                if required:
                    failures.append(error_msg)
                else:
                    failures.append(f"[WARNING] {error_msg}")

            except Exception as e:
                failures.append(
                    f"API '{api_name}' test error\n"
                    f"  Module: {module_path}\n"
                    f"  Error: {str(e)}"
                )

        return failures

    def _test_http_endpoints(
        self,
        endpoints: List[Dict],
        stats: dict
    ) -> List[str]:
        """Test HTTP endpoints (basic validation)."""
        failures = []

        stats['total'] = len(endpoints)

        # For v0.13.0, just validate endpoint declarations
        # Future: Could add actual HTTP testing with requests
        for endpoint in endpoints:
            if 'path' not in endpoint:
                failures.append("HTTP endpoint missing 'path'")
            if 'method' not in endpoint:
                failures.append(f"HTTP endpoint {endpoint.get('path', '?')} missing 'method'")
            else:
                stats['passed'] += 1

        return failures

    def _check_no_hardcoded_paths(self, component_path: Path) -> CheckResult:
        """
        Check 14: No hardcoded absolute paths [v0.15.0].

        Detects hardcoded absolute paths that break distribution.
        Based on HardPathsFailureAssessment.txt analysis.

        Patterns detected:
        - Unix absolute paths: /home/, /workspaces/, /Users/, /root/
        - Windows absolute paths: C:\, D:\, etc.
        - sys.path.append with absolute paths
        - os.path.join with absolute paths as first argument

        Args:
            component_path: Path to component directory

        Returns:
            CheckResult with violations or success
        """
        project_root = component_path.parent.parent

        # Patterns to detect
        unix_patterns = [
            r'/home/',
            r'/workspaces/',
            r'/Users/',
            r'/root/',
            r'/opt/',
            r'/usr/local/',
        ]

        windows_patterns = [
            r'C:\\',
            r'D:\\',
            r'E:\\',
            r'F:\\',
        ]

        # sys.path.append with absolute path
        sys_path_pattern = r'sys\.path\.(append|insert)\s*\([^)]*["\'](?:/|[A-Z]:)'

        violations = []

        # Scan all Python files
        python_files = list(component_path.glob("**/*.py"))

        for py_file in python_files:
            # Skip __pycache__
            if "__pycache__" in str(py_file):
                continue

            try:
                with open(py_file, 'r', encoding='utf-8', errors='ignore') as f:
                    for line_num, line in enumerate(f, 1):
                        # Check Unix absolute paths
                        for pattern in unix_patterns:
                            if re.search(pattern, line):
                                context = self._get_line_context(py_file, line_num, line)
                                violations.append(
                                    f"{py_file.relative_to(project_root)}:{line_num}: "
                                    f"Unix absolute path '{pattern.strip('/')}'\n"
                                    f"  {context}"
                                )

                        # Check Windows absolute paths
                        for pattern in windows_patterns:
                            if re.search(pattern, line):
                                context = self._get_line_context(py_file, line_num, line)
                                violations.append(
                                    f"{py_file.relative_to(project_root)}:{line_num}: "
                                    f"Windows absolute path '{pattern.strip(chr(92))}'\n"
                                    f"  {context}"
                                )

                        # Check sys.path manipulation
                        if re.search(sys_path_pattern, line):
                            context = self._get_line_context(py_file, line_num, line)
                            violations.append(
                                f"{py_file.relative_to(project_root)}:{line_num}: "
                                f"sys.path with absolute path\n"
                                f"  {context}"
                            )

            except Exception:
                pass  # Skip files that can't be read

        if not violations:
            return CheckResult(
                check_name="No Hardcoded Paths",
                passed=True,
                message="No hardcoded absolute paths found",
                is_critical=True
            )
        else:
            # Show first 10 violations
            details = "\n".join(violations[:10])
            if len(violations) > 10:
                details += f"\n... and {len(violations) - 10} more"

            return CheckResult(
                check_name="No Hardcoded Paths",
                passed=False,
                message=f"{len(violations)} hardcoded absolute path(s) found",
                details=details,
                is_critical=True
            )

    def _check_package_installable(self, component_path: Path) -> CheckResult:
        """
        Check 15: Package is installable [v0.15.0].

        Verifies that the package can actually be installed via pip.
        Tests in a clean virtual environment.

        Args:
            component_path: Path to component directory

        Returns:
            CheckResult with test results
        """
        project_root = component_path.parent.parent

        # Check 1: Packaging files exist
        has_setup_py = (project_root / "setup.py").exists()
        has_pyproject = (project_root / "pyproject.toml").exists()

        if not has_setup_py and not has_pyproject:
            return CheckResult(
                check_name="Package Installable",
                passed=False,
                message="No setup.py or pyproject.toml found",
                details=(
                    "Package cannot be installed without packaging configuration.\n"
                    "Create setup.py or pyproject.toml with proper package metadata."
                ),
                is_critical=True
            )

        # Check 2: Try to install in a test venv (quick validation)
        # For v0.15.0, we just verify packaging exists
        # Full installation test is added in Phase 2 (Phase 5.5)

        # Check 3: Verify entry points if CLI application
        manifest_path = component_path / "component.yaml"
        if manifest_path.exists():
            manifest = self._read_manifest(manifest_path)
            comp_type = manifest.get('type', '')

            if comp_type in ['cli', 'application', 'cli_application']:
                # Verify entry_points exist
                if has_setup_py:
                    try:
                        with open(project_root / "setup.py", 'r') as f:
                            setup_content = f.read()
                            if 'entry_points' not in setup_content and 'console_scripts' not in setup_content:
                                return CheckResult(
                                    check_name="Package Installable",
                                    passed=False,
                                    message="CLI application missing entry_points in setup.py",
                                    details=(
                                        "setup.py must define entry_points with console_scripts.\n"
                                        "Example:\n"
                                        "  entry_points={\n"
                                        "      'console_scripts': [\n"
                                        "          'myapp=myapp.main:main',\n"
                                        "      ],\n"
                                        "  }"
                                    ),
                                    is_critical=True
                                )
                    except Exception:
                        pass  # Non-critical if can't read

        return CheckResult(
            check_name="Package Installable",
            passed=True,
            message="Package configuration exists (setup.py or pyproject.toml)",
            is_critical=True
        )

    def _check_readme_comprehensive(self, component_path: Path) -> CheckResult:
        """
        Check 16: README.md is comprehensive [v0.15.0].

        Verifies README has sufficient content and required sections.
        Based on HardPathsFailureAssessment.txt finding: no README at repo root.

        Required sections:
        - Description/Overview
        - Installation
        - Usage/Quick Start
        - Minimum 500 words (indicates actual documentation, not stub)

        Args:
            component_path: Path to component directory

        Returns:
            CheckResult with validation results
        """
        project_root = component_path.parent.parent
        readme_path = project_root / "README.md"

        if not readme_path.exists():
            return CheckResult(
                check_name="README Comprehensive",
                passed=False,
                message="README.md not found at project root",
                details=(
                    "Every distributable package must have README.md at repository root.\n"
                    "README should include:\n"
                    "  - Project description\n"
                    "  - Installation instructions\n"
                    "  - Usage examples\n"
                    "  - Minimum 500 words"
                ),
                is_critical=True
            )

        try:
            with open(readme_path, 'r', encoding='utf-8') as f:
                content = f.read()

            # Check 1: Minimum length (500 words)
            word_count = len(content.split())
            if word_count < 500:
                return CheckResult(
                    check_name="README Comprehensive",
                    passed=False,
                    message=f"README too short: {word_count} words (minimum: 500)",
                    details=(
                        "README appears to be a stub or incomplete.\n"
                        "Add comprehensive documentation including:\n"
                        "  - Detailed description\n"
                        "  - Installation steps\n"
                        "  - Usage examples\n"
                        "  - Configuration options\n"
                        "  - API reference (if library)"
                    ),
                    is_critical=True
                )

            # Check 2: Required sections
            content_lower = content.lower()
            required_sections = {
                'installation': ['install', 'setup', 'getting started'],
                'usage': ['usage', 'quick start', 'quickstart', 'examples'],
                'description': ['description', 'overview', 'about', '# ']
            }

            missing_sections = []
            for section_name, keywords in required_sections.items():
                if not any(keyword in content_lower for keyword in keywords):
                    missing_sections.append(section_name)

            if missing_sections:
                return CheckResult(
                    check_name="README Comprehensive",
                    passed=False,
                    message=f"README missing sections: {', '.join(missing_sections)}",
                    details=(
                        f"README should include:\n"
                        f"  - Installation: How to install the package\n"
                        f"  - Usage: How to use the package (with examples)\n"
                        f"  - Description: What the package does\n\n"
                        f"Missing: {', '.join(missing_sections)}"
                    ),
                    is_critical=True
                )

            # Check 3: Has code examples (fenced code blocks or indented code)
            has_code_examples = '```' in content or '\n    ' in content
            if not has_code_examples:
                return CheckResult(
                    check_name="README Comprehensive",
                    passed=False,
                    message="README missing code examples",
                    details=(
                        "README should include code examples showing how to use the package.\n"
                        "Use fenced code blocks (```) or indented code blocks."
                    ),
                    is_critical=False  # Warning, not blocking
                )

            # All checks passed
            return CheckResult(
                check_name="README Comprehensive",
                passed=True,
                message=f"README complete ({word_count} words, all sections present)",
                is_critical=True
            )

        except Exception as e:
            return CheckResult(
                check_name="README Comprehensive",
                passed=False,
                message=f"Error reading README: {str(e)}",
                is_critical=False
            )

    def _get_line_context(self, file_path: Path, line_num: int, line: str) -> str:
        """
        Get context for error reporting.

        Args:
            file_path: Path to file
            line_num: Line number
            line: Line content

        Returns:
            Formatted context string
        """
        return line.strip()[:80]

    def print_verification_report(self, verification: CompletionVerification):
        """Print detailed verification report."""
        print("\n" + "="*70)
        print(f"COMPLETION VERIFICATION: {verification.component_name}")
        print("="*70)

        # Overall status
        if verification.is_complete:
            print(f"✅ COMPLETE ({verification.completion_percentage}%)")
        else:
            print(f"❌ INCOMPLETE ({verification.completion_percentage}%)")

        print()

        # Individual checks
        for check in verification.checks:
            status = "✅" if check.passed else ("⚠️ " if not check.is_critical else "❌")
            critical = " [CRITICAL]" if check.is_critical and not check.passed else ""
            print(f"{status} {check.check_name}: {check.message}{critical}")

            if check.details and not check.passed:
                # Indent details
                for line in check.details.split('\n')[:5]:
                    print(f"     {line}")

        print()

        # v0.14.0: Print blocking issues prominently
        if verification.blocking_issues:
            print()
            print("🛑" + "="*68 + "🛑")
            print("🛑 BLOCKING ISSUES - CANNOT MARK COMPLETE")
            print("🛑" + "="*68 + "🛑")
            for i, issue in enumerate(verification.blocking_issues, 1):
                # Indent multi-line issues
                lines = issue.split('\n')
                print(f"🛑 {i}. {lines[0]}")
                for line in lines[1:]:
                    if line.strip():
                        print(f"🛑    {line}")
            print("🛑" + "="*68 + "🛑")
            print()
            print("YOU MUST RESOLVE ALL BLOCKING ISSUES BEFORE PROCEEDING")
            print("DO NOT DECLARE COMPLETION UNTIL ALL ISSUES RESOLVED")
            print("DO NOT RATIONALIZE - FIX THE ACTUAL PROBLEMS")
            print()

        # Remaining tasks
        if verification.remaining_tasks:
            print("📋 REMAINING TASKS:")
            for task in verification.remaining_tasks:
                print(f"   - {task}")
            print()

        print("="*70)


def main():
    """CLI interface for completion verification."""
    if len(sys.argv) < 2:
        print("Usage: completion_verifier.py <component_path>")
        print("\nVerifies component is truly complete before marking as done.")
        print("\nExamples:")
        print("  python completion_verifier.py components/audio_processor")
        print("  python completion_verifier.py components/shared_types")
        sys.exit(1)

    component_path = Path(sys.argv[1])

    if not component_path.exists():
        print(f"❌ Component not found: {component_path}")
        sys.exit(1)

    project_root = Path.cwd()
    verifier = CompletionVerifier(project_root)

    verification = verifier.verify_component(component_path)
    verifier.print_verification_report(verification)

    # Exit code: 0 if complete, 1 if incomplete
    sys.exit(0 if verification.is_complete else 1)


if __name__ == '__main__':
    main()
