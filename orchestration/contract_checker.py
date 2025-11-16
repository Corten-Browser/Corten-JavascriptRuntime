"""
Contract Compatibility Checker

Analyzes OpenAPI contracts to detect incompatibilities between services.
Provides CLI and programmatic interface for contract validation.

Classes:
    Incompatibility: Represents a contract incompatibility
    CompatibilityReport: Report of all compatibility checks
    ContractChecker: Main checker class

Usage (CLI):
    python contract_checker.py                    # Check contracts/ directory
    python contract_checker.py /path/to/contracts # Check specific directory
    python contract_checker.py --json             # Output as JSON
    python contract_checker.py --output report.txt # Write to file

Usage (Programmatic):
    from contract_checker import ContractChecker

    checker = ContractChecker(Path('contracts'))
    checker.load_contracts()
    checker.detect_dependencies()
    report = checker.check_compatibility()

    if report.has_incompatibilities:
        print(f"Found {len(report.incompatibilities)} issues")
"""

from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, field
import sys
import argparse

from contract_analyzer import (
    load_openapi_spec,
    extract_schema,
    compare_schemas,
    SchemaComparison
)


@dataclass
class Incompatibility:
    """Represents a contract incompatibility."""
    provider_contract: str
    consumer_contract: str
    provider_endpoint: str
    consumer_endpoint: Optional[str]
    schema_name: str
    issues: List[Dict]
    severity: str = "error"  # error, warning

    def __str__(self):
        return (f"{self.provider_contract} → {self.consumer_contract}: "
                f"{len(self.issues)} issue(s) in {self.schema_name}")


@dataclass
class CompatibilityReport:
    """Report of all contract compatibility checks."""
    contracts_checked: List[str] = field(default_factory=list)
    pairs_checked: int = 0
    compatible_pairs: int = 0
    incompatibilities: List[Incompatibility] = field(default_factory=list)

    @property
    def has_incompatibilities(self) -> bool:
        """Check if report contains any incompatibilities."""
        return len(self.incompatibilities) > 0

    @property
    def success_rate(self) -> float:
        """Calculate percentage of compatible pairs."""
        if self.pairs_checked == 0:
            return 100.0
        return (self.compatible_pairs / self.pairs_checked) * 100

    def to_dict(self) -> Dict:
        """Convert report to dictionary."""
        return {
            'contracts_checked': self.contracts_checked,
            'pairs_checked': self.pairs_checked,
            'compatible_pairs': self.compatible_pairs,
            'incompatibilities': [
                {
                    'provider': incomp.provider_contract,
                    'consumer': incomp.consumer_contract,
                    'schema': incomp.schema_name,
                    'issues': incomp.issues,
                    'severity': incomp.severity
                }
                for incomp in self.incompatibilities
            ],
            'success_rate': self.success_rate
        }


