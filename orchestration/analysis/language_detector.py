#!/usr/bin/env python3
"""
Sophisticated Language Detection for Multi-Language Projects

Distinguishes implementation code from test code from tooling code.
Supports per-component language detection and confidence scoring.

Version: 1.0.0
"""

import sys
from pathlib import Path
from typing import Dict, List, Optional, Set
from dataclasses import dataclass, field
from collections import defaultdict


# Directory classification patterns
IMPLEMENTATION_DIRS = {'src', 'lib', 'app', 'core', 'components', 'pkg', 'internal'}
TEST_DIRS = {'test', 'tests', 'spec', '__tests__', 'e2e', 'integration', 'unit'}
TOOLING_DIRS = {'scripts', 'tools', 'build', 'ci', '.github', 'deploy', 'devops', 'automation'}

# File pattern classification by language
TEST_PATTERNS = {
    'python': ['test_*.py', '*_test.py', 'conftest.py'],
    'go': ['*_test.go'],
    'javascript': ['*.test.js', '*.spec.js'],
    'typescript': ['*.test.ts', '*.spec.ts', '*.test.tsx', '*.spec.tsx'],
    'rust': ['tests/**/*.rs'],
    'java': ['*Test.java', '*Tests.java'],
    'c': ['*test.c', 'test*.c'],
    'cpp': ['*test.cpp', '*test.cc', 'test*.cpp']
}

BUILD_PATTERNS = [
    'setup.py', 'Makefile', 'Dockerfile', 'build.rs', 'CMakeLists.txt',
    'package.json', 'Cargo.toml', 'go.mod', 'pom.xml', 'build.gradle',
    'webpack.config.js', 'vite.config.ts', 'tsconfig.json'
]

# Language extensions
LANGUAGE_EXTENSIONS = {
    'python': ['.py'],
    'javascript': ['.js', '.jsx'],
    'typescript': ['.ts', '.tsx'],
    'rust': ['.rs'],
    'go': ['.go'],
    'java': ['.java'],
    'c': ['.c', '.h'],
    'cpp': ['.cpp', '.cc', '.cxx', '.hpp', '.hh'],
    'ruby': ['.rb'],
    'php': ['.php'],
    'swift': ['.swift'],
    'kotlin': ['.kt'],
    'scala': ['.scala'],
    'shell': ['.sh', '.bash'],
    'yaml': ['.yaml', '.yml'],
    'json': ['.json']
}


@dataclass
class LanguageProfile:
    """Profile of a language's usage in project"""
    language: str
    total_files: int
    implementation_files: int
    test_files: int
    tooling_files: int
    implementation_percentage: float = 0.0
    test_percentage: float = 0.0
    tooling_percentage: float = 0.0
    confidence: str = "low"  # "high", "medium", "low"

    def calculate_percentages(self):
        """Calculate percentage breakdowns"""
        if self.total_files > 0:
            self.implementation_percentage = (self.implementation_files / self.total_files) * 100
            self.test_percentage = (self.test_files / self.total_files) * 100
            self.tooling_percentage = (self.tooling_files / self.total_files) * 100

    def calculate_confidence(self):
        """Calculate confidence level"""
        # High confidence if majority is implementation
        if self.implementation_percentage >= 60:
            self.confidence = "high"
        elif self.implementation_percentage >= 30:
            self.confidence = "medium"
        else:
            self.confidence = "low"


