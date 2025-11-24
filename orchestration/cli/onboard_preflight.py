#!/usr/bin/env python3
"""
Preflight Checks for Existing Project Onboarding

Validates whether an existing project is ready to be onboarded into the
orchestration system. Checks for conflicts, prerequisites, and potential issues.

Usage:
    python orchestration/cli/onboard_preflight.py [project_dir]
    python orchestration/cli/onboard_preflight.py --fix [project_dir]

Exit Codes:
    0 - All checks passed, ready for onboarding
    1 - Critical failures, cannot proceed
    2 - Warnings present, user should review
"""

import sys
import json
import subprocess
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass, field
from enum import Enum

# Ensure project root is in path for imports
_script_dir = Path(__file__).parent
_project_root = _script_dir.parent.parent
if str(_project_root) not in sys.path:
    sys.path.insert(0, str(_project_root))

# Import StateManager for mode detection
try:
    from orchestration.state.state_manager import StateManager, OnboardingMode
except ImportError as e:
    # Fallback if running standalone
    print(f"Warning: Could not import StateManager: {e}", file=sys.stderr)
    StateManager = None
    OnboardingMode = None


class CheckLevel(Enum):
    """Severity levels for preflight checks"""
    CRITICAL = "critical"  # Must pass to proceed
    WARNING = "warning"    # Should review but can proceed
    INFO = "info"          # Informational only


@dataclass
class CheckResult:
    """Result of a single preflight check"""
    name: str
    level: CheckLevel
    passed: bool
    message: str
    details: List[str] = field(default_factory=list)
    fix_available: bool = False
    fix_command: Optional[str] = None


