#!/usr/bin/env python3
"""
Detect stub/placeholder components that indicate incomplete implementation.
"""
import re
from pathlib import Path


def scan_for_stubs(project_dir: Path) -> list[str]:
    """
    Scan project for stub/placeholder code.
    Returns list of files containing stubs.
    """
    stub_patterns = [
        r'implementation\s+pending',
        r'TODO:\s*implement',
        r'raise\s+NotImplementedError',
        r'pass\s*#\s*stub',
        r'pass\s*#\s*TODO',
        r'unimplemented!\(\)',
        r'todo!\(\)',
        r'panic!\("not\s+implemented',
        r'\.\.\.  # TODO',
        r'return\s+None\s*#\s*placeholder',
        r'PLACEHOLDER',
        r'SKELETON',
    ]

    stubs_found = []

    # Check Python files
    for py_file in project_dir.rglob("*.py"):
        if "__pycache__" in str(py_file) or ".venv" in str(py_file):
            continue

        try:
            content = py_file.read_text()
        except Exception:
            continue

        for pattern in stub_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                stubs_found.append(str(py_file))
                break

    # Check Rust files
    for rs_file in project_dir.rglob("*.rs"):
        if "target" in str(rs_file):
            continue

        try:
            content = rs_file.read_text()
        except Exception:
            continue

        for pattern in stub_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                stubs_found.append(str(rs_file))
                break

    # Check READMEs for "implementation pending" language
    for readme in project_dir.rglob("README.md"):
        try:
            content = readme.read_text()
        except Exception:
            continue

        pending_phrases = [
            "implementation pending",
            "not yet implemented",
            "skeleton only",
            "placeholder component",
            "stub implementation",
        ]

        for phrase in pending_phrases:
            if phrase.lower() in content.lower():
                stubs_found.append(str(readme))
                break

    return list(set(stubs_found))  # Remove duplicates


if __name__ == "__main__":
    import sys
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    stubs = scan_for_stubs(target)

    if stubs:
        print(f"Found {len(stubs)} stub/placeholder files:")
        for stub in stubs:
            print(f"  - {stub}")
        sys.exit(1)
    else:
        print("No stubs found")
        sys.exit(0)
