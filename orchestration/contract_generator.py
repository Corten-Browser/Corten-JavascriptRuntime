#!/usr/bin/env python3
"""
Contract Generation and Enforcement System

Generates contracts from specifications BEFORE implementation.
Part of v0.4.0 quality enhancement system.

This module provides:
- Contract generation from natural language specifications
- Automatic error scenario generation
- Contract-based test suite generation
- Implementation validation against contracts
"""

from pathlib import Path
from typing import List, Dict, Optional, Any, Tuple
from dataclasses import dataclass, asdict, field
import yaml
import json
import re
import sys
from enum import Enum


class HTTPMethod(Enum):
    """HTTP methods."""
    GET = "get"
    POST = "post"
    PUT = "put"
    DELETE = "delete"
    PATCH = "patch"
    HEAD = "head"
    OPTIONS = "options"


@dataclass
class ErrorScenario:
    """Error scenario for an endpoint."""
    scenario: str
    when: str
    status_code: int
    error_code: str
    message: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'scenario': self.scenario,
            'when': self.when,
            'status_code': self.status_code,
            'error_code': self.error_code,
            'message': self.message
        }


@dataclass
class RateLimit:
    """Rate limiting configuration."""
    window_seconds: int
    max_requests: int
    by: str  # "ip", "user", "api_key"

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'window_seconds': self.window_seconds,
            'max_requests': self.max_requests,
            'by': self.by
        }


@dataclass
class ValidationRule:
    """Validation rule for a field."""
    field: str
    rules: List[str]

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'field': self.field,
            'rules': self.rules
        }


@dataclass
class Endpoint:
    """API endpoint specification."""
    path: str
    method: str
    summary: str
    description: str = ""
    request_schema: Optional[Dict] = None
    response_schemas: Dict[int, Dict] = field(default_factory=dict)
    error_scenarios: List[ErrorScenario] = field(default_factory=list)
    validation_rules: List[ValidationRule] = field(default_factory=list)
    rate_limits: Optional[RateLimit] = None
    timeout: int = 30
    authentication_required: bool = True
    tags: List[str] = field(default_factory=list)

    def to_openapi_operation(self) -> Dict[str, Any]:
        """Convert to OpenAPI operation object."""
        operation = {
            'summary': self.summary,
            'description': self.description or self.summary,
            'tags': self.tags or ['default'],
            'operationId': self._generate_operation_id(),
        }

        # Add parameters for path/query
        if '{' in self.path:
            operation['parameters'] = self._generate_path_parameters()

        # Add request body
        if self.request_schema and self.method.lower() in ['post', 'put', 'patch']:
            operation['requestBody'] = {
                'required': True,
                'content': {
                    'application/json': {
                        'schema': self.request_schema
                    }
                }
            }

        # Add responses
        operation['responses'] = self._generate_responses()

        # Add security
        if self.authentication_required:
            operation['security'] = [{'bearerAuth': []}]

        # Add custom extensions
        if self.rate_limits:
            operation['x-rate-limit'] = self.rate_limits.to_dict()

        if self.validation_rules:
            operation['x-validation-rules'] = [r.to_dict() for r in self.validation_rules]

        if self.error_scenarios:
            operation['x-error-scenarios'] = [e.to_dict() for e in self.error_scenarios]

        operation['x-timeout'] = self.timeout

        return operation

    def _generate_operation_id(self) -> str:
        """Generate operation ID."""
        path_parts = [p for p in self.path.split('/') if p and '{' not in p]
        return f"{self.method.lower()}{''.join(p.capitalize() for p in path_parts)}"

    def _generate_path_parameters(self) -> List[Dict[str, Any]]:
        """Generate path parameters."""
        params = []
        for match in re.finditer(r'\{(\w+)\}', self.path):
            param_name = match.group(1)
            params.append({
                'name': param_name,
                'in': 'path',
                'required': True,
                'schema': {
                    'type': 'string'
                },
                'description': f'{param_name.replace("_", " ").capitalize()}'
            })
        return params

    def _generate_responses(self) -> Dict[str, Any]:
        """Generate response definitions."""
        responses = {}

        # Success responses
        for status_code, schema in self.response_schemas.items():
            responses[str(status_code)] = {
                'description': self._get_status_description(status_code),
                'content': {
                    'application/json': {
                        'schema': schema
                    }
                }
            }

        # Add default success response if none defined
        if not self.response_schemas:
            success_code = 200 if self.method.upper() != 'POST' else 201
            responses[str(success_code)] = {
                'description': 'Successful operation',
                'content': {
                    'application/json': {
                        'schema': {
                            'type': 'object',
                            'properties': {
                                'success': {'type': 'boolean', 'example': True},
                                'data': {'type': 'object'}
                            }
                        }
                    }
                }
            }

        # Error responses from scenarios
        for scenario in self.error_scenarios:
            if str(scenario.status_code) not in responses:
                responses[str(scenario.status_code)] = {
                    'description': scenario.when,
                    'content': {
                        'application/json': {
                            'schema': {
                                '$ref': '#/components/schemas/Error'
                            },
                            'example': {
                                'error': scenario.error_code,
                                'message': scenario.message,
                                'status': scenario.status_code
                            }
                        }
                    }
                }

        return responses

    def _get_status_description(self, status_code: int) -> str:
        """Get standard description for status code."""
        descriptions = {
            200: 'Successful operation',
            201: 'Resource created successfully',
            204: 'No content',
            400: 'Bad request',
            401: 'Unauthorized',
            403: 'Forbidden',
            404: 'Resource not found',
            409: 'Conflict',
            422: 'Validation error',
            429: 'Too many requests',
            500: 'Internal server error',
            503: 'Service unavailable',
            504: 'Gateway timeout'
        }
        return descriptions.get(status_code, f'Status {status_code}')


