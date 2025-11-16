"""
Architecture Decision Record (ADR) Generator

Creates and manages Architecture Decision Records following best practices.
ADRs document significant architectural decisions and their rationale.

Classes:
    ADRGenerator: Main ADR management system
    ADR: Individual architecture decision record

Usage:
    generator = ADRGenerator()
    adr = generator.create_adr(
        title="Use PostgreSQL for user database",
        status="accepted",
        context="We need a database for storing user data...",
        decision="Use PostgreSQL 15 as the primary database",
        consequences="Strong ACID guarantees, excellent query performance..."
    )
"""

from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
import re


class ADRStatus(Enum):
    """Status of an ADR."""
    PROPOSED = "proposed"
    ACCEPTED = "accepted"
    REJECTED = "rejected"
    DEPRECATED = "deprecated"
    SUPERSEDED = "superseded"


@dataclass
class ADR:
    """Architecture Decision Record."""
    number: int
    title: str
    status: ADRStatus
    date: str
    context: str
    decision: str
    consequences: str
    superseded_by: Optional[int] = None
    supersedes: Optional[int] = None
    related_adrs: List[int] = None

    def __post_init__(self):
        """Initialize related_adrs if None."""
        if self.related_adrs is None:
            self.related_adrs = []

    def to_markdown(self) -> str:
        """
        Convert ADR to Markdown format.

        Returns:
            Markdown string
        """
        lines = [
            f"# ADR-{self.number:03d}: {self.title}",
            "",
            f"**Date**: {self.date}",
            "",
            "## Status",
            "",
            self.status.value.upper(),
            ""
        ]

        if self.superseded_by:
            lines.extend([
                f"*Superseded by [ADR-{self.superseded_by:03d}](ADR-{self.superseded_by:03d}.md)*",
                ""
            ])

        if self.supersedes:
            lines.extend([
                f"*Supersedes [ADR-{self.supersedes:03d}](ADR-{self.supersedes:03d}.md)*",
                ""
            ])

        lines.extend([
            "## Context",
            "",
            self.context,
            "",
            "## Decision",
            "",
            self.decision,
            "",
            "## Consequences",
            "",
            self.consequences,
            ""
        ])

        if self.related_adrs:
            lines.extend([
                "## Related ADRs",
                ""
            ])
            for adr_num in self.related_adrs:
                lines.append(f"- [ADR-{adr_num:03d}](ADR-{adr_num:03d}.md)")
            lines.append("")

        return "\n".join(lines)

    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'number': self.number,
            'title': self.title,
            'status': self.status.value,
            'date': self.date,
            'context': self.context,
            'decision': self.decision,
            'consequences': self.consequences,
            'superseded_by': self.superseded_by,
            'supersedes': self.supersedes,
            'related_adrs': self.related_adrs
        }


