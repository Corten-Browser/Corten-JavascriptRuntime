#!/usr/bin/env python3
"""
Structure Analyzer and Migration Guide

Analyzes project structure and provides specific migration guidance.
Safer than automated migration - guides user through manual process.

Supports multi-language projects (Python, JavaScript/TypeScript, Go, Rust).

Part of Phase 3 in distribution-first redesign (v0.15.0).
Enhanced in v0.16.0 for multi-language support.

Usage:
    python structure_analyzer.py /path/to/project
"""

from pathlib import Path
from typing import Dict, List, Tuple
from dataclasses import dataclass, field
import re

# Import multi-language support
try:
    from language_support import detect_project_languages, LanguageInfo, HardcodedPath
    MULTI_LANGUAGE_AVAILABLE = True
except ImportError:
    MULTI_LANGUAGE_AVAILABLE = False


@dataclass
class StructureAnalysis:
    """Analysis results of project structure (multi-language)."""
    is_workspace_structure: bool
    is_package_structure: bool
    components_found: List[str]
    hardcoded_paths_found: List[Tuple[Path, int, str]]
    import_issues: List[str]
    recommendations: List[str]
    migration_needed: bool
    # v0.16.0: Multi-language support
    languages_detected: Dict[str, float] = field(default_factory=dict)  # {language: confidence}
    language_specific_issues: Dict[str, List[str]] = field(default_factory=dict)  # {language: issues}


