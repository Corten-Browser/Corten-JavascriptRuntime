#!/usr/bin/env python3
"""
Verify orchestration system installation in user projects.

This module validates that the orchestration system is properly installed
in a user's project directory. It checks for required files, directories,
and configuration without expecting orchestration repository structure.

Usage:
    python orchestration/setup/verify_installation.py
    python orchestration/setup/verify_installation.py --verbose
    python orchestration/setup/verify_installation.py --fix

Exit codes:
    0 - Installation verified successfully
    1 - Some checks failed (warnings)
    2 - Critical errors (installation broken)
"""

import os
import sys
import json
from pathlib import Path
from typing import List, Tuple, Optional


class Colors:
    """ANSI color codes for terminal output."""
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

    @classmethod
    def disable(cls):
        """Disable colors (for non-TTY output)."""
        cls.GREEN = cls.YELLOW = cls.RED = cls.BLUE = cls.NC = ''


class CheckResult:
    """Result of a check operation."""

    def __init__(self, passed: bool, message: str, severity: str = 'error'):
        self.passed = passed
        self.message = message
        self.severity = severity  # 'error', 'warning', 'info'


class InstallationVerifier:
    """Verify orchestration installation in user projects."""

    def __init__(self, verbose: bool = False, fix: bool = False):
        self.verbose = verbose
        self.fix = fix
        self.results: List[CheckResult] = []

        # Disable colors if not in TTY
        if not sys.stdout.isatty():
            Colors.disable()

    def check_required_directories(self) -> CheckResult:
        """Check that required directories exist."""
        required_dirs = [
            'orchestration',
            'orchestration/cli',
            'orchestration/config',
            'orchestration/core',
            'orchestration/hooks',
            'orchestration/setup',
            '.claude',
            '.claude/commands',
        ]

        missing = []
        for dir_path in required_dirs:
            if not Path(dir_path).is_dir():
                missing.append(dir_path)

        if missing:
            msg = f"Missing {len(missing)} required directory(ies):\n"
            msg += "\n".join(f"  - {d}/" for d in missing)

            if self.fix:
                for dir_path in missing:
                    Path(dir_path).mkdir(parents=True, exist_ok=True)
                    msg += f"\n  ✅ Created {dir_path}/"
                return CheckResult(True, msg + "\n  Fixed!", 'info')

            return CheckResult(False, msg, 'error')

        return CheckResult(True, f"All {len(required_dirs)} required directories present")

    def check_core_files(self) -> CheckResult:
        """Check that core orchestration files exist."""
        core_files = [
            'orchestration/cli/git_retry.py',
            'orchestration/cli/git_status.py',
            'orchestration/config/orchestration.json',
            'orchestration/hooks/pre_commit_naming.py',
            'orchestration/hooks/pre_commit_enforcement.py',
            'orchestration/hooks/pre_commit_completion_blocker.py',
            'orchestration/setup/install_precommit.py',
            'orchestration/setup/precommit_config.py',
            '.claude/commands/orchestrate.md',
        ]

        missing = []
        for file_path in core_files:
            if not Path(file_path).exists():
                missing.append(file_path)

        if missing:
            msg = f"Missing {len(missing)} core file(s):\n"
            msg += "\n".join(f"  - {f}" for f in missing)
            msg += "\n\nInstallation may be incomplete or corrupted."
            return CheckResult(False, msg, 'error')

        return CheckResult(True, f"All {len(core_files)} core files present")

    def check_config_valid(self) -> CheckResult:
        """Check that orchestration config is valid JSON."""
        config_path = Path('orchestration/config/orchestration.json')

        if not config_path.exists():
            return CheckResult(False, "Config file not found", 'error')

        try:
            with open(config_path) as f:
                config = json.load(f)

            # Verify required keys
            required_keys = ['orchestration', 'context_limits', 'quality_standards']
            missing_keys = [k for k in required_keys if k not in config]

            if missing_keys:
                msg = f"Config missing required keys: {', '.join(missing_keys)}"
                return CheckResult(False, msg, 'warning')

            return CheckResult(True, "Configuration file is valid")

        except json.JSONDecodeError as e:
            return CheckResult(False, f"Config file invalid JSON: {e}", 'error')

    def check_version_file(self) -> CheckResult:
        """Check that VERSION file exists and is valid."""
        version_path = Path('orchestration/VERSION')

        if not version_path.exists():
            return CheckResult(
                False,
                "VERSION file not found - unable to determine orchestration version",
                'warning'
            )

        try:
            with open(version_path) as f:
                version = f.read().strip()

            # Basic semantic version validation (X.Y.Z)
            parts = version.split('.')
            if len(parts) != 3 or not all(p.isdigit() for p in parts):
                return CheckResult(
                    False,
                    f"Invalid version format: {version} (expected X.Y.Z)",
                    'warning'
                )

            return CheckResult(True, f"Version: {version}")

        except Exception as e:
            return CheckResult(False, f"Failed to read VERSION: {e}", 'warning')

    def check_git_repository(self) -> CheckResult:
        """Check that we're in a git repository."""
        if not Path('.git').is_dir():
            msg = "Not in a git repository (recommended for orchestration)"

            if self.fix:
                try:
                    import subprocess
                    subprocess.run(['git', 'init'], check=True, capture_output=True)
                    msg += "\n  ✅ Initialized git repository"
                    return CheckResult(True, msg, 'info')
                except Exception as e:
                    msg += f"\n  ❌ Failed to initialize: {e}"
                    return CheckResult(False, msg, 'warning')

            return CheckResult(False, msg, 'warning')

        return CheckResult(True, "Git repository present")

    def check_precommit_configured(self) -> CheckResult:
        """Check if pre-commit hooks are configured."""
        config_path = Path('.pre-commit-config.yaml')

        if not config_path.exists():
            return CheckResult(
                False,
                "Pre-commit config not found (run migration script to set up)",
                'warning'
            )

        try:
            import yaml
            with open(config_path) as f:
                config = yaml.safe_load(f)

            # Check for orchestration hooks
            has_orch_hooks = False
            for repo in config.get('repos', []):
                if repo.get('repo') == 'local':
                    for hook in repo.get('hooks', []):
                        if hook.get('id', '').startswith('orchestration-'):
                            has_orch_hooks = True
                            break

            if not has_orch_hooks:
                return CheckResult(
                    False,
                    "Pre-commit config exists but orchestration hooks not configured",
                    'warning'
                )

            # Check if hooks are installed
            precommit_hook = Path('.git/hooks/pre-commit')
            if not precommit_hook.exists():
                return CheckResult(
                    False,
                    "Pre-commit configured but hooks not installed (run: pre-commit install)",
                    'warning'
                )

            return CheckResult(True, "Pre-commit hooks configured and installed")

        except ImportError:
            return CheckResult(
                False,
                "PyYAML not available (cannot validate pre-commit config)",
                'warning'
            )
        except Exception as e:
            return CheckResult(
                False,
                f"Failed to validate pre-commit config: {e}",
                'warning'
            )

    def run_all_checks(self) -> Tuple[int, int, int]:
        """
        Run all checks.

        Returns:
            Tuple of (passed, failed, warnings)
        """
        checks = [
            ("Required Directories", self.check_required_directories),
            ("Core Files", self.check_core_files),
            ("Configuration", self.check_config_valid),
            ("Version File", self.check_version_file),
            ("Git Repository", self.check_git_repository),
            ("Pre-commit Hooks", self.check_precommit_configured),
        ]

        passed = 0
        failed = 0
        warnings = 0

        print(f"{Colors.BLUE}═══════════════════════════════════════════════════════════{Colors.NC}")
        print(f"{Colors.BLUE}  Orchestration Installation Verification{Colors.NC}")
        print(f"{Colors.BLUE}═══════════════════════════════════════════════════════════{Colors.NC}\n")

        for check_name, check_func in checks:
            if self.verbose:
                print(f"Running: {check_name}...", end=' ')

            result = check_func()
            self.results.append(result)

            # Determine icon and color
            if result.passed:
                icon = "✅"
                color = Colors.GREEN
                passed += 1
            elif result.severity == 'warning':
                icon = "⚠️ "
                color = Colors.YELLOW
                warnings += 1
            else:
                icon = "❌"
                color = Colors.RED
                failed += 1

            # Print result
            status = "PASS" if result.passed else "FAIL" if result.severity == 'error' else "WARN"
            print(f"{icon} {color}{check_name:.<40} {status}{Colors.NC}")

            if self.verbose or not result.passed:
                # Print message with indentation
                for line in result.message.split('\n'):
                    print(f"   {line}")

        # Print summary
        print(f"\n{Colors.BLUE}═══════════════════════════════════════════════════════════{Colors.NC}")
        print(f"{Colors.GREEN}Passed: {passed}{Colors.NC}  ", end='')
        if warnings > 0:
            print(f"{Colors.YELLOW}Warnings: {warnings}{Colors.NC}  ", end='')
        if failed > 0:
            print(f"{Colors.RED}Failed: {failed}{Colors.NC}")
        else:
            print()

        return (passed, failed, warnings)

    def print_installation_help(self):
        """Print help for fixing installation issues."""
        print(f"\n{Colors.YELLOW}To repair orchestration installation:{Colors.NC}")
        print("  # Re-run installation script")
        print("  claude-orchestration/scripts/install.sh .")
        print()
        print("  # Or run upgrade script if already installed")
        print("  bash scripts/upgrade.sh")
        print()
        print(f"{Colors.YELLOW}To set up pre-commit hooks:{Colors.NC}")
        print("  bash scripts/setup_precommit_hooks.sh .")


def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Verify orchestration system installation in user project'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Show detailed output'
    )
    parser.add_argument(
        '--fix',
        action='store_true',
        help='Attempt to fix issues automatically'
    )
    args = parser.parse_args()

    verifier = InstallationVerifier(verbose=args.verbose, fix=args.fix)
    passed, failed, warnings = verifier.run_all_checks()

    if failed > 0:
        verifier.print_installation_help()
        return 2

    if warnings > 0:
        print(f"\n{Colors.YELLOW}⚠️  Installation has warnings but should be functional.{Colors.NC}\n")
        return 1

    print(f"\n{Colors.GREEN}✅ Orchestration system properly installed and ready!{Colors.NC}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
