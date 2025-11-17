#!/usr/bin/env python3
"""
Tests for System-Wide Validator

Part of v0.4.0 quality enhancement system - Batch 3.
"""

import unittest
import tempfile
import shutil
from pathlib import Path
import json
import sys

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent))

from system_validator import (
    SystemValidator,
    ValidationCheck,
    DeploymentReadiness
)


class TestValidationCheck(unittest.TestCase):
    """Test ValidationCheck dataclass."""

    def test_validation_check_creation(self):
        """Test creating a validation check."""
        check = ValidationCheck(
            check_name="Test Check",
            passed=True,
            details="All good",
            critical=True
        )

        self.assertEqual(check.check_name, "Test Check")
        self.assertTrue(check.passed)
        self.assertEqual(check.details, "All good")
        self.assertTrue(check.critical)

    def test_validation_check_failed(self):
        """Test failed validation check."""
        check = ValidationCheck(
            check_name="Test Check",
            passed=False,
            details="Something wrong",
            critical=True
        )

        self.assertFalse(check.passed)
        self.assertTrue(check.critical)

    def test_validation_check_non_critical(self):
        """Test non-critical validation check."""
        check = ValidationCheck(
            check_name="Warning Check",
            passed=False,
            details="Non-critical issue",
            critical=False
        )

        self.assertFalse(check.passed)
        self.assertFalse(check.critical)


class TestDeploymentReadiness(unittest.TestCase):
    """Test DeploymentReadiness dataclass."""

    def test_deployment_readiness_creation(self):
        """Test creating deployment readiness."""
        checks = [
            ValidationCheck("Check 1", True, "Pass", True),
            ValidationCheck("Check 2", False, "Fail", True)
        ]

        readiness = DeploymentReadiness(
            ready_for_deployment=False,
            checks_passed=1,
            checks_failed=1,
            critical_failures=1,
            checks=checks,
            summary="Not ready"
        )

        self.assertFalse(readiness.ready_for_deployment)
        self.assertEqual(readiness.checks_passed, 1)
        self.assertEqual(readiness.checks_failed, 1)
        self.assertEqual(readiness.critical_failures, 1)
        self.assertEqual(len(readiness.checks), 2)

    def test_deployment_readiness_ready(self):
        """Test deployment readiness when ready."""
        checks = [
            ValidationCheck("Check 1", True, "Pass", True),
            ValidationCheck("Check 2", True, "Pass", True)
        ]

        readiness = DeploymentReadiness(
            ready_for_deployment=True,
            checks_passed=2,
            checks_failed=0,
            critical_failures=0,
            checks=checks,
            summary="Ready"
        )

        self.assertTrue(readiness.ready_for_deployment)
        self.assertEqual(readiness.checks_passed, 2)
        self.assertEqual(readiness.checks_failed, 0)
        self.assertEqual(readiness.critical_failures, 0)


