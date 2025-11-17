#!/usr/bin/env python3
"""
Comprehensive tests for requirements_tracker.py

Tests cover:
- Requirement parsing from specifications
- Implementation scanning
- Test scanning
- Traceability matrix generation
- Coverage analysis
- Database persistence
"""

import pytest
import json
import tempfile
from pathlib import Path
from datetime import datetime

from requirements_tracker import (
    Requirement,
    Implementation,
    Test,
    RequirementTrace,
    TraceabilityMatrix,
    RequirementsTracker
)


# ===== Fixtures =====

@pytest.fixture
def temp_project():
    """Create temporary project directory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()
        yield project_root


@pytest.fixture
def sample_spec_file(temp_project):
    """Create sample specification file."""
    spec_content = """
# Requirements Specification

## Authentication

REQ-001: User MUST be able to login with email and password.

The system SHALL validate credentials against the database.

User story: As a user, I want to reset my password if I forget it.

## Payment Processing

The system SHOULD accept credit card payments.

Payment transactions MAY be processed asynchronously.
"""
    spec_file = temp_project / "requirements.md"
    spec_file.write_text(spec_content)
    return spec_file


@pytest.fixture
def sample_implementation_file(temp_project):
    """Create sample implementation file."""
    impl_content = """
# @implements: REQ-001
def authenticate_user(email, password):
    '''Authenticate user with email and password.'''
    # REQ-001: Validate credentials
    return validate_credentials(email, password)

# Implements REQ-002
def send_password_reset(email):
    '''Send password reset email.'''
    pass
"""
    impl_dir = temp_project / "components" / "auth" / "src"
    impl_dir.mkdir(parents=True)
    impl_file = impl_dir / "auth.py"
    impl_file.write_text(impl_content)
    return impl_file


@pytest.fixture
def sample_test_file(temp_project):
    """Create sample test file."""
    test_content = """
import pytest

# @validates: REQ-001
def test_req_001_valid_login():
    '''Test valid login credentials.'''
    assert authenticate_user('test@example.com', 'password123')

# Tests REQ-001
def test_req_001_invalid_login():
    '''Test invalid login credentials.'''
    assert not authenticate_user('test@example.com', 'wrongpass')

def test_req_002_password_reset():
    '''Test password reset functionality.'''
    send_password_reset('test@example.com')
