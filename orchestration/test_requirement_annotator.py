#!/usr/bin/env python3
"""
Tests for Requirement Annotator

Comprehensive test coverage for automatic requirement annotation.
"""

import unittest
from pathlib import Path
import tempfile
import shutil
from dataclasses import dataclass

from orchestration.requirement_annotator import RequirementAnnotator, Annotation


@dataclass
class MockRequirement:
    """Mock requirement for testing."""
    id: str
    text: str


@dataclass
class MockRequirementTrace:
    """Mock requirement trace for testing."""
    requirement: MockRequirement


class TestRequirementAnnotator(unittest.TestCase):
    """Test RequirementAnnotator class."""

    def setUp(self):
        """Set up test fixtures."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.annotator = RequirementAnnotator(self.test_dir)

    def tearDown(self):
        """Clean up test directory."""
        if self.test_dir.exists():
            shutil.rmtree(self.test_dir)

    def test_annotate_function_success(self):
        """Test successful function annotation."""
        # Create test file
        test_file = self.test_dir / "test.py"
        test_file.write_text("""def calculate_total(items):
    return sum(items)
""")

        # Annotate function
        success = self.annotator.annotate_function(test_file, "calculate_total", "REQ-001")

        # Verify
        self.assertTrue(success)
        content = test_file.read_text()
        self.assertIn("# @implements: REQ-001", content)
        self.assertEqual(len(self.annotator.annotations_made), 1)

        annotation = self.annotator.annotations_made[0]
        self.assertEqual(annotation.requirement_id, "REQ-001")
        self.assertEqual(annotation.annotation_type, "implements")
        self.assertEqual(annotation.context, "calculate_total")

    def test_annotate_function_already_annotated(self):
        """Test annotating already annotated function."""
        # Create test file with existing annotation
        test_file = self.test_dir / "test.py"
        test_file.write_text("""# @implements: REQ-001
def calculate_total(items):
    return sum(items)
""")

        # Try to annotate again
        success = self.annotator.annotate_function(test_file, "calculate_total", "REQ-002")

        # Verify - should fail (already annotated)
        self.assertFalse(success)
        self.assertEqual(len(self.annotator.annotations_made), 0)

    def test_annotate_function_not_found(self):
        """Test annotating non-existent function."""
        # Create test file
        test_file = self.test_dir / "test.py"
        test_file.write_text("""def calculate_total(items):
    return sum(items)
""")

        # Try to annotate non-existent function
        success = self.annotator.annotate_function(test_file, "nonexistent", "REQ-001")

        # Verify - should fail
        self.assertFalse(success)
        self.assertEqual(len(self.annotator.annotations_made), 0)

    def test_annotate_function_with_indentation(self):
        """Test annotating function inside class (with indentation)."""
        # Create test file
        test_file = self.test_dir / "test.py"
        test_file.write_text("""class Calculator:
    def calculate_total(self, items):
        return sum(items)
""")

        # Annotate function
        success = self.annotator.annotate_function(test_file, "calculate_total", "REQ-001")

        # Verify
        self.assertTrue(success)
        content = test_file.read_text()
        self.assertIn("    # @implements: REQ-001", content)
        # Check indentation is preserved
        lines = content.split('\n')
        annotation_line = [l for l in lines if '@implements:' in l][0]
        self.assertTrue(annotation_line.startswith("    "))

    def test_annotate_test_success(self):
        """Test successful test annotation."""
        # Create test file
        test_file = self.test_dir / "test_calc.py"
        test_file.write_text("""def test_calculate_total():
    assert calculate_total([1, 2, 3]) == 6
""")

        # Annotate test
        success = self.annotator.annotate_test(test_file, "test_calculate_total", "REQ-001")

        # Verify
        self.assertTrue(success)
        content = test_file.read_text()
        self.assertIn("# @validates: REQ-001", content)
        self.assertEqual(len(self.annotator.annotations_made), 1)

        annotation = self.annotator.annotations_made[0]
        self.assertEqual(annotation.requirement_id, "REQ-001")
        self.assertEqual(annotation.annotation_type, "validates")
        self.assertEqual(annotation.context, "test_calculate_total")

    def test_annotate_test_already_annotated(self):
        """Test annotating already annotated test."""
        # Create test file with existing annotation
        test_file = self.test_dir / "test_calc.py"
        test_file.write_text("""# @validates: REQ-001
def test_calculate_total():
    assert calculate_total([1, 2, 3]) == 6
