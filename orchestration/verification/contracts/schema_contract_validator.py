#!/usr/bin/env python3
"""
Schema-Contract Validator

Validates that API contracts specify ALL database schema requirements.

This would have caught the file_hash bug:
- Database schema: file_hash TEXT NOT NULL
- Contract: No mention of file_hash requirement
- Validator: Would detect mismatch and block

Part of v1.8.0 testing gap remediation.
"""

import sqlite3
import yaml
from pathlib import Path
from typing import Dict, List, Tuple, Set
from dataclasses import dataclass


@dataclass
class SchemaColumn:
    """Database column definition."""
    name: str
    type: str
    not_null: bool
    primary_key: bool


@dataclass
class SchemaMismatch:
    """Detected schema-contract mismatch."""
    severity: str  # 'critical', 'warning'
    column_name: str
    schema_definition: str
    contract_definition: str
    fix_suggestion: str


class SchemaContractValidator:
    """Validates database schemas against API contracts."""

    def __init__(self, component_path: Path):
        self.component_path = Path(component_path).resolve()
        self.mismatches: List[SchemaMismatch] = []

    def validate(self) -> Tuple[bool, List[SchemaMismatch]]:
        """
        Validate all schemas against contracts.

        Returns:
            (is_valid, mismatches)
        """
        # Find database files
        db_files = self._find_database_files()

        if not db_files:
            # No databases found - not applicable
            return True, []

        # Find contract files
        contract_files = self._find_contract_files()

        if not contract_files:
            # No contracts found - warning
            self.mismatches.append(SchemaMismatch(
                severity='warning',
                column_name='N/A',
                schema_definition='Database exists',
                contract_definition='No contract found',
                fix_suggestion='Create contract file in contracts/ directory'
            ))
            return False, self.mismatches

        # Validate each database
        for db_file in db_files:
            self._validate_database(db_file, contract_files)

        # Return result
        critical_mismatches = [m for m in self.mismatches if m.severity == 'critical']
        is_valid = len(critical_mismatches) == 0

        return is_valid, self.mismatches

    def _find_database_files(self) -> List[Path]:
        """Find SQLite database files in component."""
        db_files = []

        # Search patterns
        patterns = ['**/*.db', '**/*.sqlite', '**/*.sqlite3']

        for pattern in patterns:
            db_files.extend(self.component_path.glob(pattern))

        # Filter out test databases
        db_files = [
            f for f in db_files
            if 'test' not in f.name.lower() and '.pytest_cache' not in str(f)
        ]

        return db_files

    def _find_contract_files(self) -> List[Path]:
        """Find contract YAML files."""
        contract_dirs = [
            self.component_path / "contracts",
            self.component_path.parent.parent / "contracts",  # Project root
        ]

        contract_files = []
        for contract_dir in contract_dirs:
            if contract_dir.exists():
                contract_files.extend(contract_dir.glob("*.yaml"))
                contract_files.extend(contract_dir.glob("*.yml"))

        return contract_files

    def _validate_database(self, db_file: Path, contract_files: List[Path]):
        """Validate one database against contracts."""
        # Extract schema
        schema = self._extract_schema(db_file)

        # Find relevant contract
        relevant_contracts = self._find_relevant_contracts(contract_files)

        if not relevant_contracts:
            return  # No relevant contract found

        # Validate each table
        for table_name, columns in schema.items():
            self._validate_table(table_name, columns, relevant_contracts)

    def _extract_schema(self, db_file: Path) -> Dict[str, List[SchemaColumn]]:
        """Extract schema from SQLite database."""
        schema = {}

        try:
            conn = sqlite3.connect(db_file)
            cursor = conn.cursor()

            # Get all tables
            cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
            tables = cursor.fetchall()

            for (table_name,) in tables:
                # Get table info
                cursor.execute(f"PRAGMA table_info({table_name})")
                table_info = cursor.fetchall()

                columns = []
                for col_info in table_info:
                    columns.append(SchemaColumn(
                        name=col_info[1],
                        type=col_info[2],
                        not_null=bool(col_info[3]),
                        primary_key=bool(col_info[5])
                    ))

                schema[table_name] = columns

            conn.close()

        except Exception as e:
            print(f"Warning: Could not extract schema from {db_file}: {e}")

        return schema

    def _find_relevant_contracts(self, contract_files: List[Path]) -> List[Dict]:
        """Find contracts relevant to this component."""
        contracts = []

        for contract_file in contract_files:
            try:
                with open(contract_file) as f:
                    contract = yaml.safe_load(f)

                # Check if contract mentions data persistence
                if self._contract_has_data_persistence(contract):
                    contracts.append(contract)

            except Exception as e:
                print(f"Warning: Could not parse {contract_file}: {e}")

        return contracts

    def _contract_has_data_persistence(self, contract: Dict) -> bool:
        """Check if contract involves data persistence."""
        persistence_keywords = ['save', 'insert', 'update', 'store', 'persist', 'write']
        contract_str = str(contract).lower()
        return any(keyword in contract_str for keyword in persistence_keywords)

    def _validate_table(
        self,
        table_name: str,
        columns: List[SchemaColumn],
        contracts: List[Dict]
    ):
        """Validate table schema against contracts."""
        # Get NOT NULL columns from schema
        not_null_columns = [col for col in columns if col.not_null and not col.primary_key]

        # Find save/persist methods in contracts
        for contract in contracts:
            methods = contract.get('methods', [])

            for method in methods:
                if isinstance(method, dict):
                    method_name = method.get('name', '')

                    if 'save' in method_name or 'insert' in method_name:
                        # Check if contract specifies required fields
                        params = method.get('params', [])

                        for param in params:
                            if isinstance(param, dict):
                                # Look for results/data parameter
                                for param_name, param_def in param.items():
                                    if param_name in ['results', 'data', 'record']:
                                        if isinstance(param_def, dict):
                                            required_fields = param_def.get('required_fields', {})

                                            # Check each NOT NULL column
                                            for col in not_null_columns:
                                                if col.name not in required_fields:
                                                    self.mismatches.append(SchemaMismatch(
                                                        severity='critical',
                                                        column_name=col.name,
                                                        schema_definition=f"{col.name} {col.type} NOT NULL",
                                                        contract_definition="Not specified in required_fields",
                                                        fix_suggestion=f"Add '{col.name}' to required_fields in {method_name}() contract"
                                                    ))

    def generate_report(self) -> str:
        """Generate human-readable validation report."""
        if not self.mismatches:
            return "✅ All schemas match contracts"

        report = []
        report.append("❌ SCHEMA-CONTRACT MISMATCHES DETECTED")
        report.append("=" * 60)
        report.append("")

        critical = [m for m in self.mismatches if m.severity == 'critical']
        warnings = [m for m in self.mismatches if m.severity == 'warning']

        if critical:
            report.append(f"CRITICAL ISSUES: {len(critical)}")
            report.append("-" * 60)
            for mismatch in critical:
                report.append(f"\n❌ Missing Required Field: {mismatch.column_name}")
                report.append(f"   Schema:   {mismatch.schema_definition}")
                report.append(f"   Contract: {mismatch.contract_definition}")
                report.append(f"   Fix:      {mismatch.fix_suggestion}")

        if warnings:
            report.append(f"\nWARNINGS: {len(warnings)}")
            report.append("-" * 60)
            for mismatch in warnings:
                report.append(f"\n⚠️  {mismatch.column_name}")
                report.append(f"   {mismatch.fix_suggestion}")

        report.append("")
        report.append("=" * 60)
        report.append("IMPACT: Schema-contract mismatches cause runtime failures")
        report.append("        that integration tests may not catch.")
        report.append("")
        report.append("Example: Music Analyzer file_hash bug")
        report.append("  - Schema: file_hash TEXT NOT NULL")
        report.append("  - Contract: No mention of file_hash")
        report.append("  - Result: CLI didn't include file_hash, database rejected inserts")
        report.append("")

        return "\n".join(report)


def main():
    """CLI entry point."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python schema_contract_validator.py <component_path>")
        sys.exit(1)

    component_path = Path(sys.argv[1])

    validator = SchemaContractValidator(component_path)
    is_valid, mismatches = validator.validate()

    print(validator.generate_report())

    sys.exit(0 if is_valid else 1)


if __name__ == '__main__':
    main()
