"""
Contract Analyzer

Low-level utilities for analyzing and comparing OpenAPI contract schemas.
Detects field mismatches, type mismatches, and missing required fields.

Classes:
    SchemaField: Represents a field in a schema
    SchemaComparison: Result of comparing two schemas

Functions:
    load_openapi_spec: Load and validate OpenAPI YAML file
    extract_schema: Extract schema definition from spec
    extract_fields: Extract fields from schema
    compare_schemas: Compare provider vs consumer schemas
    find_schema_references: Find schema references in endpoint

Usage:
    from contract_analyzer import load_openapi_spec, compare_schemas

    provider_spec = load_openapi_spec(Path('auth-api.yaml'))
    consumer_spec = load_openapi_spec(Path('user-api.yaml'))

    result = compare_schemas(provider_schema, consumer_schema)
    if result.has_issues:
        print(f"Found {len(result.field_mismatches)} field mismatches")
"""

import yaml
from pathlib import Path
from typing import Dict, List, Set
from dataclasses import dataclass


@dataclass
class SchemaField:
    """Represents a field in a schema."""
    name: str
    type: str
    required: bool = False

    def __eq__(self, other):
        if not isinstance(other, SchemaField):
            return False
        return (self.name == other.name and
                self.type == other.type and
                self.required == other.required)

    def __hash__(self):
        return hash((self.name, self.type, self.required))


class SchemaComparison:
    """Result of comparing two schemas."""
    def __init__(self):
        self.compatible = True
        self.field_mismatches: List[Dict] = []
        self.type_mismatches: List[Dict] = []
        self.missing_required: List[str] = []
        self.extra_fields: List[str] = []

    @property
    def has_issues(self) -> bool:
        """Check if comparison found any compatibility issues."""
        return bool(self.field_mismatches or self.type_mismatches or
                   self.missing_required)

    def to_dict(self) -> Dict:
        """Convert comparison to dictionary."""
        return {
            'compatible': self.compatible,
            'field_mismatches': self.field_mismatches,
            'type_mismatches': self.type_mismatches,
            'missing_required': self.missing_required,
            'extra_fields': self.extra_fields
        }


def load_openapi_spec(contract_path: Path) -> Dict:
    """
    Load and validate OpenAPI specification.

    Args:
        contract_path: Path to OpenAPI YAML file

    Returns:
        Dict containing parsed OpenAPI spec

    Raises:
        FileNotFoundError: If contract file doesn't exist
        yaml.YAMLError: If YAML is invalid
        ValueError: If not valid OpenAPI spec

    Example:
        >>> spec = load_openapi_spec(Path('contracts/auth-api.yaml'))
        >>> print(spec['info']['title'])
        'Auth Service API'
    """
    if not contract_path.exists():
        raise FileNotFoundError(f"Contract not found: {contract_path}")

    with open(contract_path, 'r') as f:
        spec = yaml.safe_load(f)

    # Basic validation
    if not isinstance(spec, dict):
        raise ValueError(f"Invalid OpenAPI spec: {contract_path}")

    if 'openapi' not in spec and 'swagger' not in spec:
        raise ValueError(f"Not an OpenAPI spec: {contract_path}")

    return spec


def extract_schema(spec: Dict, schema_ref: str) -> Dict:
    """
    Extract schema definition from OpenAPI spec.

    Args:
        spec: OpenAPI specification dict
        schema_ref: Schema reference like '#/components/schemas/User'

    Returns:
        Schema definition dict

    Raises:
        ValueError: If schema reference not found

    Example:
        >>> schema = extract_schema(spec, '#/components/schemas/User')
        >>> print(schema['type'])
        'object'
    """
    if not schema_ref.startswith('#/'):
        raise ValueError(f"Invalid schema reference: {schema_ref}")

    parts = schema_ref[2:].split('/')

    current = spec
    for part in parts:
        if part not in current:
            raise ValueError(f"Schema not found: {schema_ref}")
        current = current[part]

    return current


def extract_fields(schema: Dict) -> List[SchemaField]:
    """
    Extract fields from schema definition.

    Args:
        schema: Schema definition dict

    Returns:
        List of SchemaField objects

    Example:
        >>> schema = {'properties': {'id': {'type': 'string'}, 'name': {'type': 'string'}}, 'required': ['id']}
        >>> fields = extract_fields(schema)
        >>> assert len(fields) == 2
        >>> assert fields[0].required == True
    """
    if 'properties' not in schema:
        return []

    required_fields = set(schema.get('required', []))
    fields = []

    for field_name, field_def in schema['properties'].items():
        field_type = field_def.get('type', 'unknown')
        is_required = field_name in required_fields
        fields.append(SchemaField(field_name, field_type, is_required))

    return fields


