"""
Specification Verifier

Verifies that implemented components match the project specification.

This tool prevents architectural drift by ensuring the implementation matches
the specified architecture, preventing issues like implementing microservices
when a monolithic architecture was specified.

Classes:
    VerificationResult: Result of a single verification check
    SpecificationVerification: Complete verification report
    SpecificationVerifier: Main verifier class

Functions:
    verify_implementation: Verify implementation against specification

Usage:
    verifier = SpecificationVerifier(project_root)
    result = verifier.verify_against_specification("project-spec.md")

    if result.passed:
        print("✅ Implementation matches specification")
    else:
        print("❌ Verification failed")
        for failure in result.failures:
            print(f"  - {failure.message}")
"""

from dataclasses import dataclass, field
from typing import List, Dict, Set, Optional
from pathlib import Path
from enum import Enum

from specification_analyzer import (
    SpecificationAnalyzer,
    SpecificationAnalysis,
    ArchitectureType,
    ComponentType
)
from dependency_manager import DependencyManager


class VerificationLevel(Enum):
    """Severity levels for verification results."""
    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


@dataclass
class VerificationResult:
    """Result of a single verification check."""
    check_name: str
    passed: bool
    level: VerificationLevel
    message: str
    details: Optional[str] = None
    suggestion: Optional[str] = None


@dataclass
class SpecificationVerification:
    """Complete specification verification report."""
    passed: bool
    spec_path: str
    project_root: str
    checks_total: int
    checks_passed: int
    checks_failed: int
    checks_warning: int

    results: List[VerificationResult] = field(default_factory=list)
    missing_components: List[str] = field(default_factory=list)
    extra_components: List[str] = field(default_factory=list)
    architecture_mismatch: Optional[str] = None
    dependency_mismatches: List[str] = field(default_factory=list)

    @property
    def failures(self) -> List[VerificationResult]:
        """Get only failed checks."""
        return [r for r in self.results if not r.passed]

    @property
    def warnings(self) -> List[VerificationResult]:
        """Get only warning checks."""
        return [r for r in self.results if r.level == VerificationLevel.WARNING]

    @property
    def errors(self) -> List[VerificationResult]:
        """Get only error checks."""
        return [r for r in self.results if r.level == VerificationLevel.ERROR and not r.passed]


