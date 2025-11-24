#!/usr/bin/env python3
"""
Orchestration Compliance Checker

Checks project compliance against target orchestration version.
Generates gap analysis and upgrade plans.

Version: 1.0.0
"""

import sys
import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Callable
from dataclasses import dataclass, field
from enum import Enum


# Version Feature Map
# Defines features required for each version
VERSION_FEATURES = {
    '1.0.0': {
        'component_isolation': {
            'check': lambda pd: (pd / 'components').exists(),
            'description': 'Component isolation with components/ directory',
            'install_cmd': 'mkdir -p components'
        },
        'basic_gates': {
            'check': lambda pd: (pd / 'orchestration/gates').exists(),
            'description': 'Phase gate enforcement system',
            'install_cmd': 'Copy gates from template'
        },
        'contracts': {
            'check': lambda pd: (pd / 'contracts').exists(),
            'description': 'Contract-based component communication',
            'install_cmd': 'mkdir -p contracts'
        },
        'shared_libs': {
            'check': lambda pd: (pd / 'shared-libs').exists(),
            'description': 'Shared libraries directory',
            'install_cmd': 'mkdir -p shared-libs'
        },
        'orchestration_config': {
            'check': lambda pd: (pd / 'orchestration-config.json').exists(),
            'description': 'Orchestration configuration file',
            'install_cmd': 'Generate config from template'
        }
    },
    '1.7.0': {
        'orchestrate_command': {
            'check': lambda pd: (pd / '.claude/commands/orchestrate.md').exists(),
            'description': 'Unified /orchestrate command',
            'install_cmd': 'Copy orchestrate command template'
        },
        'adaptive_orchestration': {
            'check': lambda pd: (pd / 'orchestration/core/orchestrate.py').exists(),
            'description': 'Adaptive orchestration system',
            'install_cmd': 'Copy orchestrate.py from template'
        }
    },
    '1.7.2': {
        'component_naming': {
            'check': lambda pd: _check_component_naming(pd),
            'description': 'Component naming standard enforcement',
            'install_cmd': 'Run component name validator and fix'
        }
    },
    '1.7.4': {
        'pre_commit_hooks': {
            'check': lambda pd: (pd / '.git/hooks/pre-commit').exists(),
            'description': 'Git pre-commit hooks for enforcement',
            'install_cmd': 'bash orchestration/hooks/install_hooks.sh'
        }
    },
    '1.8.0': {
        'e2e_testing': {
            'check': lambda pd: (pd / 'orchestration/templates/e2e_test_template.py').exists(),
            'description': 'E2E test infrastructure',
            'install_cmd': 'Copy E2E test templates'
        },
        'validation_decorators': {
            'check': lambda pd: (pd / 'orchestration/validation/runtime_validators.py').exists(),
            'description': 'Runtime validation decorators',
            'install_cmd': 'Copy runtime validators'
        },
        'test_data_generators': {
            'check': lambda pd: (pd / 'orchestration/test_data/generators.py').exists(),
            'description': 'Test data generation system',
            'install_cmd': 'Copy test data generators'
        }
    }
}


def _check_component_naming(project_dir: Path) -> bool:
    """Check if all components follow naming standard"""
    components_dir = project_dir / "components"
    if not components_dir.exists():
        return True  # No components to check

    import re
    pattern = re.compile(r'^[a-z][a-z0-9_]*$')

    for component_dir in components_dir.iterdir():
        if component_dir.is_dir():
            if not pattern.match(component_dir.name):
                return False

    return True


class FeatureStatus(Enum):
    """Feature compliance status"""
    PRESENT = "present"
    MISSING = "missing"
    PARTIAL = "partial"


@dataclass
class FeatureCheck:
    """Result of a single feature check"""
    name: str
    version: str
    status: FeatureStatus
    description: str
    install_command: str
    details: Optional[str] = None


@dataclass
class UpgradePlan:
    """Plan for upgrading to target version"""
    current_version: str
    target_version: str
    missing_features: List[FeatureCheck]
    upgrade_steps: List[Dict]
    estimated_duration: str
    risk_level: str  # "low", "medium", "high"


@dataclass
class ComplianceReport:
    """Compliance check report"""
    current_version: str
    target_version: str
    compliance_percentage: float
    total_features: int
    present_features: int
    missing_features: int
    feature_checks: List[FeatureCheck] = field(default_factory=list)
    gaps: List[str] = field(default_factory=list)