""")

        # Try to annotate again
        success = self.annotator.annotate_test(test_file, "test_calculate_total", "REQ-002")

        # Verify - should fail
        self.assertFalse(success)
        self.assertEqual(len(self.annotator.annotations_made), 0)

    def test_remove_annotations_success(self):
        """Test removing annotations from file."""
        # Create test file with annotations
        test_file = self.test_dir / "test.py"
        test_file.write_text("""# @implements: REQ-001
def calculate_total(items):
    return sum(items)

# @implements: REQ-002
def calculate_average(items):
    return sum(items) / len(items)
""")

        # Remove annotations
        removed = self.annotator.remove_annotations(test_file)

        # Verify
        self.assertEqual(removed, 2)
        content = test_file.read_text()
        self.assertNotIn("@implements:", content)
        self.assertNotIn("@validates:", content)

    def test_remove_annotations_no_annotations(self):
        """Test removing annotations from file with no annotations."""
        # Create test file without annotations
        test_file = self.test_dir / "test.py"
        original_content = """def calculate_total(items):
    return sum(items)
"""
        test_file.write_text(original_content)

        # Remove annotations
        removed = self.annotator.remove_annotations(test_file)

        # Verify
        self.assertEqual(removed, 0)
        self.assertEqual(test_file.read_text(), original_content)

    def test_extract_key_terms(self):
        """Test extraction of key terms from requirement text."""
        # Test with typical requirement text
        text = "The system must validate user credentials and authenticate requests"
        terms = self.annotator._extract_key_terms(text)

        # Verify
        self.assertIn("system", terms)
        self.assertIn("validate", terms)
        self.assertIn("user", terms)
        self.assertIn("credentials", terms)
        self.assertIn("authenticate", terms)
        self.assertIn("requests", terms)

        # Verify stop words are removed
        self.assertNotIn("the", terms)
        self.assertNotIn("and", terms)
        self.assertNotIn("must", terms)

        # Verify short words are removed
        for term in terms:
            self.assertGreater(len(term), 3)

    def test_function_matches_requirement_by_name(self):
        """Test matching function to requirement by name."""
        import ast

        # Create function node
        code = """
def validate_user_credentials(username, password):
    '''Validate user credentials.'''
    pass
"""
        tree = ast.parse(code)
        func = tree.body[0]

        # Create requirement
        req = MockRequirement("REQ-001", "System must validate user credentials")

        # Test matching
        matches = self.annotator._function_matches_requirement(func, req)

        # Verify - should match (validate, user, credentials appear in both)
        self.assertTrue(matches)

    def test_function_matches_requirement_by_docstring(self):
        """Test matching function to requirement by docstring."""
        import ast

        # Create function node with relevant docstring
        code = """
def check_auth(username, password):
    '''
    Validate user credentials and authenticate the request.
    '''
    pass
