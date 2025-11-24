#!/usr/bin/env python3
"""
Pre-push hook that blocks pushing incomplete work.
Ensures remote repository only receives verified complete work.
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


def load_completion_state() -> dict:
    """Load completion state."""
    state_file = _paths.completion_state
    if state_file.exists():
        try:
            return json.loads(state_file.read_text())
        except json.JSONDecodeError:
            return {}
    return {}


def load_queue_state() -> dict:
    """Load queue state."""
    queue_file = _paths.queue_state
    if queue_file.exists():
        try:
            return json.loads(queue_file.read_text())
        except json.JSONDecodeError:
            return {"tasks": []}
    return {"tasks": []}


def queue_is_empty() -> bool:
    """Check if queue is empty."""
    state = load_queue_state()
    tasks = state.get("tasks", [])
    if not tasks:
        return True
    return all(t.get("status") == "completed" for t in tasks)


def log_activity(event_type: str, details: dict):
    """Log activity."""
    log_file = _paths.activity_log
    if not log_file.exists():
        return

    try:
        log = json.loads(log_file.read_text())
        log["events"].append({
            "timestamp": datetime.now().isoformat(),
            "type": event_type,
            "details": details
        })
        log["events"] = log["events"][-1000:]
        log_file.write_text(json.dumps(log, indent=2))
    except Exception:
        pass


def main():
    """Main pre-push enforcement logic."""
    print("=" * 60)
    print("ENFORCEMENT PRE-PUSH CHECK")
    print("=" * 60)
    print("")

    queue_state = load_queue_state()
    tasks = queue_state.get("tasks", [])

    # If queue has tasks, check they're all complete
    if tasks and not queue_is_empty():
        remaining = len([t for t in tasks if t.get("status") != "completed"])
        print("PUSH BLOCKED: Task queue not empty")
        print("")
        print(f"Tasks remaining: {remaining}/{len(tasks)}")
        print("")
        print("Cannot push incomplete work to remote repository.")
        print("Complete all tasks and pass verification first.")
        print("=" * 60)

        log_activity("push_blocked", {
            "reason": "queue_not_empty",
            "remaining_tasks": remaining
        })

        sys.exit(1)

    # Check verification status
    completion_state = load_completion_state()
    if tasks and not completion_state.get("verification_agent_approved", False):
        print("PUSH BLOCKED: Verification not approved")
        print("")
        print("Cannot push unverified work.")
        print("Run verification first:")
        print("  python orchestration/verification/run_full_verification.py")
        print("=" * 60)

        log_activity("push_blocked", {
            "reason": "verification_not_approved"
        })

        sys.exit(1)

    print("All pre-push checks PASSED")
    print("Push allowed")
    print("=" * 60)

    log_activity("push_allowed", {
        "queue_complete": queue_is_empty()
    })

    sys.exit(0)


if __name__ == "__main__":
    main()
