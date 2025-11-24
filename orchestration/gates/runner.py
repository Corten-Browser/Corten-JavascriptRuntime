#!/usr/bin/env python3
"""
Gate Runner

Orchestrates phase gate execution.
Ensures gates are run at proper phase transitions.
Manages state tracking and audit trail.

Part of v0.14.0: Programmatic Enforcement System
"""

from pathlib import Path
from typing import Dict, Optional
import subprocess
import sys
import json
from datetime import datetime

from orchestration.core.paths import DataPaths


class GateRunner:
    """Orchestrates phase gate execution."""

    GATES = {
        1: "phase_1_analysis.py",
        2: "phase_2_architecture.py",
        3: "phase_3_contracts.py",
        4: "phase_4_development.py",
        5: "phase_5_integration.py",
        6: "phase_6_verification.py",
    }

    PHASE_NAMES = {
        1: "Analysis",
        2: "Architecture",
        3: "Contracts",
        4: "Development",
        5: "Integration",
        6: "Verification",
    }

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.gate_dir = Path(__file__).parent
        self._paths = DataPaths(project_root)
        self.state_file = self._paths.orchestration_state

    def run_gate(self, phase: int) -> bool:
        """
        Run exit gate for specified phase.

        Args:
            phase: Phase number (1-6)

        Returns:
            True if gate passes, False otherwise
        """
        if phase not in self.GATES:
            print(f"❌ Invalid phase: {phase}")
            print(f"   Valid phases: 1-6")
            return False

        gate_script = self.gate_dir / self.GATES[phase]

        if not gate_script.exists():
            print(f"⚠️  Gate script not found: {gate_script}")
            print(f"   Skipping Phase {phase} gate validation")
            print(f"   Note: Only Phase 5 gate is currently implemented")
            # For unimplemented gates, return True (don't block)
            return True

        phase_name = self.PHASE_NAMES.get(phase, f"Phase {phase}")
        print(f"🔍 Running Phase {phase} ({phase_name}) Exit Gate...")
        print(f"   Script: {gate_script.name}")
        print(f"   Project: {self.project_root}")
        print()

        try:
            result = subprocess.run(
                ["python3", str(gate_script), str(self.project_root)],
                capture_output=True,
                text=True,
                timeout=600
            )

            # Print gate output
            print(result.stdout)
            if result.stderr:
                print("STDERR:", result.stderr)

            # Record result
            passed = result.returncode == 0
            self._record_gate_result(phase, passed, result.stdout, result.stderr)

            return passed

        except subprocess.TimeoutExpired:
            error_msg = f"Gate timed out after 10 minutes"
            print(f"❌ Phase {phase} {error_msg}")
            self._record_gate_result(phase, False, "", error_msg)
            return False

        except Exception as e:
            error_msg = f"Error running gate: {str(e)}"
            print(f"❌ Phase {phase} {error_msg}")
            self._record_gate_result(phase, False, "", error_msg)
            return False

    def _record_gate_result(self, phase: int, passed: bool, stdout: str, stderr: str = ""):
        """Record gate result in state file."""
        try:
            state = self._load_state()

            if 'gate_results' not in state:
                state['gate_results'] = {}

            phase_key = f'phase_{phase}'
            state['gate_results'][phase_key] = {
                'passed': passed,
                'timestamp': datetime.now().isoformat(),
                'phase_name': self.PHASE_NAMES.get(phase, f"Phase {phase}"),
                'stdout': stdout[:1000] if stdout else "",  # First 1000 chars
                'stderr': stderr[:500] if stderr else ""    # First 500 chars
            }

            # Update current phase in state
            if passed:
                state['last_completed_phase'] = phase
            else:
                state['last_failed_phase'] = phase

            self._save_state(state)

        except Exception as e:
            print(f"⚠️  Could not record gate result: {str(e)}")

    def check_phase_complete(self, phase: int) -> bool:
        """
        Check if phase has been marked complete via gate.

        Args:
            phase: Phase number to check

        Returns:
            True if phase gate has passed, False otherwise
        """
        try:
            state = self._load_state()
            gate_results = state.get('gate_results', {})
            phase_key = f'phase_{phase}'

            if phase_key not in gate_results:
                return False

            return gate_results[phase_key].get('passed', False)

        except Exception:
            return False

    def get_current_phase(self) -> int:
        """
        Get current phase based on state.

        Returns:
            Current phase number (1-6), or 1 if no state exists
        """
        try:
            state = self._load_state()
            return state.get('last_completed_phase', 0) + 1
        except Exception:
            return 1

    def _load_state(self) -> Dict:
        """Load orchestration state from file."""
        if not self.state_file.exists():
            return self._init_state()

        try:
            with open(self.state_file) as f:
                return json.load(f)
        except Exception:
            return self._init_state()

    def _save_state(self, state: Dict):
        """Save orchestration state to file."""
        # Ensure state directory exists
        self.state_file.parent.mkdir(parents=True, exist_ok=True)

        with open(self.state_file, 'w') as f:
            json.dump(state, f, indent=2)

    def _init_state(self) -> Dict:
        """Initialize new state structure."""
        return {
            'version': '0.14.0',
            'project_name': self.project_root.name,
            'created': datetime.now().isoformat(),
            'last_updated': datetime.now().isoformat(),
            'last_completed_phase': 0,
            'gate_results': {}
        }

    def print_state_summary(self):
        """Print summary of current state."""
        state = self._load_state()

        print("\n" + "="*70)
        print("ORCHESTRATION STATE SUMMARY")
        print("="*70)

        print(f"Project: {state.get('project_name', 'Unknown')}")
        print(f"Last Updated: {state.get('last_updated', 'Unknown')}")
        print(f"Current Phase: {self.get_current_phase()}")
        print()

        gate_results = state.get('gate_results', {})
        if gate_results:
            print("Gate Results:")
            for phase in range(1, 7):
                phase_key = f'phase_{phase}'
                if phase_key in gate_results:
                    result = gate_results[phase_key]
                    status = "✅ PASSED" if result['passed'] else "❌ FAILED"
                    phase_name = result.get('phase_name', f'Phase {phase}')
                    timestamp = result.get('timestamp', 'Unknown')[:19]  # Strip microseconds
                    print(f"  Phase {phase} ({phase_name:15s}): {status}  [{timestamp}]")
        else:
            print("No gate results recorded yet.")

        print("="*70 + "\n")


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Run phase gates for orchestration system"
    )
    parser.add_argument(
        "project_root",
        type=Path,
        help="Path to project root directory"
    )
    parser.add_argument(
        "phase",
        type=int,
        nargs='?',
        choices=range(1, 7),
        help="Phase number to validate (1-6)"
    )
    parser.add_argument(
        "--status",
        action="store_true",
        help="Show current orchestration state"
    )

    args = parser.parse_args()

    project_root = args.project_root.resolve()

    if not project_root.exists():
        print(f"❌ Project root not found: {project_root}")
        sys.exit(1)

    runner = GateRunner(project_root)

    if args.status:
        runner.print_state_summary()
        sys.exit(0)

    if args.phase is None:
        print("Usage: gate_runner.py <project_root> <phase>")
        print("   or: gate_runner.py <project_root> --status")
        print("\nExamples:")
        print("  python gate_runner.py . 5              # Run Phase 5 gate")
        print("  python gate_runner.py /path/to/proj 6  # Run Phase 6 gate")
        print("  python gate_runner.py . --status       # Show current state")
        sys.exit(1)

    passed = runner.run_gate(args.phase)

    print()
    if passed:
        print(f"✅ Phase {args.phase} gate PASSED - may proceed to Phase {args.phase + 1}")
    else:
        print(f"❌ Phase {args.phase} gate FAILED - cannot proceed")
        print(f"   Fix all issues above and re-run this gate")

    sys.exit(0 if passed else 1)


if __name__ == '__main__':
    main()
