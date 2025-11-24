#!/usr/bin/env python3
"""
Self-check for orchestration system REPOSITORY development.

This module validates the orchestration repository structure during development.
It is NOT for checking user project installations - use verify_installation.py
in the orchestration/setup/ directory for that purpose.

This checks for:
- CLAUDE.md (repository file, not in user projects)
- Optional development files that may have been reorganized
- Template loading in repository context

Usage:
    python orchestration/cli/self_check_repo.py
    python orchestration/cli/self_check_repo.py --verbose
    python orchestration/cli/self_check_repo.py --fix

Exit codes:
    0 - All checks passed
    1 - Some checks failed
    2 - Critical errors (missing required files)
"""

import os
import sys
import json
from pathlib import Path
from typing import List, Tuple, Optional

# Add project root to Python path for imports
# This allows 'from orchestration.core import templates' to work
project_root = Path(__file__).parent.parent.parent
if str(project_root) not in sys.path:
    sys.path.insert(0, str(project_root))


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


class OrchestratonSelfCheck:
    """Self-check for orchestration system installation."""

    def __init__(self, verbose: bool = False, fix: bool = False):
        self.verbose = verbose
        self.fix = fix
        self.results: List[CheckResult] = []

        # Disable colors if not in TTY
        if not sys.stdout.isatty():
            Colors.disable()

    def check_required_files(self) -> CheckResult:
        """Check that all required files exist."""
        required_files = [
            'orchestration/cli/git_retry.py',
            'orchestration/cli/git_status.py',
            'orchestration/core/templates.py',
            'orchestration/config/orchestration.json',
            'CLAUDE.md',
            'orchestration/commands/orchestrate.md',
        ]

        missing = []
        for file_path in required_files:
            if not Path(file_path).exists():
                missing.append(file_path)

        if missing:
            msg = f"Missing {len(missing)} required file(s):\n"
            msg += "\n".join(f"  - {f}" for f in missing)
            return CheckResult(False, msg, 'error')

        return CheckResult(True, f"All {len(required_files)} required files present")

    def check_optional_files(self) -> CheckResult:
        """Check for optional but recommended files."""
        optional_files = [
            'orchestration/context_manager.py',
            'orchestration/component_splitter.py',
            'orchestration/agent_launcher.py',
            'orchestration/version_guard.py',
        ]

        missing = []
        for file_path in optional_files:
            if not Path(file_path).exists():
                missing.append(file_path)

        if missing:
            msg = f"Missing {len(missing)} optional file(s):\n"
            msg += "\n".join(f"  - {f}" for f in missing)
            return CheckResult(False, msg, 'warning')

        return CheckResult(True, "All optional files present")

    def check_directory_structure(self) -> CheckResult:
        """Check that required directories exist."""
        required_dirs = [
            'orchestration',
            'orchestration/commands',
            'components',
            'contracts',
            'shared-libs',
        ]

        missing = []
        for dir_path in required_dirs:
            if not Path(dir_path).is_dir():
                missing.append(dir_path)

        if missing:
            msg = f"Missing {len(missing)} required director(ies):\n"
            msg += "\n".join(f"  - {d}/" for d in missing)

            if self.fix:
                for dir_path in missing:
                    Path(dir_path).mkdir(parents=True, exist_ok=True)
                    msg += f"\n  ✅ Created {dir_path}/"
                return CheckResult(True, msg + "\n  Fixed!", 'info')

            return CheckResult(False, msg, 'error')

        return CheckResult(True, f"All {len(required_dirs)} required directories present")

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

    def check_git_repository(self) -> CheckResult:
        """Check that we're in a git repository."""
        if not Path('.git').is_dir():
            msg = "Not in a git repository"
            if self.fix:
                try:
                    import subprocess
                    subprocess.run(['git', 'init'], check=True, capture_output=True)
                    msg += "\n  ✅ Initialized git repository"
                    return CheckResult(True, msg, 'info')
                except Exception as e:
                    msg += f"\n  ❌ Failed to initialize: {e}"
                    return CheckResult(False, msg, 'error')

            return CheckResult(False, msg, 'warning')

        return CheckResult(True, "Git repository present")

    def check_templates_loaded(self) -> CheckResult:
        """Check that templates are available."""
        try:
            from orchestration.core import templates

            available = templates.get_available_templates()
            if len(available) != 4:
                return CheckResult(
                    False,
                    f"Expected 4 templates, found {len(available)}",
                    'error'
                )

            # Try to load each template
            for template_type in available:
                template = templates.get_template(template_type)
                if not template or len(template) < 100:
                    return CheckResult(
                        False,
                        f"Template '{template_type}' appears empty or invalid",
                        'error'
                    )

            return CheckResult(True, f"All {len(available)} templates loaded successfully")

        except Exception as e:
            return CheckResult(False, f"Failed to load templates: {e}", 'error')

    def run_all_checks(self) -> Tuple[int, int, int]:
        """
        Run all checks.

        Returns:
            Tuple of (passed, failed, warnings)
        """
        checks = [
            ("Required Files", self.check_required_files),
            ("Optional Files", self.check_optional_files),
            ("Directory Structure", self.check_directory_structure),
            ("Configuration", self.check_config_valid),
            ("Git Repository", self.check_git_repository),
            ("Templates", self.check_templates_loaded),
        ]

        passed = 0
        failed = 0
        warnings = 0

        print(f"{Colors.BLUE}═══════════════════════════════════════════════════════════{Colors.NC}")
        print(f"{Colors.BLUE}  Orchestration Repository Self-Check{Colors.NC}")
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
        """Print help for fixing repository issues."""
        print(f"\n{Colors.YELLOW}Note: This check is for orchestration repository development.{Colors.NC}")
        print(f"{Colors.YELLOW}For user project installation verification, use:{Colors.NC}")
        print("  python orchestration/setup/verify_installation.py")
        print()
        print(f"{Colors.YELLOW}To fix repository issues:{Colors.NC}")
        print("  python orchestration/cli/self_check_repo.py --fix")


def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Verify orchestration repository structure (for development)'
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

    checker = OrchestratonSelfCheck(verbose=args.verbose, fix=args.fix)
    passed, failed, warnings = checker.run_all_checks()

    if failed > 0:
        checker.print_installation_help()
        return 2

    if warnings > 0:
        return 1

    print(f"\n{Colors.GREEN}✅ Orchestration repository structure valid!{Colors.NC}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
