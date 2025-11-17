#!/usr/bin/env python3
"""
Verify that enforcement system is properly installed.
Checks all required components are in place.
"""
import json
import sys
from pathlib import Path


def verify_git_hooks() -> tuple[bool, list[str]]:
    """Verify all git hooks are installed."""
    issues = []
    git_hooks = Path(".git/hooks")

    if not git_hooks.exists():
        issues.append("Git hooks directory does not exist")
        return False, issues

    required_hooks = ["pre-commit", "pre-push", "post-checkout"]

    for hook in required_hooks:
        hook_path = git_hooks / hook
        if not hook_path.exists():
            issues.append(f"Missing hook: {hook}")
        elif not hook_path.stat().st_mode & 0o111:
            issues.append(f"Hook not executable: {hook}")
        else:
            # Check hook content references enforcement
            content = hook_path.read_text()
            if "orchestration/hooks" not in content:
                issues.append(f"Hook {hook} doesn't reference enforcement system")

    return len(issues) == 0, issues


def verify_state_files() -> tuple[bool, list[str]]:
    """Verify state files exist and are valid."""
    issues = []

    # Check completion state
    completion_state = Path("orchestration/verification/state/completion_state.json")
    if not completion_state.exists():
        issues.append("Missing: completion_state.json")
    else:
        try:
            data = json.loads(completion_state.read_text())
            required_keys = ["all_gates_passed", "verification_agent_approved"]
            for key in required_keys:
                if key not in data:
                    issues.append(f"completion_state.json missing key: {key}")
        except json.JSONDecodeError:
            issues.append("completion_state.json is not valid JSON")

    # Check queue state
    queue_state = Path("orchestration/task_queue/queue_state.json")
    if not queue_state.exists():
        issues.append("Missing: queue_state.json")

    # Check spec manifest
    manifest = Path("orchestration/spec_manifest.json")
    if not manifest.exists():
        issues.append("Missing: spec_manifest.json")

    # Check enforcement config
    config = Path("orchestration/enforcement_config.json")
    if not config.exists():
        issues.append("Missing: enforcement_config.json")
    else:
        try:
            data = json.loads(config.read_text())
            if not data.get("blocking_mode", False):
                issues.append("Blocking mode not enabled in config")
        except json.JSONDecodeError:
            issues.append("enforcement_config.json is not valid JSON")

    return len(issues) == 0, issues


def verify_enforcement_scripts() -> tuple[bool, list[str]]:
    """Verify enforcement scripts exist."""
    issues = []

    required_scripts = [
        "orchestration/hooks/pre_commit_enforcement.py",
        "orchestration/hooks/pre_push_enforcement.py",
        "orchestration/hooks/post_checkout_enforcement.py",
        "orchestration/auto_init.py",
        "orchestration/session_init.py",
        "orchestration/verification/verification_agent.py",
        "orchestration/task_queue/queue.py",
        "orchestration/task_queue/auto_sync.py",
    ]

    for script in required_scripts:
        if not Path(script).exists():
            issues.append(f"Missing script: {script}")

    return len(issues) == 0, issues


def verify_monitoring() -> tuple[bool, list[str]]:
    """Verify monitoring infrastructure."""
    issues = []

    monitoring_dir = Path("orchestration/monitoring")
    if not monitoring_dir.exists():
        issues.append("Missing: monitoring directory")
        return False, issues

    activity_log = monitoring_dir / "activity_log.json"
    if not activity_log.exists():
        issues.append("Missing: activity_log.json")

    return len(issues) == 0, issues


def main():
    """Run all verification checks."""
    print("=" * 60)
    print("VERIFYING ENFORCEMENT SYSTEM INSTALLATION")
    print("=" * 60)
    print("")

    all_passed = True

    checks = [
        ("Git Hooks", verify_git_hooks),
        ("State Files", verify_state_files),
        ("Enforcement Scripts", verify_enforcement_scripts),
        ("Monitoring Infrastructure", verify_monitoring),
    ]

    for check_name, check_func in checks:
        print(f"Checking {check_name}...")
        passed, issues = check_func()

        if passed:
            print(f"  PASSED")
        else:
            print(f"  FAILED")
            for issue in issues:
                print(f"    - {issue}")
            all_passed = False
        print("")

    print("=" * 60)
    if all_passed:
        print("VERIFICATION PASSED - All enforcement components installed")
    else:
        print("VERIFICATION FAILED - Some components missing or misconfigured")
        print("Run: python orchestration/setup/install_enforcement.py")
    print("=" * 60)

    return all_passed


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
