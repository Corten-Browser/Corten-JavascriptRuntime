#!/usr/bin/env python3
"""
Import Template Generator

Auto-generates __init__.py files for components to make imports "just work".

This solves part of the v0.2.0 import issues where agents wasted time
configuring imports instead of implementing features.

Generates:
- components/<name>/__init__.py (root package)
- components/<name>/src/__init__.py (source package)
- components/<name>/tests/__init__.py (test package)
- components/<name>/_internal/__init__.py (private modules)

Part of v0.3.0 completion guarantee system.
"""

import sys
from pathlib import Path
from typing import List, Optional
import re


class ImportTemplateGenerator:
    """Generates __init__.py files for component packages."""

    def __init__(self, project_root: Path):
        """
        Initialize generator.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()

    def setup_component_imports(self, component_path: Path) -> None:
        """
        Set up all __init__.py files for a component.

        Args:
            component_path: Path to component directory
        """
        component_path = Path(component_path).resolve()
        component_name = component_path.name

        print(f"🔧 Setting up imports for: {component_name}")

        # Read component manifest to get public API
        manifest_path = component_path / "component.yaml"
        public_api = self._read_public_api(manifest_path)

        # Generate __init__.py files
        self._generate_root_init(component_path, component_name, public_api)
        self._generate_src_init(component_path, public_api)
        self._generate_tests_init(component_path)
        self._generate_internal_init(component_path)

        print(f"✅ Import setup complete for {component_name}")

    def _read_public_api(self, manifest_path: Path) -> dict:
        """
        Read public API definition from component manifest.

        Args:
            manifest_path: Path to component.yaml

        Returns:
            Dict with 'modules', 'classes', 'functions' keys
        """
        if not manifest_path.exists():
            # Default public API
            return {
                'modules': ['api'],
                'classes': [],
                'functions': []
            }

        try:
            import yaml
            with open(manifest_path, 'r') as f:
                manifest = yaml.safe_load(f)

            public_api = manifest.get('public_api', {})

            return {
                'modules': public_api.get('modules', ['api']),
                'classes': public_api.get('classes', []),
                'functions': public_api.get('functions', [])
            }

        except ImportError:
            print("⚠️  PyYAML not installed, using default public API")
            return {
                'modules': ['api'],
                'classes': [],
                'functions': []
            }

        except Exception as e:
            print(f"⚠️  Error reading manifest: {e}, using default public API")
            return {
                'modules': ['api'],
                'classes': [],
                'functions': []
            }

    def _generate_root_init(self, component_path: Path, component_name: str, public_api: dict) -> None:
        """
        Generate component root __init__.py.

        This file exports the public API for the component.

        Args:
            component_path: Path to component directory
            component_name: Name of component
            public_api: Public API definition
        """
        init_path = component_path / "__init__.py"

        # Read version from manifest or default
        version = self._get_component_version(component_path)

        # Build imports from public API modules
        imports = []
        exports = []

        for module_name in public_api['modules']:
            # Check if module exists
            module_file = component_path / "src" / f"{module_name}.py"
            if not module_file.exists():
                # Try as package
                module_dir = component_path / "src" / module_name
                if not module_dir.exists():
                    continue

            # Import all public symbols from this module
            imports.append(f"from .src.{module_name} import *")

        # Explicit class exports
        for class_name in public_api['classes']:
            exports.append(class_name)

        # Explicit function exports
        for func_name in public_api['functions']:
            exports.append(func_name)

        # Generate content
        content = self._format_root_init(
            component_name=component_name,
            version=version,
            imports=imports,
            exports=exports
        )

        # Write file
        init_path.write_text(content)
        print(f"  ✓ Created {init_path.relative_to(component_path)}")

    def _generate_src_init(self, component_path: Path, public_api: dict) -> None:
        """
        Generate src/__init__.py.

        This makes src/ a proper Python package.

        Args:
            component_path: Path to component directory
            public_api: Public API definition
        """
        src_path = component_path / "src"
        if not src_path.exists():
            src_path.mkdir(parents=True)

        init_path = src_path / "__init__.py"

        # List Python modules in src/
        modules = [
            f.stem for f in src_path.glob("*.py")
            if f.stem != "__init__" and not f.stem.startswith("_")
        ]

        # Generate content
        content = self._format_src_init(modules)

        # Write file
        init_path.write_text(content)
        print(f"  ✓ Created {init_path.relative_to(component_path)}")

    def _generate_tests_init(self, component_path: Path) -> None:
        """
        Generate tests/__init__.py.

        This makes tests/ a proper Python package for pytest discovery.

        Args:
            component_path: Path to component directory
        """
        tests_path = component_path / "tests"
        if not tests_path.exists():
            tests_path.mkdir(parents=True)

        init_path = tests_path / "__init__.py"

        # Generate content
        content = self._format_tests_init()

        # Write file
        init_path.write_text(content)
        print(f"  ✓ Created {init_path.relative_to(component_path)}")

    def _generate_internal_init(self, component_path: Path) -> None:
        """
        Generate _internal/__init__.py if _internal/ exists.

        This marks private implementation modules.

        Args:
            component_path: Path to component directory
        """
        internal_path = component_path / "src" / "_internal"
        if not internal_path.exists():
            return  # Only create if _internal/ directory exists

        init_path = internal_path / "__init__.py"

        # Generate content
        content = self._format_internal_init()

        # Write file
        init_path.write_text(content)
        print(f"  ✓ Created {init_path.relative_to(component_path)}")

    def _format_root_init(self, component_name: str, version: str, imports: List[str], exports: List[str]) -> str:
        """Format root __init__.py content."""
        imports_block = "\n".join(imports) if imports else "# No public API modules exported"

        exports_block = f"__all__ = {exports}" if exports else "# __all__ defined by imported modules"

        return f'''"""
{component_name} Component

