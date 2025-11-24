#!/usr/bin/env python3
"""
Specification Completeness Verifier

Prevents premature project completion by verifying that ALL features
from a specification document have been implemented, not just that tests pass.

Key insight from failure analysis: "Tests passing ≠ Specification complete"
"""

import json
import re
import sys
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class Feature:
    """A feature extracted from specification."""
    name: str
    description: str
    source_line: int
    implemented: bool = False
    has_tests: bool = False
    is_stub: bool = False
    implementation_path: Optional[str] = None


@dataclass
class VerificationResult:
    """Result of specification completeness verification."""
    spec_path: str
    total_features: int
    implemented_features: int
    stubbed_features: int
    tested_features: int
    coverage_percentage: float
    missing_features: list = field(default_factory=list)
    stub_features: list = field(default_factory=list)
    untested_features: list = field(default_factory=list)
    is_complete: bool = False
    blocking_issues: list = field(default_factory=list)


class SpecCompletenessVerifier:
    """
    Verifies that a project implements 100% of features from its specification.

    This tool addresses the core failure pattern where Claude stops at 60-70%
    spec coverage while claiming project completion.
    """

    # Patterns that indicate features in markdown specs
    FEATURE_PATTERNS = [
        r'^#+\s+(?:Feature|Requirement|Component):\s*(.+)$',
        r'^\s*[-*]\s+\*\*([^*]+)\*\*\s*[-–:]\s*(.+)$',  # **Feature** - description
        r'^\s*[-*]\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)\s*[-–:]\s*(.+)$',  # Feature - description
        r'^\s*\d+\.\s+\*\*([^*]+)\*\*',  # 1. **Feature**
        r'^#+\s+(Phase\s+\d+.*)$',  # Phase headers
        r'^\s*[-*]\s+`([^`]+)`\s*[-–:]\s*(.+)$',  # `function` - description
        r'^\s*[-*]\s+([A-Z][A-Z_]+)\s*[-–:]\s*(.+)$',  # CONSTANT - description
    ]

    # Stub/placeholder indicators
    STUB_INDICATORS = [
        'TODO',
        'FIXME',
        'NotImplemented',
        'placeholder',
        'not implemented',
        'stub',
        'skeleton',
        'pending',
        'pass  #',
        'raise NotImplementedError',
        '...',  # Python ellipsis placeholder
    ]

    def __init__(self, project_root: str):
        """Initialize verifier with project root path."""
        self.project_root = Path(project_root)
        self.features: list[Feature] = []
        self.spec_content = ""

    def load_specification(self, spec_path: str) -> bool:
        """
        Load and parse specification document.

        Returns True if spec loaded successfully.
        """
        spec_file = Path(spec_path)
        if not spec_file.exists():
            # Try relative to project root
            spec_file = self.project_root / spec_path

        if not spec_file.exists():
            print(f"❌ Specification file not found: {spec_path}")
            return False

        self.spec_content = spec_file.read_text()
        self._extract_features()
        return True

    def _extract_features(self):
        """Extract all features from specification document."""
        self.features = []
        lines = self.spec_content.split('\n')

        for i, line in enumerate(lines, 1):
            for pattern in self.FEATURE_PATTERNS:
                match = re.match(pattern, line, re.IGNORECASE)
                if match:
                    groups = match.groups()
                    name = groups[0].strip()
                    description = groups[1].strip() if len(groups) > 1 else ""

                    # Skip generic headers
                    if name.lower() in ['overview', 'introduction', 'summary', 'notes']:
                        continue

                    feature = Feature(
                        name=name,
                        description=description,
                        source_line=i
                    )
                    self.features.append(feature)
                    break

        # Also extract inline requirements (SHALL, MUST, WILL)
        shall_pattern = r'(?:SHALL|MUST|WILL)\s+(?:implement|provide|support|include|have)\s+([^.]+)'
        for i, line in enumerate(lines, 1):
            matches = re.findall(shall_pattern, line, re.IGNORECASE)
            for match in matches:
                feature = Feature(
                    name=match.strip()[:100],  # Truncate long matches
                    description=f"Requirement from line {i}",
                    source_line=i
                )
                # Avoid duplicates
                if not any(f.name == feature.name for f in self.features):
                    self.features.append(feature)

    def verify_implementation(self, components_dir: str = "components") -> VerificationResult:
        """
        Verify each feature is implemented (not stubbed) and tested.

        Returns detailed verification result.
        """
        if not self.features:
            return VerificationResult(
                spec_path="",
                total_features=0,
                implemented_features=0,
                stubbed_features=0,
                tested_features=0,
                coverage_percentage=0.0,
                blocking_issues=["No features extracted from specification"]
            )

        components_path = self.project_root / components_dir
        if not components_path.exists():
            components_path = self.project_root / "src"  # Alternative structure

        # Check each feature
        for feature in self.features:
            self._check_feature_implementation(feature, components_path)

        # Calculate results
        implemented = sum(1 for f in self.features if f.implemented and not f.is_stub)
        stubbed = sum(1 for f in self.features if f.is_stub)
        tested = sum(1 for f in self.features if f.has_tests)
        total = len(self.features)

        missing = [f for f in self.features if not f.implemented]
        stubs = [f for f in self.features if f.is_stub]
        untested = [f for f in self.features if f.implemented and not f.has_tests]

        coverage = (implemented / total * 100) if total > 0 else 0.0

        # Determine blocking issues
        blocking = []
        if coverage < 100:
            blocking.append(f"Specification coverage is {coverage:.1f}%, must be 100%")
        if missing:
            blocking.append(f"{len(missing)} features not implemented")
        if stubs:
            blocking.append(f"{len(stubs)} features are stubs/placeholders")
        if untested:
            blocking.append(f"{len(untested)} implemented features lack tests")

        is_complete = coverage >= 100 and not stubs and not missing

        return VerificationResult(
            spec_path=str(self.project_root),
            total_features=total,
            implemented_features=implemented,
            stubbed_features=stubbed,
            tested_features=tested,
            coverage_percentage=coverage,
            missing_features=[f.name for f in missing],
            stub_features=[f.name for f in stubs],
            untested_features=[f.name for f in untested],
            is_complete=is_complete,
            blocking_issues=blocking
        )

    def _check_feature_implementation(self, feature: Feature, components_path: Path):
        """Check if a specific feature is implemented."""
        # Search for feature name in source files
        search_terms = self._generate_search_terms(feature.name)

        # Search in Python files
        for py_file in components_path.rglob("*.py"):
            if "__pycache__" in str(py_file):
                continue
            try:
                content = py_file.read_text()
                for term in search_terms:
                    if term.lower() in content.lower():
                        feature.implemented = True
                        feature.implementation_path = str(py_file)

                        # Check for stubs
                        feature.is_stub = self._check_for_stubs(content, term)
                        break
            except Exception:
                continue

            if feature.implemented:
                break

        # Search in Rust files
        if not feature.implemented:
            for rs_file in components_path.rglob("*.rs"):
                try:
                    content = rs_file.read_text()
                    for term in search_terms:
                        if term.lower() in content.lower():
                            feature.implemented = True
                            feature.implementation_path = str(rs_file)
                            feature.is_stub = self._check_for_stubs(content, term)
                            break
                except Exception:
                    continue
                if feature.implemented:
                    break

        # Check for tests
        if feature.implemented:
            feature.has_tests = self._check_for_tests(feature, components_path)

    def _generate_search_terms(self, feature_name: str) -> list[str]:
        """Generate search terms from feature name."""
        terms = [feature_name]

        # Convert to snake_case
        snake = re.sub(r'(?<!^)(?=[A-Z])', '_', feature_name).lower()
        snake = re.sub(r'\s+', '_', snake)
        terms.append(snake)

        # Convert to camelCase
        parts = feature_name.replace('_', ' ').split()
        if len(parts) > 1:
            camel = parts[0].lower() + ''.join(p.capitalize() for p in parts[1:])
            terms.append(camel)

        # Convert to PascalCase
        pascal = ''.join(p.capitalize() for p in feature_name.replace('_', ' ').split())
        terms.append(pascal)

        return list(set(terms))

    def _check_for_stubs(self, content: str, term: str) -> bool:
        """Check if implementation is a stub/placeholder."""
        # Find the context around the term
        lines = content.split('\n')
        for i, line in enumerate(lines):
            if term.lower() in line.lower():
                # Check surrounding lines for stub indicators
                context = '\n'.join(lines[max(0, i-2):min(len(lines), i+10)])
                for indicator in self.STUB_INDICATORS:
                    if indicator in context:
                        return True
        return False

    def _check_for_tests(self, feature: Feature, components_path: Path) -> bool:
        """Check if feature has tests."""
        search_terms = self._generate_search_terms(feature.name)

        # Search in test files
        test_dirs = [
            components_path / "tests",
            self.project_root / "tests",
            components_path.parent / "tests",
        ]

        for test_dir in test_dirs:
            if not test_dir.exists():
                continue
            for test_file in test_dir.rglob("test_*.py"):
                try:
                    content = test_file.read_text()
                    for term in search_terms:
                        if term.lower() in content.lower():
                            return True
                except Exception:
                    continue

        return False

    def generate_report(self, result: VerificationResult) -> str:
        """Generate human-readable verification report."""
        lines = [
            "=" * 70,
            "SPECIFICATION COMPLETENESS VERIFICATION REPORT",
            "=" * 70,
            "",
            f"Project: {result.spec_path}",
            f"Total Features in Spec: {result.total_features}",
            f"Implemented (real code): {result.implemented_features}",
            f"Stubbed/Placeholder: {result.stubbed_features}",
            f"With Tests: {result.tested_features}",
            "",
            f"COVERAGE: {result.coverage_percentage:.1f}%",
            "",
        ]

        if result.is_complete:
            lines.append("✅ SPECIFICATION 100% COMPLETE")
        else:
            lines.append("❌ SPECIFICATION INCOMPLETE - CANNOT STOP")
            lines.append("")
            lines.append("BLOCKING ISSUES:")
            for issue in result.blocking_issues:
                lines.append(f"  ❌ {issue}")

        if result.missing_features:
            lines.append("")
            lines.append(f"MISSING FEATURES ({len(result.missing_features)}):")
            for name in result.missing_features[:20]:  # Limit display
                lines.append(f"  - {name}")
            if len(result.missing_features) > 20:
                lines.append(f"  ... and {len(result.missing_features) - 20} more")

        if result.stub_features:
            lines.append("")
            lines.append(f"STUB/PLACEHOLDER FEATURES ({len(result.stub_features)}):")
            for name in result.stub_features[:10]:
                lines.append(f"  - {name}")
            if len(result.stub_features) > 10:
                lines.append(f"  ... and {len(result.stub_features) - 10} more")

        lines.append("")
        lines.append("=" * 70)

        if not result.is_complete:
            lines.append("ACTION REQUIRED: Continue implementing until 100% coverage")
            lines.append("DO NOT generate completion report until all issues resolved")

        return '\n'.join(lines)

    def save_checklist(self, output_path: str = "spec_checklist.json"):
        """Save feature checklist as JSON for tracking."""
        checklist = {
            "features": [
                {
                    "name": f.name,
                    "description": f.description,
                    "source_line": f.source_line,
                    "implemented": f.implemented,
                    "is_stub": f.is_stub,
                    "has_tests": f.has_tests,
                    "implementation_path": f.implementation_path
                }
                for f in self.features
            ],
            "total": len(self.features),
            "implemented": sum(1 for f in self.features if f.implemented and not f.is_stub)
        }

        output_file = self.project_root / output_path
        output_file.write_text(json.dumps(checklist, indent=2))
        print(f"✓ Checklist saved to {output_file}")


def main():
    """CLI entry point."""
    if len(sys.argv) < 3:
        print("Usage: python spec_completeness_verifier.py <project_root> <spec_file>")
        print("Example: python spec_completeness_verifier.py . specifications/project-spec.md")
        sys.exit(1)

    project_root = sys.argv[1]
    spec_file = sys.argv[2]

    verifier = SpecCompletenessVerifier(project_root)

    if not verifier.load_specification(spec_file):
        sys.exit(1)

    print(f"Extracted {len(verifier.features)} features from specification")

    result = verifier.verify_implementation()
    report = verifier.generate_report(result)
    print(report)

    # Save checklist for reference
    verifier.save_checklist()

    # Exit with error if incomplete (for CI/gate integration)
    if not result.is_complete:
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()
