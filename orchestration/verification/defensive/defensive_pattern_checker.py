#!/usr/bin/env python3
"""
Defensive Programming Pattern Checker

Analyzes code for common defensive programming violations.
Part of v0.4.0 quality enhancement system.

This checker uses AST parsing combined with regex patterns to detect:
- Null safety violations
- Unvalidated collection access
- Missing timeouts on I/O operations
- Unsafe type conversions
- Missing bounds checks
- Inadequate exception handling
- Concurrency issues
"""

import re
import ast
import json
from pathlib import Path
from typing import List, Dict, Optional, Set, Tuple
from dataclasses import dataclass, asdict
from collections import defaultdict


@dataclass
class Violation:
    """A defensive programming violation."""
    file_path: str
    line_number: int
    violation_type: str
    description: str
    code_snippet: str
    fix_suggestion: str
    severity: str  # "critical", "warning", "info"

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)


@dataclass
class ViolationReport:
    """Complete violation report for a component."""
    component_path: str
    total_violations: int
    critical_violations: int
    violations_by_type: Dict[str, int]
    violations: List[Violation]

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            'component_path': self.component_path,
            'total_violations': self.total_violations,
            'critical_violations': self.critical_violations,
            'violations_by_type': self.violations_by_type,
            'violations': [v.to_dict() for v in self.violations]
        }


