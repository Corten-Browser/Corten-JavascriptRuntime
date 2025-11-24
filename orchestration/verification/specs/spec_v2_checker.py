#!/usr/bin/env python3
"""
Enhanced specification coverage checker for machine-readable YAML specs.
Provides objective, binary pass/fail for each feature.
"""
import yaml
import json
import subprocess
import re
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def load_spec(spec_file: Path) -> dict:
    """Load and validate specification file."""
    with open(spec_file) as f:
        spec = yaml.safe_load(f)

    # Validate against schema
    required_keys = ["name", "version", "features", "completion_criteria"]
    for key in required_keys:
        if key not in spec:
            raise ValueError(f"Specification missing required key: {key}")

    return spec


def verify_feature(feature: dict, project_dir: Path) -> tuple[bool, str]:
    """
    Verify a single feature is implemented.
    Returns (passed, evidence/reason).
    """
    verification = feature["verification"]
    v_type = verification["type"]

    if v_type == "test_pattern":
        # Count tests matching pattern
        pattern = verification["pattern"]
        min_tests = verification.get("min_tests", 1)

        # Search for test functions
        test_count = 0
        for test_file in project_dir.rglob("*test*.py"):
            if "__pycache__" in str(test_file):
                continue
            try:
                content = test_file.read_text()
                matches = re.findall(rf'def ({pattern})', content)
                test_count += len(matches)
            except Exception:
                pass

        for test_file in project_dir.rglob("*test*.rs"):
            if "target" in str(test_file):
                continue
            try:
                content = test_file.read_text()
                matches = re.findall(rf'fn ({pattern})', content)
                test_count += len(matches)
            except Exception:
                pass

        if test_count >= min_tests:
            return True, f"Found {test_count} tests (min: {min_tests})"
        else:
            return False, f"Found only {test_count} tests (need: {min_tests})"

    elif v_type == "file_exists":
        pattern = verification["pattern"]
        matches = list(project_dir.rglob(pattern))
        if matches:
            return True, f"File exists: {matches[0]}"
        else:
            return False, f"File not found: {pattern}"

    elif v_type == "function_exists":
        pattern = verification["pattern"]
        found = False
        evidence = ""

        for src_file in project_dir.rglob("*.py"):
            if "__pycache__" in str(src_file):
                continue
            try:
                content = src_file.read_text()
                if re.search(pattern, content, re.I):
                    found = True
                    evidence = str(src_file)
                    break
            except Exception:
                pass

        if not found:
            for src_file in project_dir.rglob("*.rs"):
                if "target" in str(src_file):
                    continue
                try:
                    content = src_file.read_text()
                    if re.search(pattern, content, re.I):
                        found = True
                        evidence = str(src_file)
                        break
                except Exception:
                    pass

        if found:
            return True, f"Pattern found in: {Path(evidence).name}"
        else:
            return False, f"Pattern not found: {pattern}"

    elif v_type == "cli_command":
        command = verification["command"]
        expected = verification.get("expected_output", "")

        try:
            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=30,
                cwd=project_dir
            )

            if expected and expected in result.stdout:
                return True, f"Command output contains: {expected}"
            elif result.returncode == 0 and not expected:
                return True, "Command executed successfully"
            else:
                return False, f"Output mismatch. Got: {result.stdout[:100]}"
        except subprocess.TimeoutExpired:
            return False, "Command timed out"
        except Exception as e:
            return False, f"Command failed: {str(e)}"

    elif v_type == "api_endpoint":
        # For web services - check endpoint exists
        pattern = verification.get("pattern", "")
        return False, "API endpoint verification not yet implemented"

    return False, f"Unknown verification type: {v_type}"


def check_specification_v2(spec_file: Path, project_dir: Path) -> dict:
    """
    Main specification checker for YAML format.
    Returns comprehensive report.
    """
    spec = load_spec(spec_file)

    print("=" * 70)
    print(f"SPECIFICATION VERIFICATION: {spec['name']} v{spec['version']}")
    print("=" * 70)
    print("")

    results = {
        "spec_name": spec["name"],
        "spec_version": spec["version"],
        "timestamp": datetime.now().isoformat(),
        "features": {},
        "summary": {}
    }

    total_required = 0
    passed_required = 0

    for feature in spec["features"]:
        feature_id = feature["id"]
        feature_name = feature["name"]
        is_required = feature["required"]

        passed, evidence = verify_feature(feature, project_dir)

        status_icon = "PASS" if passed else "FAIL"
        required_tag = "[REQUIRED]" if is_required else "[OPTIONAL]"

        print(f"{status_icon} {feature_id}: {feature_name} {required_tag}")
        print(f"   {evidence}")
        print("")

        results["features"][feature_id] = {
            "name": feature_name,
            "required": is_required,
            "passed": passed,
            "evidence": evidence
        }

        if is_required:
            total_required += 1
            if passed:
                passed_required += 1

    coverage = passed_required / total_required if total_required > 0 else 0

    print("=" * 70)
    print("COVERAGE SUMMARY")
    print("=" * 70)
    print(f"Required Features: {passed_required}/{total_required} ({coverage*100:.1f}%)")
    print("")

    # Check completion criteria
    criteria = spec["completion_criteria"]
    criteria_met = True

    if criteria.get("all_required_features") and coverage < 1.0:
        print(f"All required features: FAILED ({coverage*100:.1f}%)")
        criteria_met = False
    else:
        print(f"All required features: PASSED")

    results["summary"] = {
        "total_required": total_required,
        "passed_required": passed_required,
        "coverage_percentage": coverage * 100,
        "criteria_met": criteria_met
    }

    print("")
    if criteria_met:
        print("SPECIFICATION COMPLETE - All criteria met")
    else:
        print("SPECIFICATION INCOMPLETE - Criteria not met")
    print("=" * 70)

    return results


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 3:
        print("Usage: python spec_v2_checker.py <spec.yaml> <project_dir>")
        sys.exit(1)

    spec_file = Path(sys.argv[1])
    project_dir = Path(sys.argv[2])

    if not spec_file.exists():
        print(f"ERROR: Spec file not found: {spec_file}")
        sys.exit(1)

    results = check_specification_v2(spec_file, project_dir)

    # Save results
    output_file = _paths.spec_v2_report
    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_file.write_text(json.dumps(results, indent=2))

    if results["summary"]["criteria_met"]:
        sys.exit(0)
    else:
        sys.exit(1)
