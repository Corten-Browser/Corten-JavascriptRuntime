#!/usr/bin/env python3
"""Check git status for a specific component."""

import subprocess
import sys

def component_status(component_name):
    """Show git status for a specific component."""
    result = subprocess.run(
        ['git', 'status', '--short', f'components/{component_name}/'],
        capture_output=True,
        text=True
    )

    if result.stdout:
        print(f"📝 Changes in {component_name}:")
        print(result.stdout)
    else:
        print(f"✨ No changes in {component_name}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python git_status.py <component-name>")
        sys.exit(1)

    component_status(sys.argv[1])