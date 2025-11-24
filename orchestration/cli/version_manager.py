#!/usr/bin/env python3
"""
Version Manager

Manages version numbers across the orchestration system.
Ensures single source of truth and prevents version mismatches.

Usage:
    python orchestration/version_manager.py check          # Verify all versions match
    python orchestration/version_manager.py update 0.5.0   # Update all version references
    python orchestration/version_manager.py current        # Show current version
"""

import json
import re
import sys
from pathlib import Path
from typing import List, Tuple, Optional


class VersionManager:
    """Manages version numbers across all project files."""
    
    # Single source of truth
    METADATA_FILE = "orchestration/config/project_metadata.json"
    
    # Files that contain version references
    VERSION_FILES = [
        {
            "path": "claude-orchestration-system/README.md",
            "patterns": [
                (r'\*\*Version\*\*:\s*(\d+\.\d+\.\d+)', r'**Version**: {}'),
            ]
        },
        {
            "path": "claude-orchestration-system/docs/COMPONENT-ARCHITECTURE-GUIDE.md",
            "patterns": [
                (r'\*\*Version\*\*:\s*(\d+\.\d+\.\d+)', r'**Version**: {}'),
            ]
        },
        {
            "path": "claude-orchestration-system/docs/COMPLETION-GUARANTEE-GUIDE.md",
            "patterns": [
                (r'\*\*Version\*\*:\s*(\d+\.\d+\.\d+)', r'**Version**: {}'),
            ],
            "optional": True  # File might not exist
        },
    ]
    
    def __init__(self, project_root: Path):
        """Initialize version manager."""
        self.project_root = Path(project_root).resolve()
        self.metadata_path = self.project_root / self.METADATA_FILE
    
    def get_current_version(self) -> str:
        """Get current version from metadata (single source of truth)."""
        if not self.metadata_path.exists():
            raise FileNotFoundError(f"Metadata file not found: {self.metadata_path}")
        
        with open(self.metadata_path, 'r') as f:
            metadata = json.load(f)
        
        return metadata.get('version', 'unknown')
    
    def set_version(self, new_version: str) -> None:
        """Update version in metadata file."""
        if not re.match(r'^\d+\.\d+\.\d+$', new_version):
            raise ValueError(f"Invalid version format: {new_version} (expected: X.Y.Z)")
        
        with open(self.metadata_path, 'r') as f:
            metadata = json.load(f)
        
        old_version = metadata.get('version', 'unknown')
        metadata['version'] = new_version
        
        with open(self.metadata_path, 'w') as f:
            json.dump(metadata, f, indent=2)
        
        print(f"✅ Updated metadata: {old_version} → {new_version}")
    
    def check_consistency(self) -> Tuple[bool, List[str]]:
        """
        Check if all version references match the metadata version.
        
        Returns:
            (is_consistent, list_of_issues)
        """
        current_version = self.get_current_version()
        issues = []
        
        for file_info in self.VERSION_FILES:
            file_path = self.project_root / file_info['path']
            
            # Skip optional files that don't exist
            if file_info.get('optional', False) and not file_path.exists():
                continue
            
            if not file_path.exists():
                issues.append(f"❌ File not found: {file_info['path']}")
                continue
            
            content = file_path.read_text()
            
            for pattern, _ in file_info['patterns']:
                matches = re.findall(pattern, content)
                
                for match in matches:
                    if match != current_version:
                        issues.append(
                            f"❌ Version mismatch in {file_info['path']}: "
                            f"found {match}, expected {current_version}"
                        )
        
        return (len(issues) == 0, issues)
    
    def update_all_files(self, new_version: str) -> List[str]:
        """
        Update version in all files.
        
        Args:
            new_version: New version string (e.g., "0.5.0")
        
        Returns:
            List of files updated
        """
        if not re.match(r'^\d+\.\d+\.\d+$', new_version):
            raise ValueError(f"Invalid version format: {new_version} (expected: X.Y.Z)")
        
        # First update the single source of truth
        self.set_version(new_version)
        
        updated_files = []
        
        for file_info in self.VERSION_FILES:
            file_path = self.project_root / file_info['path']
            
            # Skip optional files that don't exist
            if file_info.get('optional', False) and not file_path.exists():
                print(f"⚠️  Skipping optional file: {file_info['path']} (does not exist)")
                continue
            
            if not file_path.exists():
                print(f"⚠️  File not found: {file_info['path']}")
                continue
            
            content = file_path.read_text()
            original_content = content
            
            # Apply all pattern replacements
            for pattern, replacement_template in file_info['patterns']:
                replacement = replacement_template.format(new_version)
                content = re.sub(pattern, replacement, content)
            
            # Only write if content changed
            if content != original_content:
                file_path.write_text(content)
                updated_files.append(file_info['path'])
                print(f"✅ Updated: {file_info['path']}")
            else:
                print(f"⏭️  No changes needed: {file_info['path']}")
        
        return updated_files
    
    def generate_pre_commit_hook(self) -> str:
        """Generate pre-commit hook script content (v0.5.0: added test quality check)."""
        return '''#!/bin/bash
# Pre-commit hook to verify version consistency and test quality (v0.5.0)

echo "🔍 Pre-commit checks..."
echo ""

# Check 1: Version consistency
echo "1️⃣  Checking version consistency..."
python orchestration/version_manager.py check > /dev/null 2>&1

if [ $? -ne 0 ]; then
    echo "   ❌ Version mismatch detected!"
    echo "   Run: python orchestration/version_manager.py check"
    echo ""
    exit 1
fi
echo "   ✅ Version consistency verified"
echo ""

# Check 2: Test quality (v0.5.0) - only for components with staged test files
echo "2️⃣  Checking test quality..."

# Get list of staged test files
STAGED_TEST_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep "^components/.*/tests/.*\.py$" || true)

if [ -z "$STAGED_TEST_FILES" ]; then
    echo "   ⏭️  No test files staged, skipping test quality check"
else
    # Extract unique component directories
    COMPONENTS=$(echo "$STAGED_TEST_FILES" | cut -d'/' -f1-2 | sort -u)

    TEST_QUALITY_FAILED=0

    for COMPONENT in $COMPONENTS; do
        if [ -d "$COMPONENT" ]; then
            echo "   Checking: $COMPONENT"
            python orchestration/test_quality_checker.py "$COMPONENT" > /dev/null 2>&1

            if [ $? -ne 0 ]; then
                echo "   ❌ Test quality issues in $COMPONENT"
                echo "      Run: python orchestration/test_quality_checker.py $COMPONENT"
                TEST_QUALITY_FAILED=1
            fi
        fi
    done

    if [ $TEST_QUALITY_FAILED -eq 1 ]; then
        echo ""
        echo "❌ Test quality check failed! Fix issues before committing."
        echo "   Or bypass with: git commit --no-verify (not recommended)"
        echo ""
        exit 1
    fi

    echo "   ✅ Test quality verified"
fi

echo ""
echo "✅ All pre-commit checks passed!"
exit 0
'''


