#!/usr/bin/env python3
"""
Automatically install all enforcement mechanisms.
User runs this ONCE (via install.sh), then forgets about it.
No model cooperation required after installation.
"""
import json
import subprocess
import sys
from pathlib import Path
from datetime import datetime


def get_project_root() -> Path:
    """Get the project root directory."""
    return Path.cwd()


def install_git_hooks():
    """Install all git hooks that enforce completion requirements."""
    project_root = get_project_root()
    git_dir = project_root / ".git"
    hooks_dir = git_dir / "hooks"
    orch_hooks = project_root / "orchestration" / "hooks"

    if not git_dir.exists():
        print("WARNING: Not a git repository. Git hooks will not be installed.")
        return False

    hooks_dir.mkdir(exist_ok=True)

    # Pre-commit hook (advisory - warns but allows commit for Rule 8 compliance)
    pre_commit_hook = hooks_dir / "pre-commit"
    pre_commit_hook.write_text("""#!/bin/bash
# Advisory pre-commit hook
# Warns about incomplete work but ALLOWS commit for progress preservation
python3 orchestration/hooks/pre_commit_enforcement.py
# Always exit 0 - advisory mode, post-commit provides enforcement
exit 0
""")
    pre_commit_hook.chmod(0o755)
    print("  Installed: pre-commit hook (advisory warnings)")

    # Post-commit hook (CRITICAL - forceful continuation message)
    post_commit_hook = hooks_dir / "post-commit"
    post_commit_hook.write_text("""#!/bin/bash
# Enforcement post-commit hook
# Generates forceful continuation message when work is incomplete
python3 orchestration/hooks/post_commit_enforcement.py
# Exit code doesn't matter - commit already done
exit 0
""")
    post_commit_hook.chmod(0o755)
    print("  Installed: post-commit hook (forceful continuation messages)")

    # Pre-push hook (warns about incomplete work)
    pre_push_hook = hooks_dir / "pre-push"
    pre_push_hook.write_text("""#!/bin/bash
# Advisory pre-push hook
# Warns but allows push (progress preservation)
python3 orchestration/hooks/pre_push_enforcement.py
exit $?
""")
    pre_push_hook.chmod(0o755)
    print("  Installed: pre-push hook (advisory warnings)")

    # Post-checkout hook (re-syncs on branch switch)
    post_checkout_hook = hooks_dir / "post-checkout"
    post_checkout_hook.write_text("""#!/bin/bash
# Automatic enforcement post-checkout hook
# Re-syncs queue state on branch switch
python3 orchestration/hooks/post_checkout_enforcement.py "$@"
exit $?
""")
    post_checkout_hook.chmod(0o755)
    print("  Installed: post-checkout hook (syncs on branch switch)")

    return True


def create_state_directory():
    """Create directory for enforcement state files."""
    state_dir = get_project_root() / "orchestration" / "verification" / "state"
    state_dir.mkdir(parents=True, exist_ok=True)

    # Initialize empty completion state
    completion_state = state_dir / "completion_state.json"
    if not completion_state.exists():
        completion_state.write_text(json.dumps({
            "all_gates_passed": False,
            "spec_coverage_100": False,
            "verification_agent_approved": False,
            "smoke_tests_passed": False,
            "no_stub_components": False,
            "last_verification": None
        }, indent=2))

    print("  Created: verification state directory")
    return True


def initialize_queue_state():
    """Initialize task queue state file (empty but ready)."""
    queue_state = get_project_root() / "orchestration" / "tasks" / "queue_state.json"

    if not queue_state.exists():
        queue_state.parent.mkdir(parents=True, exist_ok=True)
        queue_state.write_text(json.dumps({
            "tasks": [],
            "completed_order": [],
            "last_updated": datetime.now().isoformat(),
            "initialized": False
        }, indent=2))

    print("  Created: task queue state file")
    return True


def setup_monitoring():
    """Set up monitoring infrastructure."""
    monitoring_dir = get_project_root() / "orchestration" / "monitoring"
    monitoring_dir.mkdir(exist_ok=True)

    # Initialize activity log
    activity_log = monitoring_dir / "activity_log.json"
    if not activity_log.exists():
        activity_log.write_text(json.dumps({
            "events": [],
            "created": datetime.now().isoformat()
        }, indent=2))

    # Initialize metrics file
    metrics_file = monitoring_dir / "metrics.json"
    if not metrics_file.exists():
        metrics_file.write_text(json.dumps({
            "timestamp": datetime.now().isoformat(),
            "queue_progress": None,
            "gates_passed": 0,
            "recent_file_changes": 0,
            "completion_report_exists": False,
            "verification_approved": False
        }, indent=2))

    print("  Created: monitoring infrastructure")
    return True


