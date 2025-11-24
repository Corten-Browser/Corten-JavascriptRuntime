#!/usr/bin/env python3
"""
Specification Reconciliation System

Reconciles existing specification documents with actual code implementation.
Handles conflicts between documented and implemented features.

Version: 1.0.0
"""

import sys
import json
import yaml
from pathlib import Path
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass, field
from enum import Enum


class FeatureSource(Enum):
    """Where feature was discovered"""
    SPEC_ONLY = "spec"          # In specification, not in code
    CODE_ONLY = "code"          # In code, not in specification
    BOTH = "both"               # In both (verified)
    CONFLICT = "conflict"       # In both but conflicting


class ConflictType(Enum):
    """Types of conflicts between spec and code"""
    SIGNATURE = "signature"     # Different method signatures
    BEHAVIOR = "behavior"       # Different expected behavior
    NAME = "name"              # Different names for same feature
    TYPE = "type"              # Different types/interfaces


@dataclass
class Feature:
    """Represents a feature (from spec or code)"""
    name: str
    description: str
    source: FeatureSource
    category: str  # "api", "cli", "internal", etc.
    details: Dict = field(default_factory=dict)


@dataclass
class Conflict:
    """Represents a conflict between spec and code"""
    feature_name: str
    spec_version: str
    code_version: str
    conflict_type: ConflictType
    details: str
    resolution: Optional[str] = None  # "use_spec", "use_code", "manual"


@dataclass
class ReconciliationReport:
    """Report of specification vs code reconciliation"""
    verified_features: List[Feature] = field(default_factory=list)      # In spec + in code
    planned_features: List[Feature] = field(default_factory=list)       # In spec, not in code
    undocumented_features: List[Feature] = field(default_factory=list)  # In code, not in spec
    conflicts: List[Conflict] = field(default_factory=list)             # Spec vs code disagree
    statistics: Dict[str, int] = field(default_factory=dict)
    recommendations: List[str] = field(default_factory=list)

    def calculate_statistics(self):
        """Calculate summary statistics"""
        self.statistics = {
            "verified": len(self.verified_features),
            "planned": len(self.planned_features),
            "undocumented": len(self.undocumented_features),
            "conflicts": len(self.conflicts),
            "total_spec_features": len(self.verified_features) + len(self.planned_features) + len(self.conflicts),
            "total_code_features": len(self.verified_features) + len(self.undocumented_features) + len(self.conflicts),
        }

        # Calculate coverage percentage
        if self.statistics["total_spec_features"] > 0:
            self.statistics["spec_coverage"] = (
                (self.statistics["verified"] / self.statistics["total_spec_features"]) * 100
            )
        else:
            self.statistics["spec_coverage"] = 0.0


