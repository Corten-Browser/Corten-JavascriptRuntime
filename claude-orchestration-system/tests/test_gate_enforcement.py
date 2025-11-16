#!/usr/bin/env python3
"""
Tests for Gate Enforcement System (v0.17.0)

Tests three layers of enforcement:
1. State Manager (orchestration_state.py)
2. Enforced Wrapper (orchestrate_enforced.py)
3. Report Generator (generate_completion_report.py)

Historical Context:
These tests verify the system prevents the "Looks Good But Breaks" pattern
that caused Music Analyzer v1-v3 failures.
"""

import json
import pytest
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

# Add orchestration directory to path
sys.path.insert(0, str(Path(__file__).parent.parent / "orchestration"))

from orchestration_state import StateManager, OrchestrationState, PhaseGateResult
from orchestrate_enforced import OrchestrationEnforcer, GateEnforcementError
from generate_completion_report import ReportGenerator


class TestStateManager:
    """Tests for orchestration_state.py"""

    def test_create_new_state(self, tmp_path):
        """
        Given: Empty project directory
        When: StateManager initialized
        Then: Creates new state with defaults
        """
        manager = StateManager(tmp_path)

        assert manager.state.current_phase == 1
        assert manager.state.phase_gates == {}
        assert manager.state.gate_history == []
        assert manager.state.orchestration_version == "0.17.0"

    def test_save_and_load_state(self, tmp_path):
        """
        Given: State with gate executions
        When: Save and reload
        Then: State persists correctly
        """
        manager = StateManager(tmp_path)

        # Record a gate execution
        manager.record_gate_execution(
            phase=5,
            passed=True,
            exit_code=0,
            duration_seconds=10.5,
            output="Gate passed successfully",
            save_full_output=False
        )

        # Save
        manager.save_state()

        # Reload
        manager2 = StateManager(tmp_path)

        assert manager2.state.current_phase == 6  # Advanced after Phase 5 passed
        assert 5 in manager2.state.phase_gates
        assert manager2.state.phase_gates[5].passed is True
        assert len(manager2.state.gate_history) == 1

    def test_record_gate_execution_passed(self, tmp_path):
        """
        Given: Phase 5 completed
        When: Gate execution passes
        Then: State updated correctly, current phase advances
        """
        manager = StateManager(tmp_path)

        result = manager.record_gate_execution(
            phase=5,
            passed=True,
            exit_code=0,
            duration_seconds=12.4,
            output="All integration tests passed\n✅ Phase 5 gate PASSED",
            save_full_output=True
        )

        assert result.phase == 5
        assert result.passed is True
        assert result.exit_code == 0
        assert result.duration_seconds == 12.4

        # State should advance to Phase 6
        assert manager.state.current_phase == 6

        # Gate should be recorded
        assert 5 in manager.state.phase_gates
        assert manager.state.phase_gates[5].passed is True

        # History should have one entry
        assert len(manager.state.gate_history) == 1

    def test_record_gate_execution_failed(self, tmp_path):
        """
        Given: Phase 5 completed
        When: Gate execution fails
        Then: State updated but phase does NOT advance
        """
        manager = StateManager(tmp_path)

        result = manager.record_gate_execution(
            phase=5,
            passed=False,
            exit_code=1,
            duration_seconds=8.2,
            output="Integration tests failing (10/12 passed)\n❌ Phase 5 gate FAILED",
            save_full_output=True
        )

        assert result.passed is False
        assert result.exit_code == 1

        # State should NOT advance (still Phase 1)
        assert manager.state.current_phase == 1

        # Gate should be recorded as failed
        assert manager.state.phase_gates[5].passed is False

    def test_can_proceed_to_phase_allowed(self, tmp_path):
        """
        Given: Phase 5 gate passed
        When: Check if can proceed to Phase 6
        Then: Returns True with reason
        """
        manager = StateManager(tmp_path)

        # Pass Phase 5 gate
        manager.record_gate_execution(
            phase=5,
            passed=True,
            exit_code=0,
            duration_seconds=10.0,
            output="Gate passed",
            save_full_output=False
        )

        allowed, reason = manager.can_proceed_to_phase(6)

        assert allowed is True
        assert "Phase 5 gate passed" in reason

    def test_can_proceed_to_phase_blocked_gate_not_run(self, tmp_path):
        """
        Given: Phase 5 gate NOT run
        When: Check if can proceed to Phase 6
        Then: Returns False (blocked)
        """
        manager = StateManager(tmp_path)

        allowed, reason = manager.can_proceed_to_phase(6)

        assert allowed is False
        assert "has not been run" in reason

    def test_can_proceed_to_phase_blocked_gate_failed(self, tmp_path):
        """
        Given: Phase 5 gate failed
        When: Check if can proceed to Phase 6
        Then: Returns False (blocked)
        """
        manager = StateManager(tmp_path)

        # Fail Phase 5 gate
        manager.record_gate_execution(
            phase=5,
            passed=False,
            exit_code=1,
            duration_seconds=5.0,
            output="Gate failed",
            save_full_output=False
        )

        allowed, reason = manager.can_proceed_to_phase(6)

        assert allowed is False
        assert "failed" in reason.lower()

    def test_get_gate_history(self, tmp_path):
        """
        Given: Multiple gate executions
        When: Get gate history
        Then: Returns chronological list
        """
        manager = StateManager(tmp_path)

        # Record multiple executions
        manager.record_gate_execution(5, True, 0, 10.0, "First", False)
        manager.record_gate_execution(5, False, 1, 8.0, "Second (retry, failed)", False)
        manager.record_gate_execution(5, True, 0, 11.0, "Third (retry, passed)", False)

        history = manager.get_gate_history(phase=5)

        assert len(history) == 3
        assert history[0].output_summary == "First"
        assert history[1].passed is False
        assert history[2].passed is True

    def test_gate_output_saved_to_file(self, tmp_path):
        """
        Given: Gate execution with save_full_output=True
        When: Record gate
        Then: Output saved to file
        """
        manager = StateManager(tmp_path)

        output_text = "This is the full gate output\n" * 50  # Long output

        result = manager.record_gate_execution(
            phase=5,
            passed=True,
            exit_code=0,
            duration_seconds=10.0,
            output=output_text,
            save_full_output=True
        )

        assert result.full_output_file is not None

        # Verify file exists
        output_file = tmp_path / result.full_output_file
        assert output_file.exists()

        # Verify content
        with open(output_file, 'r') as f:
            saved_output = f.read()

        assert saved_output == output_text


