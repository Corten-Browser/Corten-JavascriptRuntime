#!/usr/bin/env python3
"""
Centralized Path Definitions for Orchestration System

All data file paths are defined here to:
1. Provide single source of truth for path locations
2. Enable easy migration of data files
3. Separate code directories from data directories

Data Directory Structure (orchestration/data/):
├── state/          - Runtime state files
├── logs/           - Activity and execution logs
├── reports/        - Generated reports
├── checkpoints/    - Checkpoint data
└── config/         - User-modifiable configuration

Usage:
    from orchestration.core.paths import DataPaths

    paths = DataPaths()  # Uses current directory
    paths = DataPaths("/path/to/project")  # Explicit project root

    queue_state = paths.queue_state
    completion_state = paths.completion_state

    # Before writing, ensure parent directory exists:
    paths.queue_state.parent.mkdir(parents=True, exist_ok=True)
    paths.queue_state.write_text(json.dumps(data))
"""

from pathlib import Path
from typing import Optional


class DataPaths:
    """
    Centralized data path definitions.

    All project-specific data files are under orchestration/data/
    to clearly separate code from data and enable safe upgrades.
    """

    def __init__(self, project_root: Optional[Path] = None):
        """
        Initialize paths relative to project root.

        Args:
            project_root: Project root directory. Defaults to current directory.
        """
        self.project_root = Path(project_root) if project_root else Path.cwd()
        self.orchestration_root = self.project_root / "orchestration"
        self.data_root = self.orchestration_root / "data"

    # =========================================================================
    # Directory Paths
    # =========================================================================

    @property
    def state_dir(self) -> Path:
        """Directory for runtime state files."""
        return self.data_root / "state"

    @property
    def logs_dir(self) -> Path:
        """Directory for activity and execution logs."""
        return self.data_root / "logs"

    @property
    def reports_dir(self) -> Path:
        """Directory for generated reports."""
        return self.data_root / "reports"

    @property
    def checkpoints_dir(self) -> Path:
        """Directory for checkpoint data."""
        return self.data_root / "checkpoints"

    @property
    def config_dir(self) -> Path:
        """Directory for user-modifiable configuration."""
        return self.data_root / "config"

    # =========================================================================
    # State Files
    # =========================================================================

    @property
    def queue_state(self) -> Path:
        """Task queue state file."""
        return self.state_dir / "queue_state.json"

    @property
    def completion_state(self) -> Path:
        """Completion/verification state file."""
        return self.state_dir / "completion_state.json"

    @property
    def spec_manifest(self) -> Path:
        """Specification manifest file."""
        return self.state_dir / "spec_manifest.json"

    @property
    def extraction_metadata(self) -> Path:
        """Feature extraction metadata file."""
        return self.state_dir / "extraction_metadata.json"

    @property
    def orchestration_state(self) -> Path:
        """Orchestration/gate state file."""
        return self.state_dir / "orchestration_state.json"

    # =========================================================================
    # Log Files
    # =========================================================================

    @property
    def activity_log(self) -> Path:
        """Activity log file."""
        return self.logs_dir / "activity_log.json"

    @property
    def metrics(self) -> Path:
        """Metrics data file."""
        return self.logs_dir / "metrics.json"

    @property
    def alerts(self) -> Path:
        """Alerts data file."""
        return self.logs_dir / "alerts.json"

    @property
    def gate_execution_log(self) -> Path:
        """Gate execution log file."""
        return self.logs_dir / "gate_execution_log.json"

    # =========================================================================
    # Report Files
    # =========================================================================

    @property
    def verification_results(self) -> Path:
        """Verification results report."""
        return self.reports_dir / "verification_results.json"

    @property
    def rationalization_log(self) -> Path:
        """Rationalization detection log."""
        return self.reports_dir / "rationalization_log.json"

    @property
    def spec_v2_report(self) -> Path:
        """Spec v2 checker report."""
        return self.reports_dir / "spec_v2_report.json"

    @property
    def coverage_report(self) -> Path:
        """Coverage report."""
        return self.reports_dir / "coverage_report.json"

    # =========================================================================
    # Config Files
    # =========================================================================

    @property
    def orchestration_config(self) -> Path:
        """Main orchestration configuration file."""
        return self.config_dir / "orchestration.json"

    @property
    def enforcement_config(self) -> Path:
        """Enforcement configuration file."""
        return self.config_dir / "enforcement_config.json"

    @property
    def gate_schedule(self) -> Path:
        """Gate schedule configuration file."""
        return self.config_dir / "gate_schedule.json"

    # =========================================================================
    # History Files
    # =========================================================================

    @property
    def migration_log(self) -> Path:
        """Migration history log."""
        return self.data_root / "migration_log.json"

    @property
    def installation_manifest(self) -> Path:
        """Installation manifest."""
        return self.data_root / "installation_manifest.json"

    # =========================================================================
    # Utility Methods
    # =========================================================================

    def ensure_directories(self) -> None:
        """Create all data directories if they don't exist."""
        directories = [
            self.state_dir,
            self.logs_dir,
            self.reports_dir,
            self.checkpoints_dir,
            self.config_dir,
        ]
        for directory in directories:
            directory.mkdir(parents=True, exist_ok=True)

    def get_checkpoint_file(self, checkpoint_id: str) -> Path:
        """Get path for a specific checkpoint file."""
        return self.checkpoints_dir / f"{checkpoint_id}.json"

    def list_checkpoints(self) -> list[Path]:
        """List all checkpoint files."""
        if self.checkpoints_dir.exists():
            return list(self.checkpoints_dir.glob("*.json"))
        return []


