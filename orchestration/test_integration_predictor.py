#!/usr/bin/env python3
"""
Tests for Integration Failure Predictor

Tests the prediction of integration failures before they occur.
Part of v0.4.0 quality enhancement system - Batch 3.
"""

import unittest
import tempfile
import shutil
from pathlib import Path
import yaml
import json

from integration_predictor import (
    IntegrationPredictor,
    PredictedFailure,
    IntegrationPrediction
)


class TestPredictedFailure(unittest.TestCase):
    """Test PredictedFailure dataclass."""

    def test_create_predicted_failure(self):
        """Test creating a predicted failure."""
        failure = PredictedFailure(
            failure_type="data_format_mismatch",
            component_a="auth-service",
            component_b="user-service",
            description="Date format mismatch",
            severity="critical",
            fix_strategy="Standardize on ISO8601",
            test_generation="Create date format tests"
        )

        self.assertEqual(failure.failure_type, "data_format_mismatch")
        self.assertEqual(failure.component_a, "auth-service")
        self.assertEqual(failure.component_b, "user-service")
        self.assertEqual(failure.severity, "critical")

    def test_to_dict(self):
        """Test converting failure to dictionary."""
        failure = PredictedFailure(
            failure_type="timeout_cascade",
            component_a="api",
            component_b="backend",
            description="Timeout issue",
            severity="warning",
            fix_strategy="Increase timeout",
            test_generation="Create timeout tests"
        )

        failure_dict = failure.to_dict()

        self.assertIsInstance(failure_dict, dict)
        self.assertEqual(failure_dict['failure_type'], "timeout_cascade")
        self.assertEqual(failure_dict['severity'], "warning")


class TestIntegrationPrediction(unittest.TestCase):
    """Test IntegrationPrediction dataclass."""

    def test_create_prediction(self):
        """Test creating a prediction."""
        failures = [
            PredictedFailure(
                failure_type="data_format_mismatch",
                component_a="a",
                component_b="b",
                description="Test",
                severity="critical",
                fix_strategy="Fix",
                test_generation="Test"
            )
        ]

        prediction = IntegrationPrediction(
            total_components=3,
            total_pairs_analyzed=3,
            predicted_failures=failures,
            data_type_incompatibilities=1,
            timeout_cascade_risks=0,
            error_propagation_issues=0,
            circular_dependencies=0
        )

        self.assertEqual(prediction.total_components, 3)
        self.assertEqual(len(prediction.predicted_failures), 1)

    def test_to_dict(self):
        """Test converting prediction to dictionary."""
        prediction = IntegrationPrediction(
            total_components=2,
            total_pairs_analyzed=1,
            predicted_failures=[],
            data_type_incompatibilities=0,
            timeout_cascade_risks=0,
            error_propagation_issues=0,
            circular_dependencies=0
        )

        pred_dict = prediction.to_dict()

        self.assertIsInstance(pred_dict, dict)
        self.assertEqual(pred_dict['total_components'], 2)
        self.assertIsInstance(pred_dict['predicted_failures'], list)