class LanguageDetector:
    """
    Sophisticated language detection system.

    Features:
    - Distinguishes implementation vs test vs tooling code
    - Per-component language detection
    - Confidence scoring
    - Multi-language project support
    """

    def __init__(self, project_dir: Path):
        """
        Initialize language detector.

        Args:
            project_dir: Root directory of project
        """
        self.project_dir = Path(project_dir).resolve()

    def detect_languages(self) -> Dict[str, LanguageProfile]:
        """
        Detect all languages used in project.

        Returns:
            Dict mapping language name to LanguageProfile
        """
        profiles = {}

        # Find all source files
        for lang, extensions in LANGUAGE_EXTENSIONS.items():
            impl_files = self._find_implementation_files(lang)
            test_files = self._find_test_files(lang)
            tool_files = self._find_tooling_files(lang)

            total = len(impl_files) + len(test_files) + len(tool_files)

            if total > 0:
                profile = LanguageProfile(
                    language=lang,
                    total_files=total,
                    implementation_files=len(impl_files),
                    test_files=len(test_files),
                    tooling_files=len(tool_files)
                )
                profile.calculate_percentages()
                profile.calculate_confidence()
                profiles[lang] = profile

        return profiles

    def identify_primary_language(self) -> str:
        """
        Identify primary implementation language.

        Returns:
            Language name
        """
        profiles = self.detect_languages()

        if not profiles:
            return "unknown"

        # Filter to high-confidence languages
        high_conf = [p for p in profiles.values() if p.confidence == "high"]

        if high_conf:
            # Return language with most implementation files
            primary = max(high_conf, key=lambda p: p.implementation_files)
            return primary.language

        # Fallback: language with most total files
        primary = max(profiles.values(), key=lambda p: p.total_files)
        return primary.language

    def detect_per_component(self, component_dirs: List[Path]) -> Dict[str, str]:
        """
        Detect language for each component.

        Args:
            component_dirs: List of component directories

        Returns:
            Dict mapping component name to language
        """
        component_languages = {}

        for comp_dir in component_dirs:
            if not comp_dir.is_dir():
                continue

            # Detect languages in this component
            temp_detector = LanguageDetector(comp_dir)
            profiles = temp_detector.detect_languages()

            if profiles:
                # Get primary language
                lang = temp_detector.identify_primary_language()
                component_languages[comp_dir.name] = lang

        return component_languages

    def _find_implementation_files(self, lang: str) -> List[Path]:
        """Find implementation files for language"""
        if lang not in LANGUAGE_EXTENSIONS:
            return []

        extensions = LANGUAGE_EXTENSIONS[lang]
        impl_files = []

        for ext in extensions:
            # Search in implementation directories
            for impl_dir in IMPLEMENTATION_DIRS:
                impl_path = self.project_dir / impl_dir
                if impl_path.exists():
                    impl_files.extend(impl_path.rglob(f"*{ext}"))

            # Also search project root (but exclude test/tool dirs)
            for file in self.project_dir.glob(f"*{ext}"):
                if self._is_implementation_file(file, lang):
                    impl_files.append(file)

        # Filter out test and tooling files
        impl_files = [f for f in impl_files if not self._is_test_file(f, lang) and not self._is_tooling_file(f)]

        return impl_files

    def _find_test_files(self, lang: str) -> List[Path]:
        """Find test files for language"""
        if lang not in LANGUAGE_EXTENSIONS:
            return []

        extensions = LANGUAGE_EXTENSIONS[lang]
        test_files = []

        for ext in extensions:
            # Search in test directories
            for test_dir in TEST_DIRS:
                test_path = self.project_dir / test_dir
                if test_path.exists():
                    test_files.extend(test_path.rglob(f"*{ext}"))

            # Search for test patterns
            if lang in TEST_PATTERNS:
                for pattern in TEST_PATTERNS[lang]:
                    test_files.extend(self.project_dir.rglob(pattern))

        # Deduplicate
        test_files = list(set(test_files))

        return test_files

    def _find_tooling_files(self, lang: str) -> List[Path]:
        """Find tooling files for language"""
        if lang not in LANGUAGE_EXTENSIONS:
            return []

        extensions = LANGUAGE_EXTENSIONS[lang]
        tool_files = []

        for ext in extensions:
            # Search in tooling directories
            for tool_dir in TOOLING_DIRS:
                tool_path = self.project_dir / tool_dir
                if tool_path.exists():
                    tool_files.extend(tool_path.rglob(f"*{ext}"))

        return tool_files

    def _is_implementation_file(self, file_path: Path, lang: str) -> bool:
        """Check if file is implementation code"""
        # Not in test or tooling directory
        parts = file_path.parts
        if any(part in TEST_DIRS for part in parts):
            return False
        if any(part in TOOLING_DIRS for part in parts):
            return False

        # Not matching test patterns
        if self._is_test_file(file_path, lang):
            return False

        # Not a build file
        if file_path.name in BUILD_PATTERNS:
            return False

        return True

    def _is_test_file(self, file_path: Path, lang: str) -> bool:
        """Check if file is test code"""
        # In test directory
        parts = file_path.parts
        if any(part in TEST_DIRS for part in parts):
            return True

        # Matches test pattern
        if lang in TEST_PATTERNS:
            for pattern in TEST_PATTERNS[lang]:
                # Simple pattern matching (could use fnmatch for more complex)
                if pattern.startswith('test_') and file_path.name.startswith('test_'):
                    return True
                if pattern.startswith('*_test') and file_path.name.endswith('_test' + file_path.suffix):
                    return True
                if '.test.' in pattern and '.test.' in file_path.name:
                    return True
                if '.spec.' in pattern and '.spec.' in file_path.name:
                    return True
                if pattern == 'conftest.py' and file_path.name == 'conftest.py':
                    return True

        return False

    def _is_tooling_file(self, file_path: Path) -> bool:
        """Check if file is tooling code"""
        # In tooling directory
        parts = file_path.parts
        if any(part in TOOLING_DIRS for part in parts):
            return True

        # Is build file
        if file_path.name in BUILD_PATTERNS:
            return True

        return False

    def generate_report(self) -> str:
        """
        Generate human-readable language detection report.

        Returns:
            Formatted report string
        """
        profiles = self.detect_languages()

        lines = []
        lines.append("=" * 70)
        lines.append("  LANGUAGE DETECTION REPORT")
        lines.append("=" * 70)
        lines.append("")

        # Primary language
        primary = self.identify_primary_language()
        lines.append(f"Primary Language: {primary.upper()}")
        lines.append("")

        # Language breakdown
        lines.append("DETECTED LANGUAGES")
        lines.append("-" * 70)

        for lang in sorted(profiles.keys()):
            profile = profiles[lang]
            lines.append(f"\n{lang.upper()} (Confidence: {profile.confidence})")
            lines.append(f"  Total Files:          {profile.total_files}")
            lines.append(f"  Implementation:       {profile.implementation_files} ({profile.implementation_percentage:.1f}%)")
            lines.append(f"  Tests:                {profile.test_files} ({profile.test_percentage:.1f}%)")
            lines.append(f"  Tooling:              {profile.tooling_files} ({profile.tooling_percentage:.1f}%)")

        lines.append("")
        lines.append("=" * 70)

        return "\n".join(lines)


def main():
    """CLI entry point"""
    import argparse
    import json

    parser = argparse.ArgumentParser(description="Sophisticated Language Detection")
    parser.add_argument("project_dir", nargs="?", default=".",
                       help="Project directory (default: current)")
    parser.add_argument("--primary", action="store_true",
                       help="Show primary language only")
    parser.add_argument("--json", action="store_true",
                       help="Output as JSON")

    args = parser.parse_args()

    detector = LanguageDetector(Path(args.project_dir))

    if args.primary:
        primary = detector.identify_primary_language()
        print(primary)
        return 0

    if args.json:
        profiles = detector.detect_languages()
        output = {}
        for lang, profile in profiles.items():
            output[lang] = {
                "total_files": profile.total_files,
                "implementation_files": profile.implementation_files,
                "test_files": profile.test_files,
                "tooling_files": profile.tooling_files,
                "implementation_percentage": profile.implementation_percentage,
                "test_percentage": profile.test_percentage,
                "tooling_percentage": profile.tooling_percentage,
                "confidence": profile.confidence
            }
        output["primary_language"] = detector.identify_primary_language()
        print(json.dumps(output, indent=2))
    else:
        print(detector.generate_report())

    return 0


if __name__ == "__main__":
    sys.exit(main())
