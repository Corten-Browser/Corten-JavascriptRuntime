#!/usr/bin/env python3
"""
Post-commit enforcement hook.
Runs AFTER every successful commit.
Generates forceful continuation messages when work is incomplete.

This hook does NOT block commits - it allows progress preservation.
Instead, it maximizes pressure on LLM to continue working.
"""
import json
import sys
from pathlib import Path
from datetime import datetime

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths
from orchestration.enforcement.continuation_message import (
    generate_forceful_message,
    generate_verification_needed_message
)

# Global paths instance
_paths = DataPaths()


def load_queue_state() -> dict:
    """Load task queue state."""
    queue_file = _paths.queue_state
    if queue_file.exists():
        try:
            return json.loads(queue_file.read_text())
        except json.JSONDecodeError:
            return {"tasks": []}
    return {"tasks": []}


def load_verification_state() -> dict:
    """Load verification state."""
    state_file = _paths.completion_state
    if state_file.exists():
        try:
            return json.loads(state_file.read_text())
        except json.JSONDecodeError:
            return {}
    return {}


def load_config() -> dict:
    """Load enforcement config."""
    config_file = _paths.enforcement_config
    if config_file.exists():
        try:
            return json.loads(config_file.read_text())
        except json.JSONDecodeError:
            return {}
    return {}


def get_current_task(tasks: list) -> dict:
    """Get current task (in_progress or first pending)."""
    # Find in_progress
    for task in tasks:
        if task.get("status") == "in_progress":
            return task

    # Find first pending
    for task in tasks:
        if task.get("status") == "pending":
            return task

    return {}


def get_remaining_tasks(tasks: list) -> list:
    """Get list of incomplete tasks."""
    return [t for t in tasks if t.get("status") != "completed"]


def log_activity(event_type: str, details: dict):
    """Log enforcement activity."""
    log_file = _paths.activity_log
    log_file.parent.mkdir(parents=True, exist_ok=True)

    if log_file.exists():
        try:
            log = json.loads(log_file.read_text())
        except json.JSONDecodeError:
            log = {"events": []}
    else:
        log = {"events": []}

    log["events"].append({
        "timestamp": datetime.now().isoformat(),
        "type": event_type,
        "details": details
    })

    # Keep last 1000 events
    log["events"] = log["events"][-1000:]

    log_file.write_text(json.dumps(log, indent=2))


def main():
    """
    Post-commit enforcement - generate continuation message if work incomplete.
    """
    config = load_config()

    # Check if enforcement is enabled
    if not config.get("blocking_mode", True):
        # Enforcement disabled - silent exit
        sys.exit(0)

    # Load states
    queue_state = load_queue_state()
    verification_state = load_verification_state()

    tasks = queue_state.get("tasks", [])
    verification_approved = verification_state.get("verification_agent_approved", False)

    if not tasks:
        # No queue initialized - nothing to enforce
        sys.exit(0)

    # Calculate status
    total_tasks = len(tasks)
    completed_tasks = sum(1 for t in tasks if t.get("status") == "completed")
    remaining_count = total_tasks - completed_tasks

    # Log the commit event
    log_activity("post_commit_check", {
        "total_tasks": total_tasks,
        "completed_tasks": completed_tasks,
        "remaining_tasks": remaining_count,
        "verification_approved": verification_approved
    })

    # Determine what message to show
    if remaining_count == 0 and verification_approved:
        # All complete - no message needed (or optional success message)
        print("")
        print("═" * 72)
        print("✅ ORCHESTRATION COMPLETE - All tasks done, verification passed ✅")
        print("═" * 72)
        print("")
        sys.exit(0)

    if remaining_count == 0 and not verification_approved:
        # Queue empty but verification needed
        message = generate_verification_needed_message(remaining_count)
        print(message)
        log_activity("verification_reminder", {
            "queue_empty": True,
            "verification_needed": True
        })
        sys.exit(0)

    # Work remains - generate forceful continuation message
    current_task = get_current_task(tasks)
    remaining_tasks = get_remaining_tasks(tasks)

    queue_status = {
        "total": total_tasks,
        "completed": completed_tasks
    }

    # Get repeat count from config (default 3)
    repeat_count = config.get("repeat_critical_message", 3)

    message = generate_forceful_message(
        queue_status=queue_status,
        current_task=current_task,
        remaining_tasks=remaining_tasks,
        verification_approved=verification_approved,
        repeat_count=repeat_count
    )

    print(message)

    # Log that continuation message was shown
    log_activity("continuation_message_shown", {
        "remaining_tasks": remaining_count,
        "next_task": current_task.get("id", "unknown"),
        "message_force_level": "maximum"
    })

    # Always exit 0 - we don't block, we inform
    sys.exit(0)


if __name__ == "__main__":
    main()
