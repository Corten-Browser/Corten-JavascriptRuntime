#!/usr/bin/env python3
"""
Base Updater

Abstract base class for language-specific component renamers.

All language updaters inherit from this class and implement:
- get_files_to_update(): Find files that need updating
- update_file(): Update imports/references in a single file
"""

from abc import ABC, abstractmethod
from pathlib import Path
from typing import List, Set, Dict


class BaseUpdater(ABC):
    """Abstract base class for language-specific updaters."""

    def __init__(self, old_name: str, new_name: str):
        """
        Initialize updater.

        Args:
            old_name: Old component name (e.g., "auth-service")
            new_name: New component name (e.g., "auth_service")
        """
        self.old_name = old_name
        self.new_name = new_name

    @abstractmethod
    def get_files_to_update(self, project_dir: Path) -> List[Path]:
        """
        Find all files that need updating for this language.

        Args:
            project_dir: Project root directory

        Returns:
            List of file paths that need updating
        """
        pass

    @abstractmethod
    def update_file(self, file_path: Path) -> bool:
        """
        Update imports/references in a single file.

        Args:
            file_path: Path to file to update

        Returns:
            True if file was modified, False if no changes needed
        """
        pass

    def update_all(self, project_dir: Path) -> Dict[str, int]:
        """
        Update all files in project.

        Args:
            project_dir: Project root directory

        Returns:
            Dictionary with update statistics:
            {
                'files_scanned': int,
                'files_modified': int,
                'errors': int
            }
        """
        files_to_update = self.get_files_to_update(project_dir)

        stats = {
            'files_scanned': len(files_to_update),
            'files_modified': 0,
            'errors': 0
        }

        for file_path in files_to_update:
            try:
                if self.update_file(file_path):
                    stats['files_modified'] += 1
            except Exception as e:
                print(f"❌ Error updating {file_path}: {e}")
                stats['errors'] += 1

        return stats

    def get_language_name(self) -> str:
        """
        Get the name of this language updater.

        Returns:
            Language name (e.g., "Python", "JavaScript")
        """
        # Default implementation: extract from class name
        # PythonUpdater -> Python
        class_name = self.__class__.__name__
        return class_name.replace('Updater', '')

    def _read_file(self, file_path: Path) -> str:
        """
        Safely read file content.

        Args:
            file_path: Path to file

        Returns:
            File content as string
        """
        try:
            return file_path.read_text(encoding='utf-8')
        except UnicodeDecodeError:
            # Try with different encoding
            return file_path.read_text(encoding='latin-1')

    def _write_file(self, file_path: Path, content: str):
        """
        Safely write file content.

        Args:
            file_path: Path to file
            content: Content to write
        """
        file_path.write_text(content, encoding='utf-8')

    def _get_excluded_dirs(self) -> Set[str]:
        """
        Get set of directory names to exclude from scanning.

        Returns:
            Set of directory names to skip
        """
        return {
            '__pycache__',
            'node_modules',
            '.git',
            'target',
            'build',
            'dist',
            '.venv',
            'venv',
            '.pytest_cache',
            '.mypy_cache',
            '.tox',
            'htmlcov',
            '.backups',
        }

    def should_skip_file(self, file_path: Path) -> bool:
        """
        Check if file should be skipped.

        Args:
            file_path: Path to check

        Returns:
            True if should skip, False otherwise
        """
        # Skip if in excluded directory
        excluded_dirs = self._get_excluded_dirs()
        if any(part in excluded_dirs for part in file_path.parts):
            return True

        # Skip backup files
        if file_path.name.endswith(('~', '.bak', '.swp')):
            return True

        # Skip binary files (basic check)
        if file_path.suffix in {'.pyc', '.so', '.dll', '.exe', '.bin', '.o'}:
            return True

        return False
