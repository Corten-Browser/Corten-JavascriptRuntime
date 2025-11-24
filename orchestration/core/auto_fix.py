#!/usr/bin/env python3
"""
Auto-Fix System

Automatically detects and fixes naming violations based on configuration.

Usage:
    from orchestration.core.auto_fix import AutoFix

    auto_fix = AutoFix()
    if auto_fix.should_run():
        auto_fix.run()
"""

import json
import sys
from pathlib import Path
from typing import Optional, Dict

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.validation.naming_scanner import ComponentNamingScanner
from orchestration.migration.migration_coordinator import MigrationCoordinator


class AutoFix:
    """Automatically fix naming violations based on configuration."""

    def __init__(self, project_dir: Optional[Path] = None, config_path: Optional[Path] = None):
        """
        Initialize auto-fix system.

        Args:
            project_dir: Project directory (default: current directory)
            config_path: Path to orchestration-config.json (default: orchestration/config/)
        """
        self.project_dir = project_dir or Path.cwd()
        self.config_path = config_path or (self.project_dir / "orchestration" / "config" / "orchestration-config.json")
        self.config = self._load_config()
        self.scanner = ComponentNamingScanner()
        self.coordinator = MigrationCoordinator(str(self.project_dir))

    def _load_config(self) -> Dict:
        """
        Load configuration from file.

        Returns:
            Configuration dictionary
        """
        if not self.config_path.exists():
            # Return default configuration
            return {
                "naming_validation": {
                    "enabled": True,
                    "auto_fix": "prompt",  # Options: "always", "prompt", "never"
                    "block_on_violations": True
                }
            }

        try:
            with open(self.config_path, 'r') as f:
                return json.load(f)
        except json.JSONDecodeError:
            print(f"⚠️  Warning: Invalid JSON in {self.config_path}, using defaults")
            return {"naming_validation": {"enabled": True, "auto_fix": "prompt", "block_on_violations": True}}

    def should_run(self) -> bool:
        """
        Check if auto-fix should run based on configuration.

        Returns:
            True if enabled and violations exist, False otherwise
        """
        config = self.config.get("naming_validation", {})

        # Check if enabled
        if not config.get("enabled", True):
            return False

        # Check for violations
        violations = self.scanner.scan(str(self.project_dir))
        if not violations:
            return False

        # Check auto_fix policy
        policy = config.get("auto_fix", "prompt")
        if policy == "never":
            return False

        return True

    def run(self) -> bool:
        """
        Run auto-fix based on configuration.

        Returns:
            True if violations were fixed or no violations exist, False otherwise
        """
        config = self.config.get("naming_validation", {})
        policy = config.get("auto_fix", "prompt")

        # Scan for violations
        violations = self.scanner.scan(str(self.project_dir))

        if not violations:
            print("✅ No naming violations found")
            return True

        print(f"❌ Found {len(violations)} naming violation(s):")
        for old_name, info in violations.items():
            print(f"   {old_name} → {info['suggestion']}")
        print()

        # Handle based on policy
        if policy == "always":
            print("⚙️  Auto-fix enabled (always mode) - applying fixes...")
            return self._fix_all_violations(violations)

        elif policy == "prompt":
            print("Auto-fix policy: prompt")
            response = input("Fix all violations? [y/N]: ").strip().lower()
            if response == 'y':
                return self._fix_all_violations(violations)
            else:
                print("Auto-fix cancelled")
                if config.get("block_on_violations", True):
                    print("❌ Orchestration blocked due to naming violations")
                    print("   Fix manually or enable auto-fix in orchestration-config.json")
                    return False
                return True

        elif policy == "never":
            print("Auto-fix disabled")
            if config.get("block_on_violations", True):
                print("❌ Orchestration blocked due to naming violations")
                print("   Fix manually: python orchestration/migration/rename_components.py")
                return False
            return True

        else:
            print(f"⚠️  Unknown auto-fix policy: {policy}, treating as 'never'")
            return False

    def _fix_all_violations(self, violations: Dict) -> bool:
        """
        Fix all violations using migration coordinator.

        Args:
            violations: Dictionary of violations from scanner

        Returns:
            True if all fixed successfully, False otherwise
        """
        success_count = 0
        error_count = 0

        for old_name, info in violations.items():
            new_name = info['suggestion']
            try:
                if self.coordinator.rename_component(old_name, new_name):
                    success_count += 1
                else:
                    error_count += 1
            except Exception as e:
                print(f"❌ Error fixing {old_name}: {e}")
                error_count += 1

        print(f"\n{'=' * 60}")
        if error_count == 0:
            print(f"✅ Successfully fixed {success_count} component(s)")
            print("=" * 60)
            return True
        else:
            print(f"⚠️  Fixed {success_count} component(s), {error_count} error(s)")
            print("=" * 60)
            return False

    def get_violations_count(self) -> int:
        """
        Get count of violations without printing.

        Returns:
            Number of violations
        """
        violations = self.scanner.scan(str(self.project_dir))
        return len(violations)


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Run auto-fix for naming violations")
    parser.add_argument(
        "--project-dir",
        type=Path,
        default=Path.cwd(),
        help="Project directory (default: current directory)"
    )
    parser.add_argument(
        "--config",
        type=Path,
        help="Path to orchestration-config.json (default: orchestration/config/orchestration-config.json)"
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="Only check for violations, don't fix"
    )

    args = parser.parse_args()

    auto_fix = AutoFix(args.project_dir, args.config)

    if args.check_only:
        count = auto_fix.get_violations_count()
        if count > 0:
            print(f"❌ Found {count} violation(s)")
            sys.exit(1)
        else:
            print("✅ No violations found")
            sys.exit(0)

    if auto_fix.should_run():
        success = auto_fix.run()
        sys.exit(0 if success else 1)
    else:
        print("Auto-fix not needed or disabled")
        sys.exit(0)


if __name__ == "__main__":
    main()
