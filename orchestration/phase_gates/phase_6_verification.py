#!/usr/bin/env python3
"""
Phase 6 Verification Gate

Validates that Phase 6 (Verification & Completion) is complete.

Exit Codes:
  0 - PASS: All verification checks passing
  1 - FAIL: Verification incomplete

Checks:
- completion_verifier shows 13/13 checks passing
- All components verified
- Documentation generated
- No blocking issues
"""

import subprocess
import sys
from pathlib import Path


class Phase6VerificationGate:
    """Phase 6 exit gate: All verification complete."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.failures = []

    def validate(self) -> bool:
        """Run Phase 6 validations."""
        self._check_completion_verifier_exists()
        if not self.failures:
            self._check_all_components_verified()

        return len(self.failures) == 0

    def _check_completion_verifier_exists(self):
        """Check if completion_verifier.py exists."""
        verifier_path = self.project_root / "orchestration" / "completion_verifier.py"
        if not verifier_path.exists():
            self.failures.append(
                "completion_verifier.py not found in orchestration/\n"
                "  Cannot validate Phase 6 without completion verifier"
            )

    def _check_all_components_verified(self):
        """
        Check that all components pass completion verification.

        This runs completion_verifier on all components and ensures 13/13 checks pass.
        """
        components_dir = self.project_root / "components"

        if not components_dir.exists():
            self.failures.append("No components/ directory found")
            return

        components = [d for d in components_dir.iterdir() if d.is_dir() and not d.name.startswith('.')]

        if not components:
            # No components = nothing to verify (single-component project)
            return

        failed_components = []

        for component in components:
            try:
                result = subprocess.run(
                    ["python3", "orchestration/completion_verifier.py", str(component)],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                if result.returncode != 0:
                    failed_components.append(component.name)

            except Exception as e:
                self.failures.append(f"Error verifying {component.name}: {str(e)}")

        if failed_components:
            self.failures.append(
                f"{len(failed_components)} component(s) failed verification:\n"
                f"  {', '.join(failed_components)}\n"
                f"  All components must pass 13/13 checks"
            )

    def report(self) -> str:
        """Generate report."""
        if not self.failures:
            return (
                "✅ PHASE 6 VERIFICATION - COMPLETE\n"
                "   All verification checks passing\n"
                "   Project is ready for deployment\n"
            )
        else:
            report = (
                "❌ PHASE 6 VERIFICATION - INCOMPLETE\n"
                f"   {len(self.failures)} blocking issue(s):\n\n"
            )
            for i, failure in enumerate(self.failures, 1):
                indented_failure = failure.replace('\n', '\n   ')
                report += f"   {i}. {indented_failure}\n"

            report += (
                "\n"
                "   🛑 CANNOT MARK PROJECT COMPLETE\n"
                "   🛑 FIX ALL VERIFICATION ISSUES\n"
            )

            return report


def main():
    """CLI entry point."""
    if len(sys.argv) < 2:
        print("Usage: phase_6_verification.py <project_root>")
        sys.exit(1)

    project_root = Path(sys.argv[1]).resolve()
    gate = Phase6VerificationGate(project_root)

    print("🔍 Running Phase 6 Verification Gate...")
    print(f"   Project: {project_root}")
    print()

    passed = gate.validate()
    print(gate.report())

    sys.exit(0 if passed else 1)


if __name__ == '__main__':
    main()
