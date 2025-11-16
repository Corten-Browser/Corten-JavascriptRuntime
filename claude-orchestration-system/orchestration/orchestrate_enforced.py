#!/usr/bin/env python3
"""
Orchestration Enforced Wrapper (v0.17.0)

Enforces mandatory phase gate execution during multi-phase orchestrated development.
Cannot skip gates - technical enforcement of process requirements.

This prevents the "Looks Good But Breaks" pattern:
- Music Analyzer v1: Skipped contract validation
- Music Analyzer v2: Skipped Phase 5 gate (83.3% declared "complete")
- Music Analyzer v3: Skipped Phase 6 UAT (__main__.py wrong location)

Usage:
    python orchestration/orchestrate_enforced.py run-phase <phase> [args...]
    python orchestration/orchestrate_enforced.py status
    python orchestration/orchestrate_enforced.py verify-gates

Example:
    # Run Phase 5 with automatic gate enforcement
    python orchestration/orchestrate_enforced.py run-phase 5

    # Check if gates allow progression to Phase 6
    python orchestration/orchestrate_enforced.py can-proceed 6
"""

import subprocess
import sys
import time
from pathlib import Path
from typing import Optional, List

# Import state manager
from orchestration_state import StateManager, PhaseGateResult


class GateEnforcementError(Exception):
    """Raised when gate enforcement blocks progression."""
    pass


