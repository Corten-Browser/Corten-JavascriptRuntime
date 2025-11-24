#!/usr/bin/env python3
"""
Anti-Stopping Enforcer

Master coordinator for all anti-stopping mechanisms.
Integrates: spec verification, stub detection, completion gates, session continuation.

This is the programmatic enforcement of the 7 Anti-Stopping Rules.
"""

import json
import sys
from pathlib import Path
from datetime import datetime

# Import enforcement tools
sys.path.insert(0, str(Path(__file__).parent))
from spec_completeness_verifier import SpecCompletenessVerifier
from stub_detector import StubDetector
from session_continuation import SessionContinuationManager

sys.path.insert(0, str(Path(__file__).parent / "phase_gates"))
from completion_gate import CompletionGate


class AntiStoppingEnforcer:
    """
    Enforces all anti-stopping rules programmatically.

    This class is the last line of defense against premature completion.
    """

    def __init__(self, project_root: str):
        """Initialize enforcer with project root."""
        self.project_root = Path(project_root)
        self.enforcement_results = {}
        self.blocking_issues = []
        self.warnings = []

    def pre_orchestration_check(self, spec_path: str) -> bool:
        """
        Run checks BEFORE orchestration begins.

        Ensures specification is loaded and understood.
        Returns True if orchestration can proceed.
        """
        print("=" * 70)
        print("PRE-ORCHESTRATION CHECK")
        print("Anti-Stopping Enforcement Active")
        print("=" * 70)
        print()

        # Verify specification exists
        spec_file = Path(spec_path)
        if not spec_file.is_absolute():
            spec_file = self.project_root / spec_path

        if not spec_file.exists():
            print(f"❌ Specification file not found: {spec_path}")
            self.blocking_issues.append(f"Specification not found: {spec_path}")
            return False

        print(f"✅ Specification loaded: {spec_file}")

        # Extract feature count
        verifier = SpecCompletenessVerifier(str(self.project_root))
        if verifier.load_specification(str(spec_file)):
            feature_count = len(verifier.features)
            print(f"✅ Features extracted from spec: {feature_count}")

            if feature_count == 0:
                print("⚠️ No features detected - manual verification required")
                self.warnings.append("No features auto-detected from specification")

            # Save feature checklist for tracking
            verifier.save_checklist("orchestration/spec_features_checklist.json")
            print(f"✅ Feature checklist saved for tracking")
        else:
            print("❌ Could not parse specification")
            self.blocking_issues.append("Specification parsing failed")
            return False

        print()
        print("ANTI-STOPPING RULES LOADED:")
        print("  Rule 1: No human time estimates")
        print("  Rule 2: Specification completeness verification")
        print("  Rule 3: Absolute phase continuity")
        print("  Rule 4: No stub/placeholder components")
        print("  Rule 5: Scope preservation")
        print("  Rule 6: Anti-rationalization detection")
        print("  Rule 7: Completion report gate")
        print()
        print("✅ Pre-orchestration check PASSED")
        print("=" * 70)

        return True

    def mid_orchestration_check(self, current_phase: int, total_phases: int) -> dict:
        """
        Run checks DURING orchestration.

        Returns status dict with any blocking issues.
        """
        result = {
            "passed": True,
            "current_phase": current_phase,
            "total_phases": total_phases,
            "issues": [],
            "recommendations": []
        }

        # Check if trying to stop at phase boundary
        if current_phase < total_phases:
            result["recommendations"].append(
                f"Phase {current_phase} complete. Rule 3 requires: Proceed to Phase {current_phase + 1}"
            )

        # Run stub detector
        detector = StubDetector(str(self.project_root))
        detector.scan_all_components()
        blocking_stubs = detector.get_blocking_stubs()

        if blocking_stubs:
            result["passed"] = False
            result["issues"].append(f"{len(blocking_stubs)} critical stubs found - Rule 4 violated")

        # Check for premature completion attempts
        if current_phase < total_phases:
            # Look for completion report files
            completion_indicators = [
                self.project_root / "COMPLETION-REPORT.md",
                self.project_root / "docs" / "COMPLETION-REPORT.md",
                self.project_root / "PROJECT-COMPLETE.md",
            ]

            for indicator in completion_indicators:
                if indicator.exists():
                    result["passed"] = False
                    result["issues"].append(
                        f"Premature completion report detected at Phase {current_phase}/{total_phases} - Rule 7 violated"
                    )
                    break

        return result

    def completion_check(self, spec_path: str, components_dir: str = "components") -> bool:
        """
        Final check before allowing completion.

        This is the ultimate gate - if this fails, project CANNOT be completed.
        """
        print()
        print("=" * 70)
        print("FINAL COMPLETION CHECK - ANTI-STOPPING ENFORCEMENT")
        print("=" * 70)
        print()

        gate = CompletionGate(str(self.project_root))
        passed = gate.run_gate(spec_path, components_dir)

        if not passed:
            self.blocking_issues.extend(gate.blocking_issues)
            print()
            print("❌ COMPLETION BLOCKED")
            print("You CANNOT generate a completion report until all issues are resolved.")
            print()
            print("REMEMBER THE ANTI-STOPPING RULES:")
            self._print_relevant_rules()

        return passed

    def _print_relevant_rules(self):
        """Print the anti-stopping rules as reminders."""
        print("""
<rule_no_rationalization>
IF you find yourself writing justifications for stopping:
🚨 STOP IMMEDIATELY
You are constructing rationalization for premature termination.
DELETE the justification and CONTINUE IMPLEMENTING instead.
</rule_no_rationalization>

<rule_completion_report_gate>
MANDATORY CHECKLIST (ALL must be YES):
1. Have you read the ENTIRE specification document? [ ]
2. Is EVERY feature from spec implemented (not stubbed)? [ ]
3. Are ALL project phases complete (not just Phase 1)? [ ]
4. Is test pass rate exactly 100%? [ ]
5. Are there ZERO placeholder/stub components? [ ]

If ANY is NO: DO NOT generate completion report
</rule_completion_report_gate>
""")

    def create_session_checkpoint(
        self,
        session_id: str,
        project_name: str,
        spec_path: str,
        current_phase: int,
        total_phases: int,
        features_completed: int,
        features_total: int,
        components_completed: list[str],
        components_remaining: list[str],
        tests_passing: int,
        tests_total: int,
        next_action: str
    ):
        """Create a checkpoint for session continuation."""
        manager = SessionContinuationManager(str(self.project_root))

        checkpoint_path = manager.create_checkpoint(
            session_id=session_id,
            project_name=project_name,
            specification_path=spec_path,
            current_phase=current_phase,
            total_phases=total_phases,
            features_total=features_total,
            features_completed=features_completed,
            components_total=len(components_completed) + len(components_remaining),
            components_completed=components_completed,
            components_in_progress=[],
            components_remaining=components_remaining,
            tests_passing=tests_passing,
            tests_total=tests_total,
            last_action=f"Completed phase {current_phase}",
            next_action=next_action,
            blocking_issues=self.blocking_issues
        )

        print(f"✅ Session checkpoint created for continuation")
        print(f"   If context limit approached, resume with:")
        print(f"   python orchestration/session_continuation.py resume")

    def generate_enforcement_report(self) -> str:
        """Generate comprehensive enforcement status report."""
        lines = [
            "=" * 70,
            "ANTI-STOPPING ENFORCEMENT REPORT",
            "=" * 70,
            f"Generated: {datetime.now().isoformat()}",
            "",
        ]

        if self.blocking_issues:
            lines.append(f"❌ BLOCKING ISSUES ({len(self.blocking_issues)}):")
            for issue in self.blocking_issues:
                lines.append(f"  - {issue}")
            lines.append("")
            lines.append("ACTION: Fix all blocking issues before proceeding")
        else:
            lines.append("✅ No blocking issues detected")

        if self.warnings:
            lines.append("")
            lines.append(f"⚠️ WARNINGS ({len(self.warnings)}):")
            for warning in self.warnings:
                lines.append(f"  - {warning}")

        lines.append("")
        lines.append("ENFORCEMENT RULES STATUS:")
        lines.append("  Rule 1 (No Time Estimates): Active - monitoring for violations")
        lines.append("  Rule 2 (Spec Verification): Programmatically enforced")
        lines.append("  Rule 3 (Phase Continuity): Monitored at phase boundaries")
        lines.append("  Rule 4 (No Stubs): Programmatically enforced via stub detector")
        lines.append("  Rule 5 (Scope Preservation): Enforced via spec completeness")
        lines.append("  Rule 6 (No Rationalization): Detected via pattern matching")
        lines.append("  Rule 7 (Completion Gate): Final gate blocks premature completion")
        lines.append("")
        lines.append("=" * 70)

        return '\n'.join(lines)


