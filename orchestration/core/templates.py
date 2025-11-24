#!/usr/bin/env python3
"""
Embedded templates for component generation.

Templates are stored as Python strings to avoid external file dependencies.
This allows the orchestration system to work immediately after cloning from
a git repository without needing to carry template files.

All templates support variable substitution using {{VARIABLE_NAME}} syntax.
"""

import re
from pathlib import Path
from typing import Dict, Optional


def get_template(template_type: str) -> str:
    """
    Get template by type.

    Args:
        template_type: One of 'master', 'backend', 'frontend', 'generic'

    Returns:
        Template string with {{VARIABLE}} placeholders

    Raises:
        ValueError: If template_type is not recognized
    """
    templates = {
        'master': MASTER_ORCHESTRATOR_TEMPLATE,
        'backend': COMPONENT_BACKEND_TEMPLATE,
        'frontend': COMPONENT_FRONTEND_TEMPLATE,
        'generic': COMPONENT_GENERIC_TEMPLATE,
    }

    template = templates.get(template_type)
    if template is None:
        raise ValueError(
            f"Unknown template type: {template_type}. "
            f"Available: {', '.join(templates.keys())}"
        )

    return template


def render_template(template_type: str, variables: Dict[str, str]) -> str:
    """
    Render template with variables.

    Args:
        template_type: One of 'master', 'backend', 'frontend', 'generic'
        variables: Dictionary of variable name -> value

    Returns:
        Rendered template string

    Example:
        >>> variables = {
        ...     'PROJECT_NAME': 'my-project',
        ...     'PROJECT_VERSION': '0.1.0',
        ... }
        >>> content = render_template('master', variables)
    """
    template = get_template(template_type)

    # Replace all {{VARIABLE}} placeholders
    for key, value in variables.items():
        placeholder = f'{{{{{key}}}}}'
        template = template.replace(placeholder, str(value))

    return template


def get_available_templates() -> list[str]:
    """Get list of available template types."""
    return ['master', 'backend', 'frontend', 'generic']


def load_template_from_file(file_path: str) -> str:
    """
    Load template from external file (for development/testing).

    This function is used during development to update embedded templates
    from the source .md files in claude-orchestration-system/templates/

    Args:
        file_path: Path to template .md file

    Returns:
        Template content as string
    """
    return Path(file_path).read_text()


# ============================================================================
# EMBEDDED TEMPLATES
# ============================================================================

# Note: These templates are loaded from the source files in
# claude-orchestration-system/templates/ during the build process.
# To update these templates, modify the source .md files and run:
#   python orchestration/update_templates.py

MASTER_ORCHESTRATOR_TEMPLATE = ""  # Will be populated below

COMPONENT_BACKEND_TEMPLATE = ""  # Will be populated below

COMPONENT_FRONTEND_TEMPLATE = ""  # Will be populated below

COMPONENT_GENERIC_TEMPLATE = ""  # Will be populated below


# Load templates from files if they exist (development mode)
# In production (after installation), templates are embedded in this file
try:
    # Look in orchestration/templates/ directory
    # Path(__file__).parent is orchestration/core/
    # Path(__file__).parent.parent is orchestration/
    # So parent.parent / 'templates' gives orchestration/templates/
    _template_dir = Path(__file__).parent.parent / 'templates'

    if _template_dir.exists():
        MASTER_ORCHESTRATOR_TEMPLATE = load_template_from_file(
            str(_template_dir / 'master-orchestrator.md')
        )
        COMPONENT_BACKEND_TEMPLATE = load_template_from_file(
            str(_template_dir / 'component-backend.md')
        )
        COMPONENT_FRONTEND_TEMPLATE = load_template_from_file(
            str(_template_dir / 'component-frontend.md')
        )
        COMPONENT_GENERIC_TEMPLATE = load_template_from_file(
            str(_template_dir / 'component-generic.md')
        )
except Exception as e:
    # In production after installation, templates will be embedded below
    # During development, this indicates template files are missing
    import sys
    if not any(MASTER_ORCHESTRATOR_TEMPLATE):  # Check if templates are embedded
        print(f"Warning: Failed to load templates from {_template_dir}: {e}", file=sys.stderr)
    pass


if __name__ == "__main__":
    # CLI for testing templates
    import sys

    if len(sys.argv) < 2:
        print("Usage: python templates.py <template_type>")
        print(f"Available templates: {', '.join(get_available_templates())}")
        sys.exit(1)

    template_type = sys.argv[1]
    try:
        template = get_template(template_type)
        print(f"Template: {template_type}")
        print(f"Length: {len(template)} characters")
        print(f"Lines: {template.count(chr(10)) + 1}")

        # Find all variables
        variables = re.findall(r'{{([A-Z_]+)}}', template)
        unique_vars = sorted(set(variables))
        print(f"Variables: {', '.join(unique_vars)}")
    except ValueError as e:
        print(f"Error: {e}")
        sys.exit(1)
