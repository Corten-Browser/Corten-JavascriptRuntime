"""
Template Engine

Renders CLAUDE.md templates with variable substitution for component creation.
Supports {{VARIABLE}} syntax and validates template completeness.

Classes:
    TemplateEngine: Render templates with variable substitution
"""

from pathlib import Path
from typing import Dict, List, Optional
import re


class TemplateEngine:
    """
    Render CLAUDE.md templates with variable substitution.

    This class handles template loading, variable substitution, and validation
    for creating component-specific CLAUDE.md files from templates.

    Attributes:
        templates_dir: Directory containing template files
    """

    def __init__(self, templates_dir: Optional[Path] = None):
        """
        Initialize TemplateEngine.

        Args:
            templates_dir: Path to templates directory
                (default: claude-orchestration-system/templates)
        """
        if templates_dir is None:
            # Try to find templates directory relative to this file
            orchestration_dir = Path(__file__).parent
            system_dir = orchestration_dir.parent
            templates_dir = system_dir / "templates"

        self.templates_dir = Path(templates_dir)

        if not self.templates_dir.exists():
            # Fallback to relative path
            self.templates_dir = Path("claude-orchestration-system/templates")

    def render(self, template_name: str, variables: Dict[str, str]) -> str:
        """
        Render a template with variable substitution.

        Args:
            template_name: Name of template file (e.g., "component-backend.md")
            variables: Dictionary of variable names to values

        Returns:
            Rendered template string

        Raises:
            FileNotFoundError: If template file doesn't exist
            ValueError: If unreplaced variables remain
        """

        template_path = self.templates_dir / template_name

        if not template_path.exists():
            # Try without .md extension
            if not template_name.endswith('.md'):
                template_path = self.templates_dir / f"{template_name}.md"

            if not template_path.exists():
                raise FileNotFoundError(f"Template not found: {template_name}")

        template_content = template_path.read_text()

        # Replace {{VARIABLE}} with values
        for var_name, var_value in variables.items():
            # Escape special regex characters in variable name
            pattern = r'\{\{' + re.escape(var_name) + r'\}\}'
            template_content = re.sub(pattern, str(var_value), template_content)

        # Check for unreplaced variables
        unreplaced = re.findall(r'\{\{([^}]+)\}\}', template_content)
        if unreplaced:
            raise ValueError(f"Unreplaced variables in template: {', '.join(unreplaced)}")

        return template_content

    def render_safe(self, template_name: str, variables: Dict[str, str],
                   default_value: str = "") -> str:
        """
        Render a template with variable substitution, using default for missing vars.

        Args:
            template_name: Name of template file
            variables: Dictionary of variable names to values
            default_value: Default value for unreplaced variables

        Returns:
            Rendered template string
        """

        template_path = self.templates_dir / template_name

        if not template_path.exists():
            if not template_name.endswith('.md'):
                template_path = self.templates_dir / f"{template_name}.md"

            if not template_path.exists():
                raise FileNotFoundError(f"Template not found: {template_name}")

        template_content = template_path.read_text()

        # Replace {{VARIABLE}} with values
        for var_name, var_value in variables.items():
            pattern = r'\{\{' + re.escape(var_name) + r'\}\}'
            template_content = re.sub(pattern, str(var_value), template_content)

        # Replace remaining variables with default value
        template_content = re.sub(r'\{\{[^}]+\}\}', default_value, template_content)

        return template_content

    def list_templates(self) -> List[str]:
        """
        List all available templates.

        Returns:
            List of template filenames
        """

        if not self.templates_dir.exists():
            return []

        return [f.name for f in self.templates_dir.glob("*.md")]

    def get_required_variables(self, template_name: str) -> List[str]:
        """
        Extract required variables from a template.

        Args:
            template_name: Name of template file

        Returns:
            List of unique variable names found in template
        """

        template_path = self.templates_dir / template_name

        if not template_path.exists():
            if not template_name.endswith('.md'):
                template_path = self.templates_dir / f"{template_name}.md"

            if not template_path.exists():
                raise FileNotFoundError(f"Template not found: {template_name}")

        content = template_path.read_text()

        # Find all {{VARIABLE}} patterns
        variables = re.findall(r'\{\{([^}]+)\}\}', content)

        # Return unique variables
        return list(set(variables))

    def validate_variables(self, template_name: str, variables: Dict[str, str]) -> Dict:
        """
        Validate that all required variables are provided.

        Args:
            template_name: Name of template file
            variables: Dictionary of variables to validate

        Returns:
            Dictionary containing:
                - valid: Whether all required variables are present
                - missing: List of missing variable names
                - extra: List of extra variable names provided
        """

        required = set(self.get_required_variables(template_name))
        provided = set(variables.keys())

        missing = required - provided
        extra = provided - required

        return {
            'valid': len(missing) == 0,
            'missing': list(missing),
            'extra': list(extra)
        }

    def render_with_validation(self, template_name: str,
                               variables: Dict[str, str]) -> Dict:
        """
        Render template with validation reporting.

        Args:
            template_name: Name of template file
            variables: Dictionary of variable names to values

        Returns:
            Dictionary containing:
                - success: Whether rendering succeeded
                - content: Rendered content (if successful)
                - validation: Validation results
                - error: Error message (if failed)
        """

        # Validate variables
        validation = self.validate_variables(template_name, variables)

        if not validation['valid']:
            return {
                'success': False,
                'validation': validation,
                'error': f"Missing required variables: {', '.join(validation['missing'])}"
            }

        # Attempt rendering
        try:
            content = self.render(template_name, variables)
            return {
                'success': True,
                'content': content,
                'validation': validation
            }
        except Exception as e:
            return {
                'success': False,
                'validation': validation,
                'error': str(e)
            }


