#!/usr/bin/env python3
"""
Scope Analyzer

Analyzes user requests to determine appropriate orchestration level.

Part of the adaptive orchestration system (v0.9.0).
"""

from typing import Dict, List, Optional
from enum import Enum
from pathlib import Path
import re


class OrchestrationLevel(Enum):
    """Orchestration complexity levels."""
    DIRECT = 1      # Single file, direct execution
    FEATURE = 2     # Multi-component feature
    FULL = 3        # Full orchestration with architecture


class ScopeAnalyzer:
    """Analyze request scope and suggest orchestration level."""

    def __init__(self, project_root: Path):
        """
        Initialize scope analyzer.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()
        self.components = self._discover_components()

    def analyze(self, user_request: str, override: Optional[str] = None) -> Dict:
        """
        Analyze user request and return orchestration recommendation.

        Args:
            user_request: The user's task description
            override: Optional level override (direct|feature|full)

        Returns:
            {
                'level': OrchestrationLevel,
                'score': int,
                'signals': List[str],
                'affected_components': List[str],
                'reasoning': str,
                'estimated_time': str
            }
        """
        # Handle override
        if override:
            return self._handle_override(override, user_request)

        # Calculate complexity score
        score = 0
        signals = []

        # Explicit scope indicators (+3 each)
        if self._has_spec_document(user_request):
            score += 3
            signals.append("Specification document mentioned (+3)")

        if "refactor architecture" in user_request.lower():
            score += 3
            signals.append("Architecture refactoring requested (+3)")

        if "split component" in user_request.lower():
            score += 3
            signals.append("Component splitting requested (+3)")

        # Component signals (+2 each)
        if "new component" in user_request.lower() or "create component" in user_request.lower():
            score += 2
            signals.append("New component creation (+2)")

        affected = self._identify_affected_components(user_request)
        if len(affected) >= 3:
            score += 2
            signals.append(f"{len(affected)} components affected (+2)")

        # Feature signals (+1 each)
        feature_keywords = ["add feature", "implement", "create", "build"]
        for keyword in feature_keywords:
            if keyword in user_request.lower():
                score += 1
                signals.append(f"Feature keyword: '{keyword}' (+1)")
                break

        # Simple change signals (-2 each)
        simple_keywords = ["fix typo", "quick fix", "just change", "update value", "change config"]
        for keyword in simple_keywords:
            if keyword in user_request.lower():
                score -= 2
                signals.append(f"Simple change keyword: '{keyword}' (-2)")
                break

        # File count analysis
        if self._mentions_single_file(user_request):
            if score == 0:  # Only apply if no other signals
                score -= 1
                signals.append("Single file mentioned (-1)")

        # Determine level
        if score >= 5:
            level = OrchestrationLevel.FULL
            reasoning = "High complexity score indicates full orchestration needed"
            estimated_time = "1-3 hours"
        elif score >= 2:
            level = OrchestrationLevel.FEATURE
            reasoning = "Moderate complexity indicates feature-level orchestration"
            estimated_time = "15-30 minutes"
        else:
            level = OrchestrationLevel.DIRECT
            reasoning = "Low complexity allows direct execution"
            estimated_time = "2-5 minutes"

        return {
            'level': level,
            'score': score,
            'signals': signals,
            'affected_components': affected,
            'reasoning': reasoning,
            'estimated_time': estimated_time
        }

    def _discover_components(self) -> List[str]:
        """Discover existing components in project."""
        components_dir = self.project_root / "components"
        if not components_dir.exists():
            return []

        return [d.name for d in components_dir.iterdir()
                if d.is_dir() and not d.name.startswith('.')]

    def _has_spec_document(self, request: str) -> bool:
        """Check if request mentions a specification document."""
        # Updated patterns to handle file paths (with / or \)
        patterns = [
            r'implement\s+[\w\-_/\\]+\.md',
            r'specification:?\s+[\w\-_/\\]+\.md',
            r'spec:?\s+[\w\-_/\\]+\.md',
            r'requirements:?\s+[\w\-_/\\]+\.md',
            r'read\s+[\w\-_/\\]+\.md',
            r'based on\s+[\w\-_/\\]+\.md'
        ]

        for pattern in patterns:
            if re.search(pattern, request, re.IGNORECASE):
                return True
        return False

    def _identify_affected_components(self, request: str) -> List[str]:
        """Identify which components are mentioned in request."""
        affected = []
        request_lower = request.lower()

        for component in self.components:
            # Check if component name is mentioned
            # Use word boundaries to avoid false matches
            pattern = r'\b' + re.escape(component.lower()) + r'\b'
            if re.search(pattern, request_lower):
                affected.append(component)

        return affected

    def _mentions_single_file(self, request: str) -> bool:
        """Check if request mentions a single specific file."""
        # Look for file extensions
        file_patterns = [
            r'\w+\.py\b',
            r'\w+\.ts\b',
            r'\w+\.js\b',
            r'\w+\.yaml\b',
            r'\w+\.json\b',
            r'\w+\.md\b',
            r'\w+\.txt\b'
        ]

        matches = 0
        for pattern in file_patterns:
            matches += len(re.findall(pattern, request, re.IGNORECASE))

        return matches == 1

    def _handle_override(self, override: str, request: str) -> Dict:
        """Handle user override of orchestration level."""
        level_map = {
            'direct': OrchestrationLevel.DIRECT,
            'feature': OrchestrationLevel.FEATURE,
            'full': OrchestrationLevel.FULL,
            '1': OrchestrationLevel.DIRECT,
            '2': OrchestrationLevel.FEATURE,
            '3': OrchestrationLevel.FULL
        }

        level = level_map.get(override.lower())
        if not level:
            raise ValueError(f"Invalid override: {override}. Use direct|feature|full or 1|2|3")

        time_estimates = {
            OrchestrationLevel.DIRECT: "2-5 minutes",
            OrchestrationLevel.FEATURE: "15-30 minutes",
            OrchestrationLevel.FULL: "1-3 hours"
        }

        return {
            'level': level,
            'score': None,
            'signals': [f'User override: --level={override}'],
            'affected_components': [],
            'reasoning': f'User explicitly requested {level.name} level orchestration',
            'estimated_time': time_estimates[level]
        }


def print_analysis_report(result: Dict) -> None:
    """Print formatted analysis report."""
    print("\n" + "="*60)
    print("SCOPE ANALYSIS")
    print("="*60)

    level_names = {
        OrchestrationLevel.DIRECT: "Level 1: Direct Execution",
        OrchestrationLevel.FEATURE: "Level 2: Feature Orchestration",
        OrchestrationLevel.FULL: "Level 3: Full Orchestration"
    }

    print(f"\nRecommended Level: {level_names[result['level']]}")

    if result['score'] is not None:
        print(f"Complexity Score: {result['score']}")

    print(f"Estimated Time: {result['estimated_time']}")
    print(f"\nReasoning: {result['reasoning']}")

    if result['signals']:
        print(f"\nSignals detected:")
        for signal in result['signals']:
            print(f"  • {signal}")

    if result['affected_components']:
        print(f"\nAffected components:")
        for comp in result['affected_components']:
            print(f"  • {comp}")

    print("="*60 + "\n")


def main():
    """CLI interface for scope analysis."""
    import sys
    import argparse

    parser = argparse.ArgumentParser(
        description='Analyze orchestration scope for a task'
    )
    parser.add_argument(
        'request',
        nargs='+',
        help='The task request to analyze'
    )
    parser.add_argument(
        '--level',
        choices=['direct', 'feature', 'full', '1', '2', '3'],
        help='Override orchestration level'
    )
    parser.add_argument(
        '--project-root',
        type=Path,
        default=Path.cwd(),
        help='Project root directory (default: current directory)'
    )

    args = parser.parse_args()

    # Join request words
    request = ' '.join(args.request)

    # Analyze scope
    analyzer = ScopeAnalyzer(args.project_root)
    result = analyzer.analyze(request, args.level)

    # Print report
    print_analysis_report(result)

    # Exit code based on level (for scripting)
    sys.exit(result['level'].value)


if __name__ == '__main__':
    main()
