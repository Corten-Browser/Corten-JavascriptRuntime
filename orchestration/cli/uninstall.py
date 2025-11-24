#!/usr/bin/env python3
"""
Uninstall the orchestration system from a project.

This script removes all orchestration files and reverts changes made during
installation. Use with caution as this cannot be easily undone.

Usage:
    python orchestration/uninstall.py
    python orchestration/uninstall.py --dry-run
    python orchestration/uninstall.py --keep-components

Exit codes:
    0 - Uninstalled successfully
    1 - Uninstall failed or user cancelled
"""

import os
import sys
import shutil
import subprocess
from pathlib import Path
from typing import List, Tuple


class Colors:
    """ANSI color codes for terminal output."""
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'

    @classmethod
    def disable(cls):
        """Disable colors (for non-TTY output)."""
        cls.GREEN = cls.YELLOW = cls.RED = cls.BLUE = cls.NC = ''


def confirm_uninstall() -> bool:
    """Ask user to confirm uninstall."""
    print(f"{Colors.RED}⚠️  WARNING: This will remove the orchestration system{Colors.NC}")
    print("The following will be deleted:")
    print("  - orchestration/ directory")
    print("  - .claude/ directory")
    print("  - CLAUDE.md file")
    print("  - Orchestration entries in .gitignore")
    print("")
    response = input("Are you sure you want to continue? (yes/no): ")
    return response.lower() in ['yes', 'y']


def check_components_exist() -> Tuple[bool, List[str]]:
    """Check if any components exist."""
    components_dir = Path('components')
    if not components_dir.exists():
        return False, []

    # Find all subdirectories in components/
    components = [d.name for d in components_dir.iterdir() if d.is_dir() and not d.name.startswith('_')]
    return len(components) > 0, components


def remove_orchestration_dir(dry_run: bool = False) -> bool:
    """Remove orchestration directory."""
    orch_dir = Path('orchestration')
    if not orch_dir.exists():
        print(f"  {Colors.YELLOW}⚠{Colors.NC}  orchestration/ not found")
        return True

    if dry_run:
        print(f"  Would remove: orchestration/")
        return True

    try:
        shutil.rmtree(orch_dir)
        print(f"  {Colors.GREEN}✓{Colors.NC} Removed orchestration/")
        return True
    except Exception as e:
        print(f"  {Colors.RED}✗{Colors.NC} Failed to remove orchestration/: {e}")
        return False


def remove_claude_dir(dry_run: bool = False) -> bool:
    """Remove .claude directory."""
    claude_dir = Path('.claude')
    if not claude_dir.exists():
        print(f"  {Colors.YELLOW}⚠{Colors.NC}  .claude/ not found")
        return True

    if dry_run:
        print(f"  Would remove: .claude/")
        return True

    try:
        shutil.rmtree(claude_dir)
        print(f"  {Colors.GREEN}✓{Colors.NC} Removed .claude/")
        return True
    except Exception as e:
        print(f"  {Colors.RED}✗{Colors.NC} Failed to remove .claude/: {e}")
        return False


def remove_claude_md(dry_run: bool = False) -> bool:
    """Remove CLAUDE.md file."""
    claude_md = Path('CLAUDE.md')
    if not claude_md.exists():
        print(f"  {Colors.YELLOW}⚠{Colors.NC}  CLAUDE.md not found")
        return True

    if dry_run:
        print(f"  Would remove: CLAUDE.md")
        return True

    try:
        claude_md.unlink()
        print(f"  {Colors.GREEN}✓{Colors.NC} Removed CLAUDE.md")
        return True
    except Exception as e:
        print(f"  {Colors.RED}✗{Colors.NC} Failed to remove CLAUDE.md: {e}")
        return False


def remove_empty_dirs(keep_components: bool = False, dry_run: bool = False) -> bool:
    """Remove empty component directories."""
    dirs_to_check = ['contracts', 'shared-libs']
    if not keep_components:
        dirs_to_check.append('components')

    success = True
    for dir_name in dirs_to_check:
        dir_path = Path(dir_name)
        if not dir_path.exists():
            continue

        # Check if directory is empty or only contains hidden files
        contents = list(dir_path.iterdir())
        is_empty = len(contents) == 0 or all(f.name.startswith('.') for f in contents)

        if is_empty:
            if dry_run:
                print(f"  Would remove: {dir_name}/ (empty)")
            else:
                try:
                    shutil.rmtree(dir_path)
                    print(f"  {Colors.GREEN}✓{Colors.NC} Removed {dir_name}/ (empty)")
                except Exception as e:
                    print(f"  {Colors.RED}✗{Colors.NC} Failed to remove {dir_name}/: {e}")
                    success = False
        else:
            print(f"  {Colors.YELLOW}⚠{Colors.NC}  Keeping {dir_name}/ (not empty)")

    return success