class TemplateValidator:
    """
    Validate templates for correctness.

    This class provides validation functionality for template files,
    checking for required sections, balanced markers, and valid variable names.
    """

    def __init__(self, templates_dir: Path):
        """
        Initialize TemplateValidator.

        Args:
            templates_dir: Path to templates directory
        """
        self.templates_dir = Path(templates_dir)

    def validate_template(self, template_name: str) -> tuple[bool, List[str]]:
        """
        Validate a template file.

        Args:
            template_name: Name of template file

        Returns:
            Tuple of (is_valid, list_of_errors)
        """

        errors = []
        template_path = self.templates_dir / template_name

        if not template_path.exists():
            return False, [f"Template not found: {template_name}"]

        content = template_path.read_text()

        # Check for required sections (basic templates should have these)
        required_sections = [
            "# ",  # Title
        ]

        for section in required_sections:
            if section not in content:
                errors.append(f"Missing required section: {section}")

        # Check for balanced variables
        open_vars = len(re.findall(r'\{\{', content))
        close_vars = len(re.findall(r'\}\}', content))
        if open_vars != close_vars:
            errors.append(f"Unbalanced variable markers: {open_vars} {{ vs {close_vars} }}")

        # Check for valid variable names (UPPERCASE_WITH_UNDERSCORES)
        variables = re.findall(r'\{\{([^}]+)\}\}', content)
        for var in variables:
            if not re.match(r'^[A-Z_][A-Z0-9_]*$', var):
                errors.append(f"Invalid variable name: {var} (should be UPPERCASE_WITH_UNDERSCORES)")

        return len(errors) == 0, errors

    def validate_all_templates(self) -> Dict[str, tuple[bool, List[str]]]:
        """
        Validate all templates in directory.

        Returns:
            Dictionary mapping template names to (is_valid, errors) tuples
        """

        results = {}

        if not self.templates_dir.exists():
            return results

        for template_file in self.templates_dir.glob("*.md"):
            is_valid, errors = self.validate_template(template_file.name)
            results[template_file.name] = (is_valid, errors)

        return results


if __name__ == "__main__":
    # CLI interface for testing
    import sys
    import json

    engine = TemplateEngine()

    if len(sys.argv) > 1:
        command = sys.argv[1]

        if command == "list":
            templates = engine.list_templates()
            print("Available templates:")
            for template in templates:
                print(f"  - {template}")

        elif command == "variables" and len(sys.argv) > 2:
            template = sys.argv[2]
            try:
                variables = engine.get_required_variables(template)
                print(f"Required variables for {template}:")
                for var in variables:
                    print(f"  - {var}")
            except FileNotFoundError as e:
                print(f"Error: {e}")

        elif command == "validate" and len(sys.argv) > 2:
            template = sys.argv[2]
            validator = TemplateValidator(engine.templates_dir)
            is_valid, errors = validator.validate_template(template)

            if is_valid:
                print(f"✓ {template} is valid")
            else:
                print(f"✗ {template} has errors:")
                for error in errors:
                    print(f"  - {error}")

        else:
            print("Unknown command")
    else:
        print("Template Engine CLI")
        print("Commands:")
        print("  list                     - List all templates")
        print("  variables <template>     - Show required variables")
        print("  validate <template>      - Validate template")
