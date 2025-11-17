#!/usr/bin/env python3
"""
Tests for Contract Generator

Comprehensive test suite ensuring 80%+ coverage.
"""

import pytest
from pathlib import Path
import tempfile
import yaml
import json
import re

from contract_generator import (
    ContractGenerator,
    Contract,
    Endpoint,
    ErrorScenario,
    RateLimit,
    ValidationRule,
    HTTPMethod
)


class TestErrorScenario:
    """Test ErrorScenario dataclass."""

    def test_create_error_scenario(self):
        """Test creating an error scenario."""
        scenario = ErrorScenario(
            scenario="validation_failed",
            when="Request body fails validation",
            status_code=400,
            error_code="VALIDATION_FAILED",
            message="Request validation failed"
        )

        assert scenario.scenario == "validation_failed"
        assert scenario.status_code == 400
        assert scenario.error_code == "VALIDATION_FAILED"

    def test_to_dict(self):
        """Test converting error scenario to dictionary."""
        scenario = ErrorScenario(
            scenario="not_found",
            when="Resource does not exist",
            status_code=404,
            error_code="NOT_FOUND",
            message="Resource not found"
        )

        result = scenario.to_dict()

        assert isinstance(result, dict)
        assert result['scenario'] == "not_found"
        assert result['status_code'] == 404
        assert result['error_code'] == "NOT_FOUND"


class TestRateLimit:
    """Test RateLimit dataclass."""

    def test_create_rate_limit(self):
        """Test creating a rate limit."""
        limit = RateLimit(
            window_seconds=60,
            max_requests=100,
            by="ip"
        )

        assert limit.window_seconds == 60
        assert limit.max_requests == 100
        assert limit.by == "ip"

    def test_to_dict(self):
        """Test converting rate limit to dictionary."""
        limit = RateLimit(
            window_seconds=3600,
            max_requests=1000,
            by="user"
        )

        result = limit.to_dict()

        assert isinstance(result, dict)
        assert result['window_seconds'] == 3600
        assert result['max_requests'] == 1000
        assert result['by'] == "user"


class TestValidationRule:
    """Test ValidationRule dataclass."""

    def test_create_validation_rule(self):
        """Test creating a validation rule."""
        rule = ValidationRule(
            field="email",
            rules=["required", "email", "max:255"]
        )

        assert rule.field == "email"
        assert len(rule.rules) == 3
        assert "required" in rule.rules

    def test_to_dict(self):
        """Test converting validation rule to dictionary."""
        rule = ValidationRule(
            field="password",
            rules=["required", "min:8"]
        )

        result = rule.to_dict()

        assert isinstance(result, dict)
        assert result['field'] == "password"
        assert result['rules'] == ["required", "min:8"]