This package provides the public API for the {component_name} component.
"""

__version__ = "{version}"

# Import public API
{imports_block}

# Explicit exports
{exports_block}
'''

    def _format_src_init(self, modules: List[str]) -> str:
        """Format src/__init__.py content."""
        if not modules:
            return '''"""
Source package.
"""
'''

        modules_list = ", ".join(f'"{m}"' for m in modules)

        return f'''"""
Source package containing implementation modules.
"""

# Available modules: {", ".join(modules)}

__all__ = [{modules_list}]
'''

    def _format_tests_init(self) -> str:
        """Format tests/__init__.py content."""
        return '''"""
Test package.

This package contains all tests for the component.
Configured for pytest discovery.
"""
'''

    def _format_internal_init(self) -> str:
        """Format _internal/__init__.py content."""
        return '''"""
Internal implementation modules.

⚠️  DO NOT import from this package outside of this component.
These are private implementation details subject to change.
"""
'''

    def _get_component_version(self, component_path: Path) -> str:
        """
        Get component version from manifest.

        Args:
            component_path: Path to component directory

        Returns:
            Version string (default: "0.1.0")
        """
        manifest_path = component_path / "component.yaml"

        if not manifest_path.exists():
            return "0.1.0"

        try:
            import yaml
            with open(manifest_path, 'r') as f:
                manifest = yaml.safe_load(f)

            return manifest.get('version', '0.1.0')

        except:
            return "0.1.0"

    def generate_pytest_config(self, component_path: Path) -> None:
        """
        Generate pytest configuration for component.

        Creates pytest.ini with proper settings.

        Args:
            component_path: Path to component directory
        """
        pytest_ini = component_path / "pytest.ini"

        content = """[pytest]
# Pytest configuration for this component

# Python path (allow imports from src/)
pythonpath = . src

# Test discovery
python_files = test_*.py *_test.py
python_classes = Test*
python_functions = test_*

# Test paths
testpaths = tests

# Output
addopts = -v --tb=short

# Coverage settings (if pytest-cov installed)
# addopts = --cov=src --cov-report=term-missing --cov-report=html
"""

        pytest_ini.write_text(content)
        print(f"  ✓ Created {pytest_ini.relative_to(component_path)}")


def setup_all_components(project_root: Path) -> None:
    """
    Set up imports for all components in project.

    Args:
        project_root: Path to project root
    """
    components_dir = project_root / "components"

    if not components_dir.exists():
        print("❌ No components/ directory found")
        return

    generator = ImportTemplateGenerator(project_root)

    # Find all components
    components = [d for d in components_dir.iterdir() if d.is_dir() and not d.name.startswith('.')]

    if not components:
        print("❌ No components found in components/")
        return

    print(f"Found {len(components)} component(s)")
    print()

    for component_dir in sorted(components):
        try:
            generator.setup_component_imports(component_dir)
            print()
        except Exception as e:
            print(f"❌ Error setting up {component_dir.name}: {e}")
            print()

    print(f"✅ Import setup complete for {len(components)} component(s)")


def main():
    """CLI interface for import template generator."""
    if len(sys.argv) < 2:
        print("Usage: import_template_generator.py <component_path>")
        print("   or: import_template_generator.py --all")
        print("\nAuto-generates __init__.py files for components.")
        print("\nExamples:")
        print("  python import_template_generator.py components/audio_processor")
        print("  python import_template_generator.py --all  # All components")
        sys.exit(1)

    if sys.argv[1] == "--all":
        project_root = Path.cwd()
        setup_all_components(project_root)
        sys.exit(0)

    component_path = Path(sys.argv[1])

    if not component_path.exists():
        print(f"❌ Component not found: {component_path}")
        sys.exit(1)

    project_root = Path.cwd()
    generator = ImportTemplateGenerator(project_root)

    try:
        generator.setup_component_imports(component_path)
        generator.generate_pytest_config(component_path)
        sys.exit(0)
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()
