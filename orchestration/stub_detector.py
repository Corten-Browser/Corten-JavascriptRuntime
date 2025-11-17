#!/usr/bin/env python3
"""
Stub/Placeholder Detection System

Scans components for incomplete implementations that should block completion.

Key insight from failure analysis: Components with stubs are 0% complete,
regardless of how elaborate the "architecture" or "API surface" appears.
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass, field


@dataclass
class StubDetection:
    """A detected stub/placeholder in code."""
    file_path: str
    line_number: int
    code_snippet: str
    stub_type: str  # TODO, NotImplemented, placeholder, etc.
    severity: str  # critical, warning, info


@dataclass
class ComponentReport:
    """Report for a single component."""
    name: str
    path: str
    total_stubs: int
    critical_stubs: int
    stub_details: list[StubDetection] = field(default_factory=list)
    is_complete: bool = False
    readme_says_pending: bool = False


class StubDetector:
    """
    Detects stub/placeholder code that indicates incomplete implementation.

    This tool enforces Rule 4: No Stub/Placeholder Components.
    """

    # Critical stub patterns - MUST block completion
    CRITICAL_PATTERNS = [
        (r'raise\s+NotImplementedError', 'NotImplementedError', 'critical'),
        (r'pass\s*#\s*TODO', 'TODO pass', 'critical'),
        (r'return\s+(?:None|0|False|\[\]|\{\})\s*#\s*(?:stub|placeholder|TODO)', 'Stub return', 'critical'),
        (r'\.\.\.  # (?:stub|placeholder|TODO)', 'Ellipsis stub', 'critical'),
        (r'unimplemented!\(\)', 'Rust unimplemented', 'critical'),
        (r'todo!\(\)', 'Rust todo', 'critical'),
        (r'panic!\("not implemented', 'Rust panic placeholder', 'critical'),
    ]

    # Warning patterns - should be fixed but less critical
    WARNING_PATTERNS = [
        (r'#\s*TODO(?:\s*:|:?\s+)(.{0,100})', 'TODO comment', 'warning'),
        (r'#\s*FIXME(?:\s*:|:?\s+)(.{0,100})', 'FIXME comment', 'warning'),
        (r'//\s*TODO(?:\s*:|:?\s+)(.{0,100})', 'TODO comment (Rust/JS)', 'warning'),
        (r'//\s*FIXME(?:\s*:|:?\s+)(.{0,100})', 'FIXME comment (Rust/JS)', 'warning'),
        (r'placeholder', 'Placeholder keyword', 'warning'),
        (r'not\s+implemented', 'Not implemented text', 'warning'),
        (r'stub\s+implementation', 'Stub implementation', 'warning'),
    ]

    # README patterns that indicate incomplete components
    README_INCOMPLETE_PATTERNS = [
        r'implementation\s+pending',
        r'not\s+yet\s+implemented',
        r'skeleton\s+only',
        r'foundation\s+for\s+future',
        r'phase\s+\d+\s+will\s+implement',
        r'TODO:\s+implement',
        r'awaiting\s+implementation',
    ]

    def __init__(self, project_root: str):
        """Initialize detector with project root."""
        self.project_root = Path(project_root)
        self.component_reports: list[ComponentReport] = []

    def scan_component(self, component_path: str) -> ComponentReport:
        """Scan a single component for stubs."""
        comp_path = Path(component_path)
        if not comp_path.is_absolute():
            comp_path = self.project_root / component_path

        component_name = comp_path.name
        stubs = []

        # Scan source files
        for pattern in ['**/*.py', '**/*.rs', '**/*.js', '**/*.ts', '**/*.go']:
            for source_file in comp_path.rglob(pattern):
                if '__pycache__' in str(source_file) or 'node_modules' in str(source_file):
                    continue
                file_stubs = self._scan_file(source_file)
                stubs.extend(file_stubs)

        # Check README for "pending" language
        readme_pending = self._check_readme_for_pending(comp_path)

        critical_count = sum(1 for s in stubs if s.severity == 'critical')

        report = ComponentReport(
            name=component_name,
            path=str(comp_path),
            total_stubs=len(stubs),
            critical_stubs=critical_count,
            stub_details=stubs,
            is_complete=(critical_count == 0 and not readme_pending),
            readme_says_pending=readme_pending
        )

        self.component_reports.append(report)
        return report

    def scan_all_components(self, components_dir: str = "components") -> list[ComponentReport]:
        """Scan all components in directory."""
        comp_dir = self.project_root / components_dir
        if not comp_dir.exists():
            print(f"Warning: Components directory not found: {comp_dir}")
            return []

        self.component_reports = []

        for component in comp_dir.iterdir():
            if component.is_dir() and not component.name.startswith('.'):
                self.scan_component(str(component))

        return self.component_reports

    def _scan_file(self, file_path: Path) -> list[StubDetection]:
        """Scan a single file for stub patterns."""
        try:
            content = file_path.read_text()
        except Exception as e:
            print(f"Warning: Could not read {file_path}: {e}")
            return []

        detections = []
        lines = content.split('\n')

        # Check critical patterns
        for pattern, stub_type, severity in self.CRITICAL_PATTERNS:
            for i, line in enumerate(lines, 1):
                if re.search(pattern, line, re.IGNORECASE):
                    detections.append(StubDetection(
                        file_path=str(file_path.relative_to(self.project_root)),
                        line_number=i,
                        code_snippet=line.strip()[:200],
                        stub_type=stub_type,
                        severity=severity
                    ))

        # Check warning patterns
        for pattern, stub_type, severity in self.WARNING_PATTERNS:
            for i, line in enumerate(lines, 1):
                if re.search(pattern, line, re.IGNORECASE):
                    # Avoid duplicate detection
                    already_detected = any(
                        d.file_path == str(file_path.relative_to(self.project_root)) and d.line_number == i
                        for d in detections
                    )
                    if not already_detected:
                        detections.append(StubDetection(
                            file_path=str(file_path.relative_to(self.project_root)),
                            line_number=i,
                            code_snippet=line.strip()[:200],
                            stub_type=stub_type,
                            severity=severity
                        ))

        return detections

    def _check_readme_for_pending(self, component_path: Path) -> bool:
        """Check if component README indicates pending implementation."""
        readme_files = ['README.md', 'README.rst', 'README.txt', 'readme.md']

        for readme_name in readme_files:
            readme_path = component_path / readme_name
            if readme_path.exists():
                try:
                    content = readme_path.read_text().lower()
                    for pattern in self.README_INCOMPLETE_PATTERNS:
                        if re.search(pattern, content, re.IGNORECASE):
                            return True
                except Exception:
                    pass

        return False

    def get_blocking_stubs(self) -> list[StubDetection]:
        """Get all critical stubs that block completion."""
        blocking = []
        for report in self.component_reports:
            for stub in report.stub_details:
                if stub.severity == 'critical':
                    blocking.append(stub)
        return blocking

    def generate_report(self) -> str:
        """Generate comprehensive stub detection report."""
        lines = [
            "=" * 70,
            "STUB/PLACEHOLDER DETECTION REPORT",
            "=" * 70,
            "",
        ]

        total_components = len(self.component_reports)
        complete_components = sum(1 for r in self.component_reports if r.is_complete)
        total_stubs = sum(r.total_stubs for r in self.component_reports)
        critical_stubs = sum(r.critical_stubs for r in self.component_reports)

        lines.extend([
            f"Components Scanned: {total_components}",
            f"Complete Components: {complete_components}/{total_components}",
            f"Total Stubs Found: {total_stubs}",
            f"Critical Stubs (blocking): {critical_stubs}",
            "",
        ])

        if critical_stubs > 0 or complete_components < total_components:
            lines.append("❌ BLOCKING ISSUES DETECTED - CANNOT COMPLETE PROJECT")
        else:
            lines.append("✅ NO BLOCKING STUBS FOUND")

        lines.append("")

        # Report each component
        for report in self.component_reports:
            status = "✅ COMPLETE" if report.is_complete else "❌ INCOMPLETE"
            lines.append(f"Component: {report.name} - {status}")

            if report.readme_says_pending:
                lines.append(f"  ⚠️ README indicates implementation pending")

            if report.critical_stubs > 0:
                lines.append(f"  ❌ {report.critical_stubs} critical stubs (blocking)")
                for stub in report.stub_details:
                    if stub.severity == 'critical':
                        lines.append(f"     - {stub.file_path}:{stub.line_number}")
                        lines.append(f"       Type: {stub.stub_type}")
                        lines.append(f"       Code: {stub.code_snippet[:80]}...")

            if report.total_stubs - report.critical_stubs > 0:
                warning_count = report.total_stubs - report.critical_stubs
                lines.append(f"  ⚠️ {warning_count} warning-level stubs")

            lines.append("")

        lines.append("=" * 70)

        if critical_stubs > 0:
            lines.extend([
                "",
                "ACTION REQUIRED:",
                "Fix all critical stubs before claiming completion.",
                "Remember: 'Skeleton created' = 0% complete, not 'architecture done'.",
                "",
                "Rule 4 (No Stub/Placeholder Components) is VIOLATED.",
            ])

        return '\n'.join(lines)

    def is_project_complete(self) -> bool:
        """Check if project has no blocking stubs."""
        return all(r.is_complete for r in self.component_reports)


def main():
    """CLI entry point."""
    if len(sys.argv) < 2:
        print("Usage: python stub_detector.py <project_root> [components_dir]")
        print("Example: python stub_detector.py . components")
        sys.exit(1)

    project_root = sys.argv[1]
    components_dir = sys.argv[2] if len(sys.argv) > 2 else "components"

    detector = StubDetector(project_root)
    detector.scan_all_components(components_dir)

    report = detector.generate_report()
    print(report)

    if not detector.is_project_complete():
        print("\n❌ PROJECT HAS INCOMPLETE COMPONENTS - CANNOT STOP")
        sys.exit(1)
    else:
        print("\n✅ All components fully implemented")
        sys.exit(0)


if __name__ == "__main__":
    main()