def main():
    """CLI interface for version manager."""
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python orchestration/version_manager.py check")
        print("  python orchestration/version_manager.py update <version>")
        print("  python orchestration/version_manager.py current")
        print("  python orchestration/version_manager.py install-hook")
        print("")
        print("Examples:")
        print("  python orchestration/version_manager.py check")
        print("  python orchestration/version_manager.py update 0.5.0")
        sys.exit(1)
    
    command = sys.argv[1]
    project_root = Path.cwd()
    manager = VersionManager(project_root)
    
    if command == "current":
        version = manager.get_current_version()
        print(f"Current version: {version}")
        sys.exit(0)
    
    elif command == "check":
        version = manager.get_current_version()
        print(f"Current version: {version}")
        print("")
        
        is_consistent, issues = manager.check_consistency()
        
        if is_consistent:
            print("✅ All version references are consistent!")
            sys.exit(0)
        else:
            print("❌ Version inconsistencies found:\n")
            for issue in issues:
                print(f"  {issue}")
            print("")
            print(f"To fix, run: python orchestration/version_manager.py update {version}")
            sys.exit(1)
    
    elif command == "update":
        if len(sys.argv) < 3:
            print("Error: Version required")
            print("Usage: python orchestration/version_manager.py update <version>")
            print("Example: python orchestration/version_manager.py update 0.5.0")
            sys.exit(1)
        
        new_version = sys.argv[2]
        
        print(f"📝 Updating version to {new_version}...")
        print("")
        
        try:
            updated_files = manager.update_all_files(new_version)
            
            print("")
            print(f"✅ Version updated to {new_version}")
            print(f"   Files updated: {len(updated_files)}")
            print("")
            print("Next steps:")
            print(f"  1. Review changes: git diff")
            print(f"  2. Commit changes: git add . && git commit -m 'chore: Bump version to {new_version}'")
            print(f"  3. Verify: python orchestration/version_manager.py check")
            
            sys.exit(0)
        
        except Exception as e:
            print(f"❌ Error: {e}")
            sys.exit(1)
    
    elif command == "install-hook":
        hook_content = manager.generate_pre_commit_hook()
        hook_path = project_root / ".git" / "hooks" / "pre-commit"
        
        hook_path.parent.mkdir(parents=True, exist_ok=True)
        hook_path.write_text(hook_content)
        hook_path.chmod(0o755)
        
        print(f"✅ Pre-commit hook installed: {hook_path}")
        print("")
        print("The hook will automatically check version consistency before each commit.")
        print("To bypass the hook (not recommended): git commit --no-verify")
        sys.exit(0)
    
    else:
        print(f"Unknown command: {command}")
        sys.exit(1)


if __name__ == '__main__':
    main()
