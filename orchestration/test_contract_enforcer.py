#!/usr/bin/env python3
"""
Comprehensive tests for Contract Enforcer

Tests contract-first development enforcement.
Part of v0.4.0 quality enhancement system - Batch 2.
"""

import pytest
import tempfile
import shutil
from pathlib import Path
import yaml
import json

from contract_enforcer import (
    ContractEnforcer,
    EnforcementViolation,
    ContractCompliance
)


@pytest.fixture
def temp_project():
    """Create a temporary project directory."""
    temp_dir = Path(tempfile.mkdtemp())
    yield temp_dir
    shutil.rmtree(temp_dir)


@pytest.fixture
def enforcer(temp_project):
    """Create a ContractEnforcer instance."""
    return ContractEnforcer(temp_project)


@pytest.fixture
def sample_contract():
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
                                    'schema': {
                                        'type': 'object'
                                    }
                                }
                            }
                        },
                        '500': {
                            'description': 'Internal server error'
                        }
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
                        '201': {
                            'description': 'Created'
                        },
                        '400': {
                            'description': 'Bad request'
                        },
                        '500': {
                            'description': 'Internal server error'
                        }
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
                        '200': {
                            'description': 'Success'
                        },
                        '404': {
                            'description': 'Not found'
                        },
                        '500': {
                            'description': 'Internal server error'
                        }
                    }
                }
            }
        }
    }


class TestContractEnforcer:
    """Test ContractEnforcer class."""

    def test_init_creates_directories(self, temp_project):
        """Test that initialization creates required directories."""
        enforcer = ContractEnforcer(temp_project)

        assert enforcer.contracts_dir.exists()
        assert enforcer.components_dir.exists()
        assert enforcer.contracts_dir == temp_project / "contracts"
        assert enforcer.components_dir == temp_project / "components"

    def test_check_contract_exists_no_contract(self, enforcer):
        """Test checking for non-existent contract."""
        assert not enforcer.check_contract_exists("test-component")

    def test_check_contract_exists_with_contract(self, enforcer, sample_contract):
        """Test checking for existing contract."""
        # Create contract file
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        assert enforcer.check_contract_exists("test-component")

    def test_get_contract_path_no_contract(self, enforcer):
        """Test getting path for non-existent contract."""
        assert enforcer.get_contract_path("test-component") is None

    def test_get_contract_path_with_contract(self, enforcer, sample_contract):
        """Test getting path for existing contract."""
        # Create contract file
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        result = enforcer.get_contract_path("test-component")
        assert result is not None
        assert result == contract_path
        assert result.exists()

    def test_has_implementation_files_no_implementation(self, enforcer):
        """Test checking for implementation when none exists."""
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()

        assert not enforcer._has_implementation_files(component_path)

    def test_has_implementation_files_with_python(self, enforcer):
        """Test checking for Python implementation."""
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()

        # Create Python file
        (component_path / "main.py").write_text("print('hello')")

        assert enforcer._has_implementation_files(component_path)

    def test_has_implementation_files_ignores_test_files(self, enforcer):
        """Test that test files are ignored."""
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()

        # Create only test files
        (component_path / "test_main.py").write_text("# test")
        (component_path / "main_test.py").write_text("# test")
        (component_path / "conftest.py").write_text("# config")
        (component_path / "__init__.py").write_text("# init")

        assert not enforcer._has_implementation_files(component_path)

    def test_has_implementation_files_with_javascript(self, enforcer):
        """Test checking for JavaScript implementation."""
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()

        # Create JavaScript file
        (component_path / "index.js").write_text("console.log('hello')")

        assert enforcer._has_implementation_files(component_path)

    def test_block_implementation_without_contract_no_impl(self, enforcer):
        """Test blocking when no implementation exists."""
        # No implementation, no contract
        assert not enforcer.block_implementation_without_contract("test-component")

    def test_block_implementation_without_contract_with_contract(self, enforcer, sample_contract):
        """Test not blocking when contract exists."""
        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        # Create implementation
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("print('hello')")

        assert not enforcer.block_implementation_without_contract("test-component")

    def test_block_implementation_without_contract_should_block(self, enforcer):
        """Test blocking when implementation exists without contract."""
        # Create implementation without contract
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("print('hello')")

        assert enforcer.block_implementation_without_contract("test-component")


