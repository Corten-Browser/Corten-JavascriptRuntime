#!/usr/bin/env python3
"""
Completion Certification Validator

Validates that completion certification checklist is properly filled out.

Exit Codes:
  0 - Valid certification (all YES answers)
  1 - Invalid certification (has NO answers or incomplete)

Part of v0.14.0: Programmatic Enforcement System
"""

from pathlib import Path
import re
import sys


class CertificationValidator:
    """Validates completion certification."""

    def __init__(self, cert_path: Path):
        self.cert_path = cert_path
        self.content = ""
        self.errors = []
        self.warnings = []
        self.no_count = 0
        self.yes_count = 0

    def validate(self) -> bool:
        """Validate certification."""
        if not self.cert_path.exists():
            self.errors.append(f"Certification not found: {self.cert_path}")
            return False

        self.content = self.cert_path.read_text()

        self._check_all_questions_answered()
        self._count_answers()
        self._check_final_decision()

        return len(self.errors) == 0 and self.no_count == 0

    def _check_all_questions_answered(self):
        """Verify all questions have YES/NO answers (not left blank)."""
        # Find patterns like "YES / NO" that haven't been modified
        unanswered_pattern = r'\|\s*YES\s*/\s*NO\s*\|'
        unanswered_table = re.findall(unanswered_pattern, self.content)

        # Also check for question format with YES / NO not in table
        unanswered_questions = re.findall(
            r'\*\*[^*]+\*\*\s+YES\s*/\s*NO(?!\s*\*)',
            self.content
        )

        total_unanswered = len(unanswered_table) + len(unanswered_questions)

        if total_unanswered > 0:
            self.errors.append(
                f"Found {total_unanswered} unanswered question(s)\n"
                f"  All questions must be answered with YES or NO"
            )

    def _count_answers(self):
        """Count YES and NO answers."""
        # Count table-style answers (| YES |, | NO |)
        yes_table = re.findall(r'\|\s*YES\s*\|', self.content)
        no_table = re.findall(r'\|\s*NO\s*\|', self.content)

        # Count inline answers (**question** YES, **question** NO)
        yes_inline = re.findall(r'\*\*[^*]+\*\*\s+YES(?:\s|$)', self.content)
        no_inline = re.findall(r'\*\*[^*]+\*\*\s+NO(?:\s|$)', self.content)

        self.yes_count = len(yes_table) + len(yes_inline)
        self.no_count = len(no_table) + len(no_inline)

        if self.no_count > 0:
            self.errors.append(
                f"Found {self.no_count} NO answer(s)\n"
                f"  CANNOT DECLARE COMPLETE with any NO answers\n"
                f"  You must fix all issues and change all answers to YES"
            )

    def _check_final_decision(self):
        """Verify final decision matches answers."""
        # Look for FINAL DECISION
        decision_match = re.search(r'\*\*FINAL DECISION\*\*:\s*(\w+)', self.content)

        if not decision_match:
            self.warnings.append("Final decision not filled out")
            return

        decision = decision_match.group(1).upper()

        if decision == 'COMPLETE':
            if self.no_count > 0:
                self.errors.append(
                    "FINAL DECISION is COMPLETE but certification contains NO answers\n"
                    f"  This is inconsistent - cannot declare complete with {self.no_count} NO answer(s)"
                )
        elif decision == 'INCOMPLETE':
            if self.no_count == 0:
                self.warnings.append(
                    "FINAL DECISION is INCOMPLETE but all answers are YES\n"
                    "  This seems inconsistent"
                )
        else:
            self.errors.append(
                f"FINAL DECISION is '{decision}' - must be COMPLETE or INCOMPLETE"
            )

    def report(self) -> str:
        """Generate validation report."""
        report = []

        report.append("="*70)
        report.append("CERTIFICATION VALIDATION REPORT")
        report.append("="*70)
        report.append(f"File: {self.cert_path}")
        report.append(f"YES answers: {self.yes_count}")
        report.append(f"NO answers: {self.no_count}")
        report.append("")

        if not self.errors and not self.warnings:
            report.append("✅ CERTIFICATION VALID")
            report.append("   All questions answered YES")
            report.append("   May proceed with completion")
        elif self.no_count > 0:
            report.append("❌ CERTIFICATION INVALID - HAS NO ANSWERS")
            report.append("")
            for error in self.errors:
                lines = error.split('\n')
                for line in lines:
                    report.append(f"   {line}")
            report.append("")
            report.append("   🛑 CANNOT DECLARE COMPLETE")
            report.append("   🛑 FIX ALL ISSUES THAT LED TO NO ANSWERS")
            report.append("   🛑 RE-RUN CHECKS AND UPDATE CERTIFICATION")
        elif self.errors:
            report.append("❌ CERTIFICATION INVALID")
            report.append("")
            for i, error in enumerate(self.errors, 1):
                lines = error.split('\n')
                report.append(f"   {i}. {lines[0]}")
                for line in lines[1:]:
                    if line.strip():
                        report.append(f"      {line}")
        else:
            report.append("✅ CERTIFICATION VALID (with warnings)")
            report.append("")
            for warning in self.warnings:
                report.append(f"   ⚠️  {warning}")

        report.append("="*70)

        return '\n'.join(report)


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Validate completion certification checklist"
    )
    parser.add_argument(
        "cert_path",
        type=Path,
        nargs='?',
        default=Path("orchestration/completion_certification.md"),
        help="Path to certification file (default: orchestration/completion_certification.md)"
    )

    args = parser.parse_args()

    validator = CertificationValidator(args.cert_path)

    passed = validator.validate()
    print(validator.report())

    if passed:
        print("\n✅ Certification is valid - all criteria met")
    else:
        print("\n❌ Certification is invalid - cannot declare completion")
        print("Fix all issues and re-validate")

    sys.exit(0 if passed else 1)


if __name__ == '__main__':
    main()