class ContractChecker:
    """Check OpenAPI contract compatibility."""

    def __init__(self, contracts_dir: Path):
        """
        Initialize contract checker.

        Args:
            contracts_dir: Path to directory containing contract YAML files
        """
        self.contracts_dir = contracts_dir
        self.contracts: Dict[str, Dict] = {}
        self.dependencies: Dict[str, List[str]] = {}

    def load_contracts(self) -> int:
        """
        Load all OpenAPI contracts from contracts directory.

        Returns:
            Number of contracts loaded

        Example:
            >>> checker = ContractChecker(Path('contracts'))
            >>> count = checker.load_contracts()
            >>> print(f"Loaded {count} contracts")
        """
        contract_files = list(self.contracts_dir.glob('*.yaml')) + \
                        list(self.contracts_dir.glob('*.yml'))

        for contract_file in contract_files:
            try:
                spec = load_openapi_spec(contract_file)
                contract_name = contract_file.stem
                # Remove -api suffix if present
                if contract_name.endswith('-api'):
                    contract_name = contract_name[:-4]
                self.contracts[contract_name] = spec
            except Exception as e:
                print(f"Warning: Could not load {contract_file.name}: {e}")

        return len(self.contracts)

    def detect_dependencies(self):
        """
        Detect dependencies between services based on contract metadata.

        Looks for x-depends-on or x-calls extensions in OpenAPI specs.

        Example:
            In auth-api.yaml:
            ```yaml
            info:
              x-depends-on:
                - user-service
                - email-service
            ```
        """
        for contract_name, spec in self.contracts.items():
            depends_on = []

            # Check for x-depends-on in info section
            if 'info' in spec and 'x-depends-on' in spec['info']:
                depends_on.extend(spec['info']['x-depends-on'])

            # Check for x-calls in endpoints
            if 'paths' in spec:
                for path, methods in spec['paths'].items():
                    for method, endpoint in methods.items():
                        if isinstance(endpoint, dict) and 'x-calls' in endpoint:
                            depends_on.append(endpoint['x-calls'])

            if depends_on:
                self.dependencies[contract_name] = list(set(depends_on))

    def check_compatibility(self) -> CompatibilityReport:
        """
        Check compatibility of all contract pairs.

        Returns:
            CompatibilityReport with findings

        Example:
            >>> report = checker.check_compatibility()
            >>> if report.has_incompatibilities:
            ...     print(f"Found {len(report.incompatibilities)} issues")
        """
        report = CompatibilityReport()
        report.contracts_checked = list(self.contracts.keys())

        # Check each dependency
        for consumer_name, providers in self.dependencies.items():
            if consumer_name not in self.contracts:
                continue

            consumer_spec = self.contracts[consumer_name]

            for provider_name in providers:
                if provider_name not in self.contracts:
                    print(f"Warning: {consumer_name} depends on {provider_name} "
                          f"but contract not found")
                    continue

                provider_spec = self.contracts[provider_name]
                report.pairs_checked += 1

                # Check schema compatibility
                incompatibilities = self._check_pair_compatibility(
                    provider_name, provider_spec,
                    consumer_name, consumer_spec
                )

                if incompatibilities:
                    report.incompatibilities.extend(incompatibilities)
                else:
                    report.compatible_pairs += 1

        return report

    def _check_pair_compatibility(
        self,
        provider_name: str,
        provider_spec: Dict,
        consumer_name: str,
        consumer_spec: Dict
    ) -> List[Incompatibility]:
        """
        Check compatibility between a provider-consumer pair.

        Args:
            provider_name: Name of provider service
            provider_spec: OpenAPI spec for provider
            consumer_name: Name of consumer service
            consumer_spec: OpenAPI spec for consumer

        Returns:
            List of incompatibilities found
        """
        incompatibilities = []

        # Get common schemas (schemas with same name in both contracts)
        provider_schemas = provider_spec.get('components', {}).get('schemas', {})
        consumer_schemas = consumer_spec.get('components', {}).get('schemas', {})

        common_schemas = set(provider_schemas.keys()) & set(consumer_schemas.keys())

        for schema_name in common_schemas:
            provider_schema = provider_schemas[schema_name]
            consumer_schema = consumer_schemas[schema_name]

            comparison = compare_schemas(provider_schema, consumer_schema)

            if not comparison.compatible:
                issues = []

                # Missing required fields
                if comparison.missing_required:
                    issues.append({
                        'type': 'missing_required_fields',
                        'fields': comparison.missing_required,
                        'message': f"Consumer requires fields not provided: "
                                  f"{', '.join(comparison.missing_required)}"
                    })

                # Field name mismatches
                if comparison.field_mismatches:
                    for mismatch in comparison.field_mismatches:
                        issues.append({
                            'type': 'field_name_mismatch',
                            'consumer_expects': mismatch['consumer_expects'],
                            'provider_has': mismatch['provider_has'],
                            'message': f"Field name mismatch: consumer expects "
                                      f"'{mismatch['consumer_expects']}' but "
                                      f"provider has '{mismatch['provider_has']}'"
                        })

                # Type mismatches
                if comparison.type_mismatches:
                    for mismatch in comparison.type_mismatches:
                        issues.append({
                            'type': 'type_mismatch',
                            'field': mismatch['field'],
                            'provider_type': mismatch['provider_type'],
                            'consumer_type': mismatch['consumer_type'],
                            'message': f"Type mismatch for '{mismatch['field']}': "
                                      f"provider={mismatch['provider_type']}, "
                                      f"consumer={mismatch['consumer_type']}"
                        })

                incompatibility = Incompatibility(
                    provider_contract=f"{provider_name}-api.yaml",
                    consumer_contract=f"{consumer_name}-api.yaml",
                    provider_endpoint="(schema level)",
                    consumer_endpoint=None,
                    schema_name=schema_name,
                    issues=issues
                )
                incompatibilities.append(incompatibility)

        return incompatibilities

    def generate_report(self, report: CompatibilityReport) -> str:
        """
        Generate human-readable compatibility report.

        Args:
            report: CompatibilityReport to format

        Returns:
            Formatted report string

        Example:
            >>> report = checker.check_compatibility()
            >>> print(checker.generate_report(report))
        """
        lines = []
        lines.append("=" * 60)
        lines.append("CONTRACT COMPATIBILITY REPORT")
        lines.append("=" * 60)
        lines.append("")

        lines.append(f"📋 Contracts analyzed: {len(report.contracts_checked)}")
        for contract in sorted(report.contracts_checked):
            lines.append(f"   - {contract}-api.yaml")
        lines.append("")

        lines.append(f"🔗 Contract pairs checked: {report.pairs_checked}")
        lines.append(f"✅ Compatible pairs: {report.compatible_pairs}")
        lines.append(f"❌ Incompatible pairs: {len(report.incompatibilities)}")
        lines.append(f"📊 Success rate: {report.success_rate:.1f}%")
        lines.append("")

        if not report.has_incompatibilities:
            lines.append("🎉 All contracts are compatible!")
            lines.append("")
            lines.append("No incompatibilities found. All provider-consumer pairs")
            lines.append("have matching schemas, field names, and types.")
        else:
            lines.append("❌ INCOMPATIBILITIES DETECTED")
            lines.append("")

            for i, incomp in enumerate(report.incompatibilities, 1):
                lines.append(f"Issue #{i}: {incomp.provider_contract} → "
                           f"{incomp.consumer_contract}")
                lines.append(f"  Schema: {incomp.schema_name}")
                lines.append(f"  Issues: {len(incomp.issues)}")
                lines.append("")

                for issue in incomp.issues:
                    lines.append(f"  • {issue['message']}")

                lines.append("")
                lines.append("  Recommendation:")
                if incomp.issues[0]['type'] == 'field_name_mismatch':
                    lines.append(f"    Standardize field names between "
                               f"{incomp.provider_contract} and "
                               f"{incomp.consumer_contract}")
                elif incomp.issues[0]['type'] == 'missing_required_fields':
                    lines.append(f"    Add missing fields to {incomp.provider_contract} "
                               f"or make them optional in {incomp.consumer_contract}")
                elif incomp.issues[0]['type'] == 'type_mismatch':
                    lines.append(f"    Align data types between contracts")
                lines.append("")
                lines.append("-" * 60)
                lines.append("")

        lines.append("=" * 60)

        return "\n".join(lines)


