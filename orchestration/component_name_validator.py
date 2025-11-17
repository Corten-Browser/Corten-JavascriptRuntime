#!/usr/bin/env python3
"""
Component Name Validator

Enforces universal component naming convention that works across all programming languages.

Convention: [a-z][a-z0-9_]*
- Lowercase letters only
- Numbers allowed (not as first character)
- Underscores allowed (not as first character)
- No hyphens, spaces, or special characters

This convention ensures compatibility with:
- Python (module imports)
- JavaScript/TypeScript (import paths)
- Rust (crate names, module paths)
- Go (package names)
- Java (package names)
- C++ (namespace names)

Part of v0.3.0 completion guarantee system.
"""

import re
from pathlib import Path
from typing import Tuple, List, Optional
from dataclasses import dataclass


@dataclass
class ValidationResult:
    """Result of component name validation."""
    is_valid: bool
    name: str
    error_message: Optional[str] = None
    suggestion: Optional[str] = None


class ComponentNameValidator:
    """Validates component names follow universal naming convention."""

    # Universal naming pattern: lowercase + numbers + underscores
    VALID_NAME_PATTERN = re.compile(r'^[a-z][a-z0-9_]*$')

    # Reserved names that should not be used as component names
    RESERVED_NAMES = {
        'test', 'tests', 'src', 'lib', 'bin', 'dist', 'build',
        'node_modules', 'venv', 'env', '__pycache__', 'target',
        'orchestration', 'contracts', 'shared_libs', 'docs',
        'scripts', 'config', 'data', 'tmp', 'temp'
    }

    def __init__(self):
        """Initialize validator."""
        pass

    def validate(self, name: str) -> ValidationResult:
        """
        Validate a component name.

        Args:
            name: Component name to validate

        Returns:
            ValidationResult with validation outcome
        """
        # Check empty
        if not name or not name.strip():
            return ValidationResult(
                is_valid=False,
                name=name,
                error_message="Component name cannot be empty",
                suggestion=None
            )

        name = name.strip()

        # Check reserved names
        if name.lower() in self.RESERVED_NAMES:
            return ValidationResult(
                is_valid=False,
                name=name,
                error_message=f"'{name}' is a reserved name and cannot be used",
                suggestion=f"Try '{name}_component' or 'app_{name}'"
            )

        # Check pattern
        if not self.VALID_NAME_PATTERN.match(name):
            error_msg = self._build_error_message(name)
            suggestion = self._suggest_valid_name(name)

            return ValidationResult(
                is_valid=False,
                name=name,
                error_message=error_msg,
                suggestion=suggestion
            )

        # Valid name
        return ValidationResult(
            is_valid=True,
            name=name,
            error_message=None,
            suggestion=None
        )

    def _build_error_message(self, name: str) -> str:
        """Build descriptive error message for invalid name."""
        errors = []

        if not name[0].islower() or not name[0].isalpha():
            errors.append("must start with lowercase letter")

        if '-' in name:
            errors.append("cannot contain hyphens (use underscores instead)")

        if ' ' in name:
            errors.append("cannot contain spaces (use underscores instead)")

        if any(c.isupper() for c in name):
            errors.append("cannot contain uppercase letters")

        invalid_chars = [c for c in name if not (c.isalnum() or c == '_')]
        if invalid_chars:
            unique_invalid = list(set(invalid_chars))
            errors.append(f"cannot contain special characters: {', '.join(repr(c) for c in unique_invalid)}")

        if not errors:
            errors.append("does not match required pattern [a-z][a-z0-9_]*")

        return f"Invalid component name '{name}': {'; '.join(errors)}"

    def _suggest_valid_name(self, name: str) -> str:
        """Suggest a valid name based on invalid input."""
        # Convert to lowercase
        suggestion = name.lower()

        # Replace hyphens and spaces with underscores
        suggestion = suggestion.replace('-', '_').replace(' ', '_')

        # Remove invalid characters
        suggestion = re.sub(r'[^a-z0-9_]', '', suggestion)

        # Ensure starts with letter
        if suggestion and not suggestion[0].isalpha():
            suggestion = 'component_' + suggestion

        # Remove consecutive underscores
        suggestion = re.sub(r'_+', '_', suggestion)

        # Remove leading/trailing underscores
        suggestion = suggestion.strip('_')

        # If empty after cleanup, provide default
        if not suggestion:
            suggestion = 'my_component'

        return suggestion

    def validate_path(self, component_path: Path) -> ValidationResult:
        """
        Validate a component directory path.

        Args:
            component_path: Path to component directory

        Returns:
            ValidationResult for the component name extracted from path
        """
        component_name = component_path.name
        return self.validate(component_name)

    def batch_validate(self, names: List[str]) -> List[ValidationResult]:
        """
        Validate multiple component names.

        Args:
            names: List of component names to validate

        Returns:
            List of ValidationResult objects
        """
        return [self.validate(name) for name in names]

    def is_valid(self, name: str) -> bool:
        """
        Quick check if name is valid.

        Args:
            name: Component name to check

        Returns:
            True if valid, False otherwise
        """
        return self.validate(name).is_valid


def validate_component_name(name: str) -> Tuple[bool, str]:
    """
    Convenience function for simple validation.

    Args:
        name: Component name to validate

    Returns:
        Tuple of (is_valid, message)
        - If valid: (True, "Valid component name")
        - If invalid: (False, "Error: description [suggestion: valid_name]")
    """
    validator = ComponentNameValidator()
    result = validator.validate(name)

    if result.is_valid:
        return True, "Valid component name"

    message = result.error_message
    if result.suggestion:
        message += f" [suggestion: {result.suggestion}]"

    return False, message


def main():
    """CLI interface for component name validation."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: component_name_validator.py <component_name> [<component_name> ...]")
        print("\nValidates component names against universal naming convention.")
        print("Convention: [a-z][a-z0-9_]*")
        print("\nExamples:")
        print("  ✅ audio_processor")
        print("  ✅ shared_types")
        print("  ✅ data_manager")
        print("  ❌ audio-processor (hyphens not allowed)")
        print("  ❌ sharedTypes (camelCase not allowed)")
        print("  ❌ AudioProcessor (PascalCase not allowed)")
        sys.exit(1)

    validator = ComponentNameValidator()
    names = sys.argv[1:]

    print(f"Validating {len(names)} component name(s)...\n")

    all_valid = True
    for name in names:
        result = validator.validate(name)

        if result.is_valid:
            print(f"✅ '{name}' - Valid")
        else:
            print(f"❌ '{name}' - Invalid")
            print(f"   {result.error_message}")
            if result.suggestion:
                print(f"   Suggestion: '{result.suggestion}'")
            all_valid = False
        print()

    if all_valid:
        print(f"✅ All {len(names)} name(s) are valid!")
        sys.exit(0)
    else:
        print(f"❌ Some names are invalid. Please fix and retry.")
        sys.exit(1)


if __name__ == '__main__':
    main()
