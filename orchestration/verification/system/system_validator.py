#!/usr/bin/env python3
"""
System-Wide Validation

Validates entire system before deployment.
Part of v0.4.0 quality enhancement system - Batch 3.
"""

from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass
import json


@dataclass
class ValidationCheck:
    """A single validation check result."""
    check_name: str
    passed: bool
    details: str
    critical: bool


@dataclass
class DeploymentReadiness:
    """Complete deployment readiness assessment."""
    ready_for_deployment: bool
    checks_passed: int
    checks_failed: int
    critical_failures: int
    checks: List[ValidationCheck]
    summary: str


class SystemValidator:
    """Validates entire system."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root).resolve()

    def validate_system(self) -> DeploymentReadiness:
        """Run all validation checks."""
        checks = []

        # Run all checks
        checks.append(self.validate_all_requirements_implemented())
        checks.append(self.validate_all_contracts_satisfied())
        checks.append(self.validate_all_components_verified())
        checks.append(self.validate_integration_tests_pass())
        checks.append(self.validate_defensive_patterns())
        checks.append(self.validate_consistency())
        checks.append(self.validate_semantic_correctness())
        checks.append(self.validate_no_integration_failures())

        # Calculate results
        passed = [c for c in checks if c.passed]
        failed = [c for c in checks if not c.passed]
        critical_failures = [c for c in failed if c.critical]

        ready = len(critical_failures) == 0

        summary = self._generate_summary(ready, len(passed), len(failed), len(critical_failures))

        return DeploymentReadiness(
            ready_for_deployment=ready,
            checks_passed=len(passed),
            checks_failed=len(failed),
            critical_failures=len(critical_failures),
            checks=checks,
            summary=summary
        )

    def validate_all_requirements_implemented(self) -> ValidationCheck:
        """Check that all requirements have implementations and tests."""
        try:
            from orchestration.requirements.requirements_tracker import RequirementsTracker

            tracker = RequirementsTracker(self.project_root)

            # Count total and complete requirements
            total = 0
            complete = 0

            for trace in tracker.requirements.values():
                total += 1
                if trace.is_complete():
                    complete += 1

            if total == 0:
                return ValidationCheck(
                    check_name="Requirements Implementation",
                    passed=True,
                    details="No requirements defined (not using requirements tracking)",
                    critical=False
                )

            passed = complete == total
            percentage = (complete / total * 100) if total > 0 else 0

            return ValidationCheck(
                check_name="Requirements Implementation",
                passed=passed,
                details=f"{complete}/{total} requirements complete ({percentage:.1f}%)",
                critical=True
            )
        except Exception as e:
            return ValidationCheck(
                check_name="Requirements Implementation",
                passed=False,
                details=f"Error checking requirements: {e}",
                critical=True
            )

    def validate_all_contracts_satisfied(self) -> ValidationCheck:
        """Check that all contracts are satisfied."""
        try:
            from orchestration.verification.contracts.contract_enforcer import ContractEnforcer

            enforcer = ContractEnforcer(self.project_root)
            results = enforcer.enforce_all_components()

            if not results:
                return ValidationCheck(
                    check_name="Contract Satisfaction",
                    passed=True,
                    details="No components to validate",
                    critical=False
                )

            total = len(results)
            compliant = sum(1 for r in results.values() if r.compliant)

            passed = compliant == total

            return ValidationCheck(
                check_name="Contract Satisfaction",
                passed=passed,
                details=f"{compliant}/{total} components contract-compliant",
                critical=True
            )
        except Exception as e:
            return ValidationCheck(
                check_name="Contract Satisfaction",
                passed=False,
                details=f"Error checking contracts: {e}",
                critical=True
            )

    def validate_all_components_verified(self) -> ValidationCheck:
        """Check that all components pass verification."""
        try:
            from orchestration.verification.completion.completion_verifier import CompletionVerifier

            verifier = CompletionVerifier(self.project_root)

            components_dir = self.project_root / "components"
            if not components_dir.exists():
                return ValidationCheck(
                    check_name="Component Verification",
                    passed=True,
                    details="No components to verify",
                    critical=False
                )

            components = [d for d in components_dir.iterdir()
                         if d.is_dir() and not d.name.startswith('.')]

            if not components:
                return ValidationCheck(
                    check_name="Component Verification",
                    passed=True,
                    details="No components found",
                    critical=False
                )

            total = len(components)
            complete = 0

            for comp in components:
                verification = verifier.verify_component(comp)
                if verification.is_complete:
                    complete += 1

            passed = complete == total

            return ValidationCheck(
                check_name="Component Verification",
                passed=passed,
                details=f"{complete}/{total} components fully verified",
                critical=True
            )
        except Exception as e:
            return ValidationCheck(
                check_name="Component Verification",
                passed=False,
                details=f"Error verifying components: {e}",
                critical=True
            )

    def validate_integration_tests_pass(self) -> ValidationCheck:
        """Check that integration tests pass."""
        # Simplified check - look for integration test results
        test_results_file = self.project_root / "tests" / "integration" / "TEST-RESULTS.md"

        if not test_results_file.exists():
            return ValidationCheck(
                check_name="Integration Tests",
                passed=False,
                details="Integration tests not run (TEST-RESULTS.md not found)",
                critical=True
            )

        try:
            content = test_results_file.read_text()

            # Simple heuristic: look for "all tests passing" or "0 failed"
            if "all tests passing" in content.lower() or "0 failed" in content.lower():
                return ValidationCheck(
                    check_name="Integration Tests",
                    passed=True,
                    details="All integration tests passing",
                    critical=True
                )
            else:
                return ValidationCheck(
                    check_name="Integration Tests",
                    passed=False,
                    details="Integration tests have failures",
                    critical=True
                )
        except Exception as e:
            return ValidationCheck(
                check_name="Integration Tests",
                passed=False,
                details=f"Error reading integration test results: {e}",
                critical=True
            )

    def validate_defensive_patterns(self) -> ValidationCheck:
        """Check defensive programming compliance."""
        try:
            from orchestration.verification.defensive.defensive_pattern_checker import DefensivePatternChecker

            checker = DefensivePatternChecker()

            components_dir = self.project_root / "components"
            if not components_dir.exists():
                return ValidationCheck(
                    check_name="Defensive Patterns",
                    passed=True,
                    details="No components to check",
                    critical=False
                )

            total_violations = 0
            critical_violations = 0

            for comp_dir in components_dir.iterdir():
                if comp_dir.is_dir() and not comp_dir.name.startswith('.'):
                    report = checker.check_component(comp_dir)
                    total_violations += report.total_violations
                    critical_violations += report.critical_violations

            passed = critical_violations == 0

            return ValidationCheck(
                check_name="Defensive Patterns",
                passed=passed,
                details=f"{total_violations} violations ({critical_violations} critical)",
                critical=True
            )
        except Exception as e:
            return ValidationCheck(
                check_name="Defensive Patterns",
                passed=False,
                details=f"Error checking defensive patterns: {e}",
                critical=True
            )

    def validate_consistency(self) -> ValidationCheck:
        """Check cross-component consistency."""
        try:
            from orchestration.verification.system.consistency_validator import ConsistencyValidator

            validator = ConsistencyValidator(self.project_root)

            components_dir = self.project_root / "components"
            if not components_dir.exists():
                return ValidationCheck(
                    check_name="Cross-Component Consistency",
                    passed=True,
                    details="No components to check",
                    critical=False
                )

            total_violations = 0

            for comp_dir in components_dir.iterdir():
                if comp_dir.is_dir() and not comp_dir.name.startswith('.'):
                    violations = validator.validate_component(comp_dir)
                    total_violations += len(violations)

            passed = total_violations == 0

            return ValidationCheck(
                check_name="Cross-Component Consistency",
                passed=passed,
                details=f"{total_violations} consistency violations",
                critical=False  # Warning, not critical
            )
        except Exception as e:
            return ValidationCheck(
                check_name="Cross-Component Consistency",
                passed=False,
                details=f"Error checking consistency: {e}",
                critical=False
            )

    def validate_semantic_correctness(self) -> ValidationCheck:
        """Check semantic correctness."""
        try:
            from orchestration.verification.system.semantic_verifier import SemanticVerifier

            verifier = SemanticVerifier(self.project_root)

            components_dir = self.project_root / "components"
            if not components_dir.exists():
                return ValidationCheck(
                    check_name="Semantic Correctness",
                    passed=True,
                    details="No components to verify",
                    critical=False
                )

            total_issues = 0
            critical_issues = 0

            for comp_dir in components_dir.iterdir():
                if comp_dir.is_dir() and not comp_dir.name.startswith('.'):
                    result = verifier.verify_component(comp_dir)
                    total_issues += len(result.issues)
                    critical_issues += len([i for i in result.issues if i.severity == "critical"])

            passed = critical_issues == 0

            return ValidationCheck(
                check_name="Semantic Correctness",
                passed=passed,
                details=f"{total_issues} issues ({critical_issues} critical)",
                critical=True
            )
        except Exception as e:
            return ValidationCheck(
                check_name="Semantic Correctness",
                passed=False,
                details=f"Error checking semantic correctness: {e}",
                critical=True
            )

    def validate_no_integration_failures(self) -> ValidationCheck:
        """Check for predicted integration failures."""
        try:
            from orchestration.verification.contracts.integration_predictor import IntegrationPredictor

            predictor = IntegrationPredictor(self.project_root)
            prediction = predictor.predict_integration_failures()

            critical_failures = len([f for f in prediction.predicted_failures
                                    if f.severity == "critical"])
            total_failures = len(prediction.predicted_failures)

            passed = critical_failures == 0

            return ValidationCheck(
                check_name="Integration Failure Prediction",
                passed=passed,
                details=f"{total_failures} predicted failures ({critical_failures} critical)",
                critical=True
            )
        except ImportError:
            # Integration predictor not implemented yet - mark as non-critical pass
            return ValidationCheck(
                check_name="Integration Failure Prediction",
                passed=True,
                details="Integration predictor not available (optional check)",
                critical=False
            )
        except Exception as e:
            return ValidationCheck(
                check_name="Integration Failure Prediction",
                passed=False,
                details=f"Error predicting integration failures: {e}",
                critical=True
            )

    def _generate_summary(self, ready: bool, passed: int, failed: int, critical: int) -> str:
        """Generate deployment readiness summary."""
        if ready:
            return f"✅ SYSTEM READY FOR DEPLOYMENT ({passed}/{passed+failed} checks passed)"
        else:
            return f"❌ SYSTEM NOT READY ({critical} critical failures, {failed} total failures)"

    def generate_deployment_readiness_report(self, readiness: DeploymentReadiness) -> str:
        """Generate formatted report."""
        report = []
        report.append("="*70)
        report.append("DEPLOYMENT READINESS REPORT")
        report.append("="*70)
        report.append("")
        report.append(readiness.summary)
        report.append("")
        report.append(f"Checks Passed: {readiness.checks_passed}")
        report.append(f"Checks Failed: {readiness.checks_failed}")
        report.append(f"Critical Failures: {readiness.critical_failures}")
        report.append("")
        report.append("Detailed Results:")
        report.append("")

        for check in readiness.checks:
            status = "✅" if check.passed else "❌"
            critical_marker = " [CRITICAL]" if check.critical and not check.passed else ""
            report.append(f"{status} {check.check_name}{critical_marker}")
            report.append(f"   {check.details}")
            report.append("")

        report.append("="*70)

        if readiness.ready_for_deployment:
            report.append("System is ready for deployment.")
        else:
            report.append("System is NOT ready for deployment.")
            report.append("Fix critical failures before deploying.")

        report.append("="*70)

        return "\n".join(report)


def main():
    """CLI interface."""
    import sys

    validator = SystemValidator(Path.cwd())
    readiness = validator.validate_system()

    print(validator.generate_deployment_readiness_report(readiness))

    sys.exit(0 if readiness.ready_for_deployment else 1)


if __name__ == '__main__':
    main()
