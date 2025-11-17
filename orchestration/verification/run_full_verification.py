#!/usr/bin/env python3
"""
Convenience script to run full verification process.
This is the main entry point for verification.
"""
import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from verification.verification_agent import run_full_verification


def main():
    if len(sys.argv) > 1:
        project_dir = Path(sys.argv[1])
    else:
        project_dir = Path.cwd()

    print("Starting full verification process...")
    print("")

    verified = run_full_verification(project_dir)

    if verified:
        print("")
        print("=" * 70)
        print("VERIFICATION COMPLETE - PROJECT APPROVED")
        print("=" * 70)
        print("")
        print("You may now commit COMPLETION-REPORT.md")
        print("The pre-commit hook will allow it to proceed.")
        sys.exit(0)
    else:
        print("")
        print("=" * 70)
        print("VERIFICATION FAILED - PROJECT NOT APPROVED")
        print("=" * 70)
        print("")
        print("Fix all identified issues and re-run this script.")
        print("Do NOT attempt to commit COMPLETION-REPORT.md until this passes.")
        sys.exit(1)


if __name__ == "__main__":
    main()
