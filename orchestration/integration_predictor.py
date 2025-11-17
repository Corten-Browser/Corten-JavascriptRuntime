#!/usr/bin/env python3
"""
Integration Failure Prediction System

Analyzes components BEFORE integration to predict failures.
Part of v0.4.0 quality enhancement system - Batch 3.
"""

from pathlib import Path
from typing import List, Dict, Optional, Set, Tuple
from dataclasses import dataclass, asdict
import yaml
import json
import re


@dataclass
class PredictedFailure:
    """A predicted integration failure."""
    failure_type: str
    component_a: str
    component_b: str
    description: str
    severity: str  # "critical", "warning", "info"
    fix_strategy: str
    test_generation: str

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)


@dataclass
class IntegrationPrediction:
    """Complete integration prediction result."""
    total_components: int
    total_pairs_analyzed: int
    predicted_failures: List[PredictedFailure]
    data_type_incompatibilities: int
    timeout_cascade_risks: int
    error_propagation_issues: int
    circular_dependencies: int

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            'total_components': self.total_components,
            'total_pairs_analyzed': self.total_pairs_analyzed,
            'predicted_failures': [f.to_dict() for f in self.predicted_failures],
            'data_type_incompatibilities': self.data_type_incompatibilities,
            'timeout_cascade_risks': self.timeout_cascade_risks,
            'error_propagation_issues': self.error_propagation_issues,
            'circular_dependencies': self.circular_dependencies
        }