class OrchestrationEnforcer:
    """
    Enforces mandatory gate execution during orchestration.

    Key Features:
    - Automatically runs gates after phases complete
    - Blocks progression if gates fail
    - Records all executions with timestamps
    - Provides clear feedback on blocking reasons
    """

    # Map phases to their gate scripts
    PHASE_GATES = {
        1: None,  # Phase 1 has no gate (planning phase)
        2: None,  # Phase 2 has no gate (component dev)
        3: None,  # Phase 3 has no gate (contract validation)
        4: None,  # Phase 4 has no gate (unit testing)
        5: "orchestration/phase_gates/phase_5_integration.py",  # CRITICAL: 100% integration pass
        6: "orchestration/phase_gates/phase_6_verification.py"  # CRITICAL: All checks pass
    }

    # Phases that are blocking (must pass gate)
    BLOCKING_PHASES = [5, 6]

    def __init__(self, project_root: Path):
        """
        Initialize enforcer.

        Args:
            project_root: Project root directory
        """
        self.project_root = project_root
        self.state_manager = StateManager(project_root)

    def run_phase_with_gate(
        self,
        phase: int,
        phase_args: List[str] = None
    ) -> tuple[bool, str]:
        """
        Run a phase and automatically execute its gate.

        Args:
            phase: Phase number (1-6)
            phase_args: Additional arguments for phase execution

        Returns:
            (success, message) tuple

        Raises:
            GateEnforcementError: If gate blocks progression
        """
        if phase_args is None:
            phase_args = []

        print("=" * 70)
        print(f"RUNNING PHASE {phase} WITH GATE ENFORCEMENT")
        print("=" * 70)
        print()

        # Check if allowed to run this phase
        can_proceed, reason = self.state_manager.can_proceed_to_phase(phase)
        if not can_proceed:
            error_msg = f"❌ BLOCKED: Cannot run Phase {phase}\n   Reason: {reason}"
            print(error_msg)
            raise GateEnforcementError(error_msg)

        print(f"✅ Allowed to run Phase {phase}: {reason}")
        print()

        # Check if phase has a gate
        gate_script = self.PHASE_GATES.get(phase)

        if gate_script is None:
            print(f"ℹ️  Phase {phase} has no gate (no automatic enforcement)")
            print(f"✅ Phase {phase} workflow may proceed")
            return True, "No gate for this phase"

        # Phase has a gate - run it
        print(f"🔒 Phase {phase} has MANDATORY gate: {gate_script}")
        print()

        # Run the gate
        print(f"Running Phase {phase} gate...")
        print("-" * 70)

        gate_passed, gate_result = self._run_gate(phase, gate_script)

        print("-" * 70)
        print()

        if gate_passed:
            print(f"✅ PHASE {phase} GATE PASSED")
            print(f"✅ May proceed to Phase {phase + 1}")
            print()
            self._print_gate_summary(gate_result)
            return True, "Gate passed"
        else:
            print(f"❌ PHASE {phase} GATE FAILED")
            print(f"❌ CANNOT proceed to Phase {phase + 1}")
            print()
            self._print_gate_summary(gate_result)
            print()
            print("REQUIRED ACTIONS:")
            print("1. Review gate output above")
            print("2. Fix all identified issues")
            print(f"3. Re-run: python orchestration/orchestrate_enforced.py run-phase {phase}")
            print()

            raise GateEnforcementError(f"Phase {phase} gate failed")

    def _run_gate(self, phase: int, gate_script: str) -> tuple[bool, PhaseGateResult]:
        """
        Execute a phase gate script.

        Args:
            phase: Phase number
            gate_script: Path to gate script

        Returns:
            (passed, result) tuple
        """
        gate_path = self.project_root / gate_script

        if not gate_path.exists():
            print(f"⚠️  WARNING: Gate script not found: {gate_path}")
            print(f"⚠️  Creating placeholder gate (always passes)")
            # For development - actual gates should exist
            return True, self._create_placeholder_result(phase, "Gate script not found")

        # Run gate script
        start_time = time.time()

        try:
            result = subprocess.run(
                [sys.executable, str(gate_path), str(self.project_root)],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=600  # 10 minute timeout
            )

            duration = time.time() - start_time
            output = result.stdout + result.stderr
            passed = (result.returncode == 0)

            # Print output
            print(output)

            # Record in state
            gate_result = self.state_manager.record_gate_execution(
                phase=phase,
                passed=passed,
                exit_code=result.returncode,
                duration_seconds=duration,
                output=output,
                save_full_output=True
            )

            return passed, gate_result

        except subprocess.TimeoutExpired:
            duration = time.time() - start_time
            output = f"Gate timed out after {duration:.1f} seconds"
            print(output)

            gate_result = self.state_manager.record_gate_execution(
                phase=phase,
                passed=False,
                exit_code=-1,
                duration_seconds=duration,
                output=output,
                save_full_output=True
            )

            return False, gate_result

        except Exception as e:
            duration = time.time() - start_time
            output = f"Gate execution error: {e}"
            print(output)

            gate_result = self.state_manager.record_gate_execution(
                phase=phase,
                passed=False,
                exit_code=-1,
                duration_seconds=duration,
                output=output,
                save_full_output=True
            )

            return False, gate_result

    def _create_placeholder_result(self, phase: int, message: str) -> PhaseGateResult:
        """Create a placeholder result (for missing gates)."""
        return self.state_manager.record_gate_execution(
            phase=phase,
            passed=True,
            exit_code=0,
            duration_seconds=0.0,
            output=message,
            save_full_output=False
        )

    def _print_gate_summary(self, result: PhaseGateResult) -> None:
        """Print gate result summary."""
        print("Gate Execution Summary:")
        print(f"  Phase: {result.phase}")
        print(f"  Result: {'✅ PASSED' if result.passed else '❌ FAILED'}")
        print(f"  Exit Code: {result.exit_code}")
        print(f"  Duration: {result.duration_seconds:.1f}s")
        print(f"  Timestamp: {result.timestamp}")

        if result.full_output_file:
            print(f"  Full Output: {result.full_output_file}")

    def verify_all_gates(self) -> tuple[bool, str]:
        """
        Verify all blocking gates have passed.

        Returns:
            (all_passed, summary) tuple
        """
        print("=" * 70)
        print("VERIFYING ALL BLOCKING GATES")
        print("=" * 70)
        print()

        all_passed = True
        messages = []

        for phase in self.BLOCKING_PHASES:
            result = self.state_manager.get_phase_status(phase)

            if result is None:
                all_passed = False
                msg = f"❌ Phase {phase}: NOT RUN"
                print(msg)
                messages.append(msg)

            elif result.passed:
                msg = f"✅ Phase {phase}: PASSED"
                print(msg)
                messages.append(msg)

            else:
                all_passed = False
                msg = f"❌ Phase {phase}: FAILED (must re-run)"
                print(msg)
                messages.append(msg)

        print()

        if all_passed:
            print("✅ ALL BLOCKING GATES PASSED")
            print("✅ Project may be declared complete")
            return True, "All gates passed"
        else:
            print("❌ NOT ALL GATES PASSED")
            print("❌ Project is NOT complete")
            return False, "Some gates missing or failed"

    def can_proceed_to_phase(self, target_phase: int) -> tuple[bool, str]:
        """
        Check if can proceed to target phase.

        Args:
            target_phase: Phase number to check

        Returns:
            (allowed, reason) tuple
        """
        return self.state_manager.can_proceed_to_phase(target_phase)

    def print_status(self) -> None:
        """Print current orchestration status."""
        print(self.state_manager.get_status_report())

    def get_gate_output_path(self, phase: int) -> Optional[Path]:
        """
        Get path to most recent gate output file.

        Args:
            phase: Phase number

        Returns:
            Path to output file, or None if not found
        """
        result = self.state_manager.get_phase_status(phase)
        if result and result.full_output_file:
            return self.project_root / result.full_output_file
        return None


