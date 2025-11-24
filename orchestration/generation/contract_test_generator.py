#!/usr/bin/env python3
"""
Contract Test Generator

Automatically generates contract tests from OpenAPI/YAML contract specifications.
This ensures components implement the EXACT APIs defined in their contracts.

Prevents "Music Analyzer Catastrophe" scenarios where components have wrong
method names (scan vs get_audio_files) that pass unit tests with mocks but
fail catastrophically in integration.

Usage:
    python orchestration/contract_test_generator.py contracts/backend-api.yaml components/backend-service
    python orchestration/contract_test_generator.py --all  # Generate for all contracts
"""

import sys
import yaml
from pathlib import Path
from typing import Dict, List, Any
import argparse


class ContractTestGenerator:
    """Generates contract compliance tests from OpenAPI/YAML specifications."""

    def __init__(self, project_root: Path = None):
        self.project_root = project_root or Path.cwd()
        self.contracts_dir = self.project_root / "contracts"
        self.components_dir = self.project_root / "components"

    def load_contract(self, contract_path: Path) -> Dict[str, Any]:
        """Load and parse OpenAPI/YAML contract."""
        with open(contract_path, 'r') as f:
            return yaml.safe_load(f)

    def extract_api_spec(self, contract: Dict[str, Any]) -> Dict[str, Any]:
        """
        Extract API specification from contract.

        Returns dict with:
        - service_name: Name of the service
        - class_name: Main class name
        - methods: List of {name, params, returns}
        """
        # This is a simplified parser - real implementation would handle
        # full OpenAPI 3.0 spec with all edge cases

        spec = {
            'service_name': contract.get('info', {}).get('title', 'Service'),
            'version': contract.get('info', {}).get('version', '1.0.0'),
            'class_name': None,
            'methods': []
        }

        # Extract from OpenAPI paths or custom format
        if 'paths' in contract:
            spec['methods'] = self._extract_from_openapi(contract)
        elif 'components' in contract:
            spec['methods'] = self._extract_from_components(contract)

        return spec

    def _extract_from_openapi(self, contract: Dict[str, Any]) -> List[Dict]:
        """Extract method specs from OpenAPI paths."""
        methods = []

        for path, path_spec in contract.get('paths', {}).items():
            for method, method_spec in path_spec.items():
                if method in ['get', 'post', 'put', 'delete', 'patch']:
                    methods.append({
                        'name': method_spec.get('operationId', path.split('/')[-1]),
                        'http_method': method.upper(),
                        'path': path,
                        'parameters': self._extract_parameters(method_spec),
                        'returns': self._extract_response_type(method_spec)
                    })

        return methods

    def _extract_from_components(self, contract: Dict[str, Any]) -> List[Dict]:
        """Extract method specs from custom components format."""
        # Custom format for Python class APIs
        methods = []

        for component_name, component_spec in contract.get('components', {}).items():
            if 'methods' in component_spec:
                for method_name, method_spec in component_spec['methods'].items():
                    methods.append({
                        'name': method_name,
                        'parameters': method_spec.get('parameters', []),
                        'returns': method_spec.get('returns', {})
                    })

        return methods

    def _extract_parameters(self, method_spec: Dict) -> List[Dict]:
        """Extract parameter specifications."""
        params = []

        for param in method_spec.get('parameters', []):
            params.append({
                'name': param.get('name'),
                'type': param.get('schema', {}).get('type', 'any'),
                'required': param.get('required', False)
            })

        # Check request body
        if 'requestBody' in method_spec:
            content = method_spec['requestBody'].get('content', {})
            if 'application/json' in content:
                schema = content['application/json'].get('schema', {})
                params.append({
                    'name': 'body',
                    'type': 'object',
                    'required': method_spec['requestBody'].get('required', False)
                })

        return params

    def _extract_response_type(self, method_spec: Dict) -> str:
        """Extract expected response type."""
        responses = method_spec.get('responses', {})
        success_response = responses.get('200', responses.get('201', {}))

        content = success_response.get('content', {})
        if 'application/json' in content:
            schema = content['application/json'].get('schema', {})
            return schema.get('type', 'object')

        return 'any'

    def generate_python_contract_tests(
        self,
        spec: Dict[str, Any],
        component_name: str,
        output_path: Path
    ) -> str:
        """Generate Python contract test file."""

        class_name = spec.get('class_name', component_name.title().replace('-', ''))
        service_name = spec['service_name']
        methods = spec['methods']

        test_code = f'''"""
Contract Tests for {service_name}

Auto-generated from contract specification.
DO NOT EDIT - Regenerate using contract_test_generator.py

Purpose: Verify {component_name} implements EXACT API from contract.
"""

import pytest
import inspect
from typing import get_type_hints
from unittest.mock import Mock, patch


class Test{class_name}ContractCompliance:
    """Contract compliance tests for {class_name}."""

    def test_class_exists(self):
        """CRITICAL: Verify main class can be imported."""
        try:
            from {component_name.replace('-', '_')} import {class_name}
        except ImportError as e:
            pytest.fail(f"CRITICAL: Cannot import {{class_name}}: {{e}}")

    def test_class_instantiates(self):
        """Verify class can be instantiated."""
        from {component_name.replace('-', '_')} import {class_name}

        try:
            instance = {class_name}()
            assert instance is not None
        except Exception as e:
            pytest.fail(f"CRITICAL: Cannot instantiate {{class_name}}: {{e}}")

'''

        # Generate test for each method
        for method in methods:
            method_name = method['name']
            params = method.get('parameters', [])

            test_code += f'''
    def test_method_{method_name}_exists(self):
        """CRITICAL: Verify {method_name}() method exists."""
        from {component_name.replace('-', '_')} import {class_name}

        instance = {class_name}()
        assert hasattr(instance, '{method_name}'), \\
            f"CRITICAL: Missing method '{method_name}' from contract"

    def test_method_{method_name}_callable(self):
        """Verify {method_name}() is callable."""
        from {component_name.replace('-', '_')} import {class_name}

        instance = {class_name}()
        assert callable(getattr(instance, '{method_name}', None)), \\
            f"CRITICAL: '{method_name}' exists but is not callable"

'''

            if params:
                param_names = [p['name'] for p in params]
                test_code += f'''
    def test_method_{method_name}_signature(self):
        """Verify {method_name}() has correct signature."""
        from {component_name.replace('-', '_')} import {class_name}

        instance = {class_name}()
        sig = inspect.signature(instance.{method_name})
        params = list(sig.parameters.keys())

        # Expected parameters from contract: {param_names}
        for param in {param_names}:
            assert param in params, \\
                f"CRITICAL: Missing parameter '{{param}}' in {method_name}()"

'''

        # Add negative tests (methods that should NOT exist)
        test_code += f'''
    def test_no_wrong_method_names(self):
        """CRITICAL: Verify component doesn't have common wrong method names."""
        from {component_name.replace('-', '_')} import {class_name}

        instance = {class_name}()

        # Common mistakes from Music Analyzer catastrophe
        wrong_names = [
'''

        # Add common wrong names based on actual method names
        for method in methods:
            # Generate common variations that are WRONG
            name = method['name']
            if name.endswith('s'):
                wrong_singular = name[:-1]  # scan -> sca, files -> file
                test_code += f"            '{wrong_singular}',  # Wrong: should be '{name}'\n"
            elif '_' in name:
                wrong_camel = ''.join(w.capitalize() for w in name.split('_'))
                test_code += f"            '{wrong_camel}',  # Wrong: should be '{name}'\n"

        test_code += '''        ]

        for wrong_name in wrong_names:
            assert not hasattr(instance, wrong_name), \\
                f"CRITICAL: Found unexpected method '{wrong_name}' - check contract for correct name"
'''

        return test_code

    def generate_typescript_contract_tests(
        self,
        spec: Dict[str, Any],
        component_name: str,
        output_path: Path
    ) -> str:
        """Generate TypeScript contract test file."""

        service_name = spec['service_name']
        methods = spec['methods']

        test_code = f'''/**
 * Contract Tests for {service_name}
 *
 * Auto-generated from contract specification.
 * DO NOT EDIT - Regenerate using contract_test_generator.py
 *
 * Purpose: Verify {component_name} implements EXACT API from contract.
 */

import {{ describe, it, expect }} from 'vitest';
import {{ {service_name}Client }} from '../src/api';

describe('{service_name} Contract Compliance', () => {{
  it('CRITICAL: Client class should be importable', () => {{
    expect({service_name}Client).toBeDefined();
  }});

  it('Client should be instantiable', () => {{
    const client = new {service_name}Client();
    expect(client).toBeInstanceOf({service_name}Client);
  }});

'''

        for method in methods:
            method_name = method['name']
            test_code += f'''
  it('CRITICAL: Should have {method_name}() method', () => {{
    const client = new {service_name}Client();
    expect(typeof client.{method_name}).toBe('function');
  }});

'''

        test_code += '''});
'''

        return test_code

    def write_test_file(self, content: str, output_path: Path) -> None:
        """Write generated test file."""
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            f.write(content)

    def generate_for_component(
        self,
        contract_path: Path,
        component_name: str,
        language: str = 'python'
    ) -> bool:
        """Generate contract tests for a specific component."""

        print(f"Generating contract tests for {component_name}...")

        try:
            # Load contract
            contract = self.load_contract(contract_path)
            spec = self.extract_api_spec(contract)

            # Determine output path
            component_path = self.components_dir / component_name
            if language == 'python':
                output_path = component_path / "tests" / "contracts" / "test_contract_compliance.py"
                test_code = self.generate_python_contract_tests(spec, component_name, output_path)
            elif language == 'typescript':
                output_path = component_path / "tests" / "contracts" / "contractCompliance.test.ts"
                test_code = self.generate_typescript_contract_tests(spec, component_name, output_path)
            else:
                print(f"❌ Unsupported language: {language}")
                return False

            # Write file
            self.write_test_file(test_code, output_path)

            print(f"✅ Generated: {output_path}")
            return True

        except Exception as e:
            print(f"❌ Failed to generate tests: {e}")
            return False

    def generate_all(self) -> int:
        """Generate contract tests for all contracts."""

        if not self.contracts_dir.exists():
            print(f"❌ Contracts directory not found: {self.contracts_dir}")
            return 1

        contracts = list(self.contracts_dir.glob("*.yaml")) + list(self.contracts_dir.glob("*.yml"))

        if not contracts:
            print(f"⚠️  No contract files found in {self.contracts_dir}")
            return 0

        print(f"Found {len(contracts)} contract(s)\n")

        success_count = 0
        for contract_path in contracts:
            # Infer component name from contract filename
            component_name = contract_path.stem

            if self.generate_for_component(contract_path, component_name):
                success_count += 1

        print(f"\n{'=' * 70}")
        print(f"Generated contract tests: {success_count}/{len(contracts)}")
        print(f"{'=' * 70}")

        return 0 if success_count == len(contracts) else 1


def main():
    parser = argparse.ArgumentParser(
        description="Generate contract compliance tests from OpenAPI/YAML contracts",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate for specific contract
  python orchestration/contract_test_generator.py contracts/backend-api.yaml backend-service

  # Generate for all contracts
  python orchestration/contract_test_generator.py --all

  # Generate TypeScript tests
  python orchestration/contract_test_generator.py contracts/frontend-api.yaml frontend --language typescript
        """
    )

    parser.add_argument(
        'contract',
        nargs='?',
        help='Path to contract file (YAML)'
    )
    parser.add_argument(
        'component',
        nargs='?',
        help='Component name (directory in components/)'
    )
    parser.add_argument(
        '--all',
        action='store_true',
        help='Generate tests for all contracts'
    )
    parser.add_argument(
        '--language',
        choices=['python', 'typescript'],
        default='python',
        help='Target language (default: python)'
    )

    args = parser.parse_args()

    if not args.all and (not args.contract or not args.component):
        parser.error("Must specify contract and component, or use --all")

    generator = ContractTestGenerator()

    if args.all:
        sys.exit(generator.generate_all())
    else:
        contract_path = Path(args.contract)
        success = generator.generate_for_component(
            contract_path,
            args.component,
            args.language
        )
        sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
