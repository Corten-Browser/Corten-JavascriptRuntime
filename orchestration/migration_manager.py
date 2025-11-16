"""
Migration Manager

Manages migration of existing projects to the orchestrated architecture.
Handles analysis, component creation, contract generation, and orchestrator setup.

Classes:
    MigrationManager: Orchestrate complete project migrations
"""

from pathlib import Path
from typing import Dict, List, Optional
import json
import shutil
import subprocess
from datetime import datetime


class MigrationManager:
    """
    Manage migration of existing projects to orchestrated architecture.

    This class coordinates the multi-phase migration process from existing
    monolithic or unstructured codebases to the orchestrated multi-component
    architecture.

    Attributes:
        analyzer: ClaudeCodeAnalyzer instance (optional)
        migration_state_file: Path to migration state JSON file
    """

    def __init__(self, migration_state_file: Optional[Path] = None):
        """
        Initialize MigrationManager.

        Args:
            migration_state_file: Path to migration state file
                (default: orchestration/migration-state.json)
        """
        # Import here to avoid circular dependencies
        try:
            from claude_code_analyzer import ClaudeCodeAnalyzer
            self.analyzer = ClaudeCodeAnalyzer()
        except ImportError:
            self.analyzer = None

        if migration_state_file is None:
            self.migration_state_file = Path("orchestration/migration-state.json")
        else:
            self.migration_state_file = Path(migration_state_file)

    def migrate_project(self, source_path: str, target_path: Optional[str] = None) -> Dict:
        """
        Migrate an existing project to orchestrated architecture.

        Args:
            source_path: Path to source project
            target_path: Path for migrated project (default: migrated_project)

        Returns:
            Dictionary containing:
                - status: 'completed' or 'failed'
                - source: Source project path
                - target: Target project path
                - components_migrated: Number of components created
                - total_lines: Total lines of code migrated
        """

        source = Path(source_path)
        target = Path(target_path or "migrated_project")

        print(f"Starting migration of {source} to {target}")

        # Phase 1: Analysis
        print("Phase 1: Analyzing project structure...")
        if self.analyzer:
            analysis = self.analyzer.analyze_with_claude_code(
                task="migration_analysis",
                target_path=source
            )

            if not analysis.get('completed'):
                return {
                    'status': 'failed',
                    'phase': 'analysis',
                    'error': analysis.get('error')
                }
        else:
            # Simple fallback analysis
            analysis = self._simple_project_analysis(source)

        # Phase 2: Setup orchestration structure
        print("Phase 2: Setting up orchestration structure...")
        self._setup_orchestration_structure(target)

        # Phase 3: Migrate components
        print("Phase 3: Migrating components...")
        components = analysis.get('components', [])
        migrated_components = []

        for component in components:
            result = self._migrate_component(component, source, target)
            migrated_components.append(result)

        # Phase 4: Generate contracts
        print("Phase 4: Generating API contracts...")
        self._generate_contracts(migrated_components, target)

        # Phase 5: Setup orchestrator
        print("Phase 5: Configuring orchestrator...")
        self._setup_orchestrator(target, analysis, migrated_components)

        # Save migration state
        self._save_migration_state({
            'source': str(source),
            'target': str(target),
            'analysis': analysis,
            'migrated_components': migrated_components,
            'timestamp': datetime.now().isoformat()
        })

        print(f"\n✓ Migration complete!")
        print(f"  Components migrated: {len(migrated_components)}")
        print(f"  Target directory: {target}")

        return {
            'status': 'completed',
            'source': str(source),
            'target': str(target),
            'components_migrated': len(migrated_components),
            'total_lines': analysis.get('total_lines', 0)
        }

    def _simple_project_analysis(self, source: Path) -> Dict:
        """
        Perform simple project analysis without Claude Code analyzer.

        Args:
            source: Source project path

        Returns:
            Analysis dictionary
        """

        # Detect project type
        project_type = "unknown"
        if (source / "package.json").exists():
            project_type = "javascript"
        elif (source / "requirements.txt").exists() or (source / "setup.py").exists():
            project_type = "python"
        elif (source / "Cargo.toml").exists():
            project_type = "rust"

        # Count total lines
        total_lines = 0
        for file in source.rglob("*.py"):
            try:
                total_lines += len(file.read_text().splitlines())
            except:
                pass

        # Create simple component breakdown
        components = [
            {
                "name": "migrated-core",
                "lines": total_lines,
                "estimated_tokens": total_lines * 10,
                "type": "core"
            }
        ]

        return {
            "project_type": project_type,
            "total_lines": total_lines,
            "components": components,
            "completed": True
        }

    def _setup_orchestration_structure(self, target: Path):
        """
        Create orchestration directory structure.

        Args:
            target: Target project path
        """

        # Create directories
        (target / "components").mkdir(parents=True, exist_ok=True)
        (target / "contracts").mkdir(parents=True, exist_ok=True)
        (target / "shared-libs").mkdir(parents=True, exist_ok=True)
        (target / "orchestration").mkdir(parents=True, exist_ok=True)

        # Create shared-libs README
        (target / "shared-libs" / "README.md").write_text("""# Shared Libraries

Code shared across multiple components.

## Usage
- Components can READ (but not modify) shared libraries
- All shared code must be versioned
- Document all public APIs
""")

    def _migrate_component(self, component: Dict, source: Path, target: Path) -> Dict:
        """
        Migrate a single component.

        Args:
            component: Component information dictionary
            source: Source project path
            target: Target project path

        Returns:
            Migration result dictionary
        """

        comp_name = component['name']
        comp_source = Path(component.get('path', source / comp_name))
        comp_target = target / "components" / comp_name

        # Create component directory
        comp_target.mkdir(parents=True, exist_ok=True)

        # Copy component files
        if comp_source.exists() and comp_source.is_dir():
            # Copy entire directory
            try:
                shutil.copytree(comp_source, comp_target, dirs_exist_ok=True)
            except Exception as e:
                print(f"Warning: Could not copy all files from {comp_source}: {e}")
                (comp_target / "src").mkdir(exist_ok=True)
        else:
            # Create from scratch
            (comp_target / "src").mkdir(exist_ok=True)
            (comp_target / "tests").mkdir(exist_ok=True)

        # Create component CLAUDE.md
        claude_md = self._generate_migrated_component_claude_md(component)
        (comp_target / "CLAUDE.md").write_text(claude_md)

        # Create README
        (comp_target / "README.md").write_text(f"""# {comp_name}

Migrated component from original project.

## Type
{component.get('type', 'general')}

## Original Size
{component.get('lines', 'unknown')} lines

## API Contract
See ../../contracts/{comp_name}-api.yaml
""")

        # Initialize git
        try:
            subprocess.run(['git', 'init'], cwd=comp_target, capture_output=True, check=False)
            subprocess.run(['git', 'add', '.'], cwd=comp_target, capture_output=True, check=False)
            subprocess.run(
                ['git', 'commit', '-m', 'Initial migration'],
                cwd=comp_target,
                capture_output=True,
                check=False
            )
        except Exception:
            # Git initialization is optional
            pass

        return {
            'name': comp_name,
            'source': str(comp_source),
            'target': str(comp_target),
            'lines': component.get('lines', 0),
            'type': component.get('type', 'unknown')
        }

    def _generate_migrated_component_claude_md(self, component: Dict) -> str:
        """
        Generate CLAUDE.md for migrated component.

        Args:
            component: Component information dictionary

        Returns:
            CLAUDE.md content string
        """

        return f"""# {component['name'].replace('-', ' ').title()} Component

This component was migrated from an existing project.

## Component Type
{component.get('type', 'general')}

## Migration Status
- Source lines: {component.get('lines', 'unknown')}
- Estimated tokens: {component.get('estimated_tokens', 'unknown')}
- Migration date: {datetime.now().isoformat()}

## Your Boundaries
- You work ONLY in this directory
- You CANNOT access other components' source code
- You can READ contracts from ../../contracts/
- You can READ shared libraries from ../../shared-libs/

## Priority Tasks
1. Review migrated code for issues
2. Add missing tests (target 80% coverage)
3. Improve documentation
4. Refactor for better maintainability
5. Ensure API contract compliance

## Development Standards
- Follow existing code style
- Maintain backward compatibility during migration
- Document all changes
- Commit regularly to local git
"""

    def _generate_contracts(self, components: List[Dict], target: Path):
        """
        Generate API contracts for migrated components.

        Args:
            components: List of component dictionaries
            target: Target project path
        """

        contracts_dir = target / "contracts"

        for comp in components:
            # Create basic contract file
            contract = {
                'openapi': '3.0.0',
                'info': {
                    'title': f"{comp['name']} API",
                    'version': '1.0.0',
                    'description': f"Migrated API for {comp['name']}"
                },
                'paths': {}
            }

            contract_file = contracts_dir / f"{comp['name']}-api.yaml"
            contract_file.write_text(json.dumps(contract, indent=2))

    def _setup_orchestrator(self, target: Path, analysis: Dict,
                          components: List[Dict]):
        """
        Set up master orchestrator.

        Args:
            target: Target project path
            analysis: Project analysis dictionary
            components: List of component dictionaries
        """

        orchestrator_md = f"""# Master Orchestrator - Migrated Project

## Project Migration Summary
- Original type: {analysis.get('project_type', 'unknown')}
- Total lines migrated: {analysis.get('total_lines', 0):,}
- Components created: {len(components)}
- Migration date: {datetime.now().isoformat()}

## Components
"""

        for comp in components:
            orchestrator_md += f"- **{comp['name']}** ({comp['type']}): {comp['lines']:,} lines\n"

        orchestrator_md += """

## Your Role
You are the master orchestrator for this migrated project. You coordinate all work but NEVER write production code directly.

## Operating Principles
1. All new development through sub-agents
2. Maximum 3 concurrent sub-agents
3. Strict component isolation
4. Contract-based communication
5. Component size monitoring (optimal: 100-120k tokens, split at 170k tokens)

## Migration Completion Tasks
1. Validate all components are functional
2. Run integration tests across components
3. Identify and fix migration issues
4. Optimize component boundaries
5. Document system architecture

## Monitoring
- Check component sizes: `python orchestration/context_manager.py`
- View agent status: `python orchestration/agent_launcher.py status`
- Split recommendations: `python orchestration/component_splitter.py recommendations`
"""

        (target / "CLAUDE.md").write_text(orchestrator_md)

    def _save_migration_state(self, state: Dict):
        """
        Save migration state for recovery.

        Args:
            state: Migration state dictionary
        """

        self.migration_state_file.parent.mkdir(parents=True, exist_ok=True)
        self.migration_state_file.write_text(json.dumps(state, indent=2))


if __name__ == "__main__":
    # CLI interface
    import sys

    if len(sys.argv) > 1:
        source = sys.argv[1]
        target = sys.argv[2] if len(sys.argv) > 2 else None

        manager = MigrationManager()
        result = manager.migrate_project(source, target)

        print(json.dumps(result, indent=2))
    else:
        print("Usage: python migration_manager.py <source_path> [target_path]")
