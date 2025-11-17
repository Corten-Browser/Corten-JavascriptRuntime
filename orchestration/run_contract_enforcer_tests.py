#!/usr/bin/env python3
"""
Simple test runner for contract_enforcer without pytest dependency.
Tests contract enforcement functionality.
"""

import sys
import tempfile
import shutil
from pathlib import Path
import yaml

# Add orchestration to path
sys.path.insert(0, str(Path(__file__).parent))

from contract_enforcer import (
    ContractEnforcer,
    EnforcementViolation,
    ContractCompliance
)


def create_temp_project():
    """Create a temporary project directory."""
    return Path(tempfile.mkdtemp())


def cleanup_temp_project(temp_dir):
    """Clean up temporary project directory."""
    if temp_dir.exists():
        shutil.rmtree(temp_dir)


def get_sample_contract():
    """Sample OpenAPI contract."""
    return {
        'openapi': '3.0.0',
        'info': {
            'title': 'Test API',
            'version': '1.0.0',
            'description': 'Test API contract'
        },
        'paths': {
            '/users': {
                'get': {
                    'summary': 'List users',
                    'operationId': 'getUsers',
                    'responses': {
                        '200': {
                            'description': 'Success',
                            'content': {
                                'application/json': {
                                    'schema': {'type': 'object'}
                                }
                            }
                        },
                        '500': {'description': 'Internal server error'}
                    }
                },
                'post': {
                    'summary': 'Create user',
                    'operationId': 'createUser',
                    'requestBody': {
                        'required': True,
                        'content': {
                            'application/json': {
                                'schema': {
                                    'type': 'object',
                                    'properties': {
                                        'name': {'type': 'string'},
                                        'email': {'type': 'string'}
                                    }
                                }
                            }
                        }
                    },
                    'responses': {
                        '201': {'description': 'Created'},
                        '400': {'description': 'Bad request'},
                        '500': {'description': 'Internal server error'}
                    }
                }
            },
            '/users/{id}': {
                'get': {
                    'summary': 'Get user by ID',
                    'operationId': 'getUserById',
                    'parameters': [
                        {
                            'name': 'id',
                            'in': 'path',
                            'required': True,
                            'schema': {'type': 'string'}
                        }
                    ],
                    'responses': {
                        '200': {'description': 'Success'},
                        '404': {'description': 'Not found'},
                        '500': {'description': 'Internal server error'}
                    }
                }
            }
        }
    }


# Test 1: Initialization
def test_init_creates_directories():
    """Test that initialization creates required directories."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)
        assert enforcer.contracts_dir.exists(), "Contracts directory not created"
        assert enforcer.components_dir.exists(), "Components directory not created"
        assert enforcer.contracts_dir == temp_project / "contracts", "Wrong contracts path"
        assert enforcer.components_dir == temp_project / "components", "Wrong components path"
        print("✓ test_init_creates_directories")
    finally:
        cleanup_temp_project(temp_project)


# Test 2: Check contract exists
def test_check_contract_exists():
    """Test checking for contract existence."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # No contract initially
        assert not enforcer.check_contract_exists("test-component"), "False positive on contract check"

        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(get_sample_contract()))

        # Now should exist
        assert enforcer.check_contract_exists("test-component"), "Contract not detected"

        print("✓ test_check_contract_exists")
    finally:
        cleanup_temp_project(temp_project)


# Test 3: Block implementation without contract
def test_block_implementation_without_contract():
    """Test blocking implementation without contract."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # Create implementation without contract
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("print('hello')")

        # Should be blocked
        assert enforcer.block_implementation_without_contract("test-component"), "Should block impl without contract"

        # Add contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(get_sample_contract()))

        # Should not be blocked now
        assert not enforcer.block_implementation_without_contract("test-component"), "Should not block with contract"

        print("✓ test_block_implementation_without_contract")
    finally:
        cleanup_temp_project(temp_project)


# Test 4: Has implementation files
def test_has_implementation_files():
    """Test checking for implementation files."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # No implementation
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        assert not enforcer._has_implementation_files(component_path), "False positive on no impl"

        # Add Python file
        (component_path / "main.py").write_text("print('hello')")
        assert enforcer._has_implementation_files(component_path), "Failed to detect Python file"

        # Clean up and test with only test files
        (component_path / "main.py").unlink()
        (component_path / "test_main.py").write_text("# test")
        (component_path / "__init__.py").write_text("# init")
        assert not enforcer._has_implementation_files(component_path), "Test files should be ignored"

        print("✓ test_has_implementation_files")
    finally:
        cleanup_temp_project(temp_project)


