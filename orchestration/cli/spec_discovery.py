#!/usr/bin/env python3
"""
Canonical specification file discovery.

This is the SINGLE SOURCE OF TRUTH for finding specification files.
All other modules should import and use discover_all_specs() from here.

Supported locations:
- specifications/ directory (all files, recursive)
- specs/ directory (all files, recursive)
- docs/ directory (pattern-matched files only)

Supported extensions:
- .md (Markdown)
- .yaml, .yml (YAML)
- .json (JSON)
"""
import re
import json
import sys
from pathlib import Path
from datetime import datetime
from typing import Optional


# Supported specification file extensions
SPEC_EXTENSIONS = {".md", ".yaml", ".yml", ".json"}

# Directories to search exhaustively (all matching files)
SPEC_DIRECTORIES = ["specifications", "specs"]

# Pattern for spec files in docs/ directory (hyphens OR underscores)
DOCS_SPEC_PATTERN = re.compile(r".*[-_]spec(ification)?s?\.md$", re.IGNORECASE)


def discover_all_specs(project_root: Optional[Path] = None) -> list[Path]:
    """
    Discover ALL specification files in standard locations.

    This is the canonical discovery function. All other code should use this.

    Args:
        project_root: Project root directory. Defaults to current working directory.

    Returns:
        Sorted list of unique specification file paths.

    Discovery rules:
    1. specifications/ and specs/ directories: ALL files with supported extensions (recursive)
    2. docs/ directory: Only files matching *-spec*.md or *_spec*.md pattern
    """
    if project_root is None:
        project_root = Path.cwd()

    project_root = Path(project_root).resolve()
    discovered = []

    # 1. Search spec directories exhaustively (recursive)
    for dir_name in SPEC_DIRECTORIES:
        spec_dir = project_root / dir_name
        if spec_dir.exists() and spec_dir.is_dir():
            for file_path in spec_dir.rglob("*"):
                if file_path.is_file() and file_path.suffix.lower() in SPEC_EXTENSIONS:
                    discovered.append(file_path)

    # 2. Search docs/ directory with pattern matching (non-recursive)
    docs_dir = project_root / "docs"
    if docs_dir.exists() and docs_dir.is_dir():
        for file_path in docs_dir.iterdir():
            if file_path.is_file() and DOCS_SPEC_PATTERN.match(file_path.name):
                discovered.append(file_path)

    # Return sorted unique paths
    return sorted(set(discovered))


def get_discovery_summary(specs: list[Path], project_root: Optional[Path] = None) -> dict:
    """
    Generate a summary of discovered specification files.

    Args:
        specs: List of discovered spec file paths.
        project_root: Project root for relative path calculation.

    Returns:
        Dictionary with discovery summary.
    """
    if project_root is None:
        project_root = Path.cwd()

    project_root = Path(project_root).resolve()

    by_directory = {}
    by_extension = {}

    for spec in specs:
        # Group by directory
        try:
            rel_path = spec.relative_to(project_root)
            dir_name = rel_path.parts[0] if rel_path.parts else "root"
        except ValueError:
            dir_name = "external"

        by_directory[dir_name] = by_directory.get(dir_name, 0) + 1

        # Group by extension
        ext = spec.suffix.lower()
        by_extension[ext] = by_extension.get(ext, 0) + 1

    return {
        "total_files": len(specs),
        "by_directory": by_directory,
        "by_extension": by_extension,
        "files": [str(s) for s in specs],
        "discovery_timestamp": datetime.now().isoformat()
    }


def print_discovery_report(specs: list[Path], project_root: Optional[Path] = None) -> None:
    """
    Print a formatted discovery report to stdout.

    Args:
        specs: List of discovered spec file paths.
        project_root: Project root for relative path calculation.
    """
    if project_root is None:
        project_root = Path.cwd()

    project_root = Path(project_root).resolve()

    print(f"Discovered {len(specs)} specification file(s)")
    print("")

    if not specs:
        print("  No specification files found.")
        print("")
        print("  Searched locations:")
        print("    - specifications/ directory")
        print("    - specs/ directory")
        print("    - docs/*-spec*.md or docs/*_spec*.md")
        print("")
        print("  Supported extensions: .md, .yaml, .yml, .json")
        return

    # Group by directory
    by_dir = {}
    for spec in specs:
        try:
            rel_path = spec.relative_to(project_root)
            dir_name = rel_path.parts[0] if rel_path.parts else "root"
        except ValueError:
            dir_name = "external"

        if dir_name not in by_dir:
            by_dir[dir_name] = []
        by_dir[dir_name].append(spec)

    for dir_name, files in sorted(by_dir.items()):
        print(f"  {dir_name}/ ({len(files)} file(s))")
        for f in sorted(files):
            try:
                rel_path = f.relative_to(project_root)
            except ValueError:
                rel_path = f
            print(f"    - {rel_path}")
        print("")


def main():
    """CLI entry point for spec discovery."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Discover specification files in a project.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python spec_discovery.py                    # Discover in current directory
  python spec_discovery.py /path/to/project   # Discover in specific project
  python spec_discovery.py --json             # Output as JSON
  python spec_discovery.py --quiet            # Only output file paths
        """
    )
    parser.add_argument(
        "project_root",
        nargs="?",
        default=".",
        help="Project root directory (default: current directory)"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output discovery summary as JSON"
    )
    parser.add_argument(
        "--quiet", "-q",
        action="store_true",
        help="Only output file paths, one per line"
    )

    args = parser.parse_args()

    project_root = Path(args.project_root).resolve()

    if not project_root.exists():
        print(f"ERROR: Project root does not exist: {project_root}", file=sys.stderr)
        sys.exit(1)

    specs = discover_all_specs(project_root)

    if args.json:
        summary = get_discovery_summary(specs, project_root)
        print(json.dumps(summary, indent=2))
    elif args.quiet:
        for spec in specs:
            print(spec)
    else:
        print_discovery_report(specs, project_root)

    # Exit with status based on whether specs were found
    sys.exit(0 if specs else 1)


if __name__ == "__main__":
    main()