def main():
    """CLI entry point for contract checker."""
    parser = argparse.ArgumentParser(
        description="Check OpenAPI contract compatibility"
    )
    parser.add_argument(
        'contracts_dir',
        type=Path,
        nargs='?',
        default=Path('contracts'),
        help='Path to contracts directory (default: contracts/)'
    )
    parser.add_argument(
        '--output',
        '-o',
        type=Path,
        help='Write report to file instead of stdout'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output report as JSON'
    )

    args = parser.parse_args()

    # Check contracts directory exists
    if not args.contracts_dir.exists():
        print(f"Error: Contracts directory not found: {args.contracts_dir}")
        return 1

    # Run checker
    print(f"🔍 Analyzing contracts in {args.contracts_dir}...")
    print()

    checker = ContractChecker(args.contracts_dir)

    num_loaded = checker.load_contracts()
    if num_loaded == 0:
        print("Error: No contract files found (*.yaml, *.yml)")
        return 1

    print(f"Loaded {num_loaded} contracts")

    checker.detect_dependencies()
    print(f"Detected {len(checker.dependencies)} service dependencies")
    print()

    report = checker.check_compatibility()

    # Generate report
    if args.json:
        import json
        output = json.dumps(report.to_dict(), indent=2)
    else:
        output = checker.generate_report(report)

    # Write output
    if args.output:
        args.output.write_text(output)
        print(f"Report written to {args.output}")
    else:
        print(output)

    # Exit code
    return 1 if report.has_incompatibilities else 0


if __name__ == '__main__':
    sys.exit(main())