class SpecReconciler:
    """
    Reconciles specification documents with code implementation.

    Workflow:
    1. Detect existing specification documents
    2. Parse specifications (YAML/Markdown)
    3. Extract documented features
    4. Compare with code-derived features
    5. Categorize features and identify conflicts
    6. Generate reconciliation report
    7. Allow user to resolve conflicts
    8. Generate unified specification
    """

    def __init__(self, project_dir: Path):
        """
        Initialize spec reconciler.

        Args:
            project_dir: Root directory of target project
        """
        self.project_dir = Path(project_dir).resolve()
        self.spec_dir = self.project_dir / "specifications"

    def detect_existing_specs(self) -> List[Path]:
        """
        Detect existing specification documents.

        Returns:
            List of specification file paths
        """
        spec_files = []

        # Check specifications/ directory
        if self.spec_dir.exists():
            # YAML specs
            spec_files.extend(self.spec_dir.glob("*.yaml"))
            spec_files.extend(self.spec_dir.glob("*.yml"))
            # Markdown specs
            spec_files.extend(self.spec_dir.glob("*.md"))

        # Check for common spec file names in root
        common_names = [
            "SPECIFICATION.md",
            "SPEC.md",
            "specification.yaml",
            "spec.yaml",
            "requirements.md",
            "REQUIREMENTS.md"
        ]

        for name in common_names:
            spec_file = self.project_dir / name
            if spec_file.exists() and spec_file not in spec_files:
                spec_files.append(spec_file)

        return spec_files

    def parse_existing_spec(self, spec_path: Path) -> Dict:
        """
        Parse specification file.

        Args:
            spec_path: Path to specification file

        Returns:
            Parsed specification data
        """
        try:
            if spec_path.suffix in ['.yaml', '.yml']:
                return self._parse_yaml_spec(spec_path)
            elif spec_path.suffix == '.md':
                return self._parse_markdown_spec(spec_path)
            else:
                raise ValueError(f"Unsupported spec format: {spec_path.suffix}")
        except Exception as e:
            print(f"Warning: Failed to parse {spec_path}: {e}", file=sys.stderr)
            return {}

    def _parse_yaml_spec(self, spec_path: Path) -> Dict:
        """Parse YAML specification"""
        with open(spec_path, 'r') as f:
            return yaml.safe_load(f) or {}

    def _parse_markdown_spec(self, spec_path: Path) -> Dict:
        """
        Parse Markdown specification.

        Extracts features from markdown structure:
        - Headings become feature categories
        - List items become features
        - Code blocks become implementation details
        """
        with open(spec_path, 'r') as f:
            content = f.read()

        spec_data = {
            "features": [],
            "source": str(spec_path.relative_to(self.project_dir))
        }

        lines = content.split('\n')
        current_category = "general"
        current_description = []

        for line in lines:
            # Headings define categories
            if line.startswith('# '):
                current_category = line[2:].strip().lower()
            elif line.startswith('## '):
                current_category = line[3:].strip().lower()

            # List items are features
            elif line.strip().startswith('- ') or line.strip().startswith('* '):
                feature_text = line.strip()[2:].strip()
                if feature_text:
                    spec_data["features"].append({
                        "name": feature_text.split(':')[0].strip() if ':' in feature_text else feature_text[:50],
                        "description": feature_text,
                        "category": current_category
                    })

        return spec_data

    def extract_documented_features(self, spec: Dict) -> List[Feature]:
        """
        Extract features from specification document.

        Args:
            spec: Parsed specification data

        Returns:
            List of documented features
        """
        features = []

        # Handle YAML spec format
        if "features" in spec:
            for feature_data in spec["features"]:
                if isinstance(feature_data, dict):
                    feature = Feature(
                        name=feature_data.get("name", "Unknown"),
                        description=feature_data.get("description", ""),
                        source=FeatureSource.SPEC_ONLY,  # Will be updated during reconciliation
                        category=feature_data.get("category", "general"),
                        details=feature_data
                    )
                    features.append(feature)
                elif isinstance(feature_data, str):
                    # Simple string feature
                    feature = Feature(
                        name=feature_data,
                        description=feature_data,
                        source=FeatureSource.SPEC_ONLY,
                        category="general",
                        details={}
                    )
                    features.append(feature)

        # Handle component-specific specs
        if "components" in spec:
            for component_name, component_spec in spec["components"].items():
                if "features" in component_spec:
                    for feature_name in component_spec["features"]:
                        feature = Feature(
                            name=feature_name,
                            description=component_spec.get("description", ""),
                            source=FeatureSource.SPEC_ONLY,
                            category=component_name,
                            details={"component": component_name}
                        )
                        features.append(feature)

        return features

    def discover_code_features(self, code_analysis: Dict) -> List[Feature]:
        """
        Extract features from code analysis data.

        Args:
            code_analysis: Code analysis data (from component analysis)

        Returns:
            List of code-derived features
        """
        features = []

        # Extract from component analysis
        if "components" in code_analysis:
            for component_name, component_data in code_analysis["components"].items():
                # API endpoints
                if "apis" in component_data:
                    for api in component_data["apis"]:
                        feature = Feature(
                            name=api.get("endpoint", api.get("name", "Unknown")),
                            description=api.get("description", "API endpoint"),
                            source=FeatureSource.CODE_ONLY,
                            category="api",
                            details={"component": component_name, "api": api}
                        )
                        features.append(feature)

                # CLI commands
                if "cli_commands" in component_data:
                    for cmd in component_data["cli_commands"]:
                        feature = Feature(
                            name=cmd.get("command", cmd.get("name", "Unknown")),
                            description=cmd.get("description", "CLI command"),
                            source=FeatureSource.CODE_ONLY,
                            category="cli",
                            details={"component": component_name, "command": cmd}
                        )
                        features.append(feature)

                # Public methods/functions
                if "public_methods" in component_data:
                    for method in component_data["public_methods"]:
                        feature = Feature(
                            name=method.get("name", "Unknown"),
                            description=method.get("description", "Public method"),
                            source=FeatureSource.CODE_ONLY,
                            category="api",
                            details={"component": component_name, "method": method}
                        )
                        features.append(feature)

        return features

    def reconcile(self, documented: List[Feature], implemented: List[Feature]) -> ReconciliationReport:
        """
        Reconcile documented features with implemented features.

        Args:
            documented: Features from specification
            implemented: Features from code

        Returns:
            Reconciliation report
        """
        report = ReconciliationReport()

        # Create lookup dictionaries
        doc_by_name = {f.name.lower(): f for f in documented}
        impl_by_name = {f.name.lower(): f for f in implemented}

        # Find verified features (in both)
        for name, doc_feature in doc_by_name.items():
            if name in impl_by_name:
                impl_feature = impl_by_name[name]

                # Check for conflicts
                conflict = self._check_for_conflict(doc_feature, impl_feature)

                if conflict:
                    report.conflicts.append(conflict)
                    # Mark as conflict
                    doc_feature.source = FeatureSource.CONFLICT
                    report.verified_features.append(doc_feature)
                else:
                    # Verified match
                    doc_feature.source = FeatureSource.BOTH
                    report.verified_features.append(doc_feature)

        # Find planned features (spec only)
        for name, doc_feature in doc_by_name.items():
            if name not in impl_by_name:
                doc_feature.source = FeatureSource.SPEC_ONLY
                report.planned_features.append(doc_feature)

        # Find undocumented features (code only)
        for name, impl_feature in impl_by_name.items():
            if name not in doc_by_name:
                impl_feature.source = FeatureSource.CODE_ONLY
                report.undocumented_features.append(impl_feature)

        # Generate recommendations
        report.recommendations = self._generate_recommendations(report)

        # Calculate statistics
        report.calculate_statistics()

        return report

    def _check_for_conflict(self, doc_feature: Feature, impl_feature: Feature) -> Optional[Conflict]:
        """
        Check if two features conflict.

        Args:
            doc_feature: Feature from documentation
            impl_feature: Feature from implementation

        Returns:
            Conflict object if conflict exists, None otherwise
        """
        # Compare descriptions
        if doc_feature.description and impl_feature.description:
            if doc_feature.description.lower() != impl_feature.description.lower():
                # Check if it's a significant difference (not just whitespace/formatting)
                doc_words = set(doc_feature.description.lower().split())
                impl_words = set(impl_feature.description.lower().split())

                # If less than 50% overlap, consider it a behavior conflict
                if len(doc_words & impl_words) / max(len(doc_words), len(impl_words)) < 0.5:
                    return Conflict(
                        feature_name=doc_feature.name,
                        spec_version=doc_feature.description,
                        code_version=impl_feature.description,
                        conflict_type=ConflictType.BEHAVIOR,
                        details="Descriptions differ significantly"
                    )

        # Compare signatures (if available)
        if "signature" in doc_feature.details and "signature" in impl_feature.details:
            if doc_feature.details["signature"] != impl_feature.details["signature"]:
                return Conflict(
                    feature_name=doc_feature.name,
                    spec_version=str(doc_feature.details["signature"]),
                    code_version=str(impl_feature.details["signature"]),
                    conflict_type=ConflictType.SIGNATURE,
                    details="Method signatures differ"
                )

        return None

    def _generate_recommendations(self, report: ReconciliationReport) -> List[str]:
        """Generate recommendations based on reconciliation results"""
        recommendations = []

        if report.planned_features:
            recommendations.append(
                f"Implement {len(report.planned_features)} planned features from specification"
            )

        if report.undocumented_features:
            recommendations.append(
                f"Document {len(report.undocumented_features)} implemented but undocumented features"
            )

        if report.conflicts:
            recommendations.append(
                f"Resolve {len(report.conflicts)} conflicts between spec and implementation"
            )

        if not report.planned_features and not report.undocumented_features and not report.conflicts:
            recommendations.append("Specification and implementation are in sync")

        return recommendations

    def generate_unified_spec(self, reconciliation: ReconciliationReport, user_choices: Dict) -> Dict:
        """
        Generate unified specification from reconciliation.

        Args:
            reconciliation: Reconciliation report
            user_choices: User decisions for conflict resolution
                         Format: {feature_name: "use_spec" | "use_code" | "manual_value"}

        Returns:
            Unified specification dictionary
        """
        unified_spec = {
            "version": "1.0.0",
            "generated_by": "spec_reconciler",
            "features": [],
            "metadata": {
                "verified_features": len(reconciliation.verified_features),
                "planned_features": len(reconciliation.planned_features),
                "undocumented_features": len(reconciliation.undocumented_features),
                "conflicts_resolved": len(reconciliation.conflicts)
            }
        }

        # Add verified features
        for feature in reconciliation.verified_features:
            unified_spec["features"].append({
                "name": feature.name,
                "description": feature.description,
                "category": feature.category,
                "status": "verified",
                "source": "both"
            })

        # Add planned features
        for feature in reconciliation.planned_features:
            unified_spec["features"].append({
                "name": feature.name,
                "description": feature.description,
                "category": feature.category,
                "status": "planned",
                "source": "spec"
            })

        # Add undocumented features
        for feature in reconciliation.undocumented_features:
            unified_spec["features"].append({
                "name": feature.name,
                "description": feature.description,
                "category": feature.category,
                "status": "implemented",
                "source": "code"
            })

        # Resolve conflicts based on user choices
        for conflict in reconciliation.conflicts:
            choice = user_choices.get(conflict.feature_name, "use_spec")

            # Find the feature
            feature = next(
                (f for f in reconciliation.verified_features if f.name == conflict.feature_name),
                None
            )

            if feature:
                if choice == "use_spec":
                    description = conflict.spec_version
                elif choice == "use_code":
                    description = conflict.code_version
                else:
                    # Manual value provided
                    description = choice

                unified_spec["features"].append({
                    "name": feature.name,
                    "description": description,
                    "category": feature.category,
                    "status": "resolved_conflict",
                    "source": "both",
                    "resolution": choice
                })

        return unified_spec

    def print_report(self, report: ReconciliationReport):
        """Print formatted reconciliation report"""
        print()
        print("=" * 70)
        print("  SPECIFICATION RECONCILIATION REPORT")
        print("=" * 70)
        print()

        # Statistics
        print("SUMMARY")
        print("-" * 70)
        print(f"✅ Verified Features:      {report.statistics['verified']}")
        print(f"📋 Planned Features:       {report.statistics['planned']}")
        print(f"📝 Undocumented Features:  {report.statistics['undocumented']}")
        print(f"⚠️  Conflicts:              {report.statistics['conflicts']}")
        print()
        print(f"Spec Coverage: {report.statistics['spec_coverage']:.1f}%")
        print()

        # Verified features
        if report.verified_features:
            print("✅ VERIFIED FEATURES (in spec + in code)")
            print("-" * 70)
            for feature in report.verified_features[:10]:  # Show first 10
                print(f"  {feature.name}")
                print(f"    {feature.description[:60]}")
            if len(report.verified_features) > 10:
                print(f"  ... and {len(report.verified_features) - 10} more")
            print()

        # Conflicts
        if report.conflicts:
            print("⚠️  CONFLICTS (spec vs code disagree)")
            print("-" * 70)
            for conflict in report.conflicts:
                print(f"  {conflict.feature_name}")
                print(f"    Spec: {conflict.spec_version[:50]}")
                print(f"    Code: {conflict.code_version[:50]}")
                print(f"    Type: {conflict.conflict_type.value}")
            print()

        # Planned features
        if report.planned_features:
            print("📋 PLANNED FEATURES (in spec, not in code)")
            print("-" * 70)
            for feature in report.planned_features[:10]:
                print(f"  {feature.name}")
            if len(report.planned_features) > 10:
                print(f"  ... and {len(report.planned_features) - 10} more")
            print()

        # Undocumented features
        if report.undocumented_features:
            print("📝 UNDOCUMENTED FEATURES (in code, not in spec)")
            print("-" * 70)
            for feature in report.undocumented_features[:10]:
                print(f"  {feature.name}")
            if len(report.undocumented_features) > 10:
                print(f"  ... and {len(report.undocumented_features) - 10} more")
            print()

        # Recommendations
        print("RECOMMENDATIONS")
        print("-" * 70)
        for rec in report.recommendations:
            print(f"  • {rec}")
        print()
        print("=" * 70)