class StructureAnalyzer:
    """Analyze project structure and provide migration guidance."""

    def __init__(self, project_root: Path):
        """
        Initialize structure analyzer.

        Args:
            project_root: Absolute path to project root
        """
        self.project_root = Path(project_root).resolve()

    def analyze(self) -> StructureAnalysis:
        """
        Analyze project structure (multi-language support in v0.16.0).

        Returns:
            StructureAnalysis with findings and recommendations
        """
        print(f"\n🔍 Analyzing Project Structure")
        print(f"   Project: {self.project_root.name}")
        print(f"   Path: {self.project_root}")
        print()

        # v0.16.0: Detect all languages in project
        languages_detected = {}
        language_specific_issues = {}

        if MULTI_LANGUAGE_AVAILABLE:
            detected_languages = detect_project_languages(self.project_root)

            if detected_languages:
                print("📦 Languages Detected:")
                for lang_support, lang_info in detected_languages:
                    languages_detected[lang_info.display_name] = lang_info.confidence
                    print(f"   - {lang_info.display_name}: {lang_info.confidence:.1%} confidence")
                print()

        # Check for workspace structure (Python-specific)
        is_workspace = self._is_workspace_structure()

        # Check for package structure (multi-language)
        is_package = self._is_package_structure_multi_language()

        # Find components
        components = self._find_components()

        # Check for hardcoded paths (multi-language)
        hardcoded_paths = self._find_hardcoded_paths_multi_language()

        # Check import patterns (multi-language)
        import_issues = self._check_import_patterns_multi_language()

        # Generate recommendations
        recommendations = self._generate_recommendations(
            is_workspace, is_package, components, hardcoded_paths, import_issues
        )

        # Determine if migration needed
        migration_needed = is_workspace or len(hardcoded_paths) > 0 or len(import_issues) > 0

        return StructureAnalysis(
            is_workspace_structure=is_workspace,
            is_package_structure=is_package,
            components_found=components,
            hardcoded_paths_found=hardcoded_paths,
            import_issues=import_issues,
            recommendations=recommendations,
            migration_needed=migration_needed,
            languages_detected=languages_detected,
            language_specific_issues=language_specific_issues
        )

    def _is_workspace_structure(self) -> bool:
        """Check if project uses workspace structure."""
        # Indicators: components/ directory with src/ subdirs
        components_dir = self.project_root / "components"

        if not components_dir.exists():
            return False

        # Check if any component has src/ subdirectory
        for item in components_dir.iterdir():
            if item.is_dir() and (item / "src").exists():
                return True

        return False

    def _is_package_structure(self) -> bool:
        """Check if project uses proper package structure."""
        # Indicators: setup.py exists and main package directory
        has_setup = (self.project_root / "setup.py").exists()
        has_pyproject = (self.project_root / "pyproject.toml").exists()

        if not (has_setup or has_pyproject):
            return False

        # Check for package directory (not components/)
        for item in self.project_root.iterdir():
            if item.is_dir() and not item.name.startswith('.'):
                if item.name not in ['components', 'tests', 'docs', 'examples']:
                    if (item / "__init__.py").exists():
                        return True

        return False

    def _find_components(self) -> List[str]:
        """Find component directories."""
        components = []
        components_dir = self.project_root / "components"

        if components_dir.exists():
            for item in components_dir.iterdir():
                if item.is_dir() and not item.name.startswith('.'):
                    components.append(item.name)

        return components

    def _find_hardcoded_paths(self) -> List[Tuple[Path, int, str]]:
        """Find hardcoded absolute paths in code."""
        hardcoded_paths = []

        # Patterns to detect
        patterns = [
            r'/home/',
            r'/workspaces/',
            r'/Users/',
            r'/root/',
            r'/opt/',
            r'C:\\',
            r'D:\\',
        ]

        # Scan Python files
        for py_file in self.project_root.glob("**/*.py"):
            if "__pycache__" in str(py_file) or ".git" in str(py_file):
                continue

            try:
                with open(py_file, 'r', encoding='utf-8', errors='ignore') as f:
                    for line_num, line in enumerate(f, 1):
                        for pattern in patterns:
                            if re.search(pattern, line):
                                hardcoded_paths.append((
                                    py_file.relative_to(self.project_root),
                                    line_num,
                                    line.strip()[:80]
                                ))
                                break  # One per line

            except Exception:
                pass

        return hardcoded_paths

    def _check_import_patterns(self) -> List[str]:
        """Check for problematic import patterns."""
        issues = []

        # Check for sys.path.append with absolute paths
        for py_file in self.project_root.glob("**/*.py"):
            if "__pycache__" in str(py_file):
                continue

            try:
                with open(py_file, 'r', encoding='utf-8') as f:
                    content = f.read()

                    # Check for sys.path.append
                    if re.search(r'sys\.path\.(append|insert)\s*\([^)]*["\']/', content):
                        issues.append(
                            f"{py_file.relative_to(self.project_root)}: "
                            f"Uses sys.path with absolute path"
                        )

                    # Check for workspace-style imports
                    if re.search(r'from components\.[\w_]+\.src\.', content):
                        issues.append(
                            f"{py_file.relative_to(self.project_root)}: "
                            f"Uses workspace-style imports (components.*.src)"
                        )

            except Exception:
                pass

        return issues

    def _generate_recommendations(
        self,
        is_workspace: bool,
        is_package: bool,
        components: List[str],
        hardcoded_paths: List,
        import_issues: List[str]
    ) -> List[str]:
        """Generate specific recommendations for this project."""
        recommendations = []

        if is_workspace and not is_package:
            recommendations.append(
                "CRITICAL: Project uses workspace structure - must migrate to package structure"
            )
            recommendations.append(
                f"Convert {len(components)} component(s) to package submodules"
            )

        if len(hardcoded_paths) > 0:
            recommendations.append(
                f"CRITICAL: Found {len(hardcoded_paths)} hardcoded absolute path(s) - must fix"
            )
            recommendations.append(
                "Replace hardcoded paths with Path(__file__).parent / 'relative/path'"
            )

        if len(import_issues) > 0:
            recommendations.append(
                f"WARNING: Found {len(import_issues)} problematic import(s)"
            )
            recommendations.append(
                "Convert workspace imports to package imports"
            )

        if not (self.project_root / "setup.py").exists() and not (self.project_root / "pyproject.toml").exists():
            recommendations.append(
                "REQUIRED: Create setup.py for package installation"
            )
            recommendations.append(
                "Use: python orchestration/package_generator.py ."
            )

        if not (self.project_root / "README.md").exists():
            recommendations.append(
                "REQUIRED: Create comprehensive README.md"
            )
            recommendations.append(
                "Use: python orchestration/readme_generator.py ."
            )

        if not recommendations:
            recommendations.append("✅ Project structure looks good!")
            recommendations.append("Run deployment verification to confirm")

        return recommendations

    def _is_package_structure_multi_language(self) -> bool:
        """
        Check if project has valid package structure (multi-language).

        v0.16.0: Uses language plugins if available, falls back to Python-only check.
        """
        if MULTI_LANGUAGE_AVAILABLE:
            detected_languages = detect_project_languages(self.project_root)

            # If any language has valid package structure, return True
            for lang_support, lang_info in detected_languages:
                is_valid, _, _ = lang_support.check_package_structure(self.project_root)
                if is_valid:
                    return True

            return False
        else:
            # Fallback to Python-only check
            return self._is_package_structure()

    def _find_hardcoded_paths_multi_language(self) -> List[Tuple[Path, int, str]]:
        """
        Find hardcoded paths across all detected languages.

        v0.16.0: Uses language plugins if available, falls back to Python-only.
        """
        if MULTI_LANGUAGE_AVAILABLE:
            all_hardcoded = []
            detected_languages = detect_project_languages(self.project_root)

            for lang_support, lang_info in detected_languages:
                paths = lang_support.find_hardcoded_paths(self.project_root)
                # Convert from HardcodedPath objects to tuples
                for hp in paths:
                    all_hardcoded.append((hp.file_path, hp.line_number, hp.line_content))

            return all_hardcoded
        else:
            # Fallback to Python-only
            return self._find_hardcoded_paths()

    def _check_import_patterns_multi_language(self) -> List[str]:
        """
        Check import patterns across all detected languages.

        v0.16.0: Uses language plugins if available, falls back to Python-only.
        """
        if MULTI_LANGUAGE_AVAILABLE:
            all_issues = []
            detected_languages = detect_project_languages(self.project_root)

            for lang_support, lang_info in detected_languages:
                issues = lang_support.check_import_patterns(self.project_root)
                all_issues.extend(issues)

            return all_issues
        else:
            # Fallback to Python-only
            return self._check_import_patterns()

    def print_report(self, analysis: StructureAnalysis):
        """Print analysis report (with multi-language support in v0.16.0)."""
        print("="*70)
        print("STRUCTURE ANALYSIS REPORT")
        print("="*70)
        print()

        # v0.16.0: Show detected languages
        if analysis.languages_detected:
            print("Languages Detected:")
            for lang_name, confidence in analysis.languages_detected.items():
                print(f"  ✅ {lang_name}: {confidence:.1%} confidence")
            print()

        # Structure type
        print("Structure Type:")
        if analysis.is_workspace_structure:
            print("  ❌ Workspace Structure (OLD - needs migration)")
        if analysis.is_package_structure:
            print("  ✅ Package Structure (NEW - distributable)")
        if not analysis.is_workspace_structure and not analysis.is_package_structure:
            print("  ⚠️  Unknown Structure")
        print()

        # Components
        if analysis.components_found:
            print(f"Components Found: {len(analysis.components_found)}")
            for comp in analysis.components_found[:10]:
                print(f"  - {comp}")
            if len(analysis.components_found) > 10:
                print(f"  ... and {len(analysis.components_found) - 10} more")
            print()

        # Hardcoded paths
        if analysis.hardcoded_paths_found:
            print(f"❌ Hardcoded Paths: {len(analysis.hardcoded_paths_found)}")
            for file_path, line_num, line_content in analysis.hardcoded_paths_found[:5]:
                print(f"  {file_path}:{line_num}")
                print(f"    {line_content}")
            if len(analysis.hardcoded_paths_found) > 5:
                print(f"  ... and {len(analysis.hardcoded_paths_found) - 5} more")
            print()

        # Import issues
        if analysis.import_issues:
            print(f"⚠️  Import Issues: {len(analysis.import_issues)}")
            for issue in analysis.import_issues[:5]:
                print(f"  - {issue}")
            if len(analysis.import_issues) > 5:
                print(f"  ... and {len(analysis.import_issues) - 5} more")
            print()

        # Recommendations
        print("Recommendations:")
        for rec in analysis.recommendations:
            if rec.startswith("CRITICAL"):
                print(f"  🔴 {rec}")
            elif rec.startswith("REQUIRED"):
                print(f"  🟠 {rec}")
            elif rec.startswith("WARNING"):
                print(f"  🟡 {rec}")
            elif rec.startswith("✅"):
                print(f"  {rec}")
            else:
                print(f"     {rec}")
        print()

        # Migration needed?
        if analysis.migration_needed:
            print("="*70)
            print("⚠️  MIGRATION REQUIRED")
            print("="*70)
            print()
            print("Next Steps:")
            print("  1. Read: docs/PACKAGE-STRUCTURE-STANDARD.md")
            print("  2. Backup: git commit -am 'Pre-migration backup'")
            print("  3. Generate setup.py: python orchestration/package_generator.py .")
            print("  4. Follow migration guide in PACKAGE-STRUCTURE-STANDARD.md")
            print("  5. Test: python orchestration/clean_install_tester.py . --auto-detect")
            print("  6. Verify: python orchestration/deployment_verifier.py . --auto-detect")
        else:
            print("="*70)
            print("✅ STRUCTURE OK - Run Final Verification")
            print("="*70)
            print()
            print("Recommended Tests:")
            print("  python orchestration/clean_install_tester.py . --auto-detect")
            print("  python orchestration/deployment_verifier.py . --auto-detect")

        print()


def main():
    """CLI interface for structure analysis."""
    import sys
    import argparse

    parser = argparse.ArgumentParser(
        description="Analyze project structure and provide migration guidance"
    )
    parser.add_argument(
        "project_root",
        nargs="?",
        default=".",
        help="Project root directory"
    )

    args = parser.parse_args()
    project_root = Path(args.project_root).resolve()

    # Run analysis
    analyzer = StructureAnalyzer(project_root)
    analysis = analyzer.analyze()

    # Print report
    analyzer.print_report(analysis)

    # Exit with appropriate code
    sys.exit(1 if analysis.migration_needed else 0)


if __name__ == '__main__':
    main()
