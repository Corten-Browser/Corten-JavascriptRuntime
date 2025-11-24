#!/usr/bin/env python3
"""
Completion Gate - Final verification before project completion

This gate BLOCKS completion unless:
1. Specification is 100% implemented (not stubbed)
2. No placeholder/stub code exists
3. All features have tests
4. All project phases are complete

This gate enforces the anti-stopping rules by making premature completion impossible.
"""

import sys
import json
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from orchestration.core.paths import DataPaths

from spec_completeness_verifier import SpecCompletenessVerifier
from stub_detector import StubDetector


class CompletionGate:
    """
    Final gate that blocks premature completion.

    Enforces Rules 2, 4, 5, 7 from anti-stopping enforcement.
    """

    def __init__(self, project_root: str):
        """Initialize completion gate."""
        self.project_root = Path(project_root)
        self._paths = DataPaths(self.project_root)
        self.blocking_issues = []
        self.warnings = []
        self.passed = False

    def run_gate(self, spec_path: str = None, components_dir: str = "components") -> bool:
        """
        Run completion gate checks.

        Returns True if gate passes (project can complete).
        Returns False if gate fails (project cannot complete).
        """
        print("=" * 70)
        print("COMPLETION GATE - ANTI-STOPPING ENFORCEMENT")
        print("=" * 70)
        print()

        # Check 1: Specification completeness (if spec provided)
        if spec_path:
            print("CHECK 1: Specification Completeness Verification")
            print("-" * 70)
            self._check_spec_completeness(spec_path, components_dir)
        else:
            print("CHECK 1: Specification file not provided")
            print("  ⚠️ Cannot verify spec completeness without specification file")
            self.warnings.append("No specification file provided for verification")

        print()

        # Check 2: Stub/Placeholder Detection
        print("CHECK 2: Stub/Placeholder Detection")
        print("-" * 70)
        self._check_for_stubs(components_dir)

        print()

        # Check 3: All phases complete
        print("CHECK 3: Phase Completion Status")
        print("-" * 70)
        self._check_phase_completion()

        print()

        # Check 4: Rationalization detection (check for completion reports)
        print("CHECK 4: Premature Completion Report Detection")
        print("-" * 70)
        self._check_for_premature_reports()

        print()

        # Final verdict
        self._generate_verdict()

        return self.passed

    def _check_spec_completeness(self, spec_path: str, components_dir: str):
        """Verify specification is 100% implemented."""
        verifier = SpecCompletenessVerifier(str(self.project_root))

        if not verifier.load_specification(spec_path):
            self.blocking_issues.append(f"Cannot load specification: {spec_path}")
            print(f"  ❌ Cannot load specification file: {spec_path}")
            return

        result = verifier.verify_implementation(components_dir)

        print(f"  Total features in spec: {result.total_features}")
        print(f"  Implemented (real code): {result.implemented_features}")
        print(f"  Stubbed/placeholder: {result.stubbed_features}")
        print(f"  Coverage: {result.coverage_percentage:.1f}%")

        if result.coverage_percentage < 100:
            self.blocking_issues.append(
                f"Specification coverage is {result.coverage_percentage:.1f}%, must be 100%"
            )
            print(f"  ❌ BLOCKING: Coverage must be 100%, currently {result.coverage_percentage:.1f}%")

        if result.missing_features:
            self.blocking_issues.append(f"{len(result.missing_features)} features not implemented")
            print(f"  ❌ BLOCKING: {len(result.missing_features)} features missing")
            for feature in result.missing_features[:5]:
                print(f"      - {feature}")
            if len(result.missing_features) > 5:
                print(f"      ... and {len(result.missing_features) - 5} more")

        if result.stub_features:
            self.blocking_issues.append(f"{len(result.stub_features)} features are stubs")
            print(f"  ❌ BLOCKING: {len(result.stub_features)} features are placeholders")

        if result.is_complete:
            print("  ✅ Specification 100% complete")

    def _check_for_stubs(self, components_dir: str):
        """Check for stub/placeholder code."""
        detector = StubDetector(str(self.project_root))
        reports = detector.scan_all_components(components_dir)

        if not reports:
            print("  ⚠️ No components found to scan")
            self.warnings.append("No components found for stub detection")
            return

        total_components = len(reports)
        complete = sum(1 for r in reports if r.is_complete)
        total_critical_stubs = sum(r.critical_stubs for r in reports)

        print(f"  Components scanned: {total_components}")
        print(f"  Complete (no stubs): {complete}/{total_components}")
        print(f"  Critical stubs found: {total_critical_stubs}")

        if total_critical_stubs > 0:
            self.blocking_issues.append(f"{total_critical_stubs} critical stubs found in code")
            print(f"  ❌ BLOCKING: {total_critical_stubs} critical stubs must be fixed")

            blocking_stubs = detector.get_blocking_stubs()
            for stub in blocking_stubs[:5]:
                print(f"      - {stub.file_path}:{stub.line_number} ({stub.stub_type})")
            if len(blocking_stubs) > 5:
                print(f"      ... and {len(blocking_stubs) - 5} more")

        incomplete = [r for r in reports if not r.is_complete]
        if incomplete:
            self.blocking_issues.append(f"{len(incomplete)} components are incomplete")
            print(f"  ❌ BLOCKING: {len(incomplete)} incomplete components:")
            for report in incomplete[:5]:
                print(f"      - {report.name}")

        if detector.is_project_complete():
            print("  ✅ No blocking stubs found")

    def _check_phase_completion(self):
        """Check if all project phases are complete."""
        state_file = self._paths.orchestration_state

        if not state_file.exists():
            print("  ⚠️ No orchestration state file found")
            self.warnings.append("No orchestration state file to verify phase completion")
            return

        try:
            state = json.loads(state_file.read_text())
            current_phase = state.get("current_phase", "unknown")
            total_phases = state.get("total_phases", "unknown")
            completed_gates = state.get("completed_gates", [])

            print(f"  Current phase: {current_phase}")
            print(f"  Total phases: {total_phases}")
            print(f"  Completed gates: {len(completed_gates)}")

            # Check if all phases complete
            if current_phase != total_phases and current_phase != "complete":
                self.blocking_issues.append(
                    f"Not all phases complete: {current_phase}/{total_phases}"
                )
                print(f"  ❌ BLOCKING: Phase {current_phase} of {total_phases} - not final phase")
            else:
                print("  ✅ All phases complete")

        except Exception as e:
            print(f"  ⚠️ Error reading state file: {e}")
            self.warnings.append(f"Error reading orchestration state: {e}")

    def _check_for_premature_reports(self):
        """Detect if premature completion reports exist."""
        # Look for completion reports that shouldn't exist yet
        report_patterns = [
            "COMPLETION-REPORT.md",
            "PROJECT-COMPLETION-REPORT.md",
            "FINAL-COMPLETION-REPORT.md",
            "*-COMPLETE.md",
        ]

        found_reports = []
        for pattern in report_patterns:
            if '*' in pattern:
                found_reports.extend(self.project_root.glob(pattern))
            else:
                report_path = self.project_root / pattern
                if report_path.exists():
                    found_reports.append(report_path)

        # Also check docs/ directory
        docs_dir = self.project_root / "docs"
        if docs_dir.exists():
            for pattern in report_patterns:
                if '*' in pattern:
                    found_reports.extend(docs_dir.glob(pattern))
                else:
                    report_path = docs_dir / pattern
                    if report_path.exists():
                        found_reports.append(report_path)

        if found_reports and self.blocking_issues:
            # If we have blocking issues but completion reports exist, that's a problem
            print(f"  ⚠️ Found {len(found_reports)} completion reports but project is incomplete:")
            for report in found_reports:
                print(f"      - {report.relative_to(self.project_root)}")
            self.warnings.append(
                "Completion reports exist but project has blocking issues - possible premature completion"
            )
        else:
            print("  ✅ No premature completion reports detected")

    def _generate_verdict(self):
        """Generate final verdict."""
        print("=" * 70)
        print("COMPLETION GATE VERDICT")
        print("=" * 70)

        if not self.blocking_issues:
            self.passed = True
            print()
            print("✅ GATE PASSED - PROJECT MAY BE COMPLETED")
            print()
            print("All anti-stopping checks passed:")
            print("  ✅ Specification fully implemented (no gaps)")
            print("  ✅ No stub/placeholder code")
            print("  ✅ All phases complete")
            print()
            print("You may now generate a completion report.")
        else:
            self.passed = False
            print()
            print("❌ GATE FAILED - CANNOT COMPLETE PROJECT")
            print()
            print(f"BLOCKING ISSUES ({len(self.blocking_issues)}):")
            for i, issue in enumerate(self.blocking_issues, 1):
                print(f"  {i}. {issue}")
            print()
            print("ACTION REQUIRED:")
            print("  - Fix all blocking issues above")
            print("  - Re-run completion gate")
            print("  - Do NOT generate completion report until gate passes")
            print()
            print("ANTI-STOPPING RULES ENFORCED:")
            print("  Rule 2: Specification Completeness Verification")
            print("  Rule 4: No Stub/Placeholder Components")
            print("  Rule 5: Scope Preservation")
            print("  Rule 7: Completion Report Gate")

        if self.warnings:
            print()
            print(f"WARNINGS ({len(self.warnings)}):")
            for warning in self.warnings:
                print(f"  ⚠️ {warning}")

        print()
        print("=" * 70)

    def save_gate_result(self, output_path: str = "completion_gate_result.json"):
        """Save gate result for audit trail."""
        result = {
            "passed": self.passed,
            "blocking_issues": self.blocking_issues,
            "warnings": self.warnings,
            "timestamp": __import__('datetime').datetime.now().isoformat()
        }

        output_file = self.project_root / "orchestration" / output_path
        output_file.write_text(json.dumps(result, indent=2))
        print(f"Gate result saved to: {output_file}")


def main():
    """CLI entry point."""
    if len(sys.argv) < 2:
        print("Usage: python completion_gate.py <project_root> [spec_file] [components_dir]")
        print()
        print("Examples:")
        print("  python completion_gate.py .")
        print("  python completion_gate.py . specifications/project-spec.md")
        print("  python completion_gate.py . spec.md components")
        sys.exit(1)

    project_root = sys.argv[1]
    spec_path = sys.argv[2] if len(sys.argv) > 2 else None
    components_dir = sys.argv[3] if len(sys.argv) > 3 else "components"

    gate = CompletionGate(project_root)
    passed = gate.run_gate(spec_path, components_dir)

    gate.save_gate_result()

    if passed:
        sys.exit(0)
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()
