#!/usr/bin/env python3
"""
Component Creation CLI with Mandatory Validation

Creates new components with validated names following the universal naming convention.

Usage:
    python orchestration/cli/create_component.py <name> <type> [options]

Examples:
    python orchestration/cli/create_component.py auth_service backend
    python orchestration/cli/create_component.py auth_service backend --lang=python
    python orchestration/cli/create_component.py payment_api backend --tech="Python,FastAPI"
"""

import sys
import argparse
import json
from pathlib import Path
from typing import Optional

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.verification.system.component_name_validator import ComponentNameValidator


class ComponentCreator:
    """Creates components with validation."""

    def __init__(self):
        self.validator = ComponentNameValidator()
        self.project_root = Path.cwd()

    def create(self, name: str, component_type: str, language: Optional[str] = None,
               tech_stack: Optional[str] = None, auto_approve: bool = False) -> str:
        """
        Create component with validation.

        Args:
            name: Component name (will be validated)
            component_type: Type (backend, frontend, library, microservice)
            language: Primary language (auto-detected if not specified)
            tech_stack: Technology stack (comma-separated)
            auto_approve: Auto-accept suggestions (orchestrator mode)

        Returns:
            Actual component name used (may be corrected)
        """

        # STEP 1: VALIDATE NAME (BLOCKING)
        result = self.validator.validate(name)
        if not result.is_valid:
            if auto_approve and result.suggestion:
                # Orchestrator mode - use suggestion
                print(f"⚠️  Invalid name '{name}': {result.error_message}")
                print(f"   Using suggestion: '{result.suggestion}'")
                name = result.suggestion
            else:
                # User mode - show error and exit
                print(f"❌ {result.error_message}")
                if result.suggestion:
                    print(f"   Suggestion: '{result.suggestion}'")
                sys.exit(1)

        # STEP 2: CHECK EXISTENCE
        comp_path = self.project_root / "components" / name
        if comp_path.exists():
            print(f"❌ Component already exists: {comp_path}")
            sys.exit(2)

        # STEP 3: DETECT/CONFIRM LANGUAGE
        if language is None:
            language = self.detect_project_language()
            if not auto_approve:
                print(f"Detected language: {language}")
                confirm = input(f"Use {language}? [Y/n]: ").strip().lower()
                if confirm and confirm != 'y':
                    language = input("Enter language: ").strip() or "python"

        # STEP 4: CREATE DIRECTORY STRUCTURE
        print(f"Creating component: {name}")
        self.create_directories(name, component_type, language)

        # STEP 5: GENERATE CLAUDE.MD
        self.generate_claude_md(name, component_type, language, tech_stack)

        # STEP 6: GENERATE MANIFEST
        self.generate_manifest(name, component_type, language, tech_stack)

        # STEP 7: GENERATE README
        self.generate_readme(name, component_type, tech_stack)

        # STEP 8: INITIALIZE GIT
        if (self.project_root / ".git").exists():
            self.init_git(name)

        # STEP 9: REPORT SUCCESS
        self.report_success(name, component_type, language, tech_stack)

        return name

    def create_directories(self, name: str, component_type: str, language: str):
        """Create directory structure."""
        base = self.project_root / "components" / name
        base.mkdir(parents=True, exist_ok=True)

        # Common directories
        (base / "src").mkdir(exist_ok=True)
        (base / "tests" / "unit").mkdir(parents=True, exist_ok=True)
        (base / "tests" / "integration").mkdir(parents=True, exist_ok=True)

        # Type-specific directories
        if component_type in ["backend", "microservice"]:
            (base / "src" / "api").mkdir(exist_ok=True)
            (base / "src" / "models").mkdir(exist_ok=True)
            (base / "src" / "services").mkdir(exist_ok=True)
            (base / "features").mkdir(exist_ok=True)

        elif component_type == "frontend":
            (base / "src" / "components").mkdir(exist_ok=True)
            (base / "src" / "pages").mkdir(exist_ok=True)
            (base / "src" / "styles").mkdir(exist_ok=True)

        # Language-specific files
        if language == "python":
            # Create __init__.py files
            for dir_path in base.rglob("*/"):
                if dir_path.name not in [".git", "__pycache__", "node_modules"]:
                    (dir_path / "__init__.py").touch()

    def generate_claude_md(self, name: str, component_type: str, language: str,
                          tech_stack: Optional[str]):
        """Generate minimal user-owned CLAUDE.md.

        The generic rules are now in orchestration/context/component-rules.md
        which is referenced by Task prompts. This CLAUDE.md is for user-specific
        specifications only.
        """
        # Minimal user-owned CLAUDE.md with context includes
        content = f"""<!-- CONTEXT INCLUDES
Read these files for complete development context:
- ../../orchestration/context/component-rules.md (generic rules)
- ./component.yaml (metadata)
-->

# {name} Component Specifications

## Overview
[TODO: Add component overview and purpose]

## Responsibilities
[TODO: Define what this component is responsible for]

## Features
[TODO: List specific features to implement]

## API Endpoints (if applicable)
[TODO: Define API endpoints this component exposes]

## Dependencies
[TODO: List dependencies on other components]

## Architecture Decisions
[TODO: Document key architectural decisions]

## Notes
[TODO: Any additional implementation notes]
"""

        # Write to component
        output_file = self.project_root / "components" / name / "CLAUDE.md"
        output_file.write_text(content)

    def generate_manifest(self, name: str, component_type: str, language: str = "python",
                         tech_stack: Optional[str] = None):
        """Generate enhanced component.yaml manifest with all metadata."""
        manifest_content = f"""# Component Metadata
name: {name}
type: {component_type}
version: 0.1.0
tech_stack: "{tech_stack or language.title()}"
responsibility: "[TODO: Add component responsibility]"

# Dependencies on other components
dependencies: []

# Context rules (system-maintained)
context_rules:
  - ../../orchestration/context/component-rules.md

# API contracts (if applicable)
contracts: []
# - ../../contracts/{name}-api.yaml

# Public API declaration (required for feature coverage testing)
user_facing_features: []
# public_api:
#   - class: MainClass
#     module: components.{name}.src.module
#     methods: [method1, method2]
#     description: Main interface
"""
        manifest_file = self.project_root / "components" / name / "component.yaml"
        manifest_file.write_text(manifest_content)

    def generate_readme(self, name: str, component_type: str, tech_stack: Optional[str]):
        """Generate README.md."""
        readme_content = f"""# {name}

**Type**: {component_type}
**Tech Stack**: {tech_stack or "TBD"}

## Responsibility

[What this component does]

## Development

Component follows TDD/BDD practices with 100% test coverage requirement.

## Testing

```bash
# Run tests
pytest tests/

# Run with coverage
pytest tests/ --cov=src --cov-report=html
```
"""
        readme_file = self.project_root / "components" / name / "README.md"
        readme_file.write_text(readme_content)

    def init_git(self, name: str):
        """Initialize git repository for component."""
        import subprocess

        comp_dir = self.project_root / "components" / name
        try:
            subprocess.run(
                ["git", "init"],
                cwd=comp_dir,
                capture_output=True,
                check=True
            )
            subprocess.run(
                ["git", "add", "."],
                cwd=comp_dir,
                capture_output=True,
                check=True
            )
            subprocess.run(
                ["git", "commit", "-m", f"Initial component setup for {name}"],
                cwd=comp_dir,
                capture_output=True,
                check=True
            )
        except subprocess.CalledProcessError:
            print("⚠️  Git initialization skipped (not critical)")

    def detect_project_language(self) -> str:
        """Detect primary language used in project."""
        # Simple heuristic: check for common files
        if (self.project_root / "pyproject.toml").exists():
            return "python"
        elif (self.project_root / "package.json").exists():
            return "javascript"
        elif (self.project_root / "Cargo.toml").exists():
            return "rust"
        elif (self.project_root / "go.mod").exists():
            return "go"
        else:
            return "python"  # Default

    def report_success(self, name: str, component_type: str, language: str,
                      tech_stack: Optional[str]):
        """Report successful creation."""
        print("\n" + "=" * 60)
        print(f"✅ Component '{name}' created successfully!")
        print("=" * 60)
        print(f"\n📁 Location: components/{name}/")
        print(f"📋 Type: {component_type}")
        print(f"💻 Language: {language}")
        print(f"🔧 Tech Stack: {tech_stack or 'Default'}")
        print(f"\n✅ Created:")
        print(f"  - Directory structure")
        print(f"  - components/{name}/CLAUDE.md")
        print(f"  - components/{name}/component.yaml")
        print(f"  - components/{name}/README.md")
        print(f"  - Test directories")
        if (self.project_root / "components" / name / ".git").exists():
            print(f"  - Local git repository")
        print("\n🚀 Ready for development via Task tool")
        print("=" * 60)


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Create validated component",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s auth_service backend
  %(prog)s auth_service backend --lang=python
  %(prog)s payment_api backend --tech="Python,FastAPI"
        """
    )
    parser.add_argument("name", help="Component name (validated)")
    parser.add_argument(
        "type",
        choices=["backend", "frontend", "library", "microservice"],
        help="Component type"
    )
    parser.add_argument("--lang", help="Primary language (auto-detected if not specified)")
    parser.add_argument("--tech", help="Tech stack (comma-separated)")
    parser.add_argument(
        "--auto-approve",
        action="store_true",
        help="Auto-accept suggestions (orchestrator mode)"
    )

    args = parser.parse_args()

    creator = ComponentCreator()
    try:
        creator.create(
            args.name,
            args.type,
            language=args.lang,
            tech_stack=args.tech,
            auto_approve=args.auto_approve
        )
    except KeyboardInterrupt:
        print("\n\nAborted by user")
        sys.exit(1)


if __name__ == "__main__":
    main()