class PreflightChecker:
    """Validates existing project readiness for orchestration onboarding"""

    def __init__(self, project_dir: Path, auto_fix: bool = False):
        self.project_dir = project_dir.resolve()
        self.auto_fix = auto_fix
        self.results: List[CheckResult] = []

    def run_all_checks(self) -> Tuple[bool, int, int]:
        """
        Run all preflight checks

        Returns:
            (can_proceed, critical_failures, warnings)
        """
        self.results = []

        # Phase 1: Critical Prerequisites
        self._check_directory_exists()
        self._check_git_repository()
        self._check_git_status()
        self._check_python_environment()

        # Phase 2: Conflict Detection
        self._check_not_already_orchestrated()
        self._check_directory_conflicts()
        self._check_file_conflicts()

        # Phase 3: Project Structure Detection
        self._check_has_source_code()
        self._check_project_type()

        # Phase 4: Tool Availability
        self._check_required_tools()

        # Calculate results
        critical_failures = sum(
            1 for r in self.results
            if r.level == CheckLevel.CRITICAL and not r.passed
        )
        warnings = sum(
            1 for r in self.results
            if r.level == CheckLevel.WARNING and not r.passed
        )

        can_proceed = critical_failures == 0

        return can_proceed, critical_failures, warnings

    def detect_onboarding_mode(self) -> Optional[str]:
        """
        Detect onboarding mode using StateManager.

        Returns:
            Mode string ("fresh", "resume", "upgrade", "verify") or None if error
        """
        if StateManager is None:
            return None

        try:
            manager = StateManager(self.project_dir)
            mode = manager.detect_mode()
            return mode.value
        except Exception as e:
            print(f"Warning: Failed to detect mode: {e}", file=sys.stderr)
            return None

    def _check_directory_exists(self):
        """Check that project directory exists and is readable"""
        if not self.project_dir.exists():
            self.results.append(CheckResult(
                name="Directory Exists",
                level=CheckLevel.CRITICAL,
                passed=False,
                message=f"Project directory does not exist: {self.project_dir}"
            ))
            return

        if not self.project_dir.is_dir():
            self.results.append(CheckResult(
                name="Directory Exists",
                level=CheckLevel.CRITICAL,
                passed=False,
                message=f"Path is not a directory: {self.project_dir}"
            ))
            return

        self.results.append(CheckResult(
            name="Directory Exists",
            level=CheckLevel.CRITICAL,
            passed=True,
            message="Project directory exists and is accessible"
        ))

    def _check_git_repository(self):
        """Check that project is a git repository"""
        git_dir = self.project_dir / ".git"

        if not git_dir.exists():
            self.results.append(CheckResult(
                name="Git Repository",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="Not a git repository (no .git directory found)",
                details=[
                    "Orchestration system requires git for history preservation",
                    "File migrations use 'git mv' to preserve history"
                ],
                fix_available=True,
                fix_command="git init"
            ))

            if self.auto_fix:
                self._run_fix("git init")
            return

        self.results.append(CheckResult(
            name="Git Repository",
            level=CheckLevel.CRITICAL,
            passed=True,
            message="Project is a git repository"
        ))

    def _check_git_status(self):
        """Check that git working tree is clean"""
        try:
            result = subprocess.run(
                ["git", "status", "--porcelain"],
                cwd=self.project_dir,
                capture_output=True,
                text=True,
                check=True
            )

            if result.stdout.strip():
                uncommitted_files = result.stdout.strip().split('\n')
                self.results.append(CheckResult(
                    name="Git Status",
                    level=CheckLevel.WARNING,
                    passed=False,
                    message=f"Git working tree has {len(uncommitted_files)} uncommitted changes",
                    details=[
                        "Recommendation: Commit or stash changes before onboarding",
                        "This allows easy rollback if needed"
                    ] + uncommitted_files[:10],  # Show first 10
                    fix_available=True,
                    fix_command="git add -A && git commit -m 'Pre-onboarding checkpoint'"
                ))
            else:
                self.results.append(CheckResult(
                    name="Git Status",
                    level=CheckLevel.INFO,
                    passed=True,
                    message="Git working tree is clean"
                ))

        except subprocess.CalledProcessError as e:
            self.results.append(CheckResult(
                name="Git Status",
                level=CheckLevel.WARNING,
                passed=False,
                message="Could not check git status",
                details=[str(e)]
            ))

    def _check_python_environment(self):
        """Check Python version and environment"""
        try:
            result = subprocess.run(
                ["python3", "--version"],
                capture_output=True,
                text=True,
                check=True
            )
            version_str = result.stdout.strip()

            # Parse version (Python 3.X.Y)
            version_parts = version_str.split()[1].split('.')
            major, minor = int(version_parts[0]), int(version_parts[1])

            if major < 3 or (major == 3 and minor < 8):
                self.results.append(CheckResult(
                    name="Python Version",
                    level=CheckLevel.CRITICAL,
                    passed=False,
                    message=f"Python 3.8+ required, found {version_str}",
                    details=["Orchestration system requires Python 3.8 or later"]
                ))
            else:
                self.results.append(CheckResult(
                    name="Python Version",
                    level=CheckLevel.CRITICAL,
                    passed=True,
                    message=f"Python version OK: {version_str}"
                ))

        except (subprocess.CalledProcessError, FileNotFoundError, ValueError) as e:
            self.results.append(CheckResult(
                name="Python Version",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="Could not verify Python version",
                details=[str(e)]
            ))

    def _check_not_already_orchestrated(self):
        """Check that project is not already orchestrated"""
        indicators = [
            self.project_dir / "orchestration" / "VERSION",
            self.project_dir / "orchestration-config.json",
            self.project_dir / ".claude" / "commands" / "orchestrate.md"
        ]

        found = [p for p in indicators if p.exists()]

        if found:
            self.results.append(CheckResult(
                name="Not Already Orchestrated",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="Project appears to already have orchestration system installed",
                details=[f"Found: {p.relative_to(self.project_dir)}" for p in found] + [
                    "",
                    "If you want to reinstall:",
                    "1. Run: python orchestration/uninstall.py",
                    "2. Then retry onboarding"
                ]
            ))
        else:
            self.results.append(CheckResult(
                name="Not Already Orchestrated",
                level=CheckLevel.CRITICAL,
                passed=True,
                message="Project is not already orchestrated"
            ))

    def _check_directory_conflicts(self):
        """Check for directories that would conflict with orchestration structure"""
        potential_conflicts = [
            ("components", "Will be used for isolated component development"),
            ("contracts", "Will be used for API contracts between components"),
            ("shared-libs", "Will be used for shared libraries"),
            ("specifications", "Will be used for specification documents")
        ]

        conflicts = []
        for dir_name, purpose in potential_conflicts:
            dir_path = self.project_dir / dir_name
            if dir_path.exists():
                # Check if it has content
                try:
                    contents = list(dir_path.iterdir())
                    if contents:
                        conflicts.append(f"{dir_name}/ ({len(contents)} items) - {purpose}")
                except PermissionError:
                    conflicts.append(f"{dir_name}/ (permission denied) - {purpose}")

        if conflicts:
            self.results.append(CheckResult(
                name="Directory Conflicts",
                level=CheckLevel.WARNING,
                passed=False,
                message=f"Found {len(conflicts)} directories that may conflict with orchestration structure",
                details=conflicts + [
                    "",
                    "These directories will be preserved during onboarding.",
                    "Onboarding process will merge existing content with orchestration structure."
                ]
            ))
        else:
            self.results.append(CheckResult(
                name="Directory Conflicts",
                level=CheckLevel.INFO,
                passed=True,
                message="No directory conflicts detected"
            ))

    def _check_file_conflicts(self):
        """Check for files that would conflict with orchestration system"""
        potential_conflicts = [
            "CLAUDE.md",
            "orchestration-config.json",
            ".claude/commands/orchestrate.md",
            ".claude/commands/orch-extract-features.md"
        ]

        conflicts = []
        for file_path_str in potential_conflicts:
            file_path = self.project_dir / file_path_str
            if file_path.exists():
                conflicts.append(file_path_str)

        if conflicts:
            self.results.append(CheckResult(
                name="File Conflicts",
                level=CheckLevel.WARNING,
                passed=False,
                message=f"Found {len(conflicts)} files that may conflict",
                details=conflicts + [
                    "",
                    "These files will be backed up with .backup suffix",
                    "You can review and merge manually after onboarding"
                ]
            ))
        else:
            self.results.append(CheckResult(
                name="File Conflicts",
                level=CheckLevel.INFO,
                passed=True,
                message="No file conflicts detected"
            ))

    def _check_has_source_code(self):
        """Check that project has actual source code"""
        # Common source code extensions
        extensions = ['.py', '.js', '.ts', '.go', '.rs', '.java', '.cpp', '.c', '.h']

        source_files = []
        for ext in extensions:
            source_files.extend(self.project_dir.rglob(f'*{ext}'))
            if len(source_files) >= 10:  # Enough to confirm
                break

        # Exclude common non-source directories
        excluded_dirs = {'.git', 'node_modules', 'venv', '.venv', '__pycache__', 'dist', 'build'}
        source_files = [
            f for f in source_files
            if not any(excluded in f.parts for excluded in excluded_dirs)
        ]

        if not source_files:
            self.results.append(CheckResult(
                name="Has Source Code",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="No source code files detected",
                details=[
                    f"Searched for: {', '.join(extensions)}",
                    "Project must have source code to onboard"
                ]
            ))
        else:
            self.results.append(CheckResult(
                name="Has Source Code",
                level=CheckLevel.CRITICAL,
                passed=True,
                message=f"Found {len(source_files)} source files"
            ))

    def _check_project_type(self):
        """Detect project type (Python, JavaScript, etc.)"""
        indicators = {
            'Python': ['setup.py', 'pyproject.toml', 'requirements.txt', 'Pipfile'],
            'JavaScript': ['package.json', 'yarn.lock', 'package-lock.json'],
            'TypeScript': ['tsconfig.json'],
            'Go': ['go.mod', 'go.sum'],
            'Rust': ['Cargo.toml', 'Cargo.lock'],
            'Java': ['pom.xml', 'build.gradle']
        }

        detected = []
        for lang, files in indicators.items():
            for file_name in files:
                if (self.project_dir / file_name).exists():
                    detected.append(lang)
                    break

        if detected:
            self.results.append(CheckResult(
                name="Project Type",
                level=CheckLevel.INFO,
                passed=True,
                message=f"Detected project type(s): {', '.join(detected)}",
                details=[
                    "Orchestration system supports multi-language projects",
                    "Structure analyzer will provide detailed analysis"
                ]
            ))
        else:
            self.results.append(CheckResult(
                name="Project Type",
                level=CheckLevel.WARNING,
                passed=False,
                message="Could not detect project type",
                details=[
                    "No standard project files found",
                    "Onboarding will proceed but may require manual configuration"
                ]
            ))

    def _check_required_tools(self):
        """Check for required external tools"""
        tools = {
            'git': 'Required for history preservation',
            'pre-commit': 'Required for hook management (will be installed if missing)'
        }

        missing = []
        for tool, purpose in tools.items():
            try:
                subprocess.run(
                    [tool, '--version'],
                    capture_output=True,
                    check=True
                )
            except (subprocess.CalledProcessError, FileNotFoundError):
                if tool == 'pre-commit':
                    # pre-commit is installable via pip, not critical
                    missing.append(f"{tool} - {purpose} (will auto-install)")
                else:
                    missing.append(f"{tool} - {purpose}")

        if missing:
            has_critical = any('will auto-install' not in m for m in missing)
            self.results.append(CheckResult(
                name="Required Tools",
                level=CheckLevel.CRITICAL if has_critical else CheckLevel.WARNING,
                passed=False,
                message=f"Missing {len(missing)} required tool(s)",
                details=missing
            ))
        else:
            self.results.append(CheckResult(
                name="Required Tools",
                level=CheckLevel.INFO,
                passed=True,
                message="All required tools available"
            ))

    def _run_fix(self, command: str):
        """Execute a fix command"""
        try:
            subprocess.run(
                command,
                shell=True,
                cwd=self.project_dir,
                check=True
            )
            print(f"✓ Auto-fix applied: {command}")
        except subprocess.CalledProcessError as e:
            print(f"✗ Auto-fix failed: {command}")
            print(f"  Error: {e}")

    def print_report(self):
        """Print formatted preflight check report"""
        # Group by level
        critical = [r for r in self.results if r.level == CheckLevel.CRITICAL]
        warnings = [r for r in self.results if r.level == CheckLevel.WARNING]
        info = [r for r in self.results if r.level == CheckLevel.INFO]

        print()
        print("=" * 70)
        print("  PREFLIGHT CHECKS - EXISTING PROJECT ONBOARDING")
        print("=" * 70)
        print()

        # Critical checks
        if critical:
            print("🔴 CRITICAL CHECKS")
            print("-" * 70)
            for result in critical:
                status = "✅ PASS" if result.passed else "❌ FAIL"
                print(f"{status} | {result.name}")
                print(f"       {result.message}")
                if result.details:
                    for detail in result.details:
                        print(f"       {detail}")
                if not result.passed and result.fix_available:
                    print(f"       Fix: {result.fix_command}")
                print()

        # Warnings
        if warnings:
            print("🟡 WARNINGS")
            print("-" * 70)
            for result in warnings:
                status = "✅ OK" if result.passed else "⚠️  REVIEW"
                print(f"{status} | {result.name}")
                print(f"       {result.message}")
                if result.details:
                    for detail in result.details[:5]:  # Show first 5
                        print(f"       {detail}")
                    if len(result.details) > 5:
                        print(f"       ... and {len(result.details) - 5} more")
                print()

        # Info
        if info:
            print("ℹ️  INFORMATION")
            print("-" * 70)
            for result in info:
                print(f"✅ {result.name}: {result.message}")
            print()

        # Summary
        critical_failures = sum(1 for r in critical if not r.passed)
        warning_count = sum(1 for r in warnings if not r.passed)

        print("=" * 70)
        print("  SUMMARY")
        print("=" * 70)
        print(f"Critical Failures: {critical_failures}")
        print(f"Warnings: {warning_count}")
        print(f"Total Checks: {len(self.results)}")
        print()

        if critical_failures > 0:
            print("❌ CANNOT PROCEED")
            print("   Fix critical issues before onboarding")
            print()
            if any(r.fix_available for r in critical if not r.passed):
                print("   Run with --fix to attempt automatic fixes:")
                print(f"   python {Path(__file__).name} --fix {self.project_dir}")
        elif warning_count > 0:
            print("⚠️  CAN PROCEED WITH CAUTION")
            print("   Review warnings above before continuing")
        else:
            print("✅ READY FOR ONBOARDING")
            print("   All checks passed!")
        print()

    def export_json(self) -> str:
        """Export results as JSON"""
        data = {
            'project_dir': str(self.project_dir),
            'summary': {
                'total_checks': len(self.results),
                'critical_failures': sum(
                    1 for r in self.results
                    if r.level == CheckLevel.CRITICAL and not r.passed
                ),
                'warnings': sum(
                    1 for r in self.results
                    if r.level == CheckLevel.WARNING and not r.passed
                ),
                'can_proceed': sum(
                    1 for r in self.results
                    if r.level == CheckLevel.CRITICAL and not r.passed
                ) == 0
            },
            'checks': [
                {
                    'name': r.name,
                    'level': r.level.value,
                    'passed': r.passed,
                    'message': r.message,
                    'details': r.details,
                    'fix_available': r.fix_available,
                    'fix_command': r.fix_command
                }
                for r in self.results
            ]
        }
        return json.dumps(data, indent=2)


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description="Preflight checks for existing project onboarding"
    )
    parser.add_argument(
        'project_dir',
        nargs='?',
        default='.',
        help='Project directory to check (default: current directory)'
    )
    parser.add_argument(
        '--fix',
        action='store_true',
        help='Attempt to automatically fix issues'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output results as JSON'
    )
    parser.add_argument(
        '--mode',
        action='store_true',
        help='Detect and display onboarding mode'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()

    # Mode detection only
    if args.mode:
        checker = PreflightChecker(project_dir, auto_fix=False)
        mode = checker.detect_onboarding_mode()
        if mode:
            print(f"Detected Mode: {mode}")
            print()
            print("Mode Descriptions:")
            print("  fresh    - New onboarding on clean or existing project")
            print("  resume   - Continue interrupted/failed onboarding")
            print("  upgrade  - Upgrade old orchestration to current version")
            print("  verify   - Verify compliance without making changes")
            sys.exit(0)
        else:
            print("Error: Unable to detect mode", file=sys.stderr)
            sys.exit(1)

    # Run checks
    checker = PreflightChecker(project_dir, auto_fix=args.fix)
    can_proceed, critical_failures, warnings = checker.run_all_checks()

    # Output results
    if args.json:
        print(checker.export_json())
    else:
        checker.print_report()

        # Also show detected mode
        mode = checker.detect_onboarding_mode()
        if mode:
            print()
            print("Detected Onboarding Mode: " + mode.upper())
            print()

    # Exit code
    if critical_failures > 0:
        sys.exit(1)
    elif warnings > 0:
        sys.exit(2)
    else:
        sys.exit(0)


if __name__ == '__main__':
    main()
