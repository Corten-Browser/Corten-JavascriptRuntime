#!/usr/bin/env python3
"""
Test suite for Defensive Programming Pattern Checker

Tests detection of:
- Null safety violations
- Collection safety issues
- Missing timeouts
- Unsafe type conversions
- Missing bounds checks
- Exception handling problems
- Concurrency issues

Target: 80%+ test coverage
"""

import unittest
import tempfile
import json
from pathlib import Path
from defensive_pattern_checker import (
    DefensivePatternChecker,
    Violation,
    ViolationReport
)


class TestDefensivePatternChecker(unittest.TestCase):
    """Test the main checker class."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.TemporaryDirectory()
        self.component_path = Path(self.temp_dir.name)
        self.checker = DefensivePatternChecker()

    def tearDown(self):
        """Clean up test fixtures."""
        self.temp_dir.cleanup()

    def _create_test_file(self, filename: str, content: str) -> Path:
        """Helper to create a test file."""
        file_path = self.component_path / filename
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.write_text(content)
        return file_path


class TestNullSafety(TestDefensivePatternChecker):
    """Test null safety violation detection."""

    def test_detect_attribute_access_without_none_check(self):
        """Should detect obj.method() without None check."""
        code = """
def process_user(user):
    name = user.get_name()
    return name
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_null_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        self.assertEqual(violations[0].violation_type, 'null_safety')
        self.assertIn('user', violations[0].description)

    def test_no_violation_with_none_check(self):
        """Should NOT detect violation when None check is present."""
        code = """
def process_user(user):
    if user is not None:
        name = user.get_name()
        return name
    return None
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_null_safety(file_path, code)

        # Should not have violations for the checked access
        self.assertEqual(len(violations), 0)

    def test_detect_dict_access_without_key_check(self):
        """Should detect dict[key] without key check."""
        code = """
