#!/usr/bin/env python3
"""
Multi-Language Import Updater Extension

Extends import_updater.py to support JavaScript, TypeScript, Go, Rust, and Java.

Version: 1.0.0
"""

import re
from pathlib import Path
from typing import List, Optional
from dataclasses import dataclass


@dataclass
class MultiLangImportStatement:
    """Language-agnostic import statement"""
    file_path: Path
    line_number: int
    original_line: str
    module_name: str
    imported_names: List[str]
    language: str  # "javascript", "typescript", "go", "rust", "java"


class MultiLanguageImportParser:
    """Parses imports for multiple languages"""

    # JavaScript/TypeScript patterns
    JS_IMPORT_PATTERN = re.compile(
        r'import\s+(?:'
        r'(?P<default>\w+)'  # import X from 'module'
        r'|(?:\{(?P<named>[^}]+)\})'  # import { X, Y } from 'module'
        r'|(?P<namespace>\*\s+as\s+\w+)'  # import * as X from 'module'
        r')\s+from\s+[\'"](?P<module>[^\'"]+)[\'"]'
    )
    JS_REQUIRE_PATTERN = re.compile(
        r'(?:const|let|var)\s+(?P<name>\w+)\s*=\s*require\([\'"](?P<module>[^\'"]+)[\'"]\)'
    )

    # Go patterns
    GO_IMPORT_PATTERN = re.compile(
        r'import\s+(?:'
        r'(?P<alias>\w+)\s+)?'  # Optional alias
        r'"(?P<module>[^"]+)"'
    )
    GO_IMPORT_BLOCK = re.compile(r'import\s*\(([^)]+)\)')

    # Rust patterns
    RUST_USE_PATTERN = re.compile(
        r'use\s+'
        r'(?P<module>[\w:]+)'
        r'(?:::(?P<items>\{[^}]+\}|\w+|\*))?'
    )

    # Java patterns
    JAVA_IMPORT_PATTERN = re.compile(
        r'import\s+'
        r'(?P<static>static\s+)?'
        r'(?P<package>[\w.]+)'
        r'(?:\.(?P<class>\w+|\*))?'
    )

    @staticmethod
    def parse_javascript(line: str, file_path: Path, line_num: int) -> Optional[MultiLangImportStatement]:
        """Parse JavaScript/TypeScript import"""
        # Try ES6 import
        match = MultiLanguageImportParser.JS_IMPORT_PATTERN.match(line.strip())
        if match:
            module = match.group('module')
            names = []
            if match.group('default'):
                names.append(match.group('default'))
            if match.group('named'):
                names.extend([n.strip() for n in match.group('named').split(',')])
            if match.group('namespace'):
                names.append(match.group('namespace'))

            return MultiLangImportStatement(
                file_path=file_path,
                line_number=line_num,
                original_line=line,
                module_name=module,
                imported_names=names,
                language="javascript"
            )

        # Try CommonJS require
        match = MultiLanguageImportParser.JS_REQUIRE_PATTERN.match(line.strip())
        if match:
            return MultiLangImportStatement(
                file_path=file_path,
                line_number=line_num,
                original_line=line,
                module_name=match.group('module'),
                imported_names=[match.group('name')],
                language="javascript"
            )

        return None

    @staticmethod
    def parse_go(line: str, file_path: Path, line_num: int) -> Optional[MultiLangImportStatement]:
        """Parse Go import"""
        match = MultiLanguageImportParser.GO_IMPORT_PATTERN.match(line.strip())
        if match:
            module = match.group('module')
            alias = match.group('alias') if match.group('alias') else module.split('/')[-1]

            return MultiLangImportStatement(
                file_path=file_path,
                line_number=line_num,
                original_line=line,
                module_name=module,
                imported_names=[alias],
                language="go"
            )

        return None

    @staticmethod
    def parse_rust(line: str, file_path: Path, line_num: int) -> Optional[MultiLangImportStatement]:
        """Parse Rust use statement"""
        match = MultiLanguageImportParser.RUST_USE_PATTERN.match(line.strip())
        if match:
            module = match.group('module')
            items_str = match.group('items')

            names = []
            if items_str:
                if items_str.startswith('{'):
                    # Parse {A, B, C}
                    items_str = items_str.strip('{}')
                    names = [item.strip() for item in items_str.split(',')]
                elif items_str == '*':
                    names = ['*']
                else:
                    names = [items_str]

            return MultiLangImportStatement(
                file_path=file_path,
                line_number=line_num,
                original_line=line,
                module_name=module,
                imported_names=names,
                language="rust"
            )

        return None

    @staticmethod
    def parse_java(line: str, file_path: Path, line_num: int) -> Optional[MultiLangImportStatement]:
        """Parse Java import"""
        match = MultiLanguageImportParser.JAVA_IMPORT_PATTERN.match(line.strip())
        if match:
            package = match.group('package')
            class_name = match.group('class') if match.group('class') else ''

            full_name = f"{package}.{class_name}" if class_name else package

            return MultiLangImportStatement(
                file_path=file_path,
                line_number=line_num,
                original_line=line,
                module_name=full_name,
                imported_names=[class_name] if class_name else [],
                language="java"
            )

        return None


