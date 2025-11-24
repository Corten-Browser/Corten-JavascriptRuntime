#!/usr/bin/env python3
"""
E2E Test Template for Libraries/Packages

This template tests libraries from the user's perspective by:
1. Installing package in clean environment
2. Importing as users would
3. Running README examples

Usage:
    1. Copy this template to your component's tests/e2e/ directory
    2. Replace all <PLACEHOLDERS> with your values
    3. Run: pytest tests/e2e/ -v
"""

import subprocess
import tempfile
import sys
from pathlib import Path
import pytest
import re

# ============================================================================
# CONFIGURATION - REPLACE THESE PLACEHOLDERS
# ============================================================================

# Replace with your package name (e.g., 'mylib')
PACKAGE_NAME = '<PACKAGE_NAME>'

# ============================================================================
# E2E TESTS - LIBRARY
# ============================================================================

class TestLibraryEndToEnd:
    """E2E tests for library package."""

    def test_readme_examples_execute(self):
        """
        Execute all code examples from README.

        Catches: README examples that throw exceptions.
        """
        readme_path = Path(__file__).parent.parent.parent / "README.md"

        if not readme_path.exists():
            pytest.skip("No README.md found")

        readme = readme_path.read_text()

        # Extract Python code blocks
        code_blocks = re.findall(r'```python\n(.*?)\n```', readme, re.DOTALL)

        for i, code in enumerate(code_blocks):
            try:
                # Add project root to path
                sys.path.insert(0, str(readme_path.parent))

                # Execute code
                namespace = {}
                exec(code, namespace)

            except Exception as e:
                pytest.fail(
                    f"README example {i+1} failed\n"
                    f"Code:\n{code}\n"
                    f"Error: {e}"
                )
            finally:
                if str(readme_path.parent) in sys.path:
                    sys.path.remove(str(readme_path.parent))

    def test_package_installable_and_importable(self):
        """
        Verify package can be installed and imported.

        Catches:
        - Package structure issues
        - Missing __init__.py
        - Import path mismatches
        """
        project_root = Path(__file__).parent.parent.parent

        with tempfile.TemporaryDirectory() as tmpdir:
            venv_path = Path(tmpdir) / "test_venv"

            # Create venv
            subprocess.run(
                ['python', '-m', 'venv', str(venv_path)],
                check=True,
                timeout=30
            )

            pip_path = venv_path / 'bin' / 'pip'
            python_path = venv_path / 'bin' / 'python'

            # Install
            result = subprocess.run(
                [str(pip_path), 'install', '-e', str(project_root)],
                check=True,
                capture_output=True,
                timeout=120
            )

            assert result.returncode == 0, (
                f"Installation failed\n"
                f"STDERR: {result.stderr.decode()}"
            )

            # Import test
            result = subprocess.run(
                [
                    str(python_path), '-c',
                    f'import {PACKAGE_NAME}; print({PACKAGE_NAME}.__version__)'
                ],
                capture_output=True,
                text=True,
                timeout=10
            )

            assert result.returncode == 0, (
                f"Cannot import package after installation\n"
                f"STDERR: {result.stderr}"
            )

            # Verify version printed
            assert result.stdout.strip() != "", (
                "Package has no __version__ attribute"
            )

    def test_public_api_documented(self):
        """
        Verify public API is documented in README or docs.

        This is a smoke test - checks that main classes/functions
        are mentioned in documentation.
        """
        readme_path = Path(__file__).parent.parent.parent / "README.md"

        if not readme_path.exists():
            pytest.skip("No README.md found")

        readme_text = readme_path.read_text().lower()

        # TODO: Customize with your main classes/functions
        # Example:
        # main_exports = ['MyClass', 'my_function', 'CONSTANT_VALUE']
        #
        # for export in main_exports:
        #     assert export.lower() in readme_text, (
        #         f"Public API '{export}' not documented in README"
        #     )


# ============================================================================
# USAGE INSTRUCTIONS
# ============================================================================

"""
To use this template:

1. Replace placeholders at top of file:
   - PACKAGE_NAME: e.g., 'mylib'

2. Customize test_public_api_documented():
   - Add your main classes/functions to check

3. Ensure your package has:
   - setup.py or pyproject.toml
   - __version__ attribute defined
   - README.md with code examples

4. Run tests:
   ```bash
   pytest tests/e2e/ -v
   ```

These E2E tests catch:
✅ Import path errors
✅ Missing __init__.py files
✅ README examples that fail
✅ Packaging configuration issues
✅ Missing documentation

REMEMBER: For libraries, E2E tests verify the package can be
installed and used exactly as documented. Integration tests
alone may miss packaging issues.
"""