"""
    test_dir = temp_project / "components" / "auth" / "tests"
    test_dir.mkdir(parents=True)
    test_file = test_dir / "test_auth.py"
    test_file.write_text(test_content)
    return test_file


@pytest.fixture
def tracker(temp_project):
    """Create RequirementsTracker instance."""
    return RequirementsTracker(temp_project)


# ===== Data Model Tests =====

def test_requirement_to_dict():
    """Test Requirement serialization."""
    req = Requirement(
        id="REQ-001",
        text="User must login",
        source="spec.md:line:10",
        priority="MUST",
        category="authentication",
        status="pending",
        created_at="2025-11-11T16:00:00"
    )
    data = req.to_dict()

    assert data['id'] == "REQ-001"
    assert data['text'] == "User must login"
    assert data['priority'] == "MUST"


def test_requirement_from_dict():
    """Test Requirement deserialization."""
    data = {
        'id': "REQ-001",
        'text': "User must login",
        'source': "spec.md:line:10",
        'priority': "MUST",
        'category': "authentication",
        'status': "pending",
        'created_at': "2025-11-11T16:00:00"
    }
    req = Requirement.from_dict(data)

    assert req.id == "REQ-001"
    assert req.text == "User must login"
    assert req.priority == "MUST"


def test_implementation_serialization():
    """Test Implementation serialization/deserialization."""
    impl = Implementation(
        file="src/auth.py",
        line=42,
        function="authenticate",
        description="Login implementation"
    )

    data = impl.to_dict()
    assert data['file'] == "src/auth.py"
    assert data['line'] == 42

    impl2 = Implementation.from_dict(data)
    assert impl2.file == impl.file
    assert impl2.line == impl.line


def test_test_serialization():
    """Test Test serialization/deserialization."""
    test = Test(
        file="tests/test_auth.py",
        line=15,
        test_name="test_login",
        status="passing"
    )

    data = test.to_dict()
    assert data['test_name'] == "test_login"
    assert data['status'] == "passing"

    test2 = Test.from_dict(data)
    assert test2.test_name == test.test_name


def test_requirement_trace_is_implemented():
    """Test RequirementTrace.is_implemented()."""
    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )

    trace = RequirementTrace(requirement=req)
    assert not trace.is_implemented()

    trace.implementations.append(Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    ))
    assert trace.is_implemented()


def test_requirement_trace_is_tested():
    """Test RequirementTrace.is_tested()."""
    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )

    trace = RequirementTrace(requirement=req)
    assert not trace.is_tested()

    trace.tests.append(Test(
        file="tests/test.py",
        line=5,
        test_name="test_feature",
        status="passing"
    ))
    assert trace.is_tested()


def test_requirement_trace_is_complete():
    """Test RequirementTrace.is_complete()."""
    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )

    trace = RequirementTrace(requirement=req)
    assert not trace.is_complete()

    # Add implementation
    trace.implementations.append(Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    ))
    assert not trace.is_complete()  # Still needs test

    # Add passing test
    trace.tests.append(Test(
        file="tests/test.py",
        line=5,
        test_name="test_feature",
        status="passing"
    ))
    assert trace.is_complete()

    # Failing test makes it incomplete
    trace.tests[0].status = "failing"
    assert not trace.is_complete()


def test_requirement_trace_serialization():
    """Test RequirementTrace serialization/deserialization."""
    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )

    trace = RequirementTrace(
        requirement=req,
        implementations=[Implementation("src/code.py", 10, "func", "Impl")],
        tests=[Test("tests/test.py", 5, "test_feature", "passing")]
    )

    data = trace.to_dict()
    assert 'requirement' in data
    assert 'implementations' in data
    assert 'tests' in data

    trace2 = RequirementTrace.from_dict(data)
    assert trace2.requirement.id == trace.requirement.id
    assert len(trace2.implementations) == 1
    assert len(trace2.tests) == 1


# ===== Requirement Parsing Tests =====

def test_parse_explicit_requirements(tracker, sample_spec_file):
    """Test parsing explicit REQ-XXX requirements."""
    requirements = tracker.parse_requirements(sample_spec_file)

    req_ids = [r.id for r in requirements]
    assert "REQ-001" in req_ids

    req_001 = next(r for r in requirements if r.id == "REQ-001")
    assert "login with email and password" in req_001.text
    assert req_001.priority == "MUST"


def test_parse_must_statements(tracker, sample_spec_file):
    """Test parsing MUST statements."""
    requirements = tracker.parse_requirements(sample_spec_file)

    # Should find the SHALL statement
    shall_reqs = [r for r in requirements if "validate credentials" in r.text]
    assert len(shall_reqs) > 0
    assert shall_reqs[0].priority == "MUST"


def test_parse_should_statements(tracker, sample_spec_file):
    """Test parsing SHOULD statements."""
    requirements = tracker.parse_requirements(sample_spec_file)

    should_reqs = [r for r in requirements if "credit card" in r.text]
    assert len(should_reqs) > 0
    assert should_reqs[0].priority == "SHOULD"


def test_parse_may_statements(tracker, sample_spec_file):
    """Test parsing MAY statements."""
    requirements = tracker.parse_requirements(sample_spec_file)

    may_reqs = [r for r in requirements if "asynchronously" in r.text]
    assert len(may_reqs) > 0
    assert may_reqs[0].priority == "MAY"


def test_parse_user_stories(tracker, sample_spec_file):
    """Test parsing user stories."""
    requirements = tracker.parse_requirements(sample_spec_file)

    story_reqs = [r for r in requirements if "reset my password" in r.text]
    assert len(story_reqs) > 0
    assert story_reqs[0].priority == "SHOULD"


def test_infer_category_authentication(tracker):
    """Test category inference for authentication."""
    category = tracker._infer_category("User must login with password")
    assert category == "authentication"


def test_infer_category_payment(tracker):
    """Test category inference for payment."""
    category = tracker._infer_category("Process payment transaction")
    assert category == "payment"


def test_infer_category_security(tracker):
    """Test category inference for security."""
    category = tracker._infer_category("Encrypt sensitive data")
    assert category == "security"


def test_infer_category_performance(tracker):
    """Test category inference for performance."""
    category = tracker._infer_category("Optimize query performance")
    assert category == "performance"


def test_infer_category_default(tracker):
    """Test default category inference."""
    category = tracker._infer_category("Some generic requirement")
    assert category == "general"


def test_parse_nonexistent_file(tracker, temp_project):
    """Test parsing nonexistent specification file."""
    with pytest.raises(FileNotFoundError):
        tracker.parse_requirements(temp_project / "nonexistent.md")


# ===== Implementation Scanning Tests =====

def test_scan_implementation_decorator_marker(tracker, sample_implementation_file, temp_project):
    """Test scanning for @implements markers."""
    component_path = temp_project / "components" / "auth"
    implementations = tracker.scan_implementation(component_path)

    req_ids = [req_id for req_id, _ in implementations]
    assert "REQ-001" in req_ids


def test_scan_implementation_comment_marker(tracker, sample_implementation_file, temp_project):
    """Test scanning for comment-based markers."""
    component_path = temp_project / "components" / "auth"
    implementations = tracker.scan_implementation(component_path)

    # Should find both decorator and comment markers
    req_001_impls = [impl for req_id, impl in implementations if req_id == "REQ-001"]
    assert len(req_001_impls) >= 1


def test_scan_implementation_implements_keyword(tracker, sample_implementation_file, temp_project):
    """Test scanning for 'Implements REQ-XXX' markers."""
    component_path = temp_project / "components" / "auth"
    implementations = tracker.scan_implementation(component_path)

    req_ids = [req_id for req_id, _ in implementations]
    assert "REQ-002" in req_ids


def test_scan_implementation_skips_test_files(tracker, temp_project):
    """Test that implementation scanning skips test files."""
    # Create a test file with implementation marker
    test_dir = temp_project / "components" / "auth"
    test_dir.mkdir(parents=True, exist_ok=True)
    test_file = test_dir / "test_something.py"
    test_file.write_text("# @implements: REQ-999\ndef test_func(): pass")

    implementations = tracker.scan_implementation(test_dir)
    req_ids = [req_id for req_id, _ in implementations]

    # Should not find REQ-999 since it's in a test file
    assert "REQ-999" not in req_ids


# ===== Test Scanning Tests =====

def test_scan_tests_validates_decorator(tracker, sample_test_file, temp_project):
    """Test scanning for @validates markers."""
    component_path = temp_project / "components" / "auth"
    tests = tracker.scan_tests(component_path)

    req_ids = [req_id for req_id, _ in tests]
    assert "REQ-001" in req_ids


def test_scan_tests_comment_marker(tracker, sample_test_file, temp_project):
    """Test scanning for 'Tests REQ-XXX' markers."""
    component_path = temp_project / "components" / "auth"
    tests = tracker.scan_tests(component_path)

    req_001_tests = [test for req_id, test in tests if req_id == "REQ-001"]
    assert len(req_001_tests) >= 1


def test_scan_tests_naming_convention(tracker, sample_test_file, temp_project):
    """Test scanning for test_req_XXX naming convention."""
    component_path = temp_project / "components" / "auth"
    tests = tracker.scan_tests(component_path)

    req_002_tests = [test for req_id, test in tests if req_id == "REQ-002"]
    assert len(req_002_tests) >= 1


# ===== Database Operations Tests =====

def test_add_requirements(tracker):
    """Test adding requirements to database."""
    req = Requirement(
        id="REQ-001",
        text="Test requirement",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )

    tracker.add_requirements([req])
    assert "REQ-001" in tracker.requirements
    assert tracker.requirements["REQ-001"].requirement.text == "Test requirement"


def test_add_implementations(tracker):
    """Test adding implementations to database."""
    # First add requirement
    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )
    tracker.add_requirements([req])

    # Add implementation
    impl = Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    )
    tracker.add_implementations([("REQ-001", impl)])

    assert len(tracker.requirements["REQ-001"].implementations) == 1
    assert tracker.requirements["REQ-001"].implementations[0].file == "src/code.py"


def test_add_implementations_unknown_requirement(tracker, capsys):
    """Test adding implementation for unknown requirement."""
    impl = Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    )

    tracker.add_implementations([("REQ-999", impl)])
    captured = capsys.readouterr()
    assert "unknown requirement" in captured.out.lower()


def test_add_tests(tracker):
    """Test adding tests to database."""
    # First add requirement
    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )
    tracker.add_requirements([req])

    # Add test
    test = Test(
        file="tests/test.py",
        line=5,
        test_name="test_feature",
        status="passing"
    )
    tracker.add_tests([("REQ-001", test)])

    assert len(tracker.requirements["REQ-001"].tests) == 1
    assert tracker.requirements["REQ-001"].tests[0].test_name == "test_feature"


def test_add_duplicate_implementations(tracker):
    """Test that duplicate implementations are not added."""
    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )
    tracker.add_requirements([req])

    impl = Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    )

    # Add same implementation twice
    tracker.add_implementations([("REQ-001", impl)])
    tracker.add_implementations([("REQ-001", impl)])

    # Should only have one
    assert len(tracker.requirements["REQ-001"].implementations) == 1


def test_database_persistence(temp_project):
    """Test database save and load."""
    tracker1 = RequirementsTracker(temp_project)

    req = Requirement(
        id="REQ-001",
        text="Test",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )
    tracker1.add_requirements([req])
    tracker1._save_database()

    # Load in new instance
    tracker2 = RequirementsTracker(temp_project)
    assert "REQ-001" in tracker2.requirements
    assert tracker2.requirements["REQ-001"].requirement.text == "Test"


# ===== Traceability Matrix Tests =====

def test_generate_traceability_matrix_empty(tracker):
    """Test generating matrix with no requirements."""
    matrix = tracker.generate_traceability_matrix()

    assert matrix.total_requirements == 0
    assert matrix.coverage_percentage == 0.0


def test_generate_traceability_matrix_with_data(tracker):
    """Test generating matrix with requirements."""
    # Add requirements
    req1 = Requirement(
        id="REQ-001",
        text="Implemented and tested",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )
    req2 = Requirement(
        id="REQ-002",
        text="Only implemented",
        source="spec.md:line:2",
        priority="MUST",
        category="general",
        status="pending"
    )
    tracker.add_requirements([req1, req2])

    # Add implementation and test for REQ-001
    tracker.add_implementations([("REQ-001", Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    ))])
    tracker.add_tests([("REQ-001", Test(
        file="tests/test.py",
        line=5,
        test_name="test_feature",
        status="passing"
    ))])

    # Add only implementation for REQ-002
    tracker.add_implementations([("REQ-002", Implementation(
        file="src/code2.py",
        line=20,
        function="func2",
        description="Impl2"
    ))])

    matrix = tracker.generate_traceability_matrix()

    assert matrix.total_requirements == 2
    assert matrix.implemented_requirements == 2
    assert matrix.tested_requirements == 1
    assert matrix.complete_requirements == 1
    assert matrix.coverage_percentage == 50.0


def test_find_unimplemented_requirements(tracker):
    """Test finding unimplemented requirements."""
    req1 = Requirement(
        id="REQ-001",
        text="Implemented",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )
    req2 = Requirement(
        id="REQ-002",
        text="Not implemented",
        source="spec.md:line:2",
        priority="MUST",
        category="general",
        status="pending"
    )
    tracker.add_requirements([req1, req2])

    tracker.add_implementations([("REQ-001", Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    ))])

    unimplemented = tracker.find_unimplemented_requirements()

    assert len(unimplemented) == 1
    assert unimplemented[0].id == "REQ-002"


def test_find_untested_requirements(tracker):
    """Test finding untested requirements."""
    req1 = Requirement(
        id="REQ-001",
        text="Tested",
        source="spec.md:line:1",
        priority="MUST",
        category="general",
        status="pending"
    )
    req2 = Requirement(
        id="REQ-002",
        text="Not tested",
        source="spec.md:line:2",
        priority="MUST",
        category="general",
        status="pending"
    )
    tracker.add_requirements([req1, req2])

    tracker.add_tests([("REQ-001", Test(
        file="tests/test.py",
        line=5,
        test_name="test_feature",
        status="passing"
    ))])

    untested = tracker.find_untested_requirements()

    assert len(untested) == 1
    assert untested[0].id == "REQ-002"


def test_verify_requirement_coverage(tracker):
    """Test coverage verification by category."""
    # Add requirements in different categories
    req1 = Requirement(
        id="REQ-001",
        text="Auth requirement",
        source="spec.md:line:1",
        priority="MUST",
        category="authentication",
        status="pending"
    )
    req2 = Requirement(
        id="REQ-002",
        text="Payment requirement",
        source="spec.md:line:2",
        priority="MUST",
        category="payment",
        status="pending"
    )
    tracker.add_requirements([req1, req2])

    # Complete REQ-001
    tracker.add_implementations([("REQ-001", Implementation(
        file="src/auth.py",
        line=10,
        function="auth",
        description="Auth impl"
    ))])
    tracker.add_tests([("REQ-001", Test(
        file="tests/test_auth.py",
        line=5,
        test_name="test_auth",
        status="passing"
    ))])

    coverage = tracker.verify_requirement_coverage()

    assert "authentication" in coverage
    assert "payment" in coverage

    assert coverage["authentication"]["total"] == 1
    assert coverage["authentication"]["complete_coverage"] == 100.0

    assert coverage["payment"]["total"] == 1
    assert coverage["payment"]["complete_coverage"] == 0.0


# ===== Helper Method Tests =====

def test_get_line_number(tracker):
    """Test line number calculation."""
    text = "line1\nline2\nline3"
    assert tracker._get_line_number(text, 0) == 1
    assert tracker._get_line_number(text, 6) == 2
    assert tracker._get_line_number(text, 12) == 3


def test_find_function_at_line(tracker, temp_project):
    """Test finding function name at line."""
    code = """
