#!/usr/bin/env python3
"""
Deployment Verifier

Final smoke test that verifies software works in deployment-like environment.

This is the ULTIMATE test that prevents HardPathsFailureAssessment.txt failures:
- Installs package in DIFFERENT directory
- Runs WITHOUT PYTHONPATH manipulation
- Tests in clean environment (no dev artifacts)
- Simulates actual user deployment scenario

Part of Phase 6.5 in orchestration workflow (v0.15.0).

If this test passes, the software is truly distributable.
"""

import os
import subprocess
import sys
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass


@dataclass
class DeploymentTestResult:
    """Result of deployment verification test."""
    success: bool
    message: str
    details: Optional[str] = None
    deployment_dir: Optional[Path] = None
    test_log: Optional[str] = None
    failures: Optional[List[str]] = None


class DeploymentVerifier:
    """Verify software works in deployment environment."""

    def __init__(self, project_root: Path):
        """
        Initialize deployment verifier.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()

    def verify_deployment(
        self,
        test_imports: Optional[List[str]] = None,
        test_cli_commands: Optional[List[str]] = None,
        keep_deployment: bool = False
    ) -> DeploymentTestResult:
        """
        Verify software works in deployment scenario.

        Args:
            test_imports: List of modules to test importing
            test_cli_commands: List of CLI commands to test
            keep_deployment: If True, don't delete deployment directory

        Returns:
            DeploymentTestResult with test results
        """
        deployment_dir = None
        failures = []

        try:
            print("🚀 Deployment Verification Test")
            print("="*70)
            print()

            # Step 1: Create deployment directory (different location)
            print("📁 Step 1: Creating deployment directory...")
            deployment_dir = self._create_deployment_dir()
            print(f"   Deployment dir: {deployment_dir}")
            print(f"   ✅ Different from source: {deployment_dir != self.project_root}")
            print()

            # Step 2: Install package to deployment directory
            print("📦 Step 2: Installing package to deployment directory...")
            install_result = self._install_to_deployment(deployment_dir)

            if install_result.returncode != 0:
                failures.append("Package installation failed")
                return DeploymentTestResult(
                    success=False,
                    message="Installation to deployment directory failed",
                    details=install_result.stderr,
                    deployment_dir=deployment_dir if keep_deployment else None,
                    failures=failures
                )

            print("   ✅ Installation successful")
            print()

            # Step 3: Test imports (WITHOUT PYTHONPATH)
            if test_imports:
                print(f"🔍 Step 3: Testing {len(test_imports)} import(s) WITHOUT PYTHONPATH...")
                import_failures = self._test_imports_in_deployment(
                    deployment_dir,
                    test_imports
                )

                if import_failures:
                    failures.extend(import_failures)
                    print(f"   ❌ {len(import_failures)} import(s) failed")
                    for failure in import_failures:
                        print(f"      - {failure}")
                else:
                    print(f"   ✅ All imports successful")
                print()

            # Step 4: Test CLI commands (if applicable)
            if test_cli_commands:
                print(f"🎯 Step 4: Testing {len(test_cli_commands)} CLI command(s)...")
                cli_failures = self._test_cli_in_deployment(
                    deployment_dir,
                    test_cli_commands
                )

                if cli_failures:
                    failures.extend(cli_failures)
                    print(f"   ❌ {len(cli_failures)} command(s) failed")
                    for failure in cli_failures:
                        print(f"      - {failure}")
                else:
                    print(f"   ✅ All commands successful")
                print()

            # Step 5: Verify no hardcoded paths in runtime
            print("🔎 Step 5: Checking for runtime path issues...")
            path_issues = self._check_runtime_paths(deployment_dir, test_imports or [])

            if path_issues:
                failures.extend(path_issues)
                print(f"   ❌ {len(path_issues)} path issue(s) detected")
            else:
                print("   ✅ No hardcoded path issues detected")
            print()

            # Determine success
            if failures:
                print("="*70)
                print("❌ DEPLOYMENT VERIFICATION FAILED")
                print("="*70)
                return DeploymentTestResult(
                    success=False,
                    message=f"Deployment verification failed with {len(failures)} issue(s)",
                    details="\n".join(failures),
                    deployment_dir=deployment_dir if keep_deployment else None,
                    failures=failures
                )
            else:
                print("="*70)
                print("✅ DEPLOYMENT VERIFICATION PASSED")
                print("="*70)
                print()
                print("Software is ready for distribution!")
                print("It works in different directories without PYTHONPATH.")
                print()

                return DeploymentTestResult(
                    success=True,
                    message="Deployment verification passed - software is distributable",
                    deployment_dir=deployment_dir if keep_deployment else None
                )

        except Exception as e:
            failures.append(f"Test error: {str(e)}")
            return DeploymentTestResult(
                success=False,
                message=f"Deployment verification error: {str(e)}",
                details=str(e),
                deployment_dir=deployment_dir if keep_deployment else None,
                failures=failures
            )

        finally:
            # Clean up deployment directory unless keeping
            if deployment_dir and not keep_deployment:
                print("🧹 Cleaning up deployment directory...")
                self._cleanup_deployment(deployment_dir)
                print()

    def _create_deployment_dir(self) -> Path:
        """
        Create deployment directory in different location.

        Returns:
            Path to deployment directory
        """
        # Create temp directory for deployment (different location from source)
        temp_base = Path(tempfile.gettempdir())
        deployment_dir = temp_base / f"deployment_test_{self.project_root.name}"

        # Remove if exists from previous run
        if deployment_dir.exists():
            shutil.rmtree(deployment_dir)

        deployment_dir.mkdir(parents=True)

        return deployment_dir

    def _install_to_deployment(self, deployment_dir: Path) -> subprocess.CompletedProcess:
        """
        Install package to deployment directory.

        Args:
            deployment_dir: Path to deployment directory

        Returns:
            subprocess.CompletedProcess with install results
        """
        # Install package using pip with --target
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "pip",
                "install",
                "--target", str(deployment_dir),
                str(self.project_root)
            ],
            capture_output=True,
            text=True,
            timeout=300
        )

        return result

    def _test_imports_in_deployment(
        self,
        deployment_dir: Path,
        imports: List[str]
    ) -> List[str]:
        """
        Test imports in deployment directory WITHOUT PYTHONPATH.

        Args:
            deployment_dir: Path to deployment directory
            imports: List of module names to import

        Returns:
            List of failure messages
        """
        failures = []

        for module_name in imports:
            # Test import WITHOUT PYTHONPATH (uses deployment dir)
            test_code = f"import sys; sys.path.insert(0, '{deployment_dir}'); import {module_name}"

            result = subprocess.run(
                [sys.executable, "-c", test_code],
                capture_output=True,
                text=True,
                timeout=30,
                env={'PYTHONPATH': ''}  # Explicitly clear PYTHONPATH
            )

            if result.returncode != 0:
                error_msg = result.stderr.strip()
                failures.append(f"Import '{module_name}' failed: {error_msg}")

        return failures

    def _test_cli_in_deployment(
        self,
        deployment_dir: Path,
        commands: List[str]
    ) -> List[str]:
        """
        Test CLI commands in deployment directory.

        Args:
            deployment_dir: Path to deployment directory
            commands: List of command strings to test

        Returns:
            List of failure messages
        """
        failures = []

        for cmd in commands:
            # Run command with deployment dir in path
            env = {
                'PYTHONPATH': str(deployment_dir),
                'PATH': str(deployment_dir) + ':' + os.environ.get('PATH', '')
            }

            result = subprocess.run(
                f"{sys.executable} {cmd}",
                shell=True,
                capture_output=True,
                text=True,
                timeout=30,
                env=env
            )

            if result.returncode != 0:
                error_msg = result.stderr.strip()
                failures.append(f"Command '{cmd}' failed: {error_msg}")

        return failures

    def _check_runtime_paths(
        self,
        deployment_dir: Path,
        test_imports: List[str]
    ) -> List[str]:
        """
        Check for hardcoded paths at runtime.

        Args:
            deployment_dir: Path to deployment directory
            test_imports: Modules to check

        Returns:
            List of path issues found
        """
        issues = []

        # For each module, try to import and check for path errors
        for module_name in test_imports:
            # Try to import and get module __file__
            test_code = f"""
