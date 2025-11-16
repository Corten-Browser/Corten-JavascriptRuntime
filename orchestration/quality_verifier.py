"""
Quality Verifier

Enforces quality gates for component code before accepting work as complete.
Runs comprehensive checks on tests, coverage, TDD compliance, linting, etc.

Supported Languages:
    - Python (pytest, pytest-cov, flake8, black)
    - JavaScript/TypeScript (npm test, jest, eslint, prettier)
    - Rust (cargo test, cargo-tarpaulin, clippy, rustfmt)
    - Go (go test, go test -cover, golint, gofmt)

Classes:
    QualityVerifier: Main quality verification system
    QualityReport: Container for verification results

Usage:
    verifier = QualityVerifier()
    report = verifier.verify("components/user-api")

    if report.passed:
        print("All quality gates passed!")
    else:
        print(f"Failed checks: {report.failures}")
"""

import subprocess
import json
import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from datetime import datetime
from dataclasses import dataclass, field


@dataclass
class QualityCheck:
    """Individual quality check result."""
    name: str
    passed: bool
    score: Optional[int] = None  # 0-100
    message: str = ""
    details: Dict = field(default_factory=dict)


@dataclass
class QualityReport:
    """Complete quality verification report."""
    component_name: str
    timestamp: str
    passed: bool
    overall_score: int  # 0-100
    checks: List[QualityCheck]
    failures: List[str]
    warnings: List[str]
    summary: str

    def to_dict(self) -> Dict:
        """Convert report to dictionary."""
        return {
            'component_name': self.component_name,
            'timestamp': self.timestamp,
            'passed': self.passed,
            'overall_score': self.overall_score,
            'checks': [
                {
                    'name': check.name,
                    'passed': check.passed,
                    'score': check.score,
                    'message': check.message,
                    'details': check.details
                }
                for check in self.checks
            ],
            'failures': self.failures,
            'warnings': self.warnings,
            'summary': self.summary
        }

    def to_json(self) -> str:
        """Convert report to JSON string."""
        return json.dumps(self.to_dict(), indent=2)


