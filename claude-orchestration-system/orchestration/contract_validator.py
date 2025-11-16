#!/usr/bin/env python3
"""
Contract Validator

Validates API contracts (OpenAPI, gRPC, AsyncAPI, Data Contracts) for the
Claude Code Orchestration System.

Usage:
    python orchestration/contract_validator.py validate contracts/my-api.yaml
    python orchestration/contract_validator.py validate-all
    python orchestration/contract_validator.py check-variables contracts/my-api.yaml
"""

import sys
import json
import yaml
import re
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple
from dataclasses import dataclass
from enum import Enum


class ContractType(Enum):
    """Types of contracts supported"""
    OPENAPI = "openapi"
    GRPC = "grpc"
    ASYNCAPI = "asyncapi"
    DATA = "data"
    UNKNOWN = "unknown"


@dataclass
class ValidationResult:
    """Result of contract validation"""
    valid: bool
    contract_type: ContractType
    errors: List[str]
    warnings: List[str]
    info: List[str]


class ContractValidator:
    """Validates various types of API contracts"""

    def __init__(self):
        self.strict_mode = False

    def detect_contract_type(self, content: str, file_path: Path) -> ContractType:
        """
        Detect the type of contract based on content and file extension.

        Args:
            content: Contract file content
            file_path: Path to contract file

        Returns:
            Detected contract type
        """
        # Check file extension
        suffix = file_path.suffix.lower()
        if suffix == '.proto':
            return ContractType.GRPC

        # Try parsing as YAML
        try:
            data = yaml.safe_load(content)
        except yaml.YAMLError:
            return ContractType.UNKNOWN

        if not isinstance(data, dict):
            return ContractType.UNKNOWN

        # Check for OpenAPI markers
        if 'openapi' in data:
            return ContractType.OPENAPI

        # Check for AsyncAPI markers
        if 'asyncapi' in data:
            return ContractType.ASYNCAPI

        # Check for data contract markers
        if 'database' in data or 'quality' in data:
            return ContractType.DATA

        return ContractType.UNKNOWN

    def validate_contract(self, file_path: Path) -> ValidationResult:
        """
        Validate a contract file.

        Args:
            file_path: Path to contract file

        Returns:
            ValidationResult with errors, warnings, and info
        """
        errors = []
        warnings = []
        info = []

        # Check file exists
        if not file_path.exists():
            return ValidationResult(
                valid=False,
                contract_type=ContractType.UNKNOWN,
                errors=[f"File not found: {file_path}"],
                warnings=[],
                info=[]
            )

        # Read content
        try:
            content = file_path.read_text()
        except Exception as e:
            return ValidationResult(
                valid=False,
                contract_type=ContractType.UNKNOWN,
                errors=[f"Error reading file: {e}"],
                warnings=[],
                info=[]
            )

        # Detect contract type
        contract_type = self.detect_contract_type(content, file_path)
        info.append(f"Detected contract type: {contract_type.value}")

        # Validate based on type
        if contract_type == ContractType.OPENAPI:
            errors.extend(self._validate_openapi(content))
        elif contract_type == ContractType.GRPC:
            errors.extend(self._validate_grpc(content))
        elif contract_type == ContractType.ASYNCAPI:
            errors.extend(self._validate_asyncapi(content))
        elif contract_type == ContractType.DATA:
            errors.extend(self._validate_data_contract(content))
        else:
            errors.append("Unknown contract type")

        # Check for unreplaced variables
        unreplaced = self._find_unreplaced_variables(content)
        if unreplaced:
            warnings.append(f"Found {len(unreplaced)} unreplaced variables: {', '.join(list(unreplaced)[:5])}")
            if len(unreplaced) > 5:
                warnings.append(f"  ... and {len(unreplaced) - 5} more")

        return ValidationResult(
            valid=len(errors) == 0,
            contract_type=contract_type,
            errors=errors,
            warnings=warnings,
            info=info
        )

    def _validate_openapi(self, content: str) -> List[str]:
        """Validate OpenAPI contract"""
        errors = []

        try:
            data = yaml.safe_load(content)
        except yaml.YAMLError as e:
            return [f"Invalid YAML: {e}"]

        # Check required top-level fields
        required_fields = ['openapi', 'info', 'paths']
        for field in required_fields:
            if field not in data:
                errors.append(f"Missing required field: {field}")

        # Check OpenAPI version
        if 'openapi' in data:
            version = data['openapi']
            if not isinstance(version, str) or not version.startswith('3.'):
                errors.append(f"Unsupported OpenAPI version: {version} (expected 3.x)")

        # Check info section
        if 'info' in data:
            info = data['info']
            if 'title' not in info:
                errors.append("Missing info.title")
            if 'version' not in info:
                errors.append("Missing info.version")

        # Check paths
        if 'paths' in data:
            paths = data['paths']
            if not isinstance(paths, dict) or len(paths) == 0:
                errors.append("No paths defined")

            # Check for health endpoint
            if '/health' not in paths and '/health/ready' not in paths:
                if self.strict_mode:
                    errors.append("No health check endpoint defined")

        return errors

    def _validate_grpc(self, content: str) -> List[str]:
        """Validate gRPC protobuf contract"""
        errors = []

        # Check syntax declaration
        if not re.search(r'syntax\s*=\s*"proto3";', content):
            errors.append("Missing or invalid syntax declaration (expected proto3)")

        # Check package declaration
        if not re.search(r'package\s+[\w.]+;', content):
            errors.append("Missing package declaration")

        # Check service definition
        if not re.search(r'service\s+\w+\s*{', content):
            errors.append("No service definition found")

        # Check for at least one RPC method
        if not re.search(r'rpc\s+\w+', content):
            errors.append("No RPC methods defined")

        # Check for message definitions
        if not re.search(r'message\s+\w+\s*{', content):
            errors.append("No message definitions found")

        return errors

    def _validate_asyncapi(self, content: str) -> List[str]:
        """Validate AsyncAPI contract"""
        errors = []

        try:
            data = yaml.safe_load(content)
        except yaml.YAMLError as e:
            return [f"Invalid YAML: {e}"]

        # Check required fields
        required_fields = ['asyncapi', 'info', 'channels']
        for field in required_fields:
            if field not in data:
                errors.append(f"Missing required field: {field}")

        # Check AsyncAPI version
        if 'asyncapi' in data:
            version = data['asyncapi']
            if not isinstance(version, str) or not version.startswith('2.'):
                errors.append(f"Unsupported AsyncAPI version: {version} (expected 2.x)")

        # Check channels
        if 'channels' in data:
            channels = data['channels']
            if not isinstance(channels, dict) or len(channels) == 0:
                errors.append("No channels defined")

        return errors

    def _validate_data_contract(self, content: str) -> List[str]:
        """Validate data contract"""
        errors = []

        try:
            data = yaml.safe_load(content)
        except yaml.YAMLError as e:
            return [f"Invalid YAML: {e}"]

        # Check metadata
        if 'metadata' not in data:
            errors.append("Missing metadata section")
        else:
            metadata = data['metadata']
            required_meta = ['name', 'version', 'owner']
            for field in required_meta:
                if field not in metadata:
                    errors.append(f"Missing metadata.{field}")

        # Check database section
        if 'database' in data:
            db = data['database']
            if 'type' not in db:
                errors.append("Missing database.type")
            if 'tables' not in db or not isinstance(db['tables'], list):
                errors.append("Missing or invalid database.tables")

        return errors

    def _find_unreplaced_variables(self, content: str) -> Set[str]:
        """Find unreplaced template variables"""
        pattern = r'\{\{([^}]+)\}\}'
        matches = re.findall(pattern, content)
        return set(matches)

    def validate_all_contracts(self, contracts_dir: Path) -> Dict[str, ValidationResult]:
        """
        Validate all contracts in a directory.

        Args:
            contracts_dir: Directory containing contracts

        Returns:
            Dictionary mapping file names to validation results
        """
        results = {}

        if not contracts_dir.exists():
            print(f"Contracts directory not found: {contracts_dir}")
            return results

        # Find all contract files
        patterns = ['*.yaml', '*.yml', '*.proto']
        for pattern in patterns:
            for file_path in contracts_dir.glob(pattern):
                # Skip template files
                if 'template' in file_path.name:
                    continue

                result = self.validate_contract(file_path)
                results[file_path.name] = result

        return results

    def check_variables(self, file_path: Path) -> Set[str]:
        """
        Check for unreplaced variables in a contract file.

        Args:
            file_path: Path to contract file

        Returns:
            Set of unreplaced variable names
        """
        if not file_path.exists():
            return set()

        content = file_path.read_text()
        return self._find_unreplaced_variables(content)