@dataclass
class Contract:
    """Complete API contract."""
    openapi_version: str
    info: Dict[str, str]
    endpoints: List[Endpoint]
    schemas: Dict[str, Dict] = field(default_factory=dict)
    security_schemes: Dict[str, Dict] = field(default_factory=dict)

    def to_openapi_yaml(self) -> str:
        """Convert to OpenAPI 3.0 YAML."""
        openapi_dict = {
            'openapi': self.openapi_version,
            'info': self.info,
            'servers': [
                {
                    'url': 'http://localhost:8000',
                    'description': 'Development server'
                }
            ],
            'paths': self._generate_paths(),
            'components': {
                'schemas': self._generate_schemas(),
                'securitySchemes': self.security_schemes or self._default_security_schemes()
            }
        }

        return yaml.dump(openapi_dict, default_flow_style=False, sort_keys=False)

    def _generate_paths(self) -> Dict[str, Dict[str, Any]]:
        """Generate paths object."""
        paths = {}
        for endpoint in self.endpoints:
            if endpoint.path not in paths:
                paths[endpoint.path] = {}
            paths[endpoint.path][endpoint.method.lower()] = endpoint.to_openapi_operation()
        return paths

    def _generate_schemas(self) -> Dict[str, Dict]:
        """Generate schemas with standard error schema."""
        schemas = dict(self.schemas)

        # Add standard error schema
        schemas['Error'] = {
            'type': 'object',
            'required': ['error', 'message', 'status'],
            'properties': {
                'error': {
                    'type': 'string',
                    'description': 'Error code',
                    'example': 'VALIDATION_FAILED'
                },
                'message': {
                    'type': 'string',
                    'description': 'Human-readable error message',
                    'example': 'Request validation failed'
                },
                'status': {
                    'type': 'integer',
                    'description': 'HTTP status code',
                    'example': 400
                },
                'details': {
                    'type': 'object',
                    'description': 'Additional error details',
                    'additionalProperties': True
                },
                'timestamp': {
                    'type': 'string',
                    'format': 'date-time',
                    'description': 'Error timestamp'
                },
                'request_id': {
                    'type': 'string',
                    'description': 'Request identifier for tracing'
                }
            }
        }

        return schemas

    def _default_security_schemes(self) -> Dict[str, Dict]:
        """Generate default security schemes."""
        return {
            'bearerAuth': {
                'type': 'http',
                'scheme': 'bearer',
                'bearerFormat': 'JWT',
                'description': 'JWT token authentication'
            }
        }


