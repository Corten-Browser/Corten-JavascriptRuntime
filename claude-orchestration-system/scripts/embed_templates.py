#!/usr/bin/env python3
"""
Embed template files into templates.py for distribution.

This script reads template .md files and embeds them as Python strings
in the templates.py file, eliminating the need to distribute template files.

Usage:
    python embed_templates.py <templates_dir> <output_file>

Example:
    python embed_templates.py ./templates ../orchestration/templates.py
"""

import sys
from pathlib import Path


def escape_string(s: str) -> str:
    """Escape string for Python triple-quoted string."""
    # Replace triple quotes with escaped version
    s = s.replace('"""', r'\"\"\"')
    return s


def embed_templates(templates_dir: Path, output_file: Path):
    """
    Read template files and embed them in templates.py.

    Args:
        templates_dir: Directory containing template .md files
        output_file: Path to templates.py file to update
    """
    # Read template files
    templates = {
        'master': templates_dir / 'master-orchestrator.md',
        'backend': templates_dir / 'component-backend.md',
        'frontend': templates_dir / 'component-frontend.md',
        'generic': templates_dir / 'component-generic.md',
    }

    # Read existing templates.py
    content = output_file.read_text()

    # Find the section where templates are defined
    # Look for: MASTER_ORCHESTRATOR_TEMPLATE = ""
    # Replace with: MASTER_ORCHESTRATOR_TEMPLATE = """..."""

    for name, template_file in templates.items():
        if not template_file.exists():
            print(f"Warning: Template file not found: {template_file}")
            continue

        template_content = template_file.read_text()
        escaped_content = escape_string(template_content)

        # Determine the variable name
        var_name = f"{name.upper().replace('-', '_')}"
        if name == 'master':
            var_name = 'MASTER_ORCHESTRATOR'
        elif name in ['backend', 'frontend', 'generic']:
            var_name = f"COMPONENT_{name.upper()}"

        var_name += "_TEMPLATE"

        # Replace the empty template definition
        # Pattern: VAR_NAME = ""  # Will be populated below
        # Or:      VAR_NAME = ""
        old_pattern = f'{var_name} = ""'

        # New content with embedded template
        new_content = f'{var_name} = """{escaped_content}"""'

        if old_pattern in content:
            content = content.replace(old_pattern, new_content, 1)
            print(f"✅ Embedded {name} template ({len(template_content)} chars)")
        else:
            print(f"⚠️  Could not find {var_name} in templates.py")

    # Remove the dynamic loading code (the try/except block at the end)
    # Find the block that starts with "# Load templates from files"
    try_block_start = content.find("# Load templates from files if they exist")
    if try_block_start != -1:
        # Find the end of the try/except block
        # Look for the "if __name__" that comes after
        main_block_start = content.find('if __name__ == "__main__":', try_block_start)
        if main_block_start != -1:
            # Remove everything between try_block_start and main_block_start
            before = content[:try_block_start]
            after = content[main_block_start:]
            content = before + "\n\n" + after
            print("✅ Removed dynamic loading code")

    # Write updated content
    output_file.write_text(content)
    print(f"\n✅ Templates embedded successfully in {output_file}")
    print(f"📦 Total size: {len(content):,} characters")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python embed_templates.py <templates_dir> <output_file>")
        print("\nExample:")
        print("  python embed_templates.py ./templates ../orchestration/templates.py")
        sys.exit(1)

    templates_dir = Path(sys.argv[1])
    output_file = Path(sys.argv[2])

    if not templates_dir.exists():
        print(f"Error: Templates directory not found: {templates_dir}")
        sys.exit(1)

    if not output_file.exists():
        print(f"Error: Output file not found: {output_file}")
        sys.exit(1)

    embed_templates(templates_dir, output_file)