"""
        tree = ast.parse(code)
        func = tree.body[0]

        # Create requirement
        req = MockRequirement("REQ-001", "System must validate user credentials")

        # Test matching
        matches = self.annotator._function_matches_requirement(func, req)

        # Verify - should match (validate, user, credentials in docstring)
        self.assertTrue(matches)

    def test_function_does_not_match_requirement(self):
        """Test function that doesn't match requirement."""
        import ast

        # Create function node
        code = """
def calculate_total(items):
    '''Calculate sum of items.'''
    pass
"""
        tree = ast.parse(code)
        func = tree.body[0]

        # Create requirement
        req = MockRequirement("REQ-001", "System must validate user credentials")

        # Test matching
        matches = self.annotator._function_matches_requirement(func, req)

        # Verify - should not match
        self.assertFalse(matches)

    def test_get_annotations_for_file(self):
        """Test retrieving annotations from a file."""
        # Create test file with annotations
        test_file = self.test_dir / "test.py"
        test_file.write_text("""# @implements: REQ-001
def calculate_total(items):
    return sum(items)

# @validates: REQ-001
def test_calculate_total():
    assert calculate_total([1, 2, 3]) == 6

# @implements: REQ-002
def calculate_average(items):
    return sum(items) / len(items)
""")

        # Get annotations
        annotations = self.annotator.get_annotations_for_file(test_file)

        # Verify
        self.assertEqual(len(annotations), 3)

        # Check first annotation
        self.assertEqual(annotations[0][0], 1)  # line number
        self.assertEqual(annotations[0][1], 'implements')
        self.assertEqual(annotations[0][2], 'REQ-001')

        # Check second annotation
        self.assertEqual(annotations[1][0], 5)
        self.assertEqual(annotations[1][1], 'validates')
        self.assertEqual(annotations[1][2], 'REQ-001')

        # Check third annotation
        self.assertEqual(annotations[2][0], 9)
        self.assertEqual(annotations[2][1], 'implements')
        self.assertEqual(annotations[2][2], 'REQ-002')

    def test_get_annotations_for_file_no_annotations(self):
        """Test retrieving annotations from file with no annotations."""
        # Create test file without annotations
        test_file = self.test_dir / "test.py"
        test_file.write_text("""def calculate_total(items):
    return sum(items)
""")

        # Get annotations
        annotations = self.annotator.get_annotations_for_file(test_file)

        # Verify
        self.assertEqual(len(annotations), 0)

    def test_get_all_annotations(self):
        """Test retrieving all annotations from project."""
        # Create components directory structure
        components_dir = self.test_dir / "components"
        comp1_dir = components_dir / "calculator"
        comp1_dir.mkdir(parents=True)

        # Create files with annotations
        file1 = comp1_dir / "calc.py"
        file1.write_text("""# @implements: REQ-001
def calculate_total(items):
    return sum(items)
""")

        file2 = comp1_dir / "test_calc.py"
        file2.write_text("""# @validates: REQ-001
def test_calculate_total():
    assert calculate_total([1, 2, 3]) == 6
""")

        # Create file without annotations
        file3 = comp1_dir / "utils.py"
        file3.write_text("""def helper():
    pass
""")

        # Get all annotations
        all_annotations = self.annotator.get_all_annotations()

        # Verify
        self.assertEqual(len(all_annotations), 2)  # Only files with annotations
        self.assertIn(file1, all_annotations)
        self.assertIn(file2, all_annotations)
        self.assertNotIn(file3, all_annotations)

        # Check annotations
        self.assertEqual(len(all_annotations[file1]), 1)
        self.assertEqual(all_annotations[file1][0][2], 'REQ-001')

        self.assertEqual(len(all_annotations[file2]), 1)
        self.assertEqual(all_annotations[file2][0][2], 'REQ-001')

    def test_get_all_annotations_no_components_dir(self):
        """Test retrieving annotations when components directory doesn't exist."""
        # Don't create components directory

        # Get all annotations
        all_annotations = self.annotator.get_all_annotations()

        # Verify - should return empty dict
        self.assertEqual(len(all_annotations), 0)

    def test_generate_annotation_report(self):
        """Test generating annotation report."""
        # Add some annotations
        test_file = self.test_dir / "test.py"
        test_file.write_text("""def calculate_total(items):
    return sum(items)

def test_calculate_total():
    assert calculate_total([1, 2, 3]) == 6
""")

        self.annotator.annotate_function(test_file, "calculate_total", "REQ-001")
        self.annotator.annotate_test(test_file, "test_calculate_total", "REQ-001")

        # Generate report
        report = self.annotator.generate_annotation_report()

        # Verify
        self.assertIn("REQUIREMENT ANNOTATION REPORT", report)
        self.assertIn("Total Annotations: 2", report)
        self.assertIn("Implementation Annotations: 1", report)
        self.assertIn("Test Annotations: 1", report)
        self.assertIn("@implements: REQ-001", report)
        self.assertIn("@validates: REQ-001", report)
        self.assertIn("calculate_total", report)
        self.assertIn("test_calculate_total", report)

    def test_generate_annotation_report_empty(self):
        """Test generating report with no annotations."""
        # Generate report without making any annotations
        report = self.annotator.generate_annotation_report()

        # Verify
        self.assertIn("REQUIREMENT ANNOTATION REPORT", report)
        self.assertIn("Total Annotations: 0", report)
        self.assertIn("Implementation Annotations: 0", report)
        self.assertIn("Test Annotations: 0", report)

    def test_annotation_dataclass(self):
        """Test Annotation dataclass."""
        # Create annotation
        annotation = Annotation(
            file_path=Path("/test/file.py"),
            line_number=42,
            annotation_type="implements",
            requirement_id="REQ-001",
            context="my_function"
        )

        # Verify
        self.assertEqual(annotation.file_path, Path("/test/file.py"))
        self.assertEqual(annotation.line_number, 42)
        self.assertEqual(annotation.annotation_type, "implements")
        self.assertEqual(annotation.requirement_id, "REQ-001")
        self.assertEqual(annotation.context, "my_function")

    def test_annotate_multiple_functions_in_same_file(self):
        """Test annotating multiple functions in the same file."""
        # Create test file with multiple functions
        test_file = self.test_dir / "test.py"
        test_file.write_text("""def calculate_total(items):
    return sum(items)

def calculate_average(items):
    return sum(items) / len(items)

def find_maximum(items):
    return max(items)
""")

        # Annotate all functions
        self.annotator.annotate_function(test_file, "calculate_total", "REQ-001")
        self.annotator.annotate_function(test_file, "calculate_average", "REQ-002")
        self.annotator.annotate_function(test_file, "find_maximum", "REQ-003")

        # Verify
        self.assertEqual(len(self.annotator.annotations_made), 3)
        content = test_file.read_text()
        self.assertIn("# @implements: REQ-001", content)
        self.assertIn("# @implements: REQ-002", content)
        self.assertIn("# @implements: REQ-003", content)

    def test_annotate_function_with_invalid_file(self):
        """Test annotating with invalid file path."""
        # Try to annotate non-existent file
        invalid_file = self.test_dir / "nonexistent.py"

        success = self.annotator.annotate_function(invalid_file, "some_func", "REQ-001")

        # Verify - should fail gracefully
        self.assertFalse(success)
        self.assertEqual(len(self.annotator.annotations_made), 0)

    def test_remove_annotations_with_invalid_file(self):
        """Test removing annotations from invalid file."""
        # Try to remove from non-existent file
        invalid_file = self.test_dir / "nonexistent.py"

        removed = self.annotator.remove_annotations(invalid_file)

        # Verify - should return 0 (fail gracefully)
        self.assertEqual(removed, 0)

    def test_get_annotations_for_invalid_file(self):
        """Test getting annotations from invalid file."""
        # Try to get annotations from non-existent file
        invalid_file = self.test_dir / "nonexistent.py"

        annotations = self.annotator.get_annotations_for_file(invalid_file)

        # Verify - should return empty list (fail gracefully)
        self.assertEqual(len(annotations), 0)