class IntegrationPredictor:
    """Predicts integration failures before they occur."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.components_dir = project_root / "components"
        self.contracts_dir = project_root / "contracts"
        self.patterns_file = project_root / "orchestration" / "integration_patterns.yaml"
        self.patterns = self._load_patterns()

    def _load_patterns(self) -> dict:
        """Load integration failure patterns."""
        if self.patterns_file.exists():
            try:
                with open(self.patterns_file, 'r') as f:
                    return yaml.safe_load(f) or {}
            except Exception as e:
                print(f"Warning: Failed to load patterns file: {e}")
        return self._default_patterns()

    def _default_patterns(self) -> dict:
        """Default integration patterns."""
        return {
            'common_integration_failures': {
                'data_format_mismatch': {
                    'detection_pattern': 'different date/time formats',
                    'fix_strategy': 'standardize on ISO8601',
                    'test_generation': 'create format validation tests'
                },
                'missing_error_handling': {
                    'detection_pattern': 'no try/catch on component calls',
                    'fix_strategy': 'add circuit breaker pattern',
                    'test_generation': 'create failure scenario tests'
                },
                'timeout_cascade': {
                    'detection_pattern': 'timeout_sum > parent_timeout',
                    'fix_strategy': 'adjust timeout hierarchy',
                    'test_generation': 'create timeout tests'
                },
                'circular_dependency': {
                    'detection_pattern': 'component cycle detected',
                    'fix_strategy': 'introduce event bus or mediator pattern',
                    'test_generation': 'create dependency tests'
                }
            }
        }

    def predict_integration_failures(self) -> IntegrationPrediction:
        """Analyze all component pairs for potential failures."""
        components = self._load_all_components()
        contracts = self._load_all_contracts()

        failures = []

        # Analyze each component pair
        for i, comp_a in enumerate(components):
            for comp_b in components[i+1:]:
                # Data type compatibility
                failures.extend(self.analyze_data_type_compatibility(comp_a, comp_b, contracts))

                # Error propagation
                failures.extend(self.analyze_error_propagation(comp_a, comp_b, contracts))

                # Timeout cascades
                failures.extend(self.analyze_timeout_cascade(comp_a, comp_b, contracts))

        # Analyze circular dependencies (entire graph)
        failures.extend(self.analyze_circular_dependencies(components, contracts))

        # Count failures by type
        data_type_count = len([f for f in failures if f.failure_type == "data_format_mismatch"])
        timeout_count = len([f for f in failures if f.failure_type == "timeout_cascade"])
        error_prop_count = len([f for f in failures if f.failure_type == "missing_error_handling"])
        circular_count = len([f for f in failures if f.failure_type == "circular_dependency"])

        return IntegrationPrediction(
            total_components=len(components),
            total_pairs_analyzed=len(components) * (len(components) - 1) // 2,
            predicted_failures=failures,
            data_type_incompatibilities=data_type_count,
            timeout_cascade_risks=timeout_count,
            error_propagation_issues=error_prop_count,
            circular_dependencies=circular_count
        )

    def analyze_data_type_compatibility(self,
                                       comp_a: str,
                                       comp_b: str,
                                       contracts: Dict) -> List[PredictedFailure]:
        """Check for data type mismatches."""
        failures = []

        contract_a = contracts.get(comp_a)
        contract_b = contracts.get(comp_b)

        if not contract_a or not contract_b:
            return failures

        # Check date/time format compatibility
        datetime_formats_a = self._extract_datetime_formats(contract_a)
        datetime_formats_b = self._extract_datetime_formats(contract_b)

        if datetime_formats_a and datetime_formats_b:
            if datetime_formats_a != datetime_formats_b:
                failures.append(PredictedFailure(
                    failure_type="data_format_mismatch",
                    component_a=comp_a,
                    component_b=comp_b,
                    description=f"Date/time format mismatch: {comp_a} uses {datetime_formats_a}, {comp_b} uses {datetime_formats_b}",
                    severity="critical",
                    fix_strategy="Standardize on ISO8601 format across all components",
                    test_generation="Create date format validation tests for API boundaries"
                ))

        # Check ID format compatibility
        id_formats_a = self._extract_id_formats(contract_a)
        id_formats_b = self._extract_id_formats(contract_b)

        if id_formats_a and id_formats_b:
            if id_formats_a != id_formats_b:
                failures.append(PredictedFailure(
                    failure_type="data_format_mismatch",
                    component_a=comp_a,
                    component_b=comp_b,
                    description=f"ID format mismatch: {comp_a} uses {id_formats_a}, {comp_b} uses {id_formats_b}",
                    severity="critical",
                    fix_strategy="Standardize on UUID format for all IDs",
                    test_generation="Create ID format validation tests"
                ))

        # Check enum compatibility
        enum_conflicts = self._check_enum_compatibility(contract_a, contract_b, comp_a, comp_b)
        failures.extend(enum_conflicts)

        # Check null handling compatibility
        null_conflicts = self._check_null_handling(contract_a, contract_b, comp_a, comp_b)
        failures.extend(null_conflicts)

        return failures

    def analyze_error_propagation(self,
                                  comp_a: str,
                                  comp_b: str,
                                  contracts: Dict) -> List[PredictedFailure]:
        """Analyze how errors propagate between components."""
        failures = []

        # Check if comp_a calls comp_b
        calls_ab = self._check_component_calls(comp_a, comp_b)
        calls_ba = self._check_component_calls(comp_b, comp_a)

        # Analyze A -> B
        if calls_ab:
            has_error_handling = self._check_error_handling_for_dependency(comp_a, comp_b)

            if not has_error_handling:
                failures.append(PredictedFailure(
                    failure_type="missing_error_handling",
                    component_a=comp_a,
                    component_b=comp_b,
                    description=f"{comp_a} calls {comp_b} but doesn't handle errors",
                    severity="critical",
                    fix_strategy="Add try/except with fallback behavior or circuit breaker",
                    test_generation="Create failure scenario tests for dependency errors"
                ))

            # Check for retry logic
            has_retry = self._check_retry_logic(comp_a, comp_b)
            if not has_retry:
                failures.append(PredictedFailure(
                    failure_type="missing_error_handling",
                    component_a=comp_a,
                    component_b=comp_b,
                    description=f"{comp_a} calls {comp_b} but doesn't implement retry logic",
                    severity="warning",
                    fix_strategy="Add exponential backoff retry mechanism",
                    test_generation="Create retry behavior tests"
                ))

        # Analyze B -> A
        if calls_ba:
            has_error_handling = self._check_error_handling_for_dependency(comp_b, comp_a)

            if not has_error_handling:
                failures.append(PredictedFailure(
                    failure_type="missing_error_handling",
                    component_a=comp_b,
                    component_b=comp_a,
                    description=f"{comp_b} calls {comp_a} but doesn't handle errors",
                    severity="critical",
                    fix_strategy="Add try/except with fallback behavior or circuit breaker",
                    test_generation="Create failure scenario tests for dependency errors"
                ))

        return failures

    def analyze_timeout_cascade(self,
                                comp_a: str,
                                comp_b: str,
                                contracts: Dict) -> List[PredictedFailure]:
        """Analyze timeout cascade risks."""
        failures = []

        contract_a = contracts.get(comp_a)
        contract_b = contracts.get(comp_b)

        if not contract_a or not contract_b:
            return failures

        # Get timeouts
        timeout_a = self._extract_timeout(contract_a)
        timeout_b = self._extract_timeout(contract_b)

        # Check if comp_a calls comp_b
        if self._check_component_calls(comp_a, comp_b):
            # Timeout cascade: if comp_a timeout <= comp_b timeout + overhead
            if timeout_a and timeout_b:
                overhead = 5  # seconds for network, processing
                if timeout_a <= timeout_b + overhead:
                    failures.append(PredictedFailure(
                        failure_type="timeout_cascade",
                        component_a=comp_a,
                        component_b=comp_b,
                        description=f"Timeout cascade risk: {comp_a} timeout ({timeout_a}s) too close to {comp_b} timeout ({timeout_b}s)",
                        severity="warning",
                        fix_strategy=f"Increase {comp_a} timeout to at least {timeout_b + overhead + 10}s",
                        test_generation="Create timeout cascade tests"
                    ))

        # Check reverse direction
        if self._check_component_calls(comp_b, comp_a):
            if timeout_a and timeout_b:
                overhead = 5
                if timeout_b <= timeout_a + overhead:
                    failures.append(PredictedFailure(
                        failure_type="timeout_cascade",
                        component_a=comp_b,
                        component_b=comp_a,
                        description=f"Timeout cascade risk: {comp_b} timeout ({timeout_b}s) too close to {comp_a} timeout ({timeout_a}s)",
                        severity="warning",
                        fix_strategy=f"Increase {comp_b} timeout to at least {timeout_a + overhead + 10}s",
                        test_generation="Create timeout cascade tests"
                    ))

        return failures

    def analyze_circular_dependencies(self,
                                     components: List[str],
                                     contracts: Dict) -> List[PredictedFailure]:
        """Detect circular dependencies."""
        failures = []

        # Build dependency graph
        graph = {}
        for comp in components:
            graph[comp] = self._get_component_dependencies(comp)

        # Detect cycles using DFS
        def find_cycles_from(node: str, visited: Set[str], rec_stack: List[str]) -> Optional[List[str]]:
            """Find cycle starting from node."""
            visited.add(node)
            rec_stack.append(node)

            for neighbor in graph.get(node, []):
                if neighbor not in visited:
                    cycle = find_cycles_from(neighbor, visited, rec_stack)
                    if cycle:
                        return cycle
                elif neighbor in rec_stack:
                    # Found cycle - return the cycle path
                    cycle_start = rec_stack.index(neighbor)
                    return rec_stack[cycle_start:] + [neighbor]

            rec_stack.pop()
            return None

        visited = set()
        detected_cycles = []

        for comp in components:
            if comp not in visited:
                cycle = find_cycles_from(comp, visited, [])
                if cycle:
                    # Avoid duplicate cycles
                    cycle_str = "->".join(sorted(cycle))
                    if cycle_str not in detected_cycles:
                        detected_cycles.append(cycle_str)

                        failures.append(PredictedFailure(
                            failure_type="circular_dependency",
                            component_a=cycle[0],
                            component_b=cycle[-1] if len(cycle) > 1 else cycle[0],
                            description=f"Circular dependency detected: {' -> '.join(cycle)}",
                            severity="critical",
                            fix_strategy="Introduce event bus or mediator pattern to break cycle",
                            test_generation="Create dependency tests"
                        ))

        return failures

    def generate_integration_test_suite(self,
                                       predictions: IntegrationPrediction) -> str:
        """Generate targeted integration tests for predicted failures."""
        test_code = [
            '"""',
            'Auto-generated integration tests',
            'Generated by IntegrationPredictor',
            '"""',
            '',
            'import pytest',
            'from datetime import datetime',
            'import time',
            ''
        ]

        if not predictions.predicted_failures:
            test_code.append('def test_no_failures_predicted():')
            test_code.append('    """No integration failures were predicted."""')
            test_code.append('    assert True')
            test_code.append('')
            return '\n'.join(test_code)

        for i, failure in enumerate(predictions.predicted_failures):
            test_name = f"test_{failure.failure_type}_{failure.component_a}_{failure.component_b}"
            test_name = test_name.replace('-', '_').replace('(', '').replace(')', '')

            test_code.append(f'def {test_name}():')
            test_code.append(f'    """')
            test_code.append(f'    Test for: {failure.description}')
            test_code.append(f'    Severity: {failure.severity}')
            test_code.append(f'    Fix: {failure.fix_strategy}')
            test_code.append(f'    """')

            # Generate test body based on failure type
            if failure.failure_type == "data_format_mismatch":
                test_code.append(f'    # TODO: Test data format compatibility between {failure.component_a} and {failure.component_b}')
                test_code.append('    # Verify both components handle the same date/time/ID formats')
                test_code.append('    pass')

            elif failure.failure_type == "missing_error_handling":
                test_code.append(f'    # TODO: Test error handling for {failure.component_a} -> {failure.component_b} calls')
                test_code.append('    # Simulate failure in dependency and verify proper error handling')
                test_code.append('    pass')

            elif failure.failure_type == "timeout_cascade":
                test_code.append(f'    # TODO: Test timeout cascade between {failure.component_a} and {failure.component_b}')
                test_code.append('    # Verify parent timeout > child timeout + overhead')
                test_code.append('    pass')

            elif failure.failure_type == "circular_dependency":
                test_code.append(f'    # TODO: Test circular dependency detection')
                test_code.append('    # Verify components do not form circular call chains')
                test_code.append('    pass')

            else:
                test_code.append(f'    # TODO: Implement test for {failure.failure_type}')
                test_code.append('    pass')

            test_code.append('')

        return '\n'.join(test_code)

    def _load_all_components(self) -> List[str]:
        """Get list of all components."""
        if not self.components_dir.exists():
            return []

        return [d.name for d in self.components_dir.iterdir()
                if d.is_dir() and not d.name.startswith('.')]

    def _load_all_contracts(self) -> Dict[str, dict]:
        """Load all contracts."""
        contracts = {}

        if not self.contracts_dir.exists():
            return contracts

        for contract_file in self.contracts_dir.glob("*.yaml"):
            try:
                with open(contract_file, 'r') as f:
                    component_name = contract_file.stem.replace('_api', '').replace('-api', '')
                    contracts[component_name] = yaml.safe_load(f)
            except Exception as e:
                print(f"Warning: Failed to load contract {contract_file}: {e}")

        return contracts

    def _extract_datetime_formats(self, contract: dict) -> Optional[str]:
        """Extract datetime format from contract."""
        # Look for datetime examples or schemas
        contract_str = str(contract).lower()

        if 'iso8601' in contract_str or 'iso-8601' in contract_str:
            return 'ISO8601'
        elif 'unix' in contract_str or 'epoch' in contract_str:
            return 'Unix timestamp'
        elif 'rfc3339' in contract_str:
            return 'RFC3339'

        # Check schema for date-time format
        if self._deep_search_dict(contract, 'format', 'date-time'):
            return 'ISO8601'

        return None

    def _extract_id_formats(self, contract: dict) -> Optional[str]:
        """Extract ID format from contract."""
        contract_str = str(contract).lower()

        if 'uuid' in contract_str:
            return 'UUID'
        elif self._deep_search_dict(contract, 'type', 'integer'):
            return 'integer'
        elif self._deep_search_dict(contract, 'type', 'string'):
            return 'string'

        return None

    def _extract_timeout(self, contract: dict) -> Optional[int]:
        """Extract timeout from contract."""
        # Look for x-timeout extension
        if 'x-timeout' in contract:
            return contract['x-timeout']

        # Look in info section
        if 'info' in contract and 'x-timeout' in contract['info']:
            return contract['info']['x-timeout']

        # Default
        return 30

    def _check_component_calls(self, comp_a: str, comp_b: str) -> bool:
        """Check if comp_a calls comp_b."""
        comp_a_path = self.components_dir / comp_a

        if not comp_a_path.exists():
            return False

        # Search for imports or references to comp_b
        for py_file in comp_a_path.rglob("*.py"):
            try:
                content = py_file.read_text()
                # Check for various reference patterns
                if any([
                    f"from components.{comp_b}" in content,
                    f"import {comp_b}" in content,
                    f"'{comp_b}'" in content,
                    f'"{comp_b}"' in content,
                ]):
                    return True
            except Exception:
                pass

        return False

    def _check_error_handling_for_dependency(self, comp_a: str, comp_b: str) -> bool:
        """Check if comp_a handles errors from comp_b."""
        comp_a_path = self.components_dir / comp_a

        if not comp_a_path.exists():
            return False

        # Look for try/except around calls to comp_b
        for py_file in comp_a_path.rglob("*.py"):
            try:
                content = py_file.read_text()
                # Check if comp_b is mentioned
                if comp_b not in content:
                    continue

                # Look for error handling patterns
                if any([
                    'try:' in content and 'except' in content,
                    'circuit' in content.lower(),
                    'fallback' in content.lower(),
                    'catch' in content.lower()
                ]):
                    return True
            except Exception:
                pass

        return False

    def _check_retry_logic(self, comp_a: str, comp_b: str) -> bool:
        """Check if comp_a has retry logic for comp_b calls."""
        comp_a_path = self.components_dir / comp_a

        if not comp_a_path.exists():
            return False

        for py_file in comp_a_path.rglob("*.py"):
            try:
                content = py_file.read_text()
                if comp_b not in content:
                    continue

                # Look for retry patterns
                if any([
                    'retry' in content.lower(),
                    'backoff' in content.lower(),
                    'tenacity' in content.lower(),
                    '@retry' in content
                ]):
                    return True
            except Exception:
                pass

        return False

    def _get_component_dependencies(self, comp: str) -> List[str]:
        """Get list of components this component depends on."""
        deps = set()
        comp_path = self.components_dir / comp

        if not comp_path.exists():
            return []

        # Look for imports
        for py_file in comp_path.rglob("*.py"):
            try:
                content = py_file.read_text()
                # Look for "from components.X import" or "from components.X-Y import"
                matches = re.findall(r'from components\.([\w\-]+)', content)
                deps.update(matches)

                # Look for "import components.X" or "import components.X-Y"
                matches = re.findall(r'import components\.([\w\-]+)', content)
                deps.update(matches)
            except Exception:
                pass

        return list(deps)

    def _check_enum_compatibility(self, contract_a: dict, contract_b: dict,
                                  comp_a: str, comp_b: str) -> List[PredictedFailure]:
        """Check for enum value mismatches."""
        failures = []

        # Extract enums from both contracts
        enums_a = self._extract_enums(contract_a)
        enums_b = self._extract_enums(contract_b)

        # Check for common enum names with different values
        for enum_name in set(enums_a.keys()) & set(enums_b.keys()):
            if enums_a[enum_name] != enums_b[enum_name]:
                failures.append(PredictedFailure(
                    failure_type="data_format_mismatch",
                    component_a=comp_a,
                    component_b=comp_b,
                    description=f"Enum '{enum_name}' has different values: {comp_a}={enums_a[enum_name]}, {comp_b}={enums_b[enum_name]}",
                    severity="critical",
                    fix_strategy=f"Standardize enum '{enum_name}' values across all components",
                    test_generation="Create enum validation tests"
                ))

        return failures

    def _check_null_handling(self, contract_a: dict, contract_b: dict,
                            comp_a: str, comp_b: str) -> List[PredictedFailure]:
        """Check for null handling incompatibilities."""
        failures = []

        # Check if one contract allows nulls and the other doesn't
        nullables_a = self._extract_nullable_fields(contract_a)
        nullables_b = self._extract_nullable_fields(contract_b)

        # Find fields that differ in nullability
        common_fields = set(nullables_a.keys()) & set(nullables_b.keys())

        for field in common_fields:
            if nullables_a[field] != nullables_b[field]:
                failures.append(PredictedFailure(
                    failure_type="data_format_mismatch",
                    component_a=comp_a,
                    component_b=comp_b,
                    description=f"Field '{field}' nullable mismatch: {comp_a}={nullables_a[field]}, {comp_b}={nullables_b[field]}",
                    severity="warning",
                    fix_strategy=f"Standardize null handling for field '{field}'",
                    test_generation="Create null handling tests"
                ))

        return failures

    def _extract_enums(self, contract: dict) -> Dict[str, List]:
        """Extract enum definitions from contract."""
        enums = {}

        def search_enums(obj, path=""):
            if isinstance(obj, dict):
                if 'enum' in obj:
                    enums[path] = obj['enum']
                for key, value in obj.items():
                    search_enums(value, f"{path}.{key}" if path else key)
            elif isinstance(obj, list):
                for i, item in enumerate(obj):
                    search_enums(item, f"{path}[{i}]")

        search_enums(contract)
        return enums

    def _extract_nullable_fields(self, contract: dict) -> Dict[str, bool]:
        """Extract nullable field information."""
        nullables = {}

        def search_nullables(obj, path=""):
            if isinstance(obj, dict):
                if 'nullable' in obj:
                    nullables[path] = obj['nullable']
                for key, value in obj.items():
                    search_nullables(value, f"{path}.{key}" if path else key)
            elif isinstance(obj, list):
                for i, item in enumerate(obj):
                    search_nullables(item, f"{path}[{i}]")

        search_nullables(contract)
        return nullables

    def _deep_search_dict(self, obj, key: str, value: str) -> bool:
        """Deep search for key-value pair in nested dict."""
        if isinstance(obj, dict):
            if obj.get(key) == value:
                return True
            for v in obj.values():
                if self._deep_search_dict(v, key, value):
                    return True
        elif isinstance(obj, list):
            for item in obj:
                if self._deep_search_dict(item, key, value):
                    return True
        return False

    def generate_report(self, prediction: IntegrationPrediction) -> str:
        """Generate formatted report."""
        report = []
        report.append("="*70)
        report.append("INTEGRATION FAILURE PREDICTION")
        report.append("="*70)
        report.append("")
        report.append(f"Total Components: {prediction.total_components}")
        report.append(f"Component Pairs Analyzed: {prediction.total_pairs_analyzed}")
        report.append(f"Predicted Failures: {len(prediction.predicted_failures)}")
        report.append("")

        if prediction.predicted_failures:
            report.append("Breakdown:")
            report.append(f"  Data Type Incompatibilities: {prediction.data_type_incompatibilities}")
            report.append(f"  Timeout Cascade Risks: {prediction.timeout_cascade_risks}")
            report.append(f"  Error Propagation Issues: {prediction.error_propagation_issues}")
            report.append(f"  Circular Dependencies: {prediction.circular_dependencies}")
            report.append("")

            # Group by severity
            critical = [f for f in prediction.predicted_failures if f.severity == "critical"]
            warnings = [f for f in prediction.predicted_failures if f.severity == "warning"]
            info = [f for f in prediction.predicted_failures if f.severity == "info"]

            if critical:
                report.append("CRITICAL ISSUES:")
                report.append("-" * 70)
                for failure in critical:
                    report.append(f"  [{failure.failure_type}]")
                    report.append(f"  Components: {failure.component_a} <-> {failure.component_b}")
                    report.append(f"  Issue: {failure.description}")
                    report.append(f"  Fix: {failure.fix_strategy}")
                    report.append("")

            if warnings:
                report.append("WARNINGS:")
                report.append("-" * 70)
                for failure in warnings:
                    report.append(f"  [{failure.failure_type}]")
                    report.append(f"  Components: {failure.component_a} <-> {failure.component_b}")
                    report.append(f"  Issue: {failure.description}")
                    report.append(f"  Fix: {failure.fix_strategy}")
                    report.append("")

            if info:
                report.append("INFORMATIONAL:")
                report.append("-" * 70)
                for failure in info:
                    report.append(f"  [{failure.failure_type}]")
                    report.append(f"  Components: {failure.component_a} <-> {failure.component_b}")
                    report.append(f"  Issue: {failure.description}")
                    report.append("")
        else:
            report.append("✅ No integration failures predicted!")

        report.append("="*70)
        return "\n".join(report)

    def save_prediction(self, prediction: IntegrationPrediction, output_path: Path):
        """Save prediction to JSON file."""
        with open(output_path, 'w') as f:
            json.dump(prediction.to_dict(), f, indent=2)