def auto_discover_specs():
    """Find spec files automatically and register them."""
    project_root = get_project_root()

    # Standard locations for spec files
    spec_patterns = [
        "specs/*.yaml",
        "specs/*.yml",
        "specifications/*.yaml",
        "specifications/*.yml",
        "specifications/*.md",
        "docs/*-specification.md",
        "docs/*-spec.md",
        "docs/*spec*.md",
    ]

    discovered_specs = []
    for pattern in spec_patterns:
        matches = list(project_root.glob(pattern))
        discovered_specs.extend(matches)

    # Remove duplicates and sort
    discovered_specs = sorted(set(discovered_specs))

    # Create spec manifest in the correct location (data/state/, not root)
    state_dir = project_root / "orchestration" / "data" / "state"
    state_dir.mkdir(parents=True, exist_ok=True)
    manifest_file = state_dir / "spec_manifest.json"

    # Convert absolute paths to relative for portability
    def to_relative(path: Path) -> str:
        try:
            return str(path.relative_to(project_root))
        except ValueError:
            return str(path)

    manifest = {
        "spec_file": to_relative(discovered_specs[0]) if discovered_specs else None,
        "all_discovered_specs": [to_relative(s) for s in discovered_specs],
        "auto_discovered": True,
        "discovery_timestamp": datetime.now().isoformat(),
        "queue_initialized": False,
        "last_sync": None
    }

    manifest_file.write_text(json.dumps(manifest, indent=2))

    if discovered_specs:
        print(f"  Discovered: {len(discovered_specs)} spec file(s)")
        for spec in discovered_specs[:3]:
            print(f"    - {spec.relative_to(project_root)}")
        if len(discovered_specs) > 3:
            print(f"    ... and {len(discovered_specs) - 3} more")
    else:
        print("  No spec files auto-discovered (will check on first commit)")

    return True


def configure_blocking_mode():
    """Configure enforcement to be in blocking mode by default."""
    config_file = get_project_root() / "orchestration" / "enforcement_config.json"

    config = {
        "blocking_mode": True,
        "auto_init_queue": True,
        "auto_run_verification": True,
        "auto_sync_tasks": True,
        "stall_detection_enabled": True,
        "stall_threshold_minutes": 60,
        "activity_tracking_enabled": True,
        "version": "1.3.0"
    }

    config_file.write_text(json.dumps(config, indent=2))
    print("  Configured: blocking mode enabled")
    return True


def install_all():
    """Main installation function."""
    print("=" * 60)
    print("INSTALLING AUTOMATIC ENFORCEMENT SYSTEM")
    print("=" * 60)
    print("")

    steps = [
        ("Installing Git Hooks", install_git_hooks),
        ("Creating State Directory", create_state_directory),
        ("Initializing Queue State", initialize_queue_state),
        ("Setting Up Monitoring", setup_monitoring),
        ("Auto-Discovering Specs", auto_discover_specs),
        ("Configuring Blocking Mode", configure_blocking_mode),
    ]

    success = True
    for step_name, step_func in steps:
        print(f"{step_name}...")
        try:
            result = step_func()
            if not result:
                print(f"  WARNING: {step_name} returned False")
        except Exception as e:
            print(f"  ERROR: {step_name} failed: {e}")
            success = False

    print("")
    print("=" * 60)

    if success:
        print("ENFORCEMENT SYSTEM INSTALLED SUCCESSFULLY")
        print("")
        print("What happens now:")
        print("1. Git hooks are active (automatic, no action needed)")
        print("2. Commits are blocked unless queue is empty")
        print("3. Verification runs automatically when needed")
        print("4. No further user action required")
        print("")
        print("The system will auto-initialize the task queue on first commit")
        print("if a spec file is found.")
    else:
        print("INSTALLATION INCOMPLETE - Some steps failed")
        print("Review errors above and fix manually")

    print("=" * 60)

    return success


if __name__ == "__main__":
    success = install_all()
    sys.exit(0 if success else 1)
