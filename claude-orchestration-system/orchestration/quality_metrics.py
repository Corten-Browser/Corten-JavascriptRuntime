"""
Quality Metrics Tracker

Tracks quality metrics over time for all components.
Generates quality dashboards and trend reports.

Classes:
    QualityMetricsTracker: Main metrics tracking system
    MetricSnapshot: Point-in-time metrics for a component
    TrendAnalyzer: Analyzes metric trends over time

Usage:
    tracker = QualityMetricsTracker()
    tracker.record_metrics("components/user-api", quality_report)
    dashboard = tracker.generate_dashboard()
"""

import json
from pathlib import Path
from typing import Dict, List, Optional
from datetime import datetime, timedelta
from dataclasses import dataclass, field
import statistics


@dataclass
class MetricSnapshot:
    """Point-in-time quality metrics for a component."""
    component_name: str
    timestamp: str
    quality_score: int  # 0-100
    test_coverage: Optional[float] = None
    tests_passing: int = 0
    tests_total: int = 0
    tdd_score: Optional[int] = None
    linting_passed: bool = False
    formatting_passed: bool = False
    security_passed: bool = False
    documentation_score: Optional[int] = None

    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'component_name': self.component_name,
            'timestamp': self.timestamp,
            'quality_score': self.quality_score,
            'test_coverage': self.test_coverage,
            'tests_passing': self.tests_passing,
            'tests_total': self.tests_total,
            'tdd_score': self.tdd_score,
            'linting_passed': self.linting_passed,
            'formatting_passed': self.formatting_passed,
            'security_passed': self.security_passed,
            'documentation_score': self.documentation_score
        }


@dataclass
class ComponentMetrics:
    """Aggregated metrics for a component."""
    component_name: str
    current_score: int
    snapshots: List[MetricSnapshot]
    trend: str  # "improving", "stable", "declining"
    trend_value: int  # Change in score
    grade: str  # A, B, C, D, F

    def __post_init__(self):
        """Calculate grade from score."""
        if self.current_score >= 90:
            self.grade = "A"
        elif self.current_score >= 80:
            self.grade = "B"
        elif self.current_score >= 70:
            self.grade = "C"
        elif self.current_score >= 60:
            self.grade = "D"
        else:
            self.grade = "F"


