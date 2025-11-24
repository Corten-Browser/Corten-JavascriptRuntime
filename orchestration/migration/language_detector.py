#!/usr/bin/env python3
"""
Language Detector

Detects the primary programming language of a component.

Usage:
    from orchestration.migration.language_detector import LanguageDetector

    detector = LanguageDetector()
    language = detector.detect_language("components/auth_service")
"""

from pathlib import Path
from typing import Dict, Optional, Tuple
from collections import Counter


class LanguageDetector:
    """Detects the primary programming language of a component."""

    # File extensions mapped to languages
    EXTENSIONS = {
        '.py': 'python',
        '.js': 'javascript',
        '.ts': 'typescript',
        '.tsx': 'typescript',
        '.jsx': 'javascript',
        '.rs': 'rust',
        '.go': 'go',
        '.cpp': 'cpp',
        '.cc': 'cpp',
        '.cxx': 'cpp',
        '.hpp': 'cpp',
        '.h': 'cpp',  # Could be C, but we assume C++ in modern contexts
        '.java': 'java',
    }

    # Config files that indicate language
    CONFIG_FILES = {
        'package.json': 'javascript',
        'tsconfig.json': 'typescript',
        'Cargo.toml': 'rust',
        'go.mod': 'go',
        'pom.xml': 'java',
        'build.gradle': 'java',
        'CMakeLists.txt': 'cpp',
        'setup.py': 'python',
        'pyproject.toml': 'python',
        'requirements.txt': 'python',
    }

    def detect_language(self, component_path: str) -> Optional[str]:
        """
        Detect the primary language of a component.

        Args:
            component_path: Path to component directory

        Returns:
            Language name ('python', 'javascript', 'typescript', 'rust', 'go', 'cpp', 'java')
            or None if cannot determine
        """
        path = Path(component_path)

        if not path.exists() or not path.is_dir():
            return None

        # Strategy 1: Check for language-specific config files (highest confidence)
        for config_file, language in self.CONFIG_FILES.items():
            if (path / config_file).exists():
                return language

        # Strategy 2: Count source files by extension
        file_counts = self._count_source_files(path)

        if not file_counts:
            return None

        # Return language with most files
        language, count = file_counts.most_common(1)[0]
        return language

    def detect_with_confidence(self, component_path: str) -> Optional[Tuple[str, float]]:
        """
        Detect language with confidence score.

        Args:
            component_path: Path to component directory

        Returns:
            Tuple of (language, confidence) where confidence is 0.0-1.0
            or None if cannot determine
        """
        path = Path(component_path)

        if not path.exists() or not path.is_dir():
            return None

        # Check config files first (confidence = 1.0)
        for config_file, language in self.CONFIG_FILES.items():
            if (path / config_file).exists():
                return (language, 1.0)

        # Count source files
        file_counts = self._count_source_files(path)

        if not file_counts:
            return None

        # Calculate confidence based on file distribution
        total_files = sum(file_counts.values())
        language, count = file_counts.most_common(1)[0]
        confidence = count / total_files

        return (language, confidence)

    def _count_source_files(self, path: Path) -> Counter:
        """
        Count source files by language in a directory.

        Args:
            path: Directory path

        Returns:
            Counter mapping language to file count
        """
        counter = Counter()

        # Recursively find source files
        for file_path in path.rglob('*'):
            if not file_path.is_file():
                continue

            # Skip common non-source directories
            if any(part in file_path.parts for part in [
                '__pycache__', 'node_modules', '.git', 'target',
                'build', 'dist', '.venv', 'venv'
            ]):
                continue

            # Check extension
            suffix = file_path.suffix.lower()
            if suffix in self.EXTENSIONS:
                language = self.EXTENSIONS[suffix]
                counter[language] += 1

        return counter

    def get_all_languages(self, component_path: str) -> Dict[str, int]:
        """
        Get all languages found in component with file counts.

        Args:
            component_path: Path to component directory

        Returns:
            Dictionary mapping language to file count
        """
        path = Path(component_path)

        if not path.exists() or not path.is_dir():
            return {}

        file_counts = self._count_source_files(path)
        return dict(file_counts)

    def is_multi_language(self, component_path: str, threshold: float = 0.2) -> bool:
        """
        Check if component contains multiple significant languages.

        Args:
            component_path: Path to component directory
            threshold: Minimum proportion to be considered significant (default: 0.2)

        Returns:
            True if multiple languages exceed threshold
        """
        path = Path(component_path)

        if not path.exists() or not path.is_dir():
            return False

        file_counts = self._count_source_files(path)

        if len(file_counts) <= 1:
            return False

        total_files = sum(file_counts.values())
        significant_languages = [
            lang for lang, count in file_counts.items()
            if count / total_files >= threshold
        ]

        return len(significant_languages) > 1


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Detect component language")
    parser.add_argument(
        "component_path",
        help="Path to component directory"
    )
    parser.add_argument(
        "--detailed",
        action="store_true",
        help="Show all languages with file counts"
    )

    args = parser.parse_args()

    detector = LanguageDetector()

    if args.detailed:
        # Show all languages
        languages = detector.get_all_languages(args.component_path)
        if not languages:
            print("No source files found")
            return

        print(f"Languages detected in {args.component_path}:")
        for language, count in sorted(languages.items(), key=lambda x: x[1], reverse=True):
            print(f"  {language}: {count} files")

        # Show primary with confidence
        result = detector.detect_with_confidence(args.component_path)
        if result:
            language, confidence = result
            print(f"\nPrimary: {language} ({confidence:.1%} confidence)")
    else:
        # Show primary only
        language = detector.detect_language(args.component_path)
        if language:
            print(language)
        else:
            print("Unknown")


if __name__ == "__main__":
    main()
