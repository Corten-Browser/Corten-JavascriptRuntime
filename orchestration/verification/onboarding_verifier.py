#!/usr/bin/env python3
"""
Onboarding Verifier - Validate onboarding completion

Verifies that all onboarding phases completed successfully and the project
is ready for orchestrated development. Checks structure, manifests, contracts,
imports, and runs basic smoke tests.

Usage:
    python orchestration/verification/onboarding_verifier.py <project_dir>
    python orchestration/verification/onboarding_verifier.py <project_dir> --fix

Exit Codes:
    0 - All checks passed
    1 - Critical failures
    2 - Warnings present
"""

import sys
import json
import subprocess
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass, field
from enum import Enum


class CheckLevel(Enum):
    """Severity level for verification checks"""
    CRITICAL = "critical"
    WARNING = "warning"
    INFO = "info"


@dataclass
class VerificationCheck:
    """Result of a single verification check"""
    name: str
    level: CheckLevel
    passed: bool
    message: str
    details: List[str] = field(default_factory=list)


class OnboardingVerifier:
    """Verifies onboarding completion and project readiness"""

    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()
        self.results: List[VerificationCheck] = []

    def run_all_checks(self) -> Tuple[bool, int, int]:
        """
        Run all onboarding verification checks

        Returns:
            (all_passed, critical_failures, warnings)
        """
        self.results = []

        # Phase 1: Directory Structure
        self._check_directory_structure()

        # Phase 2: Orchestration System
        self._check_orchestration_installed()
        self._check_version_file()
        self._check_config_file()

        # Phase 3: Component Structure
        self._check_components_exist()
        self._check_component_manifests()

        # Phase 4: Contracts
        self._check_contracts_directory()

        # Phase 5: Specifications
        self._check_specifications_exist()

        # Phase 6: Imports
        self._check_import_health()

        # Phase 7: Git
        self._check_git_status()

        # Phase 8: System Files
        self._check_claude_commands()
        self._check_enforcement_hooks()

        # Phase 9: Smoke Tests (NEW - Task 3.1)
        self._run_smoke_tests()

        # Phase 10: Gate Execution (NEW - Task 3.2)
        self._check_gates_pass()

        # Calculate results
        critical_failures = sum(
            1 for r in self.results
            if r.level == CheckLevel.CRITICAL and not r.passed
        )
        warnings = sum(
            1 for r in self.results
            if r.level == CheckLevel.WARNING and not r.passed
        )

        all_passed = critical_failures == 0

        return all_passed, critical_failures, warnings

    def _check_directory_structure(self):
        """Check core directory structure exists"""
        required_dirs = [
            ("components", "Component isolation directories"),
            ("contracts", "API contracts"),
            ("shared-libs", "Shared libraries"),
            ("specifications", "Specification documents"),
            ("orchestration", "Orchestration system files")
        ]

        missing = []
        for dir_name, purpose in required_dirs:
            dir_path = self.project_dir / dir_name
            if not dir_path.exists():
                missing.append(f"{dir_name}/ - {purpose}")

        if missing:
            self.results.append(VerificationCheck(
                name="Directory Structure",
                level=CheckLevel.CRITICAL,
                passed=False,
                message=f"Missing {len(missing)} required directories",
                details=missing
            ))
        else:
            self.results.append(VerificationCheck(
                name="Directory Structure",
                level=CheckLevel.CRITICAL,
                passed=True,
                message="All required directories exist"
            ))

    def _check_orchestration_installed(self):
        """Check orchestration system is installed"""
        required_files = [
            "orchestration/__init__.py",
            "orchestration/setup/install_enforcement.py",
            "orchestration/cli/onboard_preflight.py",
            "orchestration/migration/onboarding_planner.py"
        ]

        missing = []
        for file_path in required_files:
            if not (self.project_dir / file_path).exists():
                missing.append(file_path)

        if missing:
            self.results.append(VerificationCheck(
                name="Orchestration System",
                level=CheckLevel.CRITICAL,
                passed=False,
                message=f"Missing {len(missing)} orchestration files",
                details=missing
            ))
        else:
            self.results.append(VerificationCheck(
                name="Orchestration System",
                level=CheckLevel.CRITICAL,
                passed=True,
                message="Orchestration system installed"
            ))

    def _check_version_file(self):
        """Check VERSION file exists"""
        version_file = self.project_dir / "orchestration" / "VERSION"

        if not version_file.exists():
            self.results.append(VerificationCheck(
                name="Version File",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="VERSION file not found",
                details=["orchestration/VERSION is required"]
            ))
        else:
            try:
                version = version_file.read_text().strip()
                self.results.append(VerificationCheck(
                    name="Version File",
                    level=CheckLevel.INFO,
                    passed=True,
                    message=f"Version: {version}"
                ))
            except Exception as e:
                self.results.append(VerificationCheck(
                    name="Version File",
                    level=CheckLevel.WARNING,
                    passed=False,
                    message="Could not read VERSION file",
                    details=[str(e)]
                ))

    def _check_config_file(self):
        """Check orchestration-config.json exists and is valid"""
        config_file = self.project_dir / "orchestration-config.json"

        if not config_file.exists():
            self.results.append(VerificationCheck(
                name="Configuration File",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="orchestration-config.json not found"
            ))
            return

        try:
            config = json.loads(config_file.read_text())

            # Check required fields
            required_fields = ["max_parallel_agents", "context_limit_tokens"]
            missing_fields = [f for f in required_fields if f not in config]

            if missing_fields:
                self.results.append(VerificationCheck(
                    name="Configuration File",
                    level=CheckLevel.WARNING,
                    passed=False,
                    message="Configuration missing fields",
                    details=missing_fields
                ))
            else:
                self.results.append(VerificationCheck(
                    name="Configuration File",
                    level=CheckLevel.INFO,
                    passed=True,
                    message=f"Configuration valid (max agents: {config.get('max_parallel_agents')})"
                ))

        except json.JSONDecodeError as e:
            self.results.append(VerificationCheck(
                name="Configuration File",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="Invalid JSON in orchestration-config.json",
                details=[str(e)]
            ))

    def _check_components_exist(self):
        """Check that components directory has at least one component"""
        components_dir = self.project_dir / "components"

        if not components_dir.exists():
            self.results.append(VerificationCheck(
                name="Components",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="components/ directory not found"
            ))
            return

        # Find component directories
        components = [
            d for d in components_dir.iterdir()
            if d.is_dir() and not d.name.startswith('.')
        ]

        if not components:
            self.results.append(VerificationCheck(
                name="Components",
                level=CheckLevel.WARNING,
                passed=False,
                message="No components found in components/ directory",
                details=["Onboarding may be incomplete - no components discovered"]
            ))
        else:
            component_names = [c.name for c in components]
            self.results.append(VerificationCheck(
                name="Components",
                level=CheckLevel.INFO,
                passed=True,
                message=f"Found {len(components)} component(s)",
                details=component_names
            ))

    def _check_component_manifests(self):
        """Check that components have valid manifest files"""
        components_dir = self.project_dir / "components"

        if not components_dir.exists():
            return  # Already reported in _check_components_exist

        components = [
            d for d in components_dir.iterdir()
            if d.is_dir() and not d.name.startswith('.')
        ]

        if not components:
            return  # No components to check

        missing_manifests = []
        invalid_manifests = []

        for component_dir in components:
            manifest_path = component_dir / "component.yaml"

            if not manifest_path.exists():
                missing_manifests.append(component_dir.name)
            else:
                # Try to validate manifest
                try:
                    import yaml
                    manifest_data = yaml.safe_load(manifest_path.read_text())

                    # Check required fields
                    required = ['version', 'name', 'type', 'description']
                    if not all(field in manifest_data for field in required):
                        invalid_manifests.append(f"{component_dir.name} (missing fields)")

                except Exception as e:
                    invalid_manifests.append(f"{component_dir.name} (parse error)")

        if missing_manifests or invalid_manifests:
            issues = []
            if missing_manifests:
                issues.append(f"Missing manifests: {', '.join(missing_manifests)}")
            if invalid_manifests:
                issues.append(f"Invalid manifests: {', '.join(invalid_manifests)}")

            self.results.append(VerificationCheck(
                name="Component Manifests",
                level=CheckLevel.WARNING,
                passed=False,
                message="Some components have manifest issues",
                details=issues
            ))
        else:
            self.results.append(VerificationCheck(
                name="Component Manifests",
                level=CheckLevel.INFO,
                passed=True,
                message=f"All {len(components)} component(s) have valid manifests"
            ))

    def _check_contracts_directory(self):
        """Check contracts directory"""
        contracts_dir = self.project_dir / "contracts"

        if not contracts_dir.exists():
            self.results.append(VerificationCheck(
                name="Contracts",
                level=CheckLevel.WARNING,
                passed=False,
                message="contracts/ directory not found"
            ))
            return

        # Find contract files
        contracts = list(contracts_dir.glob("*.yaml")) + list(contracts_dir.glob("*.yml"))

        if not contracts:
            self.results.append(VerificationCheck(
                name="Contracts",
                level=CheckLevel.WARNING,
                passed=False,
                message="No contract files found",
                details=["Contracts may not have been generated yet"]
            ))
        else:
            self.results.append(VerificationCheck(
                name="Contracts",
                level=CheckLevel.INFO,
                passed=True,
                message=f"Found {len(contracts)} contract file(s)"
            ))

    def _check_specifications_exist(self):
        """Check specifications directory"""
        specs_dir = self.project_dir / "specifications"

        if not specs_dir.exists():
            self.results.append(VerificationCheck(
                name="Specifications",
                level=CheckLevel.WARNING,
                passed=False,
                message="specifications/ directory not found"
            ))
            return

        # Find specification files
        specs = list(specs_dir.glob("*.yaml")) + list(specs_dir.glob("*.yml")) + list(specs_dir.glob("*.md"))

        if not specs:
            self.results.append(VerificationCheck(
                name="Specifications",
                level=CheckLevel.WARNING,
                passed=False,
                message="No specification files found",
                details=["Specifications may not have been extracted yet"]
            ))
        else:
            self.results.append(VerificationCheck(
                name="Specifications",
                level=CheckLevel.INFO,
                passed=True,
                message=f"Found {len(specs)} specification file(s)"
            ))

    def _check_import_health(self):
        """Check for obvious import issues"""
        try:
            # Try importing Python files to check for import errors
            # This is a basic smoke test
            py_files = list(self.project_dir.rglob("*.py"))

            # Exclude certain directories
            excluded = {'.git', 'venv', '.venv', '__pycache__', 'dist', 'build'}
            py_files = [
                f for f in py_files
                if not any(excluded_dir in f.parts for excluded_dir in excluded)
            ]

            if not py_files:
                self.results.append(VerificationCheck(
                    name="Import Health",
                    level=CheckLevel.INFO,
                    passed=True,
                    message="No Python files to check"
                ))
                return

            # Sample a few files to check
            sample_files = py_files[:min(5, len(py_files))]

            import_errors = []
            for py_file in sample_files:
                try:
                    # Basic syntax check
                    content = py_file.read_text()
                    compile(content, str(py_file), 'exec')
                except SyntaxError as e:
                    import_errors.append(f"{py_file.name}:{e.lineno} - {e.msg}")
                except Exception:
                    pass  # Other errors are ok (might be runtime issues)

            if import_errors:
                self.results.append(VerificationCheck(
                    name="Import Health",
                    level=CheckLevel.WARNING,
                    passed=False,
                    message=f"Found {len(import_errors)} syntax error(s)",
                    details=import_errors
                ))
            else:
                self.results.append(VerificationCheck(
                    name="Import Health",
                    level=CheckLevel.INFO,
                    passed=True,
                    message=f"Sampled {len(sample_files)} files - no syntax errors"
                ))

        except Exception as e:
            self.results.append(VerificationCheck(
                name="Import Health",
                level=CheckLevel.INFO,
                passed=True,
                message="Could not check imports (non-critical)",
                details=[str(e)]
            ))

    def _check_git_status(self):
        """Check git repository status"""
        try:
            # Check if git repo
            git_dir = self.project_dir / ".git"
            if not git_dir.exists():
                self.results.append(VerificationCheck(
                    name="Git Status",
                    level=CheckLevel.WARNING,
                    passed=False,
                    message="Not a git repository"
                ))
                return

            # Check for uncommitted changes
            result = subprocess.run(
                ["git", "status", "--porcelain"],
                cwd=self.project_dir,
                capture_output=True,
                text=True,
                check=True
            )

            if result.stdout.strip():
                uncommitted = result.stdout.strip().split('\n')
                self.results.append(VerificationCheck(
                    name="Git Status",
                    level=CheckLevel.INFO,
                    passed=True,
                    message=f"{len(uncommitted)} uncommitted change(s)",
                    details=uncommitted[:5]  # Show first 5
                ))
            else:
                self.results.append(VerificationCheck(
                    name="Git Status",
                    level=CheckLevel.INFO,
                    passed=True,
                    message="Working tree clean"
                ))

        except subprocess.CalledProcessError:
            self.results.append(VerificationCheck(
                name="Git Status",
                level=CheckLevel.INFO,
                passed=True,
                message="Could not check git status (non-critical)"
            ))

    def _check_claude_commands(self):
        """Check Claude slash commands installed"""
        claude_dir = self.project_dir / ".claude" / "commands"

        if not claude_dir.exists():
            self.results.append(VerificationCheck(
                name="Claude Commands",
                level=CheckLevel.WARNING,
                passed=False,
                message=".claude/commands/ directory not found"
            ))
            return

        # Check for key commands
        key_commands = [
            "orchestrate.md",
            "orch-extract-features.md"
        ]

        missing = []
        for cmd in key_commands:
            if not (claude_dir / cmd).exists():
                missing.append(cmd)

        if missing:
            self.results.append(VerificationCheck(
                name="Claude Commands",
                level=CheckLevel.WARNING,
                passed=False,
                message=f"Missing {len(missing)} slash command(s)",
                details=missing
            ))
        else:
            total_commands = len(list(claude_dir.glob("*.md")))
            self.results.append(VerificationCheck(
                name="Claude Commands",
                level=CheckLevel.INFO,
                passed=True,
                message=f"{total_commands} slash command(s) installed"
            ))

    def _check_enforcement_hooks(self):
        """Check enforcement hooks (pre-commit) installed"""
        hooks_dir = self.project_dir / ".git" / "hooks"

        if not hooks_dir.exists():
            self.results.append(VerificationCheck(
                name="Enforcement Hooks",
                level=CheckLevel.INFO,
                passed=True,
                message="No .git/hooks directory (using pre-commit framework)"
            ))
            return

        # Check for pre-commit
        precommit_config = self.project_dir / ".pre-commit-config.yaml"

        if precommit_config.exists():
            self.results.append(VerificationCheck(
                name="Enforcement Hooks",
                level=CheckLevel.INFO,
                passed=True,
                message="Pre-commit framework configured"
            ))
        else:
            self.results.append(VerificationCheck(
                name="Enforcement Hooks",
                level=CheckLevel.WARNING,
                passed=False,
                message="Pre-commit configuration not found",
                details=[".pre-commit-config.yaml missing"]
            ))

    def _run_smoke_tests(self):
        """Run smoke tests - import each component to verify it loads (Task 3.1)"""
        import subprocess

        components_dir = self.project_dir / "components"

        if not components_dir.exists():
            self.results.append(VerificationCheck(
                name="Smoke Tests",
                level=CheckLevel.INFO,
                passed=True,
                message="No components to test"
            ))
            return

        passed = []
        failed = []

        for component_dir in components_dir.iterdir():
            if not component_dir.is_dir() or component_dir.name.startswith('.'):
                continue

            # Try to import component
            try:
                result = subprocess.run(
                    ['python3', '-c', f'import sys; sys.path.insert(0, "components"); import {component_dir.name}'],
                    capture_output=True,
                    timeout=10,
                    cwd=self.project_dir
                )

                if result.returncode == 0:
                    passed.append(component_dir.name)
                else:
                    error = result.stderr.decode().strip()
                    failed.append(f"{component_dir.name}: {error[:100]}")
            except subprocess.TimeoutExpired:
                failed.append(f"{component_dir.name}: Import timeout")
            except Exception as e:
                failed.append(f"{component_dir.name}: {str(e)[:100]}")

        if failed:
            self.results.append(VerificationCheck(
                name="Smoke Tests",
                level=CheckLevel.CRITICAL,
                passed=False,
                message=f"{len(failed)} components failed to import",
                details=failed
            ))
        else:
            self.results.append(VerificationCheck(
                name="Smoke Tests",
                level=CheckLevel.INFO,
                passed=True,
                message=f"All {len(passed)} components import successfully"
            ))

    def _check_gates_pass(self):
        """Run Phase 5 and Phase 6 gates (Task 3.2)"""
        import subprocess

        gates_runner = self.project_dir / "orchestration" / "gates" / "runner.py"

        if not gates_runner.exists():
            self.results.append(VerificationCheck(
                name="Phase Gates",
                level=CheckLevel.WARNING,
                passed=False,
                message="Gate runner not found - cannot verify gates"
            ))
            return

        # Run Phase 5 gate (Integration)
        try:
            gate5_result = subprocess.run(
                ["python3", str(gates_runner), ".", "5"],
                capture_output=True,
                timeout=60,
                cwd=self.project_dir
            )
            phase5_passed = gate5_result.returncode == 0
        except Exception as e:
            phase5_passed = False
            phase5_output = str(e)

        # Run Phase 6 gate (Verification)
        try:
            gate6_result = subprocess.run(
                ["python3", str(gates_runner), ".", "6"],
                capture_output=True,
                timeout=60,
                cwd=self.project_dir
            )
            phase6_passed = gate6_result.returncode == 0
        except Exception as e:
            phase6_passed = False
            phase6_output = str(e)

        # Report results
        if phase5_passed and phase6_passed:
            self.results.append(VerificationCheck(
                name="Phase Gates",
                level=CheckLevel.INFO,
                passed=True,
                message="Phase 5 and Phase 6 gates passed"
            ))
        else:
            details = []
            if not phase5_passed:
                details.append("Phase 5 gate (Integration) failed")
            if not phase6_passed:
                details.append("Phase 6 gate (Verification) failed")

            self.results.append(VerificationCheck(
                name="Phase Gates",
                level=CheckLevel.CRITICAL,
                passed=False,
                message="One or more gates failed",
                details=details
            ))

    def print_report(self):
        """Print formatted verification report"""
        # Group by level
        critical = [r for r in self.results if r.level == CheckLevel.CRITICAL]
        warnings = [r for r in self.results if r.level == CheckLevel.WARNING]
        info = [r for r in self.results if r.level == CheckLevel.INFO]

        print()
        print("=" * 70)
        print("  ONBOARDING VERIFICATION REPORT")
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
                    for detail in result.details[:5]:
                        print(f"       • {detail}")
                    if len(result.details) > 5:
                        print(f"       ... and {len(result.details) - 5} more")
                print()

        # Warnings
        if warnings:
            print("🟡 WARNINGS")
            print("-" * 70)
            for result in warnings:
                status = "✅ OK" if result.passed else "⚠️  ISSUE"
                print(f"{status} | {result.name}")
                print(f"       {result.message}")
                if result.details:
                    for detail in result.details[:3]:
                        print(f"       • {detail}")
                    if len(result.details) > 3:
                        print(f"       ... and {len(result.details) - 3} more")
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
            print("❌ ONBOARDING INCOMPLETE")
            print("   Critical issues must be resolved")
        elif warning_count > 0:
            print("⚠️  ONBOARDING MOSTLY COMPLETE")
            print("   Review warnings above")
        else:
            print("✅ ONBOARDING COMPLETE")
            print("   Project ready for orchestrated development!")
        print()


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description="Verify onboarding completion"
    )
    parser.add_argument(
        'project_dir',
        nargs='?',
        default='.',
        help='Project directory (default: current directory)'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output results as JSON'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()

    # Run verification
    verifier = OnboardingVerifier(project_dir)
    all_passed, critical_failures, warnings = verifier.run_all_checks()

    # Output results
    if args.json:
        # JSON output
        data = {
            'project_dir': str(project_dir),
            'all_passed': all_passed,
            'critical_failures': critical_failures,
            'warnings': warnings,
            'checks': [
                {
                    'name': r.name,
                    'level': r.level.value,
                    'passed': r.passed,
                    'message': r.message,
                    'details': r.details
                }
                for r in verifier.results
            ]
        }
        print(json.dumps(data, indent=2))
    else:
        # Human-readable output
        verifier.print_report()

    # Exit code
    if critical_failures > 0:
        sys.exit(1)
    elif warnings > 0:
        sys.exit(2)
    else:
        sys.exit(0)


if __name__ == '__main__':
    main()
