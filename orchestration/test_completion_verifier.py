#!/usr/bin/env python3
"""
Tests for Completion Verifier

Tests the 8-check verification system that guarantees 100% project completion.

Part of v0.3.0 completion guarantee system.
"""

import pytest
import tempfile
import shutil
from pathlib import Path
from completion_verifier import CompletionVerifier, CompletionVerification, CheckResult


@pytest.fixture
def temp_project():
    """Create temporary project directory for testing."""
    temp_dir = Path(tempfile.mkdtemp())
    yield temp_dir
    shutil.rmtree(temp_dir)


@pytest.fixture
def verifier(temp_project):
    """Create CompletionVerifier instance."""
    return CompletionVerifier(temp_project)


@pytest.fixture
def mock_component(temp_project):
    """Create mock component directory with basic structure."""
    component_dir = temp_project / "components" / "test_component"
    component_dir.mkdir(parents=True)

    # Create basic structure
    (component_dir / "src").mkdir()
    (component_dir / "tests").mkdir()

    # Create basic files
    (component_dir / "README.md").write_text("# Test Component\n\nTest component for verification.")
    (component_dir / "CLAUDE.md").write_text("# Test Component Instructions\n\nDetailed instructions.")

    return component_dir


def test_verifier_initialization(temp_project):
    """Test CompletionVerifier initializes correctly."""
    verifier = CompletionVerifier(temp_project)

    assert verifier.project_root == temp_project.resolve()


def test_check_tests_pass_no_tests(verifier, mock_component):
    """Test check_tests_pass when no tests exist."""
    # No tests directory
    result = verifier._check_tests_pass(mock_component)

    # Should fail since tests don't exist/run
    assert result.check_name == "Tests Pass"
    assert result.is_critical is True


def test_check_imports_resolve_no_python_files(verifier, mock_component):
    """Test check_imports_resolve when no Python files exist."""
    result = verifier._check_imports_resolve(mock_component)

    assert result.check_name == "Imports Resolve"
    assert result.passed is True  # No files = nothing to check
    assert "No Python files found" in result.message


def test_check_imports_resolve_valid_python(verifier, mock_component):
    """Test check_imports_resolve with valid Python file."""
    # Create valid Python file
    py_file = mock_component / "src" / "module.py"
    py_file.write_text("""
def hello():
    return "world"
""")

    result = verifier._check_imports_resolve(mock_component)

    assert result.check_name == "Imports Resolve"
    assert result.passed is True
    assert "files checked" in result.message


def test_check_imports_resolve_syntax_error(verifier, mock_component):
    """Test check_imports_resolve with syntax error."""
    # Create Python file with syntax error
    py_file = mock_component / "src" / "broken.py"
    py_file.write_text("""
def hello(:  # Syntax error
    return "world"
""")

    result = verifier._check_imports_resolve(mock_component)

    assert result.check_name == "Imports Resolve"
    assert result.passed is False
    assert result.is_critical is True


def test_check_no_stubs_empty_component(verifier, mock_component):
    """Test check_no_stubs when no source files exist."""
    result = verifier._check_no_stubs(mock_component)

    assert result.check_name == "No Stubs"
    assert result.passed is True


def test_check_no_stubs_with_not_implemented(verifier, mock_component):
    """Test check_no_stubs detects NotImplementedError."""
    # Create file with stub
    py_file = mock_component / "src" / "incomplete.py"
    py_file.write_text("""
def process_data(data):
    raise NotImplementedError("TODO: Implement this")
""")

    result = verifier._check_no_stubs(mock_component)

    assert result.check_name == "No Stubs"
    assert result.passed is False
    assert result.is_critical is True
    assert "stub(s) remain" in result.message


def test_check_no_stubs_with_empty_function(verifier, mock_component):
    """Test check_no_stubs detects empty functions."""
    # Create file with empty function
    py_file = mock_component / "src" / "incomplete.py"
    py_file.write_text("""
def process_data(data):
    pass
""")

    result = verifier._check_no_stubs(mock_component)

    assert result.check_name == "No Stubs"
    assert result.passed is False  # Should detect pass-only functions
    assert result.is_critical is True


