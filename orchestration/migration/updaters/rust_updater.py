#!/usr/bin/env python3
"""
Rust Updater

Updates Rust imports and references when renaming components.

Handles:
- use components::old_name::module;
- extern crate old_name;
- mod old_name;
"""

import re
from pathlib import Path
from typing import List

from .base_updater import BaseUpdater


class RustUpdater(BaseUpdater):
    """Updates Rust source files for component renames."""

    def get_files_to_update(self, project_dir: Path) -> List[Path]:
        """
        Find all Rust files that might need updating.

        Args:
            project_dir: Project root directory

        Returns:
            List of .rs file paths
        """
        files = []

        for file_path in project_dir.rglob('*.rs'):
            if self.should_skip_file(file_path):
                continue
            files.append(file_path)

        return files

    def update_file(self, file_path: Path) -> bool:
        """
        Update Rust imports in a single file.

        Args:
            file_path: Path to Rust file

        Returns:
            True if file was modified, False otherwise
        """
        content = self._read_file(file_path)
        original_content = content

        # Rust uses underscores, so convert hyphenated old_name to underscores
        old_rust_name = self.old_name.replace('-', '_')
        new_rust_name = self.new_name.replace('-', '_')

        # Pattern 1: use components::old_name::module;
        pattern1 = re.compile(
            rf'\buse\s+components::{re.escape(old_rust_name)}(::[\w:]+)?\b'
        )
        content = pattern1.sub(
            rf'use components::{new_rust_name}\1',
            content
        )

        # Pattern 2: extern crate old_name;
        pattern2 = re.compile(
            rf'\bextern\s+crate\s+{re.escape(old_rust_name)}\b'
        )
        content = pattern2.sub(
            rf'extern crate {new_rust_name}',
            content
        )

        # Pattern 3: mod old_name;
        pattern3 = re.compile(
            rf'\bmod\s+{re.escape(old_rust_name)}\b'
        )
        content = pattern3.sub(
            rf'mod {new_rust_name}',
            content
        )

        # Pattern 4: String literals with component paths
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

    parser = argparse.ArgumentParser(description="Update Rust imports for component rename")
    parser.add_argument("old_name", help="Old component name")
    parser.add_argument("new_name", help="New component name")
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    updater = RustUpdater(args.old_name, args.new_name)
    stats = updater.update_all(Path(args.project_dir))

    print(f"\n✅ Rust Update Complete:")
    print(f"   Files scanned: {stats['files_scanned']}")
    print(f"   Files modified: {stats['files_modified']}")
    if stats['errors'] > 0:
        print(f"   Errors: {stats['errors']}")


if __name__ == "__main__":
    main()