class TestIntegrationPredictorInit(unittest.TestCase):
    """Test IntegrationPredictor initialization."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.contracts_dir = self.test_dir / "contracts"
        self.orchestration_dir = self.test_dir / "orchestration"

        self.components_dir.mkdir()
        self.contracts_dir.mkdir()
        self.orchestration_dir.mkdir()

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_init_with_existing_patterns(self):
        """Test initialization with existing patterns file."""
        patterns = {
            'common_integration_failures': {
                'test_pattern': {
                    'fix_strategy': 'test fix'
                }
            }
        }

        patterns_file = self.orchestration_dir / "integration_patterns.yaml"
        with open(patterns_file, 'w') as f:
            yaml.dump(patterns, f)

        predictor = IntegrationPredictor(self.test_dir)

        self.assertEqual(predictor.project_root, self.test_dir)
        self.assertIn('common_integration_failures', predictor.patterns)
        self.assertIn('test_pattern', predictor.patterns['common_integration_failures'])

    def test_init_without_patterns(self):
        """Test initialization without patterns file."""
        predictor = IntegrationPredictor(self.test_dir)

        self.assertIsNotNone(predictor.patterns)
        self.assertIn('common_integration_failures', predictor.patterns)

    def test_default_patterns(self):
        """Test default patterns are comprehensive."""
        predictor = IntegrationPredictor(self.test_dir)
        patterns = predictor.patterns['common_integration_failures']

        self.assertIn('data_format_mismatch', patterns)
        self.assertIn('missing_error_handling', patterns)
        self.assertIn('timeout_cascade', patterns)
        self.assertIn('circular_dependency', patterns)


class TestLoadComponents(unittest.TestCase):
    """Test loading components and contracts."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.contracts_dir = self.test_dir / "contracts"

        self.components_dir.mkdir()
        self.contracts_dir.mkdir()

        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_load_all_components_empty(self):
        """Test loading components from empty directory."""
        components = self.predictor._load_all_components()

        self.assertEqual(components, [])

    def test_load_all_components(self):
        """Test loading multiple components."""
        (self.components_dir / "auth-service").mkdir()
        (self.components_dir / "user-service").mkdir()
        (self.components_dir / "payment-service").mkdir()
        (self.components_dir / ".hidden").mkdir()

        components = self.predictor._load_all_components()

        self.assertEqual(len(components), 3)
        self.assertIn("auth-service", components)
        self.assertIn("user-service", components)
        self.assertIn("payment-service", components)
        self.assertNotIn(".hidden", components)

    def test_load_all_contracts_empty(self):
        """Test loading contracts from empty directory."""
        contracts = self.predictor._load_all_contracts()

        self.assertEqual(contracts, {})

    def test_load_all_contracts(self):
        """Test loading multiple contracts."""
        contract1 = {
            'openapi': '3.0.0',
            'info': {'title': 'Auth API'}
        }
        contract2 = {
            'openapi': '3.0.0',
            'info': {'title': 'User API'}
        }

        with open(self.contracts_dir / "auth-service.yaml", 'w') as f:
            yaml.dump(contract1, f)

        with open(self.contracts_dir / "user-service_api.yaml", 'w') as f:
            yaml.dump(contract2, f)

        contracts = self.predictor._load_all_contracts()

        self.assertEqual(len(contracts), 2)
        self.assertIn("auth-service", contracts)
        self.assertIn("user-service", contracts)

    def test_load_contracts_handles_invalid_yaml(self):
        """Test that invalid YAML is handled gracefully."""
        invalid_file = self.contracts_dir / "invalid.yaml"
        invalid_file.write_text("invalid: yaml: content:")

        contracts = self.predictor._load_all_contracts()

        # Should not crash, just skip invalid file
        self.assertIsInstance(contracts, dict)


