#!/usr/bin/env python3
"""
Component Renaming Tool

User-friendly CLI for renaming components to follow naming conventions.

Usage:
    python orchestration/migration/rename_components.py                    # Interactive mode
    python orchestration/migration/rename_components.py --dry-run          # Show what would change
    python orchestration/migration/rename_components.py --yes              # Auto-approve all
    python orchestration/migration/rename_components.py old_name new_name  # Rename specific component
"""

import sys
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.validation.naming_scanner import ComponentNamingScanner
from orchestration.migration.migration_coordinator import MigrationCoordinator


def interactive_mode(coordinator: MigrationCoordinator, scanner: ComponentNamingScanner):
    """
    Interactive mode with user prompts.

    Args:
        coordinator: Migration coordinator instance
        scanner: Naming scanner instance
    """
    print("=" * 60)
    print("COMPONENT NAMING MIGRATION")
    print("=" * 60)

    # Scan for violations
    violations = scanner.scan(".")

    if not violations:
        print("\n✅ All component names are valid!")
        print("   No migration needed.")
        return

    print(f"\n❌ Found {len(violations)} component(s) with invalid names:")
    print()

    for old_name, info in violations.items():
        print(f"  Component: {old_name}")
        print(f"    Location: {info['path']}")
        print(f"    Issue: {info['error']}")
        print(f"    Suggested: {info['suggestion']}")
        print()

    # Ask user what to do
    print("Options:")
    print("  1. Rename all automatically")
    print("  2. Rename one at a time (interactive)")
    print("  3. Show detailed analysis")
    print("  4. Exit")
    print()

    choice = input("Select option [1-4]: ").strip()

    if choice == "1":
        # Rename all
        confirm = input(f"\nRename {len(violations)} component(s)? [y/N]: ").strip().lower()
        if confirm == 'y':
            coordinator.rename_all_invalid(dry_run=False)
        else:
            print("Cancelled")

    elif choice == "2":
        # Interactive rename
        for old_name, info in violations.items():
            print(f"\n{'=' * 60}")
            print(f"Component: {old_name}")
            print(f"  Issue: {info['error']}")
            print(f"  Suggested: {info['suggestion']}")
            print()

            action = input("Rename? [y/N/q]: ").strip().lower()
            if action == 'q':
                print("Quitting")
                break
            elif action == 'y':
                coordinator.rename_component(old_name, info['suggestion'])
            else:
                print("Skipped")

    elif choice == "3":
        # Show detailed analysis
        scanner.show_detailed_analysis(violations)

    else:
        print("Exiting")


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Rename components to follow naming conventions",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                          # Interactive mode
  %(prog)s --dry-run                # Show what would change
  %(prog)s --yes                    # Rename all without prompts
  %(prog)s auth-service auth_service # Rename specific component
        """
    )
    parser.add_argument(
        "old_name",
        nargs="?",
        help="Old component name (optional, triggers interactive if not provided)"
    )
    parser.add_argument(
        "new_name",
        nargs="?",
        help="New component name (optional, auto-suggested if not provided)"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would change without making changes"
    )
    parser.add_argument(
        "--yes",
        "-y",
        action="store_true",
        help="Auto-approve all renames (non-interactive)"
    )
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    coordinator = MigrationCoordinator(args.project_dir)
    scanner = ComponentNamingScanner()

    # Mode 1: Specific component rename
    if args.old_name:
        success = coordinator.rename_component(args.old_name, args.new_name, args.dry_run)
        sys.exit(0 if success else 1)

    # Mode 2: Auto-approve all
    elif args.yes:
        if args.dry_run:
            coordinator.rename_all_invalid(dry_run=True)
        else:
            count = coordinator.rename_all_invalid(dry_run=False)
            if count > 0:
                print(f"\n✅ Successfully renamed {count} component(s)")
            sys.exit(0)

    # Mode 3: Dry run all
    elif args.dry_run:
        coordinator.rename_all_invalid(dry_run=True)
        sys.exit(0)

    # Mode 4: Interactive
    else:
        try:
            interactive_mode(coordinator, scanner)
        except KeyboardInterrupt:
            print("\n\nCancelled by user")
            sys.exit(1)


if __name__ == "__main__":
    main()
