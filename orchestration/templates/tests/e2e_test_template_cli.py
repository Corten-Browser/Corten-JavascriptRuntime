#!/usr/bin/env python3
"""
E2E Test Template for CLI Applications

This template tests CLI applications from the user's perspective by:
1. Executing CLI as subprocess (not importing code)
2. Using real test data (generated on-the-fly)
3. Verifying complete workflow (command → database → output files)

This would have caught the Music Analyzer file_hash bug.

Usage:
    1. Copy this template to your component's tests/e2e/ directory
    2. Replace all <PLACEHOLDERS> with your values
    3. Customize test data generators for your component
    4. Run: pytest tests/e2e/ -v
"""

import subprocess
import tempfile
import shutil
import pytest
from pathlib import Path
import pandas as pd
import sqlite3
import os

# ============================================================================
# CONFIGURATION - REPLACE THESE PLACEHOLDERS
# ============================================================================

# Replace with your CLI module path (e.g., 'components.cli_interface')
MODULE_PATH = '<MODULE_PATH>'

# Replace with your primary command (e.g., 'analyze')
PRIMARY_COMMAND = '<COMMAND>'

# Replace with your database filename (e.g., 'music_analysis.db')
DATABASE_FILE = '<DATABASE_FILE>'

# Replace with your primary table name (e.g., 'analysis_results')
TABLE_NAME = '<TABLE_NAME>'

# ============================================================================
# TEST DATA GENERATION
# ============================================================================

@pytest.fixture
def test_data_directory():
    """
    Generate real test data in temporary directory.

    NOTE: This creates REAL data, not mocks. Tests should generate
    their own data rather than relying on pre-existing fixtures.

    CUSTOMIZE THIS: Implement data generation for your component.
    """
    tmpdir = tempfile.mkdtemp(prefix="e2e_test_data_")
    try:
        # TODO: Generate test data here
        # Example: Create sample files your CLI processes
        test_file = Path(tmpdir) / "sample_data.txt"
        test_file.write_text("Sample test data\n")

        yield Path(tmpdir)
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


@pytest.fixture
def clean_environment():
    """
    Ensure clean test environment (no leftover state).

    Cleans:
    - Database files
    - Output files
    - Temporary directories
    """
    # Clean before test
    cleanup_files()

    yield

    # Clean after test
    cleanup_files()


def cleanup_files():
    """Remove all generated test files."""
    patterns = [
        "*.db",
        "*.xlsx",
        "*.csv",
        "test_output_*"
    ]
    for pattern in patterns:
        for file in Path.cwd().glob(pattern):
            file.unlink(missing_ok=True)


# ============================================================================
# E2E TESTS - CLI EXECUTION
# ============================================================================

