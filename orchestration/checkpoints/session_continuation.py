#!/usr/bin/env python3
"""
Session Continuation Protocol

Enables seamless continuation of large projects across multiple sessions
without losing progress or re-planning. This is critical for projects that
exceed a single context window.

Key insight: Projects should continue from EXACT state, not re-plan from scratch.
"""

import json
import sys
from datetime import datetime
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import Optional


@dataclass
class SessionState:
    """Complete state of an orchestration session."""
    session_id: str
    project_name: str
    specification_path: str
    current_phase: int
    total_phases: int
    features_total: int
    features_completed: int
    components_total: int
    components_completed: list[str]
    components_in_progress: list[str]
    components_remaining: list[str]
    tests_passing: int
    tests_total: int
    last_action: str
    next_action: str
    blocking_issues: list[str]
    context_usage_percentage: float
    timestamp: str
    resume_instructions: str


class SessionContinuationManager:
    """
    Manages session state persistence and continuation.

    Ensures projects can be resumed exactly where they left off,
    preventing the need to re-plan or restart from beginning.
    """

    def __init__(self, project_root: str):
        """Initialize session manager."""
        self.project_root = Path(project_root)
        self.state_dir = self.project_root / "orchestration" / "session_state"
        self.state_dir.mkdir(parents=True, exist_ok=True)
        self.current_state: Optional[SessionState] = None

    def create_checkpoint(
        self,
        session_id: str,
        project_name: str,
        specification_path: str,
        current_phase: int,
        total_phases: int,
        features_total: int,
        features_completed: int,
        components_total: int,
        components_completed: list[str],
        components_in_progress: list[str],
        components_remaining: list[str],
        tests_passing: int,
        tests_total: int,
        last_action: str,
        next_action: str,
        blocking_issues: list[str] = None,
        context_usage_percentage: float = 0.0
    ) -> str:
        """
        Create a checkpoint of current session state.

        Returns path to checkpoint file.
        """
        if blocking_issues is None:
            blocking_issues = []

        # Generate resume instructions
        resume_instructions = self._generate_resume_instructions(
            next_action,
            components_remaining,
            current_phase,
            total_phases,
            features_completed,
            features_total
        )

        state = SessionState(
            session_id=session_id,
            project_name=project_name,
            specification_path=specification_path,
            current_phase=current_phase,
            total_phases=total_phases,
            features_total=features_total,
            features_completed=features_completed,
            components_total=components_total,
            components_completed=components_completed,
            components_in_progress=components_in_progress,
            components_remaining=components_remaining,
            tests_passing=tests_passing,
            tests_total=tests_total,
            last_action=last_action,
            next_action=next_action,
            blocking_issues=blocking_issues,
            context_usage_percentage=context_usage_percentage,
            timestamp=datetime.now().isoformat(),
            resume_instructions=resume_instructions
        )

        self.current_state = state

        # Save to file
        checkpoint_file = self.state_dir / f"checkpoint_{session_id}.json"
        checkpoint_file.write_text(json.dumps(asdict(state), indent=2))

        # Also save as "latest"
        latest_file = self.state_dir / "latest_checkpoint.json"
        latest_file.write_text(json.dumps(asdict(state), indent=2))

        print(f"✓ Checkpoint saved: {checkpoint_file}")

        # Auto-commit checkpoint if progress preserver is available
        try:
            from progress_preserver import ProgressPreserver
            preserver = ProgressPreserver(str(self.project_root))
            preserver.on_checkpoint_save(session_id, context_usage_percentage)
        except ImportError:
            pass  # Progress preserver not available
        except Exception as e:
            print(f"  ⚠️ Could not auto-commit checkpoint: {e}")

        return str(checkpoint_file)

    def _generate_resume_instructions(
        self,
        next_action: str,
        components_remaining: list[str],
        current_phase: int,
        total_phases: int,
        features_completed: int,
        features_total: int
    ) -> str:
        """Generate human-readable resume instructions."""
        lines = [
            "=" * 60,
            "SESSION CONTINUATION INSTRUCTIONS",
            "=" * 60,
            "",
            "CRITICAL: This is a CONTINUATION, not a new project.",
            "DO NOT re-plan. DO NOT restart from Phase 1.",
            "CONTINUE from exact state below.",
            "",
            f"Current Progress:",
            f"  Phase: {current_phase}/{total_phases}",
            f"  Features: {features_completed}/{features_total} complete",
            f"  Remaining components: {len(components_remaining)}",
            "",
            f"NEXT ACTION (start immediately):",
            f"  {next_action}",
            "",
        ]

        if components_remaining:
            lines.extend([
                "Remaining Components:",
            ])
            for comp in components_remaining:
                lines.append(f"  - {comp}")
            lines.append("")

        lines.extend([
            "ANTI-STOPPING RULES APPLY:",
            "  - Rule 3: Phase continuity (do not stop at phase boundaries)",
            "  - Rule 5: Scope preservation (implement ALL remaining features)",
            "  - Rule 6: No rationalization (continue until 100% complete)",
            "",
            "When resuming, your first action should be:",
            f"  1. Confirm state: {current_phase}/{total_phases} phases",
            f"  2. Execute: {next_action}",
            "  3. Continue to next component without stopping",
            "",
            "=" * 60,
        ])

        return '\n'.join(lines)

    def load_latest_checkpoint(self) -> Optional[SessionState]:
        """Load the most recent checkpoint."""
        latest_file = self.state_dir / "latest_checkpoint.json"

        if not latest_file.exists():
            print("No checkpoint found.")
            return None

        try:
            data = json.loads(latest_file.read_text())
            state = SessionState(**data)
            self.current_state = state
            return state
        except Exception as e:
            print(f"Error loading checkpoint: {e}")
            return None

    def load_checkpoint(self, session_id: str) -> Optional[SessionState]:
        """Load a specific checkpoint by session ID."""
        checkpoint_file = self.state_dir / f"checkpoint_{session_id}.json"

        if not checkpoint_file.exists():
            print(f"Checkpoint not found: {session_id}")
            return None

        try:
            data = json.loads(checkpoint_file.read_text())
            state = SessionState(**data)
            self.current_state = state
            return state
        except Exception as e:
            print(f"Error loading checkpoint: {e}")
            return None

    def generate_resume_prompt(self) -> str:
        """
        Generate a prompt that can be used to resume the session.

        This prompt is designed to be pasted into a new Claude session
        to continue work exactly where it left off.
        """
        if not self.current_state:
            return "No session state loaded."

        state = self.current_state

        prompt = f"""# CONTINUATION SESSION - DO NOT RE-PLAN

## Session State
- Session ID: {state.session_id}
- Project: {state.project_name}
- Specification: {state.specification_path}

## Current Progress
- Phase: {state.current_phase}/{state.total_phases}
- Features: {state.features_completed}/{state.features_total} complete ({state.features_completed/max(1,state.features_total)*100:.1f}%)
- Tests: {state.tests_passing}/{state.tests_total} passing
- Last action: {state.last_action}

## CRITICAL INSTRUCTIONS

1. **DO NOT** re-plan or re-analyze the project
2. **DO NOT** generate a new project structure
3. **DO NOT** start from Phase 1
4. **CONTINUE** from current state immediately

## NEXT ACTION (execute immediately)
{state.next_action}

## Remaining Components
{chr(10).join('- ' + comp for comp in state.components_remaining)}

## Anti-Stopping Rules Active
- Rule 3: Phase Continuity - Continue through ALL phases without stopping
- Rule 5: Scope Preservation - Implement ALL remaining features
- Rule 6: No Rationalization - Do not justify stopping early
- Rule 7: Completion Gate - Only stop when 100% specification complete

## Blocking Issues to Resolve
{chr(10).join('- ' + issue for issue in state.blocking_issues) if state.blocking_issues else 'None'}

## Resume Command
After reading this prompt, immediately execute:
1. Verify current state matches what's described above
2. Begin: {state.next_action}
3. Continue to next task after completion
4. Do not stop until specification is 100% complete

**START NOW - No planning, just execute.**
"""
        return prompt

    def should_checkpoint(self, context_usage: float = 0.0) -> bool:
        """
        Determine if a checkpoint should be created.

        Returns True if checkpoint is recommended.
        """
        # Always checkpoint at these thresholds
        if context_usage >= 70:
            print(f"⚠️ Context usage at {context_usage}% - checkpoint recommended")
            return True

        # Checkpoint after completing components
        if self.current_state:
            completed = len(self.current_state.components_completed)
            # Checkpoint every 3 components
            if completed > 0 and completed % 3 == 0:
                return True

        return False

    def print_continuation_status(self):
        """Print current continuation status."""
        if not self.current_state:
            print("No active session state.")
            return

        state = self.current_state

        print("=" * 60)
        print("SESSION CONTINUATION STATUS")
        print("=" * 60)
        print(f"Session: {state.session_id}")
        print(f"Project: {state.project_name}")
        print(f"Last checkpoint: {state.timestamp}")
        print()
        print(f"Progress:")
        print(f"  Phase: {state.current_phase}/{state.total_phases}")
        print(f"  Features: {state.features_completed}/{state.features_total}")
        print(f"  Components: {len(state.components_completed)}/{state.components_total}")
        print(f"  Tests: {state.tests_passing}/{state.tests_total}")
        print()
        print(f"Next Action: {state.next_action}")
        print()
        print(state.resume_instructions)

    def list_checkpoints(self) -> list[str]:
        """List all available checkpoints."""
        checkpoints = []
        for checkpoint_file in self.state_dir.glob("checkpoint_*.json"):
            checkpoints.append(checkpoint_file.stem.replace("checkpoint_", ""))
        return sorted(checkpoints)