def clean_gitignore(dry_run: bool = False) -> bool:
    """Remove orchestration entries from .gitignore."""
    gitignore = Path('.gitignore')
    if not gitignore.exists():
        print(f"  {Colors.YELLOW}⚠{Colors.NC}  .gitignore not found")
        return True

    if dry_run:
        print(f"  Would clean: .gitignore")
        return True

    try:
        # Read current content
        content = gitignore.read_text()

        # Find and remove orchestration section
        lines = content.split('\n')
        new_lines = []
        in_orchestration_section = False

        for line in lines:
            if '# Claude Code Orchestration' in line:
                in_orchestration_section = True
                continue
            elif in_orchestration_section:
                # Check if we've reached the next section or end
                if line.strip() and line.startswith('#') and 'orchestration' not in line.lower():
                    in_orchestration_section = False
                elif not line.strip():  # Empty line might signal end
                    # Check if next section starts
                    continue
                else:
                    continue  # Skip orchestration lines

            if not in_orchestration_section:
                new_lines.append(line)

        # Write cleaned content
        gitignore.write_text('\n'.join(new_lines))
        print(f"  {Colors.GREEN}✓{Colors.NC} Cleaned .gitignore")
        return True

    except Exception as e:
        print(f"  {Colors.RED}✗{Colors.NC} Failed to clean .gitignore: {e}")
        return False


def commit_removal(dry_run: bool = False) -> bool:
    """Commit the removal to git."""
    if not Path('.git').is_dir():
        print(f"  {Colors.YELLOW}⚠{Colors.NC}  Not a git repository, skipping commit")
        return True

    if dry_run:
        print(f"  Would commit removal to git")
        return True

    try:
        # Stage all deletions
        subprocess.run(['git', 'add', '-A'], check=True, capture_output=True)

        # Create commit
        subprocess.run([
            'git', 'commit', '-m',
            'chore: Remove Claude Code orchestration system\n\n'
            'Uninstalled orchestration system and cleaned up files.'
        ], check=True, capture_output=True)

        print(f"  {Colors.GREEN}✓{Colors.NC} Changes committed to git")
        return True

    except subprocess.CalledProcessError:
        print(f"  {Colors.YELLOW}⚠{Colors.NC}  No changes to commit")
        return True
    except Exception as e:
        print(f"  {Colors.RED}✗{Colors.NC} Failed to commit: {e}")
        return False


def uninstall(dry_run: bool = False, keep_components: bool = False) -> bool:
    """
    Uninstall orchestration system.

    Args:
        dry_run: Show what would be done without doing it
        keep_components: Keep components directory even if empty

    Returns:
        True if uninstall succeeded
    """
    print(f"{Colors.BLUE}{'=' * 63}{Colors.NC}")
    print(f"{Colors.BLUE}  Uninstalling Orchestration System{Colors.NC}")
    print(f"{Colors.BLUE}{'=' * 63}{Colors.NC}\n")

    if dry_run:
        print(f"{Colors.YELLOW}DRY RUN MODE - No changes will be made{Colors.NC}\n")

    # Check for components
    has_components, component_list = check_components_exist()
    if has_components and not keep_components:
        print(f"{Colors.YELLOW}⚠  Found {len(component_list)} component(s):{Colors.NC}")
        for comp in component_list[:5]:  # Show first 5
            print(f"   - {comp}")
        if len(component_list) > 5:
            print(f"   ... and {len(component_list) - 5} more")
        print("")
        if not dry_run:
            response = input("Remove components directory anyway? (yes/no): ")
            if response.lower() not in ['yes', 'y']:
                keep_components = True
                print("Keeping components directory\n")

    # Perform uninstall
    print("Removing orchestration system...")
    success = True

    success &= remove_orchestration_dir(dry_run)
    success &= remove_claude_dir(dry_run)
    success &= remove_claude_md(dry_run)
    success &= remove_empty_dirs(keep_components, dry_run)
    success &= clean_gitignore(dry_run)

    if not dry_run:
        success &= commit_removal(dry_run)

    print("")
    print(f"{Colors.BLUE}{'=' * 63}{Colors.NC}")

    if success:
        if dry_run:
            print(f"{Colors.GREEN}DRY RUN COMPLETE{Colors.NC}")
        else:
            print(f"{Colors.GREEN}UNINSTALL COMPLETE{Colors.NC}")
        print(f"{Colors.BLUE}{'=' * 63}{Colors.NC}\n")
        print("Orchestration system has been removed.")
        if keep_components:
            print("Component files have been preserved.")
        print("")
        return True
    else:
        print(f"{Colors.RED}UNINSTALL FAILED{Colors.NC}")
        print(f"{Colors.BLUE}{'=' * 63}{Colors.NC}\n")
        print("Some files could not be removed. Check errors above.")
        print("")
        return False


def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Uninstall Claude Code orchestration system'
    )
    parser.add_argument(
        '--dry-run', '-d',
        action='store_true',
        help='Show what would be done without doing it'
    )
    parser.add_argument(
        '--keep-components', '-k',
        action='store_true',
        help='Keep components directory'
    )
    parser.add_argument(
        '--yes', '-y',
        action='store_true',
        help='Skip confirmation prompt'
    )
    args = parser.parse_args()

    # Disable colors if not in TTY
    if not sys.stdout.isatty():
        Colors.disable()

    # Check if we're in a directory with orchestration system
    if not Path('orchestration').exists() and not Path('CLAUDE.md').exists():
        print(f"{Colors.RED}Error: Orchestration system not found{Colors.NC}")
        print("Run this command from a project with orchestration system installed.")
        return 1

    # Confirm uninstall (unless --yes flag)
    if not args.yes and not args.dry_run:
        if not confirm_uninstall():
            print("Uninstall cancelled.")
            return 1
        print("")

    # Perform uninstall
    success = uninstall(dry_run=args.dry_run, keep_components=args.keep_components)

    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
