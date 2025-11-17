#!/usr/bin/env python3
"""
Complexity Estimator

Estimates component complexity and recommends appropriate time/resource allocation.

This solves the v0.2.0 problem where all components got the same time budget,
causing simple components to have excess time and complex ones to run out.

Part of v0.3.0 completion guarantee system.
"""

import sys
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass
import re


@dataclass
class ComplexityEstimate:
    """Complexity estimate and resource recommendations for a component."""
    component_name: str
    complexity_score: float  # 0-100
    complexity_level: str  # "simple", "moderate", "complex", "very_complex"
    estimated_minutes: int
    max_iterations: int
    checkpoint_frequency_minutes: int
    factors: Dict[str, float]  # Breakdown of complexity factors
    reasoning: str


class ComplexityEstimator:
    """Estimates component complexity for dynamic resource allocation."""

    # Base time estimates by component type (minutes)
    BASE_TIME_BY_TYPE = {
        "base": 30,         # Simple data types, utilities
        "core": 50,         # Business logic
        "feature": 70,      # Feature implementations
        "integration": 90,  # Integration orchestrators
        "application": 40,  # Entry points (minimal code)
    }

    # Complexity level thresholds
    COMPLEXITY_THRESHOLDS = {
        "simple": (0, 30),
        "moderate": (30, 55),
        "complex": (55, 75),
        "very_complex": (75, 100),
    }

    def __init__(self, project_root: Path):
        """
        Initialize estimator.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()

    def estimate_component(
        self,
        component_name: str,
        spec_content: Optional[str] = None,
        component_type: str = "feature",
        dependencies: Optional[List[str]] = None
    ) -> ComplexityEstimate:
        """
        Estimate complexity of a component.

        Args:
            component_name: Name of component
            spec_content: Specification text (from CLAUDE.md or requirements)
            component_type: Type of component (base/core/feature/integration/application)
            dependencies: List of component dependencies

        Returns:
            ComplexityEstimate with recommendations
        """
        dependencies = dependencies or []

        print(f"🔍 Estimating complexity for: {component_name}")

        # Calculate complexity factors
        factors = {}

        # Factor 1: Component type complexity (30%)
        type_complexity = self._calculate_type_complexity(component_type)
        factors["type"] = type_complexity

        # Factor 2: Dependency complexity (25%)
        dep_complexity = self._calculate_dependency_complexity(dependencies, component_type)
        factors["dependencies"] = dep_complexity

        # Factor 3: Specification complexity (25%)
        spec_complexity = self._calculate_spec_complexity(spec_content) if spec_content else 0
        factors["specification"] = spec_complexity

        # Factor 4: Integration complexity (20%)
        integration_complexity = self._calculate_integration_complexity(dependencies, component_type)
        factors["integration"] = integration_complexity

        # Weighted overall complexity score (0-100)
        complexity_score = (
            type_complexity * 0.30 +
            dep_complexity * 0.25 +
            spec_complexity * 0.25 +
            integration_complexity * 0.20
        )

        # Determine complexity level
        complexity_level = self._get_complexity_level(complexity_score)

        # Calculate resource recommendations
        base_time = self.BASE_TIME_BY_TYPE.get(component_type, 60)
        estimated_minutes = self._calculate_estimated_time(base_time, complexity_score)
        max_iterations = self._calculate_max_iterations(complexity_level)
        checkpoint_frequency = self._calculate_checkpoint_frequency(estimated_minutes)

        # Generate reasoning
        reasoning = self._generate_reasoning(
            component_type, complexity_score, complexity_level, factors, estimated_minutes
        )

        return ComplexityEstimate(
            component_name=component_name,
            complexity_score=complexity_score,
            complexity_level=complexity_level,
            estimated_minutes=estimated_minutes,
            max_iterations=max_iterations,
            checkpoint_frequency_minutes=checkpoint_frequency,
            factors=factors,
            reasoning=reasoning
        )

    def _calculate_type_complexity(self, component_type: str) -> float:
        """Calculate complexity based on component type (0-100)."""
        type_scores = {
            "base": 20,         # Simple, no dependencies
            "core": 40,         # Moderate business logic
            "feature": 60,      # Feature implementation
            "integration": 80,  # Complex orchestration
            "application": 30,  # Simple entry point
        }

        return type_scores.get(component_type, 50)

    def _calculate_dependency_complexity(self, dependencies: List[str], component_type: str) -> float:
        """Calculate complexity based on dependencies (0-100)."""
        dep_count = len(dependencies)

        # Expected dependency range by type
        expected_deps = {
            "base": (0, 0),
            "core": (1, 3),
            "feature": (2, 5),
            "integration": (5, 15),
            "application": (1, 3),
        }

        min_deps, max_deps = expected_deps.get(component_type, (0, 5))

        if dep_count <= min_deps:
            return 10  # Low complexity
        elif dep_count <= max_deps:
            # Linear scale within expected range
            progress = (dep_count - min_deps) / (max_deps - min_deps) if max_deps > min_deps else 0
            return 10 + (progress * 40)  # 10-50
        else:
            # Above expected, higher complexity
            excess = dep_count - max_deps
            return min(50 + (excess * 15), 100)  # 50-100

    def _calculate_spec_complexity(self, spec_content: str) -> float:
        """Calculate complexity based on specification content (0-100)."""
        if not spec_content:
            return 50  # Unknown, assume moderate

        # Factor 1: Length (longer specs = more complex requirements)
        length_score = min((len(spec_content) / 5000) * 40, 40)  # Up to 40 points

        # Factor 2: Number of sections/requirements
        sections = len(re.findall(r'^#{1,3}\s+', spec_content, re.MULTILINE))
        section_score = min((sections / 10) * 30, 30)  # Up to 30 points

        # Factor 3: Technical keywords (database, API, cache, auth, etc.)
        technical_keywords = [
            'database', 'api', 'cache', 'authentication', 'authorization',
            'encryption', 'validation', 'transaction', 'async', 'concurrent',
            'distributed', 'scalable', 'performance', 'optimization'
        ]

        keyword_count = sum(1 for keyword in technical_keywords if keyword.lower() in spec_content.lower())
        keyword_score = min((keyword_count / 5) * 30, 30)  # Up to 30 points

        return length_score + section_score + keyword_score

    def _calculate_integration_complexity(self, dependencies: List[str], component_type: str) -> float:
        """Calculate integration complexity (0-100)."""
        # Integration components have high integration complexity by definition
        if component_type == "integration":
            return 80

        # Application components coordinate but are minimal code
        if component_type == "application":
            return 40

        # Other components scale with dependencies
        if len(dependencies) == 0:
            return 10  # No integration needed

        elif len(dependencies) <= 2:
            return 30  # Simple integration

        elif len(dependencies) <= 5:
            return 50  # Moderate integration

        else:
            return 70  # Complex integration

    def _get_complexity_level(self, score: float) -> str:
        """Get complexity level from score."""
        for level, (min_score, max_score) in self.COMPLEXITY_THRESHOLDS.items():
            if min_score <= score < max_score:
                return level

        return "very_complex"  # >= 75

    def _calculate_estimated_time(self, base_time: int, complexity_score: float) -> int:
        """Calculate estimated time in minutes."""
        # Adjust base time by complexity multiplier
        multiplier = 1.0 + (complexity_score / 100)

        estimated = int(base_time * multiplier)

        # Round to nearest 15 minutes
        return ((estimated + 7) // 15) * 15

    def _calculate_max_iterations(self, complexity_level: str) -> int:
        """Calculate maximum recommended iterations."""
        iterations_by_level = {
            "simple": 2,
            "moderate": 3,
            "complex": 4,
            "very_complex": 5,
        }

        return iterations_by_level.get(complexity_level, 3)

    def _calculate_checkpoint_frequency(self, estimated_minutes: int) -> int:
        """Calculate recommended checkpoint frequency in minutes."""
        if estimated_minutes <= 60:
            return estimated_minutes  # No checkpoint needed

        elif estimated_minutes <= 120:
            return 60  # Checkpoint every hour

        else:
            return 90  # Checkpoint every 90 minutes

    def _generate_reasoning(
        self,
        component_type: str,
        complexity_score: float,
        complexity_level: str,
        factors: Dict[str, float],
        estimated_minutes: int
    ) -> str:
        """Generate human-readable reasoning for the estimate."""
        lines = []

        lines.append(f"Component type: {component_type}")
        lines.append(f"Overall complexity: {complexity_score:.1f}/100 ({complexity_level})")
        lines.append("")
        lines.append("Breakdown:")

        for factor, score in factors.items():
            lines.append(f"  - {factor.capitalize()}: {score:.1f}/100")

        lines.append("")
        lines.append(f"Recommended time budget: {estimated_minutes} minutes")

        return "\n".join(lines)

    def print_estimate(self, estimate: ComplexityEstimate) -> None:
        """Print detailed estimate report."""
        print("\n" + "="*70)
        print(f"COMPLEXITY ESTIMATE: {estimate.component_name}")
        print("="*70)

        print(f"\n📊 Complexity Score: {estimate.complexity_score:.1f}/100")
        print(f"   Level: {estimate.complexity_level.upper()}")

        print(f"\n⏱️  Recommended Time Budget: {estimate.estimated_minutes} minutes")
        print(f"   Maximum Iterations: {estimate.max_iterations}")
        print(f"   Checkpoint Frequency: Every {estimate.checkpoint_frequency_minutes} minutes")

        print("\n📈 Factor Breakdown:")
        for factor, score in estimate.factors.items():
            print(f"   {factor.capitalize():15} {score:5.1f}/100  {'█' * int(score/5)}")

        print(f"\n💡 Reasoning:\n{estimate.reasoning}")

        print("\n" + "="*70)


def estimate_from_manifest(project_root: Path, component_name: str) -> Optional[ComplexityEstimate]:
    """
    Estimate complexity from component manifest file.

    Args:
        project_root: Path to project root
        component_name: Name of component

    Returns:
        ComplexityEstimate or None if manifest not found
    """
    component_dir = project_root / "components" / component_name
    manifest_path = component_dir / "component.yaml"

    if not manifest_path.exists():
        print(f"⚠️  No manifest found for {component_name}")
        return None

    try:
        import yaml
        with open(manifest_path, 'r') as f:
            manifest = yaml.safe_load(f)

        component_type = manifest.get('type', 'feature')
        dependencies = [dep['name'] for dep in manifest.get('dependencies', {}).get('imports', [])]

        # Read specification from CLAUDE.md
        claude_md = component_dir / "CLAUDE.md"
        spec_content = claude_md.read_text() if claude_md.exists() else None

        estimator = ComplexityEstimator(project_root)
        return estimator.estimate_component(
            component_name=component_name,
            spec_content=spec_content,
            component_type=component_type,
            dependencies=dependencies
        )

    except ImportError:
        print("⚠️  PyYAML not installed, cannot read manifest")
        return None

    except Exception as e:
        print(f"❌ Error reading manifest: {e}")
        return None


def estimate_all_components(project_root: Path) -> List[ComplexityEstimate]:
    """
    Estimate complexity for all components in project.

    Args:
        project_root: Path to project root

    Returns:
        List of complexity estimates
    """
    components_dir = project_root / "components"

    if not components_dir.exists():
        print("❌ No components/ directory found")
        return []

    components = [d for d in components_dir.iterdir() if d.is_dir() and not d.name.startswith('.')]

    if not components:
        print("❌ No components found")
        return []

    estimates = []

    for component_dir in sorted(components):
        component_name = component_dir.name

        estimate = estimate_from_manifest(project_root, component_name)

        if estimate:
            estimates.append(estimate)

    return estimates


def main():
    """CLI interface for complexity estimator."""
    if len(sys.argv) < 2:
        print("Usage: complexity_estimator.py <component_name>")
        print("   or: complexity_estimator.py --all")
        print("\nEstimates component complexity and recommends time allocation.")
        print("\nExamples:")
        print("  python complexity_estimator.py audio_processor")
        print("  python complexity_estimator.py --all")
        sys.exit(1)

    project_root = Path.cwd()

    if sys.argv[1] == "--all":
        estimates = estimate_all_components(project_root)

        if not estimates:
            sys.exit(1)

        print("\n" + "="*70)
        print("COMPLEXITY ESTIMATES FOR ALL COMPONENTS")
        print("="*70)

        # Sort by complexity descending
        estimates.sort(key=lambda e: e.complexity_score, reverse=True)

        for estimate in estimates:
            print(f"\n{estimate.component_name:20} "
                  f"{estimate.complexity_level:12} "
                  f"{estimate.complexity_score:5.1f}/100 "
                  f"{estimate.estimated_minutes:4}min")

        print("\n" + "="*70)
        print(f"Total estimated time: {sum(e.estimated_minutes for e in estimates)} minutes")
        print("="*70)

        sys.exit(0)

    component_name = sys.argv[1]

    estimate = estimate_from_manifest(project_root, component_name)

    if not estimate:
        sys.exit(1)

    estimator = ComplexityEstimator(project_root)
    estimator.print_estimate(estimate)

    sys.exit(0)


if __name__ == '__main__':
    main()