# Test 5: Verify compliance - implementation without contract
def test_verify_compliance_impl_without_contract():
    """Test compliance violation when implementation exists without contract."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # Create implementation
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("print('hello')")

        result = enforcer.verify_component_compliance("test-component")

        assert isinstance(result, ContractCompliance), "Wrong result type"
        assert not result.compliant, "Should not be compliant"
        assert result.has_contract is False, "Should not have contract"
        assert result.implementation_exists is True, "Should have implementation"
        assert len(result.violations) >= 1, "Should have violations"

        violation = result.violations[0]
        assert violation.violation_type == "missing_contract", "Wrong violation type"
        assert violation.severity == "critical", "Wrong severity"

        print("✓ test_verify_compliance_impl_without_contract")
    finally:
        cleanup_temp_project(temp_project)


# Test 6: Verify compliance - both exist
def test_verify_compliance_both_exist():
    """Test compliance when both contract and implementation exist."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(get_sample_contract()))

        # Create implementation
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("""
@router.get('/users')
def get_users():
    pass

@router.post('/users')
def create_user():
    pass

@router.get('/users/{id}')
def get_user_by_id(id: str):
    pass
""")

        result = enforcer.verify_component_compliance("test-component")

        assert result.has_contract, "Should have contract"
        assert result.implementation_exists, "Should have implementation"

        # Should be compliant (no critical violations)
        critical_violations = [v for v in result.violations if v.severity == "critical"]
        assert len(critical_violations) == 0, "Should have no critical violations"

        print("✓ test_verify_compliance_both_exist")
    finally:
        cleanup_temp_project(temp_project)


# Test 7: Contract completeness - invalid YAML
def test_contract_completeness_invalid_yaml():
    """Test handling of invalid YAML."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text("invalid: yaml: content: [")

        violations = enforcer._verify_contract_completeness(contract_path)

        assert len(violations) > 0, "Should detect invalid YAML"
        assert violations[0].violation_type == "invalid_contract", "Wrong violation type"
        assert violations[0].severity == "critical", "Wrong severity"

        print("✓ test_contract_completeness_invalid_yaml")
    finally:
        cleanup_temp_project(temp_project)


# Test 8: Contract completeness - missing sections
def test_contract_completeness_missing_sections():
    """Test detection of missing required sections."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump({'openapi': '3.0.0'}))

        violations = enforcer._verify_contract_completeness(contract_path)

        violation_types = [v.violation_type for v in violations]
        assert "incomplete_contract" in violation_types, "Should detect incomplete contract"

        descriptions = [v.description for v in violations]
        assert any("info" in d for d in descriptions), "Should detect missing info"
        assert any("paths" in d for d in descriptions), "Should detect missing paths"

        print("✓ test_contract_completeness_missing_sections")
    finally:
        cleanup_temp_project(temp_project)


# Test 9: Generate FastAPI skeleton
def test_generate_fastapi_skeleton():
    """Test FastAPI skeleton generation."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(get_sample_contract()))

        skeleton = enforcer.generate_implementation_skeleton("test-component", "fastapi")

        # Verify skeleton contains expected elements
        assert "from fastapi import APIRouter" in skeleton, "Missing FastAPI import"
        assert "router = APIRouter()" in skeleton, "Missing router creation"
        assert '@router.get("/users"' in skeleton, "Missing GET /users route"
        assert '@router.post("/users"' in skeleton, "Missing POST /users route"
        assert '@router.get("/users/{id}"' in skeleton, "Missing GET /users/{id} route"
        assert "async def" in skeleton, "Missing async functions"
        assert "NotImplementedError" in skeleton, "Missing NotImplementedError"

        print("✓ test_generate_fastapi_skeleton")
    finally:
        cleanup_temp_project(temp_project)


# Test 10: Generate Flask skeleton
def test_generate_flask_skeleton():
    """Test Flask skeleton generation."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(get_sample_contract()))

        skeleton = enforcer.generate_implementation_skeleton("test-component", "flask")

        # Verify skeleton contains expected elements
        assert "from flask import Blueprint" in skeleton, "Missing Flask import"
        assert "bp = Blueprint" in skeleton, "Missing blueprint creation"
        assert '@bp.route("/users"' in skeleton, "Missing /users route"
        assert '@bp.route("/users/<id>"' in skeleton, "Missing /users/<id> route"
        assert "methods=[" in skeleton, "Missing methods parameter"

        print("✓ test_generate_flask_skeleton")
    finally:
        cleanup_temp_project(temp_project)


# Test 11: Path to function name conversion
def test_path_to_function_name():
    """Test path to function name conversion."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        assert enforcer._path_to_function_name("/users", "get") == "get_user", "Simple path conversion failed"
        assert enforcer._path_to_function_name("/users/{id}", "get") == "get_user_by_id", "Path with ID failed"
        assert enforcer._path_to_function_name("/posts/{post_id}", "delete") == "delete_post_by_post_id", "Nested ID failed"

        print("✓ test_path_to_function_name")
    finally:
        cleanup_temp_project(temp_project)


# Test 12: Enforce all components
def test_enforce_all_components():
    """Test enforcing all components."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # Create component with contract (compliant)
        contract_path = enforcer.contracts_dir / "compliant-component_api.yaml"
        contract_path.write_text(yaml.dump(get_sample_contract()))
        component_path = enforcer.components_dir / "compliant-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("# implementation")

        # Create component without contract (non-compliant)
        component_path2 = enforcer.components_dir / "non-compliant-component"
        component_path2.mkdir()
        (component_path2 / "main.py").write_text("# implementation")

        results = enforcer.enforce_all_components()

        assert len(results) == 2, "Should find 2 components"
        assert "compliant-component" in results, "Should find compliant component"
        assert "non-compliant-component" in results, "Should find non-compliant component"

        # First should have contract
        assert results["compliant-component"].has_contract, "Compliant component should have contract"

        # Second should not be compliant
        assert not results["non-compliant-component"].compliant, "Non-compliant component should fail"
        assert not results["non-compliant-component"].has_contract, "Non-compliant should not have contract"

        print("✓ test_enforce_all_components")
    finally:
        cleanup_temp_project(temp_project)


