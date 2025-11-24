#!/usr/bin/env python3
"""
Pre-Commit Hook for Component Naming Validation

Checks for component naming violations before allowing commits.

This hook is called by git before a commit is made. If it exits with
a non-zero status, the commit is aborted.

Installation:
    Run: bash orchestration/scripts/install_hooks.sh

    Or manually:
    ln -s ../../orchestration/hooks/pre_commit_naming.py .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

Configuration:
    Set in orchestration/config/orchestration.json:
    {
        "naming_validation": {
            "pre_commit_hook": {
                "enabled": true,
                "block_commit": false,
                "show_details": true
            }
        }
    }
"""

import sys
import json
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths
from orchestration.validation.naming_scanner import ComponentNamingScanner

# Global paths instance
_paths = DataPaths()


def load_config() -> dict:
    """
    Load hook configuration.

    Returns:
        Configuration dictionary with defaults
    """
    config_path = _paths.orchestration_config

    if not config_path.exists():
        return {
            "enabled": True,
            "block_commit": False,
            "show_details": True
        }

    try:
        with open(config_path, 'r') as f:
            config = json.load(f)
            return config.get("naming_validation", {}).get("pre_commit_hook", {
                "enabled": True,
                "block_commit": False,
                "show_details": True
            })
    except (json.JSONDecodeError, KeyError):
        return {
            "enabled": True,
            "block_commit": False,
            "show_details": True
        }


def main():
    """Pre-commit hook entry point."""
    config = load_config()

    # Check if hook is enabled
    if not config.get("enabled", True):
        # Silently allow commit
        sys.exit(0)

    # Scan for violations
    scanner = ComponentNamingScanner()
    violations = scanner.scan(".")

    if not violations:
        # No violations - allow commit
        sys.exit(0)

    # Violations found
    print("\n" + "=" * 60)
    print("⚠️  COMPONENT NAMING VIOLATIONS DETECTED")
    print("=" * 60)

    if config.get("show_details", True):
        print(f"\nFound {len(violations)} component(s) with invalid names:\n")
        for old_name, info in violations.items():
            print(f"  ❌ {old_name}")
            print(f"     Error: {info['error']}")
            print(f"     Suggestion: {info['suggestion']}")
            print()
    else:
        print(f"\nFound {len(violations)} component(s) with invalid names")
        print("Run with show_details=true for more information")
        print()

    print("To fix violations:")
    print("  python orchestration/migration/rename_components.py")
    print()

    if config.get("block_commit", False):
        print("❌ COMMIT BLOCKED")
        print("   Fix violations before committing")
        print("   Or disable blocking in orchestration/config/orchestration.json")
        print("=" * 60)
        print()
        sys.exit(1)
    else:
        print("⚠️  COMMIT ALLOWED (warnings only)")
        print("   Consider fixing violations before pushing")
        print("=" * 60)
        print()
        sys.exit(0)


if __name__ == "__main__":
    main()
