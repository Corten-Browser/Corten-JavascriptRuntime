#!/usr/bin/env python3
"""
Orchestration State Manager (v0.17.0)

Tracks phase gate execution and progression state to enforce mandatory gate execution.
This prevents the "Looks Good But Breaks" pattern where orchestrators skip verification.

Historical Context:
- Music Analyzer v1: Skipped contract validation, wrong method names
- Music Analyzer v2: Skipped Phase 5 gate, 83.3% test pass rate declared "complete"
- Music Analyzer v3: Skipped Phase 6 UAT, __main__.py in wrong location

Solution: Track all gate executions with timestamps, block progression without gates.
"""

import json
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional


@dataclass
class PhaseGateResult:
    """Result of a phase gate execution."""

    phase: int
    passed: bool
    timestamp: str
    exit_code: int
    duration_seconds: float
    output_summary: str  # First 500 chars of output
    full_output_file: Optional[str] = None  # Path to full output if saved

    def to_dict(self) -> Dict:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict) -> 'PhaseGateResult':
        """Create from dictionary."""
        return cls(**data)


@dataclass
class OrchestrationState:
    """Complete orchestration state including all gate executions."""

    project_root: str
    current_phase: int
    phase_gates: Dict[int, PhaseGateResult]  # phase -> most recent result
    gate_history: List[PhaseGateResult]  # All gate executions (chronological)
    started_at: str
    last_updated: str
    orchestration_version: str = "0.17.0"

    def to_dict(self) -> Dict:
        """Convert to dictionary for JSON serialization."""
        return {
            'project_root': self.project_root,
            'current_phase': self.current_phase,
            'phase_gates': {
                str(phase): result.to_dict()
                for phase, result in self.phase_gates.items()
            },
            'gate_history': [result.to_dict() for result in self.gate_history],
            'started_at': self.started_at,
            'last_updated': self.last_updated,
            'orchestration_version': self.orchestration_version
        }

    @classmethod
    def from_dict(cls, data: Dict) -> 'OrchestrationState':
        """Create from dictionary."""
        return cls(
            project_root=data['project_root'],
            current_phase=data['current_phase'],
            phase_gates={
                int(phase): PhaseGateResult.from_dict(result)
                for phase, result in data['phase_gates'].items()
            },
            gate_history=[
                PhaseGateResult.from_dict(result)
                for result in data['gate_history']
            ],
            started_at=data['started_at'],
            last_updated=data['last_updated'],
            orchestration_version=data.get('orchestration_version', '0.17.0')
        )


