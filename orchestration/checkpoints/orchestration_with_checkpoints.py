"""
Orchestration workflow with checkpoint integration.

This module wraps the existing orchestration logic with checkpointing.
"""

from pathlib import Path
from typing import List, Dict, Any
from orchestration.checkpoints.orchestration_checkpoint import OrchestrationCheckpointManager


class CheckpointedOrchestration:
    """Orchestration with automatic checkpointing."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self.checkpoint_mgr = OrchestrationCheckpointManager(project_root)
        self.orchestration_id = None

    def start_orchestration(
        self,
        user_prompt: str,
        specification_files: List[str],
        components: List[str]
    ):
        """
        Start new orchestration with checkpoint.

        Creates initial checkpoint before beginning work.
        """
        # Create checkpoint
        self.orchestration_id = self.checkpoint_mgr.create_checkpoint(
            user_prompt=user_prompt,
            specification_files=specification_files,
            components=components
        )

        print(f"✅ Orchestration started: {self.orchestration_id}")
        print(f"   Checkpoint created at: {self.checkpoint_mgr.checkpoint_path}")

    def begin_phase(self, phase_number: int, phase_name: str):
        """Mark phase as started."""
        print(f"\n{'='*70}")
        print(f"Starting Phase {phase_number}: {phase_name}")
        print(f"{'='*70}\n")

        self.checkpoint_mgr.update_phase(phase_number, "in_progress")

    def complete_phase(
        self,
        phase_number: int,
        outputs: Dict[str, Any] = None
    ):
        """Mark phase as completed with outputs."""
        self.checkpoint_mgr.update_phase(
            phase_number,
            "completed",
            outputs=outputs
        )

        print(f"\n✅ Phase {phase_number} completed")
        print(f"   Checkpoint updated")

        # Auto-commit and push phase completion to preserve progress
        try:
            from orchestration.enforcement.progress_preserver import ProgressPreserver
            preserver = ProgressPreserver(str(self.project_root))
            phase_name = outputs.get("phase_name", f"Phase {phase_number}") if outputs else f"Phase {phase_number}"
            tests_passing = outputs.get("tests_passing") if outputs else None
            tests_total = outputs.get("tests_total") if outputs else None
            components = outputs.get("components_completed", []) if outputs else []

            preserver.on_phase_complete(
                phase_number,
                phase_name,
                tests_passing=tests_passing,
                tests_total=tests_total,
                components_completed=components
            )
            print(f"   Progress preserved to git\n")
        except ImportError:
            print(f"   ⚠️ Progress preserver not available\n")
        except Exception as e:
            print(f"   ⚠️ Could not preserve progress: {e}\n")

    def fail_phase(
        self,
        phase_number: int,
        reason: str,
        details: str
    ):
        """Mark phase as incomplete/failed."""
        self.checkpoint_mgr.update_phase(
            phase_number,
            "incomplete",
            outputs={"failure_reason": reason, "details": details}
        )

        print(f"\n❌ Phase {phase_number} incomplete")
        print(f"   Reason: {reason}")
        print(f"   Checkpoint updated\n")

    def update_component_status(
        self,
        component_name: str,
        status: str,
        git_commit: str = None,
        tests_passing: bool = None,
        contract_compliant: bool = None
    ):
        """Update component state in checkpoint."""
        metadata = {}
        if git_commit:
            metadata["git_commit"] = git_commit
        if tests_passing is not None:
            metadata["tests_passing"] = tests_passing
        if contract_compliant is not None:
            metadata["contract_compliant"] = contract_compliant

        self.checkpoint_mgr.update_component(
            component_name,
            status,
            metadata
        )

    def stop_orchestration(
        self,
        reason: str,
        details: str,
        can_resume: bool = True
    ):
        """Stop orchestration and save stopping context."""
        self.checkpoint_mgr.mark_stopped(reason, details, can_resume)

        print(f"\n⏸️ Orchestration stopped")
        print(f"   Reason: {reason}")
        print(f"   Can resume: {'Yes' if can_resume else 'No'}")
        print(f"   Use '/orchestrate-full --resume' to continue\n")

    def complete_orchestration(self):
        """Mark orchestration as successfully completed."""
        self.checkpoint_mgr.mark_completed()

        print(f"\n🎉 Orchestration completed successfully!")
        print(f"   ID: {self.orchestration_id}")
        print(f"   Checkpoint: {self.checkpoint_mgr.checkpoint_path}\n")
