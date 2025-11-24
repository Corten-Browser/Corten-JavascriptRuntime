#!/usr/bin/env python3
"""
Automatically execute gates at defined checkpoints.
No manual invocation needed - happens automatically.
"""
import json
import subprocess
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def get_gate_schedule() -> dict:
    """Load gate execution schedule."""
    schedule_file = _paths.gate_schedule

    if not schedule_file.exists():
        # Create default schedule
        default_schedule = {
            "gates": {
                "5": {
                    "name": "Integration Testing",
                    "trigger": "task_queue_percentage",
                    "threshold": 100,  # Run when all tasks complete
                    "command": "python orchestration/phase_gates/gate_runner.py . 5",
                    "blocking": True
                },
                "6": {
                    "name": "Verification",
                    "trigger": "after_gate",
                    "after": "5",
                    "command": "python orchestration/phase_gates/gate_runner.py . 6",
                    "blocking": True
                }
            }
        }
        schedule_file.parent.mkdir(parents=True, exist_ok=True)
        schedule_file.write_text(json.dumps(default_schedule, indent=2))
        return default_schedule

    return json.loads(schedule_file.read_text())


def check_gate_triggers() -> list[str]:
    """
    Check which gates should be triggered based on current state.
    Returns list of gate IDs that should run.
    """
    schedule = get_gate_schedule()
    log_file = _paths.gate_execution_log

    # Load execution history
    if log_file.exists():
        try:
            execution_log = json.loads(log_file.read_text())
        except json.JSONDecodeError:
            execution_log = {"executions": []}
    else:
        execution_log = {"executions": []}

    # Get already-run gates
    passed_gates = set(
        entry["gate_id"]
        for entry in execution_log["executions"]
        if entry.get("passed", False)
    )

    # Check each gate's trigger condition
    gates_to_run = []

    for gate_id, gate_config in schedule["gates"].items():
        if gate_id in passed_gates:
            continue  # Already passed

        trigger_type = gate_config["trigger"]

        if trigger_type == "task_queue_percentage":
            # Check task queue progress
            try:
                sys.path.insert(0, str(Path(__file__).parent.parent))
                from tasks.queue import TaskQueue
                queue = TaskQueue()
                progress = queue.get_progress()

                if progress["percentage"] >= gate_config["threshold"]:
                    gates_to_run.append(gate_id)
            except ImportError:
                # Task queue not available
                pass

        elif trigger_type == "after_gate":
            # Run after another gate passes
            prerequisite = gate_config["after"]
            if prerequisite in passed_gates:
                gates_to_run.append(gate_id)

        elif trigger_type == "manual":
            # Skip manual gates in automatic check
            continue

    return gates_to_run


def execute_gate(gate_id: str) -> bool:
    """
    Execute a single gate.
    Returns True if passed, False if failed.
    """
    schedule = get_gate_schedule()
    gate_config = schedule["gates"].get(gate_id)

    if not gate_config:
        print(f"Gate {gate_id} not found in schedule")
        return False

    print("=" * 70)
    print(f"AUTOMATED GATE EXECUTION: {gate_config['name']}")
    print("=" * 70)
    print(f"Gate ID: {gate_id}")
    print(f"Command: {gate_config['command']}")
    print(f"Blocking: {gate_config['blocking']}")
    print("")

    start_time = datetime.now()

    try:
        result = subprocess.run(
            gate_config["command"],
            shell=True,
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )
    except subprocess.TimeoutExpired:
        print("Gate execution timed out (5 minutes)")
        return False

    end_time = datetime.now()
    duration = (end_time - start_time).total_seconds()

    print("OUTPUT:")
    print(result.stdout)
    if result.stderr:
        print("STDERR:")
        print(result.stderr)

    passed = result.returncode == 0

    # Log execution
    log_file = _paths.gate_execution_log
    log_file.parent.mkdir(parents=True, exist_ok=True)
    if log_file.exists():
        try:
            log = json.loads(log_file.read_text())
        except json.JSONDecodeError:
            log = {"executions": []}
    else:
        log = {"executions": []}

    log["executions"].append({
        "gate_id": gate_id,
        "gate_name": gate_config["name"],
        "timestamp": start_time.isoformat(),
        "duration_seconds": duration,
        "passed": passed,
        "exit_code": result.returncode,
        "stdout": result.stdout[:5000],  # Truncate for storage
        "stderr": result.stderr[:1000]
    })

    log_file.write_text(json.dumps(log, indent=2))

    print("")
    if passed:
        print(f"GATE {gate_id} PASSED")
    else:
        print(f"GATE {gate_id} FAILED")
        if gate_config["blocking"]:
            print("This is a BLOCKING gate. Cannot proceed until it passes.")
    print("=" * 70)

    return passed


def run_pending_gates():
    """
    Check and run all gates that should be triggered.
    This is called automatically by the orchestration system.
    """
    gates_to_run = check_gate_triggers()

    if not gates_to_run:
        print("No gates currently triggered")
        return True

    print(f"Gates to run: {gates_to_run}")

    for gate_id in gates_to_run:
        passed = execute_gate(gate_id)

        if not passed:
            schedule = get_gate_schedule()
            if schedule["gates"][gate_id].get("blocking", False):
                print(f"BLOCKING GATE {gate_id} FAILED")
                print("Fix issues and retry")
                return False

    return True


if __name__ == "__main__":
    if len(sys.argv) > 1:
        if sys.argv[1] == "--check":
            gates = check_gate_triggers()
            print(f"Gates triggered: {gates}")
        elif sys.argv[1] == "--run":
            if len(sys.argv) > 2:
                success = execute_gate(sys.argv[2])
                sys.exit(0 if success else 1)
            else:
                print("Usage: --run <gate_id>")
        else:
            print("Usage: gate_executor.py [--check | --run <gate_id>]")
    else:
        success = run_pending_gates()
        sys.exit(0 if success else 1)