class TestOrchestrationEnforcer:
    """Tests for orchestrate_enforced.py"""

    def test_can_proceed_to_phase_delegates_to_state_manager(self, tmp_path):
        """
        Given: Enforcer with state
        When: Check can_proceed_to_phase
        Then: Delegates to state manager correctly
        """
        enforcer = OrchestrationEnforcer(tmp_path)

        # Pass Phase 5 gate
        enforcer.state_manager.record_gate_execution(
            5, True, 0, 10.0, "Passed", False
        )

        allowed, reason = enforcer.can_proceed_to_phase(6)

        assert allowed is True

    def test_verify_all_gates_all_passed(self, tmp_path):
        """
        Given: Phase 5 and 6 gates passed
        When: Verify all gates
        Then: Returns True
        """
        enforcer = OrchestrationEnforcer(tmp_path)

        # Pass both blocking gates
        enforcer.state_manager.record_gate_execution(5, True, 0, 10.0, "P5 passed", False)
        enforcer.state_manager.record_gate_execution(6, True, 0, 15.0, "P6 passed", False)

        all_passed, summary = enforcer.verify_all_gates()

        assert all_passed is True
        assert "all" in summary.lower() and "passed" in summary.lower()

    def test_verify_all_gates_phase_5_not_run(self, tmp_path):
        """
        Given: Phase 5 gate NOT run
        When: Verify all gates
        Then: Returns False
        """
        enforcer = OrchestrationEnforcer(tmp_path)

        # Only Phase 6 passed (missing Phase 5)
        enforcer.state_manager.record_gate_execution(6, True, 0, 15.0, "P6 passed", False)

        all_passed, summary = enforcer.verify_all_gates()

        assert all_passed is False

    def test_verify_all_gates_phase_6_failed(self, tmp_path):
        """
        Given: Phase 6 gate failed
        When: Verify all gates
        Then: Returns False
        """
        enforcer = OrchestrationEnforcer(tmp_path)

        # Phase 5 passed, Phase 6 failed
        enforcer.state_manager.record_gate_execution(5, True, 0, 10.0, "P5 passed", False)
        enforcer.state_manager.record_gate_execution(6, False, 1, 15.0, "P6 failed", False)

        all_passed, summary = enforcer.verify_all_gates()

        assert all_passed is False

    def test_get_gate_output_path(self, tmp_path):
        """
        Given: Gate with saved output
        When: Get gate output path
        Then: Returns correct path
        """
        enforcer = OrchestrationEnforcer(tmp_path)

        # Record gate with output
        enforcer.state_manager.record_gate_execution(
            5, True, 0, 10.0, "Output text", save_full_output=True
        )

        output_path = enforcer.get_gate_output_path(5)

        assert output_path is not None
        assert output_path.exists()


