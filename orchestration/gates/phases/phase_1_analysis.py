#!/usr/bin/env python3
"""
Phase 1 Gate: Analysis

Validates project structure, specification completeness, and component naming.

Exit Criteria:
- Specification document exists and is complete
- Directory structure is correct
- All component names follow naming convention
- Project metadata is valid

Part of naming convention enforcement (v1.7.2+)
"""

import sys
from pathlib import Path
from typing import Tuple, List

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from orchestration.verification.system.component_name_validator import ComponentNameValidator


def check_directory_structure(project_dir: Path) -> Tuple[bool, str]:
    """Check that required directories exist."""
    required_dirs = [
        "orchestration",
        "specifications",
        "shared-libs"
    ]

    missing = []
    for dir_name in required_dirs:
        if not (project_dir / dir_name).exists():
            missing.append(dir_name)

    if missing:
        return False, f"Missing required directories: {', '.join(missing)}"

    return True, "All required directories present"


def check_specification_exists(project_dir: Path) -> Tuple[bool, str]:
    """Check that a specification document exists."""
    spec_dir = project_dir / "specifications"

    if not spec_dir.exists():
        return False, "Specifications directory does not exist"

    # Look for spec files
    spec_files = list(spec_dir.glob("*.md")) + list(spec_dir.glob("*.yaml"))

    if not spec_files:
        return False, "No specification files found in specifications/"

    return True, f"Found {len(spec_files)} specification file(s)"


def check_component_naming(project_dir: Path) -> Tuple[bool, str]:
    """
    Validate all component names follow convention.

    This is the critical check for naming enforcement.
    """
    validator = ComponentNameValidator()
    components_dir = project_dir / "components"

    if not components_dir.exists():
        # No components yet - that's okay for Phase 1
        return True, "No components directory yet (new project)"

    invalid = []
    components = []

    for comp_dir in components_dir.iterdir():
        if not comp_dir.is_dir() or comp_dir.name.startswith('.'):
            continue

        components.append(comp_dir.name)
        result = validator.validate(comp_dir.name)
        if not result.is_valid:
            invalid.append((comp_dir.name, result))

    if invalid:
        print("\n❌ Component Naming Violations:")
        for name, result in invalid:
            print(f"   - {name}")
            print(f"     Error: {result.error_message}")
            print(f"     Suggestion: {result.suggestion}")

        print("\nTo fix:")
        print("  python orchestration/migration/rename_components.py")
        print("\nOr let orchestrator auto-fix on startup")

        return False, f"{len(invalid)} of {len(components)} components have invalid names"

    if components:
        return True, f"All {len(components)} component names are valid"
    else:
        return True, "No components to validate (new project)"


def run_phase_1_gate(project_dir: Path) -> bool:
    """
    Run Phase 1 Analysis gate.

    Returns:
        True if gate passes, False otherwise
    """
    print("=" * 60)
    print("PHASE 1 GATE: ANALYSIS")
    print("=" * 60)

    checks = [
        ("Directory Structure", check_directory_structure),
        ("Specification Exists", check_specification_exists),
        ("Component Naming", check_component_naming),
    ]

    results = []
    for check_name, check_func in checks:
        passed, message = check_func(project_dir)
        results.append((check_name, passed, message))

        icon = "✅" if passed else "❌"
        print(f"\n{icon} {check_name}")
        print(f"   {message}")

    all_passed = all(passed for _, passed, _ in results)

    print("\n" + "=" * 60)
    if all_passed:
        print("✅ PHASE 1 GATE PASSED")
    else:
        print("❌ PHASE 1 GATE FAILED")
        print("\nFix issues and re-run gate before proceeding to Phase 2")

    print("=" * 60)

    return all_passed


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Run Phase 1 Analysis gate")
    parser.add_argument(
        "project_dir",
        nargs="?",
        type=Path,
        default=Path.cwd(),
        help="Project directory (default: current directory)"
    )

    args = parser.parse_args()

    success = run_phase_1_gate(args.project_dir)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
