#!/usr/bin/env python3
"""
Requirements Traceability Matrix System

Tracks every requirement from specification to implementation.
Part of v0.4.0 quality enhancement system.

This module provides:
- Requirement parsing from specifications
- Implementation tracking via code markers
- Test tracing and validation
- Complete traceability matrix generation
- Coverage analysis and gap detection
"""

from pathlib import Path
from typing import List, Dict, Optional, Set, Tuple
from dataclasses import dataclass, asdict, field
import json
import re
import ast
from datetime import datetime


@dataclass
class Requirement:
    """A single requirement."""
    id: str  # REQ-001, REQ-002, etc.
    text: str
    source: str  # File:line
    priority: str  # "MUST", "SHOULD", "MAY"
    category: str  # "authentication", "payment", etc.
    status: str  # "pending", "implemented", "tested", "complete"
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> 'Requirement':
        """Create from dictionary."""
        return cls(**data)


@dataclass
class Implementation:
    """Implementation location for a requirement."""
    file: str
    line: int
    function: Optional[str]
    description: str

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> 'Implementation':
        """Create from dictionary."""
        return cls(**data)


@dataclass
class RequirementTest:
    """Test for a requirement."""
    file: str
    line: int
    test_name: str
    status: str  # "passing", "failing", "not_run"

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> 'RequirementTest':
        """Create from dictionary."""
        return cls(**data)


@dataclass
class RequirementTrace:
    """Complete trace for a requirement."""
    requirement: Requirement
    implementations: List[Implementation] = field(default_factory=list)
    tests: List[RequirementTest] = field(default_factory=list)

    def is_implemented(self) -> bool:
        """Check if requirement has implementation."""
        return len(self.implementations) > 0

    def is_tested(self) -> bool:
        """Check if requirement has tests."""
        return len(self.tests) > 0

    def is_complete(self) -> bool:
        """Check if requirement is fully traced."""
        return (
            len(self.implementations) > 0 and
            len(self.tests) > 0 and
            all(t.status == "passing" for t in self.tests)
        )

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return {
            'requirement': self.requirement.to_dict(),
            'implementations': [i.to_dict() for i in self.implementations],
            'tests': [t.to_dict() for t in self.tests]
        }

    @classmethod
    def from_dict(cls, data: dict) -> 'RequirementTrace':
        """Create from dictionary."""
        return cls(
            requirement=Requirement.from_dict(data['requirement']),
            implementations=[Implementation.from_dict(i) for i in data.get('implementations', [])],
            tests=[RequirementTest.from_dict(t) for t in data.get('tests', [])]
        )


@dataclass
class TraceabilityMatrix:
    """Complete traceability matrix."""
    requirements: List[RequirementTrace]
    total_requirements: int
    implemented_requirements: int
    tested_requirements: int
    complete_requirements: int
    coverage_percentage: float
    generated_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return {
            'requirements': [r.to_dict() for r in self.requirements],
            'total_requirements': self.total_requirements,
            'implemented_requirements': self.implemented_requirements,
            'tested_requirements': self.tested_requirements,
            'complete_requirements': self.complete_requirements,
            'coverage_percentage': self.coverage_percentage,
            'generated_at': self.generated_at
        }


