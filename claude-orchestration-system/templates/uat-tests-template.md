# User Acceptance Testing (UAT) Test Template

**Version**: 1.0
**Purpose**: Provide copy-paste UAT test patterns for verifying products work from user perspective

---

## Overview

User Acceptance Testing (UAT) verifies that your product works from the user's perspective, not just from the developer's internal testing perspective.

**Key Principle**: UAT tests execute the product the same way users will.

---

## When to Use This Template

Use UAT tests for **all project types**:
- ✅ CLI applications (command-line tools)
- ✅ Libraries/packages (importable code)
- ✅ Web servers/APIs (HTTP services)
- ✅ GUI applications (desktop apps)

**When to create UAT tests**: During Phase 5 (Integration Testing) of orchestrated development.

---

## Project Type Detection

Check your `component.yaml` file:

```yaml
type: cli  # or: library, api, web, server, gui, desktop
```

Use the appropriate template section below based on your project type.

---

## UAT Test Patterns

### CLI Applications

**File**: `tests/uat/test_cli_invocation.py`

```python
"""
User Acceptance Tests for CLI Application

Verifies that users can actually run the CLI commands as documented.
"""
import subprocess
import pytest
from pathlib import Path


def test_cli_help_works():
    """Verify --help command works as user would run it."""
    result = subprocess.run(
        ['python', '-m', '<module_name>', '--help'],
        capture_output=True,
        text=True,
        cwd=Path(__file__).parent.parent.parent  # Project root
    )

    assert result.returncode == 0, f"CLI --help failed: {result.stderr}"
    assert "usage:" in result.stdout.lower(), "Help text missing"


def test_cli_version_works():
    """Verify --version command works."""
    result = subprocess.run(
        ['python', '-m', '<module_name>', '--version'],
        capture_output=True,
        text=True,
        cwd=Path(__file__).parent.parent.parent
    )

    assert result.returncode == 0, f"CLI --version failed: {result.stderr}"
    # Verify version format (e.g., "1.0.0")
    assert result.stdout.strip() != "", "No version output"


def test_documented_workflow():
    """Run exact workflow from README."""
    # Replace with EXACT command from your README
    result = subprocess.run(
        ['python', '-m', '<module_name>', '<command>', '<args>'],
        capture_output=True,
        text=True,
        cwd=Path(__file__).parent.parent.parent
    )

    assert result.returncode == 0, f"Command failed: {result.stderr}"
    # Verify expected output
    assert "<expected_text>" in result.stdout


def test_cli_installable():
    """Verify package can be installed and run."""
    import tempfile
    import shutil

    project_root = Path(__file__).parent.parent.parent

    # Check packaging exists
    assert (project_root / "setup.py").exists() or \
           (project_root / "pyproject.toml").exists(), \
           "No setup.py or pyproject.toml found"

    # Create temporary venv
    with tempfile.TemporaryDirectory() as tmpdir:
        venv_path = Path(tmpdir) / "test_venv"

        # Create venv
        subprocess.run(['python', '-m', 'venv', str(venv_path)], check=True)

        # Install package
        pip_path = venv_path / 'bin' / 'pip'
        subprocess.run(
            [str(pip_path), 'install', '-e', str(project_root)],
            check=True
        )

        # Verify can run in clean environment
        python_path = venv_path / 'bin' / 'python'
        result = subprocess.run(
            [str(python_path), '-m', '<module_name>', '--help'],
            capture_output=True
        )
        assert result.returncode == 0, "Cannot run after installation"


def test_entry_point_works():
    """If CLI defines entry point, verify it works after installation."""
    # Only run if entry point is defined in setup.py
    result = subprocess.run(
        ['<command-name>', '--version'],  # Your entry point name
        capture_output=True,
        text=True
    )

    if result.returncode != 0 and "not found" in result.stderr.lower():
        pytest.skip("Entry point not installed (run pip install -e . first)")

    assert result.returncode == 0, f"Entry point failed: {result.stderr}"
```

---

### Libraries/Packages

**File**: `tests/uat/test_library_importability.py`

