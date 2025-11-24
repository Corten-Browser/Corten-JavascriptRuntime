#!/usr/bin/env python3
"""
Migration Coordinator

Coordinates the entire component renaming process:
1. Validates names
2. Creates backup
3. Detects languages
4. Updates imports in all languages
5. Renames directory
6. Reports results

Usage:
    from orchestration.migration.migration_coordinator import MigrationCoordinator

    coordinator = MigrationCoordinator()
    success = coordinator.rename_component("auth-service", "auth_service")
"""

import sys
import shutil
from pathlib import Path
from typing import Dict, List, Optional

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.verification.system.component_name_validator import ComponentNameValidator
from orchestration.migration.language_detector import LanguageDetector
from orchestration.migration.backup_manager import BackupManager
from orchestration.migration.updaters import get_updater, UPDATER_MAP


class MigrationCoordinator:
    """Coordinates component renaming across all languages."""

    def __init__(self, project_dir: str = "."):
        """
        Initialize coordinator.

        Args:
            project_dir: Project root directory (default: current directory)
        """
        self.project_dir = Path(project_dir)
        self.validator = ComponentNameValidator()
        self.detector = LanguageDetector()
        self.backup_mgr = BackupManager()

    def rename_component(
        self,
        old_name: str,
        new_name: Optional[str] = None,
        dry_run: bool = False
    ) -> bool:
        """
        Rename a component with full migration.

        Args:
            old_name: Old component name
            new_name: New component name (default: auto-suggested)
            dry_run: If True, show what would happen without making changes

        Returns:
            True if successful, False otherwise
        """
        print("=" * 60)
        print("COMPONENT MIGRATION")
        print("=" * 60)

        # Step 1: Validate old component exists
        old_path = self.project_dir / "components" / old_name
        if not old_path.exists():
            print(f"❌ Component not found: {old_path}")
            return False

        # Step 2: Validate/suggest new name
        if new_name is None:
            # Auto-suggest
            result = self.validator.validate(old_name)
            if result.is_valid:
                print(f"✅ Component name '{old_name}' is already valid")
                return True
            new_name = result.suggestion
            print(f"📝 Auto-suggested name: {new_name}")
        else:
            # Validate provided name
            result = self.validator.validate(new_name)
            if not result.is_valid:
                print(f"❌ Invalid new name: {result.error_message}")
                if result.suggestion:
                    print(f"   Suggestion: {result.suggestion}")
                return False

        new_path = self.project_dir / "components" / new_name

        # Step 3: Check new path doesn't exist
        if new_path.exists():
            print(f"❌ Target already exists: {new_path}")
            return False

        # Step 4: Detect language
        print(f"\n🔍 Detecting language...")
        language_result = self.detector.detect_with_confidence(str(old_path))
        if language_result:
            language, confidence = language_result
            print(f"   Detected: {language} ({confidence:.1%} confidence)")
        else:
            print(f"   ⚠️  Could not detect language, will try all updaters")
            language = None

        # Step 5: Get all languages present
        all_languages = self.detector.get_all_languages(str(old_path))
        if all_languages:
            print(f"\n📊 All languages in component:")
            for lang, count in sorted(all_languages.items(), key=lambda x: x[1], reverse=True):
                print(f"   - {lang}: {count} files")

        if dry_run:
            print(f"\n🔍 DRY RUN MODE - No changes will be made")
            self._show_dry_run_plan(old_name, new_name, all_languages or {language: 0})
            return True

        # Step 6: Create backup
        print(f"\n💾 Creating backup...")
        try:
            backup_id = self.backup_mgr.create_backup(str(old_path))
        except Exception as e:
            print(f"❌ Backup failed: {e}")
            return False

        # Step 7: Update imports in all languages
        print(f"\n🔄 Updating imports...")
        update_stats = self._update_all_imports(old_name, new_name, all_languages or {language: 0})

        # Step 8: Rename directory
        print(f"\n📁 Renaming directory...")
        try:
            shutil.move(str(old_path), str(new_path))
            print(f"   {old_name} → {new_name}")
        except Exception as e:
            print(f"❌ Directory rename failed: {e}")
            print(f"🔄 Restoring from backup...")
            self.backup_mgr.restore_backup(backup_id)
            return False

        # Step 9: Report results
        print("\n" + "=" * 60)
        print("✅ MIGRATION COMPLETE")
        print("=" * 60)
        print(f"\nRenamed: {old_name} → {new_name}")
        print(f"Backup ID: {backup_id}")
        print(f"\nImport updates:")
        for lang, stats in update_stats.items():
            print(f"  {lang}:")
            print(f"    Files scanned: {stats['files_scanned']}")
            print(f"    Files modified: {stats['files_modified']}")
            if stats['errors'] > 0:
                print(f"    Errors: {stats['errors']}")

        print(f"\n💡 To remove backup: python orchestration/migration/backup_manager.py remove {backup_id}")
        print("=" * 60)

        return True

    def _update_all_imports(
        self,
        old_name: str,
        new_name: str,
        languages: Dict[str, int]
    ) -> Dict[str, Dict]:
        """
        Update imports in all detected languages.

        Args:
            old_name: Old component name
            new_name: New component name
            languages: Dictionary of language -> file count

        Returns:
            Dictionary of language -> update stats
        """
        results = {}

        for language in languages.keys():
            if language not in UPDATER_MAP:
                print(f"   ⚠️  Skipping unsupported language: {language}")
                continue

            try:
                updater = get_updater(language, old_name, new_name)
                print(f"   Updating {language}...")
                stats = updater.update_all(self.project_dir)
                results[language] = stats
            except Exception as e:
                print(f"   ❌ Error updating {language}: {e}")
                results[language] = {'files_scanned': 0, 'files_modified': 0, 'errors': 1}

        return results

    def _show_dry_run_plan(self, old_name: str, new_name: str, languages: Dict[str, int]):
        """Show what would happen in a dry run."""
        print(f"\nPlanned actions:")
        print(f"  1. Create backup of components/{old_name}")
        print(f"  2. Update imports in:")
        for lang, count in sorted(languages.items(), key=lambda x: x[1], reverse=True):
            print(f"     - {lang} ({count} files)")
        print(f"  3. Rename components/{old_name} → components/{new_name}")
        print(f"\nTo execute: run without --dry-run flag")

    def rename_all_invalid(self, dry_run: bool = False) -> int:
        """
        Rename all components with invalid names.

        Args:
            dry_run: If True, show what would happen without making changes

        Returns:
            Number of components renamed (or would be renamed in dry run)
        """
        components_dir = self.project_dir / "components"

        if not components_dir.exists():
            print("No components directory found")
            return 0

        # Find all invalid components
        invalid = []
        for comp_dir in components_dir.iterdir():
            if not comp_dir.is_dir() or comp_dir.name.startswith('.'):
                continue

            result = self.validator.validate(comp_dir.name)
            if not result.is_valid:
                invalid.append((comp_dir.name, result.suggestion))

        if not invalid:
            print("✅ All component names are valid")
            return 0

        print(f"Found {len(invalid)} components to rename:")
        for old_name, new_name in invalid:
            print(f"  {old_name} → {new_name}")

        if dry_run:
            print(f"\nDRY RUN - No changes made")
            return len(invalid)

        # Rename each component
        renamed_count = 0
        for old_name, new_name in invalid:
            print(f"\n{'=' * 60}")
            if self.rename_component(old_name, new_name):
                renamed_count += 1
            else:
                print(f"⚠️  Failed to rename {old_name}")

        return renamed_count


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Coordinate component renaming")
    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # Rename single component
    rename_parser = subparsers.add_parser("rename", help="Rename a single component")
    rename_parser.add_argument("old_name", help="Old component name")
    rename_parser.add_argument("new_name", nargs="?", help="New component name (auto-suggested if not provided)")
    rename_parser.add_argument("--dry-run", action="store_true", help="Show plan without executing")

    # Rename all invalid
    rename_all_parser = subparsers.add_parser("rename-all", help="Rename all invalid components")
    rename_all_parser.add_argument("--dry-run", action="store_true", help="Show plan without executing")

    # Common arguments
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return

    coordinator = MigrationCoordinator(args.project_dir)

    if args.command == "rename":
        success = coordinator.rename_component(args.old_name, args.new_name, args.dry_run)
        sys.exit(0 if success else 1)

    elif args.command == "rename-all":
        count = coordinator.rename_all_invalid(args.dry_run)
        print(f"\n{'Would rename' if args.dry_run else 'Renamed'} {count} component(s)")
        sys.exit(0)


if __name__ == "__main__":
    main()