class TestDataTypeCompatibility(unittest.TestCase):
    """Test data type compatibility analysis."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.contracts_dir = self.test_dir / "contracts"

        self.components_dir.mkdir()
        self.contracts_dir.mkdir()

        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_datetime_format_mismatch(self):
        """Test detecting datetime format mismatches."""
        contract_a = {
            'description': 'Uses ISO8601 format for dates'
        }
        contract_b = {
            'description': 'Uses Unix timestamp for dates'
        }

        contracts = {
            'service-a': contract_a,
            'service-b': contract_b
        }

        failures = self.predictor.analyze_data_type_compatibility(
            'service-a', 'service-b', contracts
        )

        self.assertEqual(len(failures), 1)
        self.assertEqual(failures[0].failure_type, "data_format_mismatch")
        self.assertEqual(failures[0].severity, "critical")

    def test_id_format_mismatch(self):
        """Test detecting ID format mismatches."""
        contract_a = {
            'components': {
                'schemas': {
                    'User': {
                        'properties': {
                            'id': {'type': 'string', 'format': 'uuid'}
                        }
                    }
                }
            }
        }
        contract_b = {
            'components': {
                'schemas': {
                    'User': {
                        'properties': {
                            'id': {'type': 'integer'}
                        }
                    }
                }
            }
        }

        contracts = {
            'service-a': contract_a,
            'service-b': contract_b
        }

        failures = self.predictor.analyze_data_type_compatibility(
            'service-a', 'service-b', contracts
        )

        self.assertEqual(len(failures), 1)
        self.assertEqual(failures[0].failure_type, "data_format_mismatch")

    def test_no_format_mismatch(self):
        """Test when formats match."""
        contract_a = {
            'description': 'Uses ISO8601 format'
        }
        contract_b = {
            'description': 'Uses ISO8601 format'
        }

        contracts = {
            'service-a': contract_a,
            'service-b': contract_b
        }

        failures = self.predictor.analyze_data_type_compatibility(
            'service-a', 'service-b', contracts
        )

        self.assertEqual(len(failures), 0)

    def test_missing_contracts(self):
        """Test when contracts are missing."""
        failures = self.predictor.analyze_data_type_compatibility(
            'service-a', 'service-b', {}
        )

        self.assertEqual(len(failures), 0)


class TestErrorPropagation(unittest.TestCase):
    """Test error propagation analysis."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.contracts_dir = self.test_dir / "contracts"

        self.components_dir.mkdir()
        self.contracts_dir.mkdir()

        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_missing_error_handling(self):
        """Test detecting missing error handling."""
        # Create components
        comp_a_dir = self.components_dir / "service_a"
        comp_a_dir.mkdir()

        # Service A calls B without error handling - use proper import
        code = 'from components.service_b import api\nresult = api.call()'
        (comp_a_dir / "main.py").write_text(code)

        failures = self.predictor.analyze_error_propagation(
            'service_a', 'service_b', {}
        )

        # Should detect missing error handling and missing retry
        self.assertGreaterEqual(len(failures), 1)
        self.assertTrue(any(f.failure_type == "missing_error_handling" for f in failures))

    def test_has_error_handling(self):
        """Test detecting proper error handling."""
        comp_a_dir = self.components_dir / "service_a"
        comp_a_dir.mkdir()

        # Service A has error handling and retry
        code = 'from components.service_b import api\nfrom tenacity import retry\n\n@retry\ndef call():\n    try:\n        return api.call()\n    except:\n        pass'
        (comp_a_dir / "main.py").write_text(code)

        failures = self.predictor.analyze_error_propagation(
            'service_a', 'service_b', {}
        )

        # Should have no failures
        self.assertEqual(len(failures), 0)

    def test_no_dependency(self):
        """Test when components don't call each other."""
        comp_a_dir = self.components_dir / "service_a"
        comp_a_dir.mkdir()

        code = '''
def standalone_function():
    return "no dependencies"
'''
        (comp_a_dir / "main.py").write_text(code)

        failures = self.predictor.analyze_error_propagation(
            'service_a', 'service_b', {}
        )

        self.assertEqual(len(failures), 0)


