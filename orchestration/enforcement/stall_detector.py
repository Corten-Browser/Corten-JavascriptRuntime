#!/usr/bin/env python3
"""
Detects when orchestration has stalled.
Monitors for incomplete work without progress.
"""
import json
import sys
from pathlib import Path
from datetime import datetime, timedelta

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def load_activity_log() -> dict:
    """Load activity log."""
    log_file = _paths.activity_log
    if log_file.exists():
        try:
            return json.loads(log_file.read_text())
        except json.JSONDecodeError:
            return {"events": []}
    return {"events": []}


def load_queue_state() -> dict:
    """Load queue state."""
    queue_file = _paths.queue_state
    if queue_file.exists():
        try:
            return json.loads(queue_file.read_text())
        except json.JSONDecodeError:
            return {"tasks": []}
    return {"tasks": []}


def get_last_activity_timestamp() -> datetime:
    """Get timestamp of last activity."""
    log = load_activity_log()
    events = log.get("events", [])

    if events:
        last_event = events[-1]
        return datetime.fromisoformat(last_event["timestamp"])

    # Check queue state last updated
    queue_file = _paths.queue_state
    if queue_file.exists():
        return datetime.fromtimestamp(queue_file.stat().st_mtime)

    return datetime.now()


def has_pending_work() -> bool:
    """Check if there's incomplete work in queue."""
    state = load_queue_state()
    tasks = state.get("tasks", [])

    if not tasks:
        return False

    return any(t.get("status") != "completed" for t in tasks)


def get_stalled_tasks() -> list[dict]:
    """Find tasks that have been in_progress for too long."""
    state = load_queue_state()
    tasks = state.get("tasks", [])
    stalled = []

    for task in tasks:
        if task.get("status") == "in_progress" and task.get("started_at"):
            started = datetime.fromisoformat(task["started_at"])
            duration = datetime.now() - started

            if duration > timedelta(hours=2):
                stalled.append({
                    "task": task,
                    "duration_hours": duration.total_seconds() / 3600
                })

    return stalled


def check_for_stall() -> dict:
    """
    Check if orchestration has stalled.
    Returns alert dict if stalled, None otherwise.
    """
    if not has_pending_work():
        return None

    last_activity = get_last_activity_timestamp()
    time_since_activity = datetime.now() - last_activity

    # Load config for threshold
    config_file = _paths.enforcement_config
    threshold_minutes = 60  # Default 1 hour

    if config_file.exists():
        try:
            config = json.loads(config_file.read_text())
            threshold_minutes = config.get("stall_threshold_minutes", 60)
        except Exception:
            pass

    if time_since_activity > timedelta(minutes=threshold_minutes):
        state = load_queue_state()
        tasks = state.get("tasks", [])
        remaining = sum(1 for t in tasks if t.get("status") != "completed")

        return {
            "type": "stall_detected",
            "timestamp": datetime.now().isoformat(),
            "time_since_activity_minutes": time_since_activity.total_seconds() / 60,
            "remaining_tasks": remaining,
            "total_tasks": len(tasks),
            "severity": "warning",
            "message": f"No activity for {time_since_activity.total_seconds()/60:.0f} minutes with {remaining} tasks remaining",
            "recommendation": "Check if model has stopped prematurely. Resume work."
        }

    # Check for stuck tasks
    stalled_tasks = get_stalled_tasks()
    if stalled_tasks:
        return {
            "type": "task_stuck",
            "timestamp": datetime.now().isoformat(),
            "stalled_tasks": [
                {
                    "id": st["task"]["id"],
                    "name": st["task"]["name"],
                    "hours_stuck": st["duration_hours"]
                }
                for st in stalled_tasks
            ],
            "severity": "warning",
            "message": f"{len(stalled_tasks)} task(s) stuck in progress for >2 hours",
            "recommendation": "Complete or reset stuck tasks"
        }

    return None


def main():
    """Run stall detection."""
    print("=" * 60)
    print("STALL DETECTION CHECK")
    print("=" * 60)
    print("")

    alert = check_for_stall()

    if alert:
        print(f"ALERT: {alert['type']}")
        print(f"Severity: {alert['severity']}")
        print(f"Message: {alert['message']}")
        print(f"Recommendation: {alert['recommendation']}")

        # Log alert
        alerts_file = _paths.alerts
        alerts_file.parent.mkdir(parents=True, exist_ok=True)
        if alerts_file.exists():
            try:
                alerts = json.loads(alerts_file.read_text())
            except json.JSONDecodeError:
                alerts = []
        else:
            alerts = []

        alerts.append(alert)
        alerts = alerts[-100:]  # Keep last 100
        alerts_file.write_text(json.dumps(alerts, indent=2))

        print("")
        print(f"Alert logged to: {alerts_file}")
    else:
        print("No stall detected")
        if has_pending_work():
            last = get_last_activity_timestamp()
            minutes_ago = (datetime.now() - last).total_seconds() / 60
            print(f"Last activity: {minutes_ago:.0f} minutes ago")
        else:
            print("No pending work in queue")

    print("=" * 60)


if __name__ == "__main__":
    main()
