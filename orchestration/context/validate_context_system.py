#!/usr/bin/env python3
"""
Context System Validation Script

Validates that the context system is properly set up:
1. All context files exist
2. ComponentYamlGenerator works correctly
3. Migration script can run
"""

import sys
import tempfile
from pathlib import Path

# Add parent directory for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))


def validate_context_files(project_root: Path) -> tuple[bool, list[str]]:
    """Validate that all context files exist."""
    context_dir = project_root / "orchestration" / "context"
    issues = []

    required_files = [
        "__init__.py",
        "README.md",
        "component-rules.md",
        "component_yaml_generator.py",
        "orchestration-rules.md",
    ]

    if not context_dir.exists():
        issues.append(f"Context directory not found: {context_dir}")
        return False, issues

    for filename in required_files:
        file_path = context_dir / filename
        if not file_path.exists():
            issues.append(f"Missing file: {file_path}")
        elif file_path.stat().st_size == 0:
            issues.append(f"Empty file: {file_path}")

    return len(issues) == 0, issues


def validate_component_yaml_generator() -> tuple[bool, list[str]]:
    """Test ComponentYamlGenerator functionality."""
    issues = []

    try:
        from orchestration.context.component_yaml_generator import ComponentYamlGenerator
    except ImportError as e:
        issues.append(f"Cannot import ComponentYamlGenerator: {e}")
        return False, issues

    # Test derive_from_context with temp directory
    with tempfile.TemporaryDirectory() as tmpdir:
        component_path = Path(tmpdir) / "test_component"
        component_path.mkdir()

        # Add a Python file to trigger detection
        (component_path / "src").mkdir()
        (component_path / "src" / "main.py").write_text("# test")

        try:
            metadata = ComponentYamlGenerator.derive_from_context(component_path)

            if not metadata.get("name"):
                issues.append("derive_from_context: missing 'name'")
            if not metadata.get("type"):
                issues.append("derive_from_context: missing 'type'")
            if not metadata.get("version"):
                issues.append("derive_from_context: missing 'version'")
            if "Python" not in metadata.get("tech_stack", ""):
                issues.append("derive_from_context: did not detect Python")

        except Exception as e:
            issues.append(f"derive_from_context error: {e}")

    # Test to_yaml
    try:
        test_metadata = {
            "name": "test_component",
            "type": "backend",
            "version": "0.1.0",
            "tech_stack": "Python",
            "responsibility": "Test component",
            "context_rules": ["../../orchestration/context/component-rules.md"],
        }
        yaml_output = ComponentYamlGenerator.to_yaml(test_metadata)

        if "name: test_component" not in yaml_output:
            issues.append("to_yaml: missing name field")
        if "type: backend" not in yaml_output:
            issues.append("to_yaml: missing type field")
        if "context_rules:" not in yaml_output:
            issues.append("to_yaml: missing context_rules section")

    except Exception as e:
        issues.append(f"to_yaml error: {e}")

    # Test generate_if_missing
    with tempfile.TemporaryDirectory() as tmpdir:
        component_path = Path(tmpdir) / "new_component"
        component_path.mkdir()
        (component_path / "src").mkdir()
        (component_path / "src" / "app.py").write_text("# app")

        try:
            result = ComponentYamlGenerator.generate_if_missing(component_path)
            if not result:
                issues.append("generate_if_missing: returned False for new component")

            yaml_path = component_path / "component.yaml"
            if not yaml_path.exists():
                issues.append("generate_if_missing: did not create component.yaml")

            # Second call should return False (already exists)
            result2 = ComponentYamlGenerator.generate_if_missing(component_path)
            if result2:
                issues.append("generate_if_missing: returned True for existing yaml")

        except Exception as e:
            issues.append(f"generate_if_missing error: {e}")

    return len(issues) == 0, issues


def validate_migration_script(project_root: Path) -> tuple[bool, list[str]]:
    """Validate migration script is importable and runnable."""
    issues = []

    try:
        from orchestration.migration.migrate_to_context_system import (
            check_context_files,
            migrate_components,
            report_claude_md_status,
        )
    except ImportError as e:
        issues.append(f"Cannot import migration functions: {e}")
        return False, issues

    # Test check_context_files on real project
    try:
        context_ok, context_issues = check_context_files(project_root)
        if not context_ok:
            issues.append(f"check_context_files failed: {context_issues}")
    except Exception as e:
        issues.append(f"check_context_files error: {e}")

    return len(issues) == 0, issues


