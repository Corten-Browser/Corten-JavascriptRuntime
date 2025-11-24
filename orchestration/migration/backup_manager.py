#!/usr/bin/env python3
"""
Backup Manager

Creates and manages backups for safe component migrations.

Usage:
    from orchestration.migration.backup_manager import BackupManager

    backup_mgr = BackupManager()
    backup_id = backup_mgr.create_backup("components/auth-service")

    # ... perform migration ...

    if migration_failed:
        backup_mgr.restore_backup(backup_id)
    else:
        backup_mgr.remove_backup(backup_id)
"""

import os
import shutil
import subprocess
from pathlib import Path
from typing import Optional
from datetime import datetime


class BackupManager:
    """Manages backups for component migrations."""

    def __init__(self, backup_dir: str = ".backups"):
        """
        Initialize backup manager.

        Args:
            backup_dir: Directory for storing backups (default: .backups)
        """
        self.backup_dir = Path(backup_dir)
        self.backup_dir.mkdir(exist_ok=True)

    def create_backup(self, component_path: str, use_git: bool = True) -> str:
        """
        Create backup of a component.

        Args:
            component_path: Path to component directory
            use_git: If True, create git tag as well (default: True)

        Returns:
            Backup ID (can be used to restore)
        """
        path = Path(component_path)

        if not path.exists():
            raise ValueError(f"Component path does not exist: {component_path}")

        # Generate backup ID
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        component_name = path.name
        backup_id = f"{component_name}_{timestamp}"

        # Create directory backup
        backup_path = self.backup_dir / backup_id
        shutil.copytree(path, backup_path, symlinks=True)

        # Create git tag if requested and git is available
        if use_git and self._is_git_repo(path):
            tag_name = f"backup/{backup_id}"
            try:
                subprocess.run(
                    ["git", "tag", "-a", tag_name, "-m", f"Backup before renaming {component_name}"],
                    cwd=path.parent.parent,  # Project root
                    capture_output=True,
                    check=True
                )
                print(f"✅ Created git tag: {tag_name}")
            except subprocess.CalledProcessError as e:
                print(f"⚠️  Git tag creation failed: {e}")

        print(f"✅ Created backup: {backup_id}")
        return backup_id

    def restore_backup(self, backup_id: str, target_path: Optional[str] = None) -> bool:
        """
        Restore a component from backup.

        Args:
            backup_id: Backup ID returned from create_backup
            target_path: Where to restore (default: original location)

        Returns:
            True if successful, False otherwise
        """
        backup_path = self.backup_dir / backup_id

        if not backup_path.exists():
            print(f"❌ Backup not found: {backup_id}")
            return False

        # Determine target path
        if target_path is None:
            # Extract original component name from backup_id
            # Format: component_name_YYYYMMDD_HHMMSS
            parts = backup_id.rsplit('_', 2)
            if len(parts) != 3:
                print(f"❌ Invalid backup ID format: {backup_id}")
                return False
            component_name = parts[0]
            target_path = f"components/{component_name}"

        target = Path(target_path)

        # Remove existing directory if present
        if target.exists():
            print(f"Removing existing directory: {target}")
            shutil.rmtree(target)

        # Restore from backup
        shutil.copytree(backup_path, target, symlinks=True)
        print(f"✅ Restored from backup: {backup_id} → {target}")

        return True

    def remove_backup(self, backup_id: str) -> bool:
        """
        Remove a backup.

        Args:
            backup_id: Backup ID to remove

        Returns:
            True if successful, False otherwise
        """
        backup_path = self.backup_dir / backup_id

        if not backup_path.exists():
            print(f"⚠️  Backup not found: {backup_id}")
            return False

        shutil.rmtree(backup_path)
        print(f"✅ Removed backup: {backup_id}")

        return True

    def list_backups(self) -> list:
        """
        List all available backups.

        Returns:
            List of backup IDs
        """
        if not self.backup_dir.exists():
            return []

        backups = [
            item.name for item in self.backup_dir.iterdir()
            if item.is_dir()
        ]

        return sorted(backups, reverse=True)  # Most recent first

    def cleanup_old_backups(self, keep_count: int = 5) -> int:
        """
        Remove old backups, keeping only the most recent ones.

        Args:
            keep_count: Number of backups to keep per component (default: 5)

        Returns:
            Number of backups removed
        """
        backups = self.list_backups()

        if len(backups) <= keep_count:
            return 0

        # Group by component name
        by_component = {}
        for backup_id in backups:
            # Extract component name (everything before last two underscores)
            parts = backup_id.rsplit('_', 2)
            if len(parts) == 3:
                component_name = parts[0]
                if component_name not in by_component:
                    by_component[component_name] = []
                by_component[component_name].append(backup_id)

        # Remove old backups for each component
        removed_count = 0
        for component_name, component_backups in by_component.items():
            if len(component_backups) > keep_count:
                # Keep most recent, remove rest
                to_remove = sorted(component_backups, reverse=True)[keep_count:]
                for backup_id in to_remove:
                    if self.remove_backup(backup_id):
                        removed_count += 1

        return removed_count

    def _is_git_repo(self, path: Path) -> bool:
        """
        Check if path is within a git repository.

        Args:
            path: Path to check

        Returns:
            True if in git repo, False otherwise
        """
        try:
            subprocess.run(
                ["git", "rev-parse", "--git-dir"],
                cwd=path,
                capture_output=True,
                check=True
            )
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Manage component backups")
    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # Create backup
    create_parser = subparsers.add_parser("create", help="Create a backup")
    create_parser.add_argument("component_path", help="Path to component directory")
    create_parser.add_argument("--no-git", action="store_true", help="Skip git tag creation")

    # Restore backup
    restore_parser = subparsers.add_parser("restore", help="Restore from backup")
    restore_parser.add_argument("backup_id", help="Backup ID to restore")
    restore_parser.add_argument("--target", help="Target path (default: original location)")

    # List backups
    subparsers.add_parser("list", help="List all backups")

    # Remove backup
    remove_parser = subparsers.add_parser("remove", help="Remove a backup")
    remove_parser.add_argument("backup_id", help="Backup ID to remove")

    # Cleanup
    cleanup_parser = subparsers.add_parser("cleanup", help="Remove old backups")
    cleanup_parser.add_argument(
        "--keep",
        type=int,
        default=5,
        help="Number of backups to keep per component (default: 5)"
    )

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return

    backup_mgr = BackupManager()

    if args.command == "create":
        backup_id = backup_mgr.create_backup(args.component_path, use_git=not args.no_git)
        print(f"Backup ID: {backup_id}")

    elif args.command == "restore":
        success = backup_mgr.restore_backup(args.backup_id, args.target)
        exit(0 if success else 1)

    elif args.command == "list":
        backups = backup_mgr.list_backups()
        if backups:
            print("Available backups:")
            for backup_id in backups:
                print(f"  {backup_id}")
        else:
            print("No backups found")

    elif args.command == "remove":
        success = backup_mgr.remove_backup(args.backup_id)
        exit(0 if success else 1)

    elif args.command == "cleanup":
        removed = backup_mgr.cleanup_old_backups(args.keep)
        print(f"Removed {removed} old backup(s)")


if __name__ == "__main__":
    main()
