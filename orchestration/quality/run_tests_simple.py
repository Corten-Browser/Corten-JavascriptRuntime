#!/usr/bin/env python3
"""
Simple test runner without pytest dependency.
Tests core functionality of requirements_tracker.
"""

import sys
import tempfile
from pathlib import Path

# Add orchestration to path
sys.path.insert(0, str(Path(__file__).parent))

from requirements_tracker import (
    Requirement,
    Implementation,
    RequirementTest,
    RequirementTrace,
    RequirementsTracker
)

def test_requirement_serialization():
    """Test Requirement serialization/deserialization."""
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

    req2 = Requirement.from_dict(data)
    assert req2.id == req.id
    assert req2.text == req.text

    print("✓ test_requirement_serialization")

def test_requirement_trace():
    """Test RequirementTrace logic."""
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
    assert not trace.is_tested()
    assert not trace.is_complete()

    # Add implementation
    trace.implementations.append(Implementation(
        file="src/code.py",
        line=10,
        function="func",
        description="Impl"
    ))
    assert trace.is_implemented()
    assert not trace.is_complete()

    # Add passing test
    trace.tests.append(RequirementTest(
        file="tests/test.py",
        line=5,
        test_name="test_feature",
        status="passing"
    ))
    assert trace.is_tested()
    assert trace.is_complete()

    print("✓ test_requirement_trace")

def test_parse_requirements():
    """Test requirement parsing."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()

        # Create spec file
        spec_content = """
# Requirements

REQ-001: User MUST be able to login with email and password.

The system SHALL validate credentials.

User story: As a user, I want to reset my password.

The system SHOULD accept credit cards.

Payment MAY be processed async.
"""
        spec_file = project_root / "spec.md"
        spec_file.write_text(spec_content)

        tracker = RequirementsTracker(project_root)
        requirements = tracker.parse_requirements(spec_file)

        assert len(requirements) > 0

        # Check explicit requirement
        req_001 = next((r for r in requirements if r.id == "REQ-001"), None)
        assert req_001 is not None
        assert "login" in req_001.text.lower()

        # Check priorities
        must_reqs = [r for r in requirements if r.priority == "MUST"]
        should_reqs = [r for r in requirements if r.priority == "SHOULD"]
        may_reqs = [r for r in requirements if r.priority == "MAY"]

        assert len(must_reqs) > 0
        assert len(should_reqs) > 0
        assert len(may_reqs) > 0

        print("✓ test_parse_requirements")

def test_scan_implementation():
    """Test implementation scanning."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()

        # Create implementation file
        impl_content = """
# @implements: REQ-001
def authenticate_user(email, password):
    # REQ-002: Validate credentials
    return True

# Implements REQ-003
def send_email():
    pass
"""
        impl_dir = project_root / "src"
        impl_dir.mkdir()
        impl_file = impl_dir / "auth.py"
        impl_file.write_text(impl_content)

        tracker = RequirementsTracker(project_root)
        implementations = tracker.scan_implementation(project_root)

        req_ids = [req_id for req_id, _ in implementations]
        assert "REQ-001" in req_ids
        assert "REQ-002" in req_ids
        assert "REQ-003" in req_ids

        print("✓ test_scan_implementation")

def test_scan_tests():
    """Test test scanning."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()

        # Create test file
        test_content = """
# @validates: REQ-001
def test_login():
    assert True

# Tests REQ-002
def test_password():
    assert True

def test_req_003_feature():
    assert True
"""
        test_dir = project_root / "tests"
        test_dir.mkdir()
        test_file = test_dir / "test_auth.py"
        test_file.write_text(test_content)

        tracker = RequirementsTracker(project_root)
        tests = tracker.scan_tests(project_root)

        req_ids = [req_id for req_id, _ in tests]
        assert "REQ-001" in req_ids
        assert "REQ-002" in req_ids
        assert "REQ-003" in req_ids

        print("✓ test_scan_tests")

def test_traceability_matrix():
    """Test traceability matrix generation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()

        tracker = RequirementsTracker(project_root)

        # Add requirements
        req1 = Requirement(
            id="REQ-001",
            text="Complete requirement",
            source="spec.md:line:1",
            priority="MUST",
            category="general",
            status="pending"
        )
        req2 = Requirement(
            id="REQ-002",
            text="Incomplete requirement",
            source="spec.md:line:2",
            priority="MUST",
            category="general",
            status="pending"
        )
        tracker.add_requirements([req1, req2])

        # Complete REQ-001
        tracker.add_implementations([("REQ-001", Implementation(
            file="src/code.py",
            line=10,
            function="func",
            description="Impl"
        ))])
        tracker.add_tests([("REQ-001", RequirementTest(
            file="tests/test.py",
            line=5,
            test_name="test_feature",
            status="passing"
        ))])

        # Generate matrix
        matrix = tracker.generate_traceability_matrix()

        assert matrix.total_requirements == 2
        assert matrix.implemented_requirements == 1
        assert matrix.tested_requirements == 1
        assert matrix.complete_requirements == 1
        assert matrix.coverage_percentage == 50.0

        print("✓ test_traceability_matrix")

def test_database_persistence():
    """Test database save/load."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()

        # Save data
        tracker1 = RequirementsTracker(project_root)
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
        tracker2 = RequirementsTracker(project_root)
        assert "REQ-001" in tracker2.requirements
        assert tracker2.requirements["REQ-001"].requirement.text == "Test"

        print("✓ test_database_persistence")

def test_category_inference():
    """Test category inference."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()

        tracker = RequirementsTracker(project_root)

        assert tracker._infer_category("User must login with password") == "authentication"
        assert tracker._infer_category("Process payment transaction") == "payment"
        assert tracker._infer_category("Encrypt sensitive data") == "security"
        assert tracker._infer_category("Optimize query performance") == "performance"
        assert tracker._infer_category("Generic requirement") == "general"

        print("✓ test_category_inference")

def test_coverage_by_category():
    """Test coverage calculation by category."""
    with tempfile.TemporaryDirectory() as tmpdir:
        project_root = Path(tmpdir)
        (project_root / "orchestration").mkdir()

        tracker = RequirementsTracker(project_root)

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
        tracker.add_tests([("REQ-001", RequirementTest(
            file="tests/test_auth.py",
            line=5,
            test_name="test_auth",
            status="passing"
        ))])

        coverage = tracker.verify_requirement_coverage()

        assert "authentication" in coverage
        assert "payment" in coverage
        assert coverage["authentication"]["complete_coverage"] == 100.0
        assert coverage["payment"]["complete_coverage"] == 0.0

        print("✓ test_coverage_by_category")

def run_all_tests():
    """Run all tests."""
    print("\n=== Running Requirements Tracker Tests ===\n")

    tests = [
        test_requirement_serialization,
        test_requirement_trace,
        test_parse_requirements,
        test_scan_implementation,
        test_scan_tests,
        test_traceability_matrix,
        test_database_persistence,
        test_category_inference,
        test_coverage_by_category,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            test()
            passed += 1
        except AssertionError as e:
            print(f"✗ {test.__name__}: {e}")
            failed += 1
        except Exception as e:
            print(f"✗ {test.__name__}: Unexpected error: {e}")
            failed += 1

    print(f"\n=== Test Results ===")
    print(f"Passed: {passed}/{len(tests)}")
    print(f"Failed: {failed}/{len(tests)}")

    if failed > 0:
        sys.exit(1)
    else:
        print("\n✓ All tests passed!")
        sys.exit(0)

if __name__ == '__main__':
    run_all_tests()
