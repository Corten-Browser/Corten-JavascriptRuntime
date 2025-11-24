#!/usr/bin/env python3
"""
Automatically sync task queue with actual implementation state.
Detects completed tasks without relying on model self-reporting.
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


def task_appears_complete(task: dict, project_dir: Path) -> bool:
    """
    Heuristically check if a task appears to be complete.
    Conservative - requires strong evidence.
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
    Automatically detect and mark completed tasks.
    Does NOT rely on model claiming completion.
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
            # Check if task appears complete
            if task_appears_complete(task, project_dir):
                print(f"Auto-detected completion: {task['id']} - {task['name']}")
                task["status"] = "completed"
                task["completed_at"] = datetime.now().isoformat()
                task["verification_result"] = {"auto_detected": True}
                synced_count += 1

    if synced_count > 0:
        save_queue_state(state)
        print(f"Auto-synced {synced_count} task(s)")
    else:
        print("No auto-completions detected")


def main():
    """Run auto-sync."""
    print("=" * 60)
    print("TASK QUEUE AUTO-SYNC")
    print("=" * 60)
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
        print(f"\nQueue status: {completed}/{len(tasks)} completed")

    print("=" * 60)


if __name__ == "__main__":
    main()
