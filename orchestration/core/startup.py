#!/usr/bin/env python3
"""
Orchestrator Startup Checks

Runs automatic validation when orchestrator starts.

Usage:
    from orchestration.core.startup import run_startup_checks

    if not run_startup_checks():
        sys.exit(1)
"""

import sys
from pathlib import Path
from typing import Optional

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.auto_fix import AutoFix


def run_startup_checks(project_dir: Optional[Path] = None, config_path: Optional[Path] = None) -> bool:
    """
    Run startup validation checks with auto-fix integration.

    Args:
        project_dir: Project directory (default: current directory)
        config_path: Path to orchestration config (default: orchestration/config/orchestration.json)

    Returns:
        True if no violations or all fixed, False if violations remain
    """
    project_dir = project_dir or Path.cwd()

    print("=" * 60)
    print("ORCHESTRATOR STARTUP CHECKS")
    print("=" * 60)

    # Use AutoFix system with configuration
    auto_fix = AutoFix(project_dir, config_path)

    # Check if any violations exist
    violations_count = auto_fix.get_violations_count()

    if violations_count == 0:
        print("\n✅ Component Naming: All valid")
        print("\n" + "=" * 60)
        print("✅ STARTUP CHECKS PASSED")
        print("=" * 60)
        return True

    # Violations found - let auto_fix handle based on configuration
    print()
    success = auto_fix.run()

    if success:
        print("\n" + "=" * 60)
        print("✅ STARTUP CHECKS PASSED")
        print("=" * 60)
    else:
        print("\n" + "=" * 60)
        print("❌ STARTUP CHECKS FAILED")
        print("=" * 60)
        print("\nOrchestration blocked due to naming violations.")
        print("Fix violations manually or adjust configuration.")
        print()

    return success


def check_naming_only(project_dir: Optional[Path] = None) -> bool:
    """
    Quick check for naming violations without full startup checks.

    Args:
        project_dir: Project directory (default: current directory)

    Returns:
        True if all valid, False if violations found
    """
    project_dir = project_dir or Path.cwd()
    auto_fix = AutoFix(project_dir)
    return auto_fix.get_violations_count() == 0


def get_violations_count(project_dir: Optional[Path] = None) -> int:
    """
    Get count of naming violations without printing.

    Args:
        project_dir: Project directory (default: current directory)

    Returns:
        Number of violations
    """
    project_dir = project_dir or Path.cwd()
    auto_fix = AutoFix(project_dir)
    return auto_fix.get_violations_count()


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Run orchestrator startup checks")
    parser.add_argument(
        "--project-dir",
        type=Path,
        default=Path.cwd(),
        help="Project directory (default: current directory)"
    )
    parser.add_argument(
        "--config",
        type=Path,
        help="Path to orchestration config (default: orchestration/config/orchestration.json)"
    )

    args = parser.parse_args()

    success = run_startup_checks(args.project_dir, args.config)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
