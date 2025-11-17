#!/usr/bin/env python3
"""
Task runner that enforces queue-based execution.
Model MUST process tasks in queue order.
"""
import sys
import json
from pathlib import Path
from .queue import TaskQueue


def get_current_task() -> dict:
    """
    Get the current task model MUST work on.
    This is the AUTHORITATIVE instruction, not a suggestion.
    """
    queue = TaskQueue()

    # Check if there's a task in progress
    in_progress = [t for t in queue.tasks.values() if t.status.value == "in_progress"]

    if in_progress:
        task = in_progress[0]
        return {
            "status": "in_progress",
            "task_id": task.id,
            "task_name": task.name,
            "task_description": task.description,
            "feature_id": task.feature_id,
            "instruction": f"CONTINUE working on: {task.name}. Do NOT start anything else."
        }

    # Get next task from queue
    next_task = queue.get_next_task()

    if next_task is None:
        if queue.is_complete():
            return {
                "status": "complete",
                "instruction": "ALL TASKS COMPLETE. Queue is empty. Verification may now run."
            }
        else:
            remaining = queue.get_remaining_tasks()
            return {
                "status": "blocked",
                "remaining_tasks": [t.id for t in remaining],
                "instruction": f"Tasks blocked. Check dependencies: {[t.id for t in remaining]}"
            }

    # Mark task as started
    queue.start_task(next_task.id)

    return {
        "status": "new_task",
        "task_id": next_task.id,
        "task_name": next_task.name,
        "task_description": next_task.description,
        "feature_id": next_task.feature_id,
        "instruction": f"START TASK: {next_task.name}\n\nYou MUST implement this task completely before proceeding. No partial work. No skipping. No redefining scope."
    }


def mark_task_complete(task_id: str, verification_passed: bool = True):
    """
    Mark task as complete. Only call after full implementation + tests.
    """
    queue = TaskQueue()
    queue.complete_task(task_id, {"verified": verification_passed})

    print(f"Task {task_id} marked complete")

    # Immediately show next task
    next_info = get_current_task()
    print(f"Next: {json.dumps(next_info, indent=2)}")


def print_queue_authority_message():
    """
    Print message emphasizing queue is authoritative.
    """
    print("=" * 70)
    print("TASK QUEUE AUTHORITY")
    print("=" * 70)
    print("")
    print("The task queue determines what you work on. You do NOT decide.")
    print("")
    print("RULES:")
    print("1. Work ONLY on the current task")
    print("2. Complete it FULLY (implementation + tests)")
    print("3. Mark it complete")
    print("4. Queue provides next task")
    print("5. Repeat until queue is empty")
    print("")
    print("You CANNOT:")
    print("  Skip tasks")
    print("  Redefine task scope")
    print("  Work on future tasks")
    print("  Declare project complete while tasks remain")
    print("  Create your own task list")
    print("")
    print("=" * 70)


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--current":
        info = get_current_task()
        print(json.dumps(info, indent=2))
    elif len(sys.argv) > 2 and sys.argv[1] == "--complete":
        mark_task_complete(sys.argv[2])
    else:
        print_queue_authority_message()
        queue = TaskQueue()
        queue.print_status()