class SpecificationVerifier:
    """
    Verify that implemented components match the project specification.

    This verifier ensures architectural alignment between specification
    and implementation, preventing misinterpretation of requirements.
    """

    def __init__(self, project_root: Path):
        """
        Initialize the specification verifier.

        Args:
            project_root: Path to project root directory
        """
        self.project_root = Path(project_root)
        self.analyzer = SpecificationAnalyzer()
        self.dep_manager = DependencyManager(project_root)

    def verify_against_specification(
        self,
        spec_path: Path
    ) -> SpecificationVerification:
        """
        Verify implementation against specification.

        Args:
            spec_path: Path to specification file or directory

        Returns:
            SpecificationVerification report
        """
        spec_path = Path(spec_path)

        # Analyze specification
        spec_analysis = self.analyzer.analyze_specification(spec_path)

        # Load actual implementation
        self.dep_manager.load_all_manifests()

        # Run verification checks
        results = []

        # Check 1: Architecture pattern match
        arch_results = self._verify_architecture(spec_analysis)
        results.extend(arch_results)

        # Check 2: Component presence
        component_results = self._verify_components(spec_analysis)
        results.extend(component_results)

        # Check 3: Component types
        type_results = self._verify_component_types(spec_analysis)
        results.extend(type_results)

        # Check 4: Dependencies
        dep_results = self._verify_dependencies(spec_analysis)
        results.extend(dep_results)

        # Check 5: Integration style
        integration_results = self._verify_integration_style(spec_analysis)
        results.extend(integration_results)

        # Check 6: Tech stack
        tech_results = self._verify_tech_stack(spec_analysis)
        results.extend(tech_results)

        # Calculate statistics
        checks_passed = sum(1 for r in results if r.passed)
        checks_failed = sum(1 for r in results if not r.passed and r.level == VerificationLevel.ERROR)
        checks_warning = sum(1 for r in results if r.level == VerificationLevel.WARNING)

        # Determine overall pass/fail
        passed = checks_failed == 0

        # Extract detailed issues
        missing_components = [
            comp.name for comp in spec_analysis.suggested_components
            if comp.name not in self.dep_manager.components
        ]

        extra_components = [
            name for name in self.dep_manager.components
            if not any(comp.name == name for comp in spec_analysis.suggested_components)
        ]

        architecture_mismatch = None
        for result in results:
            if "architecture" in result.check_name.lower() and not result.passed:
                architecture_mismatch = result.message

        dependency_mismatches = [
            result.message for result in results
            if "dependency" in result.check_name.lower() and not result.passed
        ]

        return SpecificationVerification(
            passed=passed,
            spec_path=str(spec_path),
            project_root=str(self.project_root),
            checks_total=len(results),
            checks_passed=checks_passed,
            checks_failed=checks_failed,
            checks_warning=checks_warning,
            results=results,
            missing_components=missing_components,
            extra_components=extra_components,
            architecture_mismatch=architecture_mismatch,
            dependency_mismatches=dependency_mismatches
        )

    def _verify_architecture(
        self,
        spec_analysis: SpecificationAnalysis
    ) -> List[VerificationResult]:
        """Verify architecture pattern matches specification."""
        results = []

        arch = spec_analysis.architecture

        # Check: Integration style should match spec
        if arch.uses_direct_imports:
            # Specification calls for direct imports - check components can import each other
            has_import_deps = any(
                len(deps) > 0
                for deps in self.dep_manager.dependencies.values()
            )

            if has_import_deps:
                results.append(VerificationResult(
                    check_name="Architecture: Direct imports",
                    passed=True,
                    level=VerificationLevel.INFO,
                    message="✅ Components use direct imports as specified",
                    suggestion=None
                ))
            else:
                results.append(VerificationResult(
                    check_name="Architecture: Direct imports",
                    passed=False,
                    level=VerificationLevel.WARNING,
                    message="⚠️ Specification calls for direct imports, but no component dependencies found",
                    suggestion="Add component dependencies using imports in component.yaml"
                ))

        if arch.is_integrated:
            # Check that components are designed to work together
            total_deps = sum(len(deps) for deps in self.dep_manager.dependencies.values())

            if total_deps > 0:
                results.append(VerificationResult(
                    check_name="Architecture: Integrated system",
                    passed=True,
                    level=VerificationLevel.INFO,
                    message=f"✅ Components are integrated ({total_deps} dependencies)",
                    suggestion=None
                ))
            else:
                results.append(VerificationResult(
                    check_name="Architecture: Integrated system",
                    passed=False,
                    level=VerificationLevel.ERROR,
                    message="❌ Specification describes integrated system, but components have no dependencies",
                    details="Integrated architecture requires components to import and use each other",
                    suggestion="Add component dependencies - components should import from each other's public APIs"
                ))

        if arch.is_layered:
            # Check layer hierarchy
            violations = self.dep_manager.validate_dependency_levels()

            if not violations:
                results.append(VerificationResult(
                    check_name="Architecture: Layer hierarchy",
                    passed=True,
                    level=VerificationLevel.INFO,
                    message="✅ Component layers respect hierarchy",
                    suggestion=None
                ))
            else:
                results.append(VerificationResult(
                    check_name="Architecture: Layer hierarchy",
                    passed=False,
                    level=VerificationLevel.ERROR,
                    message=f"❌ Layer hierarchy violations: {len(violations)}",
                    details="\n".join(violations[:3]),
                    suggestion="Ensure components only depend on same or lower level components"
                ))

        return results

    def _verify_components(
        self,
        spec_analysis: SpecificationAnalysis
    ) -> List[VerificationResult]:
        """Verify all specified components are implemented."""
        results = []

        spec_components = {comp.name for comp in spec_analysis.suggested_components}
        impl_components = set(self.dep_manager.components.keys())

        missing = spec_components - impl_components
        extra = impl_components - spec_components

        # Check: All specified components implemented
        if not missing:
            results.append(VerificationResult(
                check_name="Components: All specified components present",
                passed=True,
                level=VerificationLevel.INFO,
                message=f"✅ All {len(spec_components)} specified components are implemented",
                suggestion=None
            ))
        else:
            results.append(VerificationResult(
                check_name="Components: All specified components present",
                passed=False,
                level=VerificationLevel.ERROR,
                message=f"❌ Missing {len(missing)} components from specification",
                details=f"Missing: {', '.join(sorted(missing))}",
                suggestion="Implement missing components or update specification"
            ))

        # Check: No unexpected components (warning only)
        if not extra:
            results.append(VerificationResult(
                check_name="Components: No extra components",
                passed=True,
                level=VerificationLevel.INFO,
                message="✅ No unexpected components",
                suggestion=None
            ))
        else:
            results.append(VerificationResult(
                check_name="Components: No extra components",
                passed=True,  # Warning, not failure
                level=VerificationLevel.WARNING,
                message=f"⚠️ Found {len(extra)} components not in specification",
                details=f"Extra: {', '.join(sorted(extra))}",
                suggestion="Update specification to document these components"
            ))

        return results

    def _verify_component_types(
        self,
        spec_analysis: SpecificationAnalysis
    ) -> List[VerificationResult]:
        """Verify component types match specification."""
        results = []

        type_mismatches = []

        for spec_comp in spec_analysis.suggested_components:
            if spec_comp.name in self.dep_manager.components:
                impl_type = self.dep_manager.get_component_type(spec_comp.name)
                spec_type = spec_comp.type.value

                if impl_type != spec_type and spec_type != "unknown":
                    type_mismatches.append(
                        f"{spec_comp.name}: spec says '{spec_type}', implemented as '{impl_type}'"
                    )

        if not type_mismatches:
            results.append(VerificationResult(
                check_name="Component types: Match specification",
                passed=True,
                level=VerificationLevel.INFO,
                message="✅ Component types match specification",
                suggestion=None
            ))
        else:
            results.append(VerificationResult(
                check_name="Component types: Match specification",
                passed=False,
                level=VerificationLevel.WARNING,
                message=f"⚠️ Component type mismatches: {len(type_mismatches)}",
                details="\n".join(type_mismatches[:3]),
                suggestion="Update component.yaml type field to match specification"
            ))

        return results

    def _verify_dependencies(
        self,
        spec_analysis: SpecificationAnalysis
    ) -> List[VerificationResult]:
        """Verify dependencies match specification."""
        results = []

        # Check for circular dependencies
        cycles = self.dep_manager.check_circular_dependencies()

        if not cycles:
            results.append(VerificationResult(
                check_name="Dependencies: No circular dependencies",
                passed=True,
                level=VerificationLevel.INFO,
                message="✅ No circular dependencies",
                suggestion=None
            ))
        else:
            results.append(VerificationResult(
                check_name="Dependencies: No circular dependencies",
                passed=False,
                level=VerificationLevel.ERROR,
                message=f"❌ Circular dependencies detected: {len(cycles)} cycles",
                details="\n".join(f"Cycle: {' → '.join(cycle)}" for cycle in cycles[:3]),
                suggestion="Break circular dependencies by refactoring or introducing interfaces"
            ))

        # Check: All dependencies exist
        missing_deps = []
        for component in self.dep_manager.components:
            missing = self.dep_manager.verify_dependencies(component)
            if missing:
                missing_deps.extend([(component, dep) for dep in missing])

        if not missing_deps:
            results.append(VerificationResult(
                check_name="Dependencies: All dependencies exist",
                passed=True,
                level=VerificationLevel.INFO,
                message="✅ All component dependencies exist",
                suggestion=None
            ))
        else:
            results.append(VerificationResult(
                check_name="Dependencies: All dependencies exist",
                passed=False,
                level=VerificationLevel.ERROR,
                message=f"❌ Missing dependencies: {len(missing_deps)}",
                details="\n".join(f"{comp} requires {dep}" for comp, dep in missing_deps[:3]),
                suggestion="Implement missing dependencies or update component.yaml"
            ))

        return results

    def _verify_integration_style(
        self,
        spec_analysis: SpecificationAnalysis
    ) -> List[VerificationResult]:
        """Verify integration style matches specification."""
        results = []

        integration_style = spec_analysis.integration_style

        if integration_style == "library_imports":
            # Check components have import dependencies
            has_imports = any(
                len(deps) > 0 and not all(d.optional for d in deps)
                for deps in self.dep_manager.dependencies.values()
            )

            if has_imports:
                results.append(VerificationResult(
                    check_name="Integration: Library imports",
                    passed=True,
                    level=VerificationLevel.INFO,
                    message="✅ Components use library-style imports as specified",
                    suggestion=None
                ))
            else:
                results.append(VerificationResult(
                    check_name="Integration: Library imports",
                    passed=False,
                    level=VerificationLevel.WARNING,
                    message="⚠️ Specification calls for library imports, but components don't import each other",
                    suggestion="Add import dependencies in component.yaml files"
                ))

        elif integration_style == "rest_apis":
            # This is informational - actual REST API verification requires runtime checks
            results.append(VerificationResult(
                check_name="Integration: REST APIs",
                passed=True,
                level=VerificationLevel.INFO,
                message="ℹ️ Specification calls for REST APIs (runtime verification required)",
                suggestion="Ensure API contracts are defined in contracts/ directory"
            ))

        return results

    def _verify_tech_stack(
        self,
        spec_analysis: SpecificationAnalysis
    ) -> List[VerificationResult]:
        """Verify tech stack matches specification (informational)."""
        results = []

        if spec_analysis.tech_stack:
            results.append(VerificationResult(
                check_name="Tech stack: Specified technologies",
                passed=True,
                level=VerificationLevel.INFO,
                message=f"ℹ️ Spec specifies: {', '.join(sorted(spec_analysis.tech_stack))}",
                suggestion="Ensure components use specified technologies"
            ))

        return results

    def generate_report(
        self,
        verification: SpecificationVerification,
        output_path: Optional[Path] = None
    ) -> str:
        """
        Generate human-readable verification report.

        Args:
            verification: Verification result
            output_path: Optional path to write report to

        Returns:
            Report as string
        """
        lines = []

        lines.append("=" * 70)
        lines.append("SPECIFICATION VERIFICATION REPORT")
        lines.append("=" * 70)
        lines.append("")

        # Overall status
        if verification.passed:
            lines.append("✅ VERIFICATION PASSED")
        else:
            lines.append("❌ VERIFICATION FAILED")
        lines.append("")

        # Summary
        lines.append(f"Specification: {verification.spec_path}")
        lines.append(f"Project Root: {verification.project_root}")
        lines.append(f"Total Checks: {verification.checks_total}")
        lines.append(f"Passed: {verification.checks_passed}")
        lines.append(f"Failed: {verification.checks_failed}")
        lines.append(f"Warnings: {verification.checks_warning}")
        lines.append("")

        # Errors
        if verification.errors:
            lines.append("ERRORS:")
            for result in verification.errors:
                lines.append(f"  {result.message}")
                if result.details:
                    for detail_line in result.details.split('\n'):
                        lines.append(f"    {detail_line}")
                if result.suggestion:
                    lines.append(f"    💡 {result.suggestion}")
            lines.append("")

        # Warnings
        if verification.warnings:
            lines.append("WARNINGS:")
            for result in verification.warnings:
                lines.append(f"  {result.message}")
                if result.details:
                    for detail_line in result.details.split('\n'):
                        lines.append(f"    {detail_line}")
                if result.suggestion:
                    lines.append(f"    💡 {result.suggestion}")
            lines.append("")

        # Missing components
        if verification.missing_components:
            lines.append("MISSING COMPONENTS:")
            for comp in verification.missing_components:
                lines.append(f"  - {comp}")
            lines.append("")

        # Extra components
        if verification.extra_components:
            lines.append("EXTRA COMPONENTS (not in spec):")
            for comp in verification.extra_components:
                lines.append(f"  - {comp}")
            lines.append("")

        # All checks
        lines.append("ALL CHECKS:")
        for result in verification.results:
            status = "✅" if result.passed else "❌"
            lines.append(f"  {status} {result.check_name}")
        lines.append("")

        lines.append("=" * 70)

        report = "\n".join(lines)

        # Write to file if requested
        if output_path:
            output_path = Path(output_path)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            with open(output_path, 'w') as f:
                f.write(report)

        return report


