#!/usr/bin/env python3
"""
Scan implementation to detect which features are actually implemented.
Uses code analysis, not just test pass rates.
"""
import re
from pathlib import Path
from dataclasses import dataclass, field


@dataclass
class ImplementedFeature:
    """Evidence that a feature is implemented."""
    name: str
    evidence: list[str] = field(default_factory=list)  # File paths where found
    confidence: float = 0.0  # 0.0 to 1.0


def is_stub_file(file_path: Path) -> bool:
    """Detect if file is a stub/placeholder."""
    try:
        content = file_path.read_text()
    except Exception:
        return False

    stub_indicators = [
        'implementation pending',
        'TODO: implement',
        'raise NotImplementedError',
        'pass  # stub',
        'unimplemented!()',
        'todo!()',
    ]

    indicator_count = sum(1 for ind in stub_indicators if ind.lower() in content.lower())

    # If more than 30% of functions are stubs, it's a stub file
    function_count = len(re.findall(r'^\s*(?:def |fn |function )', content, re.M))
    stub_count = len(re.findall(r'(?:NotImplementedError|pass\s*#|unimplemented!|todo!)', content))

    if function_count > 0 and stub_count / function_count > 0.3:
        return True

    return indicator_count >= 2


def has_actual_implementation(content: str, keywords: list[str]) -> bool:
    """Check if content has actual logic, not just definitions."""
    # Look for control flow, assignments, operations
    logic_patterns = [
        r'if\s+.+:',
        r'for\s+.+:',
        r'while\s+.+:',
        r'return\s+.+',
        r'\w+\s*=\s*.+',
        r'\w+\.\w+\(',
    ]

    logic_count = sum(
        len(re.findall(pattern, content))
        for pattern in logic_patterns
    )

    # Also check keywords appear near logic
    for kw in keywords:
        if re.search(rf'\b{kw}\b.*(?:if|for|while|return|=)', content, re.I | re.S):
            return True

    return logic_count > 10  # Has substantial logic


def scan_for_feature(feature_keywords: list[str], search_dir: Path) -> ImplementedFeature:
    """
    Search codebase for evidence that a feature is implemented.

    Evidence types:
    1. Function/class definitions matching keywords
    2. Test files specifically for the feature
    3. Documentation mentioning implementation
    4. Actual code logic (not stubs)
    """
    evidence = []

    if not feature_keywords:
        return ImplementedFeature(name="", evidence=[], confidence=0.0)

    # Search Python source files
    for src_file in search_dir.rglob("*.py"):
        if "__pycache__" in str(src_file) or ".venv" in str(src_file):
            continue

        if is_stub_file(src_file):
            continue

        try:
            content = src_file.read_text()
        except Exception:
            continue

        # Check for keyword presence in actual code
        keyword_matches = sum(
            1 for kw in feature_keywords
            if re.search(rf'\b{kw}\b', content, re.I)
        )

        if keyword_matches >= len(feature_keywords) * 0.5:
            # Check it's not just imports or comments
            if has_actual_implementation(content, feature_keywords):
                evidence.append(str(src_file))

    # Search for Rust files too (for multi-language projects)
    for src_file in search_dir.rglob("*.rs"):
        if "target" in str(src_file):
            continue

        if is_stub_file(src_file):
            continue

        try:
            content = src_file.read_text()
        except Exception:
            continue

        keyword_matches = sum(
            1 for kw in feature_keywords
            if re.search(rf'\b{kw}\b', content, re.I)
        )

        if keyword_matches >= len(feature_keywords) * 0.5:
            if has_actual_implementation(content, feature_keywords):
                evidence.append(str(src_file))

    # Calculate confidence based on evidence
    confidence = min(1.0, len(evidence) / 3.0)  # 3+ files = high confidence

    return ImplementedFeature(
        name=", ".join(feature_keywords),
        evidence=evidence,
        confidence=confidence
    )


if __name__ == "__main__":
    import sys
    if len(sys.argv) < 3:
        print("Usage: python implementation_scanner.py <search_dir> <keyword1> [keyword2] ...")
        sys.exit(1)

    search_dir = Path(sys.argv[1])
    keywords = sys.argv[2:]

    result = scan_for_feature(keywords, search_dir)
    print(f"Feature: {result.name}")
    print(f"Confidence: {result.confidence:.2f}")
    print(f"Evidence ({len(result.evidence)} files):")
    for f in result.evidence[:10]:
        print(f"  - {f}")
