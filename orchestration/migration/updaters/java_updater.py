#!/usr/bin/env python3
"""
Java Updater

Updates Java imports and references when renaming components.

Handles:
- import components.old_name.ClassName;
- import components.old_name.package.*;
- package components.old_name.package;
"""

import re
from pathlib import Path
from typing import List

from .base_updater import BaseUpdater


class JavaUpdater(BaseUpdater):
    """Updates Java source files for component renames."""

    def get_files_to_update(self, project_dir: Path) -> List[Path]:
        """
        Find all Java files that might need updating.

        Args:
            project_dir: Project root directory

        Returns:
            List of .java file paths
        """
        files = []

        for file_path in project_dir.rglob('*.java'):
            if self.should_skip_file(file_path):
                continue
            files.append(file_path)

        return files

    def update_file(self, file_path: Path) -> bool:
        """
        Update Java imports in a single file.

        Args:
            file_path: Path to Java file

        Returns:
            True if file was modified, False otherwise
        """
        content = self._read_file(file_path)
        original_content = content

        # Java uses underscores or camelCase, but for package names typically lowercase
        # Convert hyphens to underscores for Java identifiers
        old_java_name = self.old_name.replace('-', '_')
        new_java_name = self.new_name.replace('-', '_')

        # Pattern 1: import components.old_name.ClassName;
        # import components.old_name.package.ClassName;
        pattern1 = re.compile(
            rf'\bimport\s+components\.{re.escape(old_java_name)}(\.[\w.]+)?\s*;'
        )
        content = pattern1.sub(
            rf'import components.{new_java_name}\1;',
            content
        )

        # Pattern 2: import components.old_name.*;
        pattern2 = re.compile(
            rf'\bimport\s+components\.{re.escape(old_java_name)}(\.[\w.]+)?\.\*\s*;'
        )
        content = pattern2.sub(
            rf'import components.{new_java_name}\1.*;',
            content
        )

        # Pattern 3: package components.old_name.package;
        pattern3 = re.compile(
            rf'\bpackage\s+components\.{re.escape(old_java_name)}(\.[\w.]+)?\s*;'
        )
        content = pattern3.sub(
            rf'package components.{new_java_name}\1;',
            content
        )

        # Pattern 4: String literals with component paths
        # "components/old-name" -> "components/new_name"
        pattern4 = re.compile(
            rf'(["\'])components/{re.escape(self.old_name)}(/?[^"\']*)\1'
        )
        content = pattern4.sub(
            rf'\1components/{self.new_name}\2\1',
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

    parser = argparse.ArgumentParser(description="Update Java imports for component rename")
    parser.add_argument("old_name", help="Old component name")
    parser.add_argument("new_name", help="New component name")
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    updater = JavaUpdater(args.old_name, args.new_name)
    stats = updater.update_all(Path(args.project_dir))

    print(f"\n✅ Java Update Complete:")
    print(f"   Files scanned: {stats['files_scanned']}")
    print(f"   Files modified: {stats['files_modified']}")
    if stats['errors'] > 0:
        print(f"   Errors: {stats['errors']}")


if __name__ == "__main__":
    main()
