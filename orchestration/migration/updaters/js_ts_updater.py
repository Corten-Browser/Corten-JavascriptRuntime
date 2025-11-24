#!/usr/bin/env python3
"""
JavaScript/TypeScript Updater

Updates JS/TS imports and references when renaming components.

Handles:
- import { X } from 'components/old-name'
- import X from 'components/old-name/module'
- const X = require('components/old-name')
- import type { X } from 'components/old-name' (TypeScript)
"""

import re
from pathlib import Path
from typing import List

from .base_updater import BaseUpdater


class JsTsUpdater(BaseUpdater):
    """Updates JavaScript/TypeScript source files for component renames."""

    def get_files_to_update(self, project_dir: Path) -> List[Path]:
        """
        Find all JS/TS files that might need updating.

        Args:
            project_dir: Project root directory

        Returns:
            List of .js/.ts/.jsx/.tsx file paths
        """
        files = []
        extensions = {'.js', '.ts', '.jsx', '.tsx'}

        for ext in extensions:
            for file_path in project_dir.rglob(f'*{ext}'):
                if self.should_skip_file(file_path):
                    continue
                files.append(file_path)

        return files

    def update_file(self, file_path: Path) -> bool:
        """
        Update JS/TS imports in a single file.

        Args:
            file_path: Path to JS/TS file

        Returns:
            True if file was modified, False otherwise
        """
        content = self._read_file(file_path)
        original_content = content

        # Pattern 1: import ... from 'components/old-name/...'
        # import { X } from 'components/old-name'
        # import X from 'components/old-name/module'
        # import type { X } from 'components/old-name' (TypeScript)
        pattern1 = re.compile(
            r'''import\s+(?:type\s+)?(?:[\w{},\s*]+\s+from\s+)?['"]components/''' +
            re.escape(self.old_name) + r'''(/[^'"]*)?['"]'''
        )
        content = pattern1.sub(
            lambda m: m.group(0).replace(
                f'components/{self.old_name}',
                f'components/{self.new_name}'
            ),
            content
        )

        # Pattern 2: const X = require('components/old-name')
        pattern2 = re.compile(
            r'''require\s*\(\s*['"]components/''' +
            re.escape(self.old_name) + r'''(/[^'"]*)?['"]\s*\)'''
        )
        content = pattern2.sub(
            lambda m: m.group(0).replace(
                f'components/{self.old_name}',
                f'components/{self.new_name}'
            ),
            content
        )

        # Pattern 3: export ... from 'components/old-name/...'
        pattern3 = re.compile(
            r'''export\s+(?:\*|{[^}]+})\s+from\s+['"]components/''' +
            re.escape(self.old_name) + r'''(/[^'"]*)?['"]'''
        )
        content = pattern3.sub(
            lambda m: m.group(0).replace(
                f'components/{self.old_name}',
                f'components/{self.new_name}'
            ),
            content
        )

        # Pattern 4: Dynamic imports
        # import('components/old-name')
        pattern4 = re.compile(
            r'''import\s*\(\s*['"]components/''' +
            re.escape(self.old_name) + r'''(/[^'"]*)?['"]\s*\)'''
        )
        content = pattern4.sub(
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

    parser = argparse.ArgumentParser(description="Update JS/TS imports for component rename")
    parser.add_argument("old_name", help="Old component name")
    parser.add_argument("new_name", help="New component name")
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    updater = JsTsUpdater(args.old_name, args.new_name)
    stats = updater.update_all(Path(args.project_dir))

    print(f"\n✅ JS/TS Update Complete:")
    print(f"   Files scanned: {stats['files_scanned']}")
    print(f"   Files modified: {stats['files_modified']}")
    if stats['errors'] > 0:
        print(f"   Errors: {stats['errors']}")


if __name__ == "__main__":
    main()