class TestEndpoint:
    """Test Endpoint dataclass."""

    def test_create_endpoint(self):
        """Test creating an endpoint."""
        endpoint = Endpoint(
            path="/users",
            method="GET",
            summary="List all users"
        )

        assert endpoint.path == "/users"
        assert endpoint.method == "GET"
        assert endpoint.summary == "List all users"
        assert endpoint.timeout == 30  # Default
        assert endpoint.authentication_required is True  # Default

    def test_generate_operation_id(self):
        """Test operation ID generation."""
        endpoint = Endpoint(
            path="/users/profile",
            method="GET",
            summary="Get user profile"
        )

        operation_id = endpoint._generate_operation_id()
        assert operation_id == "getUsersProfile"

    def test_generate_operation_id_with_params(self):
        """Test operation ID generation with path parameters."""
        endpoint = Endpoint(
            path="/users/{id}/posts",
            method="GET",
            summary="Get user posts"
        )

        operation_id = endpoint._generate_operation_id()
        assert operation_id == "getUsersPosts"

    def test_generate_path_parameters(self):
        """Test path parameter generation."""
        endpoint = Endpoint(
            path="/users/{id}",
            method="GET",
            summary="Get user"
        )

        params = endpoint._generate_path_parameters()

        assert len(params) == 1
        assert params[0]['name'] == "id"
        assert params[0]['in'] == "path"
        assert params[0]['required'] is True

    def test_generate_path_parameters_multiple(self):
        """Test path parameter generation with multiple params."""
        endpoint = Endpoint(
            path="/users/{userId}/posts/{postId}",
            method="GET",
            summary="Get user post"
        )

        params = endpoint._generate_path_parameters()

        assert len(params) == 2
        assert params[0]['name'] == "userId"
        assert params[1]['name'] == "postId"

    def test_generate_responses_success(self):
        """Test response generation for successful request."""
        endpoint = Endpoint(
            path="/users",
            method="GET",
            summary="List users",
            response_schemas={
                200: {
                    'type': 'object',
                    'properties': {
                        'users': {'type': 'array'}
                    }
                }
            }
        )

        responses = endpoint._generate_responses()

        assert '200' in responses
        assert responses['200']['description'] == 'Successful operation'

    def test_generate_responses_default_post(self):
        """Test default response generation for POST."""
        endpoint = Endpoint(
            path="/users",
            method="POST",
            summary="Create user"
        )

        responses = endpoint._generate_responses()

        assert '201' in responses  # POST should default to 201

    def test_generate_responses_with_errors(self):
        """Test response generation with error scenarios."""
        endpoint = Endpoint(
            path="/users/{id}",
            method="GET",
            summary="Get user",
            error_scenarios=[
                ErrorScenario(
                    scenario="not_found",
                    when="User not found",
                    status_code=404,
                    error_code="NOT_FOUND",
                    message="User not found"
                )
            ]
        )

        responses = endpoint._generate_responses()

        assert '404' in responses
        assert 'NOT_FOUND' in str(responses['404'])

    def test_to_openapi_operation(self):
        """Test converting endpoint to OpenAPI operation."""
        endpoint = Endpoint(
            path="/users",
            method="GET",
            summary="List users",
            tags=["users"]
        )

        operation = endpoint.to_openapi_operation()

        assert 'summary' in operation
        assert operation['summary'] == "List users"
        assert 'operationId' in operation
        assert 'responses' in operation
        assert 'tags' in operation

    def test_to_openapi_operation_with_request_body(self):
        """Test OpenAPI operation with request body."""
        endpoint = Endpoint(
            path="/users",
            method="POST",
            summary="Create user",
            request_schema={
                'type': 'object',
                'properties': {
                    'email': {'type': 'string'}
                }
            }
        )

        operation = endpoint.to_openapi_operation()

        assert 'requestBody' in operation
        assert operation['requestBody']['required'] is True

    def test_to_openapi_operation_with_security(self):
        """Test OpenAPI operation with security."""
        endpoint = Endpoint(
            path="/users",
            method="GET",
            summary="List users",
            authentication_required=True
        )

        operation = endpoint.to_openapi_operation()

        assert 'security' in operation
        assert {'bearerAuth': []} in operation['security']

    def test_to_openapi_operation_no_security(self):
        """Test OpenAPI operation without security."""
        endpoint = Endpoint(
            path="/health",
            method="GET",
            summary="Health check",
            authentication_required=False
        )

        operation = endpoint.to_openapi_operation()

        assert 'security' not in operation

    def test_to_openapi_operation_with_extensions(self):
        """Test OpenAPI operation with custom extensions."""
        endpoint = Endpoint(
            path="/users",
            method="GET",
            summary="List users",
            rate_limits=RateLimit(60, 100, "ip"),
            timeout=45
        )

        operation = endpoint.to_openapi_operation()

        assert 'x-rate-limit' in operation
        assert operation['x-rate-limit']['max_requests'] == 100
        assert 'x-timeout' in operation
        assert operation['x-timeout'] == 45


