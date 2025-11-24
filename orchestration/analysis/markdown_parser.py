#!/usr/bin/env python3
"""
Markdown Parser for README Testing

Extracts and categorizes code blocks from Markdown files.
Supports:
- Fenced code blocks (```language)
- Skip markers
- Section extraction
- Multi-line code
"""

from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional
import re


@dataclass
class CodeBlock:
    """Represents a code block extracted from markdown."""
    language: Optional[str]  # 'bash', 'python', 'pycon', etc.
    content: str
    line_number: int
    section: Optional[str] = None  # e.g., "Quick Start"
    skip: bool = False

    def __repr__(self):
        return (
            f"CodeBlock(language={self.language}, "
            f"line={self.line_number}, "
            f"skip={self.skip})"
        )


class MarkdownParser:
    """Extract and categorize code blocks from Markdown files."""

    # Regex for fenced code blocks: ```language\n...\n```
    FENCE_PATTERN = re.compile(
        r'^```(\w+)?\s*\n(.*?)^```\s*$',
        re.MULTILINE | re.DOTALL
    )

    # Section headers: # Header, ## Header, etc.
    HEADER_PATTERN = re.compile(r'^#+\s+(.+)$', re.MULTILINE)

    # Skip markers
    SKIP_MARKERS = [
        '<!-- skip-test -->',
        '<!-- no-test -->',
        '# doctest: +SKIP',
        '# pragma: no readme test'
    ]

    # Language normalization
    LANGUAGE_ALIASES = {
        'sh': 'bash',
        'shell': 'bash',
        'console': 'bash',
        'py': 'python',
        'pycon': 'python',  # >>> format
    }

    def __init__(self, readme_path: Path):
        """
        Initialize parser with README path.

        Args:
            readme_path: Path to README.md file
        """
        self.readme_path = Path(readme_path)

        if not self.readme_path.exists():
            raise FileNotFoundError(f"README not found: {readme_path}")

        with open(self.readme_path, 'r', encoding='utf-8') as f:
            self._content = f.read()

        self._lines = self._content.split('\n')

    def extract_code_blocks(
        self,
        languages: Optional[List[str]] = None,
        sections: Optional[List[str]] = None
    ) -> List[CodeBlock]:
        """
        Extract code blocks from README.

        Args:
            languages: Filter by language (e.g., ['bash', 'python'])
            sections: Extract only from specific sections (e.g., ['Quick Start'])

        Returns:
            List of CodeBlock objects
        """
        blocks = []

        # Find all fenced code blocks
        for match in self.FENCE_PATTERN.finditer(self._content):
            language = match.group(1)  # May be None
            content = match.group(2).rstrip()

            # Normalize language
            language = self._normalize_language(language)

            # Filter by language if specified
            if languages and language not in languages:
                continue

            # Find line number
            line_number = self._content[:match.start()].count('\n') + 1

            # Extract section
            section = self._extract_section_name(line_number)

            # Filter by section if specified
            if sections and section not in sections:
                continue

            # Check for skip markers
            skip = self._should_skip_block(content, match.start(), match.end())

            blocks.append(CodeBlock(
                language=language,
                content=content,
                line_number=line_number,
                section=section,
                skip=skip
            ))

        return blocks

    def _normalize_language(self, lang: Optional[str]) -> Optional[str]:
        """
        Normalize language identifiers.

        Examples:
            'sh' → 'bash'
            'shell' → 'bash'
            'py' → 'python'
        """
        if lang is None:
            return None

        lang = lang.lower()
        return self.LANGUAGE_ALIASES.get(lang, lang)

    def _should_skip_block(
        self,
        block_content: str,
        block_start: int,
        block_end: int
    ) -> bool:
        """
        Check if block has skip markers.

        Checks:
        1. Within the code block content
        2. In surrounding lines (comment before block)
        """
        # Check within content
        for marker in self.SKIP_MARKERS:
            if marker in block_content:
                return True

        # Check 3 lines before block
        start_line = self._content[:block_start].count('\n')
        for i in range(max(0, start_line - 3), start_line):
            if i < len(self._lines):
                for marker in self.SKIP_MARKERS:
                    if marker in self._lines[i]:
                        return True

        return False

    def _extract_section_name(self, line_number: int) -> Optional[str]:
        """
        Get the section header this block belongs to.

        Searches backwards from line_number to find most recent # Header.
        """
        for i in range(line_number - 1, -1, -1):
            if i >= len(self._lines):
                continue

            line = self._lines[i]
            match = self.HEADER_PATTERN.match(line)
            if match:
                return match.group(1).strip()

        return None


# Module-level function for convenience
def extract_code_blocks(
    readme_path: Path,
    languages: Optional[List[str]] = None
) -> List[CodeBlock]:
    """
    Convenience function to extract code blocks.

    Args:
        readme_path: Path to README.md
        languages: Optional language filter

    Returns:
        List of CodeBlock objects
    """
    parser = MarkdownParser(readme_path)
    return parser.extract_code_blocks(languages=languages)


if __name__ == '__main__':
    # CLI for testing
    import sys

    if len(sys.argv) < 2:
        print("Usage: python markdown_parser.py <readme_path>")
        sys.exit(1)

    readme = Path(sys.argv[1])
    blocks = extract_code_blocks(readme)

    print(f"Found {len(blocks)} code blocks in {readme}")
    for block in blocks:
        print(f"  {block}")
