#!/usr/bin/env python3
"""
ADVISORY pre-commit hook.
Runs automatically on EVERY commit attempt.
No model decision required - git triggers it.

This hook ALLOWS commits (for progress preservation per Rule 8).
Post-commit hook provides forceful continuation messages.
"""
import json
import subprocess
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def load_config() -> dict:
    """Load enforcement configuration."""
    config_file = _paths.enforcement_config
    if config_file.exists():
        return json.loads(config_file.read_text())
    return {"blocking_mode": True, "auto_init_queue": True}


def load_spec_manifest() -> dict:
    """Load spec manifest."""
    manifest_file = _paths.spec_manifest
    if manifest_file.exists():
        return json.loads(manifest_file.read_text())
    return {"spec_file": None, "queue_initialized": False}


def load_queue_state() -> dict:
    """Load task queue state."""
    queue_file = _paths.queue_state
    if queue_file.exists():
        try:
            return json.loads(queue_file.read_text())
        except json.JSONDecodeError:
            return {"tasks": [], "initialized": False}
    return {"tasks": [], "initialized": False}


def load_completion_state() -> dict:
    """Load completion/verification state."""
    state_file = _paths.completion_state
    if state_file.exists():
        try:
            return json.loads(state_file.read_text())
        except json.JSONDecodeError:
            return {}
    return {}


def queue_is_empty() -> bool:
    """Check if all tasks in queue are completed."""
    state = load_queue_state()
    tasks = state.get("tasks", [])

    if not tasks:
        # No tasks defined = queue empty (or not initialized)
        return True

    return all(t.get("status") == "completed" for t in tasks)


def get_remaining_tasks() -> list[dict]:
    """Get list of incomplete tasks."""
    state = load_queue_state()
    tasks = state.get("tasks", [])
    return [t for t in tasks if t.get("status") != "completed"]


def get_current_task() -> dict:
    """Get the current task to work on."""
    state = load_queue_state()
    tasks = state.get("tasks", [])

    # Find in-progress task
    for task in tasks:
        if task.get("status") == "in_progress":
            return task

    # Find first pending task
    for task in tasks:
        if task.get("status") == "pending":
            return task

    return None


