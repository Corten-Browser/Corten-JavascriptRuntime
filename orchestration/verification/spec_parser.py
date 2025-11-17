#!/usr/bin/env python3
"""
Extract required features from markdown specification documents.
Uses pattern matching to identify feature declarations.
"""
import re
from pathlib import Path
from dataclasses import dataclass, field


@dataclass
class SpecFeature:
    """A feature extracted from a specification document."""
    name: str
    section: str
    line_number: int
    required: bool = True
    keywords: list[str] = field(default_factory=list)


def extract_keywords(text: str) -> list[str]:
    """Extract searchable keywords from feature name."""
    # Remove common words, keep technical terms
    stopwords = {'the', 'a', 'an', 'for', 'with', 'to', 'of', 'and', 'or', 'in', 'on', 'at', 'by'}
    words = re.findall(r'\b\w+\b', text.lower())
    return [w for w in words if w not in stopwords and len(w) > 2]


def extract_features_from_markdown(spec_file: Path) -> list[SpecFeature]:
    """
    Parse markdown spec to extract all required features.

    Patterns detected:
    - ## Feature: X
    - ### Component: Y
    - - [ ] Implement Z
    - MUST implement W
    - Required: V
    - Phase N: A, B, C
    """
    features = []

    content = spec_file.read_text()
    lines = content.split('\n')

    current_section = "root"

    for i, line in enumerate(lines):
        line_num = i + 1

        # Track current section
        if line.startswith('## '):
            current_section = line[3:].strip()

        # Pattern 1: Explicit feature declaration
        # ## Feature: User Authentication
        match = re.match(r'^#+\s*(?:Feature|Component|Module):\s*(.+)$', line, re.I)
        if match:
            features.append(SpecFeature(
                name=match.group(1).strip(),
                section=current_section,
                line_number=line_num,
                keywords=extract_keywords(match.group(1))
            ))
            continue

        # Pattern 2: TODO items in spec
        # - [ ] Implement JWT token generation
        match = re.match(r'^[\s-]*\[\s*\]\s*(?:Implement|Create|Build|Add)\s+(.+)$', line, re.I)
        if match:
            features.append(SpecFeature(
                name=match.group(1).strip(),
                section=current_section,
                line_number=line_num,
                keywords=extract_keywords(match.group(1))
            ))
            continue

        # Pattern 3: MUST/SHALL/REQUIRED statements
        # The system MUST support async/await
        match = re.match(r'^.*(?:MUST|SHALL|REQUIRED)[:\s]+(?:implement|support|provide|include)\s+(.+?)(?:\.|$)', line, re.I)
        if match:
            features.append(SpecFeature(
                name=match.group(1).strip(),
                section=current_section,
                line_number=line_num,
                keywords=extract_keywords(match.group(1))
            ))
            continue

        # Pattern 4: Phase declarations with feature lists
        # Phase 2: ES6 Features (classes, modules, destructuring)
        match = re.match(r'^#+\s*Phase\s+\d+[:\s]+([^(]+)\s*\(([^)]+)\)', line, re.I)
        if match:
            phase_name = match.group(1).strip()
            phase_features = [f.strip() for f in match.group(2).split(',')]
            for feat in phase_features:
                features.append(SpecFeature(
                    name=f"{phase_name}: {feat}",
                    section=current_section,
                    line_number=line_num,
                    keywords=extract_keywords(feat)
                ))
            continue

        # Pattern 5: Bulleted requirements
        # - Full ES2024 compliance
        # - Complete RegExp engine
        match = re.match(r'^[\s*-]+(?:Full|Complete|Implement|Support)\s+(.+)$', line, re.I)
        if match:
            features.append(SpecFeature(
                name=match.group(1).strip(),
                section=current_section,
                line_number=line_num,
                keywords=extract_keywords(match.group(1))
            ))

    return features


if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: python spec_parser.py <spec_file.md>")
        sys.exit(1)

    spec_file = Path(sys.argv[1])
    features = extract_features_from_markdown(spec_file)

    print(f"Found {len(features)} features in {spec_file.name}:")
    for feat in features:
        print(f"  - {feat.name} (line {feat.line_number})")
        print(f"    Keywords: {feat.keywords}")