```python
"""
User Acceptance Tests for Library/Package

Verifies that users can import and use the library as documented.
"""
import subprocess
import sys
import re
import pytest
from pathlib import Path


def test_import_as_documented():
    """Verify import statement from README works."""
    project_root = Path(__file__).parent.parent.parent

    # Add to path
    sys.path.insert(0, str(project_root))

    try:
        # Replace with EXACT import from README
        from <library_name> import <ClassName>
        obj = <ClassName>()
        assert obj is not None
    finally:
        # Clean up
        if str(project_root) in sys.path:
            sys.path.remove(str(project_root))


def test_readme_examples_execute():
    """Execute all Python code examples from README."""
    readme_path = Path(__file__).parent.parent.parent / "README.md"

    if not readme_path.exists():
        pytest.skip("README.md not found")

    readme = readme_path.read_text()

    # Extract all Python code blocks
    code_blocks = re.findall(r'```python\n(.*?)\n```', readme, re.DOTALL)

    if not code_blocks:
        pytest.skip("No Python code examples in README")

    for i, code in enumerate(code_blocks, 1):
        try:
            # Execute in isolated namespace
            namespace = {}
            exec(code, namespace)
        except Exception as e:
            pytest.fail(
                f"README example {i} failed: {e}\n"
                f"Code:\n{code[:200]}"
            )


def test_library_installable():
    """Verify pip install works and library is importable."""
    import tempfile

    project_root = Path(__file__).parent.parent.parent

    # Check packaging exists
    assert (project_root / "setup.py").exists() or \
           (project_root / "pyproject.toml").exists(), \
           "No packaging file found"

    # Create temporary venv
    with tempfile.TemporaryDirectory() as tmpdir:
        venv_path = Path(tmpdir) / "test_venv"

        # Create venv
        subprocess.run(['python', '-m', 'venv', str(venv_path)], check=True)

        # Install library
        pip_path = venv_path / 'bin' / 'pip'
        subprocess.run(
            [str(pip_path), 'install', '-e', str(project_root)],
            check=True
        )

        # Import in clean environment (not local files!)
        python_path = venv_path / 'bin' / 'python'
        result = subprocess.run(
            [str(python_path), '-c', 'import <library_name>; print("<library_name> imported")'],
            capture_output=True,
            text=True
        )

        assert result.returncode == 0, f"Import failed: {result.stderr}"
        assert "<library_name> imported" in result.stdout
```

---

### Web Servers/APIs

**File**: `tests/uat/test_server_deployment.py`

```python
"""
User Acceptance Tests for Web Server/API

Verifies that server can start and respond to requests.
"""
import subprocess
import time
import requests
import pytest
from pathlib import Path


def test_server_starts():
    """Verify server can start using documented command."""
    # Start server as subprocess (how users would run it)
    server = subprocess.Popen(
        ['python', '-m', '<server_module>'],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=Path(__file__).parent.parent.parent
    )

    # Allow startup time
    time.sleep(3)

    try:
        # Verify server is running (didn't crash)
        assert server.poll() is None, "Server crashed on startup"

        # Verify health endpoint responds
        response = requests.get('http://localhost:8000/health', timeout=5)
        assert response.status_code == 200, f"Health check failed: {response.status_code}"

    finally:
        # Clean shutdown
        server.terminate()
        server.wait(timeout=5)


def test_documented_endpoints_exist():
    """Verify all documented API endpoints are accessible."""
    # Start server
    server = subprocess.Popen(
        ['python', '-m', '<server_module>'],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    time.sleep(3)

    try:
        # Test documented endpoints (replace with your actual endpoints)
        endpoints = [
            ('GET', '/health'),
            ('GET', '/api/v1/items'),
            ('POST', '/api/v1/items'),
        ]

        for method, path in endpoints:
            response = requests.request(method, f'http://localhost:8000{path}')

            # Should not be 404 (endpoint must exist)
            assert response.status_code != 404, \
                f"Documented endpoint {method} {path} returns 404"

    finally:
        server.terminate()
        server.wait(timeout=5)


def test_docker_builds():
    """Verify Docker image builds (if Dockerfile exists)."""
    dockerfile = Path(__file__).parent.parent.parent / "Dockerfile"

    if not dockerfile.exists():
        pytest.skip("No Dockerfile found")

    result = subprocess.run(
        ['docker', 'build', '-t', 'test-app', '.'],
        cwd=Path(__file__).parent.parent.parent,
        capture_output=True
    )

    assert result.returncode == 0, f"Docker build failed: {result.stderr.decode()}"


def test_docker_runs():
    """Verify Docker container runs and responds."""
    dockerfile = Path(__file__).parent.parent.parent / "Dockerfile"

    if not dockerfile.exists():
        pytest.skip("No Dockerfile found")

    # Build image
    subprocess.run(['docker', 'build', '-t', 'test-app', '.'], check=True)

    # Run container
    container = subprocess.Popen(
        ['docker', 'run', '-p', '8000:8000', 'test-app']
    )

    time.sleep(5)

    try:
        response = requests.get('http://localhost:8000/health', timeout=5)
        assert response.status_code == 200
    finally:
        container.terminate()
        subprocess.run(['docker', 'stop', str(container.pid)])
```

