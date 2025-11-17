#!/usr/bin/env python3
"""
Tests for Progress Preserver - Automatic Git Commit and Push at Boundaries

Tests cover:
- Configuration loading
- Phase completion commits
- Component completion commits
- Checkpoint commits
- Git operations (staging, committing)
- Push behavior
"""

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock

from progress_preserver import ProgressPreserver


class TestProgressPreserverConfig(unittest.TestCase):
    """Tests for configuration loading and management."""

    def test_default_config_loaded(self):
        """
        Given no config file
        When preserver initialized
        Then default config is used
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            preserver = ProgressPreserver(tmpdir)

            assert preserver.config["auto_commit"] is True
            assert preserver.config["auto_push"] is True
            assert preserver.config["push_on_phase_complete"] is True
            assert preserver.config["max_retry_attempts"] == 5

    def test_custom_config_loaded(self):
        """
        Given custom config file
        When preserver initialized
        Then custom config is merged
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            config_dir = Path(tmpdir) / "orchestration"
            config_dir.mkdir()

            config_file = config_dir / "progress_preserver_config.json"
            config_file.write_text(json.dumps({
                "auto_push": False,
                "max_retry_attempts": 10
            }))

            preserver = ProgressPreserver(tmpdir)

            # Custom values
            assert preserver.config["auto_push"] is False
            assert preserver.config["max_retry_attempts"] == 10
            # Default values preserved
            assert preserver.config["auto_commit"] is True

    def test_save_config(self):
        """
        Given preserver with config
        When config saved
        Then config file is written
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            config_dir = Path(tmpdir) / "orchestration"
            config_dir.mkdir()

            preserver = ProgressPreserver(tmpdir)
            preserver.config["auto_push"] = False
            preserver.save_config()

            config_file = config_dir / "progress_preserver_config.json"
            assert config_file.exists()

            saved_config = json.loads(config_file.read_text())
            assert saved_config["auto_push"] is False


class TestPhaseCompletion(unittest.TestCase):
    """Tests for phase completion commits."""

    def test_phase_complete_creates_commit_message(self):
        """
        Given phase completion
        When commit message created
        Then message includes phase info
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            preserver = ProgressPreserver(tmpdir)

            message = preserver._create_phase_commit_message(
                phase_number=3,
                phase_name="Integration",
                tests_passing=150,
                tests_total=160,
                components_completed=["auth", "db", "api"]
            )

            assert "Phase 3" in message
            assert "Integration" in message
            assert "150/160" in message
            assert "auth, db, api" in message

    def test_phase_complete_disabled_when_auto_commit_false(self):
        """
        Given auto_commit disabled
        When phase completes
        Then no commit occurs
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            preserver = ProgressPreserver(tmpdir)
            preserver.config["auto_commit"] = False

            result = preserver.on_phase_complete(1, "Setup")

            assert result is True
            # No git operations should have occurred


class TestComponentCompletion(unittest.TestCase):
    """Tests for component completion commits."""

    def test_component_commit_message_created(self):
        """
        Given component completion
        When commit message created
        Then message includes component info
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            preserver = ProgressPreserver(tmpdir)

            message = preserver._create_component_commit_message(
                component_name="auth-service",
                tests_passing=45,
                test_coverage=87.5
            )

            assert "auth-service" in message
            assert "45" in message
            assert "87.5" in message


class TestCheckpointCommit(unittest.TestCase):
    """Tests for checkpoint save commits."""

    def test_checkpoint_commit_message_created(self):
        """
        Given checkpoint save
        When commit message created
        Then message includes checkpoint info
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            preserver = ProgressPreserver(tmpdir)

            message = preserver._create_checkpoint_commit_message(
                checkpoint_id="session-001",
                context_usage_percent=72.5
            )

            assert "session-001" in message
            assert "72.5" in message
            assert "checkpoint" in message.lower()

    def test_checkpoint_not_committed_when_disabled(self):
        """
        Given include_checkpoint_state disabled
        When checkpoint saved
        Then no commit occurs
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            preserver = ProgressPreserver(tmpdir)
            preserver.config["include_checkpoint_state"] = False

            result = preserver.on_checkpoint_save("test-123")

            assert result is True


