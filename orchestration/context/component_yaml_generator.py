#!/usr/bin/env python3
"""
Component YAML Generator for Subagent Self-Generation

When a subagent starts working on a component that lacks component.yaml,
it can use this generator to create one by extracting metadata from:
1. Existing CLAUDE.md file (old-style)
2. Directory structure and files
3. Minimal defaults

This enables backwards compatibility with existing projects while
supporting the new context system.

Usage:
    from orchestration.context.component_yaml_generator import ComponentYamlGenerator

    # Generate if missing (returns True if generated, False if already exists)
    generated = ComponentYamlGenerator.generate_if_missing(Path("components/auth_service"))

    # Force generation (overwrites existing)
    metadata = ComponentYamlGenerator.generate(Path("components/auth_service"))
"""

import re
from pathlib import Path
from typing import Dict, Any, Optional


class ComponentYamlGenerator:
    """Generate component.yaml from existing context or defaults."""

    @staticmethod
    def extract_from_claude_md(claude_path: Path) -> Optional[Dict[str, Any]]:
        """
        Extract metadata from old-style CLAUDE.md.

        Old CLAUDE.md files contain patterns like:
        - "# component_name Backend Component"
        - "**Tech Stack**: Python, FastAPI"
        - 60+ occurrences of component name

        Args:
            claude_path: Path to CLAUDE.md file

        Returns:
            Metadata dict or None if extraction fails
        """
        if not claude_path.exists():
            return None

        content = claude_path.read_text()

        # Check if this is an old-style generated CLAUDE.md
        # New minimal CLAUDE.md files have "CONTEXT INCLUDES" at the top
        if "CONTEXT INCLUDES" in content[:200]:
            return None  # This is a new-style file, don't extract

        # Extract component name from header
        # Pattern: "# component_name Backend Component" or similar
        name_match = re.search(
            r'^#\s+(\w+)\s+(?:Backend|Frontend|Library|Generic)\s+Component',
            content,
            re.MULTILINE | re.IGNORECASE
        )
        name = name_match.group(1).lower() if name_match else None

        # If no match, try getting from directory
        if not name:
            name = claude_path.parent.name

        # Determine component type from header
        comp_type = "library"  # Default
        if "Backend Component" in content[:300]:
            comp_type = "backend"
        elif "Frontend Component" in content[:300]:
            comp_type = "frontend"
        elif "Generic Component" in content[:300]:
            comp_type = "library"

        # Extract tech stack
        tech_match = re.search(r'\*\*Tech Stack\*\*:\s*(.+)', content)
        tech_stack = tech_match.group(1).strip() if tech_match else "Not specified"

        # Extract responsibility if present
        resp_match = re.search(r'\*\*Responsibility\*\*:\s*(.+)', content)
        responsibility = resp_match.group(1).strip() if resp_match else f"{name} component"

        return {
            "name": name,
            "type": comp_type,
            "version": "0.1.0",
            "tech_stack": tech_stack,
            "responsibility": responsibility,
            "dependencies": [],
            "context_rules": ["../../orchestration/context/component-rules.md"],
            "contracts": [],
            "user_facing_features": []
        }

    @staticmethod
    def derive_from_context(component_path: Path) -> Dict[str, Any]:
        """
        Derive metadata from directory structure and files.

        Used as fallback when CLAUDE.md extraction fails.

        Args:
            component_path: Path to component directory

        Returns:
            Metadata dict with derived values
        """
        name = component_path.name

        # Detect component type from file patterns
        has_api_files = (
            any(component_path.glob("**/api.py")) or
            any(component_path.glob("**/routes.py")) or
            any(component_path.glob("**/endpoints.py"))
        )
        has_ui_files = (
            any(component_path.glob("**/*.jsx")) or
            any(component_path.glob("**/*.tsx")) or
            any(component_path.glob("**/*.vue"))
        )
        has_service_files = any(component_path.glob("**/services/*.py"))

        if has_api_files or has_service_files:
            comp_type = "backend"
        elif has_ui_files:
            comp_type = "frontend"
        else:
            comp_type = "library"

        # Detect tech stack from files
        tech_stack = []

        if (component_path / "requirements.txt").exists():
            tech_stack.append("Python")
        elif (component_path / "pyproject.toml").exists():
            tech_stack.append("Python")
        elif any(component_path.glob("**/*.py")):
            tech_stack.append("Python")

        if (component_path / "package.json").exists():
            tech_stack.append("Node.js")
        if any(component_path.glob("**/*.ts")):
            tech_stack.append("TypeScript")
        if any(component_path.glob("**/*.js")):
            if "TypeScript" not in tech_stack:
                tech_stack.append("JavaScript")

        if (component_path / "Cargo.toml").exists():
            tech_stack.append("Rust")
        if (component_path / "go.mod").exists():
            tech_stack.append("Go")

        return {
            "name": name,
            "type": comp_type,
            "version": "0.1.0",
            "tech_stack": ", ".join(tech_stack) if tech_stack else "Python",
            "responsibility": f"{name} component",
            "dependencies": [],
            "context_rules": ["../../orchestration/context/component-rules.md"],
            "contracts": [],
            "user_facing_features": []
        }

    @staticmethod
    def to_yaml(metadata: Dict[str, Any]) -> str:
        """
        Convert metadata dict to YAML string.

        Using manual formatting to avoid PyYAML dependency.
        """
        lines = [
            "# Component Metadata (auto-generated)",
            f"name: {metadata['name']}",
            f"type: {metadata['type']}",
            f"version: {metadata['version']}",
            f'tech_stack: "{metadata["tech_stack"]}"',
            f'responsibility: "{metadata["responsibility"]}"',
            "",
            "# Dependencies on other components",
            "dependencies: []",
            "",
            "# Context rules (system-maintained)",
            "context_rules:"
        ]

        for rule in metadata.get("context_rules", []):
            lines.append(f"  - {rule}")

        lines.extend([
            "",
            "# API contracts (if applicable)",
            "contracts: []",
            "",
            "# Public API declaration (required for feature coverage testing)",
            "user_facing_features: []",
            "# public_api:",
            f"#   - class: MainClass",
            f"#     module: components.{metadata['name']}.src.module",
            "#     methods: [method1, method2]",
            "#     description: Main interface",
        ])

        return "\n".join(lines) + "\n"

    @classmethod
    def generate(cls, component_path: Path) -> Dict[str, Any]:
        """
        Generate component.yaml for a component.

        Tries extraction methods in order:
        1. Extract from old-style CLAUDE.md
        2. Derive from directory context

        Args:
            component_path: Path to component directory

        Returns:
            Generated metadata dict
        """
        # Try extracting from CLAUDE.md first
        claude_path = component_path / "CLAUDE.md"
        metadata = cls.extract_from_claude_md(claude_path)

        # Fall back to context derivation
        if not metadata:
            metadata = cls.derive_from_context(component_path)

        # Ensure name matches directory
        metadata["name"] = component_path.name

        return metadata

    @classmethod
    def generate_if_missing(cls, component_path: Path) -> bool:
        """
        Generate component.yaml if it doesn't exist.

        Safe to call multiple times - only generates once.

        Args:
            component_path: Path to component directory

        Returns:
            True if generated, False if already exists
        """
        yaml_path = component_path / "component.yaml"

        if yaml_path.exists():
            return False

        # Generate metadata
        metadata = cls.generate(component_path)

        # Write component.yaml
        yaml_content = cls.to_yaml(metadata)
        yaml_path.write_text(yaml_content)

        print(f"✅ Generated component.yaml for {component_path.name}")
        return True


def main():
    """CLI entry point for manual generation."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python component_yaml_generator.py <component_path>")
        print("       python component_yaml_generator.py components/auth_service")
        sys.exit(1)

    component_path = Path(sys.argv[1])

    if not component_path.exists():
        print(f"❌ Component path not found: {component_path}")
        sys.exit(1)

    if ComponentYamlGenerator.generate_if_missing(component_path):
        print("✅ component.yaml generated successfully")
    else:
        print("ℹ️  component.yaml already exists")


if __name__ == "__main__":
    main()
