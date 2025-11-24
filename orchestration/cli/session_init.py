#!/usr/bin/env python3
"""
Generates queue status that gets injected into model context.
Called automatically when orchestration session starts.
Provides mandatory visibility into enforcement state.
"""
import json
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def load_queue_state() -> dict:
    """Load task queue state."""
    queue_file = _paths.queue_state
    if queue_file.exists():
        try:
            return json.loads(queue_file.read_text())
        except json.JSONDecodeError:
            return {"tasks": [], "initialized": False}
    return {"tasks": [], "initialized": False}


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


def hooks_installed() -> bool:
    """Check if git hooks are installed."""
    return (
        Path(".git/hooks/pre-commit").exists() and
        Path(".git/hooks/pre-push").exists()
    )


def get_queue_status(queue_state: dict) -> str:
    """Get human-readable queue status."""
    tasks = queue_state.get("tasks", [])

    if not tasks:
        if queue_state.get("initialized"):
            return "EMPTY (all tasks completed)"
        else:
            return "NOT INITIALIZED (no tasks defined)"

    completed = sum(1 for t in tasks if t.get("status") == "completed")
    in_progress = sum(1 for t in tasks if t.get("status") == "in_progress")
    incomplete = sum(1 for t in tasks if t.get("status") == "incomplete")
    pending = sum(1 for t in tasks if t.get("status") == "pending")

    if completed == len(tasks):
        return "COMPLETE (all tasks done)"

    percentage = (completed / len(tasks)) * 100
    status = f"{percentage:.1f}% complete ({completed}/{len(tasks)} tasks)"

    if incomplete > 0:
        status += f" [{incomplete} need verification]"

    return status


def get_current_task(queue_state: dict) -> dict:
    """Get current task. Priority: in_progress > incomplete > pending."""
    tasks = queue_state.get("tasks", [])

    # Find in-progress first
    for task in tasks:
        if task.get("status") == "in_progress":
            return task

    # Find incomplete (needs verification) - prioritize over pending
    for task in tasks:
        if task.get("status") == "incomplete":
            return task

    # Find first pending
    for task in tasks:
        if task.get("status") == "pending":
            return task

    return None


def get_remaining_count(queue_state: dict) -> int:
    """Get count of remaining tasks."""
    tasks = queue_state.get("tasks", [])
    return sum(1 for t in tasks if t.get("status") != "completed")


def get_next_action(queue_state: dict, verification: dict) -> str:
    """Determine the next mandatory action."""
    tasks = queue_state.get("tasks", [])

    if not tasks:
        return "Initialize task queue from specification file"

    remaining = get_remaining_count(queue_state)

    if remaining > 0:
        current = get_current_task(queue_state)
        if current:
            return f"Work on: {current['id']} - {current['name']}"
        else:
            return "Start the next pending task"

    if not verification.get("verification_agent_approved"):
        return "Run verification (automatic on next commit)"

    return "All tasks complete and verified. Ready for final commit."


def check_and_recover_stale_tasks() -> int:
    """
    Check for stale IN_PROGRESS tasks and recover them.

    Called at session startup to detect tasks that were left
    IN_PROGRESS due to a crash or timeout.

    Returns:
        Number of recovered tasks
    """
    try:
        from orchestration.tasks.stale_recovery import check_and_recover_at_startup
        return check_and_recover_at_startup()
    except ImportError:
        # Module not available (shouldn't happen but handle gracefully)
        return 0
    except Exception as e:
        # Don't fail session init if recovery fails
        print(f"  Warning: Stale recovery check failed: {e}")
        return 0


def generate_enforcement_context() -> str:
    """Generate markdown context for model."""
    # Check for and recover stale tasks at session start
    recovered = check_and_recover_stale_tasks()

    queue_state = load_queue_state()
    verification = load_verification_state()
    config = load_config()

    current = get_current_task(queue_state)
    remaining = get_remaining_count(queue_state)
    total = len(queue_state.get("tasks", []))

    current_str = f"{current['id']} - {current['name']}" if current else "None"
    current_status = current.get("status", "unknown") if current else "none"

    # Get incomplete count for display
    tasks = queue_state.get("tasks", [])
    incomplete_count = sum(1 for t in tasks if t.get("status") == "incomplete")

    context = f"""
## ENFORCEMENT SYSTEM STATUS (AUTO-GENERATED)

**Queue Status**: {get_queue_status(queue_state)}
**Current Task**: {current_str} ({current_status})
**Tasks Remaining**: {remaining}/{total}
**Tasks Needing Verification**: {incomplete_count}
**Verification**: {"APPROVED" if verification.get("verification_agent_approved") else "NOT APPROVED"}

### Your Next Action (MANDATORY)
{get_next_action(queue_state, verification)}

### Active Enforcement Blocks
- Pre-commit hook: {"ACTIVE" if hooks_installed() else "NOT INSTALLED"}
- Blocking mode: {"ENABLED" if config.get("blocking_mode", True) else "DISABLED"}
- Auto-verification: {"ENABLED" if config.get("auto_run_verification", True) else "DISABLED"}

### What This Means
- You CANNOT commit until task queue is empty
- You CANNOT skip tasks or redefine their scope
- Verification runs AUTOMATICALLY when queue empties
- Git hooks enforce this - not optional guidance

### Commands
```bash
# View queue status
python orchestration/tasks/task_runner.py

# Get current task details
python orchestration/tasks/task_runner.py --current

# Mark task complete (after implementation + tests)
python orchestration/tasks/task_runner.py --complete {current['id'] if current else 'TASK-ID'}

# Run verification manually
python orchestration/verification/run_full_verification.py
```

---
"""
    return context


def print_status_summary():
    """Print a brief status summary."""
    # Check for stale tasks first
    check_and_recover_stale_tasks()

    queue_state = load_queue_state()
    verification = load_verification_state()

    print("=" * 60)
    print("ENFORCEMENT SYSTEM STATUS")
    print("=" * 60)

    tasks = queue_state.get("tasks", [])
    if tasks:
        completed = sum(1 for t in tasks if t.get("status") == "completed")
        incomplete = sum(1 for t in tasks if t.get("status") == "incomplete")
        print(f"Queue: {completed}/{len(tasks)} tasks completed")
        if incomplete > 0:
            print(f"       {incomplete} tasks need verification")

        current = get_current_task(queue_state)
        if current:
            print(f"Current task: {current['id']} - {current['name']} ({current.get('status', 'unknown')})")
    else:
        print("Queue: Not initialized")

    print(f"Verification: {'APPROVED' if verification.get('verification_agent_approved') else 'NOT APPROVED'}")
    print(f"Hooks: {'ACTIVE' if hooks_installed() else 'NOT INSTALLED'}")
    print("=" * 60)


if __name__ == "__main__":
    if len(sys.argv) > 1:
        if sys.argv[1] == "--context":
            # Generate full context for injection
            print(generate_enforcement_context())
        elif sys.argv[1] == "--json":
            # Output as JSON
            queue_state = load_queue_state()
            verification = load_verification_state()
            output = {
                "queue_status": get_queue_status(queue_state),
                "current_task": get_current_task(queue_state),
                "remaining_tasks": get_remaining_count(queue_state),
                "total_tasks": len(queue_state.get("tasks", [])),
                "verification_approved": verification.get("verification_agent_approved", False),
                "hooks_installed": hooks_installed(),
                "next_action": get_next_action(queue_state, verification)
            }
            print(json.dumps(output, indent=2))
        else:
            print_status_summary()
    else:
        print_status_summary()