class TestCLIEndToEnd:
    """
    E2E tests for CLI application.

    These tests execute the CLI as users would run it, using subprocess.
    They verify the COMPLETE workflow, not just component interactions.
    """

    def test_cli_help_command_works(self):
        """
        Verify --help command works.

        This catches:
        - Module path errors (python -m <wrong.path>)
        - Missing __main__.py
        - Import errors on CLI startup
        """
        result = subprocess.run(
            ['python', '-m', MODULE_PATH, '--help'],
            capture_output=True,
            text=True,
            timeout=10
        )

        assert result.returncode == 0, (
            f"CLI --help failed with exit code {result.returncode}\n"
            f"STDOUT: {result.stdout}\n"
            f"STDERR: {result.stderr}"
        )

        assert "usage:" in result.stdout.lower(), (
            "Help text missing 'usage:' section"
        )

    def test_documented_workflow_executes_successfully(
        self,
        test_data_directory,
        clean_environment
    ):
        """
        Execute EXACT workflow from README.md.

        This is the PRIMARY E2E test - it runs what users will run.

        Catches:
        - Missing required fields (file_hash bug)
        - Database constraint violations
        - Output file generation failures
        - Incorrect command-line arguments
        """
        # Run the EXACT command from README
        result = subprocess.run(
            [
                'python', '-m', MODULE_PATH,
                PRIMARY_COMMAND, str(test_data_directory),
                '--output', 'test_output.xlsx'
            ],
            capture_output=True,
            text=True,
            timeout=60,
            cwd=Path.cwd()
        )

        # Verify command succeeded
        assert result.returncode == 0, (
            f"CLI command failed with exit code {result.returncode}\n"
            f"Command: python -m {MODULE_PATH} {PRIMARY_COMMAND} {test_data_directory}\n"
            f"STDOUT: {result.stdout}\n"
            f"STDERR: {result.stderr}"
        )

        # Verify success message in output
        assert "complete" in result.stdout.lower() or "success" in result.stdout.lower(), (
            "No success message in CLI output"
        )

    def test_database_populated_with_required_fields(
        self,
        test_data_directory,
        clean_environment
    ):
        """
        Verify database is populated with ALL required fields.

        This would have caught the file_hash bug:
        - file_hash TEXT NOT NULL in schema
        - CLI didn't include file_hash in results dict
        - Integration tests bypassed CLI, never caught it
        - E2E test runs actual CLI → database insert fails → test fails
        """
        # Run CLI
        subprocess.run(
            [
                'python', '-m', MODULE_PATH,
                PRIMARY_COMMAND, str(test_data_directory)
            ],
            check=True,
            capture_output=True,
            timeout=60
        )

        # Connect to database
        db_path = Path(DATABASE_FILE)
        assert db_path.exists(), f"Database file not created: {db_path}"

        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()

        # Get table schema
        cursor.execute(f"PRAGMA table_info({TABLE_NAME})")
        columns = {row[1]: row for row in cursor.fetchall()}

        # Get NOT NULL columns
        not_null_columns = [
            col_name for col_name, col_info in columns.items()
            if col_info[3] == 1  # notnull flag
        ]

        # Query data
        cursor.execute(f"SELECT * FROM {TABLE_NAME}")
        rows = cursor.fetchall()
        col_names = [desc[0] for desc in cursor.description]

        conn.close()

        # Verify rows exist
        assert len(rows) > 0, "No data in database after CLI execution"

        # Verify ALL NOT NULL columns are populated
        for row in rows:
            row_dict = dict(zip(col_names, row))
            for col in not_null_columns:
                assert col in row_dict, f"Column {col} missing from results"
                assert row_dict[col] is not None, (
                    f"NOT NULL column '{col}' has NULL value\n"
                    f"This indicates CLI is not populating required fields"
                )

    def test_output_file_created_and_valid(
        self,
        test_data_directory,
        clean_environment
    ):
        """
        Verify output file is created with expected structure.

        Catches:
        - Missing output file generation
        - Incorrect file format
        - Missing expected columns
        """
        output_file = Path('test_output.xlsx')

        # Run CLI
        subprocess.run(
            [
                'python', '-m', MODULE_PATH,
                PRIMARY_COMMAND, str(test_data_directory),
                '--output', str(output_file)
            ],
            check=True,
            capture_output=True,
            timeout=60
        )

        # Verify file created
        assert output_file.exists(), f"Output file not created: {output_file}"

        # Verify file structure
        df = pd.read_excel(output_file)

        assert len(df) > 0, "Output file is empty"

        # TODO: Verify expected columns (customize for your component)
        # Example:
        # expected_columns = ['file_path', 'file_name', 'file_hash']
        # missing_columns = [col for col in expected_columns if col not in df.columns]
        # assert not missing_columns, f"Missing columns: {missing_columns}"

    def test_cli_handles_errors_gracefully(self, clean_environment):
        """
        Verify CLI provides helpful error messages for common mistakes.

        Catches:
        - Unhelpful tracebacks
        - Missing error handling
        - Confusing error messages
        """
        # Test: nonexistent directory
        result = subprocess.run(
            [
                'python', '-m', MODULE_PATH,
                PRIMARY_COMMAND, '/nonexistent/directory'
            ],
            capture_output=True,
            text=True,
            timeout=10
        )

        assert result.returncode != 0, "Should fail for nonexistent directory"
        assert "not found" in result.stderr.lower() or "does not exist" in result.stderr.lower(), (
            "Error message not helpful for missing directory"
        )


# ============================================================================
# E2E TESTS - INSTALLATION & PACKAGING
# ============================================================================

class TestInstallationAndPackaging:
    """
    Verify package can be installed and used.

    These tests catch packaging issues that integration tests miss.
    """

    def test_package_installable_in_clean_environment(self):
        """
        Verify package can be installed via pip.

        Catches:
        - Missing setup.py / pyproject.toml
        - Incorrect package structure
        - Missing dependencies in requirements
        """
        project_root = Path(__file__).parent.parent.parent

        # Check packaging files exist
        has_setup = (project_root / "setup.py").exists()
        has_pyproject = (project_root / "pyproject.toml").exists()

        assert has_setup or has_pyproject, (
            "No setup.py or pyproject.toml found\n"
            "Package cannot be installed without these files"
        )

        # Create clean virtual environment
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

            # Install package
            result = subprocess.run(
                [str(pip_path), 'install', '-e', str(project_root)],
                capture_output=True,
                text=True,
                timeout=120
            )

            assert result.returncode == 0, (
                f"Package installation failed\n"
                f"STDERR: {result.stderr}"
            )

            # Verify CLI works in clean environment
            result = subprocess.run(
                [str(python_path), '-m', MODULE_PATH, '--help'],
                capture_output=True,
                text=True,
                timeout=10
            )

            assert result.returncode == 0, (
                "CLI doesn't work after installation in clean environment"
            )


# ============================================================================
# USAGE INSTRUCTIONS
# ============================================================================

"""
To use this template:

1. Replace placeholders at top of file:
   - MODULE_PATH: e.g., 'components.cli_interface'
   - PRIMARY_COMMAND: e.g., 'analyze'
   - DATABASE_FILE: e.g., 'music_analysis.db'
   - TABLE_NAME: e.g., 'analysis_results'

2. Implement test data generator in test_data_directory fixture:
   - Generate REAL files your CLI processes
   - Don't use mocks or pre-existing fixtures

3. Customize test_output_file_created_and_valid():
   - Add your expected output columns
   - Add format-specific validations

4. Run tests:
   ```bash
   pytest tests/e2e/ -v
   ```

These E2E tests will catch:
✅ Missing required fields (file_hash bug)
✅ Module path errors (__main__.py location)
✅ Database constraint violations
✅ Packaging issues
✅ Documentation inaccuracies

REMEMBER: E2E tests execute the CLI exactly as users will run it.
This is the ONLY way to catch application layer bugs that
integration tests miss.
"""