class MultiLanguageImportUpdater:
    """Updates imports for multiple languages"""

    def __init__(self, project_dir: Path):
        self.project_dir = Path(project_dir).resolve()
        self.parser = MultiLanguageImportParser()

    def scan_all_imports(self) -> List[MultiLangImportStatement]:
        """Scan imports in all supported languages"""
        imports = []

        # JavaScript/TypeScript
        for ext in ['.js', '.jsx', '.ts', '.tsx']:
            imports.extend(self._scan_language_files(ext, 'javascript'))

        # Go
        imports.extend(self._scan_language_files('.go', 'go'))

        # Rust
        imports.extend(self._scan_language_files('.rs', 'rust'))

        # Java
        imports.extend(self._scan_language_files('.java', 'java'))

        return imports

    def _scan_language_files(self, extension: str, language: str) -> List[MultiLangImportStatement]:
        """Scan files with specific extension"""
        imports = []
        files = list(self.project_dir.rglob(f'*{extension}'))

        # Exclude common non-source directories
        excluded = {'.git', 'node_modules', 'target', 'dist', 'build', '__pycache__'}
        files = [f for f in files if not any(ex in f.parts for ex in excluded)]

        for file_path in files:
            try:
                content = file_path.read_text()
                for line_num, line in enumerate(content.splitlines(), 1):
                    import_stmt = self._parse_line(line, file_path, line_num, language)
                    if import_stmt:
                        imports.append(import_stmt)
            except (UnicodeDecodeError, PermissionError):
                continue

        return imports

    def _parse_line(self, line: str, file_path: Path, line_num: int, language: str) -> Optional[MultiLangImportStatement]:
        """Parse line based on language"""
        if language == 'javascript':
            return self.parser.parse_javascript(line, file_path, line_num)
        elif language == 'go':
            return self.parser.parse_go(line, file_path, line_num)
        elif language == 'rust':
            return self.parser.parse_rust(line, file_path, line_num)
        elif language == 'java':
            return self.parser.parse_java(line, file_path, line_num)

        return None

    def fix_import(self, import_stmt: MultiLangImportStatement, old_path: str, new_path: str) -> str:
        """Generate fixed import line"""
        # Calculate new module path
        new_module = self._calculate_new_module_path(import_stmt.module_name, old_path, new_path, import_stmt.language)

        # Generate new import line based on language
        if import_stmt.language == 'javascript':
            return self._generate_js_import(import_stmt, new_module)
        elif import_stmt.language == 'go':
            return self._generate_go_import(import_stmt, new_module)
        elif import_stmt.language == 'rust':
            return self._generate_rust_use(import_stmt, new_module)
        elif import_stmt.language == 'java':
            return self._generate_java_import(import_stmt, new_module)

        return import_stmt.original_line

    def _calculate_new_module_path(self, module: str, old_path: str, new_path: str, language: str) -> str:
        """Calculate new module path after move"""
        # Language-specific path calculation
        if language in ['javascript', 'typescript']:
            # Relative paths (./ or ../)
            if module.startswith('.'):
                # Recalculate relative path
                return self._recalculate_relative_js_path(module, old_path, new_path)
            else:
                # Package import - no change
                return module

        elif language == 'go':
            # Go module paths are absolute - may need updating if internal package moved
            return module.replace(old_path, new_path)

        elif language == 'rust':
            # Rust paths use :: separator
            return module.replace(old_path.replace('/', '::'), new_path.replace('/', '::'))

        elif language == 'java':
            # Java package names
            return module.replace(old_path.replace('/', '.'), new_path.replace('/', '.'))

        return module

    def _recalculate_relative_js_path(self, module: str, old_path: str, new_path: str) -> str:
        """Recalculate relative JavaScript path"""
        # Simple implementation - calculate new relative path
        # Full implementation would use Path.relative_to()
        return module  # Placeholder

    def _generate_js_import(self, stmt: MultiLangImportStatement, new_module: str) -> str:
        """Generate JavaScript import statement"""
        if '{' in stmt.original_line:
            # Named imports
            names = ', '.join(stmt.imported_names)
            return f"import {{ {names} }} from '{new_module}';"
        else:
            # Default import
            return f"import {stmt.imported_names[0]} from '{new_module}';"

    def _generate_go_import(self, stmt: MultiLangImportStatement, new_module: str) -> str:
        """Generate Go import statement"""
        if len(stmt.imported_names) > 0 and stmt.imported_names[0] != new_module.split('/')[-1]:
            # Has alias
            return f'import {stmt.imported_names[0]} "{new_module}"'
        else:
            return f'import "{new_module}"'

    def _generate_rust_use(self, stmt: MultiLangImportStatement, new_module: str) -> str:
        """Generate Rust use statement"""
        if stmt.imported_names:
            if stmt.imported_names == ['*']:
                return f"use {new_module}::*;"
            elif len(stmt.imported_names) > 1:
                names = ', '.join(stmt.imported_names)
                return f"use {new_module}::{{{names}}};"
            else:
                return f"use {new_module}::{stmt.imported_names[0]};"
        else:
            return f"use {new_module};"

    def _generate_java_import(self, stmt: MultiLangImportStatement, new_module: str) -> str:
        """Generate Java import statement"""
        static = "static " if "static" in stmt.original_line else ""
        return f"import {static}{new_module};"


def main():
    """CLI entry point"""
    import sys
    import json

    if len(sys.argv) < 2:
        print("Usage: python import_updater_multilang.py <project_dir>")
        return 1

    project_dir = Path(sys.argv[1])
    updater = MultiLanguageImportUpdater(project_dir)

    print("Scanning multi-language imports...")
    imports = updater.scan_all_imports()

    # Group by language
    by_lang = {}
    for imp in imports:
        if imp.language not in by_lang:
            by_lang[imp.language] = []
        by_lang[imp.language].append(imp)

    print("\nImport Statistics:")
    for lang, imps in sorted(by_lang.items()):
        print(f"  {lang}: {len(imps)} imports")

    return 0


if __name__ == "__main__":
    sys.exit(main())