class TestContractCompliance:
    """Test contract compliance checking."""

    def test_verify_compliance_no_contract_no_impl(self, enforcer):
        """Test compliance when nothing exists."""
        result = enforcer.verify_component_compliance("test-component")

        assert isinstance(result, ContractCompliance)
        assert result.component_name == "test-component"
        assert not result.has_contract
        assert not result.implementation_exists
        assert result.compliant  # No violations yet
        assert len(result.violations) == 0

    def test_verify_compliance_impl_without_contract(self, enforcer):
        """Test compliance violation when implementation exists without contract."""
        # Create implementation
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("print('hello')")

        result = enforcer.verify_component_compliance("test-component")

        assert not result.compliant
        assert result.has_contract is False
        assert result.implementation_exists is True
        assert len(result.violations) == 1

        violation = result.violations[0]
        assert violation.violation_type == "missing_contract"
        assert violation.severity == "critical"
        assert "Implementation exists without contract" in violation.description

    def test_verify_compliance_contract_without_impl(self, enforcer, sample_contract):
        """Test compliance when contract exists but no implementation."""
        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        result = enforcer.verify_component_compliance("test-component")

        assert result.compliant
        assert result.has_contract
        assert not result.implementation_exists
        # May have warnings about contract completeness, but no critical violations

    def test_verify_compliance_both_exist(self, enforcer, sample_contract):
        """Test compliance when both contract and implementation exist."""
        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

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

        assert result.has_contract
        assert result.implementation_exists
        # Should be compliant (no critical violations)
        critical_violations = [v for v in result.violations if v.severity == "critical"]
        assert len(critical_violations) == 0


class TestContractCompleteness:
    """Test contract completeness verification."""

    def test_invalid_yaml(self, enforcer):
        """Test handling of invalid YAML."""
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text("invalid: yaml: content: [")

        violations = enforcer._verify_contract_completeness(contract_path)

        assert len(violations) > 0
        assert violations[0].violation_type == "invalid_contract"
        assert violations[0].severity == "critical"

    def test_missing_required_sections(self, enforcer):
        """Test detection of missing required sections."""
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump({'openapi': '3.0.0'}))

        violations = enforcer._verify_contract_completeness(contract_path)

        violation_types = [v.violation_type for v in violations]
        assert "incomplete_contract" in violation_types

        # Check specific missing sections
        descriptions = [v.description for v in violations]
        assert any("info" in d for d in descriptions)
        assert any("paths" in d for d in descriptions)

    def test_missing_info_fields(self, enforcer):
        """Test detection of missing info fields."""
        contract = {
            'openapi': '3.0.0',
            'info': {
                'description': 'Test'
            },
            'paths': {}
        }
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(contract))

        violations = enforcer._verify_contract_completeness(contract_path)

        # Should have warnings about missing title and version
        descriptions = [v.description for v in violations]
        assert any("title" in d for d in descriptions)
        assert any("version" in d for d in descriptions)

    def test_missing_error_responses(self, enforcer):
        """Test detection of missing error responses."""
        contract = {
            'openapi': '3.0.0',
            'info': {'title': 'Test', 'version': '1.0.0'},
            'paths': {
                '/users': {
                    'post': {
                        'summary': 'Create user',
                        'responses': {
                            '201': {'description': 'Created'}
                            # Missing 400, 500
                        }
                    }
                }
            }
        }
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(contract))

        violations = enforcer._verify_contract_completeness(contract_path)

        # Should warn about missing 400 for POST
        descriptions = [v.description for v in violations]
        assert any("400" in d and "POST" in d for d in descriptions)

    def test_missing_404_for_path_params(self, enforcer):
        """Test detection of missing 404 for endpoints with path parameters."""
        contract = {
            'openapi': '3.0.0',
            'info': {'title': 'Test', 'version': '1.0.0'},
            'paths': {
                '/users/{id}': {
                    'get': {
                        'summary': 'Get user',
                        'responses': {
                            '200': {'description': 'Success'}
                            # Missing 404
                        }
                    }
                }
            }
        }
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(contract))

        violations = enforcer._verify_contract_completeness(contract_path)

        # Should warn about missing 404
        descriptions = [v.description for v in violations]
        assert any("404" in d and "{id}" in d for d in descriptions)


class TestImplementationMatching:
    """Test implementation matching verification."""

    def test_matching_implementation_found(self, enforcer, sample_contract):
        """Test that matching endpoints are found."""
        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        # Create implementation with matching endpoints
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "routes.py").write_text("""
@app.get('/users')
def get_users():
    pass

@app.post('/users')
def create_user():
    pass

@app.get('/users/{id}')
def get_user(id):
    pass
""")

        violations = enforcer._verify_implementation_matches_contract(
            component_path, contract_path
        )

        # Should find all endpoints
        assert len(violations) == 0

    def test_missing_endpoint_detected(self, enforcer, sample_contract):
        """Test that missing endpoints are detected."""
        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        # Create implementation missing some endpoints
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()
        (component_path / "routes.py").write_text("""
@app.get('/users')
def get_users():
    pass
# Missing POST /users and GET /users/{id}
""")

        violations = enforcer._verify_implementation_matches_contract(
            component_path, contract_path
        )

        # Should detect missing endpoints
        assert len(violations) > 0
        descriptions = [v.description for v in violations]
        assert any("POST /users" in d for d in descriptions)