# =============================================================================
# Migration Support (used by migration scripts only)
# =============================================================================

# Legacy path mapping for migration script
# Maps old paths to new DataPaths properties
LEGACY_PATH_MAPPING = {
    # Old path -> (DataPaths property name, description)
    "orchestration/tasks/queue_state.json": ("queue_state", "Task queue state"),
    "orchestration/verification/state/completion_state.json": ("completion_state", "Completion state"),
    "orchestration/spec_manifest.json": ("spec_manifest", "Spec manifest"),
    "orchestration/extraction_metadata.json": ("extraction_metadata", "Extraction metadata"),
    "orchestration/monitoring/activity_log.json": ("activity_log", "Activity log"),
    "orchestration/monitoring/metrics.json": ("metrics", "Metrics"),
    "orchestration/monitoring/alerts.json": ("alerts", "Alerts"),
    "orchestration/automated_gates/execution_log.json": ("gate_execution_log", "Gate execution log"),
    "orchestration/verification/verification_results.json": ("verification_results", "Verification results"),
    "orchestration/verification/rationalization_log.json": ("rationalization_log", "Rationalization log"),
    "orchestration/verification/spec_v2_report.json": ("spec_v2_report", "Spec v2 report"),
    "orchestration/verification/coverage_report.json": ("coverage_report", "Coverage report"),
    "orchestration/config/orchestration.json": ("orchestration_config", "Orchestration config"),
    "orchestration/enforcement_config.json": ("enforcement_config", "Enforcement config"),
    "orchestration/automated_gates/gate_schedule.json": ("gate_schedule", "Gate schedule"),
    "orchestration/MIGRATION_LOG.json": ("migration_log", "Migration log"),
    "orchestration/INSTALLATION_MANIFEST.json": ("installation_manifest", "Installation manifest"),
    "orchestration/orchestration-state.json": ("orchestration_state", "Orchestration state"),
}


def migrate_data_file(old_path: Path, new_path: Path, dry_run: bool = False) -> bool:
    """
    Migrate a data file from old location to new location.

    Args:
        old_path: Source path (legacy location)
        new_path: Destination path (new data/ location)
        dry_run: If True, only report what would happen

    Returns:
        True if migration occurred (or would occur), False otherwise

    Raises:
        OSError: If migration fails (file move error)
    """
    if not old_path.exists():
        return False

    if new_path.exists():
        # New path already has data - don't overwrite
        print(f"  SKIP: {old_path} -> {new_path} (destination exists)")
        return False

    if dry_run:
        print(f"  MIGRATE: {old_path} -> {new_path}")
        return True

    # Ensure parent directory exists
    new_path.parent.mkdir(parents=True, exist_ok=True)

    # Move the file (let OSError propagate on failure)
    old_path.rename(new_path)
    print(f"  Migrated: {old_path} -> {new_path}")
    return True


if __name__ == "__main__":
    # Demo: show all paths
    paths = DataPaths()
    print("Data Paths Configuration")
    print("=" * 60)
    print(f"Project root: {paths.project_root}")
    print(f"Data root: {paths.data_root}")
    print()
    print("State files:")
    print(f"  queue_state: {paths.queue_state}")
    print(f"  completion_state: {paths.completion_state}")
    print(f"  spec_manifest: {paths.spec_manifest}")
    print()
    print("Log files:")
    print(f"  activity_log: {paths.activity_log}")
    print(f"  metrics: {paths.metrics}")
    print(f"  alerts: {paths.alerts}")
    print()
    print("Config files:")
    print(f"  orchestration_config: {paths.orchestration_config}")
    print(f"  enforcement_config: {paths.enforcement_config}")
