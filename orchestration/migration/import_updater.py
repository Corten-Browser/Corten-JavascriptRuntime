#!/usr/bin/env python3
"""
Import Path Updater - Fix imports after code reorganization

After files are moved during onboarding, import paths need to be updated.
This tool automates the detection and fixing of broken import statements.

Usage:
    python orchestration/migration/import_updater.py scan <project_dir>
    python orchestration/migration/import_updater.py fix <project_dir> [--dry-run]
    python orchestration/migration/import_updater.py apply <plan.json>

Features:
    - Detects broken imports after file moves
    - Suggests correct import paths based on new structure
    - Handles relative and absolute imports
    - Preserves import style (from X import Y vs import X.Y)
    - Generates fix plan with preview
    - Applies fixes with git commit
"""

import sys
import re
import json
from pathlib import Path
from typing import List, Dict, Tuple, Optional, Set
from dataclasses import dataclass, field, asdict
from collections import defaultdict


@dataclass
class ImportStatement:
    """Represents a single import statement"""
    file_path: Path
    line_number: int
    original_line: str
    module_name: str
    imported_names: List[str]
    is_relative: bool
    is_from_import: bool


@dataclass
class ImportFix:
    """Represents a fix for a broken import"""
    import_stmt: ImportStatement
    old_module: str
    new_module: str
    reason: str
    confidence: str  # "high", "medium", "low"


@dataclass
class UpdatePlan:
    """Complete plan for updating imports"""
    project_dir: Path
    fixes: List[ImportFix]
    files_affected: int
    total_changes: int
    high_confidence: int
    medium_confidence: int
    low_confidence: int


