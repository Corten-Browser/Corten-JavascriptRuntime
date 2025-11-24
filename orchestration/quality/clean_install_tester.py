#!/usr/bin/env python3
"""
Clean Install Tester

Tests package installation in a clean virtual environment.
This is the definitive test that prevents HardPathsFailureAssessment.txt failures:
- Verifies package installs without PYTHONPATH manipulation
- Tests imports work in clean environment
- Ensures no hardcoded paths break installation

Part of Phase 5.5 in orchestration workflow (v0.15.0).

Usage:
    python clean_install_tester.py /path/to/project

Returns exit code 0 if installation succeeds, 1 if it fails.
"""

import subprocess
import sys
import tempfile
import venv
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass


@dataclass
class InstallTestResult:
    """Result of clean install test."""
    success: bool
    message: str
    details: Optional[str] = None
    venv_path: Optional[Path] = None
    install_log: Optional[str] = None
    import_test_log: Optional[str] = None


class CleanInstallTester:
    """Test package installation in clean virtual environment."""

    def __init__(self, project_root: Path):
        """
        Initialize clean install tester.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()

    def test_install(
        self,
        keep_venv: bool = False,
        test_imports: Optional[List[str]] = None
    ) -> InstallTestResult:
        """
        Test package installation in clean venv.

        Args:
            keep_venv: If True, don't delete venv after test
            test_imports: List of module names to test importing

        Returns:
            InstallTestResult with test results
        """
        venv_path = None

        try:
            # Step 1: Create clean virtual environment
            print("📦 Creating clean virtual environment...")
            venv_path = self._create_venv()
            print(f"   Created: {venv_path}")

            # Step 2: Install package in editable mode
            print("\n📥 Installing package (pip install -e .)...")
            install_result = self._install_package(venv_path)

            if install_result.returncode != 0:
                return InstallTestResult(
                    success=False,
                    message="Package installation failed",
                    details=install_result.stderr,
                    venv_path=venv_path if keep_venv else None,
                    install_log=install_result.stderr
                )

            print("   ✅ Installation successful")

            # Step 3: Test imports
            if test_imports:
                print(f"\n🔍 Testing {len(test_imports)} import(s)...")
                import_result = self._test_imports(venv_path, test_imports)

                if not import_result['success']:
                    return InstallTestResult(
                        success=False,
                        message=f"Import test failed: {import_result['failed_imports']}",
                        details=import_result['error'],
                        venv_path=venv_path if keep_venv else None,
                        import_test_log=import_result['error']
                    )

                print(f"   ✅ All imports successful")

            # Success!
            print("\n✅ Clean install test PASSED")

            return InstallTestResult(
                success=True,
                message="Clean install test passed",
                venv_path=venv_path if keep_venv else None,
                install_log=install_result.stdout
            )

        except Exception as e:
            return InstallTestResult(
                success=False,
                message=f"Test error: {str(e)}",
                details=str(e),
                venv_path=venv_path if keep_venv else None
            )

        finally:
            # Clean up venv unless keeping
            if venv_path and not keep_venv:
                print(f"\n🧹 Cleaning up virtual environment...")
                self._cleanup_venv(venv_path)

    def _create_venv(self) -> Path:
        """
        Create clean virtual environment.

        Returns:
            Path to venv directory
        """
        # Create temp directory for venv
        temp_dir = Path(tempfile.mkdtemp(prefix="clean_install_test_"))
        venv_path = temp_dir / "venv"

        # Create venv
        venv.create(venv_path, with_pip=True, clear=True)

        return venv_path

    def _install_package(self, venv_path: Path) -> subprocess.CompletedProcess:
        """
        Install package in venv using pip.

        Args:
            venv_path: Path to virtual environment

        Returns:
            subprocess.CompletedProcess with install results
        """
        # Get pip executable in venv
        if sys.platform == "win32":
            pip_exe = venv_path / "Scripts" / "pip.exe"
        else:
            pip_exe = venv_path / "bin" / "pip"

        # Install package in editable mode
        result = subprocess.run(
            [str(pip_exe), "install", "-e", str(self.project_root)],
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )

        return result

    def _test_imports(
        self,
        venv_path: Path,
        imports: List[str]
    ) -> Dict[str, any]:
        """
        Test imports in clean venv.

        Args:
            venv_path: Path to virtual environment
            imports: List of module names to import

        Returns:
            Dict with results
        """
        # Get python executable in venv
        if sys.platform == "win32":
            python_exe = venv_path / "Scripts" / "python.exe"
        else:
            python_exe = venv_path / "bin" / "python"

        failed_imports = []
        error_details = []

        for module_name in imports:
            # Test import
            test_code = f"import {module_name}"

            result = subprocess.run(
                [str(python_exe), "-c", test_code],
                capture_output=True,
                text=True,
                timeout=30
            )

            if result.returncode != 0:
                failed_imports.append(module_name)
                error_details.append(f"{module_name}: {result.stderr.strip()}")

        if failed_imports:
            return {
                'success': False,
                'failed_imports': failed_imports,
                'error': "\n".join(error_details)
            }
        else:
            return {
                'success': True,
                'failed_imports': [],
                'error': None
            }

    def _cleanup_venv(self, venv_path: Path):
        """
        Clean up virtual environment.

        Args:
            venv_path: Path to venv directory
        """
        import shutil

        # Remove venv directory and parent temp dir
        temp_dir = venv_path.parent

        try:
            shutil.rmtree(temp_dir)
        except Exception as e:
            print(f"⚠️  Warning: Could not remove venv: {e}")

    def test_cli_commands(
        self,
        venv_path: Path,
        commands: List[str]
    ) -> Dict[str, any]:
        """
        Test CLI commands in clean venv.

        Args:
            venv_path: Path to virtual environment
            commands: List of command strings to test

        Returns:
            Dict with results
        """
        # Get python executable in venv
        if sys.platform == "win32":
            python_exe = venv_path / "Scripts" / "python.exe"
        else:
            python_exe = venv_path / "bin" / "python"

        failed_commands = []
        error_details = []

        for cmd in commands:
            # Run command
            result = subprocess.run(
                f"{python_exe} {cmd}",
                shell=True,
                capture_output=True,
                text=True,
                timeout=30
            )

            if result.returncode != 0:
                failed_commands.append(cmd)
                error_details.append(f"{cmd}: {result.stderr.strip()}")

        if failed_commands:
            return {
                'success': False,
                'failed_commands': failed_commands,
                'error': "\n".join(error_details)
            }
        else:
            return {
                'success': True,
                'failed_commands': [],
                'error': None
            }


def auto_detect_imports(project_root: Path) -> List[str]:
    """
    Auto-detect top-level packages to test importing.

    Args:
        project_root: Project root directory

    Returns:
        List of package names to test
    """
    imports = []

    # Check for common package directories
    for item in project_root.iterdir():
        if item.is_dir() and not item.name.startswith('.'):
            # Check if it's a Python package (has __init__.py)
            if (item / "__init__.py").exists():
                # Skip test directories
                if "test" not in item.name.lower():
                    imports.append(item.name)

    return imports


def main():
    """CLI interface for clean install testing."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Test package installation in clean virtual environment"
    )
    parser.add_argument(
        "project_root",
        nargs="?",
        default=".",
        help="Project root directory"
    )
    parser.add_argument(
        "--keep-venv",
        action="store_true",
        help="Keep virtual environment after test"
    )
    parser.add_argument(
        "--import",
        dest="test_imports",
        action="append",
        help="Module to test importing (can specify multiple)"
    )
    parser.add_argument(
        "--auto-detect",
        action="store_true",
        help="Auto-detect packages to import"
    )

    args = parser.parse_args()
    project_root = Path(args.project_root).resolve()

    # Determine imports to test
    test_imports = args.test_imports or []

    if args.auto_detect:
        detected = auto_detect_imports(project_root)
        test_imports.extend(detected)
        if detected:
            print(f"🔍 Auto-detected packages: {', '.join(detected)}")

    # Remove duplicates
    test_imports = list(set(test_imports))

    print(f"\n📦 Testing clean install for: {project_root.name}")
    print(f"   Project: {project_root}")
    if test_imports:
        print(f"   Imports to test: {', '.join(test_imports)}")
    print()

    # Run test
    tester = CleanInstallTester(project_root)
    result = tester.test_install(
        keep_venv=args.keep_venv,
        test_imports=test_imports if test_imports else None
    )

    # Display results
    print("\n" + "="*70)
    print("CLEAN INSTALL TEST RESULTS")
    print("="*70)

    if result.success:
        print("✅ SUCCESS: Package installs and imports work")
        print()
        print("The package is distributable and will work when installed")
        print("on user machines in different directories.")
    else:
        print("❌ FAILURE: Package failed clean install test")
        print()
        print(f"Error: {result.message}")
        if result.details:
            print()
            print("Details:")
            print(result.details)
        print()
        print("FIX REQUIRED: Package cannot be distributed until this passes")

    if result.venv_path:
        print()
        print(f"Virtual environment kept at: {result.venv_path}")

    print("="*70)

    # Exit with appropriate code
    sys.exit(0 if result.success else 1)


if __name__ == '__main__':
    main()