class TestSystemValidator(unittest.TestCase):
    """Test SystemValidator class."""

    def setUp(self):
        """Set up test environment."""
        self.temp_dir = tempfile.mkdtemp()
        self.project_root = Path(self.temp_dir)

        # Create basic structure
        (self.project_root / "orchestration").mkdir()
        (self.project_root / "components").mkdir()
        (self.project_root / "contracts").mkdir()
        (self.project_root / "tests" / "integration").mkdir(parents=True)

        self.validator = SystemValidator(self.project_root)

    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.temp_dir)

    def test_init(self):
        """Test validator initialization."""
        self.assertEqual(self.validator.project_root, self.project_root)

    def test_generate_summary_ready(self):
        """Test summary generation when ready."""
        summary = self.validator._generate_summary(True, 8, 0, 0)
        self.assertIn("READY FOR DEPLOYMENT", summary)
        self.assertIn("8/8", summary)
        self.assertIn("✅", summary)

    def test_generate_summary_not_ready(self):
        """Test summary generation when not ready."""
        summary = self.validator._generate_summary(False, 5, 3, 2)
        self.assertIn("NOT READY", summary)
        self.assertIn("2 critical failures", summary)
        self.assertIn("❌", summary)

    def test_validate_integration_tests_not_run(self):
        """Test integration tests validation when not run."""
        check = self.validator.validate_integration_tests_pass()

        self.assertFalse(check.passed)
        self.assertIn("not run", check.details)
        self.assertTrue(check.critical)

    def test_validate_integration_tests_passing(self):
        """Test integration tests validation when passing."""
        test_results = self.project_root / "tests" / "integration" / "TEST-RESULTS.md"
        test_results.write_text("All tests passing\n0 failed")

        check = self.validator.validate_integration_tests_pass()

        self.assertTrue(check.passed)
        self.assertIn("All integration tests passing", check.details)

    def test_validate_integration_tests_failing(self):
        """Test integration tests validation when failing."""
        test_results = self.project_root / "tests" / "integration" / "TEST-RESULTS.md"
        test_results.write_text("Some tests failing\n2 failed")

        check = self.validator.validate_integration_tests_pass()

        self.assertFalse(check.passed)
        self.assertIn("failures", check.details)

    def test_validate_no_integration_failures_predictor_missing(self):
        """Test integration failure prediction when predictor not available."""
        check = self.validator.validate_no_integration_failures()

        # Should pass with warning when predictor not available
        self.assertTrue(check.passed)
        self.assertIn("not available", check.details)
        self.assertFalse(check.critical)

    def test_validate_system_integration(self):
        """Test full system validation integration."""
        # Create minimal passing system
        test_results = self.project_root / "tests" / "integration" / "TEST-RESULTS.md"
        test_results.write_text("All tests passing")

        readiness = self.validator.validate_system()

        # Should have all 8 checks
        self.assertEqual(len(readiness.checks), 8)

        # Check that we get a summary
        self.assertIsNotNone(readiness.summary)

        # Check counts are calculated
        self.assertEqual(readiness.checks_passed + readiness.checks_failed, 8)

        # Check that it's a boolean
        self.assertIsInstance(readiness.ready_for_deployment, bool)

    def test_generate_deployment_readiness_report(self):
        """Test deployment readiness report generation."""
        checks = [
            ValidationCheck("Check 1", True, "Passed", True),
            ValidationCheck("Check 2", False, "Failed", True),
            ValidationCheck("Check 3", True, "Passed", False)
        ]

        readiness = DeploymentReadiness(
            ready_for_deployment=False,
            checks_passed=2,
            checks_failed=1,
            critical_failures=1,
            checks=checks,
            summary="Not ready"
        )

        report = self.validator.generate_deployment_readiness_report(readiness)

        # Check report contains key elements
        self.assertIn("DEPLOYMENT READINESS REPORT", report)
        self.assertIn("Checks Passed: 2", report)
        self.assertIn("Checks Failed: 1", report)
        self.assertIn("Critical Failures: 1", report)
        self.assertIn("Check 1", report)
        self.assertIn("Check 2", report)
        self.assertIn("Check 3", report)
        self.assertIn("NOT ready for deployment", report)
        self.assertIn("[CRITICAL]", report)

    def test_generate_deployment_readiness_report_ready(self):
        """Test deployment readiness report when ready."""
        checks = [
            ValidationCheck("Check 1", True, "Passed", True),
            ValidationCheck("Check 2", True, "Passed", True)
        ]

        readiness = DeploymentReadiness(
            ready_for_deployment=True,
            checks_passed=2,
            checks_failed=0,
            critical_failures=0,
            checks=checks,
            summary="Ready"
        )

        report = self.validator.generate_deployment_readiness_report(readiness)

        self.assertIn("ready for deployment", report.lower())
        self.assertIn("Checks Passed: 2", report)
        self.assertIn("Checks Failed: 0", report)
        self.assertIn("✅", report)

    def test_validate_system_critical_failures_block_deployment(self):
        """Test that critical failures block deployment."""
        # Create system with failing integration tests (critical)
        # Don't create TEST-RESULTS.md - will fail critically

        readiness = self.validator.validate_system()

        # Should not be ready due to critical failures
        self.assertFalse(readiness.ready_for_deployment)
        self.assertGreater(readiness.critical_failures, 0)

    def test_validate_system_structure(self):
        """Test validate_system returns proper structure."""
        readiness = self.validator.validate_system()

        # Verify all required fields exist
        self.assertIsInstance(readiness.ready_for_deployment, bool)
        self.assertIsInstance(readiness.checks_passed, int)
        self.assertIsInstance(readiness.checks_failed, int)
        self.assertIsInstance(readiness.critical_failures, int)
        self.assertIsInstance(readiness.checks, list)
        self.assertIsInstance(readiness.summary, str)

        # Verify all checks have required fields
        for check in readiness.checks:
            self.assertIsInstance(check.check_name, str)
            self.assertIsInstance(check.passed, bool)
            self.assertIsInstance(check.details, str)
            self.assertIsInstance(check.critical, bool)

    def test_validate_system_counts_match(self):
        """Test that check counts match the actual checks."""
        readiness = self.validator.validate_system()

        # Count manually
        passed = sum(1 for c in readiness.checks if c.passed)
        failed = sum(1 for c in readiness.checks if not c.passed)
        critical_fail = sum(1 for c in readiness.checks if not c.passed and c.critical)

        # Verify counts match
        self.assertEqual(readiness.checks_passed, passed)
        self.assertEqual(readiness.checks_failed, failed)
        self.assertEqual(readiness.critical_failures, critical_fail)

    def test_validate_system_deployment_logic(self):
        """Test that deployment readiness logic is correct."""
        readiness = self.validator.validate_system()

        # If there are critical failures, should not be ready
        if readiness.critical_failures > 0:
            self.assertFalse(readiness.ready_for_deployment)

        # If there are no critical failures, should be ready
        if readiness.critical_failures == 0:
            self.assertTrue(readiness.ready_for_deployment)

    def test_report_formatting(self):
        """Test that report is properly formatted."""
        checks = [
            ValidationCheck("Test Check", True, "Details", True)
        ]

        readiness = DeploymentReadiness(
            ready_for_deployment=True,
            checks_passed=1,
            checks_failed=0,
            critical_failures=0,
            checks=checks,
            summary="Ready"
        )

        report = self.validator.generate_deployment_readiness_report(readiness)

        # Check for separator lines
        self.assertIn("="*70, report)

        # Check for proper structure (title, summary, details, footer)
        lines = report.split('\n')
        self.assertGreater(len(lines), 10)  # Should have multiple lines

    def test_all_validation_checks_present(self):
        """Test that all 8 validation checks are present."""
        readiness = self.validator.validate_system()

        # Should have exactly 8 checks
        self.assertEqual(len(readiness.checks), 8)

        # Check for expected check names
        check_names = [c.check_name for c in readiness.checks]

        expected_checks = [
            "Requirements Implementation",
            "Contract Satisfaction",
            "Component Verification",
            "Integration Tests",
            "Defensive Patterns",
            "Cross-Component Consistency",
            "Semantic Correctness",
            "Integration Failure Prediction"
        ]

        for expected in expected_checks:
            self.assertIn(expected, check_names,
                         f"Missing expected check: {expected}")

    def test_validate_no_components_doesnt_fail(self):
        """Test that missing components don't cause critical failures."""
        # Empty components directory
        readiness = self.validator.validate_system()

        # Should still complete validation
        self.assertIsNotNone(readiness)
        self.assertEqual(len(readiness.checks), 8)

    def test_validate_error_recovery(self):
        """Test that errors in individual checks don't crash the validator."""
        # This test ensures robustness
        readiness = self.validator.validate_system()

        # Should always return a result, even if checks error out
        self.assertIsNotNone(readiness)
        self.assertIsInstance(readiness.checks, list)
        self.assertGreater(len(readiness.checks), 0)


class TestCLIInterface(unittest.TestCase):
    """Test CLI interface."""

    def test_main_function_exists(self):
        """Test that main function exists and is callable."""
        from system_validator import main
        self.assertTrue(callable(main))


if __name__ == '__main__':
    # Run with verbosity
    unittest.main(verbosity=2)
