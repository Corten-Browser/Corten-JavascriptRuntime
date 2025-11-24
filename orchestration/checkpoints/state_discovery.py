"""
State discovery for orchestration resume without checkpoint.

Scans project files, runs tests, checks git history to infer current state.
"""

from pathlib import Path
from typing import Dict, Any, List, Optional
import json
import subprocess
from datetime import datetime

from orchestration.core.paths import DataPaths


class StateDiscovery:
    """Discover orchestration state from project artifacts."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self._paths = DataPaths(self.project_root)

    def discover_state(self) -> Dict[str, Any]:
        """
        Discover current orchestration state by scanning project.

        Returns discovery result with confidence level.
        """
        discovery = {
            "discovery_timestamp": datetime.utcnow().isoformat() + "Z",
            "discovery_method": "filesystem_scan",
            "confidence": "unknown",
            "discovered_state": {},
            "inferred_context": {},
            "missing_information": []
        }

        # Scan components
        components = self._discover_components()
        discovery["discovered_state"]["components_found"] = components

        # Check orchestration-state.json
        orch_state = self._load_orchestration_state()
        if orch_state:
            discovery["discovered_state"]["orchestration_state"] = orch_state
        else:
            discovery["missing_information"].append("orchestration-state.json not found")

        # Run tests to check status
        test_results = self._check_test_status()
        discovery["discovered_state"]["tests_run"] = test_results

        # Check phase gates
        gate_status = self._check_phase_gates()
        discovery["discovered_state"]["phase_gates"] = gate_status

        # Analyze git history
        git_info = self._analyze_git_history()
        discovery["discovered_state"].update(git_info)

        # Infer context
        discovery["inferred_context"] = self._infer_context(
            components, orch_state, test_results, gate_status
        )

        # Determine confidence
        discovery["confidence"] = self._assess_confidence(discovery)

        return discovery

    def _discover_components(self) -> List[str]:
        """Find all component directories."""
        components_dir = self.project_root / "components"
        if not components_dir.exists():
            return []

        components = []
        for item in components_dir.iterdir():
            if item.is_dir() and not item.name.startswith('.'):
                # Verify it's actually a component (has CLAUDE.md or src/)
                if (item / "CLAUDE.md").exists() or (item / "src").exists():
                    components.append(item.name)

        return sorted(components)

    def _load_orchestration_state(self) -> Optional[Dict[str, Any]]:
        """Load orchestration-state.json if exists."""
        state_file = self._paths.orchestration_state
        if not state_file.exists():
            return None

        try:
            with open(state_file, 'r') as f:
                return json.load(f)
        except:
            return None

    def _check_test_status(self) -> Dict[str, Any]:
        """Run tests and capture results."""
        results = {
            "unit_tests": {"total": 0, "passed": 0, "failed": 0},
            "integration_tests": {"total": 0, "passed": 0, "failed": 0},
            "last_run": datetime.utcnow().isoformat() + "Z"
        }

        # Try to run pytest and parse output
        try:
            result = subprocess.run(
                ["pytest", "-v", "--tb=no", "-q"],
                capture_output=True,
                text=True,
                cwd=self.project_root,
                timeout=120
            )

            # Parse pytest output
            output = result.stdout + result.stderr
            results["raw_output"] = output

            # Simple parsing (this would be more robust in production)
            if "passed" in output.lower():
                # Extract numbers from output
                import re
                match = re.search(r'(\d+) passed', output)
                if match:
                    results["unit_tests"]["passed"] = int(match.group(1))
                    results["unit_tests"]["total"] = int(match.group(1))

                match = re.search(r'(\d+) failed', output)
                if match:
                    results["unit_tests"]["failed"] = int(match.group(1))
                    results["unit_tests"]["total"] += int(match.group(1))

        except Exception as e:
            results["error"] = str(e)

        return results

    def _check_phase_gates(self) -> Dict[str, str]:
        """Check phase gate status."""
        gates = {}

        # Check if gate_runner exists
        gate_runner = self.project_root / "orchestration" / "phase_gates" / "gate_runner.py"
        if not gate_runner.exists():
            return {"error": "Phase gates not found"}

        # Try to get gate status
        try:
            result = subprocess.run(
                ["python3", str(gate_runner), str(self.project_root), "--status"],
                capture_output=True,
                text=True,
                timeout=30
            )

            # Parse output (would need actual format from gate_runner)
            output = result.stdout
            gates["raw_output"] = output

            # Try to determine which phase we're on
            for i in range(1, 7):
                if f"phase_{i}" in output.lower() or f"phase {i}" in output.lower():
                    if "passed" in output.lower():
                        gates[f"phase_{i}"] = "passed"
                    elif "failed" in output.lower():
                        gates[f"phase_{i}"] = "failed"

        except Exception as e:
            gates["error"] = str(e)

        return gates

    def _analyze_git_history(self) -> Dict[str, Any]:
        """Analyze recent git commits."""
        git_info = {}

        try:
            # Get recent commits
            result = subprocess.run(
                ["git", "log", "-10", "--oneline"],
                capture_output=True,
                text=True,
                cwd=self.project_root
            )

            commits = result.stdout.strip().split('\n')
            git_info["recent_commits"] = commits
            git_info["git_commits_since_start"] = len(commits)

            if commits:
                git_info["last_commit_message"] = commits[0]

            # Check branch
            result = subprocess.run(
                ["git", "branch", "--show-current"],
                capture_output=True,
                text=True,
                cwd=self.project_root
            )
            git_info["current_branch"] = result.stdout.strip()

        except Exception as e:
            git_info["error"] = str(e)

        return git_info

    def _infer_context(
        self,
        components: List[str],
        orch_state: Optional[Dict],
        test_results: Dict,
        gate_status: Dict
    ) -> Dict[str, Any]:
        """Infer orchestration context from discovered state."""
        context = {
            "likely_current_phase": 0,
            "likely_original_specs": [],
            "estimated_progress_percent": 0
        }

        # Infer phase from gate status
        for i in range(6, 0, -1):
            if gate_status.get(f"phase_{i}") == "passed":
                context["likely_current_phase"] = i + 1
                break
            elif gate_status.get(f"phase_{i}") == "failed":
                context["likely_current_phase"] = i
                break

        # If no gate info, infer from components
        if context["likely_current_phase"] == 0:
            if len(components) > 0:
                context["likely_current_phase"] = 2  # Has components
            if test_results.get("unit_tests", {}).get("total", 0) > 0:
                context["likely_current_phase"] = 3  # Has tests

        # Estimate progress
        phase = context["likely_current_phase"]
        context["estimated_progress_percent"] = int((phase / 6) * 100)

        # Try to find spec files
        spec_dir = self.project_root / "specifications"
        if spec_dir.exists():
            specs = list(spec_dir.glob("*.md")) + list(spec_dir.glob("*.yaml"))
            context["likely_original_specs"] = [str(s.relative_to(self.project_root)) for s in specs]

        return context

    def _assess_confidence(self, discovery: Dict[str, Any]) -> str:
        """Assess confidence level in discovered state."""
        missing = len(discovery["missing_information"])

        if missing == 0:
            return "high"
        elif missing <= 2:
            return "medium"
        else:
            return "low"

    def _analyze_test_results_detailed(self) -> Dict[str, Any]:
        """
        Detailed test analysis including integration tests.

        Parses pytest output to identify:
        - Unit vs integration test results
        - Specific failing tests
        - Coverage data
        """
        results = {
            "unit_tests": {},
            "integration_tests": {},
            "coverage_percent": None,
            "failing_tests": []
        }

        # Run unit tests
        try:
            result = subprocess.run(
                ["pytest", "tests/unit", "-v", "--tb=short"],
                capture_output=True,
                text=True,
                cwd=self.project_root,
                timeout=120
            )

            results["unit_tests"] = self._parse_pytest_output(result.stdout)
        except:
            results["unit_tests"]["error"] = "Could not run unit tests"

        # Run integration tests
        try:
            result = subprocess.run(
                ["pytest", "tests/integration", "-v", "--tb=short"],
                capture_output=True,
                text=True,
                cwd=self.project_root,
                timeout=180
            )

            results["integration_tests"] = self._parse_pytest_output(result.stdout)
        except:
            results["integration_tests"]["error"] = "Could not run integration tests"

        return results

    def _parse_pytest_output(self, output: str) -> Dict[str, Any]:
        """Parse pytest output for test counts and failures."""
        import re

        parsed = {
            "total": 0,
            "passed": 0,
            "failed": 0,
            "skipped": 0,
            "pass_rate_percent": 0,
            "failures": []
        }

        # Extract summary line
        # Example: "5 passed, 1 failed in 2.34s"
        summary_match = re.search(
            r'(\d+) passed(?:, (\d+) failed)?(?:, (\d+) skipped)?',
            output
        )

        if summary_match:
            parsed["passed"] = int(summary_match.group(1))
            parsed["failed"] = int(summary_match.group(2) or 0)
            parsed["skipped"] = int(summary_match.group(3) or 0)
            parsed["total"] = parsed["passed"] + parsed["failed"] + parsed["skipped"]

            if parsed["total"] > 0:
                parsed["pass_rate_percent"] = round(
                    (parsed["passed"] / parsed["total"]) * 100, 1
                )

        # Extract failure details
        # Look for FAILED lines
        for line in output.split('\n'):
            if line.startswith("FAILED"):
                # Extract test name and file
                match = re.match(r'FAILED ([\w/\.]+)::([\w_]+)', line)
                if match:
                    parsed["failures"].append({
                        "file": match.group(1),
                        "test": match.group(2)
                    })

        return parsed

    def _check_completion_verifier(self) -> Dict[str, Any]:
        """
        Run completion verifier to check quality metrics.

        Returns status of all 13 checks.
        """
        verifier_script = self.project_root / "orchestration" / "completion_verifier.py"
        if not verifier_script.exists():
            return {"error": "completion_verifier.py not found"}

        try:
            result = subprocess.run(
                ["python3", str(verifier_script), str(self.project_root)],
                capture_output=True,
                text=True,
                timeout=60
            )

            # Parse output
            output = result.stdout
            checks = {}

            # Look for check results (format: "✅ Check 1: Description")
            import re
            for line in output.split('\n'):
                match = re.match(r'([✅❌])\s+Check (\d+):\s+(.+)', line)
                if match:
                    status_symbol, check_num, description = match.groups()
                    checks[f"check_{check_num}"] = {
                        "number": int(check_num),
                        "description": description,
                        "status": "passed" if status_symbol == "✅" else "failed"
                    }

            # Count passed/failed
            total = len(checks)
            passed = sum(1 for c in checks.values() if c["status"] == "passed")

            return {
                "total_checks": total,
                "passed": passed,
                "failed": total - passed,
                "checks": checks,
                "all_passing": passed == total
            }

        except Exception as e:
            return {"error": str(e)}