def validate_create_component(project_root: Path) -> tuple[bool, list[str]]:
    """Validate create_component.py has new context methods."""
    issues = []

    create_component_path = project_root / "orchestration" / "cli" / "create_component.py"
    if not create_component_path.exists():
        issues.append("create_component.py not found")
        return False, issues

    content = create_component_path.read_text()

    # Check for new-style CLAUDE.md generation
    if "CONTEXT INCLUDES" not in content:
        issues.append("create_component.py: missing new-style CLAUDE.md generation")

    # Check for component.yaml generation
    if "generate_manifest" not in content:
        issues.append("create_component.py: missing generate_manifest method")

    # Check for context_rules reference
    if "context_rules" not in content:
        issues.append("create_component.py: missing context_rules in manifest")

    return len(issues) == 0, issues


def validate_orchestrate_command(project_root: Path) -> tuple[bool, list[str]]:
    """Validate orchestrate.md has context loading instructions."""
    issues = []

    orchestrate_path = project_root / "orchestration" / "commands" / "orchestrate.md"
    if not orchestrate_path.exists():
        issues.append("orchestrate.md not found")
        return False, issues

    content = orchestrate_path.read_text()

    # Check for context loading section
    if "Context Loading" not in content:
        issues.append("orchestrate.md: missing Context Loading section")

    # Check for component-rules.md reference
    if "component-rules.md" not in content:
        issues.append("orchestrate.md: missing component-rules.md reference")

    return len(issues) == 0, issues


def validate_system_manifest(project_root: Path) -> tuple[bool, list[str]]:
    """Validate SYSTEM_MANIFEST.json includes context files."""
    import json
    issues = []

    manifest_path = project_root / "SYSTEM_MANIFEST.json"
    if not manifest_path.exists():
        issues.append("SYSTEM_MANIFEST.json not found")
        return False, issues

    try:
        manifest = json.loads(manifest_path.read_text())
        system_files = manifest.get("system_files", [])

        required_context_files = [
            "orchestration/context/__init__.py",
            "orchestration/context/README.md",
            "orchestration/context/component-rules.md",
            "orchestration/context/component_yaml_generator.py",
            "orchestration/context/orchestration-rules.md",
        ]

        for ctx_file in required_context_files:
            if ctx_file not in system_files:
                issues.append(f"SYSTEM_MANIFEST.json: missing {ctx_file}")

        # Check migration script is listed
        if "orchestration/migration/migrate_to_context_system.py" not in system_files:
            issues.append("SYSTEM_MANIFEST.json: missing migrate_to_context_system.py")

    except json.JSONDecodeError as e:
        issues.append(f"SYSTEM_MANIFEST.json: invalid JSON - {e}")

    return len(issues) == 0, issues


def run_validation(project_root: Path) -> bool:
    """Run all validations."""
    print("=" * 60)
    print("Context System Validation")
    print("=" * 60)

    all_passed = True

    validators = [
        ("Context Files", lambda: validate_context_files(project_root)),
        ("ComponentYamlGenerator", validate_component_yaml_generator),
        ("Migration Script", lambda: validate_migration_script(project_root)),
        ("create_component.py", lambda: validate_create_component(project_root)),
        ("orchestrate.md", lambda: validate_orchestrate_command(project_root)),
        ("SYSTEM_MANIFEST.json", lambda: validate_system_manifest(project_root)),
    ]

    for name, validator in validators:
        print(f"\n[{name}]")
        try:
            passed, issues = validator()
            if passed:
                print(f"  ✅ PASSED")
            else:
                print(f"  ❌ FAILED")
                for issue in issues:
                    print(f"     - {issue}")
                all_passed = False
        except Exception as e:
            print(f"  ❌ ERROR: {e}")
            all_passed = False

    print("\n" + "=" * 60)
    if all_passed:
        print("✅ All validations PASSED")
    else:
        print("❌ Some validations FAILED")
    print("=" * 60)

    return all_passed


def main():
    """CLI entry point."""
    if len(sys.argv) > 1:
        project_root = Path(sys.argv[1]).resolve()
    else:
        # Default to project root (3 levels up from this file)
        project_root = Path(__file__).parent.parent.parent.resolve()

    if not (project_root / "orchestration").exists():
        print(f"❌ Not an orchestrated project: {project_root}")
        sys.exit(1)

    success = run_validation(project_root)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