class ADRGenerator:
    """
    Generate and manage Architecture Decision Records.

    ADRs are stored in docs/adr/ directory.
    Each ADR has a unique number and follows a standard format.

    Attributes:
        adr_dir: Directory where ADRs are stored
    """

    def __init__(self, adr_dir: Optional[Path] = None):
        """
        Initialize ADRGenerator.

        Args:
            adr_dir: Directory for ADR storage (default: docs/adr/)
        """
        if adr_dir is None:
            self.adr_dir = Path("docs/adr")
        else:
            self.adr_dir = Path(adr_dir)

        # Create directory if it doesn't exist
        self.adr_dir.mkdir(parents=True, exist_ok=True)

        # Create index if it doesn't exist
        index_path = self.adr_dir / "README.md"
        if not index_path.exists():
            self._create_index()

    def create_adr(
        self,
        title: str,
        status: str,
        context: str,
        decision: str,
        consequences: str,
        supersedes: Optional[int] = None,
        related_adrs: Optional[List[int]] = None
    ) -> ADR:
        """
        Create a new Architecture Decision Record.

        Args:
            title: Short title describing the decision
            status: Status (proposed, accepted, rejected, deprecated, superseded)
            context: Background and context for the decision
            decision: The decision that was made
            consequences: Positive and negative consequences of the decision
            supersedes: ADR number this supersedes (if any)
            related_adrs: List of related ADR numbers

        Returns:
            ADR instance

        Example:
            >>> gen = ADRGenerator()
            >>> adr = gen.create_adr(
            ...     title="Use PostgreSQL for user database",
            ...     status="accepted",
            ...     context="We need a database...",
            ...     decision="Use PostgreSQL 15",
            ...     consequences="Strong ACID guarantees..."
            ... )
        """
        # Get next ADR number
        number = self._get_next_adr_number()

        # Parse status
        try:
            status_enum = ADRStatus(status.lower())
        except ValueError:
            raise ValueError(f"Invalid status: {status}. Must be one of: "
                           f"{', '.join(s.value for s in ADRStatus)}")

        # Create ADR
        adr = ADR(
            number=number,
            title=title,
            status=status_enum,
            date=datetime.now().strftime("%Y-%m-%d"),
            context=context,
            decision=decision,
            consequences=consequences,
            supersedes=supersedes,
            related_adrs=related_adrs or []
        )

        # If supersedes another ADR, update that ADR
        if supersedes:
            self._mark_adr_superseded(supersedes, number)

        # Save ADR
        self._save_adr(adr)

        # Update index
        self._update_index()

        return adr

    def get_adr(self, number: int) -> Optional[ADR]:
        """
        Get ADR by number.

        Args:
            number: ADR number

        Returns:
            ADR instance or None if not found
        """
        file_path = self.adr_dir / f"ADR-{number:03d}.md"

        if not file_path.exists():
            return None

        return self._parse_adr(file_path)

    def list_adrs(self, status: Optional[str] = None) -> List[ADR]:
        """
        List all ADRs, optionally filtered by status.

        Args:
            status: Filter by status (proposed, accepted, rejected, etc.)

        Returns:
            List of ADR instances
        """
        adrs = []

        for file_path in sorted(self.adr_dir.glob("ADR-*.md")):
            adr = self._parse_adr(file_path)
            if adr:
                if status is None or adr.status.value == status.lower():
                    adrs.append(adr)

        return adrs

    def update_adr_status(
        self,
        number: int,
        new_status: str,
        superseded_by: Optional[int] = None
    ):
        """
        Update the status of an ADR.

        Args:
            number: ADR number
            new_status: New status
            superseded_by: ADR number that supersedes this one (if status=superseded)
        """
        adr = self.get_adr(number)

        if not adr:
            raise ValueError(f"ADR-{number:03d} not found")

        # Update status
        try:
            adr.status = ADRStatus(new_status.lower())
        except ValueError:
            raise ValueError(f"Invalid status: {new_status}")

        if superseded_by:
            adr.superseded_by = superseded_by

        # Save updated ADR
        self._save_adr(adr)

        # Update index
        self._update_index()

    def _get_next_adr_number(self) -> int:
        """Get the next available ADR number."""
        existing_adrs = list(self.adr_dir.glob("ADR-*.md"))

        if not existing_adrs:
            return 1

        # Extract numbers from filenames
        numbers = []
        for file_path in existing_adrs:
            match = re.search(r'ADR-(\d+)\.md', file_path.name)
            if match:
                numbers.append(int(match.group(1)))

        return max(numbers) + 1 if numbers else 1

    def _save_adr(self, adr: ADR):
        """Save ADR to file."""
        file_path = self.adr_dir / f"ADR-{adr.number:03d}.md"
        file_path.write_text(adr.to_markdown())

    def _parse_adr(self, file_path: Path) -> Optional[ADR]:
        """Parse ADR from Markdown file."""
        try:
            content = file_path.read_text()

            # Extract number from filename
            match = re.search(r'ADR-(\d+)\.md', file_path.name)
            if not match:
                return None
            number = int(match.group(1))

            # Extract title
            title_match = re.search(r'^# ADR-\d+: (.+)$', content, re.MULTILINE)
            title = title_match.group(1) if title_match else "Unknown"

            # Extract date
            date_match = re.search(r'^\*\*Date\*\*: (.+)$', content, re.MULTILINE)
            date = date_match.group(1) if date_match else datetime.now().strftime("%Y-%m-%d")

            # Extract status
            status_match = re.search(r'^## Status\n\n(.+)$', content, re.MULTILINE)
            if status_match:
                status_str = status_match.group(1).strip().lower()
                # Remove markdown formatting
                status_str = re.sub(r'\*.*\*', '', status_str).strip()
                try:
                    status = ADRStatus(status_str)
                except ValueError:
                    status = ADRStatus.PROPOSED
            else:
                status = ADRStatus.PROPOSED

            # Extract sections
            context_match = re.search(r'## Context\n\n(.+?)(?=\n## |\Z)', content, re.DOTALL)
            context = context_match.group(1).strip() if context_match else ""

            decision_match = re.search(r'## Decision\n\n(.+?)(?=\n## |\Z)', content, re.DOTALL)
            decision = decision_match.group(1).strip() if decision_match else ""

            consequences_match = re.search(r'## Consequences\n\n(.+?)(?=\n## |\Z)', content, re.DOTALL)
            consequences = consequences_match.group(1).strip() if consequences_match else ""

            # Extract superseded_by
            superseded_by = None
            superseded_match = re.search(r'Superseded by \[ADR-(\d+)\]', content)
            if superseded_match:
                superseded_by = int(superseded_match.group(1))

            # Extract supersedes
            supersedes = None
            supersedes_match = re.search(r'Supersedes \[ADR-(\d+)\]', content)
            if supersedes_match:
                supersedes = int(supersedes_match.group(1))

            # Extract related ADRs
            related_adrs = []
            related_section = re.search(r'## Related ADRs\n\n(.+?)(?=\n## |\Z)', content, re.DOTALL)
            if related_section:
                related_matches = re.findall(r'ADR-(\d+)', related_section.group(1))
                related_adrs = [int(num) for num in related_matches]

            return ADR(
                number=number,
                title=title,
                status=status,
                date=date,
                context=context,
                decision=decision,
                consequences=consequences,
                superseded_by=superseded_by,
                supersedes=supersedes,
                related_adrs=related_adrs
            )

        except Exception as e:
            print(f"Error parsing {file_path}: {e}")
            return None

    def _mark_adr_superseded(self, adr_number: int, superseded_by: int):
        """Mark an ADR as superseded."""
        adr = self.get_adr(adr_number)

        if adr:
            adr.status = ADRStatus.SUPERSEDED
            adr.superseded_by = superseded_by
            self._save_adr(adr)

    def _create_index(self):
        """Create initial index file."""
        index_content = """# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for this project.

## About ADRs

An Architecture Decision Record (ADR) is a document that captures an important architectural decision made along with its context and consequences.

## Format

Each ADR contains:
- **Title**: Short, descriptive title
- **Status**: proposed, accepted, rejected, deprecated, or superseded
- **Date**: When the decision was made
- **Context**: Background and context for the decision
- **Decision**: The decision that was made
- **Consequences**: Positive and negative consequences

## ADRs

<!-- This section is automatically updated -->

"""
        (self.adr_dir / "README.md").write_text(index_content)

    def _update_index(self):
        """Update the index with current ADRs."""
        adrs = self.list_adrs()

        # Group by status
        by_status = {
            'accepted': [],
            'proposed': [],
            'deprecated': [],
            'superseded': [],
            'rejected': []
        }

        for adr in adrs:
            by_status[adr.status.value].append(adr)

        # Build index content
        lines = [
            "# Architecture Decision Records",
            "",
            "This directory contains Architecture Decision Records (ADRs) for this project.",
            "",
            "## About ADRs",
            "",
            "An Architecture Decision Record (ADR) is a document that captures an important "
            "architectural decision made along with its context and consequences.",
            "",
            "## Format",
            "",
            "Each ADR contains:",
            "- **Title**: Short, descriptive title",
            "- **Status**: proposed, accepted, rejected, deprecated, or superseded",
            "- **Date**: When the decision was made",
            "- **Context**: Background and context for the decision",
            "- **Decision**: The decision that was made",
            "- **Consequences**: Positive and negative consequences",
            "",
            "## ADRs",
            ""
        ]

        # Add accepted ADRs
        if by_status['accepted']:
            lines.extend([
                "### Accepted",
                ""
            ])
            for adr in by_status['accepted']:
                lines.append(f"- [ADR-{adr.number:03d}](ADR-{adr.number:03d}.md): {adr.title}")
            lines.append("")

        # Add proposed ADRs
        if by_status['proposed']:
            lines.extend([
                "### Proposed",
                ""
            ])
            for adr in by_status['proposed']:
                lines.append(f"- [ADR-{adr.number:03d}](ADR-{adr.number:03d}.md): {adr.title}")
            lines.append("")

        # Add deprecated/superseded ADRs
        if by_status['deprecated'] or by_status['superseded']:
            lines.extend([
                "### Deprecated / Superseded",
                ""
            ])
            for adr in by_status['deprecated'] + by_status['superseded']:
                lines.append(f"- [ADR-{adr.number:03d}](ADR-{adr.number:03d}.md): {adr.title}")
            lines.append("")

        # Add rejected ADRs
        if by_status['rejected']:
            lines.extend([
                "### Rejected",
                ""
            ])
            for adr in by_status['rejected']:
                lines.append(f"- [ADR-{adr.number:03d}](ADR-{adr.number:03d}.md): {adr.title}")
            lines.append("")

        (self.adr_dir / "README.md").write_text("\n".join(lines))