---

### GUI Applications

**File**: `tests/uat/test_gui_launches.py`

```python
"""
User Acceptance Tests for GUI Application

Verifies that GUI can launch without crashing.

Note: Full GUI testing often requires manual UAT.
These tests catch obvious crashes and import errors.
"""
import subprocess
import time
import pytest
from pathlib import Path


def test_gui_launches_without_crash():
    """Verify GUI can start and doesn't crash immediately."""
    # Start GUI as subprocess
    gui_process = subprocess.Popen(
        ['python', '-m', '<gui_module>'],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=Path(__file__).parent.parent.parent
    )

    # Wait for initialization
    time.sleep(5)

    # Verify process is still running (didn't crash)
    assert gui_process.poll() is None, "GUI crashed on startup"

    # Terminate cleanly
    gui_process.terminate()

    # Check stderr for errors
    _, stderr = gui_process.communicate(timeout=2)
    stderr_text = stderr.decode('utf-8', errors='ignore')

    # Common crash indicators
    crash_patterns = ['Traceback', 'Error:', 'Exception:', 'Segmentation fault']
    for pattern in crash_patterns:
        assert pattern not in stderr_text, \
            f"GUI error on startup: {stderr_text[:200]}"


def test_manual_uat_checklist():
    """Manual UAT checklist for GUI testing."""
    pytest.skip(
        "Manual UAT required for GUI:\n"
        "□ Launch application\n"
        "□ Verify main window appears\n"
        "□ Test File → New\n"
        "□ Test File → Open\n"
        "□ Test primary workflow\n"
        "□ Close application cleanly\n"
        "Mark this test as passed after manual verification."
    )
```

---

## Integration with Test Suite

Add UAT tests to your test suite:

```bash
# Project structure
tests/
├── unit/           # Unit tests
├── integration/    # Integration tests
└── uat/           # User Acceptance Tests ← NEW
    ├── test_cli_invocation.py
    ├── test_library_importability.py
    ├── test_server_deployment.py
    └── test_gui_launches.py
```

**Run UAT tests**:
```bash
pytest tests/uat/ -v
```

**Run all tests**:
```bash
pytest tests/ -v
```

---

## Continuous Integration

Include UAT in CI/CD pipeline:

```yaml
# .github/workflows/ci.yml
- name: Run Unit Tests
  run: pytest tests/unit/

- name: Run Integration Tests
  run: pytest tests/integration/

- name: Run UAT Tests  # ← NEW
  run: pytest tests/uat/

- name: Verify Completion
  run: python orchestration/completion_verifier.py components/<main-app>/
```

---

## Template Usage Instructions

1. **Copy appropriate section** based on your project type
2. **Replace placeholders**:
   - `<module_name>` → Your module name (e.g., `cli_interface`)
   - `<library_name>` → Your library name (e.g., `mylib`)
   - `<server_module>` → Your server module (e.g., `api_server`)
   - `<gui_module>` → Your GUI module (e.g., `app.gui`)
   - `<ClassName>` → Your class names
   - `<command>` → Your CLI commands
   - `<expected_text>` → Expected output text
3. **Add to test suite** in `tests/uat/` directory
4. **Run tests** before marking component complete

---

## Success Criteria

UAT tests pass when:
- ✅ All subprocess commands execute successfully (exit code 0)
- ✅ Documented commands produce expected output
- ✅ Package can be installed in clean environment
- ✅ Import statements work as documented
- ✅ README examples execute without errors

UAT tests fail when:
- ❌ Command returns non-zero exit code
- ❌ ModuleNotFoundError or ImportError
- ❌ Packaging files missing (setup.py/pyproject.toml)
- ❌ Installation in clean venv fails
- ❌ README examples throw exceptions

---

**Remember**: UAT tests verify your product works from the user's perspective, ensuring that documentation matches reality.
