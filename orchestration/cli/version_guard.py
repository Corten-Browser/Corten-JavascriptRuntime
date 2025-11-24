#!/usr/bin/env python3
"""
Version Guard System - Prevents Unauthorized Major Version Bumps

This module provides critical protection against autonomous major version changes.
Major version transitions (especially 0.x.x → 1.0.0) are BUSINESS DECISIONS that
require explicit user approval, testing, and documentation review.

FORBIDDEN ACTIONS:
- Changing version from 0.x.x to 1.0.0
- Any major version increment (X.y.z → X+1.0.0)
- Changing lifecycle_state without approval
- Setting api_locked to true
- Declaring system "production ready" autonomously
"""

import json
import os
from typing import Tuple, Dict, Any
from datetime import datetime


class VersionControlError(Exception):
    """Raised when an unauthorized version change is attempted"""
    pass


class MajorVersionAttemptError(VersionControlError):
    """Raised specifically for major version bump attempts"""
    pass


class LifecycleStateError(VersionControlError):
    """Raised when lifecycle state change is attempted without approval"""
    pass


def parse_version(version: str) -> Tuple[int, int, int]:
    """
    Parse a semantic version string into major, minor, patch components.

    Args:
        version: Version string in format "X.Y.Z"

    Returns:
        Tuple of (major, minor, patch) integers

    Raises:
        ValueError: If version format is invalid
    """
    try:
        parts = version.split('.')
        if len(parts) != 3:
            raise ValueError(f"Invalid version format: {version}")
        return (int(parts[0]), int(parts[1]), int(parts[2]))
    except (ValueError, IndexError) as e:
        raise ValueError(f"Invalid version format '{version}': {e}")


def validate_version_change(old_version: str, new_version: str,
                          user_approved: bool = False) -> bool:
    """
    Validates version changes and blocks unauthorized major version bumps.

    Args:
        old_version: Current version string
        new_version: Proposed new version string
        user_approved: Explicit flag indicating user approval (must be passed deliberately)

    Returns:
        True if version change is allowed

    Raises:
        MajorVersionAttemptError: If major version bump attempted without approval
        VersionControlError: For other invalid version changes
    """
    old_major, old_minor, old_patch = parse_version(old_version)
    new_major, new_minor, new_patch = parse_version(new_version)

    # Check for major version bump
    if new_major > old_major:
        if not user_approved:
            error_msg = f"""
⚠️  MAJOR VERSION BUMP BLOCKED ⚠️
====================================
Attempted change: {old_version} → {new_version}

Major version changes are BUSINESS DECISIONS requiring:
1. Complete user testing and validation
2. Full documentation review
3. API stability assessment
4. Business readiness evaluation
5. EXPLICIT user approval

To approve this change, the user must explicitly state:
"Approve major version bump to {new_version}"

Current version will remain: {old_version}
"""
            log_version_attempt(old_version, new_version, "BLOCKED", error_msg)
            raise MajorVersionAttemptError(error_msg)

    # Special check for 0.x.x → 1.0.0 transition
    if old_major == 0 and new_major == 1:
        if not user_approved:
            error_msg = f"""
🚨 CRITICAL: TRANSITION TO 1.0.0 BLOCKED 🚨
=========================================
Attempted change: {old_version} → {new_version}

The transition from pre-release (0.x.x) to stable (1.0.0) is a
MAJOR MILESTONE requiring:

✓ Complete feature implementation
✓ Comprehensive user testing
✓ Production deployment validation
✓ Security audit completion
✓ Performance benchmarking
✓ Documentation completeness
✓ API contract finalization
✓ Business stakeholder approval

This is NOT a technical decision - it has business implications:
- Service Level Agreements (SLAs)
- Support commitments
- Backwards compatibility guarantees
- API stability promises

To approve, user must explicitly command:
"Approve transition to stable version 1.0.0"

System will remain in pre-release: {old_version}
"""
            log_version_attempt(old_version, new_version, "BLOCKED-1.0.0", error_msg)
            raise MajorVersionAttemptError(error_msg)

    # Validate version progression logic
    if new_major == old_major:
        if new_minor < old_minor:
            raise VersionControlError(f"Version downgrade not allowed: {old_version} → {new_version}")
        if new_minor == old_minor and new_patch < old_patch:
            raise VersionControlError(f"Version downgrade not allowed: {old_version} → {new_version}")

    # Log successful validation
    if old_version != new_version:
        log_version_attempt(old_version, new_version, "ALLOWED", "Version change validated")

    return True


def validate_lifecycle_change(old_state: str, new_state: str,
                            user_approved: bool = False) -> bool:
    """
    Validates lifecycle state changes.

    Args:
        old_state: Current lifecycle state
        new_state: Proposed new lifecycle state
        user_approved: Explicit flag indicating user approval

    Returns:
        True if state change is allowed

    Raises:
        LifecycleStateError: If state change attempted without approval
    """
    restricted_transitions = [
        ("pre-release", "released"),
        ("pre-release", "stable"),
        ("pre-release", "production"),
        ("development", "released"),
        ("development", "production"),
    ]

    transition = (old_state.lower(), new_state.lower())

    if transition in restricted_transitions and not user_approved:
        error_msg = f"""
⚠️  LIFECYCLE STATE CHANGE BLOCKED ⚠️
====================================
Attempted change: {old_state} → {new_state}

This lifecycle transition requires explicit user approval.

Implications of '{new_state}' state:
- Public API guarantees
- Support commitments
- Stability promises
- Documentation requirements

To approve this change, user must explicitly state:
"Approve lifecycle change to {new_state}"

Current state will remain: {old_state}
"""
        log_version_attempt(old_state, new_state, "BLOCKED-LIFECYCLE", error_msg)
        raise LifecycleStateError(error_msg)

    return True