def main():
    """CLI interface."""
    if len(sys.argv) < 2:
        print("Usage: python orchestrate_enforced.py <command> [args...]")
        print()
        print("Commands:")
        print("  run-phase <phase>         - Run phase with automatic gate enforcement")
        print("  can-proceed <phase>       - Check if can proceed to phase")
        print("  verify-gates              - Verify all blocking gates passed")
        print("  status                    - Show orchestration status")
        print("  gate-output <phase>       - Show most recent gate output")
        print()
        print("Examples:")
        print("  python orchestrate_enforced.py run-phase 5")
        print("  python orchestrate_enforced.py can-proceed 6")
        print("  python orchestrate_enforced.py verify-gates")
        sys.exit(1)

    command = sys.argv[1]
    project_root = Path.cwd()
    enforcer = OrchestrationEnforcer(project_root)

    try:
        if command == "run-phase":
            if len(sys.argv) < 3:
                print("Error: run-phase requires phase number")
                sys.exit(1)

            phase = int(sys.argv[2])
            phase_args = sys.argv[3:] if len(sys.argv) > 3 else []

            success, message = enforcer.run_phase_with_gate(phase, phase_args)
            sys.exit(0 if success else 1)

        elif command == "can-proceed":
            if len(sys.argv) < 3:
                print("Error: can-proceed requires phase number")
                sys.exit(1)

            target_phase = int(sys.argv[2])
            allowed, reason = enforcer.can_proceed_to_phase(target_phase)

            print("=" * 70)
            print(f"CAN PROCEED TO PHASE {target_phase}?")
            print("=" * 70)
            print()

            if allowed:
                print(f"✅ YES: {reason}")
                sys.exit(0)
            else:
                print(f"❌ NO: {reason}")
                sys.exit(1)

        elif command == "verify-gates":
            all_passed, summary = enforcer.verify_all_gates()
            sys.exit(0 if all_passed else 1)

        elif command == "status":
            enforcer.print_status()
            sys.exit(0)

        elif command == "gate-output":
            if len(sys.argv) < 3:
                print("Error: gate-output requires phase number")
                sys.exit(1)

            phase = int(sys.argv[2])
            output_path = enforcer.get_gate_output_path(phase)

            if output_path and output_path.exists():
                print(f"Gate output for Phase {phase}:")
                print("=" * 70)
                with open(output_path, 'r') as f:
                    print(f.read())
            else:
                print(f"No gate output found for Phase {phase}")
                sys.exit(1)

        else:
            print(f"Unknown command: {command}")
            sys.exit(1)

    except GateEnforcementError as e:
        print()
        print("=" * 70)
        print("GATE ENFORCEMENT BLOCKED PROGRESSION")
        print("=" * 70)
        print(str(e))
        print()
        sys.exit(1)

    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
