#!/usr/bin/env python3
"""
Run actual user-facing commands to verify they work.
This is the ULTIMATE test - what the user actually experiences.
"""
import subprocess
import json
from pathlib import Path


def run_smoke_tests(project_dir: Path) -> dict:
    """
    Run smoke tests based on project type.
    These test ACTUAL USER COMMANDS, not internal logic.
    """
    results = {
        "all_passed": True,
        "tests_run": 0,
        "tests_passed": 0,
        "failures": []
    }

    # Detect project type and define smoke tests
    smoke_tests = []

    # Check for Python package
    if (project_dir / "setup.py").exists() or (project_dir / "pyproject.toml").exists():
        # Find the main package
        packages = [
            d for d in project_dir.iterdir()
            if d.is_dir() and (d / "__init__.py").exists()
        ]

        for package in packages:
            if package.name.startswith("_") or package.name in ["tests", "test"]:
                continue

            # Test 1: Package imports successfully
            smoke_tests.append({
                "name": f"Import {package.name}",
                "command": f"python -c 'import {package.name}'",
                "expected_exit_code": 0
            })

            # Test 2: Package --help works (if CLI)
            if (package / "__main__.py").exists():
                smoke_tests.append({
                    "name": f"CLI --help for {package.name}",
                    "command": f"python -m {package.name} --help",
                    "expected_exit_code": 0
                })

    # Check for Rust/Cargo project
    if (project_dir / "Cargo.toml").exists():
        smoke_tests.append({
            "name": "Cargo build",
            "command": "cargo build 2>&1",
            "expected_exit_code": 0,
            "timeout": 120
        })

    # Check for custom smoke tests file
    smoke_tests_file = project_dir / "orchestration" / "smoke_tests.json"
    if smoke_tests_file.exists():
        try:
            custom_tests = json.loads(smoke_tests_file.read_text())
            smoke_tests.extend(custom_tests.get("tests", []))
        except (json.JSONDecodeError, KeyError):
            pass

    if not smoke_tests:
        print("No smoke tests defined for this project type")
        results["all_passed"] = True
        return results

    # Run all smoke tests
    print(f"Running {len(smoke_tests)} smoke tests...")

    for test in smoke_tests:
        results["tests_run"] += 1

        print(f"\n  Running: {test['name']}")
        print(f"  Command: {test['command']}")

        try:
            result = subprocess.run(
                test["command"],
                shell=True,
                capture_output=True,
                text=True,
                timeout=test.get("timeout", 30),
                cwd=project_dir
            )

            expected_code = test.get("expected_exit_code", 0)

            if result.returncode == expected_code:
                print(f"  PASSED")
                results["tests_passed"] += 1
            else:
                print(f"  FAILED (exit code {result.returncode}, expected {expected_code})")
                print(f"     stdout: {result.stdout[:200]}")
                print(f"     stderr: {result.stderr[:200]}")
                results["failures"].append({
                    "test": test["name"],
                    "reason": f"Exit code {result.returncode} != {expected_code}",
                    "stdout": result.stdout[:500],
                    "stderr": result.stderr[:500]
                })
                results["all_passed"] = False

        except subprocess.TimeoutExpired:
            print(f"  FAILED (timeout)")
            results["failures"].append({
                "test": test["name"],
                "reason": "Timeout"
            })
            results["all_passed"] = False

        except Exception as e:
            print(f"  FAILED ({str(e)})")
            results["failures"].append({
                "test": test["name"],
                "reason": str(e)
            })
            results["all_passed"] = False

    return results


if __name__ == "__main__":
    import sys
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    results = run_smoke_tests(target)

    print(f"\n\nSMOKE TEST SUMMARY:")
    print(f"  Passed: {results['tests_passed']}/{results['tests_run']}")

    if results["all_passed"]:
        print("  ALL SMOKE TESTS PASSED")
        sys.exit(0)
    else:
        print("  SOME SMOKE TESTS FAILED")
        sys.exit(1)