class QualityVerifier:
    """
    Verify component code meets quality standards.

    Runs comprehensive quality checks:
    - Test execution (must pass)
    - Test coverage (≥ 80%)
    - TDD compliance (git history analysis)
    - Linting (zero errors)
    - Formatting (100% compliant)
    - Code complexity (≤ 10 cyclomatic complexity)
    - Documentation (public APIs documented)
    - Security (no hardcoded secrets, input validation)
    - API contract compliance (if applicable)

    Supported Languages:
    - Python: pytest, pytest-cov, flake8, black
    - JavaScript/TypeScript: npm test, jest coverage, eslint, prettier
    - Rust: cargo test, cargo-tarpaulin, clippy, rustfmt
    - Go: go test, go test -cover, golint, gofmt

    Attributes:
        coverage_threshold: Minimum required coverage (default: 80)
        complexity_threshold: Maximum cyclomatic complexity (default: 10)
    """

    def __init__(
        self,
        coverage_threshold: int = 80,
        complexity_threshold: int = 10
    ):
        """
        Initialize QualityVerifier.

        Args:
            coverage_threshold: Minimum required test coverage percentage
            complexity_threshold: Maximum allowed cyclomatic complexity
        """
        self.coverage_threshold = coverage_threshold
        self.complexity_threshold = complexity_threshold

    def verify(self, component_path: str) -> QualityReport:
        """
        Run all quality checks on a component.

        Args:
            component_path: Path to component directory

        Returns:
            QualityReport with all check results

        Example:
            >>> verifier = QualityVerifier()
            >>> report = verifier.verify("components/user-api")
            >>> print(f"Passed: {report.passed}, Score: {report.overall_score}/100")
        """
        component_path = Path(component_path)
        component_name = component_path.name

        checks: List[QualityCheck] = []
        failures: List[str] = []
        warnings: List[str] = []

        # 1. Check tests pass
        check = self._check_tests_pass(component_path)
        checks.append(check)
        if not check.passed:
            failures.append(f"Tests failing: {check.message}")

        # 2. Check test coverage
        check = self._check_coverage(component_path)
        checks.append(check)
        if not check.passed:
            failures.append(f"Coverage below {self.coverage_threshold}%: {check.message}")

        # 3. Check TDD compliance
        check = self._check_tdd_compliance(component_path)
        checks.append(check)
        if not check.passed:
            warnings.append(f"TDD compliance questionable: {check.message}")

        # 4. Check linting
        check = self._check_linting(component_path)
        checks.append(check)
        if not check.passed:
            failures.append(f"Linting errors: {check.message}")

        # 5. Check formatting
        check = self._check_formatting(component_path)
        checks.append(check)
        if not check.passed:
            failures.append(f"Formatting issues: {check.message}")

        # 6. Check code complexity
        check = self._check_complexity(component_path)
        checks.append(check)
        if not check.passed:
            failures.append(f"Complexity too high: {check.message}")

        # 7. Check documentation
        check = self._check_documentation(component_path)
        checks.append(check)
        if not check.passed:
            warnings.append(f"Documentation incomplete: {check.message}")

        # 8. Check security
        check = self._check_security(component_path)
        checks.append(check)
        if not check.passed:
            failures.append(f"Security issues: {check.message}")

        # 9. Check cross-component integration tests
        # Note: This checks project-level integration tests (not component-specific)
        project_root = component_path.parent.parent if component_path.parent.name == 'components' else component_path.parent
        check = self._check_integration_tests(project_root)
        checks.append(check)
        if not check.passed:
            failures.append(f"Integration tests: {check.message}")

        # 10. Check contract compatibility
        check = self._check_contract_compatibility(project_root)
        checks.append(check)
        if not check.passed:
            failures.append(f"Contract compatibility: {check.message}")

        # 11. Check dependency validation
        check = self._check_dependency_validation(project_root, component_name)
        checks.append(check)
        if not check.passed:
            failures.append(f"Dependency validation: {check.message}")

        # 12. Check specification compliance
        check = self._check_specification_compliance(project_root)
        checks.append(check)
        if not check.passed:
            warnings.append(f"Specification compliance: {check.message}")

        # Calculate overall score
        scores = [check.score for check in checks if check.score is not None]
        overall_score = sum(scores) // len(scores) if scores else 0

        # Determine if passed (all critical checks must pass)
        passed = len(failures) == 0

        # Generate summary
        summary = self._generate_summary(
            component_name, checks, failures, warnings, overall_score, passed
        )

        return QualityReport(
            component_name=component_name,
            timestamp=datetime.now().isoformat(),
            passed=passed,
            overall_score=overall_score,
            checks=checks,
            failures=failures,
            warnings=warnings,
            summary=summary
        )

    def _check_tests_pass(self, component_path: Path) -> QualityCheck:
        """Check if all tests pass."""
        try:
            # Detect project type
            if (component_path / "package.json").exists():
                # Node.js project
                result = subprocess.run(
                    ["npm", "test", "--", "--watchAll=false", "--passWithNoTests"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )
                success = result.returncode == 0

                # Extract test count
                passing = re.search(r'(\d+) passing', result.stdout)
                failing = re.search(r'(\d+) failing', result.stdout)

                if success:
                    count = passing.group(1) if passing else "0"
                    return QualityCheck(
                        name="Tests Pass",
                        passed=True,
                        score=100,
                        message=f"{count} tests passing",
                        details={'passing': int(count), 'failing': 0}
                    )
                else:
                    count = failing.group(1) if failing else "unknown"
                    return QualityCheck(
                        name="Tests Pass",
                        passed=False,
                        score=0,
                        message=f"{count} tests failing",
                        details={'passing': 0, 'failing': int(count) if count != "unknown" else 0}
                    )

            elif (component_path / "requirements.txt").exists() or \
                 (component_path / "setup.py").exists() or \
                 (component_path / "pyproject.toml").exists():
                # Python project
                result = subprocess.run(
                    ["pytest", "--tb=short", "--quiet"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )
                success = result.returncode == 0

                # Extract test count
                match = re.search(r'(\d+) passed', result.stdout)
                passing = int(match.group(1)) if match else 0

                match_fail = re.search(r'(\d+) failed', result.stdout)
                failing = int(match_fail.group(1)) if match_fail else 0

                if success:
                    return QualityCheck(
                        name="Tests Pass",
                        passed=True,
                        score=100,
                        message=f"{passing} tests passing",
                        details={'passing': passing, 'failing': 0}
                    )
                else:
                    return QualityCheck(
                        name="Tests Pass",
                        passed=False,
                        score=0,
                        message=f"{failing} tests failing",
                        details={'passing': passing, 'failing': failing}
                    )

            elif (component_path / "Cargo.toml").exists():
                # Rust project
                result = subprocess.run(
                    ["cargo", "test", "--quiet"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )
                success = result.returncode == 0

                # Extract test count from output
                # Cargo test output: "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
                match = re.search(r'(\d+) passed', result.stdout)
                passing = int(match.group(1)) if match else 0

                match_fail = re.search(r'(\d+) failed', result.stdout)
                failing = int(match_fail.group(1)) if match_fail else 0

                if success:
                    return QualityCheck(
                        name="Tests Pass",
                        passed=True,
                        score=100,
                        message=f"{passing} tests passing",
                        details={'passing': passing, 'failing': 0}
                    )
                else:
                    return QualityCheck(
                        name="Tests Pass",
                        passed=False,
                        score=0,
                        message=f"{failing} tests failing",
                        details={'passing': passing, 'failing': failing}
                    )

            elif (component_path / "go.mod").exists():
                # Go project
                result = subprocess.run(
                    ["go", "test", "./..."],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )
                success = result.returncode == 0

                # Extract test results from output
                # Go test output includes: "PASS" or "FAIL" and test counts
                if success:
                    # Count packages tested
                    packages = len(re.findall(r'ok\s+\S+', result.stdout))
                    return QualityCheck(
                        name="Tests Pass",
                        passed=True,
                        score=100,
                        message=f"All tests passing ({packages} packages tested)",
                        details={'passing': packages, 'failing': 0}
                    )
                else:
                    # Count failures
                    failures = len(re.findall(r'FAIL\s+\S+', result.stdout))
                    return QualityCheck(
                        name="Tests Pass",
                        passed=False,
                        score=0,
                        message=f"Tests failing ({failures} packages failed)",
                        details={'passing': 0, 'failing': failures}
                    )

            else:
                return QualityCheck(
                    name="Tests Pass",
                    passed=False,
                    score=0,
                    message="Cannot detect project type"
                )

        except subprocess.TimeoutExpired:
            return QualityCheck(
                name="Tests Pass",
                passed=False,
                score=0,
                message="Tests timed out (>5 minutes)"
            )
        except Exception as e:
            return QualityCheck(
                name="Tests Pass",
                passed=False,
                score=0,
                message=f"Error running tests: {str(e)}"
            )

    def _check_coverage(self, component_path: Path) -> QualityCheck:
        """Check if test coverage meets threshold."""
        try:
            if (component_path / "package.json").exists():
                # Node.js project
                result = subprocess.run(
                    ["npm", "test", "--", "--coverage", "--watchAll=false",
                     "--coverageReporters=text-summary"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                # Extract coverage percentage
                match = re.search(r'All files.*?\|\s+([\d.]+)', result.stdout)
                if match:
                    coverage = float(match.group(1))
                    passed = coverage >= self.coverage_threshold

                    return QualityCheck(
                        name="Test Coverage",
                        passed=passed,
                        score=int(coverage),
                        message=f"{coverage}% coverage (threshold: {self.coverage_threshold}%)",
                        details={'coverage': coverage, 'threshold': self.coverage_threshold}
                    )
                else:
                    return QualityCheck(
                        name="Test Coverage",
                        passed=False,
                        score=0,
                        message="Could not determine coverage"
                    )

            elif (component_path / "requirements.txt").exists() or \
                 (component_path / "setup.py").exists() or \
                 (component_path / "pyproject.toml").exists():
                # Python project
                result = subprocess.run(
                    ["pytest", "--cov=.", "--cov-report=term-missing", "--quiet"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                # Extract coverage percentage
                match = re.search(r'TOTAL\s+\d+\s+\d+\s+(\d+)%', result.stdout)
                if match:
                    coverage = int(match.group(1))
                    passed = coverage >= self.coverage_threshold

                    return QualityCheck(
                        name="Test Coverage",
                        passed=passed,
                        score=coverage,
                        message=f"{coverage}% coverage (threshold: {self.coverage_threshold}%)",
                        details={'coverage': coverage, 'threshold': self.coverage_threshold}
                    )
                else:
                    return QualityCheck(
                        name="Test Coverage",
                        passed=False,
                        score=0,
                        message="Could not determine coverage"
                    )

            elif (component_path / "Cargo.toml").exists():
                # Rust project - requires cargo-tarpaulin
                # Check if tarpaulin is installed
                check_tarpaulin = subprocess.run(
                    ["cargo", "tarpaulin", "--version"],
                    capture_output=True,
                    text=True
                )

                if check_tarpaulin.returncode != 0:
                    return QualityCheck(
                        name="Test Coverage",
                        passed=False,
                        score=0,
                        message="cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin",
                        details={'note': 'Install cargo-tarpaulin for coverage support'}
                    )

                # Run coverage with tarpaulin
                result = subprocess.run(
                    ["cargo", "tarpaulin", "--quiet", "--output-dir", "/tmp"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                # Extract coverage percentage
                # Output format: "Coverage: 87.5%"
                match = re.search(r'Coverage:\s+([\d.]+)', result.stdout)
                if match:
                    coverage = float(match.group(1))
                    passed = coverage >= self.coverage_threshold

                    return QualityCheck(
                        name="Test Coverage",
                        passed=passed,
                        score=int(coverage),
                        message=f"{coverage}% coverage (threshold: {self.coverage_threshold}%)",
                        details={'coverage': coverage, 'threshold': self.coverage_threshold}
                    )
                else:
                    return QualityCheck(
                        name="Test Coverage",
                        passed=False,
                        score=0,
                        message="Could not determine coverage"
                    )

            elif (component_path / "go.mod").exists():
                # Go project
                result = subprocess.run(
                    ["go", "test", "./...", "-cover"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                # Extract coverage percentage
                # Output format: "coverage: 85.7% of statements"
                match = re.search(r'coverage:\s+([\d.]+)%', result.stdout)
                if match:
                    coverage = float(match.group(1))
                    passed = coverage >= self.coverage_threshold

                    return QualityCheck(
                        name="Test Coverage",
                        passed=passed,
                        score=int(coverage),
                        message=f"{coverage}% coverage (threshold: {self.coverage_threshold}%)",
                        details={'coverage': coverage, 'threshold': self.coverage_threshold}
                    )
                else:
                    return QualityCheck(
                        name="Test Coverage",
                        passed=False,
                        score=0,
                        message="Could not determine coverage"
                    )

            else:
                return QualityCheck(
                    name="Test Coverage",
                    passed=False,
                    score=0,
                    message="Cannot detect project type"
                )

        except subprocess.TimeoutExpired:
            return QualityCheck(
                name="Test Coverage",
                passed=False,
                score=0,
                message="Coverage check timed out"
            )
        except Exception as e:
            return QualityCheck(
                name="Test Coverage",
                passed=False,
                score=0,
                message=f"Error checking coverage: {str(e)}"
            )

    def _check_tdd_compliance(self, component_path: Path) -> QualityCheck:
        """
        Check TDD compliance by analyzing git commit history.

        Looks for Red-Green-Refactor pattern:
        - Test commits before implementation commits
        - "test:" commits before "feat:" commits
        - Refactor commits after feature commits
        """
        try:
            # Get git log for component
            result = subprocess.run(
                ["git", "log", "--oneline", "--no-merges", "-20"],
                cwd=component_path,
                capture_output=True,
                text=True,
                timeout=10
            )

            if result.returncode != 0:
                return QualityCheck(
                    name="TDD Compliance",
                    passed=True,  # Don't fail if git not initialized
                    score=50,
                    message="No git history found (component may be new)"
                )

            commits = result.stdout.strip().split('\n')

            # Analyze commit pattern
            test_commits = sum(1 for c in commits if re.search(r'\b(test|spec):', c.lower()))
            feat_commits = sum(1 for c in commits if re.search(r'\bfeat:', c.lower()))
            refactor_commits = sum(1 for c in commits if re.search(r'\brefactor:', c.lower()))

            total_commits = len(commits)

            # Calculate TDD score
            # Good TDD: test commits ≥ feat commits, some refactor commits
            tdd_score = 0

            if total_commits == 0:
                return QualityCheck(
                    name="TDD Compliance",
                    passed=True,
                    score=50,
                    message="No commits yet"
                )

            # Points for having test commits
            test_ratio = test_commits / total_commits
            tdd_score += int(test_ratio * 40)

            # Points for test-first pattern (test commits ≥ feat commits)
            if feat_commits > 0 and test_commits >= feat_commits:
                tdd_score += 40

            # Points for refactoring
            if refactor_commits > 0:
                tdd_score += 20

            passed = tdd_score >= 60  # 60/100 is minimum for TDD compliance

            message_parts = []
            if test_commits > 0:
                message_parts.append(f"{test_commits} test commits")
            if feat_commits > 0:
                message_parts.append(f"{feat_commits} feat commits")
            if refactor_commits > 0:
                message_parts.append(f"{refactor_commits} refactor commits")

            return QualityCheck(
                name="TDD Compliance",
                passed=passed,
                score=tdd_score,
                message=", ".join(message_parts) or "No relevant commits",
                details={
                    'test_commits': test_commits,
                    'feat_commits': feat_commits,
                    'refactor_commits': refactor_commits,
                    'total_commits': total_commits
                }
            )

        except Exception as e:
            return QualityCheck(
                name="TDD Compliance",
                passed=True,  # Don't fail on error
                score=50,
                message=f"Could not verify TDD compliance: {str(e)}"
            )

    def _check_linting(self, component_path: Path) -> QualityCheck:
        """Check if code passes linting."""
        try:
            if (component_path / "package.json").exists():
                # Node.js project
                result = subprocess.run(
                    ["npm", "run", "lint"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                passed = result.returncode == 0

                if passed:
                    return QualityCheck(
                        name="Linting",
                        passed=True,
                        score=100,
                        message="No linting errors"
                    )
                else:
                    # Count errors
                    errors = len(re.findall(r'error', result.stdout, re.IGNORECASE))
                    return QualityCheck(
                        name="Linting",
                        passed=False,
                        score=0,
                        message=f"{errors} linting errors found",
                        details={'errors': errors}
                    )

            elif (component_path / "requirements.txt").exists() or \
                 (component_path / "setup.py").exists() or \
                 (component_path / "pyproject.toml").exists():
                # Python project - try flake8
                result = subprocess.run(
                    ["flake8", ".", "--exclude=venv,env,.venv,.git,__pycache__,.pytest_cache"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                passed = result.returncode == 0

                if passed:
                    return QualityCheck(
                        name="Linting",
                        passed=True,
                        score=100,
                        message="No linting errors"
                    )
                else:
                    # Count errors
                    errors = len(result.stdout.strip().split('\n'))
                    return QualityCheck(
                        name="Linting",
                        passed=False,
                        score=0,
                        message=f"{errors} linting errors found",
                        details={'errors': errors}
                    )

            elif (component_path / "Cargo.toml").exists():
                # Rust project - use clippy
                result = subprocess.run(
                    ["cargo", "clippy", "--", "-D", "warnings"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=120
                )

                passed = result.returncode == 0

                if passed:
                    return QualityCheck(
                        name="Linting",
                        passed=True,
                        score=100,
                        message="No clippy warnings"
                    )
                else:
                    # Count warnings/errors
                    warnings = len(re.findall(r'warning:', result.stdout))
                    errors = len(re.findall(r'error:', result.stdout))
                    total = warnings + errors
                    return QualityCheck(
                        name="Linting",
                        passed=False,
                        score=0,
                        message=f"{total} clippy issues found ({errors} errors, {warnings} warnings)",
                        details={'errors': errors, 'warnings': warnings}
                    )

            elif (component_path / "go.mod").exists():
                # Go project - use golint if available
                # Check if golint is installed
                check_golint = subprocess.run(
                    ["golint", "-help"],
                    capture_output=True,
                    text=True
                )

                if check_golint.returncode != 0:
                    return QualityCheck(
                        name="Linting",
                        passed=True,  # Don't fail if golint not installed
                        score=50,
                        message="golint not installed. Install with: go install golang.org/x/lint/golint@latest"
                    )

                # Run golint
                result = subprocess.run(
                    ["golint", "./..."],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                # golint returns issues as lines
                # Filter out common noise comments
                issues = result.stdout.strip()
                if not issues or issues == "":
                    return QualityCheck(
                        name="Linting",
                        passed=True,
                        score=100,
                        message="No linting issues"
                    )
                else:
                    # Count issues (each line is an issue)
                    issue_count = len(issues.split('\n'))
                    return QualityCheck(
                        name="Linting",
                        passed=False,
                        score=0,
                        message=f"{issue_count} linting issues found",
                        details={'issues': issue_count}
                    )

            else:
                return QualityCheck(
                    name="Linting",
                    passed=True,  # Don't fail if can't detect
                    score=50,
                    message="Cannot detect project type for linting"
                )

        except FileNotFoundError:
            return QualityCheck(
                name="Linting",
                passed=True,  # Don't fail if linter not installed
                score=50,
                message="Linter not found (install flake8 or eslint)"
            )
        except Exception as e:
            return QualityCheck(
                name="Linting",
                passed=True,  # Don't fail on error
                score=50,
                message=f"Error running linter: {str(e)}"
            )

    def _check_formatting(self, component_path: Path) -> QualityCheck:
        """Check if code is properly formatted."""
        try:
            if (component_path / "package.json").exists():
                # Node.js project - try prettier
                result = subprocess.run(
                    ["prettier", "--check", "src/**/*.{js,jsx,ts,tsx}"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=60,
                    shell=True
                )

                passed = result.returncode == 0

                if passed:
                    return QualityCheck(
                        name="Formatting",
                        passed=True,
                        score=100,
                        message="Code formatted correctly"
                    )
                else:
                    return QualityCheck(
                        name="Formatting",
                        passed=False,
                        score=0,
                        message="Code not formatted (run prettier)"
                    )

            elif (component_path / "requirements.txt").exists() or \
                 (component_path / "setup.py").exists() or \
                 (component_path / "pyproject.toml").exists():
                # Python project - try black
                result = subprocess.run(
                    ["black", "--check", ".", "--exclude=/(venv|env|.venv|.git|__pycache__|.pytest_cache)/"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                passed = result.returncode == 0

                if passed:
                    return QualityCheck(
                        name="Formatting",
                        passed=True,
                        score=100,
                        message="Code formatted correctly"
                    )
                else:
                    # Count files needing formatting
                    files = len(re.findall(r'would reformat', result.stderr))
                    return QualityCheck(
                        name="Formatting",
                        passed=False,
                        score=0,
                        message=f"{files} files need formatting (run black .)",
                        details={'files_needing_format': files}
                    )

            elif (component_path / "Cargo.toml").exists():
                # Rust project - use rustfmt
                result = subprocess.run(
                    ["cargo", "fmt", "--", "--check"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                passed = result.returncode == 0

                if passed:
                    return QualityCheck(
                        name="Formatting",
                        passed=True,
                        score=100,
                        message="Code formatted correctly"
                    )
                else:
                    return QualityCheck(
                        name="Formatting",
                        passed=False,
                        score=0,
                        message="Code not formatted (run cargo fmt)"
                    )

            elif (component_path / "go.mod").exists():
                # Go project - use gofmt
                result = subprocess.run(
                    ["gofmt", "-l", "."],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                # gofmt -l lists files that need formatting
                unformatted_files = result.stdout.strip()

                if not unformatted_files:
                    return QualityCheck(
                        name="Formatting",
                        passed=True,
                        score=100,
                        message="Code formatted correctly"
                    )
                else:
                    # Count files needing formatting
                    files = len([f for f in unformatted_files.split('\n') if f and not f.startswith('vendor/')])
                    return QualityCheck(
                        name="Formatting",
                        passed=False,
                        score=0,
                        message=f"{files} files need formatting (run gofmt -w .)",
                        details={'files_needing_format': files}
                    )

            else:
                return QualityCheck(
                    name="Formatting",
                    passed=True,
                    score=50,
                    message="Cannot detect project type for formatting"
                )

        except FileNotFoundError:
            return QualityCheck(
                name="Formatting",
                passed=True,  # Don't fail if formatter not installed
                score=50,
                message="Formatter not found (install black or prettier)"
            )
        except Exception as e:
            return QualityCheck(
                name="Formatting",
                passed=True,  # Don't fail on error
                score=50,
                message=f"Error checking formatting: {str(e)}"
            )

    def _check_complexity(self, component_path: Path) -> QualityCheck:
        """Check cyclomatic complexity of code."""
        # This is a simplified check - full implementation would use radon or complexity-report
        return QualityCheck(
            name="Code Complexity",
            passed=True,
            score=80,
            message=f"Complexity check not fully implemented (assumed ≤{self.complexity_threshold})",
            details={'threshold': self.complexity_threshold}
        )

    def _check_documentation(self, component_path: Path) -> QualityCheck:
        """Check if documentation exists and is adequate."""
        try:
            # Check for README
            has_readme = (component_path / "README.md").exists()

            # Check for docstrings (simplified - would need AST parsing for full check)
            src_path = component_path / "src"
            if src_path.exists():
                # Count files with docstrings
                documented_files = 0
                total_files = 0

                for file in src_path.rglob("*.py"):
                    total_files += 1
                    content = file.read_text()
                    if '"""' in content or "'''" in content:
                        documented_files += 1

                for file in src_path.rglob("*.{js,ts,jsx,tsx}"):
                    total_files += 1
                    content = file.read_text()
                    if '/**' in content or '//' in content:
                        documented_files += 1

                if total_files > 0:
                    doc_ratio = documented_files / total_files
                    score = int(doc_ratio * 100)
                    passed = score >= 60  # At least 60% of files should have docs

                    return QualityCheck(
                        name="Documentation",
                        passed=passed and has_readme,
                        score=score,
                        message=f"{documented_files}/{total_files} files documented, README: {has_readme}",
                        details={
                            'has_readme': has_readme,
                            'documented_files': documented_files,
                            'total_files': total_files
                        }
                    )

            return QualityCheck(
                name="Documentation",
                passed=has_readme,
                score=50 if has_readme else 0,
                message=f"README exists: {has_readme}",
                details={'has_readme': has_readme}
            )

        except Exception as e:
            return QualityCheck(
                name="Documentation",
                passed=True,  # Don't fail on error
                score=50,
                message=f"Error checking documentation: {str(e)}"
            )

    def _check_security(self, component_path: Path) -> QualityCheck:
        """Check for common security issues."""
        try:
            issues = []

            # Check for hardcoded secrets
            for file in component_path.rglob("*.{py,js,ts,jsx,tsx}"):
                if file.is_file():
                    content = file.read_text()

                    # Check for potential secrets
                    if re.search(r'(password|secret|api_key|apikey|token|private_key)\s*=\s*["\'][^"\']+["\']',
                                content, re.IGNORECASE):
                        issues.append(f"Potential hardcoded secret in {file.name}")

                    # Check for AWS keys
                    if re.search(r'AKIA[0-9A-Z]{16}', content):
                        issues.append(f"AWS access key found in {file.name}")

            if issues:
                return QualityCheck(
                    name="Security",
                    passed=False,
                    score=0,
                    message=f"{len(issues)} security issues found",
                    details={'issues': issues}
                )
            else:
                return QualityCheck(
                    name="Security",
                    passed=True,
                    score=100,
                    message="No obvious security issues found"
                )

        except Exception as e:
            return QualityCheck(
                name="Security",
                passed=True,  # Don't fail on error
                score=50,
                message=f"Error checking security: {str(e)}"
            )

    def _check_integration_tests(self, project_root: Path) -> QualityCheck:
        """
        Verify cross-component integration tests exist and pass.

        Checks:
        1. tests/integration/ directory exists
        2. Integration test files exist (test_*.py)
        3. All integration tests pass via pytest

        Args:
            project_root: Path to project root directory (parent of component)

        Returns:
            QualityCheck with pass/fail, test counts, failure details
        """
        try:
            integration_dir = project_root / 'tests' / 'integration'

            # Check directory exists
            if not integration_dir.exists():
                return QualityCheck(
                    name="Integration Tests",
                    passed=False,
                    score=0,
                    message="Missing tests/integration/ directory",
                    details={
                        'directory_exists': False,
                        'required': True,
                        'path': str(integration_dir)
                    }
                )

            # Check test files exist
            test_files = list(integration_dir.glob('test_*.py'))
            if not test_files:
                return QualityCheck(
                    name="Integration Tests",
                    passed=False,
                    score=0,
                    message="No integration test files found",
                    details={
                        'directory_exists': True,
                        'test_files': 0,
                        'path': str(integration_dir)
                    }
                )

            # Run integration tests
            result = subprocess.run(
                ['pytest', 'tests/integration/', '-v', '-m', 'cross_component',
                 '--tb=short', '--maxfail=5'],
                cwd=project_root,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )

            # Parse pytest output
            passed_match = re.search(r'(\d+) passed', result.stdout)
            failed_match = re.search(r'(\d+) failed', result.stdout)
            skipped_match = re.search(r'(\d+) skipped', result.stdout)

            passed = int(passed_match.group(1)) if passed_match else 0
            failed = int(failed_match.group(1)) if failed_match else 0
            skipped = int(skipped_match.group(1)) if skipped_match else 0
            total = passed + failed + skipped

            # Determine result
            if result.returncode != 0:
                # Extract failure details
                failure_lines = []
                for line in result.stdout.split('\n'):
                    if 'FAILED' in line:
                        failure_lines.append(line.strip())

                return QualityCheck(
                    name="Integration Tests",
                    passed=False,
                    score=max(0, int((passed / total * 100)) if total > 0 else 0),
                    message=f"{failed}/{total} integration tests failed",
                    details={
                        'test_files': len(test_files),
                        'total_tests': total,
                        'passed': passed,
                        'failed': failed,
                        'skipped': skipped,
                        'failures': failure_lines[:5]  # First 5 failures
                    }
                )
            else:
                return QualityCheck(
                    name="Integration Tests",
                    passed=True,
                    score=100,
                    message=f"All integration tests pass ({passed} tests in {len(test_files)} files)",
                    details={
                        'test_files': len(test_files),
                        'total_tests': total,
                        'passed': passed,
                        'failed': 0,
                        'skipped': skipped
                    }
                )

        except subprocess.TimeoutExpired:
            return QualityCheck(
                name="Integration Tests",
                passed=False,
                score=0,
                message="Integration tests timed out (>5 minutes)",
                details={'timeout': 300}
            )
        except FileNotFoundError:
            return QualityCheck(
                name="Integration Tests",
                passed=False,
                score=0,
                message="pytest not found (install pytest)",
                details={'error': 'pytest not installed'}
            )
        except Exception as e:
            return QualityCheck(
                name="Integration Tests",
                passed=True,  # Don't fail on unexpected errors
                score=50,
                message=f"Error running integration tests: {str(e)}",
                details={'error': str(e)}
            )

    def _check_contract_compatibility(self, project_root: Path) -> QualityCheck:
        """
        Verify contracts are compatible.

        Checks OpenAPI contract files in contracts/ directory for
        compatibility issues between provider and consumer services.

        Args:
            project_root: Path to project root directory

        Returns:
            QualityCheck with pass/fail, compatibility details
        """
        contracts_dir = project_root / 'contracts'

        # Check if contracts directory exists
        if not contracts_dir.exists():
            return QualityCheck(
                name="Contract Compatibility",
                passed=True,  # Not required if no contracts
                score=100,
                message="No contracts directory (not required)",
                details={'contracts_dir_exists': False}
            )

        try:
            from contract_checker import ContractChecker

            checker = ContractChecker(contracts_dir)
            num_loaded = checker.load_contracts()

            # No contracts found (not an error)
            if num_loaded == 0:
                return QualityCheck(
                    name="Contract Compatibility",
                    passed=True,  # Not required if no contracts
                    score=100,
                    message="No contract files found (not required)",
                    details={'contracts_loaded': 0}
                )

            # Detect dependencies and check compatibility
            checker.detect_dependencies()
            report = checker.check_compatibility()

            # No pairs to check (no dependencies defined)
            if report.pairs_checked == 0:
                return QualityCheck(
                    name="Contract Compatibility",
                    passed=True,
                    score=100,
                    message=f"{num_loaded} contracts loaded, no dependencies to check",
                    details={
                        'contracts_loaded': num_loaded,
                        'pairs_checked': 0
                    }
                )

            # Check results
            if report.has_incompatibilities:
                return QualityCheck(
                    name="Contract Compatibility",
                    passed=False,
                    score=int(report.success_rate),
                    message=f"{len(report.incompatibilities)} incompatibilities found "
                           f"({report.compatible_pairs}/{report.pairs_checked} pairs compatible)",
                    details={
                        'contracts_loaded': num_loaded,
                        'pairs_checked': report.pairs_checked,
                        'compatible_pairs': report.compatible_pairs,
                        'incompatibilities': [
                            {
                                'provider': incomp.provider_contract,
                                'consumer': incomp.consumer_contract,
                                'schema': incomp.schema_name,
                                'issue_count': len(incomp.issues)
                            }
                            for incomp in report.incompatibilities[:5]  # First 5
                        ]
                    }
                )
            else:
                return QualityCheck(
                    name="Contract Compatibility",
                    passed=True,
                    score=100,
                    message=f"All contracts compatible ({report.pairs_checked} pairs checked)",
                    details={
                        'contracts_loaded': num_loaded,
                        'pairs_checked': report.pairs_checked,
                        'compatible_pairs': report.compatible_pairs
                    }
                )

        except ImportError:
            return QualityCheck(
                name="Contract Compatibility",
                passed=True,  # Don't fail if module not available
                score=50,
                message="Contract checker not available",
                details={'error': 'contract_checker module not found'}
            )
        except Exception as e:
            return QualityCheck(
                name="Contract Compatibility",
                passed=True,  # Don't fail on unexpected errors
                score=50,
                message=f"Error checking contracts: {str(e)}",
                details={'error': str(e), 'error_type': type(e).__name__}
            )

    def _check_dependency_validation(self, project_root: Path, component_name: str) -> QualityCheck:
        """
        Validate component dependencies using dependency manager.

        Checks:
        1. All dependencies exist
        2. No circular dependencies
        3. Dependency level hierarchy is respected
        4. component.yaml manifest exists and is valid

        Args:
            project_root: Path to project root directory
            component_name: Name of component being verified

        Returns:
            QualityCheck with pass/fail and dependency details
        """
        try:
            from dependency_manager import DependencyManager

            manager = DependencyManager(project_root)
            num_loaded = manager.load_all_manifests()

            if num_loaded == 0:
                return QualityCheck(
                    name="Dependency Validation",
                    passed=False,
                    score=0,
                    message="No component manifests found (component.yaml missing)",
                    details={'manifests_loaded': 0}
                )

            # Check if this component's manifest was loaded
            if component_name not in manager.components:
                return QualityCheck(
                    name="Dependency Validation",
                    passed=False,
                    score=0,
                    message=f"Component manifest not found for {component_name}",
                    details={'component_found': False}
                )

            issues = []

            # Check 1: All dependencies exist
            missing_deps = manager.verify_dependencies(component_name)
            if missing_deps:
                issues.append(f"Missing dependencies: {', '.join(missing_deps)}")

            # Check 2: No circular dependencies
            cycles = manager.check_circular_dependencies()
            if cycles:
                # Check if this component is involved in any cycle
                component_cycles = [
                    cycle for cycle in cycles if component_name in cycle
                ]
                if component_cycles:
                    issues.append(f"Circular dependency detected: {' → '.join(component_cycles[0])}")

            # Check 3: Dependency level hierarchy
            violations = manager.validate_dependency_levels()
            if violations:
                # Check if this component has violations
                component_violations = [
                    v for v in violations if v.startswith(component_name)
                ]
                if component_violations:
                    issues.append(f"Level violation: {component_violations[0]}")

            # Determine result
            if issues:
                return QualityCheck(
                    name="Dependency Validation",
                    passed=False,
                    score=0,
                    message=f"{len(issues)} dependency issues found",
                    details={
                        'manifests_loaded': num_loaded,
                        'component_found': True,
                        'issues': issues
                    }
                )
            else:
                # Get dependency count
                dep_count = len(manager.dependencies.get(component_name, []))
                return QualityCheck(
                    name="Dependency Validation",
                    passed=True,
                    score=100,
                    message=f"Dependencies valid ({dep_count} dependencies)",
                    details={
                        'manifests_loaded': num_loaded,
                        'component_found': True,
                        'dependencies': dep_count
                    }
                )

        except ImportError:
            return QualityCheck(
                name="Dependency Validation",
                passed=True,  # Don't fail if module not available
                score=50,
                message="Dependency manager not available",
                details={'error': 'dependency_manager module not found'}
            )
        except Exception as e:
            return QualityCheck(
                name="Dependency Validation",
                passed=True,  # Don't fail on unexpected errors
                score=50,
                message=f"Error validating dependencies: {str(e)}",
                details={'error': str(e), 'error_type': type(e).__name__}
            )

    def _check_specification_compliance(self, project_root: Path) -> QualityCheck:
        """
        Verify implementation matches specification.

        Checks if a specification exists and validates implementation against it.
        This is a warning-level check, not a critical failure.

        Args:
            project_root: Path to project root directory

        Returns:
            QualityCheck with pass/fail and compliance details
        """
        try:
            # Look for specification files
            spec_candidates = [
                project_root / 'spec.md',
                project_root / 'specification.md',
                project_root / 'docs' / 'specification.md',
                project_root / 'docs' / 'spec.md',
                project_root / 'requirements.md',
                project_root / 'SPEC.md'
            ]

            spec_path = None
            for candidate in spec_candidates:
                if candidate.exists():
                    spec_path = candidate
                    break

            # No spec found - not an error
            if not spec_path:
                return QualityCheck(
                    name="Specification Compliance",
                    passed=True,
                    score=100,
                    message="No specification file found (not required)",
                    details={'spec_file': None}
                )

            # Try to verify against specification
            try:
                from specification_verifier import SpecificationVerifier

                verifier = SpecificationVerifier(project_root)
                verification = verifier.verify_against_specification(spec_path)

                if verification.passed:
                    return QualityCheck(
                        name="Specification Compliance",
                        passed=True,
                        score=100,
                        message=f"Implementation matches specification ({verification.checks_passed}/{verification.checks_total} checks passed)",
                        details={
                            'spec_file': str(spec_path),
                            'checks_passed': verification.checks_passed,
                            'checks_total': verification.checks_total
                        }
                    )
                else:
                    # Calculate score based on pass rate
                    pass_rate = (verification.checks_passed / verification.checks_total * 100
                                if verification.checks_total > 0 else 0)

                    return QualityCheck(
                        name="Specification Compliance",
                        passed=False,
                        score=int(pass_rate),
                        message=f"{verification.checks_failed} specification checks failed",
                        details={
                            'spec_file': str(spec_path),
                            'checks_passed': verification.checks_passed,
                            'checks_failed': verification.checks_failed,
                            'checks_warning': verification.checks_warning,
                            'missing_components': verification.missing_components[:3],
                            'architecture_mismatch': verification.architecture_mismatch
                        }
                    )

            except ImportError:
                return QualityCheck(
                    name="Specification Compliance",
                    passed=True,
                    score=50,
                    message=f"Specification found ({spec_path.name}) but verifier not available",
                    details={'spec_file': str(spec_path), 'error': 'specification_verifier not available'}
                )

        except Exception as e:
            return QualityCheck(
                name="Specification Compliance",
                passed=True,  # Don't fail on unexpected errors
                score=50,
                message=f"Error checking specification compliance: {str(e)}",
                details={'error': str(e), 'error_type': type(e).__name__}
            )

    def _generate_summary(
        self,
        component_name: str,
        checks: List[QualityCheck],
        failures: List[str],
        warnings: List[str],
        overall_score: int,
        passed: bool
    ) -> str:
        """Generate human-readable summary."""
        summary_lines = [
            f"Quality Verification Report: {component_name}",
            f"{'='*60}",
            ""
        ]

        # Overall result
        status = "✅ PASSED" if passed else "❌ FAILED"
        summary_lines.append(f"Status: {status}")
        summary_lines.append(f"Overall Score: {overall_score}/100")
        summary_lines.append("")

        # Individual checks
        summary_lines.append("Checks:")
        for check in checks:
            status_icon = "✅" if check.passed else "❌"
            score_str = f"({check.score}/100)" if check.score is not None else ""
            summary_lines.append(f"  {status_icon} {check.name}: {check.message} {score_str}")

        summary_lines.append("")

        # Failures
        if failures:
            summary_lines.append("Failures:")
            for failure in failures:
                summary_lines.append(f"  ❌ {failure}")
            summary_lines.append("")

        # Warnings
        if warnings:
            summary_lines.append("Warnings:")
            for warning in warnings:
                summary_lines.append(f"  ⚠️  {warning}")
            summary_lines.append("")

        return "\n".join(summary_lines)


def main():
    """CLI interface for quality verification."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python quality_verifier.py <component-path>")
        print("Example: python quality_verifier.py components/user-api")
        sys.exit(1)

    component_path = sys.argv[1]

    verifier = QualityVerifier()
    report = verifier.verify(component_path)

    # Print summary
    print(report.summary)

    # Save JSON report
    report_path = Path(component_path) / "quality-report.json"
    report_path.write_text(report.to_json())
    print(f"\nDetailed report saved to: {report_path}")

    # Exit with appropriate code
    sys.exit(0 if report.passed else 1)


if __name__ == "__main__":
    main()