def auto_initialize_queue():
    """Auto-initialize queue from spec if not already done."""
    manifest = load_spec_manifest()

    if not manifest.get("spec_file"):
        return False

    if manifest.get("queue_initialized"):
        return True

    spec_file = Path(manifest["spec_file"])
    if not spec_file.exists():
        return False

    # Try to initialize queue
    try:
        result = subprocess.run(
            ["python3", "orchestration/auto_init.py", str(spec_file)],
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode == 0:
            # Update manifest
            manifest["queue_initialized"] = True
            manifest["last_sync"] = datetime.now().isoformat()
            _paths.spec_manifest.write_text(
                json.dumps(manifest, indent=2)
            )
            print("  Auto-initialized task queue from spec")
            return True
    except Exception as e:
        print(f"  Warning: Failed to auto-initialize queue: {e}")

    return False


def auto_run_verification() -> bool:
    """Auto-run verification agent."""
    try:
        result = subprocess.run(
            ["python3", "orchestration/verification/run_full_verification.py"],
            capture_output=True,
            text=True,
            timeout=120
        )

        if "VERIFIED COMPLETE" in result.stdout or "VERIFICATION PASSED" in result.stdout:
            return True

        print("Verification output:")
        print(result.stdout[-2000:] if len(result.stdout) > 2000 else result.stdout)

        return False
    except Exception as e:
        print(f"Verification error: {e}")
        return False


def check_for_rationalization_in_diff() -> list[str]:
    """Check staged changes for rationalization language."""
    try:
        result = subprocess.run(
            ["git", "diff", "--cached"],
            capture_output=True,
            text=True
        )
        diff_text = result.stdout

        # Critical phrases to block
        blocking_phrases = [
            "substantially complete",
            "known limitations",
            "future work",
            "phase 1 complete",
            "foundation established",
            "implementation pending",
            "awaiting approval",
            "milestone reached",
            "ready for review",
        ]

        found = []
        for phrase in blocking_phrases:
            if phrase.lower() in diff_text.lower():
                found.append(phrase)

        return found
    except Exception:
        return []


def check_for_stubs_in_diff() -> bool:
    """Check staged changes for stub code."""
    try:
        result = subprocess.run(
            ["git", "diff", "--cached"],
            capture_output=True,
            text=True
        )
        diff_text = result.stdout

        stub_patterns = [
            "raise NotImplementedError",
            "TODO: implement",
            "pass  # stub",
            "unimplemented!()",
            "todo!()",
        ]

        for pattern in stub_patterns:
            if pattern in diff_text:
                return True

        return False
    except Exception:
        return False


def log_activity(event_type: str, details: dict):
    """Log activity for monitoring."""
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
        # Keep last 1000 events
        log["events"] = log["events"][-1000:]
        log_file.write_text(json.dumps(log, indent=2))
    except Exception:
        pass


def main():
    """
    Advisory pre-commit check.

    IMPORTANT: This hook ALLOWS commits to proceed for Rule 8 compliance
    (progress preservation). Post-commit hook provides forceful continuation
    messages when work is incomplete.

    This is advisory mode - warns but doesn't block.
    """
    print("=" * 60)
    print("ENFORCEMENT PRE-COMMIT CHECK (Advisory Mode)")
    print("=" * 60)
    print("")

    config = load_config()

    if not config.get("blocking_mode", True):
        print("Enforcement disabled - commit proceeding")
        sys.exit(0)

    manifest = load_spec_manifest()
    warnings = []

    # Check 1: Auto-initialize queue if spec exists
    if manifest.get("spec_file") and config.get("auto_init_queue", True):
        if not manifest.get("queue_initialized"):
            print("Auto-initializing task queue from spec...")
            auto_initialize_queue()

    # Check 2: Queue status (advisory, not blocking)
    queue_state = load_queue_state()
    tasks = queue_state.get("tasks", [])

    if tasks:  # Queue has tasks defined
        if not queue_is_empty():
            remaining = get_remaining_tasks()
            current = get_current_task()

            warnings.append(f"Queue not empty: {len(remaining)}/{len(tasks)} tasks remaining")
            if current:
                warnings.append(f"Current task: {current.get('id')} - {current.get('name')}")

            log_activity("pre_commit_warning", {
                "warning": "queue_not_empty",
                "remaining_tasks": len(remaining)
            })

        else:
            # Queue is empty - check verification
            completion_state = load_completion_state()
            if not completion_state.get("verification_agent_approved", False):
                warnings.append("Queue empty but verification not approved")
                log_activity("pre_commit_warning", {
                    "warning": "verification_not_approved"
                })

    # Check 3: Rationalization language (advisory warning)
    rationalization = check_for_rationalization_in_diff()
    if rationalization:
        warnings.append(f"Rationalization phrases detected: {', '.join(rationalization)}")
        log_activity("pre_commit_warning", {
            "warning": "rationalization_language",
            "phrases": rationalization
        })

    # Check 4: Stub code (advisory warning)
    if check_for_stubs_in_diff():
        warnings.append("Stub/placeholder code detected in changes")
        log_activity("pre_commit_warning", {
            "warning": "stub_code"
        })

    # Show warnings but ALLOW commit
    if warnings:
        print("⚠️  WARNINGS (commit will proceed for progress preservation):")
        print("")
        for warning in warnings:
            print(f"  • {warning}")
        print("")
        print("Commit proceeding to preserve progress (Rule 8).")
        print("Post-commit hook will provide continuation instructions.")
        print("")
    else:
        print("All checks passed.")
        print("")

    print("Commit ALLOWED")
    print("=" * 60)

    log_activity("commit_allowed_advisory", {
        "queue_empty": queue_is_empty(),
        "tasks_defined": len(tasks),
        "warnings_count": len(warnings),
        "warnings": warnings
    })

    # CRITICAL: Always exit 0 to allow commit
    # Progress preservation (Rule 8) takes priority
    sys.exit(0)


if __name__ == "__main__":
    main()