class TestSkeletonGeneration:
    """Test skeleton code generation."""

    def test_generate_skeleton_no_contract(self, enforcer):
        """Test skeleton generation when contract doesn't exist."""
        skeleton = enforcer.generate_implementation_skeleton("test-component")
        assert skeleton == ""

    def test_generate_fastapi_skeleton(self, enforcer, sample_contract):
        """Test FastAPI skeleton generation."""
        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        skeleton = enforcer.generate_implementation_skeleton("test-component", "fastapi")

        # Verify skeleton contains expected elements
        assert "from fastapi import APIRouter" in skeleton
        assert "router = APIRouter()" in skeleton
        assert "@router.get(\"/users\"" in skeleton
        assert "@router.post(\"/users\"" in skeleton
        assert "@router.get(\"/users/{id}\"" in skeleton
        assert "async def" in skeleton
        assert "NotImplementedError" in skeleton

    def test_generate_flask_skeleton(self, enforcer, sample_contract):
        """Test Flask skeleton generation."""
        # Create contract
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        skeleton = enforcer.generate_implementation_skeleton("test-component", "flask")

        # Verify skeleton contains expected elements
        assert "from flask import Blueprint" in skeleton
        assert "bp = Blueprint" in skeleton
        assert "@bp.route(\"/users\"" in skeleton
        assert "@bp.route(\"/users/<id>\"" in skeleton
        assert "methods=[" in skeleton

    def test_path_to_function_name_simple(self, enforcer):
        """Test path to function name conversion."""
        assert enforcer._path_to_function_name("/users", "get") == "get_user"
        assert enforcer._path_to_function_name("/users", "post") == "post_user"

    def test_path_to_function_name_with_id(self, enforcer):
        """Test path to function name with path parameters."""
        assert enforcer._path_to_function_name("/users/{id}", "get") == "get_user_by_id"
        assert enforcer._path_to_function_name("/posts/{post_id}", "delete") == "delete_post_by_post_id"

    def test_path_to_function_name_nested(self, enforcer):
        """Test path to function name with nested paths."""
        assert enforcer._path_to_function_name("/users/{id}/posts", "get") == "get_user_by_id_post"
        assert enforcer._path_to_function_name("/api/v1/resources", "get") == "get_api_v1_resource"


class TestEnforceAll:
    """Test enforcing all components."""

    def test_enforce_all_no_components(self, enforcer):
        """Test enforcing when no components exist."""
        results = enforcer.enforce_all_components()
        assert len(results) == 0

    def test_enforce_all_multiple_components(self, enforcer, sample_contract):
        """Test enforcing multiple components."""
        # Create component with contract (compliant)
        contract_path = enforcer.contracts_dir / "compliant-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))
        component_path = enforcer.components_dir / "compliant-component"
        component_path.mkdir()
        (component_path / "main.py").write_text("# implementation")

        # Create component without contract (non-compliant)
        component_path2 = enforcer.components_dir / "non-compliant-component"
        component_path2.mkdir()
        (component_path2 / "main.py").write_text("# implementation")

        results = enforcer.enforce_all_components()

        assert len(results) == 2
        assert "compliant-component" in results
        assert "non-compliant-component" in results

        # First should be compliant
        assert results["compliant-component"].has_contract

        # Second should not be compliant
        assert not results["non-compliant-component"].compliant
        assert not results["non-compliant-component"].has_contract


