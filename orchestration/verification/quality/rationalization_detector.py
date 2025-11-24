#!/usr/bin/env python3
"""
Detect language patterns that indicate completion bias and premature stopping.
Based on historical failure analysis patterns.
"""
import re
import json
import sys
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


@dataclass
class RationalizationViolation:
    """A detected rationalization pattern."""
    phrase: str
    context: str
    severity: str  # "warning", "critical", "blocking"
    recommendation: str


# Phrases that appeared in actual failure cases
CRITICAL_PHRASES = [
    ("substantially complete", "Project is NOT complete. 'Substantially' means incomplete.", "blocking"),
    ("core features working", "Core is not the whole. What about non-core features?", "blocking"),
    ("known limitations", "Limitations are incomplete features. Fix them.", "blocking"),
    ("future work", "There is no future work. Complete the spec NOW.", "blocking"),
    ("phase 1 complete", "Phase 1 is not completion. Continue to Phase N.", "blocking"),
    ("foundation established", "Foundation is not implementation. Build on it.", "blocking"),
    ("implementation pending", "Pending means incomplete. Implement it.", "blocking"),
    ("if user approves", "No user approval needed. Continue autonomously.", "blocking"),
    ("awaiting approval", "No approval gates. Continue to next phase.", "blocking"),
    ("milestone reached", "Milestones are not endpoints. Continue.", "blocking"),
    ("ready for review", "Not ready until 100% complete.", "blocking"),
    ("natural completion point", "No natural completion. Only spec completion.", "blocking"),
    ("skeleton", "Skeletons are not implementations.", "critical"),
    ("scaffold", "Scaffolds need building on. Continue.", "critical"),
    ("beyond autonomous scope", "Nothing is beyond scope. Implement it.", "blocking"),
    ("production-ready for its defined scope", "Scope is SPEC, not your definition.", "blocking"),
    ("v0.x pre-release", "Version doesn't excuse incompleteness.", "critical"),
    ("stub", "Stubs are not implementations. Replace them.", "blocking"),
    ("placeholder", "Placeholders indicate incompleteness.", "blocking"),
    ("architectural decisions", "Make the decisions. Don't defer.", "critical"),
    ("multi-week project", "Time estimates are forbidden. Continue working.", "blocking"),
    ("multi-month", "Time estimates are forbidden. Continue working.", "blocking"),
    ("weeks remaining", "Time estimates are forbidden. Continue working.", "blocking"),
    ("recommend review", "No reviews until 100% complete.", "blocking"),
    ("solid foundation", "Foundation = 0% complete. Build the house.", "blocking"),
    ("good enough", "Good enough != specification complete.", "blocking"),
    ("acceptable for", "Only 100% is acceptable.", "critical"),
    ("core architecture", "Architecture is not features. Implement features.", "critical"),
    ("ready for continued development", "Continue development NOW, don't stop.", "blocking"),
]

WARNING_PHRASES = [
    ("edge case", "Edge cases are still requirements."),
    ("advanced feature", "Advanced features are still required features."),
    ("nice to have", "If in spec, it's required, not nice to have."),
    ("optional", "Nothing in spec is optional."),
    ("could be", "Could be = should be. Implement it."),
    ("might want", "You must, not might."),
]


def scan_text_for_rationalizations(text: str) -> list[RationalizationViolation]:
    """Scan text for rationalization patterns."""
    violations = []

    # Check critical phrases
    for phrase, recommendation, severity in CRITICAL_PHRASES:
        if phrase.lower() in text.lower():
            # Get context around the phrase
            pattern = re.compile(
                rf'.{{0,50}}{re.escape(phrase)}.{{0,50}}',
                re.IGNORECASE | re.DOTALL
            )
            match = pattern.search(text)
            context = match.group(0) if match else phrase

            violations.append(RationalizationViolation(
                phrase=phrase,
                context=context.strip(),
                severity=severity,
                recommendation=recommendation
            ))

    # Check warning phrases
    for phrase, recommendation in WARNING_PHRASES:
        if phrase.lower() in text.lower():
            pattern = re.compile(
                rf'.{{0,50}}{re.escape(phrase)}.{{0,50}}',
                re.IGNORECASE | re.DOTALL
            )
            match = pattern.search(text)
            context = match.group(0) if match else phrase

            violations.append(RationalizationViolation(
                phrase=phrase,
                context=context.strip(),
                severity="warning",
                recommendation=recommendation
            ))

    return violations


