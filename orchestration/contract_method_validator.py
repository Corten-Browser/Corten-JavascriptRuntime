#!/usr/bin/env python3
"""
Contract Method Validator

Validates that component code calls methods that exist in dependency contracts.

Part of the adaptive orchestration system (v0.10.0).
Prevents API mismatch errors like loader.load() vs loader.load_audio().

Note: This complements contract_enforcer.py (v0.4.0) which ensures contracts exist.
This validator checks that method calls in code match the contract specifications.
"""

from pathlib import Path
from typing import List, Dict, Set, Optional
from dataclasses import dataclass
import re
import yaml


@dataclass
class ContractViolation:
    """Represents a contract violation."""
    component: str
    dependency: str
    called_method: str
    location: str  # file:line
    available_methods: List[str]
    suggestion: Optional[str] = None


class ContractMethodValidator:
    """Validates method calls match contracts."""

    def __init__(self, project_root: Path):
        """
        Initialize contract method validator.

        Args:
            project_root: Path to project root directory
        """
        self.project_root = Path(project_root).resolve()
        self.contracts_dir = self.project_root / "contracts"
        self.components_dir = self.project_root / "components"

    def validate_component(self, component_name: str) -> List[ContractViolation]:
        """
        Validate a component's method calls against contracts.

        Args:
            component_name: Name of component to validate

        Returns:
            List of contract violations found
        """
        violations = []

        component_dir = self.components_dir / component_name
        if not component_dir.exists():
            return violations

        # Find all Python files
        for py_file in component_dir.rglob("*.py"):
            if "test" in str(py_file):
                continue  # Skip test files

            violations.extend(self._validate_file(component_name, py_file))

        return violations

    def _validate_file(self, component_name: str, file_path: Path) -> List[ContractViolation]:
        """Validate a single Python file."""
        violations = []

        try:
            content = file_path.read_text()

            # Extract method calls (simplified pattern)
            # Pattern: self.dependency.method_name(
            pattern = r'self\.(\w+)\.(\w+)\('

            for match in re.finditer(pattern, content):
                dependency = match.group(1)
                method_name = match.group(2)

                # Check if this is a known dependency
                contract = self._find_contract(dependency)
                if not contract:
                    continue  # Not a contracted dependency

                # Check if method exists in contract
                contract_methods = self._extract_contract_methods(contract)

                if method_name not in contract_methods:
                    line_num = self._get_line_number(content, match.start())

                    violations.append(ContractViolation(
                        component=component_name,
                        dependency=dependency,
                        called_method=method_name,
                        location=f"{file_path.name}:{line_num}",
                        available_methods=sorted(contract_methods),
                        suggestion=self._find_similar_method(method_name, contract_methods)
                    ))

        except Exception:
            pass  # Skip files that can't be parsed

        return violations

    def _find_contract(self, dependency_name: str) -> Optional[Path]:
        """Find contract file for a dependency."""
        # Try common naming patterns
        patterns = [
            f"{dependency_name}.yaml",
            f"{dependency_name}-api.yaml",
            f"{dependency_name.replace('_', '-')}.yaml",
        ]

        for pattern in patterns:
            contract_file = self.contracts_dir / pattern
            if contract_file.exists():
                return contract_file

        return None

    def _extract_contract_methods(self, contract_path: Path) -> Set[str]:
        """Extract method names from OpenAPI contract."""
        methods = set()

        try:
            contract = yaml.safe_load(contract_path.read_text())

            # For OpenAPI 3.0 contracts
            if 'paths' in contract:
                for path, path_item in contract['paths'].items():
                    # operationId is usually the method name
                    for method, operation in path_item.items():
                        if isinstance(operation, dict) and 'operationId' in operation:
                            methods.add(operation['operationId'])

            # Could also parse components/schemas for additional methods

        except Exception:
            pass

        return methods

    def _find_similar_method(self, method_name: str, available: Set[str]) -> Optional[str]:
        """Find similar method name (simple edit distance)."""
        # Simplified: just check for substring match
        for available_method in available:
            if method_name.lower() in available_method.lower() or \
               available_method.lower() in method_name.lower():
                return available_method

        return None

    def _get_line_number(self, content: str, position: int) -> int:
        """Get line number for a character position."""
        return content[:position].count('\n') + 1

    def print_report(self, violations: List[ContractViolation]) -> None:
        """Print validation report."""
        if not violations:
            print("✅ Contract method validation PASSED")
            print("   All method calls match contract specifications")
            return

        print("❌ Contract method validation FAILED")
        print(f"   Found {len(violations)} contract violations")
        print()

        for violation in violations:
            print(f"[{violation.component}] {violation.location}")
            print(f"  Called: {violation.dependency}.{violation.called_method}()")
            print(f"  Problem: Method not in {violation.dependency} contract")
            print(f"  Available methods: {', '.join(violation.available_methods[:5])}")

            if violation.suggestion:
                print(f"  💡 Did you mean: {violation.suggestion}()?")

            print()

        print("=" * 60)
        print("RESULT: BLOCKED - Fix contract violations before proceeding")


def main():
    """CLI interface."""
    import sys
    import argparse

    parser = argparse.ArgumentParser(
        description='Validate component method calls match contracts'
    )
    parser.add_argument(
        '--component',
        required=True,
        help='Component name to validate'
    )
    parser.add_argument(
        '--project-root',
        type=Path,
        default=Path.cwd(),
        help='Project root directory'
    )

    args = parser.parse_args()

    validator = ContractMethodValidator(args.project_root)
    violations = validator.validate_component(args.component)

    validator.print_report(violations)

    sys.exit(1 if violations else 0)


if __name__ == '__main__':
    main()
