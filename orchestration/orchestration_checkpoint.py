"""
Orchestration-level checkpoint management for resume capability.

This handles checkpoints for the entire orchestration workflow (all 6 phases),
separate from component-level agent checkpoints (checkpoint_manager.py).

Handles saving, loading, updating, and validating orchestration checkpoints.
"""

from datetime import datetime
from pathlib import Path
from typing import Optional, Dict, List, Any
import json
import hashlib


class OrchestrationCheckpointManager:
    """Manages orchestration checkpoints for resume functionality."""

    CHECKPOINT_FILE = "orchestration-checkpoint.json"
    VERSION = "1.0"

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self.checkpoint_path = self.project_root / self.CHECKPOINT_FILE

    def create_checkpoint(
        self,
        user_prompt: str,
        specification_files: List[str],
        components: List[str]
    ) -> str:
        """Create new checkpoint at orchestration start."""
        orch_id = self._generate_orchestration_id()
        timestamp = datetime.utcnow().isoformat() + "Z"

        checkpoint = {
            "orchestration_id": orch_id,
            "version": self.VERSION,
            "created_at": timestamp,
            "last_updated": timestamp,
            "status": "in_progress",
            "original_request": {
                "user_prompt": user_prompt,
                "specification_files": specification_files,
                "command": "/orchestrate-full",
                "timestamp": timestamp
            },
            "project_context": self._gather_project_context(components),
            "phase_progress": self._initialize_phases(),
            "component_states": {},
            "quality_metrics": {},
            "stopping_context": {}
        }

        self._save_checkpoint(checkpoint)
        return orch_id

    def update_phase(
        self,
        phase_number: int,
        status: str,
        outputs: Dict[str, Any] = None
    ):
        """Update phase status in checkpoint."""
        checkpoint = self.load_checkpoint()
        if not checkpoint:
            return  # Silently fail if no checkpoint

        phases = checkpoint["phase_progress"]["phases"]
        phase = phases[phase_number - 1]

        phase["status"] = status

        if status == "completed":
            phase["completed_at"] = datetime.utcnow().isoformat() + "Z"
            if phase["started_at"]:
                start = datetime.fromisoformat(phase["started_at"].replace("Z", ""))
                end = datetime.fromisoformat(phase["completed_at"].replace("Z", ""))
                phase["duration_minutes"] = int((end - start).total_seconds() / 60)
        elif status == "in_progress" and not phase["started_at"]:
            phase["started_at"] = datetime.utcnow().isoformat() + "Z"

        if outputs:
            phase["outputs"] = outputs

        checkpoint["phase_progress"]["current_phase"] = phase_number
        checkpoint["last_updated"] = datetime.utcnow().isoformat() + "Z"

        self._save_checkpoint(checkpoint)

    def update_component(
        self,
        component_name: str,
        status: str,
        metadata: Dict[str, Any]
    ):
        """Update component state in checkpoint."""
        checkpoint = self.load_checkpoint()
        if not checkpoint:
            return

        checkpoint["component_states"][component_name] = {
            "status": status,
            **metadata
        }

        checkpoint["last_updated"] = datetime.utcnow().isoformat() + "Z"
        self._save_checkpoint(checkpoint)

    def mark_stopped(self, reason: str, details: str, can_resume: bool = True):
        """Mark orchestration as stopped."""
        checkpoint = self.load_checkpoint()
        if not checkpoint:
            return

        checkpoint["status"] = "stopped"
        checkpoint["stopping_context"] = {
            "reason": reason,
            "details": details,
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "can_auto_resume": can_resume
        }
        checkpoint["last_updated"] = datetime.utcnow().isoformat() + "Z"

        self._save_checkpoint(checkpoint)

    def mark_completed(self):
        """Mark orchestration as successfully completed."""
        checkpoint = self.load_checkpoint()
        if not checkpoint:
            return

        checkpoint["status"] = "completed"
        checkpoint["last_updated"] = datetime.utcnow().isoformat() + "Z"

        self._save_checkpoint(checkpoint)

    def load_checkpoint(self) -> Optional[Dict[str, Any]]:
        """Load checkpoint from disk."""
        if not self.checkpoint_path.exists():
            return None

        try:
            with open(self.checkpoint_path, 'r') as f:
                data = json.load(f)

            # Validate checkpoint
            if not self._validate_checkpoint(data):
                return None

            return data
        except (json.JSONDecodeError, IOError) as e:
            print(f"⚠️ Checkpoint file corrupted: {e}")
            return None

    def checkpoint_exists(self) -> bool:
        """Check if valid checkpoint exists."""
        return self.load_checkpoint() is not None

    def _validate_checkpoint(self, data: Dict[str, Any]) -> bool:
        """Validate checkpoint structure and integrity."""
        required_keys = [
            "orchestration_id", "version", "created_at",
            "original_request", "phase_progress"
        ]

        for key in required_keys:
            if key not in data:
                print(f"⚠️ Checkpoint missing required key: {key}")
                return False

        # Version check
        if data["version"] != self.VERSION:
            print(f"⚠️ Checkpoint version mismatch: {data['version']} != {self.VERSION}")
            return False

        return True

    def _save_checkpoint(self, data: Dict[str, Any]):
        """Save checkpoint to disk."""
        try:
            with open(self.checkpoint_path, 'w') as f:
                json.dump(data, f, indent=2)
        except IOError as e:
            print(f"❌ Failed to save checkpoint: {e}")

    def _generate_orchestration_id(self) -> str:
        """Generate unique orchestration ID."""
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        random_suffix = hashlib.md5(
            str(datetime.utcnow().timestamp()).encode()
        ).hexdigest()[:6]
        return f"orch_{timestamp}_{random_suffix}"

    def _gather_project_context(self, components: List[str]) -> Dict[str, Any]:
        """Gather current project context."""
        import subprocess

        try:
            branch = subprocess.run(
                ["git", "branch", "--show-current"],
                capture_output=True,
                text=True,
                cwd=self.project_root
            ).stdout.strip()

            commit = subprocess.run(
                ["git", "rev-parse", "--short", "HEAD"],
                capture_output=True,
                text=True,
                cwd=self.project_root
            ).stdout.strip()
        except:
            branch = "unknown"
            commit = "unknown"

        return {
            "project_root": str(self.project_root),
            "git_branch": branch,
            "initial_commit": commit,
            "components_identified": len(components)
        }

    def _initialize_phases(self) -> Dict[str, Any]:
        """Initialize phase tracking structure."""
        phase_names = [
            "Planning & Analysis",
            "Component Development",
            "Unit Testing",
            "Contract Validation",
            "Integration Testing",
            "Documentation & Completion"
        ]

        phases = []
        for i, name in enumerate(phase_names, 1):
            phases.append({
                "phase_number": i,
                "name": name,
                "status": "not_started",
                "started_at": None,
                "completed_at": None,
                "duration_minutes": None,
                "outputs": {}
            })

        return {
            "current_phase": 0,
            "phases": phases
        }

    def recover_checkpoint(self) -> bool:
        """
        Attempt to recover corrupted checkpoint.

        Returns True if recovery successful, False otherwise.
        """
        if not self.checkpoint_path.exists():
            return False

        # Try to read raw file
        try:
            with open(self.checkpoint_path, 'r') as f:
                content = f.read()
        except:
            return False

        # Check for common corruption patterns

        # Pattern 1: Truncated JSON (missing closing braces)
        if content.count('{') > content.count('}'):
            # Try to close JSON
            missing_braces = content.count('{') - content.count('}')
            recovered = content + ('}' * missing_braces)

            try:
                data = json.loads(recovered)
                # Validate minimally
                if "orchestration_id" in data:
                    # Save recovered version
                    backup_path = self.checkpoint_path.with_suffix('.json.corrupted')
                    self.checkpoint_path.rename(backup_path)
                    self._save_checkpoint(data)
                    print(f"✅ Checkpoint recovered (backed up to {backup_path})")
                    return True
            except:
                pass

        # Pattern 2: Duplicate content (append instead of overwrite)
        # Look for multiple orchestration_id occurrences
        if content.count('"orchestration_id"') > 1:
            # Take last complete JSON object
            import re
            matches = list(re.finditer(r'\{.*?\}', content, re.DOTALL))
            if matches:
                last_json = matches[-1].group(0)
                try:
                    data = json.loads(last_json)
                    if self._validate_checkpoint(data):
                        backup_path = self.checkpoint_path.with_suffix('.json.corrupted')
                        self.checkpoint_path.rename(backup_path)
                        self._save_checkpoint(data)
                        print(f"✅ Checkpoint recovered (backed up to {backup_path})")
                        return True
                except:
                    pass

        print(f"❌ Could not recover checkpoint")
        return False

    def load_checkpoint_with_recovery(self) -> Optional[Dict[str, Any]]:
        """
        Load checkpoint with automatic recovery attempt.

        Tries normal load first, then recovery if that fails.
        """
        checkpoint = self.load_checkpoint()

        if checkpoint is None:
            # Try recovery
            if self.recover_checkpoint():
                checkpoint = self.load_checkpoint()

        return checkpoint