class DefensivePatternChecker:
    """Main checker class."""

    def __init__(self, patterns_file: Optional[Path] = None):
        """Initialize with patterns from JSON config."""
        # Default to config/ directory which contains defensive_patterns.json
        self.patterns_file = patterns_file or Path(__file__).parent.parent.parent / "config" / "defensive_patterns.json"
        self.patterns = self._load_patterns()
        self.safe_patterns = self.patterns.get('safe_patterns', {}).get('patterns', [])

    def _load_patterns(self) -> dict:
        """Load violation patterns from JSON."""
        if not self.patterns_file.exists():
            raise FileNotFoundError(f"Patterns file not found: {self.patterns_file}")

        with open(self.patterns_file, 'r') as f:
            return json.load(f)

    def check_component(self, component_path: Path) -> ViolationReport:
        """Check entire component for violations."""
        if not component_path.exists():
            raise ValueError(f"Component path does not exist: {component_path}")

        all_violations = []

        # Find all Python files
        python_files = list(component_path.rglob("*.py"))

        for py_file in python_files:
            # Skip test files and __pycache__
            if '__pycache__' in str(py_file) or py_file.name.startswith('test_'):
                continue

            try:
                violations = self.check_file(py_file)
                all_violations.extend(violations)
            except Exception as e:
                # Log error but continue checking other files
                print(f"Warning: Error checking {py_file}: {e}")

        # Calculate statistics
        violations_by_type = defaultdict(int)
        critical_count = 0

        for v in all_violations:
            violations_by_type[v.violation_type] += 1
            if v.severity == 'critical':
                critical_count += 1

        return ViolationReport(
            component_path=str(component_path),
            total_violations=len(all_violations),
            critical_violations=critical_count,
            violations_by_type=dict(violations_by_type),
            violations=all_violations
        )

    def check_file(self, source_file: Path) -> List[Violation]:
        """Check a single file for all violation types."""
        violations = []

        # Read file content
        try:
            with open(source_file, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            return violations

        # Skip empty files
        if not content.strip():
            return violations

        # Run all checks
        violations.extend(self.check_null_safety(source_file, content))
        violations.extend(self.check_collection_safety(source_file, content))
        violations.extend(self.check_external_call_safety(source_file, content))
        violations.extend(self.check_timeout_presence(source_file, content))
        violations.extend(self.check_type_safety(source_file, content))
        violations.extend(self.check_bounds_safety(source_file, content))
        violations.extend(self.check_exception_handling(source_file, content))
        violations.extend(self.check_concurrency_safety(source_file, content))

        return violations

    def check_null_safety(self, source_file: Path, content: str) -> List[Violation]:
        """
        Detect unchecked None/null references:
        - obj.method() without None check
        - dict[key] without key check
        - list[0] without empty check
        """
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for attribute access without None check
        for node in ast.walk(tree):
            if isinstance(node, ast.Attribute):
                # Skip safe method calls like .get()
                if node.attr in ['get', 'keys', 'values', 'items']:
                    continue

                # Check if preceded by None check
                line_num = node.lineno
                if line_num <= len(lines):
                    line = lines[line_num - 1]

                    # Check if this is in a safe context
                    if self._is_in_safe_context(lines, line_num - 1, 'none_check'):
                        continue

                    # Get the variable name
                    var_name = self._get_attribute_base(node)
                    if var_name and not self._has_none_check_before(lines, line_num - 1, var_name):
                        violations.append(Violation(
                            file_path=str(source_file),
                            line_number=line_num,
                            violation_type='null_safety',
                            description=f"Attribute access on '{var_name}' without None check",
                            code_snippet=line.strip(),
                            fix_suggestion=f"if {var_name} is not None:\n    {line.strip()}",
                            severity='critical'
                        ))

        # Check for dictionary subscript access
        for node in ast.walk(tree):
            if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Constant):
                line_num = node.lineno
                if line_num <= len(lines):
                    line = lines[line_num - 1]

                    # Check if using dict.get() or in safe context
                    if '.get(' in line or self._is_in_safe_context(lines, line_num - 1, 'dict_access'):
                        continue

                    # Check if it's likely a dictionary access
                    dict_name = self._get_subscript_base(node)
                    key = self._get_subscript_key(node)

                    if dict_name and key and not self._has_key_check_before(lines, line_num - 1, dict_name, key):
                        violations.append(Violation(
                            file_path=str(source_file),
                            line_number=line_num,
                            violation_type='null_safety',
                            description=f"Dictionary access '{dict_name}[{key}]' without key check",
                            code_snippet=line.strip(),
                            fix_suggestion=f"{dict_name}.get({key}, None)  # or provide default",
                            severity='critical'
                        ))

        return violations

    def check_collection_safety(self, source_file: Path, content: str) -> List[Violation]:
        """
        Detect unvalidated collection access:
        - list[index] without bounds check
        - dict[key] without key existence check
        - iteration without empty check
        """
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for list index access
        for node in ast.walk(tree):
            if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Constant):
                if isinstance(node.slice.value, int):
                    line_num = node.lineno
                    if line_num <= len(lines):
                        line = lines[line_num - 1]

                        list_name = self._get_subscript_base(node)
                        index = node.slice.value

                        if list_name and not self._has_bounds_check_before(lines, line_num - 1, list_name, index):
                            violations.append(Violation(
                                file_path=str(source_file),
                                line_number=line_num,
                                violation_type='collection_safety',
                                description=f"List access '{list_name}[{index}]' without bounds check",
                                code_snippet=line.strip(),
                                fix_suggestion=f"if len({list_name}) > {index}:\n    {line.strip()}",
                                severity='warning'
                            ))

        # Check for .pop() without empty check
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                if isinstance(node.func, ast.Attribute) and node.func.attr == 'pop':
                    line_num = node.lineno
                    if line_num <= len(lines):
                        line = lines[line_num - 1]

                        list_name = self._get_attribute_base(node.func)
                        if list_name and not self._has_empty_check_before(lines, line_num - 1, list_name):
                            violations.append(Violation(
                                file_path=str(source_file),
                                line_number=line_num,
                                violation_type='collection_safety',
                                description=f"Call to '{list_name}.pop()' without empty check",
                                code_snippet=line.strip(),
                                fix_suggestion=f"if {list_name}:\n    {line.strip()}",
                                severity='warning'
                            ))

        return violations

    def check_external_call_safety(self, source_file: Path, content: str) -> List[Violation]:
        """
        Detect unsafe external calls:
        - API calls without timeout
        - Database calls without timeout
        - Network calls without error handling
        """
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for requests library calls without timeout
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                if isinstance(node.func, ast.Attribute):
                    # Check for requests.get/post/etc
                    if isinstance(node.func.value, ast.Name) and node.func.value.id == 'requests':
                        if node.func.attr in ['get', 'post', 'put', 'delete', 'patch', 'request']:
                            # Check if timeout is in keywords
                            has_timeout = any(kw.arg == 'timeout' for kw in node.keywords)

                            if not has_timeout:
                                line_num = node.lineno
                                if line_num <= len(lines):
                                    line = lines[line_num - 1]
                                    violations.append(Violation(
                                        file_path=str(source_file),
                                        line_number=line_num,
                                        violation_type='external_call_safety',
                                        description=f"HTTP request without timeout parameter",
                                        code_snippet=line.strip(),
                                        fix_suggestion=f"{line.strip()[:-1]}, timeout=30)",
                                        severity='critical'
                                    ))

        # Check for urllib.request.urlopen without timeout
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                func_name = self._get_full_function_name(node.func)
                if 'urllib.request.urlopen' in func_name or func_name.endswith('urlopen'):
                    has_timeout = any(kw.arg == 'timeout' for kw in node.keywords)

                    if not has_timeout and len(node.args) < 2:  # timeout is 2nd positional arg
                        line_num = node.lineno
                        if line_num <= len(lines):
                            line = lines[line_num - 1]
                            violations.append(Violation(
                                file_path=str(source_file),
                                line_number=line_num,
                                violation_type='external_call_safety',
                                description="urllib.request.urlopen without timeout",
                                code_snippet=line.strip(),
                                fix_suggestion=f"{line.strip()[:-1]}, timeout=30)",
                                severity='critical'
                            ))

        return violations

    def check_timeout_presence(self, source_file: Path, content: str) -> List[Violation]:
        """Verify timeout parameters on I/O operations."""
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for subprocess calls without timeout
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                func_name = self._get_full_function_name(node.func)

                if 'subprocess.run' in func_name or 'subprocess.call' in func_name or 'subprocess.check_output' in func_name:
                    has_timeout = any(kw.arg == 'timeout' for kw in node.keywords)

                    if not has_timeout:
                        line_num = node.lineno
                        if line_num <= len(lines):
                            line = lines[line_num - 1]
                            violations.append(Violation(
                                file_path=str(source_file),
                                line_number=line_num,
                                violation_type='timeout_presence',
                                description="Subprocess call without timeout parameter",
                                code_snippet=line.strip(),
                                fix_suggestion=f"{line.strip()[:-1]}, timeout=30)",
                                severity='critical'
                            ))

        return violations

    def check_type_safety(self, source_file: Path, content: str) -> List[Violation]:
        """
        Detect unsafe type conversions:
        - int(str) without validation
        - Unchecked type casts
        """
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for int() and float() conversions without try-except
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                if isinstance(node.func, ast.Name) and node.func.id in ['int', 'float']:
                    # Check if inside try-except block
                    line_num = node.lineno
                    if not self._is_in_try_block(lines, line_num - 1):
                        if line_num <= len(lines):
                            line = lines[line_num - 1]
                            conv_type = node.func.id
                            violations.append(Violation(
                                file_path=str(source_file),
                                line_number=line_num,
                                violation_type='type_safety',
                                description=f"{conv_type}() conversion without exception handling",
                                code_snippet=line.strip(),
                                fix_suggestion=f"try:\n    {line.strip()}\nexcept ValueError:\n    # Handle invalid input",
                                severity='warning'
                            ))

        # Check for json.loads without try-except
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                func_name = self._get_full_function_name(node.func)
                if 'json.loads' in func_name:
                    line_num = node.lineno
                    if not self._is_in_try_block(lines, line_num - 1):
                        if line_num <= len(lines):
                            line = lines[line_num - 1]
                            violations.append(Violation(
                                file_path=str(source_file),
                                line_number=line_num,
                                violation_type='type_safety',
                                description="json.loads() without exception handling",
                                code_snippet=line.strip(),
                                fix_suggestion=f"try:\n    {line.strip()}\nexcept json.JSONDecodeError:\n    # Handle invalid JSON",
                                severity='warning'
                            ))

        return violations

    def check_bounds_safety(self, source_file: Path, content: str) -> List[Violation]:
        """Detect missing bounds checking on numeric operations."""
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for division operations without zero check
        for node in ast.walk(tree):
            if isinstance(node, ast.BinOp):
                if isinstance(node.op, (ast.Div, ast.FloorDiv, ast.Mod)):
                    line_num = node.lineno
                    if line_num <= len(lines):
                        line = lines[line_num - 1]

                        # Get divisor variable name
                        divisor = self._get_node_name(node.right)

                        if divisor and not self._has_zero_check_before(lines, line_num - 1, divisor):
                            op_symbol = '/' if isinstance(node.op, (ast.Div, ast.FloorDiv)) else '%'
                            violations.append(Violation(
                                file_path=str(source_file),
                                line_number=line_num,
                                violation_type='bounds_safety',
                                description=f"Division/modulo operation without zero check on '{divisor}'",
                                code_snippet=line.strip(),
                                fix_suggestion=f"if {divisor} != 0:\n    {line.strip()}",
                                severity='warning'
                            ))

        return violations

    def check_exception_handling(self, source_file: Path, content: str) -> List[Violation]:
        """Verify appropriate exception handling."""
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for bare except clauses
        for node in ast.walk(tree):
            if isinstance(node, ast.ExceptHandler):
                if node.type is None:  # Bare except
                    line_num = node.lineno
                    if line_num <= len(lines):
                        line = lines[line_num - 1]
                        violations.append(Violation(
                            file_path=str(source_file),
                            line_number=line_num,
                            violation_type='exception_handling',
                            description="Bare except clause catches all exceptions",
                            code_snippet=line.strip(),
                            fix_suggestion="except Exception as e:\n    logger.error(f'Error: {e}')",
                            severity='critical'
                        ))

                # Check for silent exceptions (except with only pass)
                if len(node.body) == 1 and isinstance(node.body[0], ast.Pass):
                    line_num = node.lineno
                    if line_num <= len(lines):
                        line = lines[line_num - 1]
                        exc_type = self._get_exception_type(node)
                        violations.append(Violation(
                            file_path=str(source_file),
                            line_number=line_num,
                            violation_type='exception_handling',
                            description="Exception silently caught and ignored",
                            code_snippet=line.strip(),
                            fix_suggestion=f"except {exc_type} as e:\n    logger.error(f'Error: {{e}}')",
                            severity='warning'
                        ))

        return violations

    def check_concurrency_safety(self, source_file: Path, content: str) -> List[Violation]:
        """Detect potential race conditions."""
        violations = []

        try:
            tree = ast.parse(content)
        except SyntaxError:
            return violations

        lines = content.split('\n')

        # Check for modifications to self.* attributes (potential shared state)
        for node in ast.walk(tree):
            # Skip nodes without line numbers (like Module)
            if not hasattr(node, 'lineno'):
                continue

            target_attr = None
            line_num = node.lineno

            # Check both regular assignment and augmented assignment (+=, -=, etc.)
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Attribute):
                        if isinstance(target.value, ast.Name) and target.value.id == 'self':
                            target_attr = target.attr
                            break
            elif isinstance(node, ast.AugAssign):
                if isinstance(node.target, ast.Attribute):
                    if isinstance(node.target.value, ast.Name) and node.target.value.id == 'self':
                        target_attr = node.target.attr

            if target_attr:
                # Check if inside a with statement (lock)
                if not self._is_in_with_lock(lines, line_num - 1):
                    if line_num <= len(lines):
                        line = lines[line_num - 1]
                        violations.append(Violation(
                            file_path=str(source_file),
                            line_number=line_num,
                            violation_type='concurrency_safety',
                            description=f"Potential shared state modification 'self.{target_attr}' without locking",
                            code_snippet=line.strip(),
                            fix_suggestion=f"with self._lock:\n    {line.strip()}",
                            severity='warning'
                        ))

        return violations

    # Helper methods

    def _get_attribute_base(self, node: ast.Attribute) -> Optional[str]:
        """Get the base variable name from an attribute access."""
        if isinstance(node.value, ast.Name):
            return node.value.id
        return None

    def _get_subscript_base(self, node: ast.Subscript) -> Optional[str]:
        """Get the base variable name from a subscript."""
        if isinstance(node.value, ast.Name):
            return node.value.id
        return None

    def _get_subscript_key(self, node: ast.Subscript) -> Optional[str]:
        """Get the key from a subscript."""
        if isinstance(node.slice, ast.Constant):
            return repr(node.slice.value)
        return None

    def _get_full_function_name(self, node) -> str:
        """Get the full qualified function name."""
        parts = []
        current = node

        while isinstance(current, ast.Attribute):
            parts.append(current.attr)
            current = current.value

        if isinstance(current, ast.Name):
            parts.append(current.id)

        return '.'.join(reversed(parts))

    def _get_node_name(self, node) -> Optional[str]:
        """Get variable name from AST node."""
        if isinstance(node, ast.Name):
            return node.id
        return None

    def _get_exception_type(self, node: ast.ExceptHandler) -> str:
        """Get exception type name."""
        if node.type:
            if isinstance(node.type, ast.Name):
                return node.type.id
            return "Exception"
        return "Exception"

    def _is_in_safe_context(self, lines: List[str], line_idx: int, check_type: str) -> bool:
        """Check if line is in a safe context (e.g., after validation)."""
        # Look at previous few lines for safety checks
        start = max(0, line_idx - 5)
        context = '\n'.join(lines[start:line_idx])

        if check_type == 'none_check':
            return 'is not None' in context
        elif check_type == 'dict_access':
            return '.get(' in context or ' in ' in context

        return False

    def _has_none_check_before(self, lines: List[str], line_idx: int, var_name: str) -> bool:
        """Check if there's a None check for variable before this line."""
        start = max(0, line_idx - 5)
        context = '\n'.join(lines[start:line_idx])

        # Look for patterns like: if var is not None
        pattern = rf'\b{re.escape(var_name)}\s+is\s+not\s+None'
        return bool(re.search(pattern, context))

    def _has_key_check_before(self, lines: List[str], line_idx: int, dict_name: str, key: str) -> bool:
        """Check if there's a key existence check before this line."""
        start = max(0, line_idx - 5)
        context = '\n'.join(lines[start:line_idx])

        # Look for patterns like: if key in dict
        pattern = rf'{re.escape(key)}\s+in\s+{re.escape(dict_name)}'
        return bool(re.search(pattern, context))

    def _has_bounds_check_before(self, lines: List[str], line_idx: int, list_name: str, index: int) -> bool:
        """Check if there's a bounds check before this line."""
        start = max(0, line_idx - 5)
        context = '\n'.join(lines[start:line_idx])

        # Look for patterns like: if len(list) > index or if len(list) >= index+1
        pattern1 = rf'len\({re.escape(list_name)}\)\s*>\s*{index}'
        pattern2 = rf'len\({re.escape(list_name)}\)\s*>=\s*{index + 1}'
        return bool(re.search(pattern1, context) or re.search(pattern2, context))

    def _has_empty_check_before(self, lines: List[str], line_idx: int, collection_name: str) -> bool:
        """Check if there's an empty check before this line."""
        start = max(0, line_idx - 5)
        context = '\n'.join(lines[start:line_idx])

        # Look for patterns like: if collection or if len(collection)
        pattern = rf'if\s+{re.escape(collection_name)}|if\s+len\({re.escape(collection_name)}\)'
        return bool(re.search(pattern, context))

    def _has_zero_check_before(self, lines: List[str], line_idx: int, var_name: str) -> bool:
        """Check if there's a zero check before this line."""
        start = max(0, line_idx - 5)
        context = '\n'.join(lines[start:line_idx])

        # Look for patterns like: if var != 0 or if var
        pattern = rf'{re.escape(var_name)}\s*!=\s*0|if\s+{re.escape(var_name)}(?!\w)'
        return bool(re.search(pattern, context))

    def _is_in_try_block(self, lines: List[str], line_idx: int) -> bool:
        """Check if line is inside a try block."""
        # Look backwards for try: and check indentation
        current_indent = len(lines[line_idx]) - len(lines[line_idx].lstrip())

        for i in range(line_idx - 1, max(0, line_idx - 20), -1):
            line = lines[i].strip()
            line_indent = len(lines[i]) - len(lines[i].lstrip())

            if line.startswith('try:') and line_indent < current_indent:
                # Check if we're before the except
                for j in range(i + 1, min(len(lines), line_idx + 5)):
                    if lines[j].strip().startswith('except'):
                        except_indent = len(lines[j]) - len(lines[j].lstrip())
                        if except_indent == line_indent:
                            return True
                return True

        return False

    def _is_in_with_lock(self, lines: List[str], line_idx: int) -> bool:
        """Check if line is inside a with lock statement."""
        current_indent = len(lines[line_idx]) - len(lines[line_idx].lstrip())

        for i in range(line_idx - 1, max(0, line_idx - 10), -1):
            line = lines[i].strip()
            line_indent = len(lines[i]) - len(lines[i].lstrip())

            if 'with' in line and 'lock' in line.lower() and line_indent < current_indent:
                return True

        return False

    def format_report(self, report: ViolationReport) -> str:
        """Format report for display."""
        lines = []
        lines.append("=" * 80)
        lines.append(f"Defensive Programming Pattern Check Report")
        lines.append(f"Component: {report.component_path}")
        lines.append("=" * 80)
        lines.append("")
        lines.append(f"Total Violations: {report.total_violations}")
        lines.append(f"Critical Violations: {report.critical_violations}")
        lines.append("")

        if report.violations_by_type:
            lines.append("Violations by Type:")
            for vtype, count in sorted(report.violations_by_type.items()):
                lines.append(f"  {vtype}: {count}")
            lines.append("")

        if report.violations:
            lines.append("Detailed Violations:")
            lines.append("-" * 80)

            # Group by severity
            by_severity = defaultdict(list)
            for v in report.violations:
                by_severity[v.severity].append(v)

            for severity in ['critical', 'warning', 'info']:
                if severity in by_severity:
                    lines.append("")
                    lines.append(f"{severity.upper()} ({len(by_severity[severity])} issues):")
                    lines.append("")

                    for v in by_severity[severity]:
                        lines.append(f"  File: {v.file_path}:{v.line_number}")
                        lines.append(f"  Type: {v.violation_type}")
                        lines.append(f"  Issue: {v.description}")
                        lines.append(f"  Code: {v.code_snippet}")
                        lines.append(f"  Fix: {v.fix_suggestion}")
                        lines.append("")
        else:
            lines.append("No violations found!")

        lines.append("=" * 80)

        return '\n'.join(lines)


def main():
    """CLI interface."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: defensive_pattern_checker.py <component_path> [--json]")
        print("\nOptions:")
        print("  --json    Output report in JSON format")
        sys.exit(1)

    component_path = Path(sys.argv[1])
    output_json = '--json' in sys.argv

    if not component_path.exists():
        print(f"Error: Component path does not exist: {component_path}", file=sys.stderr)
        sys.exit(1)

    try:
        checker = DefensivePatternChecker()
        report = checker.check_component(component_path)

        if output_json:
            print(json.dumps(report.to_dict(), indent=2))
        else:
            print(checker.format_report(report))

        # Exit with error code if critical violations found
        sys.exit(1 if report.critical_violations > 0 else 0)

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()