def function1():
    pass

def function2():
    # Line 6
    pass
"""
    code_file = temp_project / "code.py"
    code_file.write_text(code)

    func = tracker._find_function_at_line(code_file, 6)
    assert func == "function2"


def test_find_function_at_line_no_function(tracker, temp_project):
    """Test finding function when line is not in a function."""
    code = "# Just a comment\nprint('hello')\n"
    code_file = temp_project / "code.py"
    code_file.write_text(code)

    func = tracker._find_function_at_line(code_file, 1)
    assert func is None


def test_extract_priority_from_text(tracker):
    """Test priority extraction from text."""
    assert tracker._extract_priority("User MUST login") == "MUST"
    assert tracker._extract_priority("System SHOULD validate") == "SHOULD"
    assert tracker._extract_priority("Feature MAY be enabled") == "MAY"
    assert tracker._extract_priority("Some requirement") == "SHOULD"


# ===== Integration Tests =====

def test_full_workflow(tracker, sample_spec_file, sample_implementation_file, sample_test_file, temp_project):
    """Test complete workflow: parse, scan, generate matrix."""
    # Parse requirements
    requirements = tracker.parse_requirements(sample_spec_file)
    tracker.add_requirements(requirements)

    # Scan implementations and tests
    component_path = temp_project / "components" / "auth"
    implementations = tracker.scan_implementation(component_path)
    tests = tracker.scan_tests(component_path)

    tracker.add_implementations(implementations)
    tracker.add_tests(tests)

    # Generate matrix
    matrix = tracker.generate_traceability_matrix()

    assert matrix.total_requirements > 0
    assert matrix.implemented_requirements > 0
    assert matrix.tested_requirements > 0


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