class ComplianceChecker:
    """
    Checks project compliance against target orchestration version.

    Features:
    - Detect current orchestration version
    - Check for presence of required features
    - Generate gap analysis
    - Create upgrade plans
    """

    def __init__(self, project_dir: Path, target_version: str = "1.8.0"):
        """
        Initialize compliance checker.

        Args:
            project_dir: Root directory of target project
            target_version: Target orchestration version
        """
        self.project_dir = Path(project_dir).resolve()
        self.target_version = target_version

    def detect_current_version(self) -> str:
        """
        Detect currently installed orchestration version.

        Returns:
            Version string or "none" if not installed
        """
        version_file = self.project_dir / "orchestration/VERSION"

        if version_file.exists():
            with open(version_file, 'r') as f:
                version = f.read().strip()
                return version

        # Fallback: detect by features
        if (self.project_dir / "orchestration/test_data").exists():
            return "1.8.0"  # Has v1.8.0 features
        elif (self.project_dir / ".git/hooks/pre-commit").exists():
            return "1.7.4"  # Has v1.7.4 features
        elif (self.project_dir / ".claude/commands/orchestrate.md").exists():
            return "1.7.0"  # Has v1.7.0 features
        elif (self.project_dir / "orchestration").exists():
            return "1.0.0"  # Has basic orchestration
        else:
            return "none"  # No orchestration installed

    def check_compliance(self) -> ComplianceReport:
        """
        Check compliance against target version.

        Returns:
            Compliance report
        """
        current_version = self.detect_current_version()
        feature_checks = []
        present_count = 0
        missing_count = 0

        # Get all features up to target version
        required_features = self._get_required_features_for_version(self.target_version)

        # Check each feature
        for version, features in required_features.items():
            for feature_name, feature_spec in features.items():
                is_present = self._check_feature(feature_spec['check'])

                status = FeatureStatus.PRESENT if is_present else FeatureStatus.MISSING

                if is_present:
                    present_count += 1
                else:
                    missing_count += 1

                check = FeatureCheck(
                    name=feature_name,
                    version=version,
                    status=status,
                    description=feature_spec['description'],
                    install_command=feature_spec['install_cmd']
                )
                feature_checks.append(check)

        total = present_count + missing_count
        compliance_pct = (present_count / total * 100) if total > 0 else 0.0

        # Generate gap list
        gaps = [
            f"{check.description} (v{check.version})"
            for check in feature_checks
            if check.status == FeatureStatus.MISSING
        ]

        return ComplianceReport(
            current_version=current_version,
            target_version=self.target_version,
            compliance_percentage=compliance_pct,
            total_features=total,
            present_features=present_count,
            missing_features=missing_count,
            feature_checks=feature_checks,
            gaps=gaps
        )

    def _get_required_features_for_version(self, version: str) -> Dict:
        """Get all features required up to specified version"""
        required = {}

        versions = ['1.0.0', '1.7.0', '1.7.2', '1.7.4', '1.8.0']
        target_idx = versions.index(version) if version in versions else len(versions) - 1

        for i in range(target_idx + 1):
            ver = versions[i]
            if ver in VERSION_FEATURES:
                required[ver] = VERSION_FEATURES[ver]

        return required

    def _check_feature(self, check_func: Callable) -> bool:
        """
        Execute feature check function.

        Args:
            check_func: Check function that takes project_dir

        Returns:
            True if feature present
        """
        try:
            return check_func(self.project_dir)
        except Exception as e:
            print(f"Warning: Feature check failed: {e}", file=sys.stderr)
            return False

    def _detect_present_features(self) -> Dict[str, bool]:
        """Detect which features are currently present"""
        present = {}

        for version, features in VERSION_FEATURES.items():
            for feature_name, feature_spec in features.items():
                is_present = self._check_feature(feature_spec['check'])
                present[f"{version}:{feature_name}"] = is_present

        return present

    def _generate_gap_details(self, missing: List[str]) -> List[Dict]:
        """Generate detailed gap information"""
        gap_details = []

        for gap in missing:
            # Parse "version:feature" format
            if ':' in gap:
                version, feature_name = gap.split(':', 1)
                if version in VERSION_FEATURES and feature_name in VERSION_FEATURES[version]:
                    feature_spec = VERSION_FEATURES[version][feature_name]
                    gap_details.append({
                        "feature": feature_name,
                        "version": version,
                        "description": feature_spec['description'],
                        "install_command": feature_spec['install_cmd'],
                        "impact": "Required for v" + version
                    })

        return gap_details

    def generate_upgrade_plan(self) -> UpgradePlan:
        """
        Generate upgrade plan to reach target version.

        Returns:
            Upgrade plan with steps
        """
        compliance = self.check_compliance()

        # Get missing features
        missing_features = [
            check for check in compliance.feature_checks
            if check.status == FeatureStatus.MISSING
        ]

        # Sort by version
        missing_features.sort(key=lambda x: x.version)

        # Generate upgrade steps
        upgrade_steps = []

        for feature in missing_features:
            step = {
                "version": feature.version,
                "feature": feature.name,
                "description": feature.description,
                "command": feature.install_command,
                "estimated_time": "5-10 minutes"
            }
            upgrade_steps.append(step)

        # Determine risk level
        if compliance.missing_features == 0:
            risk_level = "none"
        elif compliance.missing_features <= 2:
            risk_level = "low"
        elif compliance.missing_features <= 5:
            risk_level = "medium"
        else:
            risk_level = "high"

        # Estimate duration
        total_minutes = len(missing_features) * 7.5  # Average 7.5 minutes per feature
        if total_minutes < 15:
            duration = "< 15 minutes"
        elif total_minutes < 60:
            duration = f"~{int(total_minutes)} minutes"
        else:
            duration = f"~{int(total_minutes / 60)} hours"

        return UpgradePlan(
            current_version=compliance.current_version,
            target_version=self.target_version,
            missing_features=missing_features,
            upgrade_steps=upgrade_steps,
            estimated_duration=duration,
            risk_level=risk_level
        )

    def calculate_compliance_percentage(self) -> float:
        """Calculate compliance percentage"""
        report = self.check_compliance()
        return report.compliance_percentage

    def print_report(self, report: ComplianceReport):
        """Print formatted compliance report"""
        print()
        print("=" * 70)
        print("  ORCHESTRATION COMPLIANCE REPORT")
        print("=" * 70)
        print()

        # Version info
        print(f"Current Version:  {report.current_version}")
        print(f"Target Version:   {report.target_version}")
        print()

        # Compliance summary
        print(f"Compliance:       {report.compliance_percentage:.1f}%")
        print(f"Present Features: {report.present_features}/{report.total_features}")
        print(f"Missing Features: {report.missing_features}/{report.total_features}")
        print()

        # Feature breakdown
        if report.missing_features > 0:
            print("MISSING FEATURES")
            print("-" * 70)

            by_version = {}
            for check in report.feature_checks:
                if check.status == FeatureStatus.MISSING:
                    if check.version not in by_version:
                        by_version[check.version] = []
                    by_version[check.version].append(check)

            for version in sorted(by_version.keys()):
                print(f"\nVersion {version}:")
                for check in by_version[version]:
                    print(f"  ❌ {check.description}")
                    print(f"     Install: {check.install_command}")

            print()

        # Present features
        present_checks = [c for c in report.feature_checks if c.status == FeatureStatus.PRESENT]
        if present_checks:
            print("PRESENT FEATURES")
            print("-" * 70)
            for check in present_checks:
                print(f"  ✅ {check.description} (v{check.version})")
            print()

        print("=" * 70)

    def print_upgrade_plan(self, plan: UpgradePlan):
        """Print formatted upgrade plan"""
        print()
        print("=" * 70)
        print("  UPGRADE PLAN")
        print("=" * 70)
        print()

        print(f"Current Version:    {plan.current_version}")
        print(f"Target Version:     {plan.target_version}")
        print(f"Missing Features:   {len(plan.missing_features)}")
        print(f"Estimated Duration: {plan.estimated_duration}")
        print(f"Risk Level:         {plan.risk_level.upper()}")
        print()

        if plan.upgrade_steps:
            print("UPGRADE STEPS")
            print("-" * 70)
            for i, step in enumerate(plan.upgrade_steps, 1):
                print(f"\n{i}. {step['description']} (v{step['version']})")
                print(f"   Command: {step['command']}")
                print(f"   Time:    {step['estimated_time']}")
        else:
            print("✅ No upgrade needed - fully compliant!")

        print()
        print("=" * 70)


