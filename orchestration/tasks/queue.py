#!/usr/bin/env python3
"""
Task queue that provides the AUTHORITATIVE work list.
Model CANNOT redefine tasks or skip them.
"""
import json
import sys
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass, asdict, field
from enum import Enum

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths


class TaskStatus(Enum):
    PENDING = "pending"           # No work detected
    INCOMPLETE = "incomplete"     # Work started, not verified complete (NEW)
    IN_PROGRESS = "in_progress"   # Being actively worked on now
    COMPLETED = "completed"       # Verified complete via passing tests
    BLOCKED = "blocked"           # Cannot proceed due to dependency


@dataclass
class Task:
    """A single task in the queue."""
    id: str
    name: str
    description: str
    feature_id: str  # Links to spec feature
    dependencies: list[str] = field(default_factory=list)  # Task IDs this depends on
    status: TaskStatus = TaskStatus.PENDING
    started_at: str = None
    completed_at: str = None
    verification_result: dict = None


class TaskQueue:
    """
    Authoritative task queue. The queue is truth, not model's judgment.
    """

    def __init__(self, state_file: Path = None):
        self._paths = DataPaths()
        self.state_file = state_file or self._paths.queue_state
        self.tasks: dict[str, Task] = {}
        self.completed_order: list[str] = []

        self._load_state()

    def _load_state(self):
        """Load queue state from disk."""
        if self.state_file.exists():
            try:
                data = json.loads(self.state_file.read_text())
                for task_data in data.get("tasks", []):
                    task_data["status"] = TaskStatus(task_data["status"])
                    # Handle missing fields for backward compatibility
                    if "dependencies" not in task_data:
                        task_data["dependencies"] = []
                    self.tasks[task_data["id"]] = Task(**task_data)
                self.completed_order = data.get("completed_order", [])
            except (json.JSONDecodeError, KeyError, TypeError):
                # Reset on corruption
                self.tasks = {}
                self.completed_order = []

    def _save_state(self):
        """Persist queue state to disk."""
        data = {
            "tasks": [
                {**asdict(task), "status": task.status.value}
                for task in self.tasks.values()
            ],
            "completed_order": self.completed_order,
            "last_updated": datetime.now().isoformat()
        }
        self.state_file.parent.mkdir(parents=True, exist_ok=True)
        self.state_file.write_text(json.dumps(data, indent=2))

    def add_task(self, task: Task):
        """Add task to queue."""
        if task.id in self.tasks:
            raise ValueError(f"Task {task.id} already exists")
        self.tasks[task.id] = task
        self._save_state()

    def get_next_task(self) -> Task | None:
        """
        Get next available task.
        Returns None if no tasks available (all complete or blocked).

        Priority order:
        1. INCOMPLETE tasks (work detected, needs verification)
        2. PENDING tasks (no work detected yet)
        """
        # First check for INCOMPLETE tasks (partially done work needs attention)
        for task in self.tasks.values():
            if task.status == TaskStatus.INCOMPLETE:
                # Check dependencies
                deps_met = all(
                    self.tasks[dep_id].status == TaskStatus.COMPLETED
                    for dep_id in task.dependencies
                    if dep_id in self.tasks
                )
                if deps_met:
                    return task

        # Then check PENDING tasks
        for task in self.tasks.values():
            if task.status == TaskStatus.PENDING:
                # Check dependencies
                deps_met = all(
                    self.tasks[dep_id].status == TaskStatus.COMPLETED
                    for dep_id in task.dependencies
                    if dep_id in self.tasks
                )

                if deps_met:
                    return task

        return None

    def start_task(self, task_id: str):
        """Mark task as in progress."""
        if task_id not in self.tasks:
            raise ValueError(f"Task {task_id} not found")

        task = self.tasks[task_id]
        task.status = TaskStatus.IN_PROGRESS
        task.started_at = datetime.now().isoformat()
        self._save_state()

    def complete_task(self, task_id: str, verification_result: dict = None):
        """Mark task as completed."""
        if task_id not in self.tasks:
            raise ValueError(f"Task {task_id} not found")

        task = self.tasks[task_id]
        task.status = TaskStatus.COMPLETED
        task.completed_at = datetime.now().isoformat()
        task.verification_result = verification_result
        self.completed_order.append(task_id)
        self._save_state()

    def is_complete(self) -> bool:
        """Check if all tasks are completed."""
        return all(
            task.status == TaskStatus.COMPLETED
            for task in self.tasks.values()
        )

    def get_progress(self) -> dict:
        """Get current progress statistics."""
        total = len(self.tasks)
        completed = sum(1 for t in self.tasks.values() if t.status == TaskStatus.COMPLETED)
        in_progress = sum(1 for t in self.tasks.values() if t.status == TaskStatus.IN_PROGRESS)
        incomplete = sum(1 for t in self.tasks.values() if t.status == TaskStatus.INCOMPLETE)
        pending = sum(1 for t in self.tasks.values() if t.status == TaskStatus.PENDING)
        blocked = sum(1 for t in self.tasks.values() if t.status == TaskStatus.BLOCKED)

        return {
            "total": total,
            "completed": completed,
            "in_progress": in_progress,
            "incomplete": incomplete,
            "pending": pending,
            "blocked": blocked,
            "percentage": (completed / total * 100) if total > 0 else 0
        }

    def get_remaining_tasks(self) -> list[Task]:
        """Get all tasks not yet completed."""
        return [
            task for task in self.tasks.values()
            if task.status != TaskStatus.COMPLETED
        ]

    def reset(self):
        """Reset queue to initial state."""
        for task in self.tasks.values():
            task.status = TaskStatus.PENDING
            task.started_at = None
            task.completed_at = None
            task.verification_result = None
        self.completed_order = []
        self._save_state()

    def clear(self):
        """Clear all tasks from queue."""
        self.tasks = {}
        self.completed_order = []
        self._save_state()

    def print_status(self):
        """Print current queue status."""
        progress = self.get_progress()

        print("=" * 60)
        print("TASK QUEUE STATUS")
        print("=" * 60)
        print(f"Progress: {progress['percentage']:.1f}% ({progress['completed']}/{progress['total']})")
        print(f"  Completed: {progress['completed']}")
        print(f"  In Progress: {progress['in_progress']}")
        print(f"  Incomplete: {progress['incomplete']}")
        print(f"  Pending: {progress['pending']}")
        print(f"  Blocked: {progress['blocked']}")
        print("")

        if not self.is_complete():
            next_task = self.get_next_task()
            if next_task:
                print(f"NEXT TASK: {next_task.id}")
                print(f"  Name: {next_task.name}")
                print(f"  Feature: {next_task.feature_id}")
                print(f"  Status: {next_task.status.value}")
            else:
                print("NO TASKS AVAILABLE (check dependencies or blockages)")
        else:
            print("ALL TASKS COMPLETED")

        print("=" * 60)

    def reset_to_incomplete(self, task_id: str, reason: str = None):
        """
        Reset a single task to INCOMPLETE status.

        Used for recovery from crashes or manual reset.

        Args:
            task_id: Task to reset
            reason: Optional reason for the reset
        """
        if task_id not in self.tasks:
            raise ValueError(f"Task {task_id} not found")

        task = self.tasks[task_id]
        old_status = task.status.value
        task.status = TaskStatus.INCOMPLETE
        task.started_at = None  # Clear - not currently being worked on
        task.completed_at = None
        task.verification_result = {
            "reset": True,
            "reset_timestamp": datetime.now().isoformat(),
            "previous_status": old_status,
            "reason": reason or "Manual reset"
        }
        self._save_state()

    def reset_all_to_incomplete(self, include_statuses: list[str] = None) -> int:
        """
        Reset multiple tasks to INCOMPLETE status.

        Args:
            include_statuses: List of status values to reset.
                             Default: ["completed", "blocked", "in_progress"]

        Returns:
            Number of tasks reset
        """
        if include_statuses is None:
            include_statuses = ["completed", "blocked", "in_progress"]

        reset_count = 0
        reset_timestamp = datetime.now().isoformat()

        for task in self.tasks.values():
            if task.status.value in include_statuses:
                old_status = task.status.value
                task.status = TaskStatus.INCOMPLETE
                task.started_at = None
                task.completed_at = None
                task.verification_result = {
                    "reset": True,
                    "reset_timestamp": reset_timestamp,
                    "previous_status": old_status,
                    "reason": "Bulk reset via --reset flag"
                }
                reset_count += 1

        if reset_count > 0:
            self.completed_order = []  # Clear completion order
            self._save_state()

        return reset_count


if __name__ == "__main__":
    # Demo usage
    queue = TaskQueue()
    queue.print_status()
