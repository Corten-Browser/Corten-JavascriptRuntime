#!/usr/bin/env python3
"""
Testing Infrastructure Generator

Generates comprehensive testing infrastructure for onboarded components including:
- E2E tests for CLI applications and web servers
- Test directory structure
- Test data generators
- Language-specific test configurations

Version: 1.0.0
"""

import sys
from pathlib import Path
from typing import Dict, List, Optional
import json


class TestInfrastructureGenerator:
    """Generates testing infrastructure for components"""

    def __init__(self, project_dir: Path):
        """
        Initialize test infrastructure generator.

        Args:
            project_dir: Root directory of project
        """
        self.project_dir = Path(project_dir).resolve()
        self.template_dir = Path(__file__).parent.parent / "templates" / "tests"

    def generate_for_component(self, component: Dict) -> List[Path]:
        """
        Generate testing infrastructure for a component.

        Args:
            component: Component metadata dict with keys:
                      name, type, language, entry_point

        Returns:
            List of created file paths
        """
        created_files = []

        component_name = component.get("name", "")
        component_type = component.get("type", "generic")
        language = component.get("language", "python")
        component_dir = self.project_dir / "components" / component_name

        if not component_dir.exists():
            raise ValueError(f"Component directory not found: {component_dir}")

        # Create test directories
        test_dirs = self.create_test_directories(component_dir)
        created_files.extend(test_dirs)

        # Copy test templates
        template_files = self.copy_test_templates(component_type, language, component_dir)
        created_files.extend(template_files)

        # Generate E2E tests if applicable
        if component_type in ['cli_application', 'web_server']:
            e2e_test = self.generate_e2e_tests(component)
            if e2e_test:
                created_files.append(e2e_test)

        # Generate test data generators
        test_data_gen = self.generate_test_data_generators(component)
        if test_data_gen:
            created_files.append(test_data_gen)

        # Create test configuration
        config = self.create_test_config(component_type, language, component_dir)
        if config:
            created_files.append(config)

        return created_files

    def create_test_directories(self, component_dir: Path) -> List[Path]:
        """Create test directory structure"""
        dirs_to_create = [
            component_dir / "tests",
            component_dir / "tests" / "unit",
            component_dir / "tests" / "integration",
            component_dir / "tests" / "e2e",
            component_dir / "tests" / "fixtures"
        ]

        created = []
        for dir_path in dirs_to_create:
            dir_path.mkdir(parents=True, exist_ok=True)

            # Create __init__.py for Python
            init_file = dir_path / "__init__.py"
            if not init_file.exists():
                init_file.write_text("# Test package\n")
                created.append(init_file)

        return created

    def copy_test_templates(self, component_type: str, language: str, dest: Path) -> List[Path]:
        """Copy language-specific test templates"""
        created = []

        # Map language to template file
        template_files = {
            "python": ["test_example_unit.py", "test_example_integration.py"],
            "typescript": ["example.test.ts"],
            "rust": ["test_example.rs"],
            "go": ["example_test.go"]
        }

        if language not in template_files:
            return created

        lang_template_dir = self.template_dir / language
        if not lang_template_dir.exists():
            return created

        for template_name in template_files.get(language, []):
            template_path = lang_template_dir / template_name
            if template_path.exists():
                dest_path = dest / "tests" / "unit" / template_name
                dest_path.write_text(template_path.read_text())
                created.append(dest_path)

        return created

    def generate_e2e_tests(self, component: Dict) -> Optional[Path]:
        """Generate E2E tests for CLI or web server components"""
        component_name = component.get("name", "")
        component_type = component.get("type", "")
        language = component.get("language", "python")
        entry_point = component.get("entry_point", "")

        component_dir = self.project_dir / "components" / component_name

        if component_type == "cli_application":
            return self._generate_cli_e2e_test(component_name, entry_point, language, component_dir)
        elif component_type == "web_server":
            return self._generate_web_e2e_test(component_name, entry_point, language, component_dir)

        return None

    def _generate_cli_e2e_test(self, component_name: str, entry_point: str,
                                language: str, component_dir: Path) -> Path:
        """Generate CLI E2E test"""
        if language == "python":
            test_content = f'''"""
E2E tests for {component_name} CLI application

Tests the actual command-line interface end-to-end.
"""

import subprocess
import pytest
from pathlib import Path


def test_cli_help():
    """Test --help flag"""
    result = subprocess.run(
        ["python", "-m", "components.{component_name}", "--help"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    assert "usage:" in result.stdout.lower() or "help" in result.stdout.lower()


def test_cli_version():
    """Test --version flag"""
    result = subprocess.run(
        ["python", "-m", "components.{component_name}", "--version"],
        capture_output=True,
        text=True
    )
    # Should not crash
    assert result.returncode in [0, 1]  # Some CLIs exit 1 for --version


def test_cli_invalid_command():
    """Test invalid command handling"""
    result = subprocess.run(
        ["python", "-m", "components.{component_name}", "invalid_command_xyz"],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0
    assert "error" in result.stderr.lower() or "invalid" in result.stderr.lower()


# TODO: Add component-specific E2E tests
# Example:
# def test_main_functionality():
#     result = subprocess.run(
#         ["python", "-m", "components.{component_name}", "command", "args"],
#         capture_output=True,
#         text=True
#     )
#     assert result.returncode == 0
#     assert "expected output" in result.stdout
'''
        else:
            # Generic template for other languages
            test_content = f"// TODO: Add E2E tests for {component_name}\n"

        test_file = component_dir / "tests" / "e2e" / f"test_{component_name}_e2e.py"
        test_file.write_text(test_content)
        return test_file

    def _generate_web_e2e_test(self, component_name: str, entry_point: str,
                                language: str, component_dir: Path) -> Path:
        """Generate web server E2E test"""
        if language == "python":
            test_content = f'''"""
E2E tests for {component_name} web server

Tests the actual HTTP endpoints end-to-end.
"""

import pytest
import requests
from time import sleep
import subprocess
import signal


@pytest.fixture(scope="module")
def server():
    """Start server for testing"""
    proc = subprocess.Popen(
        ["python", "-m", "components.{component_name}"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    # Wait for server to start
    sleep(2)

    yield "http://localhost:8000"  # Adjust port as needed

    # Cleanup
    proc.send_signal(signal.SIGTERM)
    proc.wait()


def test_server_health(server):
    """Test server health endpoint"""
    response = requests.get(f"{{server}}/health")
    assert response.status_code == 200


def test_server_root(server):
    """Test root endpoint"""
    response = requests.get(server)
    assert response.status_code in [200, 404]  # Either works or returns 404


# TODO: Add component-specific E2E tests
# Example:
# def test_api_endpoint(server):
#     response = requests.get(f"{{server}}/api/endpoint")
#     assert response.status_code == 200
#     assert response.json()["status"] == "success"
'''
        else:
            test_content = f"// TODO: Add E2E tests for {component_name}\n"

        test_file = component_dir / "tests" / "e2e" / f"test_{component_name}_e2e.py"
        test_file.write_text(test_content)
        return test_file

    def generate_test_data_generators(self, component: Dict) -> Optional[Path]:
        """Generate test data generators"""
        component_name = component.get("name", "")
        language = component.get("language", "python")
        component_dir = self.project_dir / "components" / component_name

        if language != "python":
            return None  # Only Python for now

        generator_content = f'''"""
Test Data Generators for {component_name}

Provides factories and fixtures for generating test data.
"""

from dataclasses import dataclass
from typing import List, Dict, Any
import random
import string


class TestDataGenerator:
    """Generates test data for {component_name}"""

    @staticmethod
    def random_string(length: int = 10) -> str:
        """Generate random string"""
        return ''.join(random.choices(string.ascii_letters + string.digits, k=length))

    @staticmethod
    def random_email() -> str:
        """Generate random email"""
        username = TestDataGenerator.random_string(8)
        domain = TestDataGenerator.random_string(6)
        return f"{{username}}@{{domain}}.com"

    @staticmethod
    def random_number(min_val: int = 0, max_val: int = 100) -> int:
        """Generate random number"""
        return random.randint(min_val, max_val)

    # TODO: Add component-specific generators
    # Example:
    # @staticmethod
    # def generate_user() -> Dict[str, Any]:
    #     return {{
    #         "id": TestDataGenerator.random_number(1, 1000),
    #         "name": TestDataGenerator.random_string(12),
    #         "email": TestDataGenerator.random_email()
    #     }}


# Example usage in tests:
# from tests.fixtures.generators import TestDataGenerator
#
# def test_something():
#     test_email = TestDataGenerator.random_email()
#     assert "@" in test_email
'''

        generator_file = component_dir / "tests" / "fixtures" / "generators.py"
        generator_file.write_text(generator_content)
        return generator_file

    def create_test_config(self, component_type: str, language: str,
                           component_dir: Path) -> Optional[Path]:
        """Create test configuration file"""
        if language == "python":
            return self._create_pytest_config(component_dir)
        elif language in ["javascript", "typescript"]:
            return self._create_jest_config(component_dir)

        return None

    def _create_pytest_config(self, component_dir: Path) -> Path:
        """Create pytest.ini configuration"""
        config_content = '''[pytest]
testpaths = tests
python_files = test_*.py
python_classes = Test*
python_functions = test_*
addopts =
    -v
    --strict-markers
    --tb=short
    --cov=src
    --cov-report=term-missing
    --cov-report=html

markers =
    unit: Unit tests
    integration: Integration tests
    e2e: End-to-end tests
    slow: Slow running tests
'''

        config_file = component_dir / "pytest.ini"
        config_file.write_text(config_content)
        return config_file

    def _create_jest_config(self, component_dir: Path) -> Path:
        """Create jest.config.js configuration"""
        config_content = '''module.exports = {
  testEnvironment: 'node',
  testMatch: ['**/*.test.ts', '**/*.test.js'],
  coverageDirectory: 'coverage',
  collectCoverageFrom: [
    'src/**/*.{ts,js}',
    '!src/**/*.d.ts'
  ],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  }
};
'''

        config_file = component_dir / "jest.config.js"
        config_file.write_text(config_content)
        return config_file


def main():
    """CLI entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Test Infrastructure Generator")
    parser.add_argument("project_dir", nargs="?", default=".",
                       help="Project directory")
    parser.add_argument("--component", required=True,
                       help="Component name")
    parser.add_argument("--type", default="generic",
                       choices=["cli_application", "web_server", "library", "generic"],
                       help="Component type")
    parser.add_argument("--language", default="python",
                       choices=["python", "javascript", "typescript", "rust", "go"],
                       help="Programming language")
    parser.add_argument("--entry-point", default="",
                       help="Entry point (e.g., __main__.py)")

    args = parser.parse_args()

    generator = TestInfrastructureGenerator(Path(args.project_dir))

    component = {
        "name": args.component,
        "type": args.type,
        "language": args.language,
        "entry_point": args.entry_point
    }

    try:
        created_files = generator.generate_for_component(component)
        print(f"\n✅ Generated test infrastructure for {args.component}")
        print(f"\nCreated {len(created_files)} files:")
        for file_path in created_files:
            rel_path = file_path.relative_to(Path(args.project_dir))
            print(f"  - {rel_path}")

        return 0
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
