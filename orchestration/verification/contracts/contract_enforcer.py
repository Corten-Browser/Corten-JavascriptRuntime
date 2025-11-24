#!/usr/bin/env python3
"""
Contract-First Development Enforcer

Ensures no code written without contract.
Part of v0.4.0 quality enhancement system - Batch 2.
"""

from pathlib import Path
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass, field
import yaml
import json
import re
import sys


@dataclass
class EnforcementViolation:
    """A contract enforcement violation."""
    component_name: str
    violation_type: str
    description: str
    severity: str  # "critical", "warning", "info"

    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'component_name': self.component_name,
            'violation_type': self.violation_type,
            'description': self.description,
            'severity': self.severity
        }


@dataclass
class ContractCompliance:
    """Contract compliance result."""
    component_name: str
    has_contract: bool
    contract_path: Optional[Path]
    implementation_exists: bool
    implementation_path: Optional[Path]
    compliant: bool
    violations: List[EnforcementViolation] = field(default_factory=list)

    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'component_name': self.component_name,
            'has_contract': self.has_contract,
            'contract_path': str(self.contract_path) if self.contract_path else None,
            'implementation_exists': self.implementation_exists,
            'implementation_path': str(self.implementation_path) if self.implementation_path else None,
            'compliant': self.compliant,
            'violations': [v.to_dict() for v in self.violations]
        }


