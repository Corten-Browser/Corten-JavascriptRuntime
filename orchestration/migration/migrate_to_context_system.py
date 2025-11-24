#!/usr/bin/env python3
"""
Migration Script: Convert to New Context System

This script migrates existing projects to the new context system:
1. Verifies orchestration/context/ files exist
2. Generates component.yaml for existing components
3. Reports on migration status

The migration is non-destructive:
- Old CLAUDE.md files continue working (backwards compatible)
- Component.yaml is generated alongside existing files
- No files are deleted or modified

Usage:
    python orchestration/migration/migrate_to_context_system.py [project_root]

Example:
    cd /path/to/project
    python orchestration/migration/migrate_to_context_system.py
"""

import sys
from pathlib import Path

# Add parent directory for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.context.component_yaml_generator import ComponentYamlGenerator


def check_context_files(project_root: Path) -> tuple[bool, list[str]]:
    """Check if context files exist."""
    context_dir = project_root / "orchestration" / "context"
    issues = []

    if not context_dir.exists():
        issues.append("orchestration/context/ directory not found")
        return False, issues

    required_files = [
        "component-rules.md",
        "README.md"
    ]

    for filename in required_files:
        if not (context_dir / filename).exists():
            issues.append(f"Missing: orchestration/context/{filename}")

    return len(issues) == 0, issues


def migrate_components(project_root: Path) -> dict:
    """Generate component.yaml for existing components."""
    components_dir = project_root / "components"
    results = {
        "generated": [],
        "skipped": [],
        "errors": []
    }

    if not components_dir.exists():
        return results

    for component_path in components_dir.iterdir():
        if not component_path.is_dir():
            continue

        # Skip hidden directories
        if component_path.name.startswith("."):
            continue

        try:
            if ComponentYamlGenerator.generate_if_missing(component_path):
                results["generated"].append(component_path.name)
            else:
                results["skipped"].append(component_path.name)
        except Exception as e:
            results["errors"].append(f"{component_path.name}: {str(e)}")

    return results


def report_claude_md_status(project_root: Path) -> dict:
    """Report on CLAUDE.md files in components."""
    components_dir = project_root / "components"
    status = {
        "old_style": [],
        "new_style": [],
        "missing": []
    }

    if not components_dir.exists():
        return status

    for component_path in components_dir.iterdir():
        if not component_path.is_dir() or component_path.name.startswith("."):
            continue

        claude_path = component_path / "CLAUDE.md"
        if not claude_path.exists():
            status["missing"].append(component_path.name)
        else:
            content = claude_path.read_text()
            if "CONTEXT INCLUDES" in content[:200]:
                status["new_style"].append(component_path.name)
            else:
                status["old_style"].append(component_path.name)

    return status


def migrate_project(project_root: Path) -> bool:
    """Run full migration."""
    print("=" * 60)
    print("Context System Migration")
    print("=" * 60)
    print(f"\nProject: {project_root}")

    # Step 1: Check context files
    print("\n[1/3] Checking context files...")
    context_ok, issues = check_context_files(project_root)

    if not context_ok:
        print("❌ Context files not found or incomplete:")
        for issue in issues:
            print(f"   - {issue}")
        print("\n💡 Run upgrade.sh to install context files first.")
        return False

    print("✅ Context files present")

    # Step 2: Migrate components
    print("\n[2/3] Generating component.yaml files...")
    results = migrate_components(project_root)

    if results["generated"]:
        print(f"✅ Generated {len(results['generated'])} component.yaml files:")
        for name in results["generated"]:
            print(f"   - {name}")

    if results["skipped"]:
        print(f"ℹ️  Skipped {len(results['skipped'])} (already have component.yaml):")
        for name in results["skipped"]:
            print(f"   - {name}")

    if results["errors"]:
        print(f"⚠️  Errors in {len(results['errors'])} components:")
        for error in results["errors"]:
            print(f"   - {error}")

    # Step 3: Report CLAUDE.md status
    print("\n[3/3] CLAUDE.md status report...")
    status = report_claude_md_status(project_root)

    if status["new_style"]:
        print(f"✅ {len(status['new_style'])} components using new-style CLAUDE.md")

    if status["old_style"]:
        print(f"ℹ️  {len(status['old_style'])} components using old-style CLAUDE.md:")
        for name in status["old_style"]:
            print(f"   - {name} (will continue working)")

    if status["missing"]:
        print(f"⚠️  {len(status['missing'])} components missing CLAUDE.md:")
        for name in status["missing"]:
            print(f"   - {name}")

    # Summary
    print("\n" + "=" * 60)
    print("Migration Summary")
    print("=" * 60)
    print(f"✅ Context files:     Present")
    print(f"✅ Components with yaml: {len(results['generated']) + len(results['skipped'])}")
    print(f"ℹ️  Old-style CLAUDE.md: {len(status['old_style'])} (backwards compatible)")
    print(f"✅ New-style CLAUDE.md: {len(status['new_style'])}")

    if results["errors"]:
        print(f"⚠️  Errors: {len(results['errors'])}")
        return False

    print("\n✅ Migration complete!")
    print("\nNotes:")
    print("- Old CLAUDE.md files continue working (no action needed)")
    print("- New components will use the new context system")
    print("- To manually update old CLAUDE.md, replace content with minimal template")

    return True


def main():
    """CLI entry point."""
    if len(sys.argv) > 1:
        project_root = Path(sys.argv[1]).resolve()
    else:
        project_root = Path.cwd()

    if not (project_root / "orchestration").exists():
        print("❌ Not an orchestrated project (orchestration/ not found)")
        print(f"   Searched in: {project_root}")
        sys.exit(1)

    success = migrate_project(project_root)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