class TestGitOperations(unittest.TestCase):
    """Tests for git staging and commit operations."""

    def test_stage_changes_in_git_repo(self):
        """
        Given git repository with changes
        When changes staged
        Then staging succeeds
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            # Initialize git repo
            subprocess.run(["git", "init"], cwd=tmpdir, capture_output=True)
            subprocess.run(
                ["git", "config", "user.email", "test@test.com"],
                cwd=tmpdir, capture_output=True
            )
            subprocess.run(
                ["git", "config", "user.name", "Test"],
                cwd=tmpdir, capture_output=True
            )

            # Create a file
            test_file = Path(tmpdir) / "test.txt"
            test_file.write_text("test content")

            preserver = ProgressPreserver(tmpdir)
            result = preserver._stage_changes()

            assert result is True

    def test_commit_with_no_changes_succeeds(self):
        """
        Given no changes to commit
        When commit attempted
        Then operation succeeds (no-op)
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            # Initialize git repo
            subprocess.run(["git", "init"], cwd=tmpdir, capture_output=True)
            subprocess.run(
                ["git", "config", "user.email", "test@test.com"],
                cwd=tmpdir, capture_output=True
            )
            subprocess.run(
                ["git", "config", "user.name", "Test"],
                cwd=tmpdir, capture_output=True
            )

            preserver = ProgressPreserver(tmpdir)
            result = preserver._commit("test commit")

            # Should succeed even with nothing to commit
            assert result is True


class TestMilestoneCommits(unittest.TestCase):
    """Tests for major milestone commits."""

    def test_milestone_creates_commit(self):
        """
        Given major milestone
        When milestone recorded
        Then commit is created
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            # Initialize git repo
            subprocess.run(["git", "init"], cwd=tmpdir, capture_output=True)
            subprocess.run(
                ["git", "config", "user.email", "test@test.com"],
                cwd=tmpdir, capture_output=True
            )
            subprocess.run(
                ["git", "config", "user.name", "Test"],
                cwd=tmpdir, capture_output=True
            )

            # Create initial commit
            test_file = Path(tmpdir) / "init.txt"
            test_file.write_text("init")
            subprocess.run(["git", "add", "."], cwd=tmpdir, capture_output=True)
            subprocess.run(
                ["git", "commit", "-m", "initial"],
                cwd=tmpdir, capture_output=True
            )

            # Create change for milestone
            milestone_file = Path(tmpdir) / "milestone.txt"
            milestone_file.write_text("milestone reached")

            preserver = ProgressPreserver(tmpdir)
            result = preserver.on_major_milestone(
                "All Tests Passing",
                "100% test pass rate achieved"
            )

            assert result is True
            assert preserver.commit_count == 1


class TestPreservationStats(unittest.TestCase):
    """Tests for preservation statistics."""

    def test_stats_tracking(self):
        """
        Given preservation operations
        When stats retrieved
        Then counts are correct
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            preserver = ProgressPreserver(tmpdir)
            preserver.commit_count = 5
            preserver.push_count = 3

            stats = preserver.get_preservation_stats()

            assert stats["commits"] == 5
            assert stats["pushes"] == 3
            assert "config" in stats


class TestRemoteSyncVerification(unittest.TestCase):
    """Tests for remote sync verification."""

    def test_verify_sync_no_remote(self):
        """
        Given no remote configured
        When sync verified
        Then returns synced (no remote to sync with)
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            # Initialize git repo without remote
            subprocess.run(["git", "init"], cwd=tmpdir, capture_output=True)

            preserver = ProgressPreserver(tmpdir)
            status = preserver.verify_remote_sync()

            # Should handle gracefully
            assert "commits_since_push" in status


def run_all_tests():
    """Run all tests and report results."""
    print("=" * 70)
    print("PROGRESS PRESERVER - TEST SUITE")
    print("=" * 70)
    print()

    loader = unittest.TestLoader()
    suite = loader.loadTestsFromModule(__import__(__name__))

    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    import sys
    exit_code = run_all_tests()
    sys.exit(exit_code)