def compare_schemas(provider_schema: Dict, consumer_schema: Dict) -> SchemaComparison:
    """
    Compare provider schema against consumer expectations.

    Checks for:
    - Missing required fields (consumer needs, provider doesn't have)
    - Field name case mismatches (userId vs user_id)
    - Type mismatches (string vs integer)

    Args:
        provider_schema: Schema from provider's contract
        consumer_schema: Schema from consumer's contract

    Returns:
        SchemaComparison with compatibility results

    Example:
        >>> provider = {'properties': {'user_id': {'type': 'string'}}, 'required': ['user_id']}
        >>> consumer = {'properties': {'userId': {'type': 'string'}}, 'required': ['userId']}
        >>> result = compare_schemas(provider, consumer)
        >>> assert not result.compatible
        >>> assert len(result.field_mismatches) == 1
    """
    result = SchemaComparison()

    provider_fields = {f.name: f for f in extract_fields(provider_schema)}
    consumer_fields = {f.name: f for f in extract_fields(consumer_schema)}

    # Check for missing required fields
    for field_name, field in consumer_fields.items():
        if field.required and field_name not in provider_fields:
            result.missing_required.append(field_name)
            result.compatible = False

    # Check for field name case mismatches (common error)
    provider_names_lower = {name.lower(): name for name in provider_fields.keys()}
    for consumer_name in consumer_fields.keys():
        if consumer_name not in provider_fields:
            # Check if case-insensitive match exists
            if consumer_name.lower() in provider_names_lower:
                provider_name = provider_names_lower[consumer_name.lower()]
                result.field_mismatches.append({
                    'consumer_expects': consumer_name,
                    'provider_has': provider_name,
                    'issue': 'case_mismatch'
                })
                result.compatible = False

    # Check for type mismatches on matching fields
    for field_name in set(provider_fields.keys()) & set(consumer_fields.keys()):
        provider_type = provider_fields[field_name].type
        consumer_type = consumer_fields[field_name].type

        if provider_type != consumer_type:
            result.type_mismatches.append({
                'field': field_name,
                'provider_type': provider_type,
                'consumer_type': consumer_type
            })
            result.compatible = False

    # Track extra fields (not an error, but informational)
    extra = set(provider_fields.keys()) - set(consumer_fields.keys())
    result.extra_fields = list(extra)

    return result


def find_schema_references(spec: Dict, endpoint_path: str, method: str) -> List[str]:
    """
    Find schema references used in endpoint response.

    Args:
        spec: OpenAPI specification
        endpoint_path: API path like '/login'
        method: HTTP method like 'post'

    Returns:
        List of schema references like ['#/components/schemas/AuthToken']

    Example:
        >>> refs = find_schema_references(spec, '/login', 'post')
        >>> assert '#/components/schemas/AuthResponse' in refs
    """
    refs = []

    try:
        endpoint = spec['paths'][endpoint_path][method.lower()]
        responses = endpoint.get('responses', {})

        for status_code, response in responses.items():
            if 'content' in response:
                for content_type, content_def in response['content'].items():
                    if 'schema' in content_def:
                        schema = content_def['schema']
                        if '$ref' in schema:
                            refs.append(schema['$ref'])
                        # Also check for array items
                        if schema.get('type') == 'array' and 'items' in schema:
                            if '$ref' in schema['items']:
                                refs.append(schema['items']['$ref'])
    except KeyError:
        pass

    return refs


def normalize_field_name(field_name: str) -> str:
    """
    Normalize field name for comparison (convert to snake_case).

    Args:
        field_name: Field name in any case format

    Returns:
        Normalized field name in snake_case

    Example:
        >>> normalize_field_name('userId')
        'user_id'
        >>> normalize_field_name('user_id')
        'user_id'
        >>> normalize_field_name('UserID')
        'user_id'
    """
    import re
    # Convert camelCase to snake_case
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', field_name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


def find_similar_fields(field_name: str, available_fields: List[str], threshold: float = 0.8) -> List[str]:
    """
    Find fields with similar names (for suggestions).

    Uses simple character-based similarity.

    Args:
        field_name: Field name to match
        available_fields: List of available field names
        threshold: Similarity threshold (0.0-1.0)

    Returns:
        List of similar field names

    Example:
        >>> similar = find_similar_fields('userId', ['user_id', 'user_name', 'email'])
        >>> assert 'user_id' in similar
    """
    similar = []
    normalized_target = normalize_field_name(field_name)

    for available in available_fields:
        normalized_available = normalize_field_name(available)

        # Check exact match after normalization
        if normalized_target == normalized_available:
            similar.append(available)
            continue

        # Check if one contains the other
        if normalized_target in normalized_available or normalized_available in normalized_target:
            similar.append(available)

    return similar