class ContractGenerator:
    """Generates contracts from specifications."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self.contracts_dir = self.project_root / "contracts"
        self.contracts_dir.mkdir(exist_ok=True)

    def generate_from_specification(self, spec_text: str, component_name: str) -> Contract:
        """
        Generate complete OpenAPI contract from specification.

        Extracts:
        - All endpoints mentioned
        - Request/response formats
        - Error scenarios (or generates standard ones)
        - Validation rules (or generates from types)
        - Rate limits (or uses defaults)
        - Authentication requirements

        Args:
            spec_text: Natural language specification
            component_name: Name of the component

        Returns:
            Complete Contract object
        """
        endpoints = self._extract_endpoints(spec_text)
        schemas = self._generate_schemas(endpoints, spec_text)
        security = self._determine_security_schemes(spec_text)

        contract = Contract(
            openapi_version="3.0.0",
            info={
                "title": f"{component_name} API",
                "version": "1.0.0",
                "description": f"Auto-generated contract for {component_name}\n\n{self._extract_description(spec_text)}"
            },
            endpoints=endpoints,
            schemas=schemas,
            security_schemes=security
        )

        return contract

    def _extract_endpoints(self, spec_text: str) -> List[Endpoint]:
        """
        Extract endpoints from specification.

        Look for patterns:
        - "POST /users/register"
        - "GET endpoint for user profile"
        - "The registration API should..."
        """
        endpoints = []

        # Pattern matching for API endpoints
        endpoint_pattern = r'(GET|POST|PUT|DELETE|PATCH)\s+(/[\w/{}:\-]+)'

        for match in re.finditer(endpoint_pattern, spec_text, re.IGNORECASE):
            method = match.group(1).upper()
            path = match.group(2)

            # Extract context around endpoint
            context = self._extract_context(spec_text, match.start(), match.end())

            endpoint = Endpoint(
                path=path,
                method=method,
                summary=self._extract_summary(context, path, method),
                description=self._extract_description(context),
                request_schema=self._extract_request_schema(context, method),
                response_schemas=self._generate_response_schemas(context, method),
                error_scenarios=self._generate_error_scenarios(path, method, context),
                validation_rules=self._extract_validation_rules(context),
                rate_limits=self._determine_rate_limit(path, method, context),
                timeout=self._extract_timeout(context),
                authentication_required=self._requires_auth(context),
                tags=self._extract_tags(path)
            )

            endpoints.append(endpoint)

        # If no explicit endpoints found, try to infer from resource mentions
        if not endpoints:
            endpoints = self._infer_crud_endpoints(spec_text)

        return endpoints

    def _infer_crud_endpoints(self, spec_text: str) -> List[Endpoint]:
        """Infer CRUD endpoints from resource mentions."""
        endpoints = []

        # Look for resource mentions like "user resource", "product API"
        resource_pattern = r'\b(\w+)\s+(?:resource|API|endpoint|service)\b'
        resources = set()

        for match in re.finditer(resource_pattern, spec_text, re.IGNORECASE):
            resources.add(match.group(1).lower())

        # Generate standard CRUD for each resource
        for resource in resources:
            base_path = f"/{resource}s"

            endpoints.extend([
                Endpoint(
                    path=base_path,
                    method='GET',
                    summary=f'List all {resource}s',
                    error_scenarios=self._generate_error_scenarios(base_path, 'GET', ''),
                    tags=[resource]
                ),
                Endpoint(
                    path=f"{base_path}/{{id}}",
                    method='GET',
                    summary=f'Get {resource} by ID',
                    error_scenarios=self._generate_error_scenarios(f"{base_path}/{{id}}", 'GET', ''),
                    tags=[resource]
                ),
                Endpoint(
                    path=base_path,
                    method='POST',
                    summary=f'Create new {resource}',
                    error_scenarios=self._generate_error_scenarios(base_path, 'POST', ''),
                    tags=[resource]
                ),
                Endpoint(
                    path=f"{base_path}/{{id}}",
                    method='PUT',
                    summary=f'Update {resource}',
                    error_scenarios=self._generate_error_scenarios(f"{base_path}/{{id}}", 'PUT', ''),
                    tags=[resource]
                ),
                Endpoint(
                    path=f"{base_path}/{{id}}",
                    method='DELETE',
                    summary=f'Delete {resource}',
                    error_scenarios=self._generate_error_scenarios(f"{base_path}/{{id}}", 'DELETE', ''),
                    tags=[resource]
                )
            ])

        return endpoints

    def _generate_error_scenarios(self, path: str, method: str, context: str) -> List[ErrorScenario]:
        """
        Generate standard error scenarios for endpoint.

        Every endpoint gets:
        - 400 VALIDATION_FAILED (if POST/PUT/PATCH)
        - 401 UNAUTHORIZED (if auth required)
        - 404 NOT_FOUND (if GET by ID)
        - 409 CONFLICT (if POST with unique constraint)
        - 429 RATE_LIMIT_EXCEEDED
        - 500 INTERNAL_ERROR
        - 503 SERVICE_UNAVAILABLE
        - 504 TIMEOUT
        """
        scenarios = []

        # Standard error scenarios based on method
        if method.upper() in ['POST', 'PUT', 'PATCH']:
            scenarios.append(ErrorScenario(
                scenario="validation_failed",
                when="Request body fails validation",
                status_code=400,
                error_code="VALIDATION_FAILED",
                message="Request validation failed"
            ))

        if '{id}' in path or '{' in path:
            scenarios.append(ErrorScenario(
                scenario="not_found",
                when="Resource does not exist",
                status_code=404,
                error_code="NOT_FOUND",
                message="Resource not found"
            ))

        if method.upper() == 'POST':
            scenarios.append(ErrorScenario(
                scenario="conflict",
                when="Resource already exists",
                status_code=409,
                error_code="CONFLICT",
                message="Resource already exists"
            ))

        # Universal scenarios
        scenarios.extend([
            ErrorScenario(
                scenario="unauthorized",
                when="Authentication token is missing or invalid",
                status_code=401,
                error_code="UNAUTHORIZED",
                message="Authentication required"
            ),
            ErrorScenario(
                scenario="rate_limit",
                when="Rate limit exceeded",
                status_code=429,
                error_code="RATE_LIMIT_EXCEEDED",
                message="Too many requests"
            ),
            ErrorScenario(
                scenario="timeout",
                when="Request timeout",
                status_code=504,
                error_code="TIMEOUT",
                message="Request timeout"
            ),
            ErrorScenario(
                scenario="server_error",
                when="Internal server error",
                status_code=500,
                error_code="INTERNAL_ERROR",
                message="Internal server error"
            ),
            ErrorScenario(
                scenario="service_unavailable",
                when="Service is temporarily unavailable",
                status_code=503,
                error_code="SERVICE_UNAVAILABLE",
                message="Service unavailable"
            )
        ])

        return scenarios

    def _extract_context(self, text: str, start: int, end: int, window: int = 300) -> str:
        """Extract text around a position."""
        context_start = max(0, start - window)
        context_end = min(len(text), end + window)
        return text[context_start:context_end]

    def _extract_summary(self, context: str, path: str, method: str) -> str:
        """Extract endpoint summary from context."""
        # Look for sentences containing the endpoint
        sentences = re.split(r'[.!?]\s+', context)
        for sentence in sentences:
            if path in sentence or method.lower() in sentence.lower():
                return sentence.strip()

        # Default summary
        resource = path.split('/')[-1].replace('{', '').replace('}', '')
        return f"{method.upper()} {resource}"

    def _extract_description(self, context: str) -> str:
        """Extract detailed description from context."""
        # Take first paragraph or sentence
        paragraphs = context.split('\n\n')
        if paragraphs:
            return paragraphs[0].strip()
        return ""

    def _extract_request_schema(self, context: str, method: str) -> Optional[Dict]:
        """Extract request schema from context."""
        if method.upper() not in ['POST', 'PUT', 'PATCH']:
            return None

        # Look for field mentions
        fields = {}

        # Pattern: "field_name (type)"
        field_pattern = r'(\w+)\s*\((\w+)\)'
        for match in re.finditer(field_pattern, context):
            field_name = match.group(1)
            field_type = match.group(2).lower()

            schema_type = 'string'
            if field_type in ['int', 'integer', 'number']:
                schema_type = 'integer'
            elif field_type in ['float', 'double']:
                schema_type = 'number'
            elif field_type in ['bool', 'boolean']:
                schema_type = 'boolean'

            fields[field_name] = {'type': schema_type}

        if fields:
            return {
                'type': 'object',
                'properties': fields,
                'required': list(fields.keys())
            }

        # Default generic schema
        return {
            'type': 'object',
            'properties': {
                'data': {
                    'type': 'object',
                    'description': 'Request payload'
                }
            }
        }

    def _generate_response_schemas(self, context: str, method: str) -> Dict[int, Dict]:
        """Generate response schemas."""
        schemas = {}

        success_code = 200
        if method.upper() == 'POST':
            success_code = 201
        elif method.upper() == 'DELETE':
            success_code = 204

        if success_code != 204:
            schemas[success_code] = {
                'type': 'object',
                'properties': {
                    'success': {'type': 'boolean', 'example': True},
                    'data': {'type': 'object', 'description': 'Response data'}
                }
            }

        return schemas

    def _extract_validation_rules(self, context: str) -> List[ValidationRule]:
        """Extract validation rules from context."""
        rules = []

        # Look for validation keywords
        if 'required' in context.lower():
            rules.append(ValidationRule(
                field='*',
                rules=['required']
            ))

        if 'email' in context.lower():
            rules.append(ValidationRule(
                field='email',
                rules=['email', 'required']
            ))

        if 'password' in context.lower():
            rules.append(ValidationRule(
                field='password',
                rules=['min:8', 'required']
            ))

        return rules

    def _determine_rate_limit(self, path: str, method: str, context: str) -> Optional[RateLimit]:
        """Determine rate limit for endpoint."""
        # Check for rate limit mentions in context
        rate_pattern = r'(\d+)\s+requests?\s+per\s+(\w+)'
        match = re.search(rate_pattern, context, re.IGNORECASE)

        if match:
            max_requests = int(match.group(1))
            unit = match.group(2).lower()

            window_seconds = 60
            if 'hour' in unit:
                window_seconds = 3600
            elif 'day' in unit:
                window_seconds = 86400

            return RateLimit(
                window_seconds=window_seconds,
                max_requests=max_requests,
                by='ip'
            )

        # Default rate limits based on method
        if method.upper() in ['POST', 'PUT', 'DELETE']:
            return RateLimit(window_seconds=60, max_requests=10, by='ip')
        else:
            return RateLimit(window_seconds=60, max_requests=100, by='ip')

    def _extract_timeout(self, context: str) -> int:
        """Extract timeout from context."""
        timeout_pattern = r'timeout[:\s]+(\d+)'
        match = re.search(timeout_pattern, context, re.IGNORECASE)

        if match:
            return int(match.group(1))

        return 30  # Default

    def _requires_auth(self, context: str) -> bool:
        """Determine if endpoint requires authentication."""
        # Look for auth-related keywords
        no_auth_keywords = ['public', 'no auth', 'unauthenticated', 'anonymous']

        for keyword in no_auth_keywords:
            if keyword in context.lower():
                return False

        return True  # Default to requiring auth

    def _extract_tags(self, path: str) -> List[str]:
        """Extract tags from path."""
        parts = [p for p in path.split('/') if p and '{' not in p]
        if parts:
            return [parts[0]]
        return ['default']

    def _generate_schemas(self, endpoints: List[Endpoint], spec_text: str) -> Dict[str, Dict]:
        """Generate reusable schemas."""
        schemas = {}

        # Extract schema definitions from spec
        # Look for "schema:" or "model:" sections
        schema_pattern = r'(?:schema|model):\s*(\w+)\s*\{([^}]+)\}'

        for match in re.finditer(schema_pattern, spec_text, re.IGNORECASE):
            schema_name = match.group(1)
            fields_text = match.group(2)

            properties = {}
            for field_match in re.finditer(r'(\w+)\s*:\s*(\w+)', fields_text):
                field_name = field_match.group(1)
                field_type = field_match.group(2).lower()

                schema_type = 'string'
                if field_type in ['int', 'integer']:
                    schema_type = 'integer'
                elif field_type in ['float', 'number']:
                    schema_type = 'number'
                elif field_type in ['bool', 'boolean']:
                    schema_type = 'boolean'

                properties[field_name] = {'type': schema_type}

            if properties:
                schemas[schema_name] = {
                    'type': 'object',
                    'properties': properties
                }

        return schemas

    def _determine_security_schemes(self, spec_text: str) -> Dict[str, Dict]:
        """Determine security schemes from specification."""
        schemes = {}

        if 'jwt' in spec_text.lower() or 'bearer' in spec_text.lower():
            schemes['bearerAuth'] = {
                'type': 'http',
                'scheme': 'bearer',
                'bearerFormat': 'JWT'
            }

        if 'api key' in spec_text.lower() or 'apikey' in spec_text.lower():
            schemes['apiKey'] = {
                'type': 'apiKey',
                'in': 'header',
                'name': 'X-API-Key'
            }

        if 'oauth' in spec_text.lower():
            schemes['oauth2'] = {
                'type': 'oauth2',
                'flows': {
                    'authorizationCode': {
                        'authorizationUrl': 'https://example.com/oauth/authorize',
                        'tokenUrl': 'https://example.com/oauth/token',
                        'scopes': {
                            'read': 'Read access',
                            'write': 'Write access'
                        }
                    }
                }
            }

        # Default to bearer auth
        if not schemes:
            schemes['bearerAuth'] = {
                'type': 'http',
                'scheme': 'bearer',
                'bearerFormat': 'JWT'
            }

        return schemes

    def generate_contract_tests(self, contract: Contract, component_name: str) -> str:
        """
        Auto-generate comprehensive test suite from contract.

        Generates tests for:
        - Happy path (200 responses)
        - Each error scenario
        - Validation rules
        - Rate limits
        - Authentication
        """
        test_code = f'''#!/usr/bin/env python3
"""
Auto-generated contract tests for {component_name}