class ImportUpdater:
    """Detects and fixes broken imports after code reorganization"""

    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()
        self.file_moves: Dict[Path, Path] = {}  # old_path -> new_path
        self.module_map: Dict[str, Path] = {}   # module.name -> file_path

    def load_file_moves(self, moves: List[Tuple[str, str]]):
        """
        Load file movement map from reorganization plan
        moves: [(source_path, dest_path), ...]
        """
        for source, dest in moves:
            source_path = self.project_dir / source
            dest_path = self.project_dir / dest
            self.file_moves[source_path] = dest_path

        print(f"Loaded {len(self.file_moves)} file movements")

    def build_module_map(self):
        """
        Build map of module names to file paths in new structure
        """
        print("Building module map from new structure...")

        # Find all Python files
        py_files = list(self.project_dir.rglob('*.py'))

        # Exclude orchestration system files and common non-source dirs
        excluded_dirs = {'.git', 'venv', '.venv', '__pycache__', 'dist', 'build', 'orchestration'}
        py_files = [
            f for f in py_files
            if not any(excluded in f.parts for excluded in excluded_dirs)
        ]

        for py_file in py_files:
            # Calculate module name from path
            rel_path = py_file.relative_to(self.project_dir)

            # Remove .py extension
            module_parts = list(rel_path.parts[:-1]) + [rel_path.stem]

            # Remove __init__
            if module_parts[-1] == '__init__':
                module_parts = module_parts[:-1]

            module_name = '.'.join(module_parts)
            self.module_map[module_name] = py_file

        print(f"✓ Built module map with {len(self.module_map)} modules")

    def scan_imports(self) -> List[ImportStatement]:
        """
        Scan all Python files for import statements
        """
        print("Scanning for import statements...")

        imports = []

        py_files = list(self.project_dir.rglob('*.py'))
        excluded_dirs = {'.git', 'venv', '.venv', '__pycache__', 'dist', 'build', 'orchestration'}
        py_files = [
            f for f in py_files
            if not any(excluded in f.parts for excluded in excluded_dirs)
        ]

        for py_file in py_files:
            try:
                content = py_file.read_text()
                for line_num, line in enumerate(content.splitlines(), 1):
                    import_stmt = self._parse_import_line(py_file, line_num, line)
                    if import_stmt:
                        imports.append(import_stmt)
            except (UnicodeDecodeError, PermissionError):
                continue

        print(f"✓ Found {len(imports)} import statements in {len(py_files)} files")
        return imports

    def detect_broken_imports(self, imports: List[ImportStatement]) -> List[ImportStatement]:
        """
        Detect imports that are likely broken after reorganization
        """
        print("Detecting broken imports...")

        broken = []

        for import_stmt in imports:
            # Check if module exists in new structure
            if not self._module_exists(import_stmt.module_name):
                broken.append(import_stmt)

        print(f"✓ Detected {len(broken)} potentially broken imports")
        return broken

    def generate_fixes(self, broken_imports: List[ImportStatement]) -> List[ImportFix]:
        """
        Generate fix suggestions for broken imports
        """
        print("Generating fix suggestions...")

        fixes = []

        for import_stmt in broken_imports:
            fix = self._suggest_fix(import_stmt)
            if fix:
                fixes.append(fix)

        print(f"✓ Generated {len(fixes)} fix suggestions")
        return fixes

    def create_update_plan(self, fixes: List[ImportFix]) -> UpdatePlan:
        """
        Create comprehensive update plan
        """
        files_affected = len(set(fix.import_stmt.file_path for fix in fixes))

        high = sum(1 for f in fixes if f.confidence == "high")
        medium = sum(1 for f in fixes if f.confidence == "medium")
        low = sum(1 for f in fixes if f.confidence == "low")

        return UpdatePlan(
            project_dir=self.project_dir,
            fixes=fixes,
            files_affected=files_affected,
            total_changes=len(fixes),
            high_confidence=high,
            medium_confidence=medium,
            low_confidence=low
        )

    def apply_fixes(self, plan: UpdatePlan, dry_run: bool = False) -> int:
        """
        Apply import fixes from plan
        Returns number of files modified
        """
        print(f"Applying {plan.total_changes} fixes to {plan.files_affected} files...")

        if dry_run:
            print("DRY RUN - No changes will be made")

        # Group fixes by file
        fixes_by_file = defaultdict(list)
        for fix in plan.fixes:
            fixes_by_file[fix.import_stmt.file_path].append(fix)

        files_modified = 0

        for file_path, file_fixes in fixes_by_file.items():
            if self._apply_fixes_to_file(file_path, file_fixes, dry_run):
                files_modified += 1

        if dry_run:
            print(f"✓ DRY RUN: Would modify {files_modified} files")
        else:
            print(f"✓ Modified {files_modified} files")

        return files_modified

    def _parse_import_line(self, file_path: Path, line_number: int, line: str) -> Optional[ImportStatement]:
        """
        Parse a single line for import statements
        """
        line = line.strip()

        # Skip comments and empty lines
        if not line or line.startswith('#'):
            return None

        # Pattern 1: from X import Y
        from_match = re.match(r'^from\s+(\.+)?([a-zA-Z0-9_.]+)\s+import\s+(.+)$', line)
        if from_match:
            relative_dots, module_name, imports = from_match.groups()
            is_relative = relative_dots is not None
            if is_relative:
                module_name = relative_dots + module_name

            # Parse imported names (handle "import a, b, c")
            imported_names = [name.strip().split()[0] for name in imports.split(',')]

            return ImportStatement(
                file_path=file_path,
                line_number=line_number,
                original_line=line,
                module_name=module_name,
                imported_names=imported_names,
                is_relative=is_relative,
                is_from_import=True
            )

        # Pattern 2: import X
        import_match = re.match(r'^import\s+([a-zA-Z0-9_.]+)(?:\s+as\s+\w+)?$', line)
        if import_match:
            module_name = import_match.group(1)

            return ImportStatement(
                file_path=file_path,
                line_number=line_number,
                original_line=line,
                module_name=module_name,
                imported_names=[],
                is_relative=False,
                is_from_import=False
            )

        return None

    def _module_exists(self, module_name: str) -> bool:
        """
        Check if module exists in new structure
        """
        # Handle relative imports
        if module_name.startswith('.'):
            # Relative imports are harder to validate without context
            # For now, assume they might be broken
            return False

        # Check if module is in our map
        if module_name in self.module_map:
            return True

        # Check for partial matches (submodules)
        for known_module in self.module_map.keys():
            if known_module.startswith(module_name + '.'):
                return True

        # Check if it's a standard library or third-party module
        if self._is_external_module(module_name):
            return True

        return False

    def _is_external_module(self, module_name: str) -> bool:
        """
        Check if module is from standard library or third-party package
        """
        # Common standard library modules
        stdlib = {
            'os', 'sys', 'pathlib', 'json', 'typing', 're', 'subprocess',
            'datetime', 'collections', 'dataclasses', 'enum', 'abc',
            'functools', 'itertools', 'argparse', 'logging', 'unittest',
            'pytest', 'io', 'copy', 'pickle', 'shutil', 'tempfile'
        }

        root_module = module_name.split('.')[0]
        return root_module in stdlib

    def _suggest_fix(self, import_stmt: ImportStatement) -> Optional[ImportFix]:
        """
        Suggest a fix for a broken import
        """
        old_module = import_stmt.module_name

        # Strategy 1: Look for exact module name match
        if old_module in self.module_map:
            # Module exists, might just need path update
            return None  # Actually not broken

        # Strategy 2: Look for partial matches (file might have moved)
        candidates = []
        module_parts = old_module.split('.')

        for known_module, file_path in self.module_map.items():
            known_parts = known_module.split('.')

            # Check if the last part (filename) matches
            if module_parts[-1] == known_parts[-1]:
                candidates.append((known_module, self._calculate_similarity(module_parts, known_parts)))

        if candidates:
            # Sort by similarity score
            candidates.sort(key=lambda x: x[1], reverse=True)
            best_match, score = candidates[0]

            confidence = "high" if score > 0.8 else "medium" if score > 0.5 else "low"

            return ImportFix(
                import_stmt=import_stmt,
                old_module=old_module,
                new_module=best_match,
                reason=f"File moved: {old_module} → {best_match}",
                confidence=confidence
            )

        # Strategy 3: Check if this was a workspace import that needs component prefix
        # Old: from src.utils import helper
        # New: from components.core.src.utils import helper

        for known_module in self.module_map.keys():
            if known_module.endswith('.' + old_module):
                return ImportFix(
                    import_stmt=import_stmt,
                    old_module=old_module,
                    new_module=known_module,
                    reason=f"Added component prefix: {old_module} → {known_module}",
                    confidence="medium"
                )

        return None  # No fix found

    def _calculate_similarity(self, parts1: List[str], parts2: List[str]) -> float:
        """
        Calculate similarity score between two module paths
        """
        # Simple heuristic: count matching parts
        matches = sum(1 for p1, p2 in zip(parts1, parts2) if p1 == p2)
        max_len = max(len(parts1), len(parts2))

        if max_len == 0:
            return 0.0

        return matches / max_len

    def _apply_fixes_to_file(self, file_path: Path, fixes: List[ImportFix], dry_run: bool) -> bool:
        """
        Apply all fixes to a single file
        Returns True if file was modified
        """
        try:
            content = file_path.read_text()
            lines = content.splitlines()

            # Apply fixes in reverse line order (to preserve line numbers)
            fixes_sorted = sorted(fixes, key=lambda f: f.import_stmt.line_number, reverse=True)

            modified = False
            for fix in fixes_sorted:
                line_idx = fix.import_stmt.line_number - 1
                if 0 <= line_idx < len(lines):
                    old_line = lines[line_idx]
                    new_line = old_line.replace(fix.old_module, fix.new_module)

                    if new_line != old_line:
                        lines[line_idx] = new_line
                        modified = True

                        if dry_run:
                            print(f"  {file_path.name}:{fix.import_stmt.line_number}")
                            print(f"    - {old_line}")
                            print(f"    + {new_line}")

            if modified and not dry_run:
                new_content = '\n'.join(lines)
                if content.endswith('\n'):
                    new_content += '\n'
                file_path.write_text(new_content)

            return modified

        except (UnicodeDecodeError, PermissionError) as e:
            print(f"✗ Error processing {file_path}: {e}")
            return False


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description="Update import paths after code reorganization"
    )
    parser.add_argument(
        'command',
        choices=['scan', 'fix', 'apply'],
        help='Command to run'
    )
    parser.add_argument(
        'project_dir',
        help='Project directory'
    )
    parser.add_argument(
        '--plan',
        help='JSON file with reorganization plan (for fix command)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Preview changes without modifying files'
    )
    parser.add_argument(
        '--output',
        default='import_fix_plan.json',
        help='Output file for fix plan (default: import_fix_plan.json)'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()
    updater = ImportUpdater(project_dir)

    if args.command == 'scan':
        # Just scan and report
        updater.build_module_map()
        imports = updater.scan_imports()
        broken = updater.detect_broken_imports(imports)

        print(f"\nSummary:")
        print(f"  Total imports: {len(imports)}")
        print(f"  Potentially broken: {len(broken)}")

        if broken:
            print(f"\nBroken imports:")
            for imp in broken[:20]:  # Show first 20
                print(f"  {imp.file_path.name}:{imp.line_number} - {imp.module_name}")
            if len(broken) > 20:
                print(f"  ... and {len(broken) - 20} more")

    elif args.command == 'fix':
        # Generate fix plan
        if args.plan:
            # Load file moves from reorganization plan
            plan_data = json.loads(Path(args.plan).read_text())
            file_moves = [(m['source'], m['destination']) for m in plan_data.get('file_moves', [])]
            updater.load_file_moves(file_moves)

        updater.build_module_map()
        imports = updater.scan_imports()
        broken = updater.detect_broken_imports(imports)
        fixes = updater.generate_fixes(broken)
        plan = updater.create_update_plan(fixes)

        # Save plan
        plan_data = {
            'project_dir': str(plan.project_dir),
            'summary': {
                'files_affected': plan.files_affected,
                'total_changes': plan.total_changes,
                'high_confidence': plan.high_confidence,
                'medium_confidence': plan.medium_confidence,
                'low_confidence': plan.low_confidence
            },
            'fixes': [
                {
                    'file': str(f.import_stmt.file_path.relative_to(project_dir)),
                    'line': f.import_stmt.line_number,
                    'old_module': f.old_module,
                    'new_module': f.new_module,
                    'reason': f.reason,
                    'confidence': f.confidence
                }
                for f in plan.fixes
            ]
        }

        output_path = Path(args.output)
        output_path.write_text(json.dumps(plan_data, indent=2))
        print(f"\n✓ Fix plan saved: {output_path}")

        print(f"\nSummary:")
        print(f"  Files affected: {plan.files_affected}")
        print(f"  Total changes: {plan.total_changes}")
        print(f"  High confidence: {plan.high_confidence}")
        print(f"  Medium confidence: {plan.medium_confidence}")
        print(f"  Low confidence: {plan.low_confidence}")

        print(f"\nNext steps:")
        print(f"  1. Review: {output_path}")
        print(f"  2. Apply: python {__file__} apply {project_dir} --plan {output_path}")

    elif args.command == 'apply':
        # Apply fixes from plan
        if not args.plan:
            print("ERROR: --plan required for apply command")
            sys.exit(1)

        plan_data = json.loads(Path(args.plan).read_text())

        # Reconstruct fixes
        updater.build_module_map()
        fixes = []

        for fix_data in plan_data['fixes']:
            file_path = project_dir / fix_data['file']

            import_stmt = ImportStatement(
                file_path=file_path,
                line_number=fix_data['line'],
                original_line="",  # Will be read when applying
                module_name=fix_data['old_module'],
                imported_names=[],
                is_relative=False,
                is_from_import=True
            )

            fix = ImportFix(
                import_stmt=import_stmt,
                old_module=fix_data['old_module'],
                new_module=fix_data['new_module'],
                reason=fix_data['reason'],
                confidence=fix_data['confidence']
            )

            fixes.append(fix)

        plan = updater.create_update_plan(fixes)
        files_modified = updater.apply_fixes(plan, dry_run=args.dry_run)

        if not args.dry_run:
            print(f"\n✓ Import fixes applied to {files_modified} files")
            print(f"\nNext steps:")
            print(f"  1. Run tests to verify changes")
            print(f"  2. Commit: git add -A && git commit -m 'fix: update import paths after reorganization'")


if __name__ == '__main__':
    main()