def main():
    """CLI entry point."""
    if len(sys.argv) < 2:
        print("Usage: python session_continuation.py <command> [args]")
        print()
        print("Commands:")
        print("  status           - Show current session status")
        print("  resume           - Generate resume prompt for new session")
        print("  list             - List all checkpoints")
        print("  load <id>        - Load specific checkpoint")
        print()
        print("Example:")
        print("  python session_continuation.py status")
        print("  python session_continuation.py resume > resume_prompt.md")
        sys.exit(1)

    command = sys.argv[1]
    project_root = sys.argv[2] if len(sys.argv) > 2 else "."

    manager = SessionContinuationManager(project_root)

    if command == "status":
        state = manager.load_latest_checkpoint()
        if state:
            manager.print_continuation_status()
        else:
            print("No checkpoints found. Start a new session.")

    elif command == "resume":
        state = manager.load_latest_checkpoint()
        if state:
            prompt = manager.generate_resume_prompt()
            print(prompt)
        else:
            print("No checkpoint to resume from.")
            sys.exit(1)

    elif command == "list":
        checkpoints = manager.list_checkpoints()
        if checkpoints:
            print("Available checkpoints:")
            for cp in checkpoints:
                print(f"  - {cp}")
        else:
            print("No checkpoints found.")

    elif command == "load":
        if len(sys.argv) < 3:
            print("Usage: python session_continuation.py load <session_id>")
            sys.exit(1)
        session_id = sys.argv[2]
        state = manager.load_checkpoint(session_id)
        if state:
            manager.print_continuation_status()
        else:
            sys.exit(1)

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == "__main__":
    main()