def main():
    """CLI interface for ADR generation."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python adr_generator.py <command> [args]")
        print("")
        print("Commands:")
        print("  create <title> <status> - Create new ADR interactively")
        print("  list [status]           - List all ADRs (optionally filter by status)")
        print("  show <number>           - Show specific ADR")
        print("  update-status <number> <new_status> [superseded_by]")
        sys.exit(1)

    command = sys.argv[1]
    generator = ADRGenerator()

    if command == "create":
        if len(sys.argv) < 4:
            print("Usage: ... create <title> <status>")
            print("Status: proposed, accepted, rejected, deprecated, superseded")
            sys.exit(1)

        title = sys.argv[2]
        status = sys.argv[3]

        # Interactive prompts for context, decision, consequences
        print("Enter context (end with empty line):")
        context_lines = []
        while True:
            line = input()
            if not line:
                break
            context_lines.append(line)
        context = "\n".join(context_lines)

        print("\nEnter decision (end with empty line):")
        decision_lines = []
        while True:
            line = input()
            if not line:
                break
            decision_lines.append(line)
        decision = "\n".join(decision_lines)

        print("\nEnter consequences (end with empty line):")
        consequences_lines = []
        while True:
            line = input()
            if not line:
                break
            consequences_lines.append(line)
        consequences = "\n".join(consequences_lines)

        adr = generator.create_adr(
            title=title,
            status=status,
            context=context,
            decision=decision,
            consequences=consequences
        )

        print(f"\nCreated ADR-{adr.number:03d}: {adr.title}")
        print(f"File: docs/adr/ADR-{adr.number:03d}.md")

    elif command == "list":
        status_filter = sys.argv[2] if len(sys.argv) > 2 else None
        adrs = generator.list_adrs(status=status_filter)

        print(f"Architecture Decision Records{' (' + status_filter + ')' if status_filter else ''}:")
        print("=" * 60)

        for adr in adrs:
            print(f"ADR-{adr.number:03d}: {adr.title}")
            print(f"  Status: {adr.status.value}")
            print(f"  Date: {adr.date}")
            print()

    elif command == "show":
        if len(sys.argv) < 3:
            print("Usage: ... show <number>")
            sys.exit(1)

        number = int(sys.argv[2])
        adr = generator.get_adr(number)

        if not adr:
            print(f"ADR-{number:03d} not found")
            sys.exit(1)

        print(adr.to_markdown())

    elif command == "update-status":
        if len(sys.argv) < 4:
            print("Usage: ... update-status <number> <new_status> [superseded_by]")
            sys.exit(1)

        number = int(sys.argv[2])
        new_status = sys.argv[3]
        superseded_by = int(sys.argv[4]) if len(sys.argv) > 4 else None

        generator.update_adr_status(number, new_status, superseded_by)
        print(f"Updated ADR-{number:03d} status to {new_status}")

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == "__main__":
    main()
