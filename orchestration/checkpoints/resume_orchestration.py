#!/usr/bin/env python3
"""
Main resume logic - coordinates checkpoint vs discovery.
"""

import sys
from pathlib import Path
from typing import Dict, Any, Optional, List
import subprocess

# Add parent to path for standalone script execution
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.checkpoints.orchestration_checkpoint import OrchestrationCheckpointManager
from orchestration.checkpoints.state_discovery import StateDiscovery
from orchestration.checkpoints.resume_display import ResumeDisplay
from orchestration.checkpoints.resume_confirmation import ResumeConfirmation


class OrchestrationResume:
    """Coordinate orchestration resume using hybrid approach."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self.checkpoint_mgr = OrchestrationCheckpointManager(project_root)
        self.discovery = StateDiscovery(project_root)

    def resume(self, dry_run: bool = False) -> Optional[Dict[str, Any]]:
        """
        Resume orchestration using hybrid approach.

        Args:
            dry_run: If True, show what would happen without executing

        Returns resume context if user confirms, None otherwise.
        """
        if dry_run:
            print("\n🔍 DRY RUN MODE - No changes will be made\n")
        else:
            print("\n🔄 Attempting to resume orchestration...\n")

        # Try checkpoint first
        checkpoint = self.checkpoint_mgr.load_checkpoint_with_recovery()

        if checkpoint:
            return self._resume_from_checkpoint(checkpoint, dry_run)
        else:
            return self._resume_from_discovery(dry_run)

    def _resume_from_checkpoint(
        self,
        checkpoint: Dict[str, Any],
        dry_run: bool = False
    ) -> Optional[Dict[str, Any]]:
        """Resume using checkpoint data."""
        print("✅ Checkpoint found - resuming from saved state\n")

        # Validate state consistency
        inconsistencies = self._validate_state_consistency(checkpoint)
        if inconsistencies:
            print("\n⚠️ State Inconsistencies Detected:\n")
            for issue in inconsistencies:
                print(f"  - {issue}")
            print()
            print("These inconsistencies may indicate:")
            print("  - Files were moved/deleted manually")
            print("  - Git history was modified")
            print("  - Checkpoint is from different branch")
            print()

            if not dry_run:
                response = input("Continue anyway? [y/N]: ").strip().lower()
                if response not in ['y', 'yes']:
                    print("\n❌ Resume aborted due to state inconsistencies\n")
                    return None

        # Check for spec changes
        spec_changes = self._check_spec_changes(checkpoint)
        if spec_changes:
            print("\n⚠️ WARNING: Specification files changed since orchestration started!\n")
            for change in spec_changes:
                file = change["file"]
                change_type = change["change"]
                severity = change["severity"]

                icon = "🔴" if severity == "high" else "🟡"
                print(f"{icon} {file}: {change_type}")
                if "commits" in change:
                    print(f"   {change['commits']} commits since start")

        # Display status
        status_display = ResumeDisplay.format_checkpoint_status(checkpoint)

        # Generate resume plan
        current_phase = self._determine_resume_phase(checkpoint)
        phases = checkpoint["phase_progress"]["phases"]
        blocking_issues = self._identify_blocking_issues(checkpoint)

        plan_display = ResumeDisplay.format_resume_plan(
            current_phase,
            phases,
            blocking_issues
        )

        if dry_run:
            print("\n" + status_display)
            print(plan_display)
            print("\n✅ Dry run complete - no changes made")
            print("   Run without --dry-run to actually resume\n")
            return None

        # Ask confirmation
        if not ResumeConfirmation.confirm_resume(status_display, plan_display):
            print("\n❌ Resume cancelled by user\n")
            return None

        # Build resume context
        return {
            "source": "checkpoint",
            "orchestration_id": checkpoint["orchestration_id"],
            "current_phase": current_phase,
            "original_request": checkpoint["original_request"],
            "components": list(checkpoint["component_states"].keys()),
            "blocking_issues": blocking_issues,
            "phases": phases
        }

    def _resume_from_discovery(self, dry_run: bool = False) -> Optional[Dict[str, Any]]:
        """Resume using state discovery."""
        print("⚠️ No checkpoint found - discovering state from project...\n")

        # Run discovery
        discovery_result = self.discovery.discover_state()

        # Display discovered state
        status_display = ResumeDisplay.format_discovery_status(discovery_result)
        print(status_display)

        # Ask for additional context
        specs = ResumeConfirmation.ask_for_specs()
        goal = ResumeConfirmation.ask_for_goal()

        # Generate resume plan
        inferred = discovery_result["inferred_context"]
        current_phase = inferred.get("likely_current_phase", 1)

        # Create placeholder phases for planning
        phases = self._create_placeholder_phases(current_phase)

        plan_display = ResumeDisplay.format_resume_plan(
            current_phase,
            phases,
            []
        )

        # Ask confirmation
        print("\n" + status_display)
        if not ResumeConfirmation.confirm_resume(status_display, plan_display):
            print("\n❌ Resume cancelled by user\n")
            return None

        # Build resume context
        return {
            "source": "discovery",
            "orchestration_id": None,
            "current_phase": current_phase,
            "original_request": {
                "user_prompt": goal,
                "specification_files": specs
            },
            "components": discovery_result["discovered_state"].get("components_found", []),
            "blocking_issues": [],
            "discovery_result": discovery_result
        }

    def _determine_resume_phase(self, checkpoint: Dict[str, Any]) -> int:
        """
        Determine which phase to resume from.

        Finds first incomplete phase.
        """
        phases = checkpoint["phase_progress"]["phases"]

        for phase in phases:
            if phase["status"] in ["incomplete", "in_progress", "not_started"]:
                return phase["phase_number"]

        # All complete? Go to last phase
        return 6

    def _identify_blocking_issues(
        self,
        checkpoint: Dict[str, Any]
    ) -> List[str]:
        """Identify blocking issues from checkpoint."""
        issues = []

        # Check stopping context
        stopping = checkpoint.get("stopping_context", {})
        if stopping.get("reason"):
            issues.append(f"{stopping['reason']}: {stopping.get('details', 'Unknown')}")

        # Check phase gates
        phases = checkpoint["phase_progress"]["phases"]
        for phase in phases:
            if phase.get("gate_status") == "FAILED":
                reason = phase.get("blocking_reason", "Phase gate failed")
                issues.append(f"Phase {phase['phase_number']} gate: {reason}")

        # Check test failures
        for phase in phases:
            if "test_results" in phase:
                tests = phase["test_results"]
                if tests.get("failed", 0) > 0:
                    failures = tests.get("failures", [])
                    if failures:
                        for failure in failures[:3]:  # Show first 3
                            issues.append(f"Test failure: {failure.get('test', 'unknown')}")

        return issues

    def _create_placeholder_phases(self, current_phase: int) -> List[Dict[str, Any]]:
        """Create placeholder phase data for discovery mode."""
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
            status = "completed" if i < current_phase else "not_started"
            if i == current_phase:
                status = "incomplete"

            phases.append({
                "phase_number": i,
                "name": name,
                "status": status
            })

        return phases

    def _check_spec_changes(self, checkpoint: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Check if specification files changed since orchestration started.

        Returns list of changes detected.
        """
        changes = []

        original_specs = checkpoint["original_request"].get("specification_files", [])
        created_at = checkpoint["created_at"]

        for spec_path_str in original_specs:
            spec_path = self.project_root / spec_path_str

            if not spec_path.exists():
                changes.append({
                    "file": spec_path_str,
                    "change": "deleted",
                    "severity": "high"
                })
                continue

            # Check git history for modifications
            try:
                result = subprocess.run(
                    [
                        "git", "log",
                        "--since", created_at,
                        "--pretty=format:%H %s",
                        "--", str(spec_path)
                    ],
                    capture_output=True,
                    text=True,
                    cwd=self.project_root
                )

                commits = result.stdout.strip().split('\n')
                if commits and commits[0]:
                    changes.append({
                        "file": spec_path_str,
                        "change": "modified",
                        "severity": "medium",
                        "commits": len(commits),
                        "last_commit": commits[0]
                    })
            except:
                pass

        return changes

    def _validate_state_consistency(self, checkpoint: Dict[str, Any]) -> List[str]:
        """
        Validate that checkpoint state matches actual project state.

        Returns list of inconsistencies found.
        """
        inconsistencies = []

        # Check components exist
        component_states = checkpoint.get("component_states", {})
        for comp_name in component_states.keys():
            comp_dir = self.project_root / "components" / comp_name
            if not comp_dir.exists():
                inconsistencies.append(
                    f"Component '{comp_name}' in checkpoint but directory not found"
                )

        # Check for new components not in checkpoint
        components_dir = self.project_root / "components"
        if components_dir.exists():
            actual_comps = [
                d.name for d in components_dir.iterdir()
                if d.is_dir() and not d.name.startswith('.')
            ]
            for comp in actual_comps:
                if comp not in component_states:
                    inconsistencies.append(
                        f"Component '{comp}' exists but not in checkpoint"
                    )

        # Validate git commit hashes
        for comp_name, comp_state in component_states.items():
            if "git_commit" in comp_state:
                commit = comp_state["git_commit"]
                # Verify commit exists
                try:
                    result = subprocess.run(
                        ["git", "cat-file", "-t", commit],
                        capture_output=True,
                        cwd=self.project_root
                    )
                    if result.returncode != 0:
                        inconsistencies.append(
                            f"Git commit {commit} for {comp_name} not found"
                        )
                except:
                    pass

        return inconsistencies


def main():
    """Command-line entry point for resume."""
    import sys
    import json
    import argparse
    import logging

    # Setup logging
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler('orchestration-resume.log'),
            logging.StreamHandler()
        ]
    )
    logger = logging.getLogger(__name__)

    # Parse arguments
    parser = argparse.ArgumentParser(description="Resume orchestration")
    parser.add_argument(
        "project_root",
        nargs="?",
        default=".",
        help="Project root directory"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would happen without executing"
    )

    args = parser.parse_args()
    project_root = Path(args.project_root)

    logger.info(f"Starting resume process for {project_root}")

    resumer = OrchestrationResume(project_root)
    resume_context = resumer.resume(dry_run=args.dry_run)

    if resume_context:
        if not args.dry_run:
            # Write resume context to file for orchestrator to read
            context_file = project_root / "orchestration-resume-context.json"
            with open(context_file, 'w') as f:
                json.dump(resume_context, f, indent=2)

            print(f"\n✅ Resume context saved to: {context_file}")
            print("   Orchestrator will read this file to continue\n")
            logger.info(f"Resume context saved to {context_file}")

        return 0
    else:
        logger.info("Resume cancelled or failed")
        return 1


if __name__ == "__main__":
    exit(main())