# Test 13: Report generation
def test_generate_report():
    """Test report generation."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        compliance = ContractCompliance(
            component_name="test-component",
            has_contract=True,
            contract_path=Path("/path/to/contract.yaml"),
            implementation_exists=True,
            implementation_path=Path("/path/to/component"),
            compliant=True,
            violations=[]
        )

        report = enforcer.generate_report(compliance, format="text")

        assert "test-component" in report, "Component name not in report"
        assert "Contract Exists:" in report, "Missing contract status"
        assert "✅" in report, "Missing success indicator"
        assert "No violations found" in report, "Missing violations message"

        print("✓ test_generate_report")
    finally:
        cleanup_temp_project(temp_project)


# Test 14: Data class serialization
def test_data_class_serialization():
    """Test data class to_dict methods."""
    violation = EnforcementViolation(
        component_name="test",
        violation_type="missing_contract",
        description="Test violation",
        severity="critical"
    )

    data = violation.to_dict()
    assert data["component_name"] == "test", "Wrong component name"
    assert data["violation_type"] == "missing_contract", "Wrong violation type"
    assert data["description"] == "Test violation", "Wrong description"
    assert data["severity"] == "critical", "Wrong severity"

    compliance = ContractCompliance(
        component_name="test",
        has_contract=True,
        contract_path=Path("/path/to/contract"),
        implementation_exists=False,
        implementation_path=None,
        compliant=True,
        violations=[]
    )

    data = compliance.to_dict()
    assert data["component_name"] == "test", "Wrong component name"
    assert data["has_contract"] is True, "Wrong contract status"
    assert data["compliant"] is True, "Wrong compliance status"

    print("✓ test_data_class_serialization")


# Test 15: Edge cases
def test_edge_cases():
    """Test edge cases and error handling."""
    temp_project = create_temp_project()
    try:
        enforcer = ContractEnforcer(temp_project)

        # Non-existent component
        result = enforcer.verify_component_compliance("nonexistent")
        assert result.component_name == "nonexistent", "Wrong component name"
        assert not result.has_contract, "Should not have contract"
        assert not result.implementation_exists, "Should not have implementation"

        # Component with only config files
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "package.json").write_text("{}")
        (component_path / "README.md").write_text("# Readme")
        assert not enforcer._has_implementation_files(component_path), "Should ignore config files"

        # Nested source files
        component_path2 = enforcer.components_dir / "test-component2"
        src_path = component_path2 / "src" / "api"
        src_path.mkdir(parents=True)
        (src_path / "routes.py").write_text("# routes")
        assert enforcer._has_implementation_files(component_path2), "Should detect nested files"

        print("✓ test_edge_cases")
    finally:
        cleanup_temp_project(temp_project)


def run_all_tests():
    """Run all tests."""
    tests = [
        test_init_creates_directories,
        test_check_contract_exists,
        test_block_implementation_without_contract,
        test_has_implementation_files,
        test_verify_compliance_impl_without_contract,
        test_verify_compliance_both_exist,
        test_contract_completeness_invalid_yaml,
        test_contract_completeness_missing_sections,
        test_generate_fastapi_skeleton,
        test_generate_flask_skeleton,
        test_path_to_function_name,
        test_enforce_all_components,
        test_generate_report,
        test_data_class_serialization,
        test_edge_cases,
    ]

    print("=== Running Contract Enforcer Tests ===\n")

    passed = 0
    failed = 0
    errors = []

    for test in tests:
        try:
            test()
            passed += 1
        except AssertionError as e:
            failed += 1
            errors.append(f"{test.__name__}: {str(e)}")
            print(f"✗ {test.__name__}: {str(e)}")
        except Exception as e:
            failed += 1
            errors.append(f"{test.__name__}: {str(e)}")
            print(f"✗ {test.__name__}: {str(e)}")

    print("\n=== Test Results ===")
    print(f"Passed: {passed}/{passed + failed}")
    print(f"Failed: {failed}/{passed + failed}")

    if failed == 0:
        print("\n✓ All tests passed!")
        return 0
    else:
        print("\n✗ Some tests failed:")
        for error in errors:
            print(f"  - {error}")
        return 1


if __name__ == '__main__':
    sys.exit(run_all_tests())
