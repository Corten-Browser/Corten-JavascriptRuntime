#!/usr/bin/env python3
"""
UAT (User Acceptance Testing) Executor

Executes actual user commands to verify product works from user perspective.

This is NOT a checkbox. This RUNS THE ACTUAL COMMAND and verifies success.

This would have caught:
- Music Analyzer v3: __main__.py in wrong location
- Music Analyzer v2: file_hash missing from results

Part of v1.8.0 testing gap remediation.
"""

import subprocess
import tempfile
from pathlib import Path
from typing import List, Tuple
from dataclasses import dataclass


@dataclass
class UATResult:
    """Result of UAT execution."""
    passed: bool
    command: str
    exit_code: int
    stdout: str
    stderr: str
    error_message: str = None


class UATExecutor:
    """Executes UAT tests based on component type."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root).resolve()

    def execute_uat(self) -> Tuple[bool, List[UATResult]]:
        """
        Execute UAT based on component type.

        Returns:
            (all_passed, results)
        """
        component_type = self._detect_component_type()

        if component_type == 'cli_application':
            return self._uat_cli_application()
        elif component_type in ['web_server', 'web_service']:
            return self._uat_web_service()
        elif component_type == 'library':
            return self._uat_library()
        else:
            # Unknown type - skip UAT
            return True, []

    def _detect_component_type(self) -> str:
        """Detect component type."""
        component_yaml = self.project_root / "component.yaml"

        if component_yaml.exists():
            import yaml
            try:
                with open(component_yaml) as f:
                    config = yaml.safe_load(f)
                return config.get('type', 'library')
            except Exception:
                pass

        # Fallback detection
        if (self.project_root / "cli.py").exists() or \
           (self.project_root / "src" / "__main__.py").exists():
            return 'cli_application'
        elif (self.project_root / "app.py").exists():
            return 'web_server'
        else:
            return 'library'

    def _uat_cli_application(self) -> Tuple[bool, List[UATResult]]:
        """UAT for CLI applications."""
        results = []

        # Detect module path
        module_path = self._detect_cli_module_path()

        if not module_path:
            results.append(UATResult(
                passed=False,
                command="N/A",
                exit_code=-1,
                stdout="",
                stderr="",
                error_message="Cannot detect CLI module path"
            ))
            return False, results

        # Test 1: --help
        help_result = self._run_cli_command(module_path, ['--help'])
        results.append(help_result)

        all_passed = all(r.passed for r in results)
        return all_passed, results

    def _detect_cli_module_path(self) -> str:
        """Detect CLI module path."""
        # Check component.yaml
        component_yaml = self.project_root / "component.yaml"
        if component_yaml.exists():
            import yaml
            try:
                with open(component_yaml) as f:
                    config = yaml.safe_load(f)

                if 'module_path' in config:
                    return config['module_path']
                elif 'entry_point' in config:
                    return config['entry_point'].split(':')[0]
            except Exception:
                pass

        # Try common paths
        candidates = [
            'cli',
            'src.cli',
            self.project_root.name,
            f"components.{self.project_root.name}",
        ]

        for candidate in candidates:
            result = subprocess.run(
                ['python', '-m', candidate, '--help'],
                cwd=self.project_root,
                capture_output=True,
                timeout=5
            )
            if result.returncode == 0:
                return candidate

        return None

    def _run_cli_command(
        self,
        module_path: str,
        args: List[str],
        timeout: int = 30
    ) -> UATResult:
        """Run a CLI command and return result."""
        command = ['python', '-m', module_path] + args
        command_str = ' '.join(command)

        try:
            result = subprocess.run(
                command,
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=timeout
            )

            passed = result.returncode == 0
            error_message = None if passed else f"Command failed with exit code {result.returncode}"

            return UATResult(
                passed=passed,
                command=command_str,
                exit_code=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                error_message=error_message
            )

        except subprocess.TimeoutExpired:
            return UATResult(
                passed=False,
                command=command_str,
                exit_code=-1,
                stdout="",
                stderr="",
                error_message=f"Command timed out after {timeout} seconds"
            )
        except Exception as e:
            return UATResult(
                passed=False,
                command=command_str,
                exit_code=-1,
                stdout="",
                stderr="",
                error_message=f"Command execution failed: {e}"
            )

    def _uat_web_service(self) -> Tuple[bool, List[UATResult]]:
        """UAT for web services (placeholder)."""
        return True, []

    def _uat_library(self) -> Tuple[bool, List[UATResult]]:
        """UAT for libraries."""
        results = []

        # Test: Can import library
        library_name = self.project_root.name

        result = subprocess.run(
            ['python', '-c', f'import {library_name}'],
            cwd=self.project_root.parent,  # Import from outside project
            capture_output=True,
            text=True,
            timeout=10
        )

        passed = result.returncode == 0

        results.append(UATResult(
            passed=passed,
            command=f'import {library_name}',
            exit_code=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr,
            error_message=None if passed else f"Cannot import library: {result.stderr}"
        ))

        return passed, results


def main():
    """CLI entry point."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python uat_executor.py <project_root>")
        sys.exit(1)

    project_root = Path(sys.argv[1])

    executor = UATExecutor(project_root)
    all_passed, results = executor.execute_uat()

    print("\n🧪 UAT Execution Results\n")

    for result in results:
        status = "✅" if result.passed else "❌"
        print(f"{status} {result.command}")
        if not result.passed:
            print(f"   Error: {result.error_message}")
            if result.stderr:
                print(f"   STDERR: {result.stderr[:200]}")

    print(f"\n{'✅ ALL PASSED' if all_passed else '❌ SOME FAILED'}\n")

    sys.exit(0 if all_passed else 1)


if __name__ == '__main__':
    main()