def main():
    """CLI entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Specification Reconciliation")
    parser.add_argument("project_dir", nargs="?", default=".",
                       help="Project directory (default: current)")
    parser.add_argument("--list-specs", action="store_true",
                       help="List detected specification files")
    parser.add_argument("--code-analysis", type=Path,
                       help="Path to code analysis JSON file")

    args = parser.parse_args()

    reconciler = SpecReconciler(Path(args.project_dir))

    if args.list_specs:
        specs = reconciler.detect_existing_specs()
        if specs:
            print("Detected Specification Files:")
            for spec in specs:
                print(f"  - {spec.relative_to(reconciler.project_dir)}")
        else:
            print("No specification files detected")
        return 0

    # Full reconciliation (requires code analysis)
    if args.code_analysis:
        # Load code analysis
        with open(args.code_analysis, 'r') as f:
            code_analysis = json.load(f)

        # Detect and parse specs
        specs = reconciler.detect_existing_specs()
        if not specs:
            print("No specification files found")
            return 1

        # Parse first spec (TODO: handle multiple)
        spec_data = reconciler.parse_existing_spec(specs[0])
        documented = reconciler.extract_documented_features(spec_data)
        implemented = reconciler.discover_code_features(code_analysis)

        # Reconcile
        report = reconciler.reconcile(documented, implemented)
        reconciler.print_report(report)

        return 0

    print("Use --list-specs to detect specifications or --code-analysis to run reconciliation")
    return 1


if __name__ == "__main__":
    sys.exit(main())