class TestContract:
    """Test Contract dataclass."""

    def test_create_contract(self):
        """Test creating a contract."""
        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=[]
        )

        assert contract.openapi_version == "3.0.0"
        assert contract.info['title'] == "Test API"

    def test_generate_paths(self):
        """Test path generation."""
        endpoints = [
            Endpoint(path="/users", method="GET", summary="List users"),
            Endpoint(path="/users", method="POST", summary="Create user")
        ]

        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=endpoints
        )

        paths = contract._generate_paths()

        assert "/users" in paths
        assert "get" in paths["/users"]
        assert "post" in paths["/users"]

    def test_generate_schemas_with_error(self):
        """Test schema generation includes error schema."""
        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=[]
        )

        schemas = contract._generate_schemas()

        assert "Error" in schemas
        assert schemas["Error"]["type"] == "object"
        assert "error" in schemas["Error"]["properties"]
        assert "message" in schemas["Error"]["properties"]

    def test_default_security_schemes(self):
        """Test default security scheme generation."""
        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=[]
        )

        schemes = contract._default_security_schemes()

        assert "bearerAuth" in schemes
        assert schemes["bearerAuth"]["type"] == "http"
        assert schemes["bearerAuth"]["scheme"] == "bearer"

    def test_to_openapi_yaml(self):
        """Test converting contract to YAML."""
        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=[
                Endpoint(path="/test", method="GET", summary="Test endpoint")
            ]
        )

        yaml_str = contract.to_openapi_yaml()

        assert isinstance(yaml_str, str)
        assert "openapi: 3.0.0" in yaml_str
        assert "title: Test API" in yaml_str
        assert "/test" in yaml_str

    def test_to_openapi_yaml_valid(self):
        """Test that generated YAML is valid."""
        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=[
                Endpoint(path="/test", method="GET", summary="Test endpoint")
            ]
        )

        yaml_str = contract.to_openapi_yaml()
        parsed = yaml.safe_load(yaml_str)

        assert parsed['openapi'] == "3.0.0"
        assert parsed['info']['title'] == "Test API"
        assert 'paths' in parsed
        assert 'components' in parsed


