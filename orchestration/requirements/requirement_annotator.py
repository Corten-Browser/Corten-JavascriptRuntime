#!/usr/bin/env python3
"""
Automatic Requirement Annotation Tool

Annotates code and tests with requirement IDs to enable traceability.
Part of v0.4.0 quality enhancement system - Batch 2.
"""

import ast
import re
from pathlib import Path
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass


@dataclass
class Annotation:
    """A code annotation."""
    file_path: Path
    line_number: int
    annotation_type: str  # "implements", "validates"
    requirement_id: str
    context: str  # function/class name


class RequirementAnnotator:
    """Automatically annotates code with requirement IDs."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.annotations_made: List[Annotation] = []

    def annotate_function(self,
                         file_path: Path,
                         function_name: str,
                         requirement_id: str) -> bool:
        """Add @implements decorator to function."""
        try:
            content = file_path.read_text()
            lines = content.split('\n')

            # Find function definition
            for i, line in enumerate(lines):
                if f'def {function_name}(' in line:
                    # Check if already annotated
                    if i > 0 and '@implements:' in lines[i-1]:
                        return False  # Already annotated

                    # Add annotation
                    indent = len(line) - len(line.lstrip())
                    annotation = ' ' * indent + f'# @implements: {requirement_id}'
                    lines.insert(i, annotation)

                    # Write back
                    file_path.write_text('\n'.join(lines))

                    self.annotations_made.append(Annotation(
                        file_path=file_path,
                        line_number=i+1,
                        annotation_type="implements",
                        requirement_id=requirement_id,
                        context=function_name
                    ))

                    return True

            return False
        except Exception as e:
            print(f"Error annotating {file_path}: {e}")
            return False

    def annotate_test(self,
                      file_path: Path,
                      test_name: str,
                      requirement_id: str) -> bool:
        """Add @validates decorator to test."""
        try:
            content = file_path.read_text()
            lines = content.split('\n')

            # Find test function
            for i, line in enumerate(lines):
                if f'def {test_name}(' in line:
                    # Check if already annotated
                    if i > 0 and '@validates:' in lines[i-1]:
                        return False

                    # Add annotation
                    indent = len(line) - len(line.lstrip())
                    annotation = ' ' * indent + f'# @validates: {requirement_id}'
                    lines.insert(i, annotation)

                    # Write back
                    file_path.write_text('\n'.join(lines))

                    self.annotations_made.append(Annotation(
                        file_path=file_path,
                        line_number=i+1,
                        annotation_type="validates",
                        requirement_id=requirement_id,
                        context=test_name
                    ))

                    return True

            return False
        except Exception as e:
            print(f"Error annotating {file_path}: {e}")
            return False

    def auto_annotate_component(self, component_path: Path) -> List[Annotation]:
        """
        Automatically annotate component based on heuristics.

        Heuristics:
        - Function name contains requirement keywords -> annotate
        - Test tests specific functionality -> annotate
        - Docstring mentions requirements -> annotate
        """
        annotations = []

        # Load requirements for project
        try:
            from orchestration.requirements.requirements_tracker import RequirementsTracker
            tracker = RequirementsTracker(self.project_root)

            # Get all requirements
            all_requirements = []
            for trace in tracker.requirements.values():
                all_requirements.append(trace.requirement)
        except Exception as e:
            print(f"Warning: Could not load requirements: {e}")
            return annotations

        # Annotate implementation files
        for py_file in component_path.rglob("*.py"):
            if "test" not in str(py_file):
                annotations.extend(self._auto_annotate_file(
                    py_file, all_requirements, "implements"
                ))

        # Annotate test files
        for test_file in component_path.rglob("test_*.py"):
            annotations.extend(self._auto_annotate_file(
                test_file, all_requirements, "validates"
            ))

        return annotations

    def _auto_annotate_file(self,
                           file_path: Path,
                           requirements: List,
                           annotation_type: str) -> List[Annotation]:
        """Auto-annotate a single file."""
        annotations = []

        try:
            tree = ast.parse(file_path.read_text())
        except Exception:
            return annotations

        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef):
                # Check if function relates to a requirement
                for req in requirements:
                    if self._function_matches_requirement(node, req):
                        # Annotate
                        if annotation_type == "implements":
                            success = self.annotate_function(
                                file_path, node.name, req.id
                            )
                        else:
                            success = self.annotate_test(
                                file_path, node.name, req.id
                            )

                        if success:
                            annotations.append(self.annotations_made[-1])
                        break  # One annotation per function

        return annotations

    def _function_matches_requirement(self, func: ast.FunctionDef, requirement) -> bool:
        """Check if function implements a requirement."""
        func_name_lower = func.name.lower()
        req_text_lower = requirement.text.lower()

        # Extract key terms from requirement
        req_terms = self._extract_key_terms(req_text_lower)

        # Check if function name contains requirement terms
        matches = sum(1 for term in req_terms if term in func_name_lower)

        # Also check docstring
        docstring = ast.get_docstring(func)
        if docstring:
            docstring_lower = docstring.lower()
            matches += sum(1 for term in req_terms if term in docstring_lower)

        # Threshold: at least 2 term matches
        return matches >= 2

    def _extract_key_terms(self, text: str) -> List[str]:
        """Extract key terms from requirement text."""
        # Remove common words
        stop_words = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at',
                     'to', 'for', 'of', 'with', 'by', 'from', 'as', 'is', 'are',
                     'must', 'should', 'shall', 'will', 'can', 'may'}

        # Extract words
        words = re.findall(r'\w+', text.lower())

        # Filter and return
        return [w for w in words if len(w) > 3 and w not in stop_words]

    def generate_annotation_report(self) -> str:
        """Generate report of annotations made."""
        report = []
        report.append("="*70)
        report.append("REQUIREMENT ANNOTATION REPORT")
        report.append("="*70)
        report.append("")
        report.append(f"Total Annotations: {len(self.annotations_made)}")
        report.append("")

        # Group by type
        implements = [a for a in self.annotations_made if a.annotation_type == "implements"]
        validates = [a for a in self.annotations_made if a.annotation_type == "validates"]

        report.append(f"Implementation Annotations: {len(implements)}")
        report.append(f"Test Annotations: {len(validates)}")
        report.append("")

        # List annotations
        if self.annotations_made:
            report.append("Annotations Made:")
            report.append("")
            for ann in self.annotations_made:
                report.append(f"  {ann.file_path.name}:{ann.line_number}")
                report.append(f"    @{ann.annotation_type}: {ann.requirement_id}")
                report.append(f"    Context: {ann.context}")
                report.append("")

        report.append("="*70)
        return "\n".join(report)

    def remove_annotations(self, file_path: Path) -> int:
        """Remove all requirement annotations from file."""
        try:
            content = file_path.read_text()
            lines = content.split('\n')

            # Remove annotation lines
            filtered_lines = [
                line for line in lines
                if not ('@implements:' in line or '@validates:' in line)
            ]

            removed = len(lines) - len(filtered_lines)

            if removed > 0:
                file_path.write_text('\n'.join(filtered_lines))

            return removed
        except Exception:
            return 0

    def get_annotations_for_file(self, file_path: Path) -> List[Tuple[int, str, str]]:
        """
        Get all annotations from a file.

        Returns:
            List of (line_number, annotation_type, requirement_id) tuples
        """
        annotations = []
        try:
            content = file_path.read_text()
            lines = content.split('\n')

            for i, line in enumerate(lines, 1):
                if '@implements:' in line:
                    req_id = line.split('@implements:')[1].strip()
                    annotations.append((i, 'implements', req_id))
                elif '@validates:' in line:
                    req_id = line.split('@validates:')[1].strip()
                    annotations.append((i, 'validates', req_id))
        except Exception:
            pass

        return annotations

    def get_all_annotations(self) -> Dict[Path, List[Tuple[int, str, str]]]:
        """
        Get all annotations from the project.

        Returns:
            Dict mapping file paths to lists of annotations
        """
        all_annotations = {}

        # Find all Python files in components directory
        components_dir = self.project_root / "components"
        if not components_dir.exists():
            return all_annotations

        for py_file in components_dir.rglob("*.py"):
            annotations = self.get_annotations_for_file(py_file)
            if annotations:
                all_annotations[py_file] = annotations

        return all_annotations


def main():
    """CLI interface."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: requirement_annotator.py <command> [args]")
        print("\nCommands:")
        print("  annotate-function <file> <function> <req_id>")
        print("  annotate-test <file> <test> <req_id>")
        print("  auto-annotate <component_path>")
        print("  remove <file>")
        print("  list [file]")
        sys.exit(1)

    command = sys.argv[1]
    annotator = RequirementAnnotator(Path.cwd())

    if command == "annotate-function":
        if len(sys.argv) < 5:
            print("Error: Missing arguments")
            print("Usage: annotate-function <file> <function> <req_id>")
            sys.exit(1)

        file_path = Path(sys.argv[2])
        function_name = sys.argv[3]
        req_id = sys.argv[4]

        success = annotator.annotate_function(file_path, function_name, req_id)
        print(f"{'✅' if success else '❌'} Annotation {'added' if success else 'failed'}")

    elif command == "annotate-test":
        if len(sys.argv) < 5:
            print("Error: Missing arguments")
            print("Usage: annotate-test <file> <test> <req_id>")
            sys.exit(1)

        file_path = Path(sys.argv[2])
        test_name = sys.argv[3]
        req_id = sys.argv[4]

        success = annotator.annotate_test(file_path, test_name, req_id)
        print(f"{'✅' if success else '❌'} Annotation {'added' if success else 'failed'}")

    elif command == "auto-annotate":
        if len(sys.argv) < 3:
            print("Error: Missing component path")
            print("Usage: auto-annotate <component_path>")
            sys.exit(1)

        component_path = Path(sys.argv[2])
        annotations = annotator.auto_annotate_component(component_path)
        print(annotator.generate_annotation_report())

    elif command == "remove":
        if len(sys.argv) < 3:
            print("Error: Missing file path")
            print("Usage: remove <file>")
            sys.exit(1)

        file_path = Path(sys.argv[2])
        removed = annotator.remove_annotations(file_path)
        print(f"Removed {removed} annotation(s)")

    elif command == "list":
        if len(sys.argv) >= 3:
            # List annotations for specific file
            file_path = Path(sys.argv[2])
            annotations = annotator.get_annotations_for_file(file_path)

            if annotations:
                print(f"\nAnnotations in {file_path}:")
                for line_num, ann_type, req_id in annotations:
                    print(f"  Line {line_num}: @{ann_type}: {req_id}")
            else:
                print(f"No annotations found in {file_path}")
        else:
            # List all annotations in project
            all_annotations = annotator.get_all_annotations()

            if all_annotations:
                print("\nAll Annotations in Project:")
                for file_path, annotations in all_annotations.items():
                    print(f"\n{file_path}:")
                    for line_num, ann_type, req_id in annotations:
                        print(f"  Line {line_num}: @{ann_type}: {req_id}")
            else:
                print("No annotations found in project")

    else:
        print(f"Error: Unknown command '{command}'")
        sys.exit(1)


if __name__ == '__main__':
    main()