class StateManager:
    """
    Manages orchestration state persistence and validation.

    Responsibilities:
    - Load/save state to orchestration-state.json
    - Record gate executions with timestamps
    - Enforce phase progression rules (cannot skip gates)
    - Provide gate execution history
    """

    STATE_FILE = "orchestration-state.json"

    def __init__(self, project_root: Path):
        """
        Initialize state manager.

        Args:
            project_root: Project root directory
        """
        self.project_root = project_root
        self.state_file = project_root / self.STATE_FILE
        self.state = self._load_or_create_state()

    def _load_or_create_state(self) -> OrchestrationState:
        """Load existing state or create new one."""
        if self.state_file.exists():
            try:
                with open(self.state_file, 'r') as f:
                    data = json.load(f)
                return OrchestrationState.from_dict(data)
            except (json.JSONDecodeError, KeyError) as e:
                print(f"Warning: Invalid state file, creating new: {e}")
                return self._create_new_state()
        else:
            return self._create_new_state()

    def _create_new_state(self) -> OrchestrationState:
        """Create new orchestration state."""
        now = datetime.now().isoformat()
        return OrchestrationState(
            project_root=str(self.project_root),
            current_phase=1,
            phase_gates={},
            gate_history=[],
            started_at=now,
            last_updated=now
        )

    def save_state(self) -> None:
        """Save current state to disk."""
        self.state.last_updated = datetime.now().isoformat()
        with open(self.state_file, 'w') as f:
            json.dump(self.state.to_dict(), f, indent=2)

    def record_gate_execution(
        self,
        phase: int,
        passed: bool,
        exit_code: int,
        duration_seconds: float,
        output: str,
        save_full_output: bool = True
    ) -> PhaseGateResult:
        """
        Record a gate execution result.

        Args:
            phase: Phase number (1-6)
            passed: Whether gate passed
            exit_code: Gate script exit code
            duration_seconds: How long gate took
            output: Gate output text
            save_full_output: Whether to save full output to file

        Returns:
            PhaseGateResult object
        """
        timestamp = datetime.now().isoformat()
        output_summary = output[:500]

        # Save full output if requested
        full_output_file = None
        if save_full_output:
            output_dir = self.project_root / "orchestration" / "gate_outputs"
            output_dir.mkdir(exist_ok=True)

            timestamp_str = datetime.now().strftime("%Y%m%d_%H%M%S")
            output_filename = f"phase_{phase}_gate_{timestamp_str}.txt"
            full_output_path = output_dir / output_filename

            with open(full_output_path, 'w') as f:
                f.write(output)

            full_output_file = str(full_output_path.relative_to(self.project_root))

        # Create result
        result = PhaseGateResult(
            phase=phase,
            passed=passed,
            timestamp=timestamp,
            exit_code=exit_code,
            duration_seconds=duration_seconds,
            output_summary=output_summary,
            full_output_file=full_output_file
        )

        # Update state
        self.state.phase_gates[phase] = result
        self.state.gate_history.append(result)

        # Update current phase if gate passed
        if passed and phase >= self.state.current_phase:
            self.state.current_phase = phase + 1

        self.save_state()
        return result

    def get_phase_status(self, phase: int) -> Optional[PhaseGateResult]:
        """
        Get most recent gate result for a phase.

        Args:
            phase: Phase number (1-6)

        Returns:
            PhaseGateResult if gate has been run, None otherwise
        """
        return self.state.phase_gates.get(phase)

    def can_proceed_to_phase(self, target_phase: int) -> tuple[bool, str]:
        """
        Check if progression to target phase is allowed.

        Args:
            target_phase: Phase number to proceed to

        Returns:
            (allowed, reason) tuple
                allowed: True if can proceed, False otherwise
                reason: Explanation of decision
        """
        # Phase 1 is always allowed (starting phase)
        if target_phase == 1:
            return True, "Phase 1 is the starting phase"

        # Check if previous phase gate passed
        previous_phase = target_phase - 1
        previous_result = self.get_phase_status(previous_phase)

        if previous_result is None:
            return False, f"Phase {previous_phase} gate has not been run yet"

        if not previous_result.passed:
            return False, f"Phase {previous_phase} gate failed (must pass before proceeding)"

        return True, f"Phase {previous_phase} gate passed, may proceed"

    def get_gate_history(
        self,
        phase: Optional[int] = None,
        only_passed: bool = False
    ) -> List[PhaseGateResult]:
        """
        Get gate execution history.

        Args:
            phase: Filter by phase number (None = all phases)
            only_passed: Only return passed gates

        Returns:
            List of PhaseGateResult objects (chronological)
        """
        history = self.state.gate_history

        if phase is not None:
            history = [r for r in history if r.phase == phase]

        if only_passed:
            history = [r for r in history if r.passed]

        return history

    def get_status_report(self) -> str:
        """
        Generate human-readable status report.

        Returns:
            Multi-line status report string
        """
        lines = []
        lines.append("=" * 60)
        lines.append("Orchestration State Report")
        lines.append("=" * 60)
        lines.append(f"Project Root: {self.state.project_root}")
        lines.append(f"Current Phase: {self.state.current_phase}")
        lines.append(f"Started: {self.state.started_at}")
        lines.append(f"Last Updated: {self.state.last_updated}")
        lines.append(f"Version: {self.state.orchestration_version}")
        lines.append("")
        lines.append("Phase Gate Status:")
        lines.append("-" * 60)

        for phase in range(1, 7):
            result = self.get_phase_status(phase)
            if result is None:
                lines.append(f"  Phase {phase}: NOT RUN")
            else:
                status = "✅ PASSED" if result.passed else "❌ FAILED"
                lines.append(f"  Phase {phase}: {status} (exit {result.exit_code})")
                lines.append(f"             at {result.timestamp}")
                lines.append(f"             duration: {result.duration_seconds:.1f}s")

        lines.append("")
        lines.append(f"Total Gate Executions: {len(self.state.gate_history)}")
        lines.append("=" * 60)

        return "\n".join(lines)

    def reset_state(self) -> None:
        """Reset state to initial (use with caution)."""
        self.state = self._create_new_state()
        self.save_state()


def main():
    """CLI interface for state management."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python orchestration_state.py <command> [args]")
        print("Commands:")
        print("  status              - Show current state")
        print("  reset               - Reset state (requires confirmation)")
        print("  history [phase]     - Show gate execution history")
        sys.exit(1)

    command = sys.argv[1]
    project_root = Path.cwd()
    manager = StateManager(project_root)

    if command == "status":
        print(manager.get_status_report())

    elif command == "reset":
        print("⚠️  WARNING: This will reset all orchestration state!")
        print("Continue? (yes/no): ", end="")
        response = input().strip().lower()
        if response == "yes":
            manager.reset_state()
            print("✅ State reset successfully")
        else:
            print("Cancelled")

    elif command == "history":
        phase = int(sys.argv[2]) if len(sys.argv) > 2 else None
        history = manager.get_gate_history(phase=phase)

        if not history:
            print("No gate executions recorded")
        else:
            for i, result in enumerate(history, 1):
                status = "✅ PASSED" if result.passed else "❌ FAILED"
                print(f"{i}. Phase {result.phase}: {status}")
                print(f"   Timestamp: {result.timestamp}")
                print(f"   Duration: {result.duration_seconds:.1f}s")
                print(f"   Exit Code: {result.exit_code}")
                if result.full_output_file:
                    print(f"   Full Output: {result.full_output_file}")
                print()

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == "__main__":
    main()