class TestReportGeneration:
    """Test report generation."""

    def test_generate_report_text_format(self, enforcer):
        """Test text format report generation."""
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

        assert "test-component" in report
        assert "Contract Exists:" in report
        assert "✅" in report
        assert "No violations found" in report

    def test_generate_report_json_format(self, enforcer):
        """Test JSON format report generation."""
        compliance = ContractCompliance(
            component_name="test-component",
            has_contract=True,
            contract_path=Path("/path/to/contract.yaml"),
            implementation_exists=False,
            implementation_path=None,
            compliant=True,
            violations=[]
        )

        report = enforcer.generate_report(compliance, format="json")

        data = json.loads(report)
        assert data["component_name"] == "test-component"
        assert data["has_contract"] is True
        assert data["implementation_exists"] is False
        assert data["compliant"] is True

    def test_generate_report_with_violations(self, enforcer):
        """Test report with violations."""
        violations = [
            EnforcementViolation(
                component_name="test-component",
                violation_type="missing_contract",
                description="Implementation without contract",
                severity="critical"
            ),
            EnforcementViolation(
                component_name="test-component",
                violation_type="missing_error_response",
                description="Missing 400 response",
                severity="warning"
            ),
            EnforcementViolation(
                component_name="test-component",
                violation_type="info",
                description="Consider adding rate limiting",
                severity="info"
            )
        ]

        compliance = ContractCompliance(
            component_name="test-component",
            has_contract=False,
            contract_path=None,
            implementation_exists=True,
            implementation_path=Path("/path/to/component"),
            compliant=False,
            violations=violations
        )

        report = enforcer.generate_report(compliance, format="text")

        assert "CRITICAL VIOLATIONS:" in report
        assert "WARNINGS:" in report
        assert "INFORMATIONAL:" in report
        assert "❌" in report
        assert "⚠️" in report
        assert "ℹ️" in report

    def test_generate_summary_report(self, enforcer, sample_contract):
        """Test summary report generation."""
        # Create mix of compliant and non-compliant components
        contract_path = enforcer.contracts_dir / "good-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))
        component_path = enforcer.components_dir / "good-component"
        component_path.mkdir()

        component_path2 = enforcer.components_dir / "bad-component"
        component_path2.mkdir()
        (component_path2 / "main.py").write_text("# code")

        results = enforcer.enforce_all_components()
        summary = enforcer.generate_summary_report(results)

        assert "CONTRACT ENFORCEMENT SUMMARY" in summary
        assert "Total Components:" in summary
        assert "Compliant:" in summary
        assert "Non-Compliant:" in summary
        assert "bad-component" in summary


class TestDataClasses:
    """Test data class functionality."""

    def test_enforcement_violation_to_dict(self):
        """Test EnforcementViolation to_dict."""
        violation = EnforcementViolation(
            component_name="test",
            violation_type="missing_contract",
            description="Test violation",
            severity="critical"
        )

        data = violation.to_dict()
        assert data["component_name"] == "test"
        assert data["violation_type"] == "missing_contract"
        assert data["description"] == "Test violation"
        assert data["severity"] == "critical"

    def test_contract_compliance_to_dict(self):
        """Test ContractCompliance to_dict."""
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
        assert data["component_name"] == "test"
        assert data["has_contract"] is True
        assert data["contract_path"] == "/path/to/contract"
        assert data["implementation_exists"] is False
        assert data["implementation_path"] is None
        assert data["compliant"] is True
        assert data["violations"] == []


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_nonexistent_component_directory(self, enforcer):
        """Test checking non-existent component."""
        result = enforcer.verify_component_compliance("nonexistent")

        assert result.component_name == "nonexistent"
        assert not result.has_contract
        assert not result.implementation_exists
        assert result.compliant

    def test_empty_contract_file(self, enforcer):
        """Test handling empty contract file."""
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text("")

        violations = enforcer._verify_contract_completeness(contract_path)

        # Should detect missing required sections
        assert len(violations) > 0

    def test_component_with_only_config_files(self, enforcer):
        """Test component with only configuration files."""
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()

        # Create only config files
        (component_path / "package.json").write_text("{}")
        (component_path / "tsconfig.json").write_text("{}")
        (component_path / "README.md").write_text("# Readme")

        assert not enforcer._has_implementation_files(component_path)

    def test_multiple_source_file_types(self, enforcer):
        """Test detection with multiple source file types."""
        component_path = enforcer.components_dir / "test-component"
        component_path.mkdir()

        # Mix of file types
        (component_path / "main.py").write_text("# python")
        (component_path / "index.js").write_text("// javascript")
        (component_path / "app.ts").write_text("// typescript")

        assert enforcer._has_implementation_files(component_path)

    def test_nested_source_files(self, enforcer):
        """Test detection of nested source files."""
        component_path = enforcer.components_dir / "test-component"
        src_path = component_path / "src" / "api"
        src_path.mkdir(parents=True)

        (src_path / "routes.py").write_text("# routes")

        assert enforcer._has_implementation_files(component_path)

    def test_generate_skeleton_unsupported_framework(self, enforcer, sample_contract):
        """Test skeleton generation for unsupported framework."""
        contract_path = enforcer.contracts_dir / "test-component_api.yaml"
        contract_path.write_text(yaml.dump(sample_contract))

        skeleton = enforcer.generate_implementation_skeleton("test-component", "ruby")

        assert "not yet implemented" in skeleton.lower()


if __name__ == '__main__':
    pytest.main([__file__, '-v', '--tb=short'])
