"""
Display formatting for resume status and confirmation.
"""

from typing import Dict, Any
from datetime import datetime


class ResumeDisplay:
    """Format resume status for user display."""

    @staticmethod
    def format_checkpoint_status(checkpoint: Dict[str, Any]) -> str:
        """Format checkpoint data for display."""
        output = []

        output.append("📋 Orchestration Resume Analysis")
        output.append("")

        # Original request
        req = checkpoint["original_request"]
        output.append(f"Original Request: {req['user_prompt']}")
        output.append(f"Started: {ResumeDisplay._format_timestamp(req['timestamp'])}")

        # Current status
        current_phase = checkpoint["phase_progress"]["current_phase"]
        phases = checkpoint["phase_progress"]["phases"]

        if current_phase > 0:
            phase = phases[current_phase - 1]
            output.append(f"Last Activity: Phase {current_phase} - {phase['name']}")
            output.append("")

        # Phase status
        output.append("Phase Status:")
        for phase in phases:
            num = phase["phase_number"]
            name = phase["name"]
            status = phase["status"]

            if status == "completed":
                icon = "✅"
                detail = ""
                if phase.get("duration_minutes"):
                    detail = f" ({phase['duration_minutes']} min)"
            elif status == "in_progress":
                icon = "🔄"
                detail = " (in progress)"
            elif status == "incomplete":
                icon = "🔴"
                detail = " (incomplete)"
            else:
                icon = "⏸️"
                detail = " (not started)"

            output.append(f"{icon} Phase {num}: {name}{detail}")

            # Show test results if available
            if status in ["completed", "incomplete"] and "test_results" in phase:
                tests = phase["test_results"]
                if "pass_rate_percent" in tests:
                    output.append(f"   Tests: {tests['passed']}/{tests['total']} passing ({tests['pass_rate_percent']}%)")

            # Show gate status
            if "gate_status" in phase:
                gate = phase["gate_status"]
                if gate == "FAILED":
                    output.append(f"   ❌ Phase gate: {phase.get('blocking_reason', 'FAILED')}")
                elif gate == "PASSED":
                    output.append(f"   ✅ Phase gate: PASSED")

        output.append("")

        # Blocking issues
        stopping = checkpoint.get("stopping_context", {})
        if stopping.get("reason"):
            output.append("Blocking Issue:")
            output.append(f"  Reason: {stopping['reason']}")
            output.append(f"  Details: {stopping['details']}")
            output.append("")

        return "\n".join(output)

    @staticmethod
    def format_discovery_status(discovery: Dict[str, Any]) -> str:
        """Format state discovery data for display."""
        output = []

        output.append("📋 State Discovery Results")
        output.append("")
        output.append(f"Discovery Method: {discovery['discovery_method']}")
        output.append(f"Confidence: {discovery['confidence'].upper()}")
        output.append("")

        # Components found
        state = discovery["discovered_state"]
        components = state.get("components_found", [])
        output.append(f"Components Found: {len(components)}")
        if components:
            for comp in components[:5]:  # Show first 5
                output.append(f"  - {comp}")
            if len(components) > 5:
                output.append(f"  ... and {len(components) - 5} more")
        output.append("")

        # Test status
        if "tests_run" in state:
            tests = state["tests_run"]
            output.append("Test Status:")
            if "unit_tests" in tests:
                ut = tests["unit_tests"]
                output.append(f"  Unit Tests: {ut.get('passed', 0)}/{ut.get('total', 0)} passing")
            if "integration_tests" in tests:
                it = tests["integration_tests"]
                output.append(f"  Integration Tests: {it.get('passed', 0)}/{it.get('total', 0)} passing")
            output.append("")

        # Phase gates
        if "phase_gates" in state:
            gates = state["phase_gates"]
            output.append("Phase Gate Status:")
            for phase_num in range(1, 7):
                gate_key = f"phase_{phase_num}"
                if gate_key in gates:
                    status = gates[gate_key]
                    icon = "✅" if status == "passed" else "❌"
                    output.append(f"  {icon} Phase {phase_num}: {status}")
            output.append("")

        # Inferred context
        inferred = discovery.get("inferred_context", {})
        if inferred:
            output.append("Inferred Context:")
            if "likely_current_phase" in inferred:
                output.append(f"  Current Phase: {inferred['likely_current_phase']}")
            if "estimated_progress_percent" in inferred:
                output.append(f"  Progress: ~{inferred['estimated_progress_percent']}%")
            output.append("")

        # Missing information warning
        missing = discovery.get("missing_information", [])
        if missing:
            output.append("⚠️ Missing Information:")
            for item in missing:
                output.append(f"  - {item}")
            output.append("")

        return "\n".join(output)

    @staticmethod
    def format_resume_plan(
        current_phase: int,
        phases: list,
        blocking_issues: list = None
    ) -> str:
        """Format the resume execution plan."""
        output = []

        output.append("Resume Plan:")
        output.append("")

        # If blocking issues, show them first
        if blocking_issues:
            output.append("1. Address blocking issues:")
            for issue in blocking_issues:
                output.append(f"   - {issue}")
            step_num = 2
        else:
            step_num = 1

        # Show remaining phases
        for phase in phases[current_phase - 1:]:
            output.append(f"{step_num}. Phase {phase['phase_number']}: {phase['name']}")
            if phase.get("status") == "incomplete":
                output.append("   (Fix incomplete work)")
            step_num += 1

        output.append("")
        output.append(f"{step_num}. Run completion verifier (13/13 checks)")
        output.append(f"{step_num + 1}. Generate final documentation")
        output.append("")

        return "\n".join(output)

    @staticmethod
    def _format_timestamp(ts_str: str) -> str:
        """Format ISO timestamp for display."""
        try:
            dt = datetime.fromisoformat(ts_str.replace("Z", ""))
            return dt.strftime("%Y-%m-%d %H:%M")
        except:
            return ts_str