class TestTimeoutCascade(unittest.TestCase):
    """Test timeout cascade analysis."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.contracts_dir = self.test_dir / "contracts"

        self.components_dir.mkdir()
        self.contracts_dir.mkdir()

        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_timeout_cascade_detected(self):
        """Test detecting timeout cascade."""
        # Service A calls B
        comp_a_dir = self.components_dir / "service-a"
        comp_a_dir.mkdir()
        (comp_a_dir / "main.py").write_text("import service-b")

        contract_a = {'x-timeout': 10}  # 10 seconds
        contract_b = {'x-timeout': 9}   # 9 seconds

        contracts = {
            'service-a': contract_a,
            'service-b': contract_b
        }

        failures = self.predictor.analyze_timeout_cascade(
            'service-a', 'service-b', contracts
        )

        self.assertEqual(len(failures), 1)
        self.assertEqual(failures[0].failure_type, "timeout_cascade")
        self.assertEqual(failures[0].severity, "warning")

    def test_timeout_cascade_safe(self):
        """Test when timeout hierarchy is safe."""
        comp_a_dir = self.components_dir / "service-a"
        comp_a_dir.mkdir()
        (comp_a_dir / "main.py").write_text("import service-b")

        contract_a = {'x-timeout': 30}  # 30 seconds
        contract_b = {'x-timeout': 10}  # 10 seconds

        contracts = {
            'service-a': contract_a,
            'service-b': contract_b
        }

        failures = self.predictor.analyze_timeout_cascade(
            'service-a', 'service-b', contracts
        )

        self.assertEqual(len(failures), 0)

    def test_no_dependency_no_cascade(self):
        """Test no cascade when no dependency."""
        contract_a = {'x-timeout': 10}
        contract_b = {'x-timeout': 9}

        contracts = {
            'service-a': contract_a,
            'service-b': contract_b
        }

        failures = self.predictor.analyze_timeout_cascade(
            'service-a', 'service-b', contracts
        )

        self.assertEqual(len(failures), 0)


class TestCircularDependencies(unittest.TestCase):
    """Test circular dependency detection."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.contracts_dir = self.test_dir / "contracts"

        self.components_dir.mkdir()
        self.contracts_dir.mkdir()

        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_detect_simple_cycle(self):
        """Test detecting simple A->B->A cycle."""
        # Create components
        comp_a_dir = self.components_dir / "service_a"
        comp_b_dir = self.components_dir / "service_b"
        comp_a_dir.mkdir()
        comp_b_dir.mkdir()

        # A imports B
        (comp_a_dir / "main.py").write_text("from components.service_b import something")

        # B imports A
        (comp_b_dir / "main.py").write_text("from components.service_a import something")

        components = ['service_a', 'service_b']

        failures = self.predictor.analyze_circular_dependencies(components, {})

        self.assertGreaterEqual(len(failures), 1)
        self.assertTrue(any(f.failure_type == "circular_dependency" for f in failures))

    def test_detect_three_way_cycle(self):
        """Test detecting A->B->C->A cycle."""
        # Create components
        comp_a_dir = self.components_dir / "service_a"
        comp_b_dir = self.components_dir / "service_b"
        comp_c_dir = self.components_dir / "service_c"
        comp_a_dir.mkdir()
        comp_b_dir.mkdir()
        comp_c_dir.mkdir()

        # A -> B -> C -> A
        (comp_a_dir / "main.py").write_text("from components.service_b import b")
        (comp_b_dir / "main.py").write_text("from components.service_c import c")
        (comp_c_dir / "main.py").write_text("from components.service_a import a")

        components = ['service_a', 'service_b', 'service_c']

        failures = self.predictor.analyze_circular_dependencies(components, {})

        self.assertGreaterEqual(len(failures), 1)

    def test_no_cycle(self):
        """Test when there are no cycles."""
        comp_a_dir = self.components_dir / "service_a"
        comp_b_dir = self.components_dir / "service_b"
        comp_a_dir.mkdir()
        comp_b_dir.mkdir()

        # A -> B (no reverse dependency)
        (comp_a_dir / "main.py").write_text("from components.service_b import something")
        (comp_b_dir / "main.py").write_text("def something(): pass")

        components = ['service_a', 'service_b']

        failures = self.predictor.analyze_circular_dependencies(components, {})

        self.assertEqual(len(failures), 0)