def main():
    """CLI entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Orchestration Compliance Checker")
    parser.add_argument("project_dir", nargs="?", default=".",
                       help="Project directory (default: current)")
    parser.add_argument("--target-version", default="1.8.0",
                       help="Target version to check against (default: 1.8.0)")
    parser.add_argument("--current-version", action="store_true",
                       help="Show currently detected version")
    parser.add_argument("--upgrade-plan", action="store_true",
                       help="Generate upgrade plan")
    parser.add_argument("--json", action="store_true",
                       help="Output as JSON")

    args = parser.parse_args()

    checker = ComplianceChecker(Path(args.project_dir), args.target_version)

    # Show current version only
    if args.current_version:
        version = checker.detect_current_version()
        print(f"Current Version: {version}")
        return 0

    # Generate and show upgrade plan
    if args.upgrade_plan:
        plan = checker.generate_upgrade_plan()
        if args.json:
            output = {
                "current_version": plan.current_version,
                "target_version": plan.target_version,
                "missing_features": len(plan.missing_features),
                "estimated_duration": plan.estimated_duration,
                "risk_level": plan.risk_level,
                "steps": plan.upgrade_steps
            }
            print(json.dumps(output, indent=2))
        else:
            checker.print_upgrade_plan(plan)
        return 0

    # Full compliance report
    report = checker.check_compliance()

    if args.json:
        output = {
            "current_version": report.current_version,
            "target_version": report.target_version,
            "compliance_percentage": report.compliance_percentage,
            "total_features": report.total_features,
            "present_features": report.present_features,
            "missing_features": report.missing_features,
            "gaps": report.gaps
        }
        print(json.dumps(output, indent=2))
    else:
        checker.print_report(report)

    # Exit code based on compliance
    if report.compliance_percentage == 100.0:
        return 0
    elif report.compliance_percentage >= 75.0:
        return 1  # Mostly compliant
    else:
        return 2  # Low compliance


if __name__ == "__main__":
    sys.exit(main())
