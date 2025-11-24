"""
Pre-flight Validation System

Runs before orchestration starts to detect blocking issues.

Usage:
    from orchestration.core.preflight_checks import run_preflight_checks

    if not run_preflight_checks():
        sys.exit(1)
"""

import sys
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Optional

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.verification.system.component_name_validator import ComponentNameValidator


@dataclass
class ValidationResult:
    """Result of a validation check."""
    passed: bool
    check_name: str
    message: str
    blocker: bool = False
    auto_fix_available: bool = False
    details: Optional[Dict] = None


class ComponentNamingCheck:
    """Check all component names follow convention."""

    def __init__(self):
        self.validator = ComponentNameValidator()

    def run(self, project_dir: Path) -> ValidationResult:
        """Run component naming validation."""
        components_dir = project_dir / "components"

        if not components_dir.exists():
            return ValidationResult(
                passed=True,
                check_name="Component Naming",
                message="No components directory (new project)"
            )

        # Scan all component directories
        invalid_components = {}
        for comp_dir in components_dir.iterdir():
            if not comp_dir.is_dir():
                continue

            if comp_dir.name.startswith('.'):
                continue  # Skip hidden directories

            result = self.validator.validate(comp_dir.name)
            if not result.is_valid:
                invalid_components[comp_dir.name] = {
                    'error': result.error_message,
                    'suggestion': result.suggestion
                }

        if invalid_components:
            message = f"Found {len(invalid_components)} components with invalid names"
            return ValidationResult(
                passed=False,
                check_name="Component Naming",
                message=message,
                blocker=True,  # Blocks orchestration
                auto_fix_available=True,
                details=invalid_components
            )

        total_components = len([d for d in components_dir.iterdir()
                               if d.is_dir() and not d.name.startswith('.')])
        return ValidationResult(
            passed=True,
            check_name="Component Naming",
            message=f"All {total_components} components have valid names"
        )


class PreflightValidator:
    """Run all pre-flight checks."""

    def __init__(self, project_dir: Optional[Path] = None):
        self.project_dir = project_dir or Path.cwd()
        self.checks = [
            ComponentNamingCheck(),
            # Add more checks here as needed
        ]

    def run_all(self) -> List[ValidationResult]:
        """Run all pre-flight checks."""
        results = []
        for check in self.checks:
            result = check.run(self.project_dir)
            results.append(result)
        return results

    def has_blockers(self, results: List[ValidationResult]) -> bool:
        """Check if any results are blocking."""
        return any(r.blocker and not r.passed for r in results)

    def report(self, results: List[ValidationResult]):
        """Print formatted report."""
        print("=" * 60)
        print("PRE-FLIGHT VALIDATION")
        print("=" * 60)

        for result in results:
            icon = "✅" if result.passed else "❌"
            print(f"\n{icon} {result.check_name}: {result.message}")

            if not result.passed and result.details:
                print()
                for name, info in result.details.items():
                    print(f"   Component: {name}")
                    print(f"     Error: {info['error']}")
                    if info.get('suggestion'):
                        print(f"     Suggestion: {info['suggestion']}")

        print("\n" + "=" * 60)


def run_preflight_checks(project_dir: Optional[Path] = None) -> bool:
    """
    Run pre-flight checks and return whether orchestration can proceed.

    Args:
        project_dir: Project directory (default: current directory)

    Returns:
        True if safe to proceed, False if blockers found
    """
    validator = PreflightValidator(project_dir)
    results = validator.run_all()
    validator.report(results)

    if validator.has_blockers(results):
        print("\n❌ Pre-flight checks failed - cannot proceed with orchestration")
        print("   Fix issues or enable auto-fix in orchestration-config.json")
        return False

    print("\n✅ Pre-flight checks passed - safe to proceed")
    return True


def main():
    """CLI interface for pre-flight checks."""
    import argparse

    parser = argparse.ArgumentParser(description="Run pre-flight validation checks")
    parser.add_argument(
        "--project-dir",
        type=Path,
        default=Path.cwd(),
        help="Project directory (default: current directory)"
    )

    args = parser.parse_args()

    success = run_preflight_checks(args.project_dir)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
