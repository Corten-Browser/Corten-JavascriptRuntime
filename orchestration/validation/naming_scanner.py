"""
Component Naming Scanner

Scans project for component naming violations.

Usage:
    from orchestration.validation.naming_scanner import ComponentNamingScanner

    scanner = ComponentNamingScanner()
    violations = scanner.scan(".")

    if violations:
        scanner.show_detailed_analysis(violations)
"""

import sys
from pathlib import Path
from typing import Dict

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.verification.system.component_name_validator import ComponentNameValidator


class ComponentNamingScanner:
    """Scan for component naming violations."""

    def __init__(self):
        self.validator = ComponentNameValidator()

    def scan(self, project_dir: str = ".") -> Dict[str, Dict]:
        """
        Scan project for naming violations.

        Args:
            project_dir: Project directory to scan

        Returns:
            Dict mapping invalid names to their info:
            {
                "auth-service": {
                    "error": "cannot contain hyphens",
                    "suggestion": "auth_service",
                    "path": "components/auth-service"
                }
            }
        """
        violations = {}
        components_dir = Path(project_dir) / "components"

        if not components_dir.exists():
            return violations

        for comp_dir in components_dir.iterdir():
            if not comp_dir.is_dir() or comp_dir.name.startswith('.'):
                continue

            result = self.validator.validate(comp_dir.name)
            if not result.is_valid:
                violations[comp_dir.name] = {
                    'error': result.error_message,
                    'suggestion': result.suggestion,
                    'path': str(comp_dir)
                }

        return violations

    def show_detailed_analysis(self, violations: Dict):
        """Show detailed analysis of violations."""
        print()
        print("=" * 60)
        print("DETAILED NAMING VIOLATION ANALYSIS")
        print("=" * 60)

        for old_name, info in violations.items():
            print(f"\nComponent: {old_name}")
            print(f"  Location: {info['path']}")
            print(f"  Issue: {info['error']}")
            print(f"  Suggested name: {info['suggestion']}")

            # Estimate impact
            impact = self._estimate_impact(info['path'])
            print(f"  Impact:")
            print(f"    - Files in component: {impact['file_count']}")
            print(f"    - Estimated imports to update: {impact['import_estimate']}")

        print()
        print("=" * 60)
        print(f"Total violations: {len(violations)}")
        print()
        print("To fix automatically:")
        print("  python orchestration/migration/rename_components.py")
        print()

    def _estimate_impact(self, component_path: str) -> Dict:
        """Estimate migration impact for a component."""
        path = Path(component_path)

        # Count source files
        file_count = 0
        for ext in ['.py', '.js', '.ts', '.tsx', '.jsx', '.rs', '.go']:
            file_count += len(list(path.rglob(f"*{ext}")))

        # Rough estimate: each file might have 3 imports
        import_estimate = file_count * 3

        return {
            'file_count': file_count,
            'import_estimate': import_estimate
        }

    def get_summary(self, violations: Dict) -> str:
        """Get summary message for violations."""
        if not violations:
            return "✅ All component names are valid"

        count = len(violations)
        return f"❌ Found {count} component{'s' if count != 1 else ''} with invalid names"


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Scan for component naming violations")
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current directory)"
    )
    parser.add_argument(
        "--detailed",
        action="store_true",
        help="Show detailed analysis"
    )

    args = parser.parse_args()

    scanner = ComponentNamingScanner()
    violations = scanner.scan(args.project_dir)

    print(scanner.get_summary(violations))

    if violations:
        if args.detailed:
            scanner.show_detailed_analysis(violations)
        else:
            print()
            for old_name, info in violations.items():
                print(f"  {old_name} → {info['suggestion']}")
            print()
            print("Run with --detailed for more information")

        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()