def main():
    """CLI interface."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: integration_predictor.py <command> [options]")
        print("\nCommands:")
        print("  predict              - Predict integration failures")
        print("  generate-tests       - Generate integration tests")
        print("  report [--json]      - Generate detailed report")
        print("  save <output-file>   - Save prediction to JSON")
        sys.exit(1)

    command = sys.argv[1]
    predictor = IntegrationPredictor(Path.cwd())

    if command == "predict":
        prediction = predictor.predict_integration_failures()
        print(predictor.generate_report(prediction))
        sys.exit(0 if len(prediction.predicted_failures) == 0 else 1)

    elif command == "generate-tests":
        prediction = predictor.predict_integration_failures()
        tests = predictor.generate_integration_test_suite(prediction)
        print(tests)

    elif command == "report":
        prediction = predictor.predict_integration_failures()
        if "--json" in sys.argv:
            print(json.dumps(prediction.to_dict(), indent=2))
        else:
            print(predictor.generate_report(prediction))

    elif command == "save":
        if len(sys.argv) < 3:
            print("Error: output file required")
            sys.exit(1)

        prediction = predictor.predict_integration_failures()
        output_path = Path(sys.argv[2])
        predictor.save_prediction(prediction, output_path)
        print(f"Prediction saved to {output_path}")

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == '__main__':
    main()
