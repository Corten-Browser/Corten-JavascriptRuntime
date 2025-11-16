"""
Dependency Manager

Manages component dependencies, resolves build order, and validates dependency graphs.

Classes:
    ComponentDependency: Represents a single dependency
    DependencyManager: Manages all component dependencies

Functions:
    validate_version: Validate version compatibility

Usage:
    manager = DependencyManager(project_root)
    manager.load_all_manifests()
    build_order = manager.get_build_order()

    for component in build_order:
        print(f"Build: {component}")
"""

from dataclasses import dataclass
from typing import Dict, List, Set, Optional, Tuple
from pathlib import Path
import yaml
from datetime import datetime


class CircularDependencyError(Exception):
    """Raised when circular dependencies are detected."""
    pass


class MissingDependencyError(Exception):
    """Raised when a required dependency is missing."""
    pass


class InvalidDependencyLevel(Exception):
    """Raised when a component depends on a higher-level component."""
    pass


@dataclass
class ComponentDependency:
    """Represents a dependency on another component."""
    name: str
    version: str
    import_from: str
    optional: bool = False
    uses: List[str] = None

    def __post_init__(self):
        if self.uses is None:
            self.uses = []


class DependencyManager:
    """
    Manage component dependencies and build order.

    Responsibilities:
    - Load component manifests
    - Build dependency graph
    - Detect circular dependencies
    - Validate dependency levels
    - Determine build order
    - Check version compatibility
    """

    # Component type levels (lower numbers = lower level)
    TYPE_LEVELS = {
        'base': 0,
        'core': 1,
        'feature': 2,
        'integration': 3,
        'application': 4
    }

    def __init__(self, project_root: Path):
        """
        Initialize DependencyManager.

        Args:
            project_root: Path to project root directory
        """
        self.project_root = Path(project_root)
        self.components: Dict[str, Dict] = {}
        self.dependencies: Dict[str, List[ComponentDependency]] = {}

    def load_all_manifests(self) -> int:
        """
        Load all component manifests from the project.

        Returns:
            Number of manifests loaded
        """
        components_dir = self.project_root / "components"

        if not components_dir.exists():
            return 0

        count = 0
        for component_dir in components_dir.iterdir():
            if component_dir.is_dir():
                manifest_path = component_dir / "component.yaml"
                if manifest_path.exists():
                    try:
                        self.load_manifest(manifest_path)
                        count += 1
                    except Exception as e:
                        print(f"Warning: Failed to load {manifest_path}: {e}")

        return count

    def load_manifest(self, manifest_path: Path) -> None:
        """
        Load a single component manifest.

        Args:
            manifest_path: Path to component.yaml file
        """
        with open(manifest_path) as f:
            manifest = yaml.safe_load(f)

        component_name = manifest['name']
        self.components[component_name] = manifest

        # Extract dependencies
        deps = []
        if 'dependencies' in manifest:
            if 'imports' in manifest['dependencies']:
                for dep_dict in manifest['dependencies']['imports']:
                    dep = ComponentDependency(
                        name=dep_dict['name'],
                        version=dep_dict.get('version', '*'),
                        import_from=dep_dict.get('import_from', f"components.{dep_dict['name']}"),
                        uses=dep_dict.get('uses', [])
                    )
                    deps.append(dep)

            if 'optional' in manifest['dependencies']:
                for dep_dict in manifest['dependencies']['optional']:
                    dep = ComponentDependency(
                        name=dep_dict['name'],
                        version=dep_dict.get('version', '*'),
                        import_from=dep_dict.get('import_from', f"components.{dep_dict['name']}"),
                        optional=True,
                        uses=dep_dict.get('uses', [])
                    )
                    deps.append(dep)

        self.dependencies[component_name] = deps

    def get_build_order(self) -> List[str]:
        """
        Get topological build order for components.

        Components with no dependencies are built first.

        Returns:
            List of component names in build order

        Raises:
            CircularDependencyError: If circular dependencies detected
        """
        # Build adjacency list (reversed - dependencies point to dependents)
        graph = {name: [] for name in self.components}
        in_degree = {name: 0 for name in self.components}

        for component, deps in self.dependencies.items():
            for dep in deps:
                if not dep.optional and dep.name in self.components:
                    graph[dep.name].append(component)
                    in_degree[component] += 1

        # Kahn's algorithm for topological sort
        queue = [name for name, degree in in_degree.items() if degree == 0]
        build_order = []

        while queue:
            # Sort for deterministic output
            queue.sort()
            current = queue.pop(0)
            build_order.append(current)

            for dependent in graph[current]:
                in_degree[dependent] -= 1
                if in_degree[dependent] == 0:
                    queue.append(dependent)

        # Check for circular dependencies
        if len(build_order) != len(self.components):
            missing = set(self.components.keys()) - set(build_order)
            raise CircularDependencyError(
                f"Circular dependency detected involving: {missing}"
            )

        return build_order

    def verify_dependencies(self, component_name: str) -> List[str]:
        """
        Verify all dependencies of a component exist.

        Args:
            component_name: Name of component to verify

        Returns:
            List of missing dependency names (empty if all exist)
        """
        missing = []

        if component_name not in self.dependencies:
            return [f"Component {component_name} not found"]

        for dep in self.dependencies[component_name]:
            if not dep.optional and dep.name not in self.components:
                missing.append(dep.name)

        return missing

    def check_circular_dependencies(self) -> List[List[str]]:
        """
        Check for circular dependencies.

        Returns:
            List of cycles (each cycle is a list of component names)
        """
        cycles = []
        visited = set()
        rec_stack = set()

        def dfs(node: str, path: List[str]) -> None:
            visited.add(node)
            rec_stack.add(node)
            path.append(node)

            if node in self.dependencies:
                for dep in self.dependencies[node]:
                    if not dep.optional and dep.name in self.components:
                        if dep.name not in visited:
                            dfs(dep.name, path.copy())
                        elif dep.name in rec_stack:
                            # Found a cycle
                            cycle_start = path.index(dep.name)
                            cycle = path[cycle_start:] + [dep.name]
                            cycles.append(cycle)

            rec_stack.remove(node)

        for component in self.components:
            if component not in visited:
                dfs(component, [])

        return cycles

    def get_component_type(self, component_name: str) -> str:
        """
        Get the type of a component.

        Args:
            component_name: Name of component

        Returns:
            Component type (base, core, feature, integration, application)
        """
        manifest = self.components.get(component_name, {})
        return manifest.get('type', 'unknown')

    def get_component_level(self, component_name: str) -> int:
        """
        Get the level of a component (0-4).

        Args:
            component_name: Name of component

        Returns:
            Level number (0=base, 4=application)
        """
        component_type = self.get_component_type(component_name)
        return self.TYPE_LEVELS.get(component_type, 999)

    def validate_dependency_levels(self) -> List[str]:
        """
        Validate that components only depend on lower or equal level components.

        Returns:
            List of violations (empty if valid)
        """
        violations = []

        for component_name, deps in self.dependencies.items():
            component_level = self.get_component_level(component_name)
            component_type = self.get_component_type(component_name)

            for dep in deps:
                if dep.optional:
                    continue

                if dep.name not in self.components:
                    continue

                dep_level = self.get_component_level(dep.name)
                dep_type = self.get_component_type(dep.name)

                # Components can only depend on same or lower level
                if dep_level > component_level:
                    violations.append(
                        f"{component_name} ({component_type}, level {component_level}) "
                        f"depends on {dep.name} ({dep_type}, level {dep_level}) - "
                        f"cannot depend on higher level component"
                    )

        return violations

    def get_dependents(self, component_name: str) -> List[str]:
        """
        Get list of components that depend on the given component.

        Args:
            component_name: Name of component

        Returns:
            List of dependent component names
        """
        dependents = []

        for comp_name, deps in self.dependencies.items():
            for dep in deps:
                if dep.name == component_name:
                    dependents.append(comp_name)
                    break

        return dependents

    def get_dependency_chain(self, component_name: str) -> List[str]:
        """
        Get full dependency chain for a component (recursive).

        Args:
            component_name: Name of component

        Returns:
            List of all transitive dependencies
        """
        visited = set()

        def collect_deps(name: str) -> None:
            if name in visited or name not in self.dependencies:
                return

            visited.add(name)
            for dep in self.dependencies[name]:
                if not dep.optional and dep.name in self.components:
                    collect_deps(dep.name)

        collect_deps(component_name)
        visited.discard(component_name)  # Remove self

        return list(visited)

    def generate_dependency_report(self) -> str:
        """
        Generate human-readable dependency report.

        Returns:
            Formatted report string
        """
        lines = []
        lines.append("=" * 60)
        lines.append("COMPONENT DEPENDENCY REPORT")
        lines.append("=" * 60)
        lines.append("")

        lines.append(f"Components: {len(self.components)}")
        lines.append("")

        # Group by type
        by_type = {}
        for name, manifest in self.components.items():
            comp_type = manifest.get('type', 'unknown')
            if comp_type not in by_type:
                by_type[comp_type] = []
            by_type[comp_type].append(name)

        lines.append("Components by Type:")
        for comp_type in ['base', 'core', 'feature', 'integration', 'application']:
            if comp_type in by_type:
                lines.append(f"  {comp_type.capitalize()}: {len(by_type[comp_type])}")
                for name in sorted(by_type[comp_type]):
                    dep_count = len([d for d in self.dependencies.get(name, []) if not d.optional])
                    lines.append(f"    - {name} ({dep_count} dependencies)")
        lines.append("")

        # Check for issues
        lines.append("Validation:")

        # Circular dependencies
        cycles = self.check_circular_dependencies()
        if cycles:
            lines.append(f"  ❌ Circular dependencies: {len(cycles)} cycles found")
            for i, cycle in enumerate(cycles, 1):
                lines.append(f"     Cycle {i}: {' → '.join(cycle)}")
        else:
            lines.append("  ✅ No circular dependencies")

        # Dependency level violations
        violations = self.validate_dependency_levels()
        if violations:
            lines.append(f"  ❌ Dependency level violations: {len(violations)}")
            for violation in violations[:5]:  # Show first 5
                lines.append(f"     {violation}")
        else:
            lines.append("  ✅ All dependencies respect level hierarchy")

        # Missing dependencies
        missing_any = False
        for component in self.components:
            missing = self.verify_dependencies(component)
            if missing:
                if not missing_any:
                    lines.append("  ❌ Missing dependencies found:")
                    missing_any = True
                lines.append(f"     {component}: {', '.join(missing)}")

        if not missing_any:
            lines.append("  ✅ All dependencies satisfied")

        lines.append("")

        # Build order
        try:
            build_order = self.get_build_order()
            lines.append("Build Order:")
            for i, component in enumerate(build_order, 1):
                comp_type = self.get_component_type(component)
                lines.append(f"  {i:2d}. {component} ({comp_type})")
        except CircularDependencyError as e:
            lines.append(f"Build Order: Cannot determine (circular dependency)")

        lines.append("")
        lines.append("=" * 60)

        return "\n".join(lines)

    def to_dict(self) -> Dict:
        """
        Convert dependency information to dictionary.

        Returns:
            Dict with all dependency information
        """
        return {
            'components': {
                name: {
                    'type': manifest.get('type'),
                    'version': manifest.get('version'),
                    'dependencies': [
                        {
                            'name': dep.name,
                            'version': dep.version,
                            'optional': dep.optional
                        }
                        for dep in self.dependencies.get(name, [])
                    ]
                }
                for name, manifest in self.components.items()
            },
            'build_order': self.get_build_order() if not self.check_circular_dependencies() else None,
            'cycles': self.check_circular_dependencies(),
            'violations': self.validate_dependency_levels()
        }


def main():
    """CLI entry point for dependency manager."""
    import sys
    import argparse

    parser = argparse.ArgumentParser(
        description="Manage component dependencies"
    )
    parser.add_argument(
        'project_root',
        type=Path,
        nargs='?',
        default=Path.cwd(),
        help='Path to project root (default: current directory)'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output as JSON'
    )

    args = parser.parse_args()

    manager = DependencyManager(args.project_root)
    count = manager.load_all_manifests()

    if count == 0:
        print("No component manifests found")
        return 1

    if args.json:
        import json
        print(json.dumps(manager.to_dict(), indent=2))
    else:
        print(manager.generate_dependency_report())

    # Exit with error if there are issues
    if manager.check_circular_dependencies() or manager.validate_dependency_levels():
        return 1

    return 0


if __name__ == '__main__':
    import sys
    sys.exit(main())