class TestPredictIntegrationFailures(unittest.TestCase):
    """Test complete integration failure prediction."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.contracts_dir = self.test_dir / "contracts"

        self.components_dir.mkdir()
        self.contracts_dir.mkdir()

        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_predict_empty_project(self):
        """Test prediction for empty project."""
        prediction = self.predictor.predict_integration_failures()

        self.assertEqual(prediction.total_components, 0)
        self.assertEqual(prediction.total_pairs_analyzed, 0)
        self.assertEqual(len(prediction.predicted_failures), 0)

    def test_predict_single_component(self):
        """Test prediction with single component."""
        (self.components_dir / "service-a").mkdir()

        prediction = self.predictor.predict_integration_failures()

        self.assertEqual(prediction.total_components, 1)
        self.assertEqual(prediction.total_pairs_analyzed, 0)

    def test_predict_multiple_components(self):
        """Test prediction with multiple components."""
        # Create components
        comp_a_dir = self.components_dir / "service-a"
        comp_b_dir = self.components_dir / "service-b"
        comp_a_dir.mkdir()
        comp_b_dir.mkdir()

        # A calls B without error handling
        (comp_a_dir / "main.py").write_text("import service-b\nresponse = call_b()")

        # Mismatched datetime formats
        contract_a = {'description': 'Uses ISO8601'}
        contract_b = {'description': 'Uses Unix timestamp'}

        with open(self.contracts_dir / "service-a.yaml", 'w') as f:
            yaml.dump(contract_a, f)
        with open(self.contracts_dir / "service-b.yaml", 'w') as f:
            yaml.dump(contract_b, f)

        prediction = self.predictor.predict_integration_failures()

        self.assertEqual(prediction.total_components, 2)
        self.assertEqual(prediction.total_pairs_analyzed, 1)
        self.assertGreater(len(prediction.predicted_failures), 0)


class TestGenerateIntegrationTestSuite(unittest.TestCase):
    """Test integration test suite generation."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_generate_tests_no_failures(self):
        """Test generating tests when no failures predicted."""
        prediction = IntegrationPrediction(
            total_components=2,
            total_pairs_analyzed=1,
            predicted_failures=[],
            data_type_incompatibilities=0,
            timeout_cascade_risks=0,
            error_propagation_issues=0,
            circular_dependencies=0
        )

        tests = self.predictor.generate_integration_test_suite(prediction)

        self.assertIn("import pytest", tests)
        self.assertIn("def test_no_failures_predicted", tests)

    def test_generate_tests_with_failures(self):
        """Test generating tests for predicted failures."""
        failures = [
            PredictedFailure(
                failure_type="data_format_mismatch",
                component_a="service-a",
                component_b="service-b",
                description="Date format mismatch",
                severity="critical",
                fix_strategy="Standardize on ISO8601",
                test_generation="Create format tests"
            ),
            PredictedFailure(
                failure_type="timeout_cascade",
                component_a="api",
                component_b="backend",
                description="Timeout cascade",
                severity="warning",
                fix_strategy="Increase timeout",
                test_generation="Create timeout tests"
            )
        ]

        prediction = IntegrationPrediction(
            total_components=3,
            total_pairs_analyzed=3,
            predicted_failures=failures,
            data_type_incompatibilities=1,
            timeout_cascade_risks=1,
            error_propagation_issues=0,
            circular_dependencies=0
        )

        tests = self.predictor.generate_integration_test_suite(prediction)

        self.assertIn("import pytest", tests)
        self.assertIn("def test_data_format_mismatch", tests)
        self.assertIn("def test_timeout_cascade", tests)
        self.assertIn("service-a", tests)
        self.assertIn("service-b", tests)