class TestReportGenerator:
    """Tests for generate_completion_report.py"""

    def test_generate_report_no_gates_run(self, tmp_path):
        """
        Given: No gates executed
        When: Generate report
        Then: Report contains missing evidence warnings
        """
        generator = ReportGenerator(tmp_path)

        report = generator.generate_report(project_name="Test Project")

        assert "Test Project" in report
        assert "⚠️⚠️⚠️ MISSING EVIDENCE ⚠️⚠️⚠️" in report
        assert "Phase 5 gate has NOT been executed" in report

    def test_generate_report_phase_5_passed(self, tmp_path):
        """
        Given: Phase 5 gate passed
        When: Generate report
        Then: Report includes Phase 5 gate data
        """
        generator = ReportGenerator(tmp_path)

        # Pass Phase 5 gate
        generator.state_manager.record_gate_execution(
            phase=5,
            passed=True,
            exit_code=0,
            duration_seconds=12.4,
            output="✅ All integration tests passing (12/12)\n✅ PHASE 5 GATE PASSED",
            save_full_output=True
        )

        report = generator.generate_report()

        assert "✅ PASSED" in report
        assert "12.4s" in report
        assert "GATE OUTPUT LOADED" in report

    def test_generate_report_phase_6_failed(self, tmp_path):
        """
        Given: Phase 6 gate failed
        When: Generate report
        Then: Report includes failure warning
        """
        generator = ReportGenerator(tmp_path)

        # Fail Phase 6 gate
        generator.state_manager.record_gate_execution(
            phase=6,
            passed=False,
            exit_code=1,
            duration_seconds=8.5,
            output="❌ Check #10 failed\n❌ PHASE 6 GATE FAILED",
            save_full_output=True
        )

        report = generator.generate_report()

        assert "❌ FAILED" in report
        assert "WARNING: Phase 6 gate FAILED" in report

    def test_validate_report_valid(self, tmp_path):
        """
        Given: Report with all evidence present
        When: Validate report
        Then: Returns valid=True
        """
        generator = ReportGenerator(tmp_path)

        # Pass all gates
        generator.state_manager.record_gate_execution(5, True, 0, 10.0, "P5 passed", False)
        generator.state_manager.record_gate_execution(6, True, 0, 15.0, "P6 passed", False)

        # Generate report
        report = generator.generate_report()
        report_path = tmp_path / "test-report.md"

        # Remove placeholders (simulate manual completion)
        report = report.replace("[PASTE PRIMARY CLI COMMAND OUTPUT HERE]", "$ myapp --help\nUsage: myapp ...")
        report = report.replace("[PASTE OUTPUT OF: python orchestration/orchestrate_enforced.py verify-gates]", "✅ All gates passed")

        with open(report_path, 'w') as f:
            f.write(report)

        valid, issues = generator.validate_report(report_path)

        assert valid is True
        assert len(issues) == 0

    def test_validate_report_missing_evidence(self, tmp_path):
        """
        Given: Report with missing evidence markers
        When: Validate report
        Then: Returns valid=False with issues
        """
        generator = ReportGenerator(tmp_path)

        # Create incomplete report
        report_path = tmp_path / "incomplete-report.md"
        with open(report_path, 'w') as f:
            f.write("# Completion Report\n")
            f.write("[PASTE PHASE 5 GATE OUTPUT HERE - MANDATORY]\n")
            f.write("[PASTE PRIMARY CLI COMMAND OUTPUT HERE]\n")

        valid, issues = generator.validate_report(report_path)

        assert valid is False
        assert len(issues) > 0
        assert any("PHASE 5" in issue for issue in issues)

    def test_validate_report_gates_not_passed(self, tmp_path):
        """
        Given: Report generated but Phase 5 gate failed
        When: Validate report
        Then: Returns valid=False (gates must pass)
        """
        generator = ReportGenerator(tmp_path)

        # Fail Phase 5 gate
        generator.state_manager.record_gate_execution(5, False, 1, 10.0, "Failed", False)

        report = generator.generate_report()
        report_path = tmp_path / "report.md"

        with open(report_path, 'w') as f:
            f.write(report)

        valid, issues = generator.validate_report(report_path)

        assert valid is False
        assert any("Phase 5 gate not passed" in issue for issue in issues)


