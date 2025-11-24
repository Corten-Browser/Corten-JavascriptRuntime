#!/usr/bin/env python3
"""
C++ Updater

Updates C++ includes and references when renaming components.

Handles:
- #include "components/old-name/header.h"
- #include <components/old-name/header.h>
"""

import re
from pathlib import Path
from typing import List

from .base_updater import BaseUpdater


class CppUpdater(BaseUpdater):
    """Updates C++ source files for component renames."""

    def get_files_to_update(self, project_dir: Path) -> List[Path]:
        """
        Find all C++ files that might need updating.

        Args:
            project_dir: Project root directory

        Returns:
            List of .cpp/.hpp/.h/.cc/.cxx file paths
        """
        files = []
        extensions = {'.cpp', '.hpp', '.h', '.cc', '.cxx', '.hxx'}

        for ext in extensions:
            for file_path in project_dir.rglob(f'*{ext}'):
                if self.should_skip_file(file_path):
                    continue
                files.append(file_path)

        return files

    def update_file(self, file_path: Path) -> bool:
        """
        Update C++ includes in a single file.

        Args:
            file_path: Path to C++ file

        Returns:
            True if file was modified, False otherwise
        """
        content = self._read_file(file_path)
        original_content = content

        # C++ typically uses hyphens or underscores in paths
        # Pattern 1: #include "components/old-name/header.h"
        pattern1 = re.compile(
            rf'#include\s+"components/{re.escape(self.old_name)}(/[^"]+)"'
        )
        content = pattern1.sub(
            rf'#include "components/{self.new_name}\1"',
            content
        )

        # Pattern 2: #include <components/old-name/header.h>
        pattern2 = re.compile(
            rf'#include\s+<components/{re.escape(self.old_name)}(/[^>]+)>'
        )
        content = pattern2.sub(
            rf'#include <components/{self.new_name}\1>',
            content
        )

        # Pattern 3: Namespace references (if component name is used as namespace)
        # namespace old_name {
        old_cpp_name = self.old_name.replace('-', '_')
        new_cpp_name = self.new_name.replace('-', '_')

        pattern3 = re.compile(
            rf'\bnamespace\s+{re.escape(old_cpp_name)}\b'
        )
        content = pattern3.sub(
            rf'namespace {new_cpp_name}',
            content
        )

        # Pattern 4: using namespace old_name;
        pattern4 = re.compile(
            rf'\busing\s+namespace\s+{re.escape(old_cpp_name)}\b'
        )
        content = pattern4.sub(
            rf'using namespace {new_cpp_name}',
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

    parser = argparse.ArgumentParser(description="Update C++ includes for component rename")
    parser.add_argument("old_name", help="Old component name")
    parser.add_argument("new_name", help="New component name")
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current)"
    )

    args = parser.parse_args()

    updater = CppUpdater(args.old_name, args.new_name)
    stats = updater.update_all(Path(args.project_dir))

    print(f"\n✅ C++ Update Complete:")
    print(f"   Files scanned: {stats['files_scanned']}")
    print(f"   Files modified: {stats['files_modified']}")
    if stats['errors'] > 0:
        print(f"   Errors: {stats['errors']}")


if __name__ == "__main__":
    main()
