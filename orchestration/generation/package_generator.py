#!/usr/bin/env python3
"""
Package Configuration Generator

Generates proper Python package configuration files:
- setup.py: Package metadata and installation configuration
- MANIFEST.in: Package data file inclusion
- requirements.txt: Dependency list

Part of Phase 4.8 in orchestration workflow (v0.15.0).

Prevents HardPathsFailureAssessment.txt failure mode:
- No installable package (missing setup.py)
- Cannot be distributed or installed on user machines
"""

import re
from pathlib import Path
from typing import Dict, List, Optional, Set
from dataclasses import dataclass


@dataclass
class PackageMetadata:
    """Package metadata extracted from project."""
    name: str
    version: str
    description: str
    author: Optional[str] = None
    author_email: Optional[str] = None
    url: Optional[str] = None
    license: Optional[str] = "MIT"
    python_requires: str = ">=3.8"
    entry_points: Optional[Dict[str, List[str]]] = None
    dependencies: Optional[List[str]] = None
    classifiers: Optional[List[str]] = None


class PackageGenerator:
    """Generate Python package configuration files."""

    def __init__(self, project_root: Path):
        """
        Initialize package generator.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()

    def generate_all(
        self,
        metadata: PackageMetadata,
        force: bool = False
    ) -> Dict[str, Path]:
        """
        Generate all package configuration files.

        Args:
            metadata: Package metadata
            force: If True, overwrite existing files

        Returns:
            Dict mapping filename -> created file path
        """
        created = {}

        # Generate setup.py
        setup_path = self._generate_setup_py(metadata, force)
        if setup_path:
            created['setup.py'] = setup_path

        # Generate MANIFEST.in
        manifest_path = self._generate_manifest_in(force)
        if manifest_path:
            created['MANIFEST.in'] = manifest_path

        # Generate requirements.txt
        if metadata.dependencies:
            req_path = self._generate_requirements_txt(metadata.dependencies, force)
            if req_path:
                created['requirements.txt'] = req_path

        return created

    def _generate_setup_py(
        self,
        metadata: PackageMetadata,
        force: bool = False
    ) -> Optional[Path]:
        """
        Generate setup.py file.

        Args:
            metadata: Package metadata
            force: If True, overwrite existing file

        Returns:
            Path to created setup.py, or None if skipped
        """
        setup_path = self.project_root / "setup.py"

        if setup_path.exists() and not force:
            print(f"⚠️  setup.py already exists (use force=True to overwrite)")
            return None

        # Build setup.py content
        content = self._build_setup_py_content(metadata)

        # Write file
        with open(setup_path, 'w', encoding='utf-8') as f:
            f.write(content)

        print(f"✅ Generated: {setup_path}")
        return setup_path

    def _build_setup_py_content(self, metadata: PackageMetadata) -> str:
        """
        Build setup.py file content.

        Args:
            metadata: Package metadata

        Returns:
            setup.py content as string
        """
        lines = []

        # Header
        lines.append('"""Package setup configuration."""')
        lines.append('')
        lines.append('from setuptools import setup, find_packages')
        lines.append('from pathlib import Path')
        lines.append('')

        # Read README if exists
        lines.append('# Read README for long description')
        lines.append('readme_path = Path(__file__).parent / "README.md"')
        lines.append('if readme_path.exists():')
        lines.append('    long_description = readme_path.read_text(encoding="utf-8")')
        lines.append('else:')
        lines.append('    long_description = ""')
        lines.append('')

        # Read requirements if exists
        if metadata.dependencies:
            lines.append('# Read requirements')
            lines.append('requirements_path = Path(__file__).parent / "requirements.txt"')
            lines.append('if requirements_path.exists():')
            lines.append('    install_requires = [')
            lines.append('        line.strip()')
            lines.append('        for line in requirements_path.read_text().splitlines()')
            lines.append('        if line.strip() and not line.startswith("#")')
            lines.append('    ]')
            lines.append('else:')
            lines.append(f'    install_requires = {metadata.dependencies!r}')
            lines.append('')

        # Setup configuration
        lines.append('setup(')
        lines.append(f'    name="{metadata.name}",')
        lines.append(f'    version="{metadata.version}",')
        lines.append(f'    description="{metadata.description}",')
        lines.append('    long_description=long_description,')
        lines.append('    long_description_content_type="text/markdown",')

        if metadata.author:
            lines.append(f'    author="{metadata.author}",')
        if metadata.author_email:
            lines.append(f'    author_email="{metadata.author_email}",')
        if metadata.url:
            lines.append(f'    url="{metadata.url}",')

        lines.append(f'    license="{metadata.license}",')
        lines.append('    packages=find_packages(exclude=["tests", "tests.*"]),')
        lines.append('    include_package_data=True,')

        if metadata.dependencies:
            lines.append('    install_requires=install_requires,')
        else:
            lines.append('    install_requires=[],')

        lines.append(f'    python_requires="{metadata.python_requires}",')

        # Entry points (for CLI applications)
        if metadata.entry_points:
            lines.append('    entry_points={')
            for ep_type, commands in metadata.entry_points.items():
                lines.append(f'        "{ep_type}": [')
                for cmd in commands:
                    lines.append(f'            "{cmd}",')
                lines.append('        ],')
            lines.append('    },')

        # Classifiers
        if metadata.classifiers:
            lines.append('    classifiers=[')
            for classifier in metadata.classifiers:
                lines.append(f'        "{classifier}",')
            lines.append('    ],')
        else:
            # Default classifiers
            lines.append('    classifiers=[')
            lines.append('        "Development Status :: 3 - Alpha",')
            lines.append('        "Intended Audience :: Developers",')
            lines.append(f'        "License :: OSI Approved :: {metadata.license} License",')
            lines.append('        "Programming Language :: Python :: 3",')
            lines.append('        "Programming Language :: Python :: 3.8",')
            lines.append('        "Programming Language :: Python :: 3.9",')
            lines.append('        "Programming Language :: Python :: 3.10",')
            lines.append('        "Programming Language :: Python :: 3.11",')
            lines.append('    ],')

        lines.append(')')
        lines.append('')

        return '\n'.join(lines)

    def _generate_manifest_in(self, force: bool = False) -> Optional[Path]:
        """
        Generate MANIFEST.in file.

        Args:
            force: If True, overwrite existing file

        Returns:
            Path to created MANIFEST.in, or None if skipped
        """
        manifest_path = self.project_root / "MANIFEST.in"

        if manifest_path.exists() and not force:
            print(f"⚠️  MANIFEST.in already exists (use force=True to overwrite)")
            return None

        # Build MANIFEST.in content
        lines = [
            "# Include documentation",
            "include README.md",
            "include LICENSE",
            "include CHANGELOG.md",
            "",
            "# Include requirements",
            "include requirements.txt",
            "",
            "# Include package data",
            "recursive-include */templates *",
            "recursive-include */static *",
            "recursive-include */data *.json *.yaml *.yml",
            "",
            "# Exclude development files",
            "global-exclude __pycache__",
            "global-exclude *.py[co]",
            "global-exclude .DS_Store",
            "global-exclude .git*",
            "prune tests",
            "prune docs",
            "prune examples",
            "",
        ]

        content = '\n'.join(lines)

        # Write file
        with open(manifest_path, 'w', encoding='utf-8') as f:
            f.write(content)

        print(f"✅ Generated: {manifest_path}")
        return manifest_path

    def _generate_requirements_txt(
        self,
        dependencies: List[str],
        force: bool = False
    ) -> Optional[Path]:
        """
        Generate requirements.txt file.

        Args:
            dependencies: List of dependencies
            force: If True, overwrite existing file

        Returns:
            Path to created requirements.txt, or None if skipped
        """
        req_path = self.project_root / "requirements.txt"

        if req_path.exists() and not force:
            print(f"⚠️  requirements.txt already exists (use force=True to overwrite)")
            return None

        # Build requirements.txt content
        lines = [
            "# Python package dependencies",
            "# Install with: pip install -r requirements.txt",
            "",
        ]

        for dep in dependencies:
            lines.append(dep)

        lines.append("")  # Trailing newline

        content = '\n'.join(lines)

        # Write file
        with open(req_path, 'w', encoding='utf-8') as f:
            f.write(content)

        print(f"✅ Generated: {req_path}")
        return req_path

    def extract_dependencies(self) -> List[str]:
        """
        Extract dependencies from project imports.

        Scans Python files and extracts third-party imports.

        Returns:
            List of dependency names
        """
        dependencies = set()
        stdlib_modules = self._get_stdlib_modules()

        # Find all Python files
        python_files = list(self.project_root.glob("**/*.py"))

        for py_file in python_files:
            # Skip test files and cache directories
            # Check for /tests/ or /test/ directory, or files starting with test_
            relative_path = str(py_file.relative_to(self.project_root))
            if "/tests/" in relative_path or "/test/" in relative_path or \
               relative_path.startswith("tests/") or relative_path.startswith("test/") or \
               py_file.name.startswith("test_") or "__pycache__" in str(py_file):
                continue

            try:
                with open(py_file, 'r', encoding='utf-8') as f:
                    content = f.read()

                # Extract imports
                imports = self._extract_imports_from_code(content)

                for imp in imports:
                    # Skip stdlib and local imports
                    if imp not in stdlib_modules and not imp.startswith('.'):
                        dependencies.add(imp)

            except Exception:
                pass  # Skip files that can't be read

        return sorted(list(dependencies))

    def _extract_imports_from_code(self, code: str) -> Set[str]:
        """
        Extract import statements from Python code.

        Args:
            code: Python source code

        Returns:
            Set of module names
        """
        imports = set()

        # Pattern: import module
        for match in re.finditer(r'^\s*import\s+([\w.]+)', code, re.MULTILINE):
            module = match.group(1).split('.')[0]
            imports.add(module)

        # Pattern: from module import ...
        for match in re.finditer(r'^\s*from\s+([\w.]+)\s+import', code, re.MULTILINE):
            module = match.group(1).split('.')[0]
            imports.add(module)

        return imports

    def _get_stdlib_modules(self) -> Set[str]:
        """
        Get set of Python standard library modules.

        Returns:
            Set of stdlib module names
        """
        # Common stdlib modules (not exhaustive, but covers most)
        return {
            'abc', 'argparse', 'ast', 'asyncio', 'base64', 'collections',
            'contextlib', 'copy', 'dataclasses', 'datetime', 'decimal',
            'enum', 'functools', 'glob', 'hashlib', 'importlib', 'io',
            'itertools', 'json', 'logging', 'math', 'os', 'pathlib',
            'pickle', 're', 'shutil', 'socket', 'string', 'subprocess',
            'sys', 'tempfile', 'threading', 'time', 'traceback', 'typing',
            'unittest', 'urllib', 'uuid', 'warnings', 'weakref', 'xml',
        }


def main():
    """CLI interface for package generation."""
    import sys
    import argparse

    parser = argparse.ArgumentParser(description="Generate Python package configuration")
    parser.add_argument(
        "project_root",
        nargs="?",
        default=".",
        help="Project root directory"
    )
    parser.add_argument(
        "--name",
        required=True,
        help="Package name"
    )
    parser.add_argument(
        "--version",
        default="0.1.0",
        help="Package version (default: 0.1.0)"
    )
    parser.add_argument(
        "--description",
        required=True,
        help="Package description"
    )
    parser.add_argument(
        "--author",
        help="Package author"
    )
    parser.add_argument(
        "--author-email",
        help="Author email"
    )
    parser.add_argument(
        "--url",
        help="Project URL"
    )
    parser.add_argument(
        "--entry-point",
        action="append",
        help="Entry point (format: 'name=module:function')"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing files"
    )

    args = parser.parse_args()
    project_root = Path(args.project_root)

    # Build entry points
    entry_points = None
    if args.entry_point:
        entry_points = {'console_scripts': args.entry_point}

    # Create metadata
    metadata = PackageMetadata(
        name=args.name,
        version=args.version,
        description=args.description,
        author=args.author,
        author_email=args.author_email,
        url=args.url,
        entry_points=entry_points
    )

    # Generate files
    generator = PackageGenerator(project_root)

    # Extract dependencies
    dependencies = generator.extract_dependencies()
    metadata.dependencies = dependencies

    print(f"\n📦 Generating package configuration for: {metadata.name} v{metadata.version}\n")

    created = generator.generate_all(metadata, force=args.force)

    print(f"\n✅ Generated {len(created)} file(s):")
    for filename, path in created.items():
        print(f"   - {filename}")

    if dependencies:
        print(f"\n📋 Detected {len(dependencies)} dependencies:")
        for dep in dependencies:
            print(f"   - {dep}")

    print()


if __name__ == '__main__':
    main()
