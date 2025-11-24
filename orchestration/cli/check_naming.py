#!/usr/bin/env python3
"""
Component Naming Check CLI

Standalone tool for checking component naming violations.

Usage:
    python orchestration/cli/check_naming.py
    python orchestration/cli/check_naming.py --detailed
    python orchestration/cli/check_naming.py --fix
    python orchestration/cli/check_naming.py --project-dir /path/to/project
"""

import sys
import argparse
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.validation.naming_scanner import ComponentNamingScanner


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Check component naming violations",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                                # Quick check
  %(prog)s --detailed                     # Detailed analysis with impact
  %(prog)s --fix                          # Suggest fixes
  %(prog)s --project-dir /path/to/project # Check specific project
        """
    )
    parser.add_argument(
        "--project-dir",
        default=".",
        help="Project directory (default: current directory)"
    )
    parser.add_argument(
        "--detailed",
        action="store_true",
        help="Show detailed analysis with impact estimates"
    )
    parser.add_argument(
        "--fix",
        action="store_true",
        help="Show how to fix violations (implies --detailed)"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output in JSON format"
    )

    args = parser.parse_args()

    # Create scanner
    scanner = ComponentNamingScanner()

    # Scan for violations
    violations = scanner.scan(args.project_dir)

    # Output results
    if args.json:
        import json
        output = {
            "violations_count": len(violations),
            "violations": violations
        }
        print(json.dumps(output, indent=2))
    else:
        # Human-readable output
        print(scanner.get_summary(violations))

        if violations:
            if args.detailed or args.fix:
                scanner.show_detailed_analysis(violations)
            else:
                print()
                for old_name, info in violations.items():
                    print(f"  {old_name} → {info['suggestion']}")
                print()
                print("Run with --detailed for more information")
                print("Run with --fix to see how to fix automatically")

    # Exit with appropriate code
    sys.exit(1 if violations else 0)


if __name__ == "__main__":
    main()
