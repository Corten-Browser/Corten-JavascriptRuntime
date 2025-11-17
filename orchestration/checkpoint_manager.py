#!/usr/bin/env python3
"""
Checkpoint Manager

Enables agents to save progress and resume from exact point when time limits are reached.

This solves the v0.2.0 problem where agents that ran out of time had to restart
from scratch, wasting time redoing completed work.

Part of v0.3.0 completion guarantee system.
"""

import json
import sys
from pathlib import Path
from typing import List, Optional, Dict
from dataclasses import dataclass, asdict
from datetime import datetime


@dataclass
class Checkpoint:
    """Checkpoint data for agent progress."""
    component_name: str
    iteration: int
    timestamp: str
    completed_tasks: List[str]
    remaining_tasks: List[str]
    time_spent_minutes: int
    estimated_remaining_minutes: int
    blocking_issues: List[str]
    context_summary: str
    files_modified: List[str]
    tests_status: str  # "passing", "failing", "not_run"
    coverage_percentage: Optional[int]

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> 'Checkpoint':
        """Create from dictionary."""
        return cls(**data)


class CheckpointManager:
    """Manages checkpoint saving and resuming for components."""

    def __init__(self, project_root: Path):
        """
        Initialize checkpoint manager.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()
        self.checkpoints_dir = self.project_root / "orchestration" / "checkpoints"
        self.checkpoints_dir.mkdir(parents=True, exist_ok=True)

    def save_checkpoint(self, checkpoint: Checkpoint) -> Path:
        """
        Save checkpoint for a component.

        Args:
            checkpoint: Checkpoint data to save

        Returns:
            Path to saved checkpoint file
        """
        # Create component checkpoint directory
        component_dir = self.checkpoints_dir / checkpoint.component_name
        component_dir.mkdir(exist_ok=True)

        # Checkpoint filename with iteration number
        checkpoint_file = component_dir / f"checkpoint_{checkpoint.iteration}.json"

        # Save checkpoint
        with open(checkpoint_file, 'w') as f:
            json.dump(checkpoint.to_dict(), f, indent=2)

        # Update latest symlink
        latest_link = component_dir / "latest.json"
        if latest_link.exists():
            latest_link.unlink()

        # Copy to latest (not symlink for cross-platform compatibility)
        with open(latest_link, 'w') as f:
            json.dump(checkpoint.to_dict(), f, indent=2)

        print(f"💾 Checkpoint saved: {checkpoint.component_name} iteration {checkpoint.iteration}")
        print(f"   Location: {checkpoint_file}")

        return checkpoint_file

    def load_checkpoint(self, component_name: str, iteration: Optional[int] = None) -> Optional[Checkpoint]:
        """
        Load checkpoint for a component.

        Args:
            component_name: Name of component
            iteration: Specific iteration to load (None = latest)

        Returns:
            Checkpoint object or None if not found
        """
        component_dir = self.checkpoints_dir / component_name

        if not component_dir.exists():
            return None

        # Load specific iteration or latest
        if iteration is not None:
            checkpoint_file = component_dir / f"checkpoint_{iteration}.json"
        else:
            checkpoint_file = component_dir / "latest.json"

        if not checkpoint_file.exists():
            return None

        try:
            with open(checkpoint_file, 'r') as f:
                data = json.load(f)

            return Checkpoint.from_dict(data)

        except Exception as e:
            print(f"⚠️  Error loading checkpoint: {e}")
            return None

    def get_all_checkpoints(self, component_name: str) -> List[Checkpoint]:
        """
        Get all checkpoints for a component.

        Args:
            component_name: Name of component

        Returns:
            List of checkpoints, sorted by iteration
        """
        component_dir = self.checkpoints_dir / component_name

        if not component_dir.exists():
            return []

        checkpoints = []

        for checkpoint_file in component_dir.glob("checkpoint_*.json"):
            try:
                with open(checkpoint_file, 'r') as f:
                    data = json.load(f)

                checkpoints.append(Checkpoint.from_dict(data))

            except Exception:
                continue

        return sorted(checkpoints, key=lambda c: c.iteration)

    def delete_checkpoints(self, component_name: str) -> int:
        """
        Delete all checkpoints for a component.

        Args:
            component_name: Name of component

        Returns:
            Number of checkpoints deleted
        """
        component_dir = self.checkpoints_dir / component_name

        if not component_dir.exists():
            return 0

        count = 0

        for checkpoint_file in component_dir.glob("*.json"):
            checkpoint_file.unlink()
            count += 1

        # Remove directory if empty
        try:
            component_dir.rmdir()
        except:
            pass

        return count

    def generate_resume_prompt(self, checkpoint: Checkpoint) -> str:
        """
        Generate resume prompt for agent to continue from checkpoint.

        Args:
            checkpoint: Checkpoint to resume from

        Returns:
            Formatted prompt for agent
        """
        prompt = f"""# RESUMING FROM CHECKPOINT

You are continuing work on the **{checkpoint.component_name}** component.

## Previous Progress (Iteration {checkpoint.iteration})