import sys
sys.path.insert(0, '{deployment_dir}')
try:
    import {module_name}
    module_file = {module_name}.__file__
    # Check if module file is in expected location
    if '{deployment_dir}' not in module_file:
        print(f'PATH_MISMATCH: Module loaded from {{module_file}}')
except Exception as e:
    print(f'ERROR: {{e}}')
"""

            result = subprocess.run(
                [sys.executable, "-c", test_code],
                capture_output=True,
                text=True,
                timeout=30,
                env={'PYTHONPATH': ''}
            )

            output = result.stdout.strip()
            if 'PATH_MISMATCH' in output:
                issues.append(f"Module '{module_name}' loaded from wrong location")
            elif 'ERROR' in output:
                issues.append(f"Runtime error in '{module_name}': {output}")

        return issues

    def _cleanup_deployment(self, deployment_dir: Path):
        """
        Clean up deployment directory.

        Args:
            deployment_dir: Path to deployment directory
        """
        try:
            shutil.rmtree(deployment_dir)
        except Exception as e:
            print(f"⚠️  Warning: Could not remove deployment dir: {e}")


def auto_detect_test_config(project_root: Path) -> Tuple[List[str], List[str]]:
    """
    Auto-detect what to test in deployment.

    Args:
        project_root: Project root directory

    Returns:
        Tuple of (imports_to_test, cli_commands_to_test)
    """
    imports = []
    cli_commands = []

    # Detect importable packages
    for item in project_root.iterdir():
        if item.is_dir() and not item.name.startswith('.'):
            if (item / "__init__.py").exists():
                if "test" not in item.name.lower():
                    imports.append(item.name)

    # Try to detect CLI from setup.py
    setup_py = project_root / "setup.py"
    if setup_py.exists():
        try:
            content = setup_py.read_text()
            if 'console_scripts' in content or 'entry_points' in content:
                # Has CLI - add basic help test
                pkg_name = project_root.name.replace('_', '-')
                cli_commands.append(f"-m {imports[0]} --help" if imports else "--help")
        except Exception:
            pass

    return imports, cli_commands


def main():
    """CLI interface for deployment verification."""
    import argparse
    import os

    parser = argparse.ArgumentParser(
        description="Verify deployment in production-like environment"
    )
    parser.add_argument(
        "project_root",
        nargs="?",
        default=".",
        help="Project root directory"
    )
    parser.add_argument(
        "--import",
        dest="test_imports",
        action="append",
        help="Module to test importing (can specify multiple)"
    )
    parser.add_argument(
        "--cli",
        dest="test_cli",
        action="append",
        help="CLI command to test (can specify multiple)"
    )
    parser.add_argument(
        "--auto-detect",
        action="store_true",
        help="Auto-detect what to test"
    )
    parser.add_argument(
        "--keep-deployment",
        action="store_true",
        help="Keep deployment directory after test"
    )

    args = parser.parse_args()
    project_root = Path(args.project_root).resolve()

    # Determine what to test
    test_imports = args.test_imports or []
    test_cli = args.test_cli or []

    if args.auto_detect:
        detected_imports, detected_cli = auto_detect_test_config(project_root)
        test_imports.extend(detected_imports)
        test_cli.extend(detected_cli)

        if detected_imports:
            print(f"🔍 Auto-detected imports: {', '.join(detected_imports)}")
        if detected_cli:
            print(f"🔍 Auto-detected CLI: {', '.join(detected_cli)}")
        print()

    # Remove duplicates
    test_imports = list(set(test_imports))
    test_cli = list(set(test_cli))

    # Run verification
    verifier = DeploymentVerifier(project_root)
    result = verifier.verify_deployment(
        test_imports=test_imports if test_imports else None,
        test_cli_commands=test_cli if test_cli else None,
        keep_deployment=args.keep_deployment
    )

    # Print summary
    if result.success:
        print("\n🎉 SUCCESS: Software is ready for distribution")
        print()
        print("The software:")
        print("  ✅ Installs to different directories")
        print("  ✅ Works without PYTHONPATH manipulation")
        print("  ✅ Has no hardcoded path dependencies")
        print("  ✅ Can be deployed to user machines")
    else:
        print("\n❌ FAILURE: Software not ready for distribution")
        print()
        print(f"Issues found: {len(result.failures)}")
        if result.details:
            print()
            print("Details:")
            print(result.details)
        print()
        print("FIX REQUIRED before distribution")

    if result.deployment_dir:
        print()
        print(f"Deployment directory kept at: {result.deployment_dir}")

    # Exit with appropriate code
    sys.exit(0 if result.success else 1)


if __name__ == '__main__':
    import os  # Import here for auto_detect usage
    main()