class TestIntegration:
    """Integration tests for complete workflow"""

    def test_full_workflow_happy_path(self, tmp_path):
        """
        Given: Clean project
        When: Execute full workflow (Phase 5 → Phase 6 → Report)
        Then: All steps succeed with evidence
        """
        # Create enforcer and state manager
        enforcer = OrchestrationEnforcer(tmp_path)

        # 1. Pass Phase 5 gate
        enforcer.state_manager.record_gate_execution(
            5, True, 0, 10.0, "Phase 5 passed", save_full_output=True
        )

        # 2. Verify can proceed to Phase 6
        allowed, _ = enforcer.can_proceed_to_phase(6)
        assert allowed is True

        # 3. Pass Phase 6 gate
        enforcer.state_manager.record_gate_execution(
            6, True, 0, 15.0, "Phase 6 passed", save_full_output=True
        )

        # 4. Verify all gates
        all_passed, _ = enforcer.verify_all_gates()
        assert all_passed is True

        # 5. Generate report
        generator = ReportGenerator(tmp_path)
        report = generator.generate_report()

        # 6. Validate report
        report_path = tmp_path / "COMPLETION-REPORT.md"

        # Simulate manual evidence addition
        report = report.replace(
            "[PASTE PRIMARY CLI COMMAND OUTPUT HERE]",
            "$ python -m myapp --help\nUsage: myapp [options]\n✅ Command works"
        )
        report = report.replace(
            "[PASTE OUTPUT OF: python orchestration/orchestrate_enforced.py verify-gates]",
            "✅ All blocking gates passed"
        )

        with open(report_path, 'w') as f:
            f.write(report)

        valid, issues = generator.validate_report(report_path)

        assert valid is True
        assert len(issues) == 0

    def test_full_workflow_blocked_by_phase_5(self, tmp_path):
        """
        Given: Phase 5 gate fails
        When: Try to proceed to Phase 6
        Then: Blocked with clear reason
        """
        enforcer = OrchestrationEnforcer(tmp_path)

        # Fail Phase 5 gate
        enforcer.state_manager.record_gate_execution(
            5, False, 1, 10.0, "Integration tests failing", save_full_output=True
        )

        # Cannot proceed to Phase 6
        allowed, reason = enforcer.can_proceed_to_phase(6)

        assert allowed is False
        assert "failed" in reason.lower()

        # Verify all gates shows failure
        all_passed, _ = enforcer.verify_all_gates()
        assert all_passed is False

        # Report shows invalid
        generator = ReportGenerator(tmp_path)
        report = generator.generate_report()

        assert "⚠️ WARNING: Phase 5 gate FAILED" in report


@pytest.fixture
def tmp_path(tmp_path_factory):
    """Create temporary directory for tests."""
    return tmp_path_factory.mktemp("test_orchestration")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