Generated from contract. DO NOT EDIT MANUALLY.
Regenerate with: python orchestration/contract_generator.py generate-tests

These tests verify that the implementation matches the contract exactly.
"""

import pytest
import requests
from typing import Dict, Any
import time

# Test configuration
BASE_URL = "http://localhost:8000"

class TestConfig:
    """Test configuration."""
    base_url = BASE_URL
    timeout = 30
    auth_token = None

    @classmethod
    def set_auth_token(cls, token: str):
        """Set authentication token for tests."""
        cls.auth_token = token

    @classmethod
    def get_headers(cls) -> Dict[str, str]:
        """Get headers with authentication."""
        headers = {{"Content-Type": "application/json"}}
        if cls.auth_token:
            headers["Authorization"] = f"Bearer {{cls.auth_token}}"
        return headers


@pytest.fixture(scope="session")
def api_client():
    """Create API client for tests."""
    # TODO: Implement authentication if needed
    # token = authenticate()
    # TestConfig.set_auth_token(token)
    return TestConfig


'''

        # Group endpoints by path for organization
        paths = {}
        for endpoint in contract.endpoints:
            if endpoint.path not in paths:
                paths[endpoint.path] = []
            paths[endpoint.path].append(endpoint)

        # Generate test class for each path
        for path, endpoints in paths.items():
            class_name = self._path_to_class_name(path)
            test_code += f'\nclass Test{class_name}:\n'
            test_code += f'    """Tests for {path}"""\n\n'

            for endpoint in endpoints:
                # Generate happy path test
                test_code += self._generate_happy_path_test(endpoint)
                test_code += '\n'

                # Generate error scenario tests
                for scenario in endpoint.error_scenarios:
                    test_code += self._generate_error_test(endpoint, scenario)
                    test_code += '\n'

                # Generate validation tests
                if endpoint.validation_rules:
                    test_code += self._generate_validation_test(endpoint)
                    test_code += '\n'

        return test_code

    def _path_to_class_name(self, path: str) -> str:
        """Convert path to class name."""
        parts = [p for p in path.split('/') if p and '{' not in p]
        return ''.join(p.capitalize() for p in parts) or 'Root'

    def _generate_happy_path_test(self, endpoint: Endpoint) -> str:
        """Generate test code for happy path."""
        method_lower = endpoint.method.lower()
        test_name = f"test_{method_lower}_{endpoint._generate_operation_id()}_success"

        # Generate request data
        request_data = ''
        if endpoint.request_schema and method_lower in ['post', 'put', 'patch']:
            request_data = '''
        data = {
            # TODO: Add valid request data based on schema
            "example": "data"
        }'''

        # Generate path with example values
        test_path = endpoint.path
        path_params = re.findall(r'\{(\w+)\}', test_path)
        for param in path_params:
            test_path = test_path.replace(f'{{{param}}}', f'{{test_{param}}}')
            request_data += f'\n        test_{param} = "test-id-123"'

        expected_status = 200
        if method_lower == 'post':
            expected_status = 201
        elif method_lower == 'delete':
            expected_status = 204

        # Build request parameters
        json_param = "json=data," if request_data else ""

        # Build assertion section
        if expected_status == 204:
            assertion_section = "# No content expected for DELETE"
        else:
            assertion_section = '''if response.content:
            result = response.json()
            # TODO: Add assertions for response schema
            assert "success" in result or "data" in result
'''

        return f'''    def {test_name}(self, api_client):
        """Test successful {endpoint.method} {endpoint.path}"""
        {request_data}

        url = f"{{{{api_client.base_url}}}}{test_path}"
        response = requests.{method_lower}(
            url,
            {json_param}
            headers=api_client.get_headers(),
            timeout=api_client.timeout
        )

        assert response.status_code == {expected_status}, f"Expected {expected_status}, got {{{{response.status_code}}}}: {{{{response.text}}}}"

        {assertion_section}'''

    def _generate_error_test(self, endpoint: Endpoint, scenario: ErrorScenario) -> str:
        """Generate test code for error scenario."""
        method_lower = endpoint.method.lower()
        test_name = f"test_{method_lower}_{endpoint._generate_operation_id()}_{scenario.scenario}"

        return f'''    def {test_name}(self, api_client):
        """Test {scenario.scenario}: {scenario.when}"""
        # TODO: Set up conditions for: {scenario.when}

        url = f"{{api_client.base_url}}{endpoint.path.replace('{id}', 'invalid-id')}"
        response = requests.{method_lower}(
            url,
            headers=api_client.get_headers(),
            timeout=api_client.timeout
        )

        assert response.status_code == {scenario.status_code}
        result = response.json()
        assert result.get("error") == "{scenario.error_code}"
'''

    def _generate_validation_test(self, endpoint: Endpoint) -> str:
        """Generate test for validation rules."""
        method_lower = endpoint.method.lower()
        test_name = f"test_{method_lower}_{endpoint._generate_operation_id()}_validation"

        return f'''    def {test_name}(self, api_client):
        """Test validation rules for {endpoint.path}"""
        # Test with invalid data
        invalid_data = {{}}  # Empty data should fail validation

        url = f"{{api_client.base_url}}{endpoint.path}"
        response = requests.{method_lower}(
            url,
            json=invalid_data,
            headers=api_client.get_headers(),
            timeout=api_client.timeout
        )

        assert response.status_code == 400
        result = response.json()
        assert result.get("error") == "VALIDATION_FAILED"
'''

    def save_contract(self, contract: Contract, component_name: str) -> Path:
        """Save contract to contracts directory."""
        contract_file = self.contracts_dir / f"{component_name}_api.yaml"
        contract_file.write_text(contract.to_openapi_yaml())
        return contract_file

    def save_tests(self, test_code: str, component_name: str) -> Path:
        """Save generated tests."""
        tests_dir = self.project_root / "tests" / "contract_tests"
        tests_dir.mkdir(parents=True, exist_ok=True)

        test_file = tests_dir / f"test_{component_name}_contract.py"
        test_file.write_text(test_code)
        return test_file


def main():
    """CLI interface."""
    if len(sys.argv) < 2:
        print("Usage: contract_generator.py <command> [args]")
        print("\nCommands:")
        print("  generate <spec_file> <component_name>  - Generate contract from spec")
        print("  generate-tests <contract_file>         - Generate tests from contract")
        print("  validate <component_path> <contract>   - Validate implementation")
        print("\nExample:")
        print("  python orchestration/contract_generator.py generate specs/auth.md auth-service")
        sys.exit(1)

    command = sys.argv[1]

    if command == "generate":
        if len(sys.argv) < 4:
            print("Error: generate requires <spec_file> <component_name>")
            sys.exit(1)

        spec_file = Path(sys.argv[2])
        component_name = sys.argv[3]

        if not spec_file.exists():
            print(f"Error: Specification file not found: {spec_file}")
            sys.exit(1)

        spec_text = spec_file.read_text()
        generator = ContractGenerator(Path.cwd())
        contract = generator.generate_from_specification(spec_text, component_name)

        # Save contract
        contract_file = generator.save_contract(contract, component_name)
        print(f"Generated contract: {contract_file}")

        # Generate tests
        test_code = generator.generate_contract_tests(contract, component_name)
        test_file = generator.save_tests(test_code, component_name)
        print(f"Generated tests: {test_file}")

        print("\nNext steps:")
        print(f"1. Review the contract: {contract_file}")
        print(f"2. Review the tests: {test_file}")
        print(f"3. Implement the component to match the contract")
        print(f"4. Run tests: pytest {test_file}")

    elif command == "generate-tests":
        if len(sys.argv) < 3:
            print("Error: generate-tests requires <contract_file>")
            sys.exit(1)

        contract_file = Path(sys.argv[2])
        if not contract_file.exists():
            print(f"Error: Contract file not found: {contract_file}")
            sys.exit(1)

        # Load contract and generate tests
        contract_data = yaml.safe_load(contract_file.read_text())
        component_name = contract_data['info']['title'].split()[0].lower()

        # TODO: Parse contract back into Contract object
        print("TODO: Implement contract parsing")

    elif command == "validate":
        print("TODO: Implement validation")

    else:
        print(f"Error: Unknown command: {command}")
        sys.exit(1)


if __name__ == '__main__':
    main()