class QualityMetricsTracker:
    """
    Track quality metrics over time.

    Stores metric snapshots in JSON file.
    Provides methods to analyze trends and generate reports.

    Attributes:
        metrics_file: Path to metrics storage file
    """

    def __init__(self, metrics_file: Optional[Path] = None):
        """
        Initialize QualityMetricsTracker.

        Args:
            metrics_file: Path to metrics JSON file
                (default: orchestration/quality-metrics.json)
        """
        if metrics_file is None:
            self.metrics_file = Path("orchestration/quality-metrics.json")
        else:
            self.metrics_file = Path(metrics_file)

        # Create file if it doesn't exist
        if not self.metrics_file.exists():
            self.metrics_file.parent.mkdir(parents=True, exist_ok=True)
            self._save_metrics({})

    def record_metrics(self, component_path: str, quality_report: Dict):
        """
        Record quality metrics from a verification report.

        Args:
            component_path: Path to component
            quality_report: Quality report dictionary from QualityVerifier

        Example:
            >>> from quality_verifier import QualityVerifier
            >>> verifier = QualityVerifier()
            >>> report = verifier.verify("components/user-api")
            >>> tracker.record_metrics("components/user-api", report.to_dict())
        """
        component_name = Path(component_path).name

        # Extract metrics from report
        snapshot = self._create_snapshot_from_report(component_name, quality_report)

        # Load existing metrics
        all_metrics = self._load_metrics()

        # Add snapshot to component history
        if component_name not in all_metrics:
            all_metrics[component_name] = []

        all_metrics[component_name].append(snapshot.to_dict())

        # Keep only last 100 snapshots per component
        all_metrics[component_name] = all_metrics[component_name][-100:]

        # Save updated metrics
        self._save_metrics(all_metrics)

    def get_component_metrics(self, component_name: str) -> Optional[ComponentMetrics]:
        """
        Get aggregated metrics for a component.

        Args:
            component_name: Name of component

        Returns:
            ComponentMetrics or None if no data
        """
        all_metrics = self._load_metrics()

        if component_name not in all_metrics:
            return None

        snapshots_data = all_metrics[component_name]
        snapshots = [
            MetricSnapshot(**s) for s in snapshots_data
        ]

        if not snapshots:
            return None

        # Current score (latest snapshot)
        current_score = snapshots[-1].quality_score

        # Calculate trend (compare last 5 snapshots to previous 5)
        trend, trend_value = self._calculate_trend(snapshots)

        return ComponentMetrics(
            component_name=component_name,
            current_score=current_score,
            snapshots=snapshots,
            trend=trend,
            trend_value=trend_value,
            grade=""  # Will be set in __post_init__
        )

    def get_all_component_metrics(self) -> List[ComponentMetrics]:
        """Get metrics for all components."""
        all_metrics = self._load_metrics()

        components = []
        for component_name in all_metrics.keys():
            metrics = self.get_component_metrics(component_name)
            if metrics:
                components.append(metrics)

        # Sort by score descending
        components.sort(key=lambda x: x.current_score, reverse=True)

        return components

    def generate_dashboard(self) -> str:
        """
        Generate quality dashboard in Markdown format.

        Returns:
            Markdown string with quality dashboard

        Example:
            >>> tracker = QualityMetricsTracker()
            >>> dashboard = tracker.generate_dashboard()
            >>> Path("docs/quality-dashboard.md").write_text(dashboard)
        """
        components = self.get_all_component_metrics()

        if not components:
            return "# Quality Dashboard\n\nNo metrics available yet."

        lines = [
            "# Quality Dashboard",
            "",
            f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            ""
        ]

        # Overall statistics
        avg_score = statistics.mean([c.current_score for c in components])
        lines.extend([
            "## Overall Statistics",
            "",
            f"- **Total Components**: {len(components)}",
            f"- **Average Quality Score**: {avg_score:.1f}/100",
            f"- **Components with A grade**: {sum(1 for c in components if c.grade == 'A')}",
            f"- **Components with B grade**: {sum(1 for c in components if c.grade == 'B')}",
            f"- **Components with C grade**: {sum(1 for c in components if c.grade == 'C')}",
            f"- **Components with D/F grade**: {sum(1 for c in components if c.grade in ['D', 'F'])}",
            ""
        ])

        # Component scores table
        lines.extend([
            "## Component Quality Scores",
            "",
            "| Component | Score | Grade | Trend | Coverage | Tests | Status |",
            "|-----------|-------|-------|-------|----------|-------|--------|"
        ])

        for comp in components:
            latest = comp.snapshots[-1]

            # Format trend
            trend_icon = {
                "improving": "↑",
                "stable": "→",
                "declining": "↓"
            }.get(comp.trend, "?")

            trend_str = f"{trend_icon} {comp.trend_value:+d}" if comp.trend_value != 0 else "→ 0"

            # Format coverage
            coverage_str = f"{latest.test_coverage:.1f}%" if latest.test_coverage else "N/A"

            # Format tests
            tests_str = f"{latest.tests_passing}/{latest.tests_total}"

            # Status icon
            if comp.current_score >= 90:
                status = "⭐"
            elif comp.current_score >= 80:
                status = "✅"
            elif comp.current_score >= 70:
                status = "⚠️"
            else:
                status = "❌"

            lines.append(
                f"| {comp.component_name} | {comp.current_score}/100 | {comp.grade} | "
                f"{trend_str} | {coverage_str} | {tests_str} | {status} |"
            )

        lines.extend(["", ""])

        # Detailed metrics per component
        lines.extend([
            "## Detailed Component Metrics",
            ""
        ])

        for comp in components:
            latest = comp.snapshots[-1]

            lines.extend([
                f"### {comp.component_name}",
                "",
                f"**Score**: {comp.current_score}/100 (Grade: {comp.grade})",
                "",
                f"**Trend**: {comp.trend.capitalize()} ({comp.trend_value:+d} points)",
                "",
                "**Latest Metrics**:",
                ""
            ])

            # Detailed metrics
            if latest.test_coverage is not None:
                lines.append(f"- Test Coverage: {latest.test_coverage:.1f}%")

            lines.append(f"- Tests: {latest.tests_passing}/{latest.tests_total} passing")

            if latest.tdd_score is not None:
                lines.append(f"- TDD Score: {latest.tdd_score}/100")

            lines.append(f"- Linting: {'✅ Passed' if latest.linting_passed else '❌ Failed'}")
            lines.append(f"- Formatting: {'✅ Passed' if latest.formatting_passed else '❌ Failed'}")
            lines.append(f"- Security: {'✅ Passed' if latest.security_passed else '❌ Failed'}")

            if latest.documentation_score is not None:
                lines.append(f"- Documentation: {latest.documentation_score}/100")

            lines.extend(["", ""])

        # Trends section
        lines.extend([
            "## Trends (Last 30 Days)",
            ""
        ])

        improving = [c for c in components if c.trend == "improving"]
        declining = [c for c in components if c.trend == "declining"]

        if improving:
            lines.extend([
                "**Improving Components** ↑:",
                ""
            ])
            for comp in improving:
                lines.append(f"- {comp.component_name}: {comp.trend_value:+d} points")
            lines.append("")

        if declining:
            lines.extend([
                "**Declining Components** ↓:",
                ""
            ])
            for comp in declining:
                lines.append(f"- {comp.component_name}: {comp.trend_value:+d} points")
            lines.append("")

        # Recommendations
        lines.extend([
            "## Recommendations",
            ""
        ])

        needs_attention = [c for c in components if c.current_score < 80]

        if needs_attention:
            lines.extend([
                "**Components Needing Attention** (Score < 80):",
                ""
            ])
            for comp in needs_attention:
                latest = comp.snapshots[-1]
                issues = []

                if latest.test_coverage and latest.test_coverage < 80:
                    issues.append("low coverage")
                if not latest.linting_passed:
                    issues.append("linting errors")
                if not latest.formatting_passed:
                    issues.append("formatting issues")
                if not latest.security_passed:
                    issues.append("security concerns")

                issues_str = ", ".join(issues) if issues else "general quality issues"
                lines.append(f"- **{comp.component_name}** ({comp.current_score}/100): {issues_str}")

            lines.append("")
        else:
            lines.extend([
                "✅ All components meet quality standards (≥80/100)!",
                ""
            ])

        return "\n".join(lines)

    def generate_report(self) -> str:
        """Generate text report of current quality metrics."""
        components = self.get_all_component_metrics()

        if not components:
            return "No quality metrics available."

        lines = [
            "Quality Metrics Report",
            "=" * 60,
            ""
        ]

        for comp in components:
            latest = comp.snapshots[-1]

            lines.extend([
                f"{comp.component_name}:",
                f"  Score: {comp.current_score}/100 (Grade: {comp.grade})",
                f"  Trend: {comp.trend} ({comp.trend_value:+d})",
                ""
            ])

            if latest.test_coverage:
                lines.append(f"  Coverage: {latest.test_coverage:.1f}%")

            lines.append(f"  Tests: {latest.tests_passing}/{latest.tests_total}")

            if latest.tdd_score:
                lines.append(f"  TDD: {latest.tdd_score}/100")

            lines.extend(["", ""])

        return "\n".join(lines)

    def _create_snapshot_from_report(
        self,
        component_name: str,
        quality_report: Dict
    ) -> MetricSnapshot:
        """Create metric snapshot from quality report."""
        # Extract metrics from checks
        test_coverage = None
        tests_passing = 0
        tests_total = 0
        tdd_score = None
        linting_passed = False
        formatting_passed = False
        security_passed = False
        documentation_score = None

        for check in quality_report.get('checks', []):
            if check['name'] == 'Test Coverage':
                test_coverage = check.get('details', {}).get('coverage')
            elif check['name'] == 'Tests Pass':
                tests_passing = check.get('details', {}).get('passing', 0)
                tests_total = tests_passing + check.get('details', {}).get('failing', 0)
            elif check['name'] == 'TDD Compliance':
                tdd_score = check.get('score')
            elif check['name'] == 'Linting':
                linting_passed = check.get('passed', False)
            elif check['name'] == 'Formatting':
                formatting_passed = check.get('passed', False)
            elif check['name'] == 'Security':
                security_passed = check.get('passed', False)
            elif check['name'] == 'Documentation':
                documentation_score = check.get('score')

        return MetricSnapshot(
            component_name=component_name,
            timestamp=quality_report['timestamp'],
            quality_score=quality_report['overall_score'],
            test_coverage=test_coverage,
            tests_passing=tests_passing,
            tests_total=tests_total,
            tdd_score=tdd_score,
            linting_passed=linting_passed,
            formatting_passed=formatting_passed,
            security_passed=security_passed,
            documentation_score=documentation_score
        )

    def _calculate_trend(self, snapshots: List[MetricSnapshot]) -> Tuple[str, int]:
        """
        Calculate quality trend.

        Args:
            snapshots: List of metric snapshots

        Returns:
            Tuple of (trend_name, trend_value)
            trend_name: "improving", "stable", or "declining"
            trend_value: Point change
        """
        if len(snapshots) < 2:
            return ("stable", 0)

        # Compare recent average to older average
        if len(snapshots) >= 10:
            # Use last 5 vs previous 5
            recent_scores = [s.quality_score for s in snapshots[-5:]]
            older_scores = [s.quality_score for s in snapshots[-10:-5]]
        else:
            # Use last half vs first half
            mid = len(snapshots) // 2
            recent_scores = [s.quality_score for s in snapshots[mid:]]
            older_scores = [s.quality_score for s in snapshots[:mid]]

        recent_avg = statistics.mean(recent_scores)
        older_avg = statistics.mean(older_scores)

        diff = int(recent_avg - older_avg)

        if diff > 3:
            return ("improving", diff)
        elif diff < -3:
            return ("declining", diff)
        else:
            return ("stable", diff)

    def _load_metrics(self) -> Dict:
        """Load metrics from file."""
        if not self.metrics_file.exists():
            return {}

        try:
            return json.loads(self.metrics_file.read_text())
        except (json.JSONDecodeError, FileNotFoundError):
            return {}

    def _save_metrics(self, metrics: Dict):
        """Save metrics to file."""
        self.metrics_file.write_text(json.dumps(metrics, indent=2))