class ContractEnforcer:
    """Enforces contract-first development."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self.contracts_dir = self.project_root / "contracts"
        self.components_dir = self.project_root / "components"

        # Create directories if they don't exist
        self.contracts_dir.mkdir(exist_ok=True, parents=True)
        self.components_dir.mkdir(exist_ok=True, parents=True)

    def check_contract_exists(self, component_name: str) -> bool:
        """
        Check if component has a contract.

        Args:
            component_name: Name of the component

        Returns:
            True if contract exists, False otherwise
        """
        contract_path = self.contracts_dir / f"{component_name}_api.yaml"
        return contract_path.exists()

    def get_contract_path(self, component_name: str) -> Optional[Path]:
        """
        Get the path to a component's contract if it exists.

        Args:
            component_name: Name of the component

        Returns:
            Path to contract file or None
        """
        contract_path = self.contracts_dir / f"{component_name}_api.yaml"
        return contract_path if contract_path.exists() else None

    def block_implementation_without_contract(self, component_name: str) -> bool:
        """
        Check if implementation should be blocked.

        Returns True if implementation exists without contract.

        Args:
            component_name: Name of the component

        Returns:
            True if implementation should be blocked (has code but no contract)
        """
        has_contract = self.check_contract_exists(component_name)
        component_path = self.components_dir / component_name
        has_implementation = self._has_implementation_files(component_path)

        # Block if implementation exists but no contract
        return has_implementation and not has_contract

    def _has_implementation_files(self, component_path: Path) -> bool:
        """Check if component has implementation files."""
        if not component_path.exists():
            return False

        # Look for source code files (Python, JavaScript, TypeScript, etc.)
        source_extensions = ['.py', '.js', '.ts', '.jsx', '.tsx', '.go', '.rs', '.java']

        for ext in source_extensions:
            if any(component_path.rglob(f"*{ext}")):
                # Check if it's not just test files or config
                for file in component_path.rglob(f"*{ext}"):
                    # Skip test files, config files, and __init__.py
                    if not any(x in file.name for x in ['test_', '_test.', 'conftest', '__init__']):
                        return True

        return False

    def verify_component_compliance(self, component_name: str) -> ContractCompliance:
        """
        Verify component compliance with contract-first policy.

        Args:
            component_name: Name of the component

        Returns:
            ContractCompliance object with detailed violation information
        """
        violations = []

        # Check contract existence
        contract_path = self.contracts_dir / f"{component_name}_api.yaml"
        has_contract = contract_path.exists()

        # Check implementation existence
        component_path = self.components_dir / component_name
        has_implementation = self._has_implementation_files(component_path)

        # CRITICAL: Implementation without contract
        if has_implementation and not has_contract:
            violations.append(EnforcementViolation(
                component_name=component_name,
                violation_type="missing_contract",
                description="Implementation exists without contract. Contract-first development requires a contract BEFORE implementation.",
                severity="critical"
            ))

        if has_contract:
            # Verify contract completeness
            contract_issues = self._verify_contract_completeness(contract_path)
            violations.extend(contract_issues)

            if has_implementation:
                # Verify implementation matches contract
                impl_issues = self._verify_implementation_matches_contract(
                    component_path, contract_path
                )
                violations.extend(impl_issues)

        # Component is compliant if no critical violations
        critical_violations = [v for v in violations if v.severity == "critical"]
        compliant = len(critical_violations) == 0

        return ContractCompliance(
            component_name=component_name,
            has_contract=has_contract,
            contract_path=contract_path if has_contract else None,
            implementation_exists=has_implementation,
            implementation_path=component_path if has_implementation else None,
            compliant=compliant,
            violations=violations
        )

    def _verify_contract_completeness(self, contract_path: Path) -> List[EnforcementViolation]:
        """
        Verify contract has all required sections.

        Args:
            contract_path: Path to the contract file

        Returns:
            List of violations found
        """
        violations = []
        component_name = contract_path.stem.replace('_api', '')

        try:
            with open(contract_path, 'r') as f:
                contract = yaml.safe_load(f)
        except yaml.YAMLError as e:
            violations.append(EnforcementViolation(
                component_name=component_name,
                violation_type="invalid_contract",
                description=f"Contract YAML parsing failed: {e}",
                severity="critical"
            ))
            return violations
        except Exception as e:
            violations.append(EnforcementViolation(
                component_name=component_name,
                violation_type="invalid_contract",
                description=f"Contract reading failed: {e}",
                severity="critical"
            ))
            return violations

        # Check required OpenAPI sections
        required_sections = ['openapi', 'info', 'paths']
        for section in required_sections:
            if section not in contract:
                violations.append(EnforcementViolation(
                    component_name=component_name,
                    violation_type="incomplete_contract",
                    description=f"Missing required section: {section}",
                    severity="critical"
                ))

        # Check info section has required fields
        if 'info' in contract:
            info = contract['info']
            required_info_fields = ['title', 'version']
            for field in required_info_fields:
                if field not in info:
                    violations.append(EnforcementViolation(
                        component_name=component_name,
                        violation_type="incomplete_contract",
                        description=f"Missing required info field: {field}",
                        severity="warning"
                    ))

        # Check endpoints have error scenarios
        if 'paths' in contract:
            for path, methods in contract['paths'].items():
                for method, spec in methods.items():
                    if method.upper() in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH']:
                        # Check for error responses
                        responses = spec.get('responses', {})

                        # POST/PUT/PATCH should have 400 Bad Request
                        if method.upper() in ['POST', 'PUT', 'PATCH']:
                            if '400' not in responses:
                                violations.append(EnforcementViolation(
                                    component_name=component_name,
                                    violation_type="missing_error_response",
                                    description=f"{method.upper()} {path} missing 400 error response",
                                    severity="warning"
                                ))

                        # All endpoints should have 500 Internal Server Error
                        if '500' not in responses:
                            violations.append(EnforcementViolation(
                                component_name=component_name,
                                violation_type="missing_error_response",
                                description=f"{method.upper()} {path} missing 500 error response",
                                severity="info"
                            ))

                        # Endpoints with path parameters should have 404
                        if '{' in path and '404' not in responses:
                            violations.append(EnforcementViolation(
                                component_name=component_name,
                                violation_type="missing_error_response",
                                description=f"{method.upper()} {path} has path parameters but missing 404 error response",
                                severity="warning"
                            ))

        return violations

    def _verify_implementation_matches_contract(self,
                                               component_path: Path,
                                               contract_path: Path) -> List[EnforcementViolation]:
        """
        Verify implementation matches contract.

        This is a heuristic check that looks for endpoint definitions in code.
        For full verification, use the actual test suite.

        Args:
            component_path: Path to component directory
            contract_path: Path to contract file

        Returns:
            List of violations found
        """
        violations = []
        component_name = component_path.name

        try:
            with open(contract_path, 'r') as f:
                contract = yaml.safe_load(f)
        except:
            # Already reported in completeness check
            return violations

        # Get all endpoints from contract
        contract_endpoints = []
        if 'paths' in contract:
            for path, methods in contract['paths'].items():
                for method in methods:
                    if method.upper() in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH']:
                        contract_endpoints.append((method.upper(), path))

        # Read all implementation files
        impl_files = []
        for ext in ['.py', '.js', '.ts', '.jsx', '.tsx']:
            impl_files.extend(list(component_path.rglob(f"*{ext}")))

        impl_content = ""
        for f in impl_files:
            try:
                # Skip test files
                if 'test' in f.name.lower():
                    continue
                impl_content += f.read_text() + "\n"
            except:
                pass

        # Check implementation has these endpoints
        # This is a simple heuristic - look for route decorators or path mentions
        for method, path in contract_endpoints:
            # Convert path to various formats that might appear in code
            # e.g., /users/{id} might be '/users/<id>' in Flask, '/users/:id' in Express
            path_variants = [
                path,  # /users/{id}
                path.replace('{', '<').replace('}', '>'),  # /users/<id>
                path.replace('{', ':').replace('}', ''),  # /users/:id
                re.sub(r'\{[^}]+\}', r'\\w+', path),  # Regex pattern
            ]

            found = False
            for variant in path_variants:
                if variant in impl_content:
                    found = True
                    break

            if not found:
                violations.append(EnforcementViolation(
                    component_name=component_name,
                    violation_type="missing_endpoint",
                    description=f"Contract defines {method} {path} but implementation not found (heuristic check - verify with tests)",
                    severity="warning"  # Warning because detection is heuristic
                ))

        return violations

    def generate_implementation_skeleton(self, component_name: str, framework: str = "fastapi") -> str:
        """
        Generate code skeleton from contract.

        Args:
            component_name: Name of the component
            framework: Framework to generate for (fastapi, flask, express, etc.)

        Returns:
            Generated code as string
        """
        contract_path = self.contracts_dir / f"{component_name}_api.yaml"

        if not contract_path.exists():
            return ""

        try:
            with open(contract_path, 'r') as f:
                contract = yaml.safe_load(f)
        except:
            return ""

        if framework == "fastapi":
            return self._generate_fastapi_skeleton(contract, component_name)
        elif framework == "flask":
            return self._generate_flask_skeleton(contract, component_name)
        else:
            return f"# Skeleton generation for {framework} not yet implemented\n"

    def _generate_fastapi_skeleton(self, contract: Dict, component_name: str) -> str:
        """Generate FastAPI skeleton from contract."""
        code = ['from fastapi import APIRouter, HTTPException, status',
                'from pydantic import BaseModel',
                'from typing import Optional, Dict, Any',
                '',
                'router = APIRouter()',
                '']

        if 'paths' in contract:
            for path, methods in contract['paths'].items():
                for method, spec in methods.items():
                    if method.upper() in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH']:
                        summary = spec.get('summary', 'TODO: Add description')
                        operation_id = spec.get('operationId', self._path_to_function_name(path, method))

                        # Extract response status
                        responses = spec.get('responses', {})
                        success_status = self._get_success_status(method, responses)

                        # Generate route decorator
                        code.append(f'@router.{method.lower()}("{path}", status_code={success_status})')

                        # Generate function signature
                        params = self._extract_fastapi_parameters(path, spec)
                        code.append(f'async def {operation_id}({params}):')

                        # Add docstring
                        code.append(f'    """')
                        code.append(f'    {summary}')
                        code.append(f'    ')
                        code.append(f'    Contract: contracts/{component_name}_api.yaml')

                        # Add error scenarios to docstring
                        if 'x-error-scenarios' in spec:
                            code.append(f'    ')
                            code.append(f'    Error Scenarios:')
                            for scenario in spec['x-error-scenarios']:
                                code.append(f'    - {scenario["status_code"]}: {scenario["when"]}')

                        code.append(f'    """')
                        code.append(f'    # TODO: Implement according to contract')
                        code.append(f'    raise NotImplementedError("Endpoint not implemented")')
                        code.append('')

        return '\n'.join(code)

    def _generate_flask_skeleton(self, contract: Dict, component_name: str) -> str:
        """Generate Flask skeleton from contract."""
        code = ['from flask import Blueprint, request, jsonify',
                'from typing import Dict, Any',
                '',
                f'bp = Blueprint("{component_name}", __name__)',
                '']

        if 'paths' in contract:
            for path, methods in contract['paths'].items():
                for method, spec in methods.items():
                    if method.upper() in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH']:
                        summary = spec.get('summary', 'TODO: Add description')
                        operation_id = spec.get('operationId', self._path_to_function_name(path, method))

                        # Convert OpenAPI path to Flask path
                        flask_path = path.replace('{', '<').replace('}', '>')

                        # Generate route decorator
                        code.append(f'@bp.route("{flask_path}", methods=["{method.upper()}"])')
                        code.append(f'def {operation_id}():')
                        code.append(f'    """')
                        code.append(f'    {summary}')
                        code.append(f'    ')
                        code.append(f'    Contract: contracts/{component_name}_api.yaml')
                        code.append(f'    """')
                        code.append(f'    # TODO: Implement according to contract')
                        code.append(f'    return jsonify({{"error": "Not implemented"}}), 501')
                        code.append('')

        return '\n'.join(code)

    def _path_to_function_name(self, path: str, method: str) -> str:
        """Convert path to function name."""
        # /users/{id} -> get_user_by_id
        parts = path.strip('/').split('/')
        name_parts = [method.lower()]

        for part in parts:
            if part.startswith('{') and part.endswith('}'):
                name_parts.append('by')
                name_parts.append(part[1:-1])
            else:
                # Convert to singular if it ends with 's'
                singular = part.rstrip('s') if part.endswith('s') and len(part) > 1 else part
                name_parts.append(singular)

        return '_'.join(name_parts)

    def _extract_fastapi_parameters(self, path: str, spec: Dict) -> str:
        """Extract parameters for FastAPI function signature."""
        params = []

        # Path parameters
        path_params = re.findall(r'\{(\w+)\}', path)
        for param in path_params:
            params.append(f'{param}: str')

        # Request body
        if 'requestBody' in spec:
            params.append('request_data: Dict[str, Any]')

        return ', '.join(params) if params else ''

    def _get_success_status(self, method: str, responses: Dict) -> int:
        """Get success status code from responses."""
        # Look for 2xx responses
        for status in ['200', '201', '204']:
            if status in responses:
                return int(status)

        # Default based on method
        if method.upper() == 'POST':
            return 201
        elif method.upper() == 'DELETE':
            return 204
        return 200

    def enforce_all_components(self) -> Dict[str, ContractCompliance]:
        """
        Check compliance for all components.

        Returns:
            Dictionary mapping component name to compliance result
        """
        results = {}

        if not self.components_dir.exists():
            return results

        for component_dir in self.components_dir.iterdir():
            if component_dir.is_dir() and not component_dir.name.startswith('.'):
                results[component_dir.name] = self.verify_component_compliance(component_dir.name)

        return results

    def generate_report(self, compliance: ContractCompliance, format: str = "text") -> str:
        """
        Generate formatted report.

        Args:
            compliance: Compliance result
            format: Output format ("text" or "json")

        Returns:
            Formatted report as string
        """
        if format == "json":
            return json.dumps(compliance.to_dict(), indent=2)

        # Text format
        report = []
        report.append("="*70)
        report.append(f"CONTRACT ENFORCEMENT: {compliance.component_name}")
        report.append("="*70)
        report.append("")

        report.append(f"Contract Exists:        {'✅' if compliance.has_contract else '❌'}")
        if compliance.contract_path:
            report.append(f"Contract Path:          {compliance.contract_path}")

        report.append(f"Implementation Exists:  {'✅' if compliance.implementation_exists else '❌'}")
        if compliance.implementation_path:
            report.append(f"Implementation Path:    {compliance.implementation_path}")

        report.append("")
        report.append(f"Compliant:              {'✅' if compliance.compliant else '❌'}")

        if compliance.violations:
            report.append("")
            report.append(f"Violations: {len(compliance.violations)}")
            report.append("")

            # Group by severity
            critical = [v for v in compliance.violations if v.severity == "critical"]
            warnings = [v for v in compliance.violations if v.severity == "warning"]
            info = [v for v in compliance.violations if v.severity == "info"]

            if critical:
                report.append("CRITICAL VIOLATIONS:")
                for v in critical:
                    report.append(f"  ❌ {v.description}")
                    report.append(f"     Type: {v.violation_type}")
                    report.append("")

            if warnings:
                report.append("WARNINGS:")
                for v in warnings:
                    report.append(f"  ⚠️  {v.description}")
                    report.append(f"     Type: {v.violation_type}")
                    report.append("")

            if info:
                report.append("INFORMATIONAL:")
                for v in info:
                    report.append(f"  ℹ️  {v.description}")
                    report.append(f"     Type: {v.violation_type}")
                    report.append("")
        else:
            report.append("")
            report.append("✅ No violations found!")

        report.append("="*70)
        return "\n".join(report)

    def generate_summary_report(self, results: Dict[str, ContractCompliance]) -> str:
        """
        Generate summary report for all components.

        Args:
            results: Dictionary of compliance results

        Returns:
            Formatted summary report
        """
        report = []
        report.append("="*70)
        report.append("CONTRACT ENFORCEMENT SUMMARY")
        report.append("="*70)
        report.append("")

        total = len(results)
        compliant = sum(1 for r in results.values() if r.compliant)
        non_compliant = total - compliant

        report.append(f"Total Components:    {total}")
        report.append(f"Compliant:           {compliant} ✅")
        report.append(f"Non-Compliant:       {non_compliant} ❌")
        report.append("")

        if non_compliant > 0:
            report.append("NON-COMPLIANT COMPONENTS:")
            report.append("")

            for name, result in results.items():
                if not result.compliant:
                    critical = sum(1 for v in result.violations if v.severity == "critical")
                    warnings = sum(1 for v in result.violations if v.severity == "warning")

                    status = "❌"
                    if not result.has_contract:
                        status += " NO CONTRACT"
                    elif critical > 0:
                        status += f" {critical} CRITICAL"

                    report.append(f"  {status} {name}")

                    if not result.has_contract and result.implementation_exists:
                        report.append(f"     Implementation exists without contract!")

                    for v in result.violations:
                        if v.severity == "critical":
                            report.append(f"     - {v.description}")

                    report.append("")

        report.append("="*70)
        return "\n".join(report)