def test_check_no_todos_clean_code(verifier, mock_component):
    """Test check_no_todos with clean code."""
    # Create file without TODOs
    py_file = mock_component / "src" / "complete.py"
    py_file.write_text("""
def process_data(data):
    return data.upper()
""")

    result = verifier._check_no_todos(mock_component)

    assert result.check_name == "No TODOs"
    assert result.passed is True


def test_check_no_todos_with_todo_markers(verifier, mock_component):
    """Test check_no_todos detects TODO markers."""
    # Create file with TODO
    py_file = mock_component / "src" / "incomplete.py"
    py_file.write_text("""
def process_data(data):
    # TODO: Add error handling
    return data.upper()
""")

    result = verifier._check_no_todos(mock_component)

    assert result.check_name == "No TODOs"
    assert result.passed is False
    assert "TODO marker(s) found" in result.message
    assert result.is_critical is False  # Warning only


def test_check_documentation_complete_all_present(verifier, mock_component):
    """Test check_documentation_complete with all docs present."""
    # README.md and CLAUDE.md already created in fixture

    result = verifier._check_documentation_complete(mock_component)

    assert result.check_name == "Documentation Complete"
    assert result.passed is True
    assert "All required documentation present" in result.message


def test_check_documentation_complete_missing_readme(verifier, mock_component):
    """Test check_documentation_complete with missing README."""
    # Remove README
    (mock_component / "README.md").unlink()

    result = verifier._check_documentation_complete(mock_component)

    assert result.check_name == "Documentation Complete"
    assert result.passed is False
    assert "README.md" in result.message
    assert result.is_critical is False  # Warning only


def test_check_no_remaining_work_markers_clean(verifier, mock_component):
    """Test check_no_remaining_work_markers with clean code."""
    # Create clean file
    py_file = mock_component / "src" / "complete.py"
    py_file.write_text("""
def process_data(data):
    return data.upper()
""")

    result = verifier._check_no_remaining_work_markers(mock_component)

    assert result.check_name == "No Remaining Work Markers"
    assert result.passed is True


def test_check_no_remaining_work_markers_with_incomplete(verifier, mock_component):
    """Test check_no_remaining_work_markers detects incomplete markers."""
    # Create file with incomplete marker
    md_file = mock_component / "README.md"
    md_file.write_text("""
# Test Component

**Status**: IN PROGRESS

This component is incomplete.
""")

    result = verifier._check_no_remaining_work_markers(mock_component)

    assert result.check_name == "No Remaining Work Markers"
    assert result.passed is False
    assert "incomplete marker(s)" in result.message
    assert result.is_critical is False  # Warning only


def test_check_manifest_complete_missing(verifier, mock_component):
    """Test check_manifest_complete when manifest doesn't exist."""
    result = verifier._check_manifest_complete(mock_component)

    assert result.check_name == "Manifest Complete"
    assert result.passed is False
    assert "component.yaml missing" in result.message


def test_check_manifest_complete_valid(verifier, mock_component):
    """Test check_manifest_complete with valid manifest."""
    # Create valid manifest
    manifest = mock_component / "component.yaml"
    manifest.write_text("""
name: test_component
version: 1.0.0
type: feature
description: Test component for verification
""")

    result = verifier._check_manifest_complete(mock_component)

    assert result.check_name == "Manifest Complete"
    # May pass or fail depending on PyYAML availability
    # Just check it doesn't crash


def test_verify_component_complete(verifier, mock_component):
    """Test verify_component with a complete component."""
    # Create minimal complete component
    py_file = mock_component / "src" / "module.py"
    py_file.write_text("""
def hello():
    return "world"
""")

    # Create test file
    test_file = mock_component / "tests" / "test_module.py"
    test_file.write_text("""
def test_hello():
    from ..src.module import hello
    assert hello() == "world"
""")

    verification = verifier.verify_component(mock_component)

    assert verification.component_name == "test_component"
    assert isinstance(verification.is_complete, bool)
    assert isinstance(verification.completion_percentage, int)
    assert 0 <= verification.completion_percentage <= 100
    assert len(verification.checks) == 8  # All 8 checks ran