def main():
    """CLI entry point."""
    if len(sys.argv) < 3:
        print("Usage: python anti_stopping_enforcer.py <command> <project_root> [args]")
        print()
        print("Commands:")
        print("  pre <root> <spec>     - Run pre-orchestration check")
        print("  mid <root> <phase> <total>  - Run mid-orchestration check")
        print("  final <root> <spec>   - Run final completion check")
        print("  report <root>         - Generate enforcement report")
        print()
        print("Example:")
        print("  python anti_stopping_enforcer.py pre . specifications/project.md")
        print("  python anti_stopping_enforcer.py final . spec.md")
        sys.exit(1)

    command = sys.argv[1]
    project_root = sys.argv[2]

    enforcer = AntiStoppingEnforcer(project_root)

    if command == "pre":
        if len(sys.argv) < 4:
            print("Usage: pre <root> <spec_path>")
            sys.exit(1)
        spec_path = sys.argv[3]
        success = enforcer.pre_orchestration_check(spec_path)
        sys.exit(0 if success else 1)

    elif command == "mid":
        if len(sys.argv) < 5:
            print("Usage: mid <root> <current_phase> <total_phases>")
            sys.exit(1)
        current_phase = int(sys.argv[3])
        total_phases = int(sys.argv[4])
        result = enforcer.mid_orchestration_check(current_phase, total_phases)
        print(json.dumps(result, indent=2))
        sys.exit(0 if result["passed"] else 1)

    elif command == "final":
        if len(sys.argv) < 4:
            print("Usage: final <root> <spec_path>")
            sys.exit(1)
        spec_path = sys.argv[3]
        success = enforcer.completion_check(spec_path)
        sys.exit(0 if success else 1)

    elif command == "report":
        report = enforcer.generate_enforcement_report()
        print(report)

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == "__main__":
    main()