def get_value(data):
    value = data['key']
    return value
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_null_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('Dictionary access' in v.description for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_dict_get(self):
        """Should NOT detect violation when using dict.get()."""
        code = """
def get_value(data):
    value = data.get('key', None)
    return value
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_null_safety(file_path, code)

        # Should not flag dict.get() usage
        self.assertEqual(len(violations), 0)


class TestCollectionSafety(TestDefensivePatternChecker):
    """Test collection safety violation detection."""

    def test_detect_list_access_without_bounds_check(self):
        """Should detect list[0] without empty check."""
        code = """
def get_first(items):
    first = items[0]
    return first
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_collection_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('List access' in v.description and 'bounds check' in v.description
                    for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_bounds_check(self):
        """Should NOT detect violation when bounds check is present."""
        code = """
def get_first(items):
    if len(items) > 0:
        first = items[0]
        return first
    return None
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_collection_safety(file_path, code)

        # Should not have violations for the checked access
        self.assertEqual(len(violations), 0)

    def test_detect_pop_without_empty_check(self):
        """Should detect .pop() without empty check."""
        code = """
def process_stack(stack):
    item = stack.pop()
    return item
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_collection_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('pop()' in v.description for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_empty_check_before_pop(self):
        """Should NOT detect violation when empty check precedes pop."""
        code = """
def process_stack(stack):
    if stack:
        item = stack.pop()
        return item
    return None
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_collection_safety(file_path, code)

        # Should not have violations for the checked pop
        self.assertEqual(len(violations), 0)


class TestExternalCallSafety(TestDefensivePatternChecker):
    """Test external call safety violation detection."""

    def test_detect_requests_without_timeout(self):
        """Should detect requests.get() without timeout."""
        code = """
import requests

def fetch_data(url):
    response = requests.get(url)
    return response.json()
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_external_call_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('timeout' in v.description.lower() for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_timeout(self):
        """Should NOT detect violation when timeout is present."""
        code = """
import requests

def fetch_data(url):
    response = requests.get(url, timeout=30)
    return response.json()
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_external_call_safety(file_path, code)

        # Should not have violations for calls with timeout
        self.assertEqual(len(violations), 0)

    def test_detect_urllib_without_timeout(self):
        """Should detect urllib.request.urlopen() without timeout."""
        code = """
import urllib.request

def fetch_data(url):
    response = urllib.request.urlopen(url)
    return response.read()
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_external_call_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('urlopen' in v.description for v in violations)
        self.assertTrue(found)


class TestTimeoutPresence(TestDefensivePatternChecker):
    """Test timeout presence on I/O operations."""

    def test_detect_subprocess_without_timeout(self):
        """Should detect subprocess.run() without timeout."""
        code = """
import subprocess

def run_command(cmd):
    result = subprocess.run(cmd)
    return result.returncode
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_timeout_presence(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('subprocess' in v.description.lower() for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_subprocess_timeout(self):
        """Should NOT detect violation when subprocess has timeout."""
        code = """
import subprocess

def run_command(cmd):
    result = subprocess.run(cmd, timeout=30)
    return result.returncode
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_timeout_presence(file_path, code)

        # Should not have violations for calls with timeout
        self.assertEqual(len(violations), 0)


class TestTypeSafety(TestDefensivePatternChecker):
    """Test type safety violation detection."""

    def test_detect_int_conversion_without_try(self):
        """Should detect int() without try-except."""
        code = """
def parse_age(user_input):
    age = int(user_input)
    return age
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_type_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('int()' in v.description for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_try_except(self):
        """Should NOT detect violation when try-except is present."""
        code = """
def parse_age(user_input):
    try:
        age = int(user_input)
        return age
    except ValueError:
        return None
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_type_safety(file_path, code)

        # Should not have violations for protected conversions
        self.assertEqual(len(violations), 0)

    def test_detect_float_conversion_without_try(self):
        """Should detect float() without try-except."""
        code = """
def parse_price(value):
    price = float(value)
    return price
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_type_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('float()' in v.description for v in violations)
        self.assertTrue(found)

    def test_detect_json_loads_without_try(self):
        """Should detect json.loads() without try-except."""
        code = """
import json

def parse_json(text):
    data = json.loads(text)
    return data
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_type_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('json.loads' in v.description for v in violations)
        self.assertTrue(found)


class TestBoundsSafety(TestDefensivePatternChecker):
    """Test bounds safety violation detection."""

    def test_detect_division_without_zero_check(self):
        """Should detect division without zero check."""
        code = """
def calculate(a, b):
    result = a / b
    return result
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_bounds_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('Division' in v.description or 'zero check' in v.description
                    for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_zero_check(self):
        """Should NOT detect violation when zero check is present."""
        code = """
def calculate(a, b):
    if b != 0:
        result = a / b
        return result
    return None
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_bounds_safety(file_path, code)

        # Should not have violations for checked divisions
        self.assertEqual(len(violations), 0)

    def test_detect_modulo_without_zero_check(self):
        """Should detect modulo without zero check."""
        code = """
def get_remainder(a, b):
    remainder = a % b
    return remainder
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_bounds_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('modulo' in v.description.lower() or 'zero check' in v.description
                    for v in violations)
        self.assertTrue(found)


class TestExceptionHandling(TestDefensivePatternChecker):
    """Test exception handling violation detection."""

    def test_detect_bare_except(self):
        """Should detect bare except clause."""
        code = """
def risky_operation():
    try:
        do_something()
    except:
        pass
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_exception_handling(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('bare except' in v.description.lower() for v in violations)
        self.assertTrue(found)

    def test_detect_silent_exception(self):
        """Should detect exception caught but not logged."""
        code = """
def risky_operation():
    try:
        do_something()
    except ValueError:
        pass
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_exception_handling(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('silent' in v.description.lower() for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_proper_handling(self):
        """Should NOT detect violation with proper exception handling."""
        code = """
import logging

def risky_operation():
    try:
        do_something()
    except ValueError as e:
        logging.error(f"Error: {e}")
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_exception_handling(file_path, code)

        # Should not flag proper exception handling
        silent_violations = [v for v in violations if 'silent' in v.description.lower()]
        self.assertEqual(len(silent_violations), 0)


class TestConcurrencySafety(TestDefensivePatternChecker):
    """Test concurrency safety violation detection."""

    def test_detect_shared_state_without_lock(self):
        """Should detect self.* modification without locking."""
        code = """
class Counter:
    def increment(self):
        self.count += 1
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_concurrency_safety(file_path, code)

        self.assertTrue(len(violations) > 0)
        found = any('shared state' in v.description.lower() for v in violations)
        self.assertTrue(found)

    def test_no_violation_with_lock(self):
        """Should NOT detect violation when lock is used."""
        code = """
class Counter:
    def increment(self):
        with self._lock:
            self.count += 1
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_concurrency_safety(file_path, code)

        # Should not have violations for locked access
        self.assertEqual(len(violations), 0)


class TestComponentChecking(TestDefensivePatternChecker):
    """Test component-level checking."""

    def test_check_component_with_multiple_files(self):
        """Should check all Python files in component."""
        # Create multiple files with violations
        self._create_test_file("module1.py", """
def func1(obj):
    return obj.method()
""")
        self._create_test_file("module2.py", """
def func2(items):
    return items[0]
""")

        report = self.checker.check_component(self.component_path)

        self.assertIsInstance(report, ViolationReport)
        self.assertTrue(report.total_violations > 0)
        self.assertGreater(len(report.violations), 0)

    def test_skip_test_files(self):
        """Should skip files starting with test_."""
        self._create_test_file("test_module.py", """
def test_something():
    obj.method()  # This should be skipped
""")

        report = self.checker.check_component(self.component_path)

        # Should have 0 violations since test file is skipped
        self.assertEqual(report.total_violations, 0)

    def test_violations_by_type_counting(self):
        """Should correctly count violations by type."""
        self._create_test_file("module.py", """
def func(obj, items):
    name = obj.get_name()  # null_safety
    first = items[0]  # collection_safety
    return name, first
""")

        report = self.checker.check_component(self.component_path)

        self.assertTrue(len(report.violations_by_type) > 0)
        self.assertTrue(report.total_violations > 0)

    def test_critical_violations_counting(self):
        """Should correctly count critical violations."""
        self._create_test_file("module.py", """
import requests

def fetch(url):
    response = requests.get(url)  # critical
    return response.json()
""")

        report = self.checker.check_component(self.component_path)

        # Should have at least one critical violation
        self.assertTrue(report.critical_violations > 0)


class TestReportFormatting(TestDefensivePatternChecker):
    """Test report formatting."""

    def test_format_report_with_violations(self):
        """Should format report with violations."""
        self._create_test_file("module.py", """
def func(obj):
    return obj.method()
""")

        report = self.checker.check_component(self.component_path)
        formatted = self.checker.format_report(report)

        self.assertIsInstance(formatted, str)
        self.assertIn('Defensive Programming Pattern Check Report', formatted)
        self.assertIn('Total Violations', formatted)

    def test_format_report_without_violations(self):
        """Should format report when no violations found."""
        self._create_test_file("module.py", """
def safe_func(obj):
    if obj is not None:
        return obj.method()
    return None
""")

        report = self.checker.check_component(self.component_path)
        formatted = self.checker.format_report(report)

        self.assertIn('No violations found', formatted)


class TestHelperMethods(TestDefensivePatternChecker):
    """Test helper methods."""

    def test_get_attribute_base(self):
        """Should extract base variable from attribute access."""
        code = "user.get_name()"
        tree = __import__('ast').parse(code)
        node = list(__import__('ast').walk(tree))[0]

        # Find Attribute node
        for n in __import__('ast').walk(tree):
            if isinstance(n, __import__('ast').Attribute):
                result = self.checker._get_attribute_base(n)
                self.assertEqual(result, 'user')
                break

    def test_get_subscript_base(self):
        """Should extract base variable from subscript."""
        code = "data['key']"
        tree = __import__('ast').parse(code)

        # Find Subscript node
        for n in __import__('ast').walk(tree):
            if isinstance(n, __import__('ast').Subscript):
                result = self.checker._get_subscript_base(n)
                self.assertEqual(result, 'data')
                break

    def test_is_in_try_block(self):
        """Should detect if line is in try block."""
        code = """
try:
    x = int(value)
except ValueError:
    x = 0
"""
        lines = code.split('\n')
        # Line 2 (index 2) is inside try block
        result = self.checker._is_in_try_block(lines, 2)
        self.assertTrue(result)

    def test_has_none_check_before(self):
        """Should detect None check before line."""
        code = """
if user is not None:
    name = user.get_name()
"""
        lines = code.split('\n')
        # Line 2 (index 2) has None check before it
        result = self.checker._has_none_check_before(lines, 2, 'user')
        self.assertTrue(result)


class TestEdgeCases(TestDefensivePatternChecker):
    """Test edge cases and error handling."""

    def test_empty_file(self):
        """Should handle empty files gracefully."""
        file_path = self._create_test_file("empty.py", "")
        violations = self.checker.check_file(file_path)
        self.assertEqual(len(violations), 0)

    def test_syntax_error_file(self):
        """Should handle syntax errors gracefully."""
        file_path = self._create_test_file("broken.py", "def func(\n")
        violations = self.checker.check_file(file_path)
        # Should return empty list rather than crash
        self.assertEqual(len(violations), 0)

    def test_nonexistent_component(self):
        """Should raise error for nonexistent component."""
        with self.assertRaises(ValueError):
            self.checker.check_component(Path("/nonexistent/path"))

    def test_unicode_content(self):
        """Should handle Unicode content correctly."""
        code = """
def func(obj):
    # Comment with émojis 🎉
    return obj.method()
"""
        file_path = self._create_test_file("unicode.py", code)
        violations = self.checker.check_file(file_path)
        # Should process without errors
        self.assertIsInstance(violations, list)


class TestFixSuggestions(TestDefensivePatternChecker):
    """Test fix suggestion generation."""

    def test_fix_suggestion_for_none_check(self):
        """Should provide correct fix suggestion for None check."""
        code = """
def func(obj):
    return obj.method()
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_null_safety(file_path, code)

        if violations:
            fix = violations[0].fix_suggestion
            self.assertIn('is not None', fix)

    def test_fix_suggestion_for_dict_access(self):
        """Should provide correct fix suggestion for dict access."""
        code = """
def func(data):
    return data['key']
"""
        file_path = self._create_test_file("test.py", code)
        violations = self.checker.check_null_safety(file_path, code)

        found = False
        for v in violations:
            if 'Dictionary access' in v.description:
                self.assertIn('.get(', v.fix_suggestion)
                found = True
                break
        self.assertTrue(found)


def run_tests():
    """Run all tests."""
    loader = unittest.TestLoader()
    suite = loader.loadTestsFromModule(__import__(__name__))
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    return result.wasSuccessful()


if __name__ == '__main__':
    import sys
    success = run_tests()
    sys.exit(0 if success else 1)
