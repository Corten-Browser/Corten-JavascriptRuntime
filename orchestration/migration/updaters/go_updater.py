#!/usr/bin/env python3
"""
Go Updater

Updates Go imports and references when renaming components.

Handles:
- import "project/components/old-name"
- import old_name "project/components/old-name"
"""

import re
from pathlib import Path
from typing import List

from .base_updater import BaseUpdater


class GoUpdater(BaseUpdater):
    """Updates Go source files for component renames."""

    def get_files_to_update(self, project_dir: Path) -> List[Path]:
        """
        Find all Go files that might need updating.

        Args:
            project_dir: Project root directory

        Returns:
            List of .go file paths
        """
        files = []

        for file_path in project_dir.rglob('*.go'):
            if self.should_skip_file(file_path):
                continue
            files.append(file_path)

        return files

    def update_file(self, file_path: Path) -> bool:
        """
        Update Go imports in a single file.

        Args:
            file_path: Path to Go file

        Returns:
            True if file was modified, False otherwise
        """
        content = self._read_file(file_path)
        original_content = content

        # Go uses underscores/hyphens in paths but we need to update both
        # Pattern 1: import "components/old-name"
        # import "components/old-name/module"
        pattern1 = re.compile(
            rf'''import\s+"[^"]*components/{re.escape(self.old_name)}(/[^"]*)?"'''
        )
        content = pattern1.sub(
            lambda m: m.group(0).replace(
                f'components/{self.old_name}',
                f'components/{self.new_name}'
            ),
            content
        )

        # Pattern 2: import alias "components/old-name"
        pattern2 = re.compile(
            rf'''import\s+\w+\s+"[^"]*components/{re.escape(self.old_name)}(/[^"]*)?"'''
        )
        content = pattern2.sub(
            lambda m: m.group(0).replace(
                f'components/{self.old_name}',
                f'components/{self.new_name}'
            ),
            content
        )

        # Pattern 3: Multi-line import blocks
        # import (
        #     "components/old-name"
        # )
        pattern3 = re.compile(
            rf'''"[^"]*components/{re.escape(self.old_name)}(/[^"]*)?"'''
        )
        content = pattern3.sub(
            lambda m: m.group(0).replace(
                f'components/{self.old_name}',
                f'components/{self.new_name}'
            ),
            content
        )

        # Check if modified
        if content != original_content:
            self._write_file(file_path, content)
            return True

        return False


def main():
    """CLI entry point for testing."""
    import argparse

    parser = argparse.ArgumentParser(description="Update Go imports for component rename")
    parser.add_argument("old_name", help="Old component name")
    parser.add_argument("new_name", help="New component name")
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    updater = GoUpdater(args.old_name, args.new_name)
    stats = updater.update_all(Path(args.project_dir))

    print(f"\n✅ Go Update Complete:")
    print(f"   Files scanned: {stats['files_scanned']}")
    print(f"   Files modified: {stats['files_modified']}")
    if stats['errors'] > 0:
        print(f"   Errors: {stats['errors']}")


if __name__ == "__main__":
    main()
