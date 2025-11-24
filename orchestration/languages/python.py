"""
Python language support for distribution-ready validation.

Detects hardcoded paths, validates setup.py/pyproject.toml structure, and verifies
pip installability and deployment.

Part of v0.16.0 multi-language distribution-first redesign.
Refactored from existing v0.15.0 Python-specific code.
"""

import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import List, Tuple, Optional

from .base import LanguageSupport, LanguageInfo, HardcodedPath


class PythonSupport(LanguageSupport):
    """Python distribution support."""

    @property
    def language_name(self) -> str:
        return "python"

    @property
    def display_name(self) -> str:
        return "Python"

    @property
    def file_extensions(self) -> List[str]:
        return [".py"]

    @property
    def package_files(self) -> List[str]:
        return ["setup.py", "pyproject.toml", "setup.cfg"]

    def detect(self, project_root: Path) -> Optional[LanguageInfo]:
        """Detect Python project."""
        # Check for package files
        setup_py = project_root / "setup.py"
        pyproject_toml = project_root / "pyproject.toml"
        setup_cfg = project_root / "setup.cfg"

        has_package_file = setup_py.exists() or pyproject_toml.exists() or setup_cfg.exists()

        # Collect Python source files
        source_files = self._collect_source_files(project_root)

        if not has_package_file and not source_files:
            return None

        # Filter test files
        test_files = [
            f for f in source_files
            if any(pattern in str(f) for pattern in ["/test_", "/tests/", "_test.py"])
        ]

        # Build package files list
        package_files_found = []
        if setup_py.exists():
            package_files_found.append(setup_py)
        if pyproject_toml.exists():
            package_files_found.append(pyproject_toml)
        if setup_cfg.exists():
            package_files_found.append(setup_cfg)

        # Confidence: 1.0 if has package file, 0.7 if just .py files
        confidence = 1.0 if has_package_file else 0.7

        return LanguageInfo(
            name=self.language_name,
            display_name=self.display_name,
            version=self._detect_python_version(),
            package_files=package_files_found,
            source_files=source_files,
            test_files=test_files,
            confidence=confidence
        )

    def find_hardcoded_paths(self, project_root: Path) -> List[HardcodedPath]:
        """Find hardcoded absolute paths in Python files."""
        hardcoded = []

        # Patterns to detect (Unix and Windows)
        patterns = [
            r'/workspaces/',
            r'/home/',
            r'/Users/',
            r'/root/',
            r'/opt/',
            r'C:\\\\',
            r'D:\\\\',
        ]

        # Scan all Python files
        for source_file in self._collect_source_files(project_root):
            try:
                with open(source_file, 'r', encoding='utf-8', errors='ignore') as f:
                    for line_num, line in enumerate(f, 1):
                        # Skip comments
                        stripped = line.strip()
                        if stripped.startswith('#'):
                            continue

                        for pattern in patterns:
                            if re.search(pattern, line):
                                # Extract the path value from string
                                path_match = re.search(r'["\']([^"\']+)["\']', line)
                                path_value = path_match.group(1) if path_match else "unknown"

                                hardcoded.append(HardcodedPath(
                                    file_path=source_file.relative_to(project_root),
                                    line_number=line_num,
                                    line_content=line.strip()[:80],
                                    path_value=path_value,
                                    severity="critical"
                                ))
                                break  # One per line
            except Exception:
                pass

        return hardcoded

    def check_package_structure(self, project_root: Path) -> Tuple[bool, str, List[str]]:
        """Check setup.py/pyproject.toml structure."""
        setup_py = project_root / "setup.py"
        pyproject_toml = project_root / "pyproject.toml"
        issues = []

        if not setup_py.exists() and not pyproject_toml.exists():
            return False, "No package configuration found", [
                "Missing setup.py or pyproject.toml",
                "Run: python orchestration/package_generator.py ."
            ]

        # Check setup.py if exists
        if setup_py.exists():
            try:
                with open(setup_py, 'r', encoding='utf-8') as f:
                    content = f.read()

                # Check for basic setup() call
                if 'setup(' not in content:
                    issues.append("setup.py missing setup() call")

                # Check for hardcoded absolute paths in setup.py
                for pattern in [r'/workspaces/', r'/home/', r'/Users/']:
                    if re.search(pattern, content):
                        issues.append(f"setup.py contains hardcoded path matching {pattern}")

            except Exception as e:
                issues.append(f"Failed to read setup.py: {str(e)}")

        # Check pyproject.toml if exists
        if pyproject_toml.exists():
            try:
                with open(pyproject_toml, 'r', encoding='utf-8') as f:
                    content = f.read()

                # Check for [build-system] or [project] section
                if '[build-system]' not in content and '[project]' not in content:
                    issues.append("pyproject.toml missing [build-system] or [project] section")

            except Exception as e:
                issues.append(f"Failed to read pyproject.toml: {str(e)}")

        if issues:
            return False, f"Found {len(issues)} issue(s) in package configuration", issues

        return True, "Package configuration is valid", []

    def check_import_patterns(self, project_root: Path) -> List[str]:
        """Check for problematic import patterns."""
        issues = []

        for source_file in self._collect_source_files(project_root):
            try:
                with open(source_file, 'r', encoding='utf-8') as f:
                    content = f.read()

                rel_path = source_file.relative_to(project_root)

                # Check for sys.path.append with absolute paths
                if re.search(r'sys\.path\.(append|insert)\s*\([^)]*["\']/', content):
                    issues.append(
                        f"{rel_path}: Uses sys.path with absolute path"
                    )

                # Check for workspace-style imports (components.*.src pattern)
                if re.search(r'from components\.[\\w.]+\\.src import', content):
                    issues.append(
                        f"{rel_path}: Uses workspace-style imports (components.*.src)"
                    )

            except Exception:
                pass

        return issues

    def verify_installability(self, project_root: Path) -> Tuple[bool, str, Optional[str]]:
        """Verify pip install works."""
        if not self.get_package_manager_available():
            return False, "pip not available", "pip command not found"

        setup_py = project_root / "setup.py"
        pyproject_toml = project_root / "pyproject.toml"

        if not setup_py.exists() and not pyproject_toml.exists():
            return False, "No package configuration found", None

        # Try to install in isolated environment
        with tempfile.TemporaryDirectory() as tmpdir:
            test_dir = Path(tmpdir)

            try:
                # Create minimal venv
                venv_dir = test_dir / "venv"
                subprocess.run(
                    [sys.executable, "-m", "venv", str(venv_dir)],
                    capture_output=True,
                    timeout=60
                )

                # Get pip path in venv
                if sys.platform == "win32":
                    pip_path = venv_dir / "Scripts" / "pip.exe"
                else:
                    pip_path = venv_dir / "bin" / "pip"

                # Install package
                result = subprocess.run(
                    [str(pip_path), "install", "--no-deps", str(project_root)],
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                if result.returncode != 0:
                    return False, "pip install failed", result.stderr

                return True, "pip install successful", None

            except subprocess.TimeoutExpired:
                return False, "pip install timed out", "Installation took longer than 5 minutes"
            except Exception as e:
                return False, "pip install error", str(e)

    def verify_deployment(
        self,
        project_root: Path,
        deployment_dir: Path,
        test_imports: Optional[List[str]] = None
    ) -> Tuple[bool, str, List[str]]:
        """Verify package works in different directory."""
        failures = []

        if not self.get_package_manager_available():
            return False, "pip not available", ["pip command not found"]

        # Install package to deployment directory
        try:
            result = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "pip",
                    "install",
                    "--target", str(deployment_dir),
                    "--no-deps",
                    str(project_root)
                ],
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode != 0:
                return False, "Failed to install to deployment directory", [result.stderr]

        except subprocess.TimeoutExpired:
            return False, "pip install timed out", ["Installation took longer than 5 minutes"]
        except Exception as e:
            return False, "pip install error", [str(e)]

        # Test imports if specified
        if test_imports:
            for module_name in test_imports:
                test_code = f"import sys; sys.path.insert(0, '{deployment_dir}'); import {module_name}"

                try:
                    result = subprocess.run(
                        [sys.executable, "-c", test_code],
                        capture_output=True,
                        text=True,
                        timeout=30,
                        env={'PYTHONPATH': ''}  # Clear PYTHONPATH
                    )

                    if result.returncode != 0:
                        failures.append(f"Import '{module_name}' failed: {result.stderr}")

                except subprocess.TimeoutExpired:
                    failures.append(f"Import '{module_name}' timed out")
                except Exception as e:
                    failures.append(f"Import '{module_name}' error: {str(e)}")

        if failures:
            return False, f"{len(failures)} import(s) failed in deployment", failures

        return True, "Deployment verification passed", []

    def get_package_manager_available(self) -> bool:
        """Check if pip is available."""
        try:
            subprocess.run(
                [sys.executable, "-m", "pip", "--version"],
                capture_output=True,
                timeout=5
            )
            return True
        except Exception:
            return False

    def _detect_python_version(self) -> Optional[str]:
        """Detect Python version."""
        return f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"

    def _collect_source_files(self, project_root: Path) -> List[Path]:
        """Collect all Python source files, excluding build/cache directories."""
        source_files = []

        # Directories to exclude
        exclude_dirs = {
            "__pycache__",
            ".git",
            ".venv",
            "venv",
            "env",
            ".tox",
            "dist",
            "build",
            "*.egg-info",
            ".pytest_cache",
            ".mypy_cache",
            "node_modules"
        }

        for py_file in project_root.glob("**/*.py"):
            # Check if file is in excluded directory
            if any(excluded in py_file.parts for excluded in exclude_dirs):
                continue
            # Skip if contains .egg-info
            if any(".egg-info" in part for part in py_file.parts):
                continue
            source_files.append(py_file)

        return source_files