def log_version_attempt(old_value: str, new_value: str,
                        status: str, message: str) -> None:
    """
    Log version change attempts for audit purposes.

    Args:
        old_value: Current value
        new_value: Attempted new value
        status: Status of attempt (ALLOWED/BLOCKED/etc)
        message: Detailed message about the attempt
    """
    log_dir = "orchestration/logs"
    os.makedirs(log_dir, exist_ok=True)

    log_file = os.path.join(log_dir, "version_control.log")

    timestamp = datetime.now().isoformat()
    log_entry = {
        "timestamp": timestamp,
        "status": status,
        "old_value": old_value,
        "new_value": new_value,
        "message": message.strip()
    }

    # Append to log file
    with open(log_file, 'a') as f:
        f.write(json.dumps(log_entry) + "\n")


def check_version_readiness(metadata: Dict[str, Any]) -> Dict[str, Any]:
    """
    Check if system is ready for major version bump and generate report.

    Args:
        metadata: Project metadata dictionary

    Returns:
        Dictionary with readiness assessment
    """
    readiness = {
        "ready_for_1.0.0": False,
        "current_version": metadata.get("version", "0.1.0"),
        "lifecycle_state": metadata.get("lifecycle_state", "pre-release"),
        "criteria": {
            "all_tests_passing": False,
            "documentation_complete": False,
            "api_stable": False,
            "user_tested": False,
            "security_reviewed": False,
            "performance_validated": False
        },
        "recommendation": "",
        "blocking_issues": [],
        "timestamp": datetime.now().isoformat()
    }

    # This is a template - actual implementation would check real criteria
    # For now, we'll always recommend staying in pre-release
    readiness["recommendation"] = (
        f"System remains at version {readiness['current_version']}. "
        "User testing and validation required before 1.0.0 release. "
        "When ready, explicitly approve with: 'Approve transition to stable version 1.0.0'"
    )

    readiness["blocking_issues"] = [
        "User acceptance testing not completed",
        "Production deployment validation pending",
        "API stability period not observed",
        "Documentation review pending"
    ]

    return readiness


def create_readiness_report(metadata: Dict[str, Any]) -> str:
    """
    Generate a readiness report for major version consideration.

    Args:
        metadata: Project metadata dictionary

    Returns:
        Markdown formatted readiness report
    """
    readiness = check_version_readiness(metadata)

    report = f"""# Version 1.0.0 Readiness Assessment

**Generated**: {readiness['timestamp']}
**Current Version**: {readiness['current_version']}
**Lifecycle State**: {readiness['lifecycle_state']}

## Readiness Status: {'✅ READY' if readiness['ready_for_1.0.0'] else '❌ NOT READY'}

## Criteria Checklist

"""

    for criterion, status in readiness['criteria'].items():
        status_icon = "✅" if status else "❌"
        criterion_text = criterion.replace('_', ' ').title()
        report += f"- {status_icon} {criterion_text}\n"

    report += f"""
## Blocking Issues

"""
    for issue in readiness['blocking_issues']:
        report += f"- {issue}\n"

    report += f"""
## Recommendation

{readiness['recommendation']}

## Next Steps

1. Complete all blocking issues
2. Conduct thorough user testing
3. Review all documentation
4. Validate in production environment
5. When ready, explicitly approve with:
   ```
   Approve transition to stable version 1.0.0
   ```

---
*This report was generated automatically. Major version transitions require explicit user approval.*
"""

    return report


def safe_update_version(metadata_path: str, new_version: str,
                       user_approved: bool = False) -> Dict[str, Any]:
    """
    Safely update version in project metadata with validation.

    Args:
        metadata_path: Path to project-metadata.json
        new_version: Proposed new version
        user_approved: Explicit user approval flag

    Returns:
        Updated metadata dictionary

    Raises:
        VersionControlError: If update is not allowed
    """
    # Read current metadata
    with open(metadata_path, 'r') as f:
        metadata = json.load(f)

    old_version = metadata.get('version', '0.1.0')

    # Validate the change
    validate_version_change(old_version, new_version, user_approved)

    # Update version
    metadata['version'] = new_version
    metadata['last_updated'] = datetime.now().isoformat()

    # Write back
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)

    return metadata


# Example usage and testing
if __name__ == "__main__":
    print("Version Guard System - Testing")
    print("=" * 40)

    # Test cases
    test_cases = [
        ("0.1.0", "0.1.1", False, "SHOULD PASS - patch version"),
        ("0.1.0", "0.2.0", False, "SHOULD PASS - minor version"),
        ("0.1.0", "1.0.0", False, "SHOULD FAIL - major version"),
        ("0.9.9", "1.0.0", False, "SHOULD FAIL - transition to stable"),
        ("1.0.0", "2.0.0", False, "SHOULD FAIL - major version"),
        ("0.1.0", "0.0.9", False, "SHOULD FAIL - downgrade"),
    ]

    for old, new, approved, description in test_cases:
        try:
            validate_version_change(old, new, approved)
            print(f"✅ {description}: {old} → {new}")
        except VersionControlError as e:
            print(f"❌ {description}: {old} → {new} [BLOCKED]")

    print("\nVersion Guard System Active - Major version bumps require explicit user approval")