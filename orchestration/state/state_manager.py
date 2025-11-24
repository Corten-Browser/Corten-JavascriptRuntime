#!/usr/bin/env python3
"""
State Management System for Onboarding Process

Provides idempotency, resume capability, and mode detection for the
onboarding system. Tracks phase completion, caches artifacts, and
manages orchestration compliance.

Version: 1.0.0
"""

import sys
import json
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime
from enum import Enum
import shutil

from orchestration.core.paths import DataPaths


class OnboardingMode(Enum):
    """Onboarding operation modes"""
    FRESH = "fresh"          # New onboarding on clean/existing project
    RESUME = "resume"        # Continue failed/interrupted onboarding
    UPGRADE = "upgrade"      # Upgrade old orchestration to new version
    VERIFY = "verify"        # Verify compliance without changes


class PhaseStatus(Enum):
    """Phase completion status"""
    NOT_STARTED = "not_started"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"


class StateManager:
    """
    Manages onboarding state for idempotency and resume capability.

    Features:
    - Phase completion tracking
    - Artifact caching
    - Mode detection (fresh/resume/upgrade/verify)
    - Compliance status tracking
    - Safe state persistence
    """

    CACHE_DIR = "onboarding_cache"

    # Phase definitions (0-10 for onboarding)
    PHASES = [
        "0_preflight",
        "1_analysis",
        "2_planning",
        "3_installation",
        "4_preparation",
        "5_migration",
        "6_imports",
        "7_manifests",
        "8_contracts",
        "9_specifications",
        "10_verification"
    ]

    def __init__(self, project_dir: Path):
        """
        Initialize state manager.

        Args:
            project_dir: Root directory of target project
        """
        self.project_dir = Path(project_dir).resolve()
        self._paths = DataPaths(self.project_dir)
        self.state_file = self._paths.orchestration_state
        self.cache_dir = self.project_dir / self.CACHE_DIR
        self.state: Dict = {}

        # Load existing state or initialize new
        if self.state_file.exists():
            self.load_state()
        else:
            self._initialize_state()

    def _initialize_state(self):
        """Initialize new state structure"""
        self.state = {
            "version": "1.8.0",
            "onboarding_status": "not_started",
            "started_at": None,
            "last_updated": None,
            "mode": None,
            "phases_completed": {},
            "artifacts": {},
            "orchestration_features": {},
            "compliance_percentage": 0.0,
            "compliance_gaps": []
        }

        # Initialize phase tracking
        for phase in self.PHASES:
            self.state["phases_completed"][phase] = {
                "completed": False,
                "started_at": None,
                "completed_at": None,
                "duration_seconds": None,
                "artifacts": [],
                "error": None
            }

    def load_state(self) -> Dict:
        """
        Load state from file.

        Returns:
            Current state dictionary
        """
        try:
            with open(self.state_file, 'r') as f:
                self.state = json.load(f)
            return self.state
        except json.JSONDecodeError as e:
            raise ValueError(f"Corrupted state file: {e}")
        except Exception as e:
            raise RuntimeError(f"Failed to load state: {e}")

    def save_state(self, state: Optional[Dict] = None):
        """
        Save state to file.

        Args:
            state: State to save (uses self.state if not provided)
        """
        if state is not None:
            self.state = state

        self.state["last_updated"] = datetime.now().isoformat()

        # Atomic write (write to temp, then rename)
        temp_file = self.state_file.with_suffix('.tmp')
        try:
            with open(temp_file, 'w') as f:
                json.dump(self.state, f, indent=2)
            temp_file.replace(self.state_file)
        except Exception as e:
            if temp_file.exists():
                temp_file.unlink()
            raise RuntimeError(f"Failed to save state: {e}")

    def detect_mode(self) -> OnboardingMode:
        """
        Detect appropriate onboarding mode.

        Returns:
            Detected mode (fresh/resume/upgrade/verify)
        """
        # Check if orchestration already installed
        has_orchestration = (self.project_dir / "orchestration").exists()
        has_components = (self.project_dir / "components").exists()
        has_config = (self.project_dir / "orchestration-config.json").exists()

        # Check state file status
        has_state = self.state_file.exists()

        if not has_state:
            # No state file
            if has_orchestration and has_config:
                # Orchestration exists but no state - verify/upgrade mode
                return OnboardingMode.UPGRADE
            else:
                # Fresh onboarding
                return OnboardingMode.FRESH
        else:
            # State file exists
            status = self.state.get("onboarding_status", "not_started")

            if status == "complete":
                # Complete onboarding - verify mode
                return OnboardingMode.VERIFY
            elif status in ["in_progress", "partial"]:
                # Incomplete onboarding - resume mode
                return OnboardingMode.RESUME
            else:
                # Unknown status - treat as fresh
                return OnboardingMode.FRESH

    def mark_phase_started(self, phase: str):
        """
        Mark phase as started.

        Args:
            phase: Phase identifier (e.g., "1_analysis")
        """
        if phase not in self.PHASES:
            raise ValueError(f"Invalid phase: {phase}")

        # Set started timestamp if not already set
        if self.state["started_at"] is None:
            self.state["started_at"] = datetime.now().isoformat()

        self.state["onboarding_status"] = "in_progress"
        self.state["phases_completed"][phase]["started_at"] = datetime.now().isoformat()
        self.state["phases_completed"][phase]["completed"] = False
        self.state["phases_completed"][phase]["error"] = None

        self.save_state()

    def mark_phase_completed(self, phase: str, artifacts: Optional[List[str]] = None):
        """
        Mark phase as completed.

        Args:
            phase: Phase identifier
            artifacts: List of artifact names created by this phase
        """
        if phase not in self.PHASES:
            raise ValueError(f"Invalid phase: {phase}")

        phase_data = self.state["phases_completed"][phase]

        # Calculate duration
        if phase_data["started_at"]:
            started = datetime.fromisoformat(phase_data["started_at"])
            completed = datetime.now()
            duration = (completed - started).total_seconds()
            phase_data["duration_seconds"] = duration

        phase_data["completed"] = True
        phase_data["completed_at"] = datetime.now().isoformat()
        phase_data["artifacts"] = artifacts or []

        # Check if all phases complete
        all_complete = all(
            self.state["phases_completed"][p]["completed"]
            for p in self.PHASES
        )

        if all_complete:
            self.state["onboarding_status"] = "complete"
        else:
            self.state["onboarding_status"] = "partial"

        self.save_state()

    def mark_phase_failed(self, phase: str, error: str):
        """
        Mark phase as failed.

        Args:
            phase: Phase identifier
            error: Error message
        """
        if phase not in self.PHASES:
            raise ValueError(f"Invalid phase: {phase}")

        phase_data = self.state["phases_completed"][phase]

        # Calculate duration up to failure
        if phase_data["started_at"]:
            started = datetime.fromisoformat(phase_data["started_at"])
            failed = datetime.now()
            duration = (failed - started).total_seconds()
            phase_data["duration_seconds"] = duration

        phase_data["completed"] = False
        phase_data["error"] = error
        self.state["onboarding_status"] = "partial"

        self.save_state()

    def get_last_completed_phase(self) -> Optional[str]:
        """
        Get last successfully completed phase.

        Returns:
            Phase identifier or None if no phases complete
        """
        for phase in reversed(self.PHASES):
            if self.state["phases_completed"][phase]["completed"]:
                return phase
        return None

    def get_next_phase(self) -> Optional[str]:
        """
        Get next phase to execute.

        Returns:
            Phase identifier or None if all complete
        """
        for phase in self.PHASES:
            if not self.state["phases_completed"][phase]["completed"]:
                return phase
        return None

    def is_phase_completed(self, phase: str) -> bool:
        """
        Check if phase is completed.

        Args:
            phase: Phase identifier

        Returns:
            True if completed
        """
        if phase not in self.PHASES:
            raise ValueError(f"Invalid phase: {phase}")
        return self.state["phases_completed"][phase]["completed"]

    def cache_artifact(self, name: str, data: Any):
        """
        Cache artifact to disk.

        Args:
            name: Artifact name (used as filename)
            data: Data to cache (must be JSON-serializable)
        """
        # Ensure cache directory exists
        self.cache_dir.mkdir(exist_ok=True)

        # Write artifact
        artifact_path = self.cache_dir / f"{name}.json"
        try:
            with open(artifact_path, 'w') as f:
                json.dump(data, f, indent=2)

            # Record in state
            self.state["artifacts"][name] = str(artifact_path.relative_to(self.project_dir))
            self.save_state()
        except Exception as e:
            raise RuntimeError(f"Failed to cache artifact '{name}': {e}")

    def load_artifact(self, name: str) -> Optional[Any]:
        """
        Load cached artifact.

        Args:
            name: Artifact name

        Returns:
            Cached data or None if not found
        """
        artifact_path = self.cache_dir / f"{name}.json"

        if not artifact_path.exists():
            return None

        try:
            with open(artifact_path, 'r') as f:
                return json.load(f)
        except Exception as e:
            raise RuntimeError(f"Failed to load artifact '{name}': {e}")

    def get_compliance_status(self) -> Dict:
        """
        Get compliance status summary.

        Returns:
            Dict with compliance_percentage and compliance_gaps
        """
        return {
            "compliance_percentage": self.state.get("compliance_percentage", 0.0),
            "compliance_gaps": self.state.get("compliance_gaps", []),
            "orchestration_features": self.state.get("orchestration_features", {})
        }

    def set_compliance_status(self, percentage: float, gaps: List[str], features: Dict[str, List[str]]):
        """
        Update compliance status.

        Args:
            percentage: Compliance percentage (0-100)
            gaps: List of missing features/gaps
            features: Dict of version -> feature list
        """
        self.state["compliance_percentage"] = percentage
        self.state["compliance_gaps"] = gaps
        self.state["orchestration_features"] = features
        self.save_state()

    def clear_cache(self):
        """Clear artifact cache"""
        if self.cache_dir.exists():
            shutil.rmtree(self.cache_dir)
        self.state["artifacts"] = {}
        self.save_state()

    def reset_state(self):
        """Reset state to initial (useful for re-onboarding)"""
        self._initialize_state()
        self.save_state()
        self.clear_cache()

    def get_summary(self) -> str:
        """
        Get human-readable state summary.

        Returns:
            Formatted summary string
        """
        lines = []
        lines.append("=" * 60)
        lines.append("ONBOARDING STATE SUMMARY")
        lines.append("=" * 60)

        # Status
        status = self.state.get("onboarding_status", "unknown")
        lines.append(f"Status: {status.upper()}")

        # Mode
        mode = self.detect_mode()
        lines.append(f"Detected Mode: {mode.value}")

        # Timing
        if self.state.get("started_at"):
            lines.append(f"Started: {self.state['started_at']}")
        if self.state.get("last_updated"):
            lines.append(f"Last Updated: {self.state['last_updated']}")

        # Phase progress
        completed_count = sum(
            1 for p in self.PHASES
            if self.state["phases_completed"][p]["completed"]
        )
        lines.append(f"\nPhase Progress: {completed_count}/{len(self.PHASES)} phases complete")

        # Phase details
        lines.append("\nPhases:")
        for phase in self.PHASES:
            phase_data = self.state["phases_completed"][phase]
            status_icon = "✅" if phase_data["completed"] else "⏳" if phase_data["started_at"] else "⬜"
            error_text = f" (ERROR: {phase_data['error']})" if phase_data["error"] else ""
            lines.append(f"  {status_icon} {phase}{error_text}")

        # Compliance
        compliance_pct = self.state.get("compliance_percentage", 0.0)
        lines.append(f"\nCompliance: {compliance_pct:.1f}%")

        gaps = self.state.get("compliance_gaps", [])
        if gaps:
            lines.append(f"Gaps: {len(gaps)} identified")
            for gap in gaps[:3]:  # Show first 3
                lines.append(f"  - {gap}")
            if len(gaps) > 3:
                lines.append(f"  ... and {len(gaps) - 3} more")

        # Cached artifacts
        artifacts = self.state.get("artifacts", {})
        if artifacts:
            lines.append(f"\nCached Artifacts: {len(artifacts)}")
            for name in list(artifacts.keys())[:3]:
                lines.append(f"  - {name}")
            if len(artifacts) > 3:
                lines.append(f"  ... and {len(artifacts) - 3} more")

        lines.append("=" * 60)

        return "\n".join(lines)


def main():
    """CLI entry point for testing/debugging"""
    import sys
    import argparse

    parser = argparse.ArgumentParser(description="Onboarding State Manager")
    parser.add_argument("project_dir", nargs="?", default=".",
                       help="Project directory (default: current)")
    parser.add_argument("--summary", action="store_true",
                       help="Show state summary")
    parser.add_argument("--mode", action="store_true",
                       help="Detect and show mode")
    parser.add_argument("--reset", action="store_true",
                       help="Reset state (DESTRUCTIVE)")
    parser.add_argument("--clear-cache", action="store_true",
                       help="Clear artifact cache")

    args = parser.parse_args()

    try:
        manager = StateManager(Path(args.project_dir))

        if args.reset:
            print("⚠️  Resetting state...")
            manager.reset_state()
            print("✅ State reset complete")
            return 0

        if args.clear_cache:
            print("⚠️  Clearing artifact cache...")
            manager.clear_cache()
            print("✅ Cache cleared")
            return 0

        if args.mode:
            mode = manager.detect_mode()
            print(f"Detected Mode: {mode.value}")
            return 0

        if args.summary or True:  # Default action
            print(manager.get_summary())
            return 0

    except Exception as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