def print_validation_result(file_name: str, result: ValidationResult):
    """Print formatted validation result"""
    print(f"\n{'='*80}")
    print(f"File: {file_name}")
    print(f"Type: {result.contract_type.value}")
    print(f"Valid: {'✓' if result.valid else '✗'}")

    if result.info:
        print("\nInfo:")
        for msg in result.info:
            print(f"  ℹ {msg}")

    if result.warnings:
        print("\nWarnings:")
        for msg in result.warnings:
            print(f"  ⚠ {msg}")

    if result.errors:
        print("\nErrors:")
        for msg in result.errors:
            print(f"  ✗ {msg}")

    print('='*80)


def main():
    """Main CLI interface"""
    import argparse

    parser = argparse.ArgumentParser(
        description="Validate API contracts for Claude Code Orchestration System"
    )
    subparsers = parser.add_subparsers(dest='command', help='Command to execute')

    # Validate command
    validate_parser = subparsers.add_parser('validate', help='Validate a single contract')
    validate_parser.add_argument('file', type=Path, help='Path to contract file')
    validate_parser.add_argument('--strict', action='store_true', help='Enable strict mode')

    # Validate all command
    validate_all_parser = subparsers.add_parser('validate-all', help='Validate all contracts')
    validate_all_parser.add_argument(
        '--dir',
        type=Path,
        default=Path('contracts'),
        help='Contracts directory (default: contracts/)'
    )
    validate_all_parser.add_argument('--strict', action='store_true', help='Enable strict mode')

    # Check variables command
    check_vars_parser = subparsers.add_parser(
        'check-variables',
        help='Check for unreplaced template variables'
    )
    check_vars_parser.add_argument('file', type=Path, help='Path to contract file')

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 1

    validator = ContractValidator()

    if args.command == 'validate':
        validator.strict_mode = getattr(args, 'strict', False)
        result = validator.validate_contract(args.file)
        print_validation_result(str(args.file), result)
        return 0 if result.valid else 1

    elif args.command == 'validate-all':
        validator.strict_mode = getattr(args, 'strict', False)
        results = validator.validate_all_contracts(args.dir)

        if not results:
            print(f"No contracts found in {args.dir}")
            return 1

        valid_count = sum(1 for r in results.values() if r.valid)
        total_count = len(results)

        for file_name, result in results.items():
            print_validation_result(file_name, result)

        print(f"\n{'='*80}")
        print(f"Summary: {valid_count}/{total_count} contracts valid")
        print('='*80)

        return 0 if valid_count == total_count else 1

    elif args.command == 'check-variables':
        variables = validator.check_variables(args.file)
        if variables:
            print(f"\nFound {len(variables)} unreplaced variables in {args.file}:")
            for var in sorted(variables):
                print(f"  {{{{{{var}}}}}}")
            return 1
        else:
            print(f"\n✓ No unreplaced variables in {args.file}")
            return 0

    return 0


if __name__ == '__main__':
    sys.exit(main())
