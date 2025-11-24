#!/usr/bin/env python3
"""
Python Updater

Updates Python imports and references when renaming components.

Handles:
- from components.old_name import X
- from components.old_name.module import X
- import components.old_name
- import components.old_name.module as alias
"""

import re
from pathlib import Path
from typing import List

from .base_updater import BaseUpdater


class PythonUpdater(BaseUpdater):
    """Updates Python source files for component renames."""

    def get_files_to_update(self, project_dir: Path) -> List[Path]:
        """
        Find all Python files that might need updating.

        Args:
            project_dir: Project root directory

        Returns:
            List of .py file paths
        """
        files = []

        for file_path in project_dir.rglob('*.py'):
            if self.should_skip_file(file_path):
                continue
            files.append(file_path)

        return files

    def update_file(self, file_path: Path) -> bool:
        """
        Update Python imports in a single file.

        Args:
            file_path: Path to Python file

        Returns:
            True if file was modified, False otherwise
        """
        content = self._read_file(file_path)
        original_content = content

        # Pattern 1: from components.old_name import X
        # from components.old_name.module import X
        pattern1 = re.compile(
            rf'\bfrom\s+components\.{re.escape(self.old_name)}(\.[\w.]+)?\s+import\b'
        )
        content = pattern1.sub(
            rf'from components.{self.new_name}\1 import',
            content
        )

        # Pattern 2: import components.old_name
        # import components.old_name.module as alias
        pattern2 = re.compile(
            rf'\bimport\s+components\.{re.escape(self.old_name)}(\.[\w.]+)?(\s+as\s+\w+)?\b'
        )
        content = pattern2.sub(
            rf'import components.{self.new_name}\1\2',
            content
        )

        # Pattern 3: String literals containing component path
        # "components/old-name" -> "components/new_name"
        pattern3 = re.compile(
            rf'(["\'])components/{re.escape(self.old_name)}(/?[^"\']*)\1'
        )
        content = pattern3.sub(
            rf'\1components/{self.new_name}\2\1',
            content
        )

        # Pattern 4: Path-like strings with underscores (if old name had hyphens)
        # "components.old-name" -> "components.new_name"
        if '-' in self.old_name:
            pattern4 = re.compile(
                rf'(["\'])components\.{re.escape(self.old_name)}([\."\']*)'
            )
            content = pattern4.sub(
                rf'\1components.{self.new_name}\2',
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

    parser = argparse.ArgumentParser(description="Update Python imports for component rename")
    parser.add_argument("old_name", help="Old component name")
    parser.add_argument("new_name", help="New component name")
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    updater = PythonUpdater(args.old_name, args.new_name)
    stats = updater.update_all(Path(args.project_dir))

    print(f"\n✅ Python Update Complete:")
    print(f"   Files scanned: {stats['files_scanned']}")
    print(f"   Files modified: {stats['files_modified']}")
    if stats['errors'] > 0:
        print(f"   Errors: {stats['errors']}")


if __name__ == "__main__":
    main()
