#!/usr/bin/env python3
"""
Runtime Validation Template

Provides decorators for runtime validation of data before persistence.

This is DEFENSE-IN-DEPTH - catches bugs at point of failure with clear messages.

This would have caught the file_hash bug:
- DataManager.save_results() called with results dict
- Runtime validator checks for required fields
- file_hash missing → raises ValueError immediately
- Bug caught during development, not production

Usage:
    from runtime_validation import validate_required_fields

    class DataManager:
        @validate_required_fields(['file_path', 'file_name', 'file_hash'])
        def save_results(self, results: Dict):
            self.db.insert(results)
"""

from functools import wraps
from typing import Dict, List, Any, Callable
import inspect


class ValidationError(Exception):
    """Raised when runtime validation fails."""
    pass


def validate_required_fields(required_fields: List[str]):
    """
    Decorator to validate dictionary has required fields.

    Args:
        required_fields: List of required field names

    Usage:
        @validate_required_fields(['id', 'name', 'email'])
        def save_user(self, user_dict: Dict):
            self.db.insert(user_dict)

    Raises:
        ValidationError: If any required field is missing or None
    """
    def decorator(func: Callable) -> Callable:
        @wraps(func)
        def wrapper(*args, **kwargs):
            # Get function signature
            sig = inspect.signature(func)
            bound = sig.bind(*args, **kwargs)
            bound.apply_defaults()

            # Find dictionary parameter (heuristic: first dict param)
            dict_param = None
            dict_param_name = None

            for param_name, param_value in bound.arguments.items():
                if isinstance(param_value, dict):
                    dict_param = param_value
                    dict_param_name = param_name
                    break

            if dict_param is None:
                raise ValidationError(
                    f"Function {func.__name__} has @validate_required_fields "
                    f"but no dictionary parameter found"
                )

            # Validate required fields
            missing_fields = []
            none_fields = []

            for field in required_fields:
                if field not in dict_param:
                    missing_fields.append(field)
                elif dict_param[field] is None:
                    none_fields.append(field)

            if missing_fields or none_fields:
                error_parts = []

                if missing_fields:
                    error_parts.append(
                        f"Missing required fields: {', '.join(missing_fields)}"
                    )

                if none_fields:
                    error_parts.append(
                        f"Fields cannot be None: {', '.join(none_fields)}"
                    )

                error_message = (
                    f"Validation failed in {func.__name__}():\n"
                    f"  {' | '.join(error_parts)}\n"
                    f"\n"
                    f"This indicates a bug in the calling code.\n"
                    f"The {dict_param_name} dictionary is missing required fields.\n"
                    f"\n"
                    f"Required fields: {required_fields}\n"
                    f"Received fields: {list(dict_param.keys())}\n"
                )

                raise ValidationError(error_message)

            # Call original function
            return func(*args, **kwargs)

        return wrapper
    return decorator


def validate_schema(schema: Dict[str, Dict[str, Any]]):
    """
    Decorator to validate dictionary against complete schema.

    Args:
        schema: Dict of {field_name: {
            'type': type,
            'required': bool,
            'not_null': bool
        }}

    Usage:
        @validate_schema({
            'file_path': {'type': str, 'required': True, 'not_null': True},
            'file_hash': {'type': str, 'required': True, 'not_null': True},
            'score': {'type': float, 'required': False}
        })
        def save_results(self, results: Dict):
            self.db.insert(results)

    This is the MOST COMPREHENSIVE validator.

    Raises:
        ValidationError: If schema validation fails
    """
    def decorator(func: Callable) -> Callable:
        @wraps(func)
        def wrapper(*args, **kwargs):
            # Get dictionary parameter
            sig = inspect.signature(func)
            bound = sig.bind(*args, **kwargs)
            bound.apply_defaults()

            dict_param = None
            for param_value in bound.arguments.values():
                if isinstance(param_value, dict):
                    dict_param = param_value
                    break

            if dict_param is None:
                raise ValidationError(
                    f"Function {func.__name__} has @validate_schema "
                    f"but no dictionary parameter found"
                )

            # Validate schema
            errors = []

            for field, field_schema in schema.items():
                required = field_schema.get('required', False)
                not_null = field_schema.get('not_null', False)
                expected_type = field_schema.get('type')

                # Check required
                if required and field not in dict_param:
                    errors.append(f"Missing required field: {field}")
                    continue

                if field in dict_param:
                    value = dict_param[field]

                    # Check not_null
                    if not_null and value is None:
                        errors.append(f"Field cannot be None: {field}")

                    # Check type
                    if value is not None and expected_type:
                        if not isinstance(value, expected_type):
                            errors.append(
                                f"{field}: expected {expected_type.__name__}, "
                                f"got {type(value).__name__}"
                            )

            if errors:
                error_message = (
                    f"Schema validation failed in {func.__name__}():\n" +
                    "\n".join(f"  - {err}" for err in errors) +
                    f"\n\nReceived fields: {list(dict_param.keys())}\n"
                )

                raise ValidationError(error_message)

            return func(*args, **kwargs)

        return wrapper
    return decorator


# Example usage:
"""
class DataManager:
    '''Data persistence with runtime validation.'''

    @validate_schema({
        'file_path': {'type': str, 'required': True, 'not_null': True},
        'file_name': {'type': str, 'required': True, 'not_null': True},
        'file_hash': {'type': str, 'required': True, 'not_null': True},
        'date_analyzed': {'type': str, 'required': True, 'not_null': True},
    })
    def save_results(self, results: Dict):
        '''Save with validation - would catch file_hash bug!'''
        self.db_manager.insert_result(results)


# When CLI calls with incomplete dict:
results = {
    'file_path': '/path/to/file.mp3',
    'file_name': 'file.mp3',
    # file_hash missing!
    'date_analyzed': '2025-11-20'
}

data_manager.save_results(results)
# ValidationError: Missing required field: file_hash
"""
