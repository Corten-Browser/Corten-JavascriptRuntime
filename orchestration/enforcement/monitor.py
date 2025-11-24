#!/usr/bin/env python3
"""
Continuous monitoring system.
Detects stalling, premature completion attempts, and other anomalies.
"""
import json
import time
import sys
from pathlib import Path
from datetime import datetime, timedelta

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths


class OrchestratorMonitor:
    """
    Monitor orchestration progress and detect anomalies.
    """

    def __init__(self):
        self._paths = DataPaths()
        self.metrics_file = self._paths.metrics
        self.alerts_file = self._paths.alerts

    def collect_metrics(self) -> dict:
        """Collect current system metrics."""
        metrics = {
            "timestamp": datetime.now().isoformat(),
        }

        # Task queue progress
        try:
            sys.path.insert(0, str(Path(__file__).parent.parent))
            from tasks.queue import TaskQueue
            queue = TaskQueue()
            metrics["queue_progress"] = queue.get_progress()
        except ImportError:
            metrics["queue_progress"] = None

        # Gate status
        try:
            log_file = self._paths.gate_execution_log
            if log_file.exists():
                log = json.loads(log_file.read_text())
                metrics["gates_passed"] = sum(
                    1 for e in log.get("executions", []) if e.get("passed")
                )
            else:
                metrics["gates_passed"] = 0
        except (json.JSONDecodeError, KeyError):
            metrics["gates_passed"] = 0

        # File modification activity
        recent_activity = self._check_recent_activity()
        metrics["recent_file_changes"] = recent_activity

        # Check for completion report attempts
        metrics["completion_report_exists"] = Path("COMPLETION-REPORT.md").exists()

        # Check verification state
        state_file = self._paths.completion_state
        if state_file.exists():
            try:
                state = json.loads(state_file.read_text())
                metrics["verification_approved"] = state.get("verification_agent_approved", False)
            except json.JSONDecodeError:
                metrics["verification_approved"] = False
        else:
            metrics["verification_approved"] = False

        return metrics

    def _check_recent_activity(self) -> int:
        """Count files modified in last hour."""
        count = 0
        cutoff = datetime.now() - timedelta(hours=1)

        for src_file in Path(".").rglob("*.py"):
            if "__pycache__" in str(src_file) or ".venv" in str(src_file):
                continue

            try:
                mtime = datetime.fromtimestamp(src_file.stat().st_mtime)
                if mtime > cutoff:
                    count += 1
            except OSError:
                pass

        return count

    def detect_anomalies(self, metrics: dict) -> list[dict]:
        """Detect potential issues."""
        alerts = []

        # Alert 1: Stalling (no activity but incomplete)
        if metrics.get("queue_progress"):
            progress = metrics["queue_progress"]
            if progress["percentage"] < 100 and metrics["recent_file_changes"] < 2:
                alerts.append({
                    "type": "stalling",
                    "severity": "warning",
                    "message": f"Low activity ({metrics['recent_file_changes']} files) but only {progress['percentage']:.1f}% complete",
                    "recommendation": "Check if orchestration has stopped prematurely"
                })

        # Alert 2: Completion report exists but not verified
        if metrics.get("completion_report_exists"):
            if not metrics.get("verification_approved"):
                alerts.append({
                    "type": "unverified_completion",
                    "severity": "critical",
                    "message": "COMPLETION-REPORT.md exists but verification agent has not approved",
                    "recommendation": "Delete report and run full verification first"
                })

        # Alert 3: Progress regression
        if self.metrics_file.exists():
            try:
                old_metrics = json.loads(self.metrics_file.read_text())
                old_progress = old_metrics.get("queue_progress", {}).get("percentage", 0)
                new_progress = metrics.get("queue_progress", {}).get("percentage", 0)

                if new_progress < old_progress:
                    alerts.append({
                        "type": "regression",
                        "severity": "error",
                        "message": f"Progress decreased from {old_progress:.1f}% to {new_progress:.1f}%",
                        "recommendation": "Check for task queue corruption or improper resets"
                    })
            except (json.JSONDecodeError, KeyError):
                pass

        # Alert 4: Tasks in progress for too long
        try:
            from tasks.queue import TaskQueue
            queue = TaskQueue()
            for task in queue.tasks.values():
                if task.status.value == "in_progress" and task.started_at:
                    started = datetime.fromisoformat(task.started_at)
                    duration = datetime.now() - started
                    if duration > timedelta(hours=2):
                        alerts.append({
                            "type": "task_stuck",
                            "severity": "warning",
                            "message": f"Task {task.id} in progress for {duration.total_seconds()/3600:.1f} hours",
                            "recommendation": "Check if task is blocked or needs attention"
                        })
        except (ImportError, Exception):
            pass

        return alerts

    def run_check(self):
        """Run a single monitoring check."""
        metrics = self.collect_metrics()
        alerts = self.detect_anomalies(metrics)

        # Save metrics
        self.metrics_file.parent.mkdir(parents=True, exist_ok=True)
        self.metrics_file.write_text(json.dumps(metrics, indent=2))

        # Save alerts
        if self.alerts_file.exists():
            try:
                all_alerts = json.loads(self.alerts_file.read_text())
            except json.JSONDecodeError:
                all_alerts = []
        else:
            all_alerts = []

        for alert in alerts:
            alert["timestamp"] = datetime.now().isoformat()
            all_alerts.append(alert)

        # Keep only last 100 alerts
        all_alerts = all_alerts[-100:]
        self.alerts_file.write_text(json.dumps(all_alerts, indent=2))

        # Print alerts
        if alerts:
            print("=" * 60)
            print("MONITORING ALERTS")
            print("=" * 60)
            for alert in alerts:
                print(f"[{alert['severity'].upper()}] {alert['type']}")
                print(f"  {alert['message']}")
                print(f"  Recommendation: {alert['recommendation']}")
                print("")
        else:
            print("No anomalies detected")
            print(f"Metrics saved to: {self.metrics_file}")

        return alerts

    def start_continuous_monitoring(self, interval_minutes: int = 5):
        """Run monitoring continuously."""
        print(f"Starting continuous monitoring (interval: {interval_minutes} min)")

        while True:
            print(f"\n[{datetime.now().isoformat()}] Running check...")
            self.run_check()
            time.sleep(interval_minutes * 60)

    def get_status_summary(self) -> dict:
        """Get a summary of current status."""
        metrics = self.collect_metrics()

        summary = {
            "timestamp": metrics["timestamp"],
            "queue_progress": metrics.get("queue_progress", {}).get("percentage", 0),
            "gates_passed": metrics.get("gates_passed", 0),
            "recent_activity": metrics.get("recent_file_changes", 0),
            "completion_report_exists": metrics.get("completion_report_exists", False),
            "verification_approved": metrics.get("verification_approved", False),
        }

        return summary


if __name__ == "__main__":
    monitor = OrchestratorMonitor()

    if len(sys.argv) > 1:
        if sys.argv[1] == "--continuous":
            interval = int(sys.argv[2]) if len(sys.argv) > 2 else 5
            monitor.start_continuous_monitoring(interval)
        elif sys.argv[1] == "--status":
            summary = monitor.get_status_summary()
            print(json.dumps(summary, indent=2))
        else:
            print("Usage: monitor.py [--continuous [interval_minutes] | --status]")
    else:
        monitor.run_check()