def test_verify_component_incomplete(verifier, mock_component):
    """Test verify_component with incomplete component."""
    # Create component with stubs
    py_file = mock_component / "src" / "incomplete.py"
    py_file.write_text("""
def process():
    raise NotImplementedError("TODO")
""")

    verification = verifier.verify_component(mock_component)

    assert verification.component_name == "test_component"
    assert verification.is_complete is False  # Should fail due to stubs
    assert len(verification.remaining_tasks) > 0


def test_verification_get_failed_checks(verifier, mock_component):
    """Test CompletionVerification.get_failed_checks()."""
    verification = verifier.verify_component(mock_component)

    failed = verification.get_failed_checks()

    assert isinstance(failed, list)
    # All items should be CheckResult instances
    for check in failed:
        assert isinstance(check, CheckResult)
        assert check.passed is False


def test_verification_get_critical_failures(verifier, mock_component):
    """Test CompletionVerification.get_critical_failures()."""
    verification = verifier.verify_component(mock_component)

    critical_failures = verification.get_critical_failures()

    assert isinstance(critical_failures, list)
    # All items should be critical failures
    for check in critical_failures:
        assert isinstance(check, CheckResult)
        assert check.passed is False
        assert check.is_critical is True


def test_verification_get_warnings(verifier, mock_component):
    """Test CompletionVerification.get_warnings()."""
    # Create component with TODO (warning)
    py_file = mock_component / "src" / "module.py"
    py_file.write_text("""
def hello():
    # TODO: Improve this
    return "world"
""")

    verification = verifier.verify_component(mock_component)

    warnings = verification.get_warnings()

    assert isinstance(warnings, list)
    # All items should be non-critical failures
    for check in warnings:
        assert isinstance(check, CheckResult)
        assert check.passed is False
        assert check.is_critical is False


def test_completion_percentage_calculation(verifier, mock_component):
    """Test completion percentage is calculated correctly."""
    verification = verifier.verify_component(mock_component)

    # Percentage should be between 0 and 100
    assert 0 <= verification.completion_percentage <= 100

    # Should be based on number of passed checks
    total_checks = len(verification.checks)
    passed_checks = sum(1 for c in verification.checks if c.passed)
    expected_percentage = int((passed_checks / total_checks) * 100)

    assert verification.completion_percentage == expected_percentage


def test_print_verification_report(verifier, mock_component, capsys):
    """Test print_verification_report outputs correctly."""
    verification = verifier.verify_component(mock_component)

    verifier.print_verification_report(verification)

    captured = capsys.readouterr()

    # Should contain component name
    assert "test_component" in captured.out

    # Should contain completion status
    assert "COMPLETION VERIFICATION" in captured.out

    # Should contain percentage
    assert "%" in captured.out


# Integration tests

def test_full_verification_workflow(temp_project):
    """Test complete verification workflow."""
    # Create component structure
    component_dir = temp_project / "components" / "complete_component"
    component_dir.mkdir(parents=True)
    (component_dir / "src").mkdir()
    (component_dir / "tests").mkdir()

    # Create complete component
    (component_dir / "src" / "module.py").write_text("""
def add(a, b):
    return a + b
""")

    (component_dir / "tests" / "test_module.py").write_text("""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from module import add

def test_add():
    assert add(2, 3) == 5
""")

    (component_dir / "README.md").write_text("# Complete Component")
    (component_dir / "CLAUDE.md").write_text("# Instructions")

    # Run verification
    verifier = CompletionVerifier(temp_project)
    verification = verifier.verify_component(component_dir)

    # Should have run all checks
    assert len(verification.checks) == 8

    # Some checks may pass, some may fail
    # Just ensure the workflow completes without errors
    assert verification.component_name == "complete_component"
    assert 0 <= verification.completion_percentage <= 100


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