def scan_file_for_rationalizations(file_path: Path) -> list[RationalizationViolation]:
    """Scan a file for rationalization patterns."""
    try:
        content = file_path.read_text()
        return scan_text_for_rationalizations(content)
    except Exception:
        return []


def scan_directory_for_rationalizations(directory: Path) -> dict:
    """Scan entire directory for rationalization patterns."""
    all_violations = {}

    # Files likely to contain completion language
    suspect_patterns = [
        "*COMPLETION*.md",
        "*REPORT*.md",
        "*README*.md",
        "*STATUS*.md",
        "*SUMMARY*.md",
    ]

    suspect_files = []
    for pattern in suspect_patterns:
        suspect_files.extend(directory.rglob(pattern))

    # Remove duplicates while preserving order
    seen = set()
    unique_files = []
    for f in suspect_files:
        if f not in seen:
            seen.add(f)
            unique_files.append(f)

    for file_path in unique_files:
        violations = scan_file_for_rationalizations(file_path)
        if violations:
            all_violations[str(file_path)] = violations

    return all_violations


def run_rationalization_check(target_dir: Path = None) -> bool:
    """
    Main check function. Returns True if clean, False if violations found.
    """
    if target_dir is None:
        target_dir = Path.cwd()

    print("=" * 60)
    print("RATIONALIZATION PATTERN DETECTION")
    print("=" * 60)
    print("")

    violations = scan_directory_for_rationalizations(target_dir)

    blocking_count = 0
    critical_count = 0
    warning_count = 0

    for file_path, file_violations in violations.items():
        print(f"File: {file_path}")
        for v in file_violations:
            icon = "BLOCK" if v.severity == "blocking" else "CRIT" if v.severity == "critical" else "WARN"
            print(f"  [{icon}] '{v.phrase}'")
            print(f"     Context: ...{v.context[:80]}...")
            print(f"     Action: {v.recommendation}")
            print("")

            if v.severity == "blocking":
                blocking_count += 1
            elif v.severity == "critical":
                critical_count += 1
            else:
                warning_count += 1

    print("=" * 60)
    print(f"SUMMARY: {blocking_count} blocking, {critical_count} critical, {warning_count} warnings")
    print("")

    # Log violations
    log_file = _paths.rationalization_log
    log_entry = {
        "timestamp": datetime.now().isoformat(),
        "blocking_count": blocking_count,
        "critical_count": critical_count,
        "warning_count": warning_count,
        "violations": {
            path: [
                {
                    "phrase": v.phrase,
                    "severity": v.severity,
                    "context": v.context,
                    "recommendation": v.recommendation
                }
                for v in viols
            ]
            for path, viols in violations.items()
        }
    }

    # Append to log
    log_file.parent.mkdir(parents=True, exist_ok=True)
    if log_file.exists():
        try:
            log = json.loads(log_file.read_text())
        except json.JSONDecodeError:
            log = []
    else:
        log = []
    log.append(log_entry)
    log_file.write_text(json.dumps(log, indent=2))

    if blocking_count > 0:
        print("BLOCKING VIOLATIONS FOUND")
        print("Cannot proceed with completion until these are resolved.")
        print("These phrases indicate premature completion rationalization.")
        return False
    elif critical_count > 0:
        print("CRITICAL VIOLATIONS FOUND")
        print("Strong indicators of incomplete work. Review carefully.")
        return False
    else:
        print("No blocking rationalization patterns detected")
        return True


if __name__ == "__main__":
    import sys
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    success = run_rationalization_check(target)
    sys.exit(0 if success else 1)
