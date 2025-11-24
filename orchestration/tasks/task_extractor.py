#!/usr/bin/env python3
"""
Extract tasks from machine-readable specification.
Creates objective, trackable work items.
"""
import yaml
from pathlib import Path
from .queue import TaskQueue, Task, TaskStatus


def extract_tasks_from_spec(spec_file: Path) -> TaskQueue:
    """
    Convert specification features into executable tasks.
    """
    with open(spec_file) as f:
        spec = yaml.safe_load(f)

    queue = TaskQueue()
    queue.clear()  # Start fresh

    # Group features by phase if phases exist
    if "phases" in spec:
        for phase in spec["phases"]:
            phase_num = phase["number"]
            phase_name = phase["name"]

            for feature_id in phase["features"]:
                # Find feature details
                feature = next(
                    (f for f in spec["features"] if f["id"] == feature_id),
                    None
                )

                if not feature:
                    continue

                # Create task for this feature
                task = Task(
                    id=f"TASK-{feature_id}",
                    name=f"Implement {feature['name']}",
                    description=feature.get("description", feature["name"]),
                    feature_id=feature_id,
                    dependencies=[
                        f"TASK-{dep}"
                        for dep in feature.get("dependencies", [])
                    ]
                )

                queue.add_task(task)
    else:
        # No phases, just extract all features as tasks
        for feature in spec["features"]:
            if not feature.get("required", True):
                continue

            task = Task(
                id=f"TASK-{feature['id']}",
                name=f"Implement {feature['name']}",
                description=feature.get("description", feature["name"]),
                feature_id=feature["id"],
                dependencies=[
                    f"TASK-{dep}"
                    for dep in feature.get("dependencies", [])
                ]
            )

            queue.add_task(task)

    return queue


def main():
    import sys

    if len(sys.argv) < 2:
        print("Usage: python task_extractor.py <spec.yaml>")
        sys.exit(1)

    spec_file = Path(sys.argv[1])

    if not spec_file.exists():
        print(f"ERROR: Spec file not found: {spec_file}")
        sys.exit(1)

    queue = extract_tasks_from_spec(spec_file)

    print(f"Extracted {len(queue.tasks)} tasks from specification")
    queue.print_status()


if __name__ == "__main__":
    main()