class RequirementsTracker:
    """Main requirements tracking system."""

    def __init__(self, project_root: Path):
        """
        Initialize requirements tracker.

        Args:
            project_root: Root directory of the project
        """
        self.project_root = Path(project_root)
        self.db_file = self.project_root / "orchestration" / "requirements_db.json"
        self.requirements: Dict[str, RequirementTrace] = self._load_database()

    def _load_database(self) -> Dict[str, RequirementTrace]:
        """Load requirements database from JSON file."""
        if self.db_file.exists():
            try:
                with open(self.db_file, 'r') as f:
                    data = json.load(f)
                return {
                    req_id: RequirementTrace.from_dict(trace_data)
                    for req_id, trace_data in data.items()
                }
            except (json.JSONDecodeError, KeyError) as e:
                print(f"Warning: Could not load requirements database: {e}")
                return {}
        return {}

    def _save_database(self):
        """Save requirements database to JSON file."""
        data = {
            req_id: trace.to_dict()
            for req_id, trace in self.requirements.items()
        }
        self.db_file.parent.mkdir(parents=True, exist_ok=True)
        with open(self.db_file, 'w') as f:
            json.dump(data, f, indent=2)

    def parse_requirements(self, spec_file: Path) -> List[Requirement]:
        """
        Extract requirements from specification.

        Patterns supported:
        - "MUST do X" -> Priority: MUST
        - "SHOULD do X" -> Priority: SHOULD
        - "MAY do X" -> Priority: MAY
        - "User story: As a user, I want X" -> Extract X
        - "REQ-001: Description" -> Explicit ID
        - Numbered lists: "1. Requirement" -> Auto-assign ID

        Args:
            spec_file: Path to specification file

        Returns:
            List of parsed requirements
        """
        if not spec_file.exists():
            raise FileNotFoundError(f"Specification file not found: {spec_file}")

        spec_text = spec_file.read_text()
        requirements = []
        req_id_counter = 1
        used_ids: Set[str] = set()

        # Pattern 1: Explicit requirement IDs (highest priority)
        explicit_pattern = r'(REQ-\d+):\s*(.+?)(?:\n|$)'
        for match in re.finditer(explicit_pattern, spec_text, re.MULTILINE):
            req_id = match.group(1)
            req_text = match.group(2).strip()

            if req_id in used_ids:
                print(f"Warning: Duplicate requirement ID {req_id}")
                continue

            # Determine priority from text
            priority = self._extract_priority(req_text)

            req = Requirement(
                id=req_id,
                text=req_text,
                source=f"{spec_file}:line:{self._get_line_number(spec_text, match.start())}",
                priority=priority,
                category=self._infer_category(req_text),
                status="pending"
            )
            requirements.append(req)
            used_ids.add(req_id)

            # Update counter if explicit ID is higher
            id_num = int(req_id.split('-')[1])
            if id_num >= req_id_counter:
                req_id_counter = id_num + 1

        # Pattern 2: MUST/SHALL statements
        must_pattern = r'(?:MUST|SHALL)\s+(.+?)(?:\.|$)'
        requirements.extend(self._extract_requirements_by_pattern(
            spec_text, spec_file, must_pattern, "MUST",
            req_id_counter, used_ids
        ))
        req_id_counter += len([r for r in requirements if r.priority == "MUST"])

        # Pattern 3: SHOULD statements
        should_pattern = r'SHOULD\s+(.+?)(?:\.|$)'
        requirements.extend(self._extract_requirements_by_pattern(
            spec_text, spec_file, should_pattern, "SHOULD",
            req_id_counter, used_ids
        ))
        req_id_counter += len([r for r in requirements if r.priority == "SHOULD"])

        # Pattern 4: MAY statements
        may_pattern = r'MAY\s+(.+?)(?:\.|$)'
        requirements.extend(self._extract_requirements_by_pattern(
            spec_text, spec_file, may_pattern, "MAY",
            req_id_counter, used_ids
        ))

        # Pattern 5: User stories
        story_pattern = r'(?:User story:|As a \w+,)\s*(?:As a \w+,\s*)?I want (.+?)(?:\.|$|\n)'
        for match in re.finditer(story_pattern, spec_text, re.IGNORECASE):
            req_text = match.group(1).strip()
            req_id = f"REQ-{req_id_counter:03d}"

            if req_id in used_ids:
                req_id_counter += 1
                continue

            req = Requirement(
                id=req_id,
                text=req_text,
                source=f"{spec_file}:line:{self._get_line_number(spec_text, match.start())}",
                priority="SHOULD",  # User stories typically SHOULD be implemented
                category=self._infer_category(req_text),
                status="pending"
            )
            requirements.append(req)
            used_ids.add(req_id)
            req_id_counter += 1

        return requirements

    def _extract_requirements_by_pattern(
        self,
        spec_text: str,
        spec_file: Path,
        pattern: str,
        priority: str,
        start_counter: int,
        used_ids: Set[str]
    ) -> List[Requirement]:
        """Extract requirements matching a specific pattern."""
        requirements = []
        counter = start_counter

        for match in re.finditer(pattern, spec_text, re.IGNORECASE):
            req_text = match.group(1).strip()
            req_id = f"REQ-{counter:03d}"

            # Skip if already extracted as explicit requirement
            if req_id in used_ids:
                counter += 1
                continue

            req = Requirement(
                id=req_id,
                text=req_text,
                source=f"{spec_file}:line:{self._get_line_number(spec_text, match.start())}",
                priority=priority,
                category=self._infer_category(req_text),
                status="pending"
            )
            requirements.append(req)
            used_ids.add(req_id)
            counter += 1

        return requirements

    def _extract_priority(self, text: str) -> str:
        """Extract priority from requirement text."""
        text_upper = text.upper()
        if 'MUST' in text_upper or 'SHALL' in text_upper:
            return 'MUST'
        elif 'SHOULD' in text_upper:
            return 'SHOULD'
        elif 'MAY' in text_upper:
            return 'MAY'
        return 'SHOULD'  # Default

    def scan_implementation(self, component_path: Path) -> List[Tuple[str, Implementation]]:
        """
        Find requirement markers in code.

        Supported markers:
        - @implements: REQ-001
        - @satisfies: REQ-002
        - @fulfills: REQ-003
        - # REQ-001: Implementation
        - # Implements REQ-001

        Args:
            component_path: Path to component directory

        Returns:
            List of (requirement_id, Implementation) tuples
        """
        implementations = []

        # Ensure component_path is absolute
        component_path = Path(component_path).resolve()

        # Search in both .py files and other source files
        patterns = ['**/*.py', '**/*.js', '**/*.ts', '**/*.java']

        for pattern in patterns:
            for src_file in component_path.glob(pattern):
                if 'test' in src_file.name.lower():
                    continue  # Skip test files

                try:
                    with open(src_file, 'r', encoding='utf-8') as f:
                        content = f.read()
                except (UnicodeDecodeError, PermissionError):
                    continue

                # Try to make path relative to project root, fall back to component path
                try:
                    rel_path = src_file.relative_to(self.project_root)
                except ValueError:
                    rel_path = src_file.relative_to(component_path)

                # Pattern 1: Decorator-style markers
                decorator_pattern = r'@(?:implements|satisfies|fulfills):\s*(REQ-\d+)'
                for match in re.finditer(decorator_pattern, content):
                    req_id = match.group(1)
                    line_num = self._get_line_number(content, match.start())
                    function = self._find_function_at_line(src_file, line_num)

                    impl = Implementation(
                        file=str(rel_path),
                        line=line_num,
                        function=function,
                        description=f"Implements {req_id}"
                    )
                    implementations.append((req_id, impl))

                # Pattern 2: Comment-style markers
                comment_pattern = r'#\s*(?:(?:Implements|Satisfies|Fulfills)\s+)?(REQ-\d+)'
                for match in re.finditer(comment_pattern, content):
                    req_id = match.group(1)
                    line_num = self._get_line_number(content, match.start())
                    function = self._find_function_at_line(src_file, line_num)

                    impl = Implementation(
                        file=str(rel_path),
                        line=line_num,
                        function=function,
                        description=f"Implements {req_id}"
                    )
                    implementations.append((req_id, impl))

        return implementations

    def scan_tests(self, component_path: Path) -> List[Tuple[str, RequirementTest]]:
        """
        Find tests that validate requirements.

        Supported markers:
        - @validates: REQ-001
        - # Tests REQ-001
        - def test_req_001_...

        Args:
            component_path: Path to component directory

        Returns:
            List of (requirement_id, RequirementTest) tuples
        """
        tests = []

        # Ensure component_path is absolute
        component_path = Path(component_path).resolve()

        for test_file in component_path.rglob("test_*.py"):
            try:
                with open(test_file, 'r', encoding='utf-8') as f:
                    content = f.read()
            except (UnicodeDecodeError, PermissionError):
                continue

            # Try to make path relative to project root, fall back to component path
            try:
                rel_path = test_file.relative_to(self.project_root)
            except ValueError:
                rel_path = test_file.relative_to(component_path)

            # Pattern 1: @validates decorator
            decorator_pattern = r'@validates:\s*(REQ-\d+)'
            for match in re.finditer(decorator_pattern, content):
                req_id = match.group(1)
                line_num = self._get_line_number(content, match.start())
                test_name = self._find_test_at_line(test_file, line_num)

                test = RequirementTest(
                    file=str(rel_path),
                    line=line_num,
                    test_name=test_name or "unknown",
                    status="not_run"
                )
                tests.append((req_id, test))

            # Pattern 2: Comment markers
            comment_pattern = r'#\s*(?:Tests|Validates)\s+(REQ-\d+)'
            for match in re.finditer(comment_pattern, content):
                req_id = match.group(1)
                line_num = self._get_line_number(content, match.start())
                test_name = self._find_test_at_line(test_file, line_num)

                test = RequirementTest(
                    file=str(rel_path),
                    line=line_num,
                    test_name=test_name or "unknown",
                    status="not_run"
                )
                tests.append((req_id, test))

            # Pattern 3: Test function naming convention
            func_pattern = r'def\s+(test_req_(\d+)_\w+)'
            for match in re.finditer(func_pattern, content):
                test_name = match.group(1)
                req_num = match.group(2)
                req_id = f"REQ-{int(req_num):03d}"
                line_num = self._get_line_number(content, match.start())

                test = RequirementTest(
                    file=str(rel_path),
                    line=line_num,
                    test_name=test_name,
                    status="not_run"
                )
                tests.append((req_id, test))

        return tests

    def add_requirements(self, requirements: List[Requirement]):
        """Add requirements to the database."""
        for req in requirements:
            if req.id not in self.requirements:
                self.requirements[req.id] = RequirementTrace(requirement=req)
            else:
                # Update existing requirement
                self.requirements[req.id].requirement = req
        self._save_database()

    def add_implementations(self, implementations: List[Tuple[str, Implementation]]):
        """Add implementations to the database."""
        for req_id, impl in implementations:
            if req_id not in self.requirements:
                print(f"Warning: Implementation for unknown requirement {req_id}")
                continue

            # Avoid duplicates
            existing = self.requirements[req_id].implementations
            if not any(e.file == impl.file and e.line == impl.line for e in existing):
                self.requirements[req_id].implementations.append(impl)

        self._save_database()

    def add_tests(self, tests: List[Tuple[str, RequirementTest]]):
        """Add tests to the database."""
        for req_id, test in tests:
            if req_id not in self.requirements:
                print(f"Warning: Test for unknown requirement {req_id}")
                continue

            # Avoid duplicates
            existing = self.requirements[req_id].tests
            if not any(e.file == test.file and e.line == test.line for e in existing):
                self.requirements[req_id].tests.append(test)

        self._save_database()

    def generate_traceability_matrix(self) -> TraceabilityMatrix:
        """Generate complete traceability matrix."""
        total = len(self.requirements)
        implemented = sum(1 for trace in self.requirements.values() if trace.is_implemented())
        tested = sum(1 for trace in self.requirements.values() if trace.is_tested())
        complete = sum(1 for trace in self.requirements.values() if trace.is_complete())

        coverage = (complete / total * 100) if total > 0 else 0.0

        return TraceabilityMatrix(
            requirements=list(self.requirements.values()),
            total_requirements=total,
            implemented_requirements=implemented,
            tested_requirements=tested,
            complete_requirements=complete,
            coverage_percentage=coverage
        )

    def find_unimplemented_requirements(self) -> List[Requirement]:
        """Find requirements with no implementation."""
        return [
            trace.requirement
            for trace in self.requirements.values()
            if not trace.is_implemented()
        ]

    def find_untested_requirements(self) -> List[Requirement]:
        """Find requirements with no tests."""
        return [
            trace.requirement
            for trace in self.requirements.values()
            if not trace.is_tested()
        ]

    def verify_requirement_coverage(self) -> Dict[str, float]:
        """Calculate coverage percentages by category."""
        coverage_by_category: Dict[str, Dict[str, int]] = {}

        for trace in self.requirements.values():
            category = trace.requirement.category
            if category not in coverage_by_category:
                coverage_by_category[category] = {
                    'total': 0,
                    'implemented': 0,
                    'tested': 0,
                    'complete': 0
                }

            coverage_by_category[category]['total'] += 1
            if trace.is_implemented():
                coverage_by_category[category]['implemented'] += 1
            if trace.is_tested():
                coverage_by_category[category]['tested'] += 1
            if trace.is_complete():
                coverage_by_category[category]['complete'] += 1

        # Calculate percentages
        result = {}
        for category, counts in coverage_by_category.items():
            total = counts['total']
            result[category] = {
                'total': total,
                'implementation_coverage': (counts['implemented'] / total * 100) if total > 0 else 0,
                'test_coverage': (counts['tested'] / total * 100) if total > 0 else 0,
                'complete_coverage': (counts['complete'] / total * 100) if total > 0 else 0
            }

        return result

    def _get_line_number(self, text: str, position: int) -> int:
        """Get line number from character position."""
        return text[:position].count('\n') + 1

    def _infer_category(self, text: str) -> str:
        """Infer requirement category from text."""
        keywords = {
            'authentication': ['login', 'password', 'auth', 'token', 'credential', 'session'],
            'authorization': ['permission', 'role', 'access', 'privilege', 'authorize'],
            'payment': ['payment', 'transaction', 'billing', 'invoice', 'charge'],
            'security': ['encrypt', 'secure', 'hash', 'protect', 'sanitize', 'validate'],
            'performance': ['fast', 'optimize', 'cache', 'performance', 'speed', 'latency'],
            'database': ['database', 'query', 'persist', 'store', 'retrieve', 'sql'],
            'api': ['api', 'endpoint', 'rest', 'graphql', 'request', 'response'],
            'ui': ['user interface', 'display', 'render', 'view', 'component', 'widget'],
            'testing': ['test', 'verify', 'validate', 'assert', 'check'],
            'monitoring': ['monitor', 'log', 'metric', 'alert', 'trace', 'observe']
        }

        text_lower = text.lower()
        for category, kw_list in keywords.items():
            if any(kw in text_lower for kw in kw_list):
                return category

        return 'general'

    def _find_function_at_line(self, file_path: Path, line_num: int) -> Optional[str]:
        """Find function name at given line using AST."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()

            tree = ast.parse(content)

            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    if hasattr(node, 'lineno') and node.lineno <= line_num:
                        # Check if line is within function body
                        end_line = node.end_lineno if hasattr(node, 'end_lineno') else node.lineno
                        if line_num <= end_line:
                            return node.name

            return None
        except (SyntaxError, UnicodeDecodeError):
            return None

    def _find_test_at_line(self, file_path: Path, line_num: int) -> Optional[str]:
        """Find test name at given line using AST."""
        return self._find_function_at_line(file_path, line_num)


def main():
    """CLI interface."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: requirements_tracker.py <command> [args]")
        print("\nCommands:")
        print("  parse <spec_file>           - Parse requirements from specification")
        print("  scan <component_path>       - Scan for implementations and tests")
        print("  matrix                      - Generate traceability matrix")
        print("  coverage                    - Show coverage statistics")
        print("  unimplemented               - List unimplemented requirements")
        print("  untested                    - List untested requirements")
        print("  full-scan                   - Parse specs and scan entire project")
        sys.exit(1)

    command = sys.argv[1]
    project_root = Path.cwd()
    tracker = RequirementsTracker(project_root)

    if command == "parse":
        if len(sys.argv) < 3:
            print("Usage: requirements_tracker.py parse <spec_file>")
            sys.exit(1)

        spec_file = Path(sys.argv[2])
        requirements = tracker.parse_requirements(spec_file)
        tracker.add_requirements(requirements)
        print(f"Parsed {len(requirements)} requirements")
        for req in requirements:
            print(f"  {req.id}: [{req.priority}] {req.text[:60]}...")

    elif command == "scan":
        if len(sys.argv) < 3:
            print("Usage: requirements_tracker.py scan <component_path>")
            sys.exit(1)

        component_path = Path(sys.argv[2])
        implementations = tracker.scan_implementation(component_path)
        tests = tracker.scan_tests(component_path)

        tracker.add_implementations(implementations)
        tracker.add_tests(tests)

        print(f"Found {len(implementations)} implementations")
        print(f"Found {len(tests)} tests")

    elif command == "matrix":
        matrix = tracker.generate_traceability_matrix()
        print("\n=== Traceability Matrix ===")
        print(f"Total Requirements:      {matrix.total_requirements}")
        print(f"Implemented:             {matrix.implemented_requirements}")
        print(f"Tested:                  {matrix.tested_requirements}")
        print(f"Complete:                {matrix.complete_requirements}")
        print(f"Coverage:                {matrix.coverage_percentage:.1f}%")
        print(f"Generated:               {matrix.generated_at}")

        # Show incomplete requirements
        incomplete = [r for r in matrix.requirements if not r.is_complete()]
        if incomplete:
            print(f"\n{len(incomplete)} Incomplete Requirements:")
            for trace in incomplete[:10]:  # Show first 10
                req = trace.requirement
                impl = "✓" if trace.is_implemented() else "✗"
                test = "✓" if trace.is_tested() else "✗"
                print(f"  {req.id} [Impl:{impl} Test:{test}] {req.text[:50]}...")

    elif command == "coverage":
        coverage = tracker.verify_requirement_coverage()
        print("\n=== Coverage by Category ===")
        for category, stats in coverage.items():
            print(f"\n{category.upper()}:")
            print(f"  Total:              {stats['total']}")
            print(f"  Implementation:     {stats['implementation_coverage']:.1f}%")
            print(f"  Test:               {stats['test_coverage']:.1f}%")
            print(f"  Complete:           {stats['complete_coverage']:.1f}%")

    elif command == "unimplemented":
        unimplemented = tracker.find_unimplemented_requirements()
        print(f"\n{len(unimplemented)} Unimplemented Requirements:")
        for req in unimplemented:
            print(f"  {req.id}: [{req.priority}] {req.text[:60]}...")

    elif command == "untested":
        untested = tracker.find_untested_requirements()
        print(f"\n{len(untested)} Untested Requirements:")
        for req in untested:
            print(f"  {req.id}: [{req.priority}] {req.text[:60]}...")

    elif command == "full-scan":
        # Scan all spec files and components
        spec_files = list(project_root.glob("**/*spec*.md")) + list(project_root.glob("**/requirements*.md"))
        for spec_file in spec_files:
            print(f"Parsing {spec_file}...")
            requirements = tracker.parse_requirements(spec_file)
            tracker.add_requirements(requirements)

        # Scan components directory
        components_dir = project_root / "components"
        if components_dir.exists():
            for component in components_dir.iterdir():
                if component.is_dir():
                    print(f"Scanning {component.name}...")
                    implementations = tracker.scan_implementation(component)
                    tests = tracker.scan_tests(component)
                    tracker.add_implementations(implementations)
                    tracker.add_tests(tests)

        # Show summary
        matrix = tracker.generate_traceability_matrix()
        print(f"\nTotal Requirements: {matrix.total_requirements}")
        print(f"Coverage: {matrix.coverage_percentage:.1f}%")

    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == '__main__':
    main()
