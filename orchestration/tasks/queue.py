#!/usr/bin/env python3
"""
Task queue that provides the AUTHORITATIVE work list.
Model CANNOT redefine tasks or skip them.

Extended in v1.16.0 to support LLM Extraction Specification fields:
- source: Traceability to specification file:line
- requirement_id: Links to REQ-xxx requirements
- type: Requirement type classification
- phase: Task execution phase
- external_reference: External standards/suites
- verification: Verification method and thresholds
- not_satisfied_by: Anti-bypass clauses
- acceptance_criteria: Measurable acceptance criteria
"""
import json
import sys
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass, asdict, field
from enum import Enum
from typing import Any, Optional

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths


class TaskStatus(Enum):
    PENDING = "pending"           # No work detected
    INCOMPLETE = "incomplete"     # Work started, not verified complete (NEW)
    IN_PROGRESS = "in_progress"   # Being actively worked on now
    COMPLETED = "completed"       # Verified complete via passing tests
    BLOCKED = "blocked"           # Cannot proceed due to dependency


class RequirementType(Enum):
    """Types of requirements (from LLM Extraction Specification)."""
    FEATURE = "feature"
    CONSTRAINT = "constraint"
    COMPLIANCE = "compliance"
    PERFORMANCE = "performance"
    QUALITY = "quality"
    INTEGRATION = "integration"


class TaskPhase(Enum):
    """Phases in task execution."""
    SETUP = "setup"
    IMPLEMENTATION = "implementation"
    TESTING = "testing"
    VERIFICATION = "verification"


class VerificationMethod(Enum):
    """Methods for verifying task completion."""
    UNIT_TEST = "unit_test"
    INTEGRATION_TEST = "integration_test"
    EXTERNAL_SUITE = "external_suite"
    MANUAL_REVIEW = "manual_review"
    BENCHMARK = "benchmark"
    STATIC_ANALYSIS = "static_analysis"


@dataclass
class SourceLocation:
    """Source location for traceability."""
    file: str
    line: int
    verbatim: str
    section: Optional[str] = None
    phase: Optional[str] = None

    def to_dict(self) -> dict[str, Any]:
        result = {"file": self.file, "line": self.line, "verbatim": self.verbatim}
        if self.section:
            result["section"] = self.section
        if self.phase:
            result["phase"] = self.phase
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "SourceLocation":
        return cls(
            file=data["file"],
            line=data["line"],
            verbatim=data["verbatim"],
            section=data.get("section"),
            phase=data.get("phase"),
        )


@dataclass
class ExternalReference:
    """Reference to external standard, specification, or test suite."""
    name: str
    url: str
    type: str  # conformance_suite, specification, standard, library, tool
    acronym: Optional[str] = None
    test_paths: list[str] = field(default_factory=list)
    version: Optional[str] = None

    def to_dict(self) -> dict[str, Any]:
        result = {"name": self.name, "url": self.url, "type": self.type}
        if self.acronym:
            result["acronym"] = self.acronym
        if self.test_paths:
            result["test_paths"] = self.test_paths
        if self.version:
            result["version"] = self.version
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "ExternalReference":
        return cls(
            name=data["name"],
            url=data["url"],
            type=data["type"],
            acronym=data.get("acronym"),
            test_paths=data.get("test_paths", []),
            version=data.get("version"),
        )


@dataclass
class Threshold:
    """Threshold for verification."""
    metric: str
    value: float
    operator: str  # >=, <=, ==, >, <
    unit: Optional[str] = None

    def to_dict(self) -> dict[str, Any]:
        result = {"metric": self.metric, "value": self.value, "operator": self.operator}
        if self.unit:
            result["unit"] = self.unit
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "Threshold":
        return cls(
            metric=data["metric"],
            value=data["value"],
            operator=data["operator"],
            unit=data.get("unit"),
        )

    def is_satisfied(self, actual: float) -> bool:
        ops = {">=": lambda a, b: a >= b, "<=": lambda a, b: a <= b,
               "==": lambda a, b: a == b, ">": lambda a, b: a > b, "<": lambda a, b: a < b}
        return ops.get(self.operator, lambda a, b: False)(actual, self.value)


@dataclass
class VerificationSpec:
    """Specification for how a task should be verified."""
    method: str  # VerificationMethod value
    threshold: Optional[Threshold] = None
    evidence_required: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        result = {"method": self.method}
        if self.threshold:
            result["threshold"] = self.threshold.to_dict()
        if self.evidence_required:
            result["evidence_required"] = self.evidence_required
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "VerificationSpec":
        return cls(
            method=data["method"],
            threshold=Threshold.from_dict(data["threshold"]) if data.get("threshold") else None,
            evidence_required=data.get("evidence_required", []),
        )


@dataclass
class AntiBypassClause:
    """Clause specifying what does NOT satisfy a requirement."""
    description: str
    rationale: str

    def to_dict(self) -> dict[str, Any]:
        return {"description": self.description, "rationale": self.rationale}

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "AntiBypassClause":
        return cls(description=data["description"], rationale=data["rationale"])


@dataclass
class AcceptanceCriterion:
    """Measurable acceptance criterion."""
    criterion: str
    measurable: bool
    measurement: Optional[str] = None

    def to_dict(self) -> dict[str, Any]:
        result = {"criterion": self.criterion, "measurable": self.measurable}
        if self.measurement:
            result["measurement"] = self.measurement
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "AcceptanceCriterion":
        return cls(
            criterion=data["criterion"],
            measurable=data["measurable"],
            measurement=data.get("measurement"),
        )