class TestAutoAnnotation(unittest.TestCase):
    """Test automatic annotation functionality."""

    def setUp(self):
        """Set up test fixtures."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.annotator = RequirementAnnotator(self.test_dir)

    def tearDown(self):
        """Clean up test directory."""
        if self.test_dir.exists():
            shutil.rmtree(self.test_dir)

    def test_auto_annotate_file_with_unparseable_python(self):
        """Test auto-annotating file with syntax errors."""
        # Create component directory
        component_dir = self.test_dir / "component"
        component_dir.mkdir()

        # Create file with syntax error
        bad_file = component_dir / "bad.py"
        bad_file.write_text("""def broken function(:
    this is not valid python
""")

        # Create mock requirement
        requirements = [MockRequirement("REQ-001", "broken function syntax")]

        # Try to auto-annotate
        annotations = self.annotator._auto_annotate_file(bad_file, requirements, "implements")

        # Verify - should fail gracefully
        self.assertEqual(len(annotations), 0)

    def test_extract_key_terms_with_special_characters(self):
        """Test key term extraction with special characters."""
        # Test with special characters
        text = "System must handle UTF-8 encoding & special chars!"
        terms = self.annotator._extract_key_terms(text)

        # Verify - should extract valid terms
        self.assertIn("system", terms)
        self.assertIn("handle", terms)
        self.assertIn("encoding", terms)
        self.assertIn("special", terms)
        self.assertIn("chars", terms)

    def test_extract_key_terms_empty_string(self):
        """Test key term extraction with empty string."""
        terms = self.annotator._extract_key_terms("")

        # Verify - should return empty list
        self.assertEqual(len(terms), 0)

    def test_function_matches_requirement_no_docstring(self):
        """Test matching function without docstring."""
        import ast

        code = """
def validate_user_credentials(username, password):
    pass
"""
        tree = ast.parse(code)
        func = tree.body[0]

        req = MockRequirement("REQ-001", "System must validate user credentials")

        # Should still match based on function name
        matches = self.annotator._function_matches_requirement(func, req)
        self.assertTrue(matches)

    def test_annotation_preserves_file_structure(self):
        """Test that annotation preserves original file structure."""
        # Create test file with specific structure
        test_file = self.test_dir / "test.py"
        original_content = """# Header comment
import os

def function_one():
    pass

class MyClass:
    def method_one(self):
        pass

    def method_two(self):
        pass
"""
        test_file.write_text(original_content)

        # Annotate method
        self.annotator.annotate_function(test_file, "method_one", "REQ-001")

        # Verify structure is preserved
        content = test_file.read_text()
        self.assertIn("# Header comment", content)
        self.assertIn("import os", content)
        self.assertIn("def function_one():", content)
        self.assertIn("class MyClass:", content)
        self.assertIn("def method_two(self):", content)


def run_tests():
    """Run all tests."""
    unittest.main()


if __name__ == '__main__':
    run_tests()