**Time Spent**: {checkpoint.time_spent_minutes} minutes
**Estimated Remaining**: {checkpoint.estimated_remaining_minutes} minutes

### ✅ Completed Tasks

{self._format_task_list(checkpoint.completed_tasks)}

### 📋 Remaining Tasks

{self._format_task_list(checkpoint.remaining_tasks)}

### 📂 Files Modified So Far

{self._format_file_list(checkpoint.files_modified)}

### 🧪 Test Status

**Status**: {checkpoint.tests_status}
{"**Coverage**: " + str(checkpoint.coverage_percentage) + "%" if checkpoint.coverage_percentage else "**Coverage**: Not measured yet"}

## Context Summary

{checkpoint.context_summary}

## Blocking Issues

{self._format_blocking_issues(checkpoint.blocking_issues)}

---

## Your Task

**DO NOT** redo the completed tasks above. They are already done.

**FOCUS ON** the remaining tasks listed above.

**CONTINUE FROM** where the previous iteration left off.

Start by reviewing the files listed above to understand the current state,
then proceed with the next remaining task.

When you complete all remaining tasks:
1. Run all tests
2. Verify coverage ≥ 80%
3. Mark component as complete

If you encounter new blocking issues, document them clearly.
If you run out of time, create a new checkpoint with updated progress.
"""

        return prompt

    def _format_task_list(self, tasks: List[str]) -> str:
        """Format task list for prompt."""
        if not tasks:
            return "(None)"

        return "\n".join(f"{i+1}. {task}" for i, task in enumerate(tasks))

    def _format_file_list(self, files: List[str]) -> str:
        """Format file list for prompt."""
        if not files:
            return "(No files modified yet)"

        return "\n".join(f"- {file}" for file in files)

    def _format_blocking_issues(self, issues: List[str]) -> str:
        """Format blocking issues for prompt."""
        if not issues:
            return "**No blocking issues** (good progress!)"

        formatted = "\n".join(f"- ⚠️  {issue}" for issue in issues)
        return f"**CRITICAL**: The following issues are blocking progress:\n\n{formatted}"

    def create_checkpoint_from_agent_report(
        self,
        component_name: str,
        iteration: int,
        agent_report: str,
        time_spent_minutes: int
    ) -> Checkpoint:
        """
        Create checkpoint from agent's final report.

        This is a helper to parse agent reports and create checkpoints.

        Args:
            component_name: Name of component
            iteration: Iteration number
            agent_report: Agent's status report text
            time_spent_minutes: Time agent spent working

        Returns:
            Checkpoint object (not yet saved)
        """
        # Parse agent report to extract tasks
        # This is a simple implementation - can be enhanced with LLM parsing

        completed_tasks = self._extract_completed_tasks(agent_report)
        remaining_tasks = self._extract_remaining_tasks(agent_report)
        blocking_issues = self._extract_blocking_issues(agent_report)
        files_modified = self._extract_modified_files(agent_report)
        tests_status = self._extract_tests_status(agent_report)
        coverage = self._extract_coverage(agent_report)

        # Estimate remaining time (simple heuristic)
        estimated_remaining = len(remaining_tasks) * 15  # 15 min per task

        return Checkpoint(
            component_name=component_name,
            iteration=iteration,
            timestamp=datetime.now().isoformat(),
            completed_tasks=completed_tasks,
            remaining_tasks=remaining_tasks,
            time_spent_minutes=time_spent_minutes,
            estimated_remaining_minutes=estimated_remaining,
            blocking_issues=blocking_issues,
            context_summary=self._extract_context_summary(agent_report),
            files_modified=files_modified,
            tests_status=tests_status,
            coverage_percentage=coverage
        )

    def _extract_completed_tasks(self, report: str) -> List[str]:
        """Extract completed tasks from agent report."""
        # Simple parsing - look for common patterns
        completed = []

        lines = report.split('\n')
        in_completed_section = False

        for line in lines:
            line = line.strip()

            # Detect section headers
            if 'completed' in line.lower() and any(c in line for c in [':', '#', '*']):
                in_completed_section = True
                continue

            if in_completed_section:
                # Stop at next section
                if line.startswith('#') or (line.startswith('-') and 'remaining' in line.lower()):
                    in_completed_section = False
                    continue

                # Extract task
                if line.startswith(('- ', '* ', '✓', '✅')):
                    task = line.lstrip('- *✓✅ ').strip()
                    if task:
                        completed.append(task)

        return completed or ["Work in progress"]

    def _extract_remaining_tasks(self, report: str) -> List[str]:
        """Extract remaining tasks from agent report."""
        remaining = []

        lines = report.split('\n')
        in_remaining_section = False

        for line in lines:
            line = line.strip()

            # Detect section headers
            if 'remaining' in line.lower() or 'todo' in line.lower() or 'next' in line.lower():
                if any(c in line for c in [':', '#', '*']):
                    in_remaining_section = True
                    continue

            if in_remaining_section:
                # Stop at next section
                if line.startswith('#'):
                    in_remaining_section = False
                    continue

                # Extract task
                if line.startswith(('- ', '* ', '☐', '⬜', '[]')):
                    task = line.lstrip('- *☐⬜[] ').strip()
                    if task:
                        remaining.append(task)

        return remaining or ["Complete remaining work"]

    def _extract_blocking_issues(self, report: str) -> List[str]:
        """Extract blocking issues from agent report."""
        issues = []

        lines = report.split('\n')

        for line in lines:
            line_lower = line.lower()

            if any(keyword in line_lower for keyword in ['blocked', 'blocking', 'issue', 'problem', 'error']):
                # Extract the issue
                issue = line.strip().lstrip('- *⚠️  ')
                if issue and len(issue) > 10:  # Meaningful issue
                    issues.append(issue)

        return issues

    def _extract_modified_files(self, report: str) -> List[str]:
        """Extract list of modified files from report."""
        files = []

        lines = report.split('\n')

        for line in lines:
            # Look for file paths
            if '/' in line or '\\' in line:
                # Simple heuristic: if line contains path separators
                # Extract what looks like a file path
                parts = line.split()
                for part in parts:
                    if ('/' in part or '\\' in part) and '.' in part:
                        files.append(part.strip('`"\','))

        return list(set(files))  # Remove duplicates

    def _extract_tests_status(self, report: str) -> str:
        """Extract test status from report."""
        report_lower = report.lower()

        if 'tests passing' in report_lower or 'all tests pass' in report_lower:
            return "passing"
        elif 'tests failing' in report_lower or 'test fail' in report_lower:
            return "failing"
        else:
            return "not_run"

    def _extract_coverage(self, report: str) -> Optional[int]:
        """Extract test coverage percentage from report."""
        import re

        # Look for coverage percentage
        match = re.search(r'coverage[:\s]+(\d+)%', report, re.IGNORECASE)

        if match:
            return int(match.group(1))

        return None

    def _extract_context_summary(self, report: str) -> str:
        """Extract context summary from report."""
        # Take first few paragraphs as context
        lines = report.split('\n')
        summary_lines = []

        for line in lines[:20]:  # First 20 lines
            if line.strip() and not line.strip().startswith('#'):
                summary_lines.append(line)

            if len(summary_lines) >= 5:
                break

        return '\n'.join(summary_lines) if summary_lines else "No context summary available"


def main():
    """CLI interface for checkpoint manager."""
    if len(sys.argv) < 2:
        print("Usage: checkpoint_manager.py <command> [args]")
        print("\nCommands:")
        print("  list <component>           - List all checkpoints for component")
        print("  load <component> [iter]    - Load checkpoint (latest or specific iteration)")
        print("  resume <component> [iter]  - Generate resume prompt")
        print("  delete <component>         - Delete all checkpoints for component")
        print("\nExamples:")
        print("  python checkpoint_manager.py list audio_processor")
        print("  python checkpoint_manager.py load audio_processor")
        print("  python checkpoint_manager.py resume audio_processor 2")
        sys.exit(1)

    command = sys.argv[1]
    project_root = Path.cwd()
    manager = CheckpointManager(project_root)

    if command == "list":
        if len(sys.argv) < 3:
            print("Error: Component name required")
            sys.exit(1)

        component_name = sys.argv[2]
        checkpoints = manager.get_all_checkpoints(component_name)

        if not checkpoints:
            print(f"No checkpoints found for {component_name}")
            sys.exit(0)

        print(f"Checkpoints for {component_name}:")
        print()

        for checkpoint in checkpoints:
            print(f"Iteration {checkpoint.iteration}:")
            print(f"  Timestamp: {checkpoint.timestamp}")
            print(f"  Time spent: {checkpoint.time_spent_minutes} min")
            print(f"  Completed: {len(checkpoint.completed_tasks)} task(s)")
            print(f"  Remaining: {len(checkpoint.remaining_tasks)} task(s)")
            print(f"  Tests: {checkpoint.tests_status}")
            print()

    elif command == "load":
        if len(sys.argv) < 3:
            print("Error: Component name required")
            sys.exit(1)

        component_name = sys.argv[2]
        iteration = int(sys.argv[3]) if len(sys.argv) > 3 else None

        checkpoint = manager.load_checkpoint(component_name, iteration)

        if not checkpoint:
            print(f"No checkpoint found for {component_name}")
            sys.exit(1)

        print(json.dumps(checkpoint.to_dict(), indent=2))

    elif command == "resume":
        if len(sys.argv) < 3:
            print("Error: Component name required")
            sys.exit(1)

        component_name = sys.argv[2]
        iteration = int(sys.argv[3]) if len(sys.argv) > 3 else None

        checkpoint = manager.load_checkpoint(component_name, iteration)

        if not checkpoint:
            print(f"No checkpoint found for {component_name}")
            sys.exit(1)

        resume_prompt = manager.generate_resume_prompt(checkpoint)
        print(resume_prompt)

    elif command == "delete":
        if len(sys.argv) < 3:
            print("Error: Component name required")
            sys.exit(1)

        component_name = sys.argv[2]
        count = manager.delete_checkpoints(component_name)

        print(f"Deleted {count} checkpoint(s) for {component_name}")

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == '__main__':
    main()