class TestGenerateReport(unittest.TestCase):
    """Test report generation."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_generate_report_no_failures(self):
        """Test report with no failures."""
        prediction = IntegrationPrediction(
            total_components=3,
            total_pairs_analyzed=3,
            predicted_failures=[],
            data_type_incompatibilities=0,
            timeout_cascade_risks=0,
            error_propagation_issues=0,
            circular_dependencies=0
        )

        report = self.predictor.generate_report(prediction)

        self.assertIn("INTEGRATION FAILURE PREDICTION", report)
        self.assertIn("Total Components: 3", report)
        self.assertIn("No integration failures predicted", report)

    def test_generate_report_with_failures(self):
        """Test report with failures."""
        failures = [
            PredictedFailure(
                failure_type="data_format_mismatch",
                component_a="service-a",
                component_b="service-b",
                description="Date format mismatch",
                severity="critical",
                fix_strategy="Standardize on ISO8601",
                test_generation="Create tests"
            ),
            PredictedFailure(
                failure_type="missing_error_handling",
                component_a="api",
                component_b="backend",
                description="No error handling",
                severity="warning",
                fix_strategy="Add try/except",
                test_generation="Create error tests"
            )
        ]

        prediction = IntegrationPrediction(
            total_components=3,
            total_pairs_analyzed=3,
            predicted_failures=failures,
            data_type_incompatibilities=1,
            timeout_cascade_risks=0,
            error_propagation_issues=1,
            circular_dependencies=0
        )

        report = self.predictor.generate_report(prediction)

        self.assertIn("Predicted Failures: 2", report)
        self.assertIn("CRITICAL ISSUES:", report)
        self.assertIn("WARNINGS:", report)
        self.assertIn("service-a", report)
        self.assertIn("Standardize on ISO8601", report)


class TestSavePrediction(unittest.TestCase):
    """Test saving predictions to file."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_save_prediction(self):
        """Test saving prediction to JSON."""
        prediction = IntegrationPrediction(
            total_components=2,
            total_pairs_analyzed=1,
            predicted_failures=[],
            data_type_incompatibilities=0,
            timeout_cascade_risks=0,
            error_propagation_issues=0,
            circular_dependencies=0
        )

        output_file = self.test_dir / "prediction.json"
        self.predictor.save_prediction(prediction, output_file)

        self.assertTrue(output_file.exists())

        with open(output_file, 'r') as f:
            loaded = json.load(f)

        self.assertEqual(loaded['total_components'], 2)
        self.assertEqual(loaded['total_pairs_analyzed'], 1)


class TestHelperMethods(unittest.TestCase):
    """Test helper methods."""

    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.components_dir = self.test_dir / "components"
        self.components_dir.mkdir()

        self.predictor = IntegrationPredictor(self.test_dir)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir)

    def test_extract_datetime_formats(self):
        """Test extracting datetime formats from contracts."""
        contract = {'description': 'Uses ISO8601 format'}
        fmt = self.predictor._extract_datetime_formats(contract)
        self.assertEqual(fmt, 'ISO8601')

        contract = {'description': 'Uses Unix timestamp'}
        fmt = self.predictor._extract_datetime_formats(contract)
        self.assertEqual(fmt, 'Unix timestamp')

    def test_extract_id_formats(self):
        """Test extracting ID formats from contracts."""
        contract = {'id': {'type': 'string', 'format': 'uuid'}}
        fmt = self.predictor._extract_id_formats(contract)
        self.assertEqual(fmt, 'UUID')

    def test_extract_timeout(self):
        """Test extracting timeout values."""
        contract = {'x-timeout': 60}
        timeout = self.predictor._extract_timeout(contract)
        self.assertEqual(timeout, 60)

        contract = {}
        timeout = self.predictor._extract_timeout(contract)
        self.assertEqual(timeout, 30)  # default

    def test_check_component_calls(self):
        """Test checking if components call each other."""
        comp_a_dir = self.components_dir / "service-a"
        comp_a_dir.mkdir()

        # No call
        (comp_a_dir / "main.py").write_text("def main(): pass")
        result = self.predictor._check_component_calls('service-a', 'service-b')
        self.assertFalse(result)

        # Has call
        (comp_a_dir / "main.py").write_text("from components.service-b import func")
        result = self.predictor._check_component_calls('service-a', 'service-b')
        self.assertTrue(result)

    def test_get_component_dependencies(self):
        """Test getting component dependencies."""
        comp_a_dir = self.components_dir / "service-a"
        comp_a_dir.mkdir()

        code = 'from components.service_b import b\nfrom components.service_c import c\nimport components.service_d\n'
        (comp_a_dir / "main.py").write_text(code)

        deps = self.predictor._get_component_dependencies('service-a')

        self.assertIn('service_b', deps)
        self.assertIn('service_c', deps)
        self.assertIn('service_d', deps)


def run_tests():
    """Run all tests."""
    unittest.main(argv=[''], exit=False, verbosity=2)


if __name__ == '__main__':
    run_tests()