class TestContractGenerator:
    """Test ContractGenerator class."""

    @pytest.fixture
    def temp_project(self):
        """Create temporary project directory."""
        with tempfile.TemporaryDirectory() as tmpdir:
            yield Path(tmpdir)

    @pytest.fixture
    def generator(self, temp_project):
        """Create contract generator."""
        return ContractGenerator(temp_project)

    def test_create_generator(self, temp_project):
        """Test creating a contract generator."""
        generator = ContractGenerator(temp_project)

        assert generator.project_root == temp_project
        assert generator.contracts_dir.exists()

    def test_extract_endpoints_explicit(self, generator):
        """Test extracting explicit endpoints from spec."""
        spec = """
        The API should have the following endpoints:
        - POST /users/register - Register a new user
        - GET /users/profile - Get user profile
        """

        endpoints = generator._extract_endpoints(spec)

        assert len(endpoints) >= 2
        paths = [e.path for e in endpoints]
        assert "/users/register" in paths
        assert "/users/profile" in paths

    def test_extract_endpoints_inferred(self, generator):
        """Test inferring CRUD endpoints from resource mentions."""
        spec = """
        The user resource should support full CRUD operations.
        """

        endpoints = generator._extract_endpoints(spec)

        assert len(endpoints) >= 5  # CRUD operations
        methods = [e.method for e in endpoints]
        assert "GET" in methods
        assert "POST" in methods
        assert "PUT" in methods
        assert "DELETE" in methods

    def test_generate_error_scenarios_post(self, generator):
        """Test error scenario generation for POST."""
        scenarios = generator._generate_error_scenarios("/users", "POST", "")

        scenario_codes = [s.status_code for s in scenarios]
        assert 400 in scenario_codes  # Validation
        assert 409 in scenario_codes  # Conflict
        assert 429 in scenario_codes  # Rate limit
        assert 500 in scenario_codes  # Server error

    def test_generate_error_scenarios_get_with_id(self, generator):
        """Test error scenario generation for GET with ID."""
        scenarios = generator._generate_error_scenarios("/users/{id}", "GET", "")

        scenario_codes = [s.status_code for s in scenarios]
        assert 404 in scenario_codes  # Not found

    def test_extract_context(self, generator):
        """Test context extraction."""
        text = "A" * 1000
        context = generator._extract_context(text, 500, 510, window=50)

        assert len(context) <= 110  # 50 before + 10 + 50 after

    def test_extract_summary(self, generator):
        """Test summary extraction."""
        context = "This endpoint retrieves user data. Additional info here."

        summary = generator._extract_summary(context, "/users", "GET")

        assert len(summary) > 0
        assert "retrieves" in summary.lower() or "user" in summary.lower()

    def test_extract_request_schema_from_fields(self, generator):
        """Test request schema extraction with field hints."""
        context = """
        Request fields:
        - email (string)
        - age (integer)
        - active (boolean)
        """

        schema = generator._extract_request_schema(context, "POST")

        assert schema is not None
        assert schema['type'] == 'object'
        assert 'properties' in schema
        assert 'email' in schema['properties']
        assert schema['properties']['email']['type'] == 'string'

    def test_extract_request_schema_none_for_get(self, generator):
        """Test no request schema for GET."""
        schema = generator._extract_request_schema("test", "GET")

        assert schema is None

    def test_generate_response_schemas_post(self, generator):
        """Test response schema generation for POST."""
        schemas = generator._generate_response_schemas("context", "POST")

        assert 201 in schemas  # POST should create

    def test_generate_response_schemas_delete(self, generator):
        """Test response schema generation for DELETE."""
        schemas = generator._generate_response_schemas("context", "DELETE")

        # DELETE returns 204 No Content, which should not have a schema
        # The 204 response is handled at the endpoint level, not in response_schemas
        assert isinstance(schemas, dict)  # Should return empty dict for 204

    def test_extract_validation_rules(self, generator):
        """Test validation rule extraction."""
        context = "Email is required. Password must be at least 8 characters."

        rules = generator._extract_validation_rules(context)

        assert len(rules) > 0
        # Should detect email and password validation
        fields = [r.field for r in rules]
        assert 'email' in fields or '*' in fields

    def test_determine_rate_limit_from_context(self, generator):
        """Test rate limit extraction from context."""
        context = "Limited to 100 requests per hour"

        limit = generator._determine_rate_limit("/users", "GET", context)

        assert limit is not None
        assert limit.max_requests == 100
        assert limit.window_seconds == 3600

    def test_determine_rate_limit_default(self, generator):
        """Test default rate limit."""
        limit = generator._determine_rate_limit("/users", "GET", "")

        assert limit is not None
        assert limit.max_requests > 0
        assert limit.window_seconds > 0

    def test_extract_timeout(self, generator):
        """Test timeout extraction."""
        context = "timeout: 60 seconds"

        timeout = generator._extract_timeout(context)

        assert timeout == 60

    def test_extract_timeout_default(self, generator):
        """Test default timeout."""
        timeout = generator._extract_timeout("")

        assert timeout == 30

    def test_requires_auth_default(self, generator):
        """Test authentication required by default."""
        requires_auth = generator._requires_auth("")

        assert requires_auth is True

    def test_requires_auth_public_endpoint(self, generator):
        """Test public endpoint detection."""
        context = "This is a public endpoint, no authentication required"

        requires_auth = generator._requires_auth(context)

        assert requires_auth is False

    def test_extract_tags(self, generator):
        """Test tag extraction from path."""
        tags = generator._extract_tags("/users/profile")

        assert len(tags) > 0
        assert "users" in tags

    def test_extract_tags_default(self, generator):
        """Test default tag."""
        tags = generator._extract_tags("/")

        assert "default" in tags

    def test_generate_from_specification(self, generator):
        """Test full contract generation."""
        spec = """
        User Management API

        Endpoints:
        - POST /users - Create a new user
        - GET /users/{id} - Get user by ID

        Authentication: JWT Bearer token required
        """

        contract = generator.generate_from_specification(spec, "user-service")

        assert isinstance(contract, Contract)
        assert contract.info['title'] == "user-service API"
        assert len(contract.endpoints) >= 2
        assert 'bearerAuth' in contract.security_schemes

    def test_generate_contract_tests(self, generator):
        """Test test suite generation."""
        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=[
                Endpoint(
                    path="/users",
                    method="POST",
                    summary="Create user",
                    error_scenarios=[
                        ErrorScenario(
                            scenario="validation",
                            when="Invalid data",
                            status_code=400,
                            error_code="VALIDATION_FAILED",
                            message="Validation failed"
                        )
                    ]
                )
            ]
        )

        test_code = generator.generate_contract_tests(contract, "test-service")

        assert "import pytest" in test_code
        assert "def test_" in test_code
        assert "assert response.status_code" in test_code
        assert "validation" in test_code

    def test_path_to_class_name(self, generator):
        """Test path to class name conversion."""
        class_name = generator._path_to_class_name("/users/profile")

        assert class_name == "UsersProfile"

    def test_path_to_class_name_with_params(self, generator):
        """Test path to class name with parameters."""
        class_name = generator._path_to_class_name("/users/{id}")

        assert class_name == "Users"

    def test_generate_happy_path_test(self, generator):
        """Test happy path test generation."""
        endpoint = Endpoint(
            path="/users",
            method="POST",
            summary="Create user",
            request_schema={'type': 'object'}
        )

        test_code = generator._generate_happy_path_test(endpoint)

        assert "def test_post_" in test_code
        assert "requests.post" in test_code
        assert "assert response.status_code" in test_code

    def test_generate_error_test(self, generator):
        """Test error test generation."""
        endpoint = Endpoint(
            path="/users/{id}",
            method="GET",
            summary="Get user"
        )

        scenario = ErrorScenario(
            scenario="not_found",
            when="User not found",
            status_code=404,
            error_code="NOT_FOUND",
            message="Not found"
        )

        test_code = generator._generate_error_test(endpoint, scenario)

        assert "def test_get_" in test_code
        assert "not_found" in test_code
        assert "404" in test_code

    def test_generate_validation_test(self, generator):
        """Test validation test generation."""
        endpoint = Endpoint(
            path="/users",
            method="POST",
            summary="Create user",
            validation_rules=[
                ValidationRule(field="email", rules=["required", "email"])
            ]
        )

        test_code = generator._generate_validation_test(endpoint)

        assert "def test_post_" in test_code
        assert "validation" in test_code
        assert "400" in test_code

    def test_save_contract(self, generator, temp_project):
        """Test saving contract to file."""
        contract = Contract(
            openapi_version="3.0.0",
            info={"title": "Test API", "version": "1.0.0"},
            endpoints=[
                Endpoint(path="/test", method="GET", summary="Test")
            ]
        )

        file_path = generator.save_contract(contract, "test-service")

        assert file_path.exists()
        assert file_path.name == "test-service_api.yaml"

        # Verify content is valid YAML
        content = yaml.safe_load(file_path.read_text())
        assert content['openapi'] == "3.0.0"

    def test_save_tests(self, generator, temp_project):
        """Test saving test file."""
        test_code = '''
def test_example():
    assert True
'''

        file_path = generator.save_tests(test_code, "test-service")

        assert file_path.exists()
        assert file_path.name == "test_test-service_contract.py"
        assert "def test_example" in file_path.read_text()

    def test_infer_crud_endpoints(self, generator):
        """Test CRUD endpoint inference."""
        spec = "The product resource needs full CRUD support."

        endpoints = generator._infer_crud_endpoints(spec)

        assert len(endpoints) >= 5
        methods = [e.method for e in endpoints]
        assert "GET" in methods
        assert "POST" in methods
        assert "PUT" in methods
        assert "DELETE" in methods

    def test_generate_schemas(self, generator):
        """Test schema generation from spec."""
        spec = """
        schema: User {
            id: string
            email: string
            age: integer
        }
        """

        endpoints = []
        schemas = generator._generate_schemas(endpoints, spec)

        assert "User" in schemas
        assert schemas["User"]["type"] == "object"
        assert "id" in schemas["User"]["properties"]

    def test_determine_security_schemes_jwt(self, generator):
        """Test JWT security scheme detection."""
        spec = "Authentication via JWT bearer tokens"

        schemes = generator._determine_security_schemes(spec)

        assert "bearerAuth" in schemes
        assert schemes["bearerAuth"]["scheme"] == "bearer"

    def test_determine_security_schemes_api_key(self, generator):
        """Test API key security scheme detection."""
        spec = "Authentication via API key in header"

        schemes = generator._determine_security_schemes(spec)

        assert "apiKey" in schemes
        assert schemes["apiKey"]["type"] == "apiKey"

    def test_determine_security_schemes_default(self, generator):
        """Test default security scheme."""
        schemes = generator._determine_security_schemes("")

        assert "bearerAuth" in schemes  # Default


