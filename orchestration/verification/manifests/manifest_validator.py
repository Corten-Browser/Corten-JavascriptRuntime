#!/usr/bin/env python3
"""
Manifest Validator

Validates component manifest schema (component.yaml).
Supports manifest v2.0 with user_facing_features.

Part of v0.13.0: Feature Coverage Testing
"""

from pathlib import Path
from typing import Dict, List, Optional
import yaml


class ManifestValidator:
    """Validate component manifest schema."""

    REQUIRED_FIELDS = ['name', 'type', 'version']

    VALID_TYPES = [
        'cli_application',
        'library',
        'web_server',
        'gui_application',
        'generic'
    ]

    USER_FACING_TYPES = ['cli_application', 'library', 'web_server', 'gui_application']

    def __init__(self):
        """Initialize validator."""
        pass

    def validate(self, manifest_path: Path) -> Dict:
        """
        Validate manifest schema.

        Args:
            manifest_path: Path to component.yaml

        Returns:
            {
                'valid': bool,
                'errors': List[str],
                'warnings': List[str]
            }
        """
        result = {
            'valid': True,
            'errors': [],
            'warnings': []
        }

        try:
            with open(manifest_path) as f:
                manifest = yaml.safe_load(f)
        except FileNotFoundError:
            result['valid'] = False
            result['errors'].append(f"Manifest not found: {manifest_path}")
            return result
        except yaml.YAMLError as e:
            result['valid'] = False
            result['errors'].append(f"Cannot parse YAML: {str(e)}")
            return result
        except Exception as e:
            result['valid'] = False
            result['errors'].append(f"Cannot read manifest: {str(e)}")
            return result

        if manifest is None:
            result['valid'] = False
            result['errors'].append("Manifest is empty")
            return result

        # Check required fields
        for field in self.REQUIRED_FIELDS:
            if field not in manifest:
                result['valid'] = False
                result['errors'].append(f"Missing required field: {field}")

        # Check type validity
        if 'type' in manifest:
            if manifest['type'] not in self.VALID_TYPES:
                result['valid'] = False
                result['errors'].append(
                    f"Invalid type: {manifest['type']}. "
                    f"Valid types: {', '.join(self.VALID_TYPES)}"
                )

        # Validate user_facing_features if present
        if 'user_facing_features' in manifest:
            feature_errors, feature_warnings = self._validate_features(
                manifest['user_facing_features'],
                manifest.get('type')
            )
            result['errors'].extend(feature_errors)
            result['warnings'].extend(feature_warnings)
        else:
            # Warning if no features declared for user-facing components
            comp_type = manifest.get('type')
            if comp_type in self.USER_FACING_TYPES:
                result['warnings'].append(
                    f"Type '{comp_type}' should declare user_facing_features. "
                    f"Check #13 will require this in v0.13.0+"
                )

        if result['errors']:
            result['valid'] = False

        return result

    def _validate_features(self, features: Dict, comp_type: Optional[str]) -> tuple:
        """
        Validate user_facing_features section.

        Returns:
            (errors: List[str], warnings: List[str])
        """
        errors = []
        warnings = []

        if not isinstance(features, dict):
            errors.append("user_facing_features must be a dictionary")
            return errors, warnings

        # Validate CLI commands
        if 'cli_commands' in features:
            cmd_errors, cmd_warnings = self._validate_cli_commands(
                features['cli_commands']
            )
            errors.extend(cmd_errors)
            warnings.extend(cmd_warnings)

            # Check consistency with component type
            if comp_type and comp_type != 'cli_application':
                warnings.append(
                    f"cli_commands declared but type is '{comp_type}' (expected 'cli_application')"
                )

        # Validate public API
        if 'public_api' in features:
            api_errors, api_warnings = self._validate_public_api(
                features['public_api']
            )
            errors.extend(api_errors)
            warnings.extend(api_warnings)

            # Check consistency with component type
            if comp_type and comp_type not in ['library', 'generic']:
                warnings.append(
                    f"public_api declared but type is '{comp_type}' (usually for libraries)"
                )

        # Validate HTTP endpoints
        if 'http_endpoints' in features:
            endpoint_errors, endpoint_warnings = self._validate_http_endpoints(
                features['http_endpoints']
            )
            errors.extend(endpoint_errors)
            warnings.extend(endpoint_warnings)

            # Check consistency with component type
            if comp_type and comp_type != 'web_server':
                warnings.append(
                    f"http_endpoints declared but type is '{comp_type}' (expected 'web_server')"
                )

        # Check if any features are declared
        if not any(k in features for k in ['cli_commands', 'public_api', 'http_endpoints']):
            warnings.append("user_facing_features section is empty")

        return errors, warnings

    def _validate_cli_commands(self, commands: List) -> tuple:
        """Validate cli_commands section."""
        errors = []
        warnings = []

        if not isinstance(commands, list):
            errors.append("cli_commands must be a list")
            return errors, warnings

        for idx, cmd in enumerate(commands):
            if not isinstance(cmd, dict):
                errors.append(f"cli_commands[{idx}]: must be a dictionary")
                continue

            # Required: name
            if 'name' not in cmd:
                errors.append(f"cli_commands[{idx}]: missing 'name' field")

            # Recommended: smoke_test
            if 'smoke_test' not in cmd:
                cmd_name = cmd.get('name', f'index {idx}')
                warnings.append(
                    f"CLI command '{cmd_name}': no smoke_test defined. "
                    f"Recommended: add smoke_test for automatic validation"
                )

            # Optional but good: description
            if 'description' not in cmd:
                cmd_name = cmd.get('name', f'index {idx}')
                warnings.append(f"CLI command '{cmd_name}': no description")

        return errors, warnings

    def _validate_public_api(self, api_list: List) -> tuple:
        """Validate public_api section."""
        errors = []
        warnings = []

        if not isinstance(api_list, list):
            errors.append("public_api must be a list")
            return errors, warnings

        for idx, api in enumerate(api_list):
            if not isinstance(api, dict):
                errors.append(f"public_api[{idx}]: must be a dictionary")
                continue

            # Must have either 'class' or 'function'
            if 'class' not in api and 'function' not in api:
                errors.append(
                    f"public_api[{idx}]: must have either 'class' or 'function' field"
                )

            # Required: module
            if 'module' not in api:
                api_name = api.get('class') or api.get('function') or f'index {idx}'
                errors.append(f"public_api '{api_name}': missing 'module' field")

            # For classes: validate methods
            if 'class' in api and 'methods' in api:
                methods = api['methods']
                if not isinstance(methods, list):
                    errors.append(
                        f"public_api '{api['class']}': methods must be a list"
                    )

        return errors, warnings

    def _validate_http_endpoints(self, endpoints: List) -> tuple:
        """Validate http_endpoints section."""
        errors = []
        warnings = []

        if not isinstance(endpoints, list):
            errors.append("http_endpoints must be a list")
            return errors, warnings

        for idx, endpoint in enumerate(endpoints):
            if not isinstance(endpoint, dict):
                errors.append(f"http_endpoints[{idx}]: must be a dictionary")
                continue

            # Required: path
            if 'path' not in endpoint:
                errors.append(f"http_endpoints[{idx}]: missing 'path' field")

            # Required: method
            if 'method' not in endpoint:
                path = endpoint.get('path', f'index {idx}')
                errors.append(f"http_endpoint '{path}': missing 'method' field")
            else:
                # Validate HTTP method
                valid_methods = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS', 'HEAD']
                method = endpoint['method'].upper()
                if method not in valid_methods:
                    errors.append(
                        f"http_endpoint '{endpoint.get('path')}': "
                        f"invalid method '{method}'. Valid: {', '.join(valid_methods)}"
                    )

        return errors, warnings


def validate_manifest(manifest_path: Path) -> Dict:
    """
    Convenience function to validate a manifest.

    Args:
        manifest_path: Path to component.yaml

    Returns:
        Validation result dictionary
    """
    validator = ManifestValidator()
    return validator.validate(manifest_path)


if __name__ == '__main__':
    import sys

    if len(sys.argv) < 2:
        print("Usage: python manifest_validator.py <path/to/component.yaml>")
        sys.exit(1)

    manifest_path = Path(sys.argv[1])
    result = validate_manifest(manifest_path)

    print(f"Validating: {manifest_path}")
    print()

    if result['valid']:
        print("✅ Valid manifest")
    else:
        print("❌ Invalid manifest")

    if result['errors']:
        print(f"\nErrors ({len(result['errors'])}):")
        for error in result['errors']:
            print(f"  - {error}")

    if result['warnings']:
        print(f"\nWarnings ({len(result['warnings'])}):")
        for warning in result['warnings']:
            print(f"  - {warning}")

    sys.exit(0 if result['valid'] else 1)
