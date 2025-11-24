#!/usr/bin/env python3
"""
Completion Report Generator (v0.17.0)

Automatically generates evidence-based completion reports by:
- Reading orchestration state (gate executions)
- Extracting gate outputs from files
- Pre-filling report with verified data
- Validating evidence completeness
- Flagging missing evidence

This prevents "completion reports without evidence" that allowed 3 Music Analyzer failures.

Usage:
    python orchestration/generate_completion_report.py
    python orchestration/generate_completion_report.py --output COMPLETION-REPORT.md
    python orchestration/generate_completion_report.py --validate existing-report.md
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Import state manager - use sys.path manipulation for flexibility
_this_dir = Path(__file__).parent
sys.path.insert(0, str(_this_dir.parent))
from checkpoints.orchestration_state import StateManager


class ReportGenerator:
    """
    Generates evidence-based completion reports.

    Key Features:
    - Auto-fills gate execution data from state
    - Extracts and includes full gate outputs
    - Validates evidence sections are complete
    - Flags missing evidence prominently
    """

    TEMPLATE_PATH = Path("orchestration/templates/COMPLETION-REPORT-TEMPLATE.md")

    def __init__(self, project_root: Path):
        """
        Initialize report generator.

        Args:
            project_root: Project root directory
        """
        self.project_root = project_root
        self.state_manager = StateManager(project_root)
        self.template = self._load_template()

    def _load_template(self) -> str:
        """Load completion report template."""
        template_path = self.project_root / self.TEMPLATE_PATH

        if not template_path.exists():
            raise FileNotFoundError(f"Template not found: {template_path}")

        with open(template_path, 'r') as f:
            return f.read()

    def generate_report(
        self,
        project_name: str = None,
        orchestrator_name: str = "Claude Code"
    ) -> str:
        """
        Generate completion report with auto-filled data.

        Args:
            project_name: Project name (defaults to directory name)
            orchestrator_name: Name/version of orchestrator

        Returns:
            Generated report markdown
        """
        if project_name is None:
            project_name = self.project_root.name

        report = self.template

        # Fill in basic project info
        report = report.replace("[Project Name]", project_name)
        report = report.replace("[YYYY-MM-DD]", datetime.now().strftime("%Y-%m-%d"))
        report = report.replace("[Your Name/Claude Version]", orchestrator_name)

        # Fill in phase gate data
        report = self._fill_phase_5_gate(report)
        report = self._fill_phase_6_gate(report)

        # Fill in state data
        report = self._fill_state_data(report)

        # Add validation warnings
        report = self._add_validation_warnings(report)

        return report

    def _fill_phase_5_gate(self, report: str) -> str:
        """Fill Phase 5 gate execution data."""
        phase_5 = self.state_manager.get_phase_status(5)

        if phase_5 is None:
            # Phase 5 gate not run
            gate_status = "⚠️ NOT RUN"
            evidence_block = self._create_missing_evidence_warning(
                "Phase 5 gate has NOT been executed",
                "python orchestration/phase_gates/gate_runner.py . 5"
            )
        elif phase_5.passed:
            # Phase 5 gate passed
            gate_status = "✅ PASSED"
            evidence_block = self._load_gate_output(phase_5)
        else:
            # Phase 5 gate failed
            gate_status = "❌ FAILED"
            evidence_block = self._load_gate_output(phase_5)
            evidence_block += "\n\n⚠️ WARNING: Phase 5 gate FAILED. Fix issues and re-run."

        # Replace placeholders
        report = report.replace("**Gate Status:** [✅ PASSED / ❌ FAILED / ⚠️ NOT RUN]", f"**Gate Status:** {gate_status}", 1)

        # Find Phase 5 evidence section and replace
        phase_5_marker = "[PASTE PHASE 5 GATE OUTPUT HERE - MANDATORY]"
        report = report.replace(phase_5_marker, evidence_block)

        # Fill timestamp if available
        if phase_5:
            report = report.replace(
                "**Gate Timestamp:** [ISO 8601 timestamp from actual execution]",
                f"**Gate Timestamp:** {phase_5.timestamp}",
                1
            )
            report = report.replace(
                "**Gate Duration:** [Seconds from actual execution]",
                f"**Gate Duration:** {phase_5.duration_seconds:.1f}s",
                1
            )
            if phase_5.full_output_file:
                report = report.replace(
                    "**Full Output File:** [Path to orchestration/gate_outputs/phase_5_gate_*.txt]",
                    f"**Full Output File:** {phase_5.full_output_file}",
                    1
                )

        return report

    def _fill_phase_6_gate(self, report: str) -> str:
        """Fill Phase 6 gate execution data."""
        phase_6 = self.state_manager.get_phase_status(6)

        if phase_6 is None:
            # Phase 6 gate not run
            gate_status = "⚠️ NOT RUN"
            evidence_block = self._create_missing_evidence_warning(
                "Phase 6 gate has NOT been executed",
                "python orchestration/phase_gates/gate_runner.py . 6"
            )
        elif phase_6.passed:
            # Phase 6 gate passed
            gate_status = "✅ PASSED"
            evidence_block = self._load_gate_output(phase_6)
        else:
            # Phase 6 gate failed
            gate_status = "❌ FAILED"
            evidence_block = self._load_gate_output(phase_6)
            evidence_block += "\n\n⚠️ WARNING: Phase 6 gate FAILED. Fix issues and re-run."

        # Replace placeholders (2nd occurrence for Phase 6)
        lines = report.split('\n')
        gate_status_count = 0
        for i, line in enumerate(lines):
            if "**Gate Status:** [✅ PASSED / ❌ FAILED / ⚠️ NOT RUN]" in line:
                gate_status_count += 1
                if gate_status_count == 2:  # Phase 6 is the 2nd occurrence
                    lines[i] = f"**Gate Status:** {gate_status}"
                    break
        report = '\n'.join(lines)

        # Find Phase 6 evidence section and replace
        phase_6_marker = "[PASTE PHASE 6 GATE OUTPUT HERE - MANDATORY]"
        report = report.replace(phase_6_marker, evidence_block)

        # Fill timestamp if available
        if phase_6:
            # For Phase 6, we need to replace the 2nd occurrence
            timestamp_marker = "**Gate Timestamp:** [ISO 8601 timestamp from actual execution]"
            first_pos = report.find(timestamp_marker)
            if first_pos != -1:
                second_pos = report.find(timestamp_marker, first_pos + 1)
                if second_pos != -1:
                    report = report[:second_pos] + f"**Gate Timestamp:** {phase_6.timestamp}" + report[second_pos + len(timestamp_marker):]

            duration_marker = "**Gate Duration:** [Seconds from actual execution]"
            first_pos = report.find(duration_marker)
            if first_pos != -1:
                second_pos = report.find(duration_marker, first_pos + 1)
                if second_pos != -1:
                    report = report[:second_pos] + f"**Gate Duration:** {phase_6.duration_seconds:.1f}s" + report[second_pos + len(duration_marker):]

            if phase_6.full_output_file:
                output_marker = "**Full Output File:** [Path to orchestration/gate_outputs/phase_6_gate_*.txt]"
                report = report.replace(output_marker, f"**Full Output File:** {phase_6.full_output_file}")

        return report

    def _load_gate_output(self, gate_result) -> str:
        """Load full gate output from file."""
        if gate_result.full_output_file:
            output_path = self.project_root / gate_result.full_output_file

            if output_path.exists():
                with open(output_path, 'r') as f:
                    output = f.read()

                return f"✅ GATE OUTPUT LOADED FROM: {gate_result.full_output_file}\n\n```\n{output}\n```"
            else:
                return f"⚠️ WARNING: Gate output file not found: {gate_result.full_output_file}\n\nSummary: {gate_result.output_summary}"
        else:
            return f"Gate Output (summary):\n```\n{gate_result.output_summary}\n```"

    def _create_missing_evidence_warning(self, message: str, required_command: str) -> str:
        """Create a warning block for missing evidence."""
        return f"""⚠️⚠️⚠️ MISSING EVIDENCE ⚠️⚠️⚠️