def verify_implementation(
    project_root: Path,
    spec_path: Path,
    output_path: Optional[Path] = None
) -> SpecificationVerification:
    """
    Convenience function to verify implementation against specification.

    Args:
        project_root: Path to project root directory
        spec_path: Path to specification file or directory
        output_path: Optional path to write report to

    Returns:
        SpecificationVerification result
    """
    verifier = SpecificationVerifier(project_root)
    verification = verifier.verify_against_specification(spec_path)

    if output_path:
        verifier.generate_report(verification, output_path)

    return verification


def main():
    """CLI entry point for specification verifier."""
    import sys
    import argparse

    parser = argparse.ArgumentParser(
        description="Verify implementation matches specification"
    )
    parser.add_argument(
        'project_root',
        type=Path,
        help='Path to project root directory'
    )
    parser.add_argument(
        'spec_path',
        type=Path,
        help='Path to specification file or directory'
    )
    parser.add_argument(
        '--output',
        type=Path,
        help='Output path for report'
    )

    args = parser.parse_args()

    if not args.project_root.exists():
        print(f"Error: Project root not found: {args.project_root}")
        return 1

    if not args.spec_path.exists():
        print(f"Error: Specification not found: {args.spec_path}")
        return 1

    # Verify implementation
    verifier = SpecificationVerifier(args.project_root)
    verification = verifier.verify_against_specification(args.spec_path)

    # Generate and print report
    report = verifier.generate_report(verification, args.output)
    print(report)

    # Exit with error code if verification failed
    return 0 if verification.passed else 1


if __name__ == '__main__':
    import sys
    sys.exit(main())
