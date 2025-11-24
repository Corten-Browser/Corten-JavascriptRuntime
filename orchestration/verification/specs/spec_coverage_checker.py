#!/usr/bin/env python3
"""
Main specification coverage checker.
Calculates objective coverage percentage.
"""
import json
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from orchestration.core.paths import DataPaths
from .spec_parser import extract_features_from_markdown
from ..quality.implementation_scanner import scan_for_feature

# Global paths instance
_paths = DataPaths()


def check_specification_coverage(
    spec_file: Path,
    implementation_dir: Path,
    output_file: Path = None
) -> float:
    """
    Calculate specification coverage percentage.

    Returns value between 0.0 and 1.0.
    Blocks if < 1.0 (100%).
    """
    print("=" * 60)
    print("SPECIFICATION COVERAGE ANALYSIS")
    print("=" * 60)
    print(f"Spec file: {spec_file}")
    print(f"Implementation: {implementation_dir}")
    print("")

    # Extract all required features from spec
    required_features = extract_features_from_markdown(spec_file)
    print(f"Found {len(required_features)} required features in specification")
    print("")

    if not required_features:
        print("WARNING: No features found in specification")
        print("Check that specification uses recognized patterns:")
        print("  - ## Feature: X")
        print("  - - [ ] Implement Y")
        print("  - MUST implement Z")
        print("=" * 60)
        return 0.0

    # Check implementation for each feature
    implemented = []
    missing = []

    for feature in required_features:
        result = scan_for_feature(feature.keywords, implementation_dir)

        if result.confidence >= 0.5 and len(result.evidence) > 0:
            implemented.append({
                "feature": feature.name,
                "evidence": result.evidence[:3],  # Top 3 files
                "confidence": result.confidence
            })
            evidence_str = ', '.join([Path(e).name for e in result.evidence[:2]])
            print(f"  {feature.name}")
            print(f"   Evidence: {evidence_str}")
        else:
            missing.append({
                "feature": feature.name,
                "section": feature.section,
                "spec_line": feature.line_number
            })
            print(f"  {feature.name}")
            print(f"   NOT FOUND in implementation")

    print("")
    print("=" * 60)

    coverage = len(implemented) / len(required_features) if required_features else 0.0

    print(f"COVERAGE: {coverage * 100:.1f}%")
    print(f"Implemented: {len(implemented)}/{len(required_features)}")
    print("")

    if missing:
        print("MISSING FEATURES (MUST IMPLEMENT):")
        for feat in missing:
            print(f"  - {feat['feature']}")
            print(f"    Spec: {spec_file.name}:{feat['spec_line']}")
        print("")

    # Save report
    if output_file:
        report = {
            "timestamp": datetime.now().isoformat(),
            "spec_file": str(spec_file),
            "implementation_dir": str(implementation_dir),
            "coverage_percentage": coverage * 100,
            "total_features": len(required_features),
            "implemented_count": len(implemented),
            "missing_count": len(missing),
            "implemented_features": implemented,
            "missing_features": missing,
            "is_complete": coverage >= 1.0
        }

        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(report, indent=2))
        print(f"Report saved: {output_file}")

    print("=" * 60)

    return coverage


def main():
    if len(sys.argv) < 3:
        print("Usage: python spec_coverage_checker.py <spec_file> <implementation_dir>")
        sys.exit(1)

    spec_file = Path(sys.argv[1])
    impl_dir = Path(sys.argv[2])
    output = _paths.coverage_report
    output.parent.mkdir(parents=True, exist_ok=True)

    if not spec_file.exists():
        print(f"ERROR: Specification file not found: {spec_file}")
        sys.exit(1)

    if not impl_dir.exists():
        print(f"ERROR: Implementation directory not found: {impl_dir}")
        sys.exit(1)

    coverage = check_specification_coverage(spec_file, impl_dir, output)

    if coverage < 1.0:
        print("")
        print("INCOMPLETE: Specification not fully implemented")
        print(f"   Missing {(1 - coverage) * 100:.1f}% of required features")
        sys.exit(1)
    else:
        print("")
        print("COMPLETE: All specification features implemented")
        sys.exit(0)


if __name__ == "__main__":
    main()