{message}

REQUIRED ACTION:
1. Run command: {required_command}
2. Gate must PASS (exit code 0)
3. Re-generate this report to auto-fill output

This report is INVALID without this evidence.

Historical lesson: Music Analyzer v1-v3 all had completion reports without evidence.
All three failed immediately on user commands. Don't repeat this mistake.
"""

    def _fill_state_data(self, report: str) -> str:
        """Fill orchestration state data."""
        state = self.state_manager.state

        # Create state summary
        state_summary = f"""Current Phase: {state.current_phase}
Started: {state.started_at}
Last Updated: {state.last_updated}
Version: {state.orchestration_version}

Phase Gate Status:
"""

        for phase in range(1, 7):
            result = self.state_manager.get_phase_status(phase)
            if result is None:
                state_summary += f"  Phase {phase}: NOT RUN\n"
            else:
                status = "✅ PASSED" if result.passed else "❌ FAILED"
                state_summary += f"  Phase {phase}: {status} (exit {result.exit_code}) at {result.timestamp}\n"

        state_summary += f"\nTotal Gate Executions: {len(state.gate_history)}"

        # Replace state placeholder
        state_marker = "[PASTE OUTPUT OF: python orchestration/orchestrate_enforced.py status]"
        report = report.replace(state_marker, f"```\n{state_summary}\n```")

        # Also include raw JSON
        json_marker = "[PASTE CONTENTS OF: orchestration-state.json]"
        state_json = json.dumps(state.to_dict(), indent=2)
        report = report.replace(json_marker, f"```json\n{state_json}\n```")

        return report

    def _add_validation_warnings(self, report: str) -> str:
        """Add validation warnings for missing evidence."""
        phase_5 = self.state_manager.get_phase_status(5)
        phase_6 = self.state_manager.get_phase_status(6)

        warnings = []

        if phase_5 is None:
            warnings.append("⚠️ Phase 5 gate NOT RUN - report INVALID")
        elif not phase_5.passed:
            warnings.append("⚠️ Phase 5 gate FAILED - project NOT complete")

        if phase_6 is None:
            warnings.append("⚠️ Phase 6 gate NOT RUN - report INVALID")
        elif not phase_6.passed:
            warnings.append("⚠️ Phase 6 gate FAILED - project NOT complete")

        if warnings:
            warning_block = "\n\n## ⚠️ VALIDATION WARNINGS ⚠️\n\n"
            warning_block += "\n".join(f"- {w}" for w in warnings)
            warning_block += "\n\n**This report is INVALID until all warnings are resolved.**\n"

            # Insert before "Orchestrator Declaration"
            marker = "## Orchestrator Declaration"
            report = report.replace(marker, warning_block + "\n" + marker)

        return report

    def validate_report(self, report_path: Path) -> Tuple[bool, List[str]]:
        """
        Validate an existing completion report for evidence completeness.

        Args:
            report_path: Path to existing report

        Returns:
            (valid, issues) tuple
                valid: True if report has all required evidence
                issues: List of validation issues found
        """
        if not report_path.exists():
            return False, [f"Report file not found: {report_path}"]

        with open(report_path, 'r') as f:
            content = f.read()

        issues = []

        # Check for missing evidence markers
        missing_markers = [
            "[PASTE PHASE 5 GATE OUTPUT HERE - MANDATORY]",
            "[PASTE PHASE 6 GATE OUTPUT HERE - MANDATORY]",
            "[PASTE PRIMARY CLI COMMAND OUTPUT HERE]",
            "[PASTE OUTPUT OF: python orchestration/orchestrate_enforced.py status]",
            "[PASTE OUTPUT OF: python orchestration/orchestrate_enforced.py verify-gates]"
        ]

        for marker in missing_markers:
            if marker in content:
                issues.append(f"Missing evidence: {marker}")

        # Check for warning indicators
        if "⚠️⚠️⚠️ MISSING EVIDENCE ⚠️⚠️⚠️" in content:
            issues.append("Report contains missing evidence warnings")

        # Check gate status
        phase_5 = self.state_manager.get_phase_status(5)
        phase_6 = self.state_manager.get_phase_status(6)

        if phase_5 is None or not phase_5.passed:
            issues.append("Phase 5 gate not passed")

        if phase_6 is None or not phase_6.passed:
            issues.append("Phase 6 gate not passed")

        valid = len(issues) == 0

        return valid, issues


def main():
    """CLI interface."""
    parser = argparse.ArgumentParser(
        description="Generate evidence-based completion reports"
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("COMPLETION-REPORT.md"),
        help="Output file path (default: COMPLETION-REPORT.md)"
    )
    parser.add_argument(
        "--validate",
        type=Path,
        help="Validate existing report instead of generating new one"
    )
    parser.add_argument(
        "--project-name",
        help="Project name (defaults to directory name)"
    )
    parser.add_argument(
        "--orchestrator",
        default="Claude Code",
        help="Orchestrator name/version"
    )

    args = parser.parse_args()

    project_root = Path.cwd()
    generator = ReportGenerator(project_root)

    if args.validate:
        # Validate existing report
        print("=" * 70)
        print("VALIDATING COMPLETION REPORT")
        print("=" * 70)
        print()

        valid, issues = generator.validate_report(args.validate)

        if valid:
            print("✅ REPORT IS VALID")
            print("✅ All required evidence present")
            sys.exit(0)
        else:
            print("❌ REPORT IS INVALID")
            print()
            print("Issues found:")
            for issue in issues:
                print(f"  - {issue}")
            print()
            print("Fix all issues and re-validate")
            sys.exit(1)

    else:
        # Generate new report
        print("=" * 70)
        print("GENERATING COMPLETION REPORT")
        print("=" * 70)
        print()

        report = generator.generate_report(
            project_name=args.project_name,
            orchestrator_name=args.orchestrator
        )

        # Write report
        with open(args.output, 'w') as f:
            f.write(report)

        print(f"✅ Report generated: {args.output}")
        print()

        # Validate generated report
        valid, issues = generator.validate_report(args.output)

        if valid:
            print("✅ Report is VALID (all evidence present)")
        else:
            print("⚠️ Report has MISSING EVIDENCE")
            print()
            print("Issues:")
            for issue in issues:
                print(f"  - {issue}")
            print()
            print("Complete missing sections before using this report")

        print()
        print("Next steps:")
        print("1. Review generated report")
        print("2. Fill in any manual sections (UAT command output, etc.)")
        print("3. Re-validate: python orchestration/generate_completion_report.py --validate", str(args.output))


if __name__ == "__main__":
    main()