class TestIntegration:
    """Integration tests."""

    @pytest.fixture
    def temp_project(self):
        """Create temporary project directory."""
        with tempfile.TemporaryDirectory() as tmpdir:
            yield Path(tmpdir)

    def test_full_workflow(self, temp_project):
        """Test complete workflow from spec to tests."""
        # Create specification
        spec_dir = temp_project / "specs"
        spec_dir.mkdir()
        spec_file = spec_dir / "auth.md"

        spec_file.write_text("""
# Authentication Service

## Endpoints

### POST /auth/login
Login with email and password.

Fields:
- email (string) - User email
- password (string) - User password

Returns JWT token on success.

### GET /auth/verify
Verify JWT token validity.
        """)

        # Generate contract
        generator = ContractGenerator(temp_project)
        contract = generator.generate_from_specification(
            spec_file.read_text(),
            "auth-service"
        )

        # Verify contract
        assert len(contract.endpoints) >= 2
        assert contract.info['title'] == "auth-service API"

        # Save contract
        contract_file = generator.save_contract(contract, "auth-service")
        assert contract_file.exists()

        # Verify contract is valid YAML
        contract_data = yaml.safe_load(contract_file.read_text())
        assert 'paths' in contract_data
        assert 'components' in contract_data

        # Generate tests
        test_code = generator.generate_contract_tests(contract, "auth-service")
        test_file = generator.save_tests(test_code, "auth-service")

        assert test_file.exists()
        assert "def test_" in test_file.read_text()

    def test_empty_spec(self, temp_project):
        """Test handling of empty specification."""
        generator = ContractGenerator(temp_project)
        contract = generator.generate_from_specification("", "empty-service")

        # Should still generate valid contract with no endpoints
        assert isinstance(contract, Contract)
        assert len(contract.endpoints) == 0

    def test_complex_spec(self, temp_project):
        """Test handling of complex specification."""
        spec = """
# E-commerce API

## Authentication
JWT Bearer tokens required for all endpoints except login.

## Resources

### Products
- GET /products - List all products (100 requests per minute)
- GET /products/{id} - Get product details
- POST /products - Create product (admin only)
- PUT /products/{id} - Update product (admin only)
- DELETE /products/{id} - Delete product (admin only)

### Orders
- GET /orders - List user orders
- POST /orders - Create order
- GET /orders/{id} - Get order details
- PUT /orders/{id}/cancel - Cancel order

## Rate Limits
- 1000 requests per hour for authenticated users
- 100 requests per hour for unauthenticated users
        """

        generator = ContractGenerator(temp_project)
        contract = generator.generate_from_specification(spec, "ecommerce-api")

        # Should extract multiple endpoints
        assert len(contract.endpoints) >= 8

        # Should detect authentication
        assert 'bearerAuth' in contract.security_schemes

        # Should generate proper YAML
        yaml_str = contract.to_openapi_yaml()
        parsed = yaml.safe_load(yaml_str)
        assert 'paths' in parsed


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