@dataclass
class Task:
    """
    A single task in the queue.

    Extended in v1.16.0 to support LLM Extraction Specification fields.
    All new fields are optional for backward compatibility.
    """
    id: str
    name: str
    description: str
    feature_id: str  # Links to spec feature (legacy) or requirement_id (new)
    dependencies: list[str] = field(default_factory=list)  # Task IDs this depends on
    status: TaskStatus = TaskStatus.PENDING
    started_at: str = None
    completed_at: str = None
    verification_result: dict = None

    # New fields from LLM Extraction Specification (all optional for backward compat)
    requirement_id: Optional[str] = None  # REQ-xxx ID
    source: Optional[SourceLocation] = None  # Traceability to spec file:line
    type: Optional[RequirementType] = None  # Requirement type
    phase: Optional[TaskPhase] = None  # Execution phase
    external_reference: Optional[ExternalReference] = None  # External suite/spec
    verification: Optional[VerificationSpec] = None  # Verification method
    not_satisfied_by: list[AntiBypassClause] = field(default_factory=list)  # Anti-bypass
    acceptance_criteria: list[AcceptanceCriterion] = field(default_factory=list)
    blocks: list[str] = field(default_factory=list)  # Tasks this blocks

    def is_compliance_task(self) -> bool:
        """Check if this is a compliance task requiring external verification."""
        return self.type == RequirementType.COMPLIANCE

    def requires_external_verification(self) -> bool:
        """Check if this task requires external suite verification."""
        return (
            self.verification is not None
            and self.verification.method == VerificationMethod.EXTERNAL_SUITE.value
        )

    def has_anti_bypass_clauses(self) -> bool:
        """Check if this task has anti-bypass clauses."""
        return len(self.not_satisfied_by) > 0


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

                    # Handle new LLM Extraction Specification fields (v1.16.0+)
                    # These are optional for backward compatibility with existing queue files

                    # Convert source dict to SourceLocation
                    if "source" in task_data and task_data["source"]:
                        task_data["source"] = SourceLocation.from_dict(task_data["source"])
                    else:
                        task_data["source"] = None

                    # Convert type string to RequirementType enum
                    if "type" in task_data and task_data["type"]:
                        task_data["type"] = RequirementType(task_data["type"])
                    else:
                        task_data["type"] = None

                    # Convert phase string to TaskPhase enum
                    if "phase" in task_data and task_data["phase"]:
                        task_data["phase"] = TaskPhase(task_data["phase"])
                    else:
                        task_data["phase"] = None

                    # Convert external_reference dict to ExternalReference
                    if "external_reference" in task_data and task_data["external_reference"]:
                        task_data["external_reference"] = ExternalReference.from_dict(
                            task_data["external_reference"]
                        )
                    else:
                        task_data["external_reference"] = None

                    # Convert verification dict to VerificationSpec
                    if "verification" in task_data and task_data["verification"]:
                        task_data["verification"] = VerificationSpec.from_dict(
                            task_data["verification"]
                        )
                    else:
                        task_data["verification"] = None

                    # Convert not_satisfied_by list of dicts to AntiBypassClause objects
                    if "not_satisfied_by" in task_data:
                        task_data["not_satisfied_by"] = [
                            AntiBypassClause.from_dict(c)
                            for c in task_data["not_satisfied_by"]
                        ]
                    else:
                        task_data["not_satisfied_by"] = []

                    # Convert acceptance_criteria list of dicts to AcceptanceCriterion objects
                    if "acceptance_criteria" in task_data:
                        task_data["acceptance_criteria"] = [
                            AcceptanceCriterion.from_dict(c)
                            for c in task_data["acceptance_criteria"]
                        ]
                    else:
                        task_data["acceptance_criteria"] = []

                    # Handle other new optional fields
                    if "requirement_id" not in task_data:
                        task_data["requirement_id"] = None
                    if "blocks" not in task_data:
                        task_data["blocks"] = []

                    self.tasks[task_data["id"]] = Task(**task_data)
                self.completed_order = data.get("completed_order", [])
            except (json.JSONDecodeError, KeyError, TypeError) as e:
                # Reset on corruption
                print(f"Warning: Error loading queue state: {e}")
                self.tasks = {}
                self.completed_order = []

    def _save_state(self):
        """Persist queue state to disk."""
        tasks_data = []
        for task in self.tasks.values():
            task_dict = {
                "id": task.id,
                "name": task.name,
                "description": task.description,
                "feature_id": task.feature_id,
                "dependencies": task.dependencies,
                "status": task.status.value,
                "started_at": task.started_at,
                "completed_at": task.completed_at,
                "verification_result": task.verification_result,
            }

            # Add new LLM Extraction Specification fields (v1.16.0+)
            if task.requirement_id:
                task_dict["requirement_id"] = task.requirement_id
            if task.source:
                task_dict["source"] = task.source.to_dict()
            if task.type:
                task_dict["type"] = task.type.value
            if task.phase:
                task_dict["phase"] = task.phase.value
            if task.external_reference:
                task_dict["external_reference"] = task.external_reference.to_dict()
            if task.verification:
                task_dict["verification"] = task.verification.to_dict()
            if task.not_satisfied_by:
                task_dict["not_satisfied_by"] = [c.to_dict() for c in task.not_satisfied_by]
            if task.acceptance_criteria:
                task_dict["acceptance_criteria"] = [c.to_dict() for c in task.acceptance_criteria]
            if task.blocks:
                task_dict["blocks"] = task.blocks

            tasks_data.append(task_dict)

        data = {
            "tasks": tasks_data,
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