def main():
    """CLI interface for quality metrics."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python quality_metrics.py <command> [args]")
        print("")
        print("Commands:")
        print("  report              - Print text report")
        print("  dashboard           - Generate Markdown dashboard")
        print("  record <component> <report.json>  - Record metrics from report")
        sys.exit(1)

    command = sys.argv[1]
    tracker = QualityMetricsTracker()

    if command == "report":
        print(tracker.generate_report())

    elif command == "dashboard":
        dashboard = tracker.generate_dashboard()
        print(dashboard)

        # Also save to file
        dashboard_path = Path("docs/quality-dashboard.md")
        dashboard_path.parent.mkdir(parents=True, exist_ok=True)
        dashboard_path.write_text(dashboard)
        print(f"\nDashboard saved to: {dashboard_path}")

    elif command == "record":
        if len(sys.argv) < 4:
            print("Usage: python quality_metrics.py record <component> <report.json>")
            sys.exit(1)

        component_path = sys.argv[2]
        report_path = Path(sys.argv[3])

        if not report_path.exists():
            print(f"Error: Report file not found: {report_path}")
            sys.exit(1)

        report_data = json.loads(report_path.read_text())
        tracker.record_metrics(component_path, report_data)

        print(f"Recorded metrics for {Path(component_path).name}")

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == "__main__":
    main()
