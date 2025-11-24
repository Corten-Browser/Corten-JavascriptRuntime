#!/usr/bin/env python3
"""
Automatically sync task queue with actual implementation state.
Detects STARTED tasks (not completed) without relying on model self-reporting.

CRITICAL: This module marks tasks as INCOMPLETE, never as COMPLETED.
COMPLETED status requires verified test pass via /orchestrate run.
"""
import json
import re
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def load_queue_state() -> dict:
    """Load queue state."""
    queue_file = _paths.queue_state
    if queue_file.exists():
        try:
            return json.loads(queue_file.read_text())
        except json.JSONDecodeError:
            return {"tasks": []}
    return {"tasks": []}


def save_queue_state(state: dict):
    """Save queue state."""
    queue_file = _paths.queue_state
    queue_file.parent.mkdir(parents=True, exist_ok=True)
    state["last_updated"] = datetime.now().isoformat()
    queue_file.write_text(json.dumps(state, indent=2))


def extract_keywords(text: str) -> list[str]:
    """Extract keywords from task name."""
    stopwords = {'the', 'a', 'an', 'for', 'with', 'to', 'of', 'and', 'or', 'implement'}
    words = re.findall(r'\b\w+\b', text.lower())
    return [w for w in words if w not in stopwords and len(w) > 2]


def task_appears_started(task: dict, project_dir: Path) -> bool:
    """
    Heuristically check if work has been started on a task.
    Conservative - requires evidence of both implementation and tests.

    NOTE: This does NOT verify the task is complete or working.
    It only detects that work has begun.
    """
    task_name = task.get("name", "")
    keywords = extract_keywords(task_name)

    if not keywords:
        return False

    # Check 1: Look for tests related to task
    test_count = 0
    for test_file in project_dir.rglob("*test*.py"):
        if "__pycache__" in str(test_file):
            continue
        try:
            content = test_file.read_text()
            # Check if keywords appear in test names
            for keyword in keywords:
                if re.search(rf'def test.*{keyword}', content, re.I):
                    test_count += 1
        except Exception:
            pass

    # Check 2: Look for implementation code
    impl_files = 0
    for src_file in project_dir.rglob("*.py"):
        if "__pycache__" in str(src_file) or "test" in str(src_file).lower():
            continue
        try:
            content = src_file.read_text()
            keyword_matches = sum(
                1 for kw in keywords
                if re.search(rf'\b{kw}\b', content, re.I)
            )
            if keyword_matches >= len(keywords) * 0.5:
                impl_files += 1
        except Exception:
            pass

    # Conservative: Need both tests AND implementation
    return test_count >= 1 and impl_files >= 1


def sync_queue_with_reality(project_dir: Path = None):
    """
    Automatically detect tasks where work has started.
    Marks as INCOMPLETE (not COMPLETED).

    CRITICAL: This function NEVER marks tasks as COMPLETED.
    COMPLETED status requires verified test pass via /orchestrate run.
    """
    if project_dir is None:
        project_dir = Path.cwd()

    state = load_queue_state()
    tasks = state.get("tasks", [])

    if not tasks:
        print("No tasks in queue")
        return

    synced_count = 0

    for task in tasks:
        if task.get("status") == "pending":
            # Check if work appears to have started
            if task_appears_started(task, project_dir):
                print(f"Work detected: {task['id']} - {task['name']}")
                task["status"] = "incomplete"  # NOT completed - needs verification
                task["detection_timestamp"] = datetime.now().isoformat()
                task["detection_result"] = {
                    "detected_via": "auto_sync",
                    "verified": False,  # NOT verified - just detected
                    "note": "Code/tests detected. Requires /orchestrate verification for completion."
                }
                synced_count += 1

    if synced_count > 0:
        save_queue_state(state)
        print(f"Marked {synced_count} task(s) as INCOMPLETE (work detected)")
    else:
        print("No started work detected")


def main():
    """Run auto-sync."""
    print("=" * 60)
    print("TASK QUEUE AUTO-SYNC")
    print("=" * 60)
    print("")
    print("NOTE: This detects started work and marks as INCOMPLETE.")
    print("      COMPLETED status requires /orchestrate verification.")
    print("")

    project_dir = Path.cwd()
    if len(sys.argv) > 1:
        project_dir = Path(sys.argv[1])

    sync_queue_with_reality(project_dir)

    # Show updated status
    state = load_queue_state()
    tasks = state.get("tasks", [])
    if tasks:
        completed = sum(1 for t in tasks if t.get("status") == "completed")
        incomplete = sum(1 for t in tasks if t.get("status") == "incomplete")
        pending = sum(1 for t in tasks if t.get("status") == "pending")
        print(f"\nQueue status:")
        print(f"  Completed:  {completed}/{len(tasks)} (verified)")
        print(f"  Incomplete: {incomplete}/{len(tasks)} (needs verification)")
        print(f"  Pending:    {pending}/{len(tasks)} (no work detected)")

    print("=" * 60)


if __name__ == "__main__":
    main()