def main():
    """CLI interface."""
    if len(sys.argv) < 2:
        print("Usage: contract_enforcer.py <command> [options]")
        print("\nCommands:")
        print("  check <component>         - Check contract compliance for a component")
        print("  check-all                 - Check all components")
        print("  check-all --json          - Check all components (JSON output)")
        print("  skeleton <component>      - Generate implementation skeleton")
        print("  skeleton <component> --framework flask  - Generate Flask skeleton")
        print("  block <component>         - Check if component should be blocked")
        print("\nExamples:")
        print("  python orchestration/contract_enforcer.py check auth-service")
        print("  python orchestration/contract_enforcer.py check-all")
        print("  python orchestration/contract_enforcer.py skeleton auth-service")
        sys.exit(1)

    command = sys.argv[1]
    enforcer = ContractEnforcer(Path.cwd())

    if command == "check":
        if len(sys.argv) < 3:
            print("Error: check requires component name")
            sys.exit(1)

        component = sys.argv[2]
        compliance = enforcer.verify_component_compliance(component)

        # Check for --json flag
        format_type = "json" if "--json" in sys.argv else "text"
        print(enforcer.generate_report(compliance, format_type))

        sys.exit(0 if compliance.compliant else 1)

    elif command == "check-all":
        results = enforcer.enforce_all_components()

        if "--json" in sys.argv:
            # JSON output
            output = {name: result.to_dict() for name, result in results.items()}
            print(json.dumps(output, indent=2))
        else:
            # Text output with summary
            print(enforcer.generate_summary_report(results))
            print()

            # Detailed reports for non-compliant components
            for name, result in results.items():
                if not result.compliant:
                    print(enforcer.generate_report(result))
                    print()

        # Exit with error if any component is non-compliant
        all_compliant = all(r.compliant for r in results.values())
        sys.exit(0 if all_compliant else 1)

    elif command == "skeleton":
        if len(sys.argv) < 3:
            print("Error: skeleton requires component name")
            sys.exit(1)

        component = sys.argv[2]

        # Check for framework option
        framework = "fastapi"
        if "--framework" in sys.argv:
            framework_idx = sys.argv.index("--framework")
            if framework_idx + 1 < len(sys.argv):
                framework = sys.argv[framework_idx + 1]

        skeleton = enforcer.generate_implementation_skeleton(component, framework)

        if skeleton:
            print(skeleton)
        else:
            print(f"Error: Contract not found for component: {component}")
            print(f"Expected: {enforcer.contracts_dir}/{component}_api.yaml")
            sys.exit(1)

    elif command == "block":
        if len(sys.argv) < 3:
            print("Error: block requires component name")
            sys.exit(1)

        component = sys.argv[2]
        should_block = enforcer.block_implementation_without_contract(component)

        if should_block:
            print(f"❌ BLOCKED: Component '{component}' has implementation without contract!")
            print(f"   Contract-first development requires a contract BEFORE implementation.")
            print(f"   Create contract at: {enforcer.contracts_dir}/{component}_api.yaml")
            sys.exit(1)
        else:
            print(f"✅ Component '{component}' is allowed to proceed")
            sys.exit(0)

    else:
        print(f"Error: Unknown command: {command}")
        sys.exit(1)


if __name__ == '__main__':
    main()
