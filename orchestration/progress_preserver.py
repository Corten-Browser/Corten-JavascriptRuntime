#!/usr/bin/env python3
"""
Progress Preserver - Automatic Git Commit and Push at Boundaries

Ensures work is preserved to remote repository at natural boundaries:
- Phase completions
- Component completions
- Checkpoint saves
- Major milestones

This prevents loss of work in unstable environments and reduces wasted
tokens from restarting after environment failures.
"""

import json
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Optional


class ProgressPreserver:
    """
    Automatically preserves work progress via git commits and pushes.

    Key features:
    - Auto-commit at phase boundaries
    - Optional auto-push to remote
    - Safe retry mechanism for concurrent operations
    - Detailed commit messages documenting progress
    """

    def __init__(self, project_root: str, config_path: str = None):
        """
        Initialize progress preserver.

        Args:
            project_root: Path to project root
            config_path: Optional path to config file
        """
        self.project_root = Path(project_root)
        self.config = self._load_config(config_path)
        self.commit_count = 0
        self.push_count = 0

    def _load_config(self, config_path: str = None) -> dict:
        """Load configuration from file or use defaults."""
        default_config = {
            "auto_commit": True,
            "auto_push": True,  # Push to remote after commit
            "push_on_phase_complete": True,
            "push_on_component_complete": True,
            "push_on_checkpoint": False,  # Checkpoints can be frequent
            "max_retry_attempts": 5,
            "retry_delay_seconds": 2,
            "include_checkpoint_state": True,
            "commit_message_prefix": "wip:",
            "branch_prefix": "orchestration/",
        }

        if config_path:
            config_file = Path(config_path)
        else:
            config_file = self.project_root / "orchestration" / "progress_preserver_config.json"

        if config_file.exists():
            try:
                user_config = json.loads(config_file.read_text())
                default_config.update(user_config)
            except Exception as e:
                print(f"⚠️ Could not load config: {e}, using defaults")

        return default_config

    def save_config(self):
        """Save current configuration to file."""
        config_file = self.project_root / "orchestration" / "progress_preserver_config.json"
        config_file.write_text(json.dumps(self.config, indent=2))
        print(f"✓ Configuration saved to {config_file}")

    def on_phase_complete(
        self,
        phase_number: int,
        phase_name: str,
        tests_passing: int = None,
        tests_total: int = None,
        components_completed: list = None
    ) -> bool:
        """
        Called when a phase completes - commits and optionally pushes.

        Returns True if successful, False otherwise.
        """
        if not self.config["auto_commit"]:
            print("  ℹ️ Auto-commit disabled, skipping")
            return True

        # Stage all changes
        if not self._stage_changes():
            return False

        # Create detailed commit message
        message = self._create_phase_commit_message(
            phase_number, phase_name, tests_passing, tests_total, components_completed
        )

        # Commit
        if not self._commit(message):
            return False

        self.commit_count += 1
        print(f"  ✓ Phase {phase_number} progress committed ({self.commit_count} total)")

        # Push if configured
        if self.config["auto_push"] and self.config["push_on_phase_complete"]:
            if self._push():
                self.push_count += 1
                print(f"  ✓ Pushed to remote ({self.push_count} total)")
            else:
                print("  ⚠️ Push failed (work is committed locally)")

        return True

    def on_component_complete(
        self,
        component_name: str,
        tests_passing: int = None,
        test_coverage: float = None
    ) -> bool:
        """
        Called when a component completes - commits and optionally pushes.

        Returns True if successful, False otherwise.
        """
        if not self.config["auto_commit"]:
            return True

        if not self._stage_changes():
            return False

        message = self._create_component_commit_message(
            component_name, tests_passing, test_coverage
        )

        if not self._commit(message):
            return False

        self.commit_count += 1
        print(f"  ✓ Component {component_name} progress committed")

        if self.config["auto_push"] and self.config["push_on_component_complete"]:
            if self._push():
                self.push_count += 1
                print(f"  ✓ Pushed to remote")

        return True

    def on_checkpoint_save(
        self,
        checkpoint_id: str,
        context_usage_percent: float = None
    ) -> bool:
        """
        Called when checkpoint is saved - commits checkpoint state.

        Returns True if successful, False otherwise.
        """
        if not self.config["auto_commit"]:
            return True

        if not self.config["include_checkpoint_state"]:
            return True

        if not self._stage_changes():
            return False

        message = self._create_checkpoint_commit_message(checkpoint_id, context_usage_percent)

        if not self._commit(message):
            return False

        self.commit_count += 1

        if self.config["auto_push"] and self.config["push_on_checkpoint"]:
            self._push()

        return True

    def on_major_milestone(
        self,
        milestone_name: str,
        description: str = ""
    ) -> bool:
        """
        Called at major milestones - always commits and pushes.

        Returns True if successful, False otherwise.
        """
        if not self._stage_changes():
            return False

        message = f"milestone: {milestone_name}\n\n{description}"

        if not self._commit(message):
            return False

        self.commit_count += 1
        print(f"  ✓ Milestone committed: {milestone_name}")

        # Always push milestones
        if self.config["auto_push"]:
            if self._push():
                self.push_count += 1
                print(f"  ✓ Milestone pushed to remote")

        return True

    def _stage_changes(self) -> bool:
        """Stage all changes for commit."""
        try:
            result = subprocess.run(
                ["git", "add", "-A"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30
            )
            return result.returncode == 0
        except Exception as e:
            print(f"  ⚠️ Failed to stage changes: {e}")
            return False

    def _commit(self, message: str) -> bool:
        """
        Commit staged changes with retry mechanism.

        Uses exponential backoff for index.lock conflicts.
        """
        import time
        import random

        for attempt in range(self.config["max_retry_attempts"]):
            try:
                # Check if there are changes to commit
                status_result = subprocess.run(
                    ["git", "status", "--porcelain"],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=10
                )

                if not status_result.stdout.strip():
                    # No changes to commit
                    return True

                result = subprocess.run(
                    ["git", "commit", "-m", message],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                if result.returncode == 0:
                    return True

                # Check for index.lock error
                if "index.lock" in result.stderr:
                    delay = self.config["retry_delay_seconds"] * (2 ** attempt) + random.uniform(0, 1)
                    print(f"  ⚠️ Git index locked, retrying in {delay:.1f}s...")
                    time.sleep(delay)
                    continue

                print(f"  ⚠️ Commit failed: {result.stderr}")
                return False

            except subprocess.TimeoutExpired:
                print(f"  ⚠️ Commit timed out (attempt {attempt + 1})")
            except Exception as e:
                print(f"  ⚠️ Commit error: {e}")

        return False

    def _push(self) -> bool:
        """
        Push to remote repository with retry mechanism.

        Returns True if successful or no remote configured.
        """
        import time
        import random

        # Check if remote exists
        try:
            remote_check = subprocess.run(
                ["git", "remote", "-v"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=10
            )

            if not remote_check.stdout.strip():
                # No remote configured, skip push
                return True

        except Exception:
            return True  # No remote, skip push

        # Get current branch
        try:
            branch_result = subprocess.run(
                ["git", "branch", "--show-current"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=10
            )
            current_branch = branch_result.stdout.strip()
        except Exception:
            current_branch = "main"

        # Push with retry
        for attempt in range(self.config["max_retry_attempts"]):
            try:
                result = subprocess.run(
                    ["git", "push", "-u", "origin", current_branch],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=120
                )

                if result.returncode == 0:
                    return True

                if "rejected" in result.stderr.lower():
                    # Need to pull first
                    print(f"  ⚠️ Push rejected, pulling first...")
                    pull_result = subprocess.run(
                        ["git", "pull", "--rebase", "origin", current_branch],
                        cwd=self.project_root,
                        capture_output=True,
                        text=True,
                        timeout=120
                    )
                    if pull_result.returncode == 0:
                        continue  # Retry push

                delay = self.config["retry_delay_seconds"] * (2 ** attempt) + random.uniform(0, 1)
                print(f"  ⚠️ Push failed, retrying in {delay:.1f}s...")
                time.sleep(delay)

            except subprocess.TimeoutExpired:
                print(f"  ⚠️ Push timed out (attempt {attempt + 1})")
            except Exception as e:
                print(f"  ⚠️ Push error: {e}")

        return False

    def _create_phase_commit_message(
        self,
        phase_number: int,
        phase_name: str,
        tests_passing: int = None,
        tests_total: int = None,
        components_completed: list = None
    ) -> str:
        """Create detailed commit message for phase completion."""
        prefix = self.config["commit_message_prefix"]
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

        lines = [
            f"{prefix} complete Phase {phase_number} - {phase_name}",
            "",
            f"Orchestration progress checkpoint",
            f"Timestamp: {timestamp}",
            "",
        ]

        if tests_passing is not None and tests_total is not None:
            pass_rate = (tests_passing / tests_total * 100) if tests_total > 0 else 0
            lines.append(f"Tests: {tests_passing}/{tests_total} passing ({pass_rate:.1f}%)")

        if components_completed:
            lines.append(f"Components completed: {', '.join(components_completed)}")

        lines.extend([
            "",
            "This is an automated progress commit to preserve work in progress.",
            "Environment may be unstable - committing at natural boundaries.",
        ])

        return '\n'.join(lines)

    def _create_component_commit_message(
        self,
        component_name: str,
        tests_passing: int = None,
        test_coverage: float = None
    ) -> str:
        """Create commit message for component completion."""
        prefix = self.config["commit_message_prefix"]
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

        lines = [
            f"{prefix} complete component {component_name}",
            "",
            f"Timestamp: {timestamp}",
        ]

        if tests_passing is not None:
            lines.append(f"Tests passing: {tests_passing}")

        if test_coverage is not None:
            lines.append(f"Coverage: {test_coverage:.1f}%")

        lines.append("")
        lines.append("Automated progress preservation commit.")

        return '\n'.join(lines)

    def _create_checkpoint_commit_message(
        self,
        checkpoint_id: str,
        context_usage_percent: float = None
    ) -> str:
        """Create commit message for checkpoint save."""
        prefix = self.config["commit_message_prefix"]
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

        lines = [
            f"{prefix} checkpoint {checkpoint_id}",
            "",
            f"Timestamp: {timestamp}",
        ]

        if context_usage_percent is not None:
            lines.append(f"Context usage: {context_usage_percent:.1f}%")

        lines.append("")
        lines.append("Session checkpoint for resumption.")

        return '\n'.join(lines)

    def get_preservation_stats(self) -> dict:
        """Get statistics about preservation operations."""
        return {
            "commits": self.commit_count,
            "pushes": self.push_count,
            "config": self.config
        }

    def verify_remote_sync(self) -> dict:
        """
        Verify local and remote are in sync.

        Returns status dict.
        """
        try:
            # Fetch remote info
            subprocess.run(
                ["git", "fetch", "origin"],
                cwd=self.project_root,
                capture_output=True,
                timeout=60
            )

            # Check status
            result = subprocess.run(
                ["git", "status", "-sb"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=10
            )

            status = result.stdout.strip()
            is_synced = "ahead" not in status and "behind" not in status

            return {
                "synced": is_synced,
                "status": status,
                "commits_since_push": self.commit_count - self.push_count
            }

        except Exception as e:
            return {
                "synced": False,
                "error": str(e),
                "commits_since_push": self.commit_count - self.push_count
            }


def main():
    """CLI entry point."""
    if len(sys.argv) < 3:
        print("Usage: python progress_preserver.py <command> <project_root> [args]")
        print()
        print("Commands:")
        print("  phase <root> <num> <name>  - Commit phase completion")
        print("  component <root> <name>    - Commit component completion")
        print("  milestone <root> <name>    - Commit major milestone")
        print("  stats <root>               - Show preservation statistics")
        print("  sync <root>                - Verify remote sync status")
        print("  config <root>              - Show/save configuration")
        print()
        print("Example:")
        print("  python progress_preserver.py phase . 3 Integration")
        print("  python progress_preserver.py component . auth-service")
        sys.exit(1)

    command = sys.argv[1]
    project_root = sys.argv[2]

    preserver = ProgressPreserver(project_root)

    if command == "phase":
        if len(sys.argv) < 5:
            print("Usage: phase <root> <number> <name>")
            sys.exit(1)
        phase_num = int(sys.argv[3])
        phase_name = sys.argv[4]
        success = preserver.on_phase_complete(phase_num, phase_name)
        sys.exit(0 if success else 1)

    elif command == "component":
        if len(sys.argv) < 4:
            print("Usage: component <root> <name>")
            sys.exit(1)
        comp_name = sys.argv[3]
        success = preserver.on_component_complete(comp_name)
        sys.exit(0 if success else 1)

    elif command == "milestone":
        if len(sys.argv) < 4:
            print("Usage: milestone <root> <name> [description]")
            sys.exit(1)
        milestone_name = sys.argv[3]
        description = sys.argv[4] if len(sys.argv) > 4 else ""
        success = preserver.on_major_milestone(milestone_name, description)
        sys.exit(0 if success else 1)

    elif command == "stats":
        stats = preserver.get_preservation_stats()
        print(json.dumps(stats, indent=2))

    elif command == "sync":
        status = preserver.verify_remote_sync()
        print(json.dumps(status, indent=2))
        if not status.get("synced", False):
            sys.exit(1)

    elif command == "config":
        print("Current configuration:")
        print(json.dumps(preserver.config, indent=2))
        if len(sys.argv) > 3 and sys.argv[3] == "--save":
            preserver.save_config()

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == "__main__":
    main()
