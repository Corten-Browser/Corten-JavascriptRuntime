"""
Component Splitter

Automatically splits components that exceed size limits into smaller,
manageable components. Uses Claude Code sub-agents for analysis and planning.

Classes:
    AdaptiveComponentSplitter: Split oversized components adaptively
    AutomaticSplitMonitor: Monitor all components and trigger automatic splits
"""

from pathlib import Path
from typing import Dict, List, Optional
import json
import shutil
import subprocess
from datetime import datetime


class AdaptiveComponentSplitter:
    """
    Split components based on token limits.

    This class analyzes components and automatically splits them when they
    exceed safe size limits. It uses Claude Code sub-agents to plan optimal
    split strategies.

    Attributes:
        context_manager: ContextWindowManager instance
        analyzer: ClaudeCodeAnalyzer instance (optional)
        split_history_file: Path to split history JSON file
    """

    def __init__(self, split_history_file: Optional[Path] = None):
        """
        Initialize AdaptiveComponentSplitter.

        Args:
            split_history_file: Path to split history file
                (default: orchestration/split-history.json)
        """
        # Import here to avoid circular dependencies
        from context_manager import ContextWindowManager

        self.context_manager = ContextWindowManager()

        # Claude Code analyzer is optional for now
        try:
            from claude_code_analyzer import ClaudeCodeAnalyzer
            self.analyzer = ClaudeCodeAnalyzer()
        except ImportError:
            self.analyzer = None

        if split_history_file is None:
            self.split_history_file = Path("orchestration/split-history.json")
        else:
            self.split_history_file = Path(split_history_file)

    def check_and_split(self, component_path: Path, force: bool = False) -> Dict:
        """
        Check component size and split if necessary.

        Uses new four-tier system:
        - green (optimal): < 120k tokens - No action
        - yellow (monitor): 120-150k tokens - Monitor, recommend if growing
        - orange (split_recommended): 150-170k tokens - Plan split soon
        - red (emergency): > 170k tokens - Split immediately

        Args:
            component_path: Path to component directory
            force: Force split regardless of size

        Returns:
            Dictionary containing:
                - action: 'none', 'monitor', 'split_recommended', 'split_completed', or 'failed'
                - reason: Reason for action
                - tier: Component tier (optimal, monitor, split_recommended, emergency)
                - analysis: Component analysis (if no split)
                - new_components: List of new components (if split)
        """

        # Analyze component
        analysis = self.context_manager.analyze_component(component_path)
        tier = analysis.get('tier', 'unknown')

        if force or tier == 'emergency':
            # Immediate split required
            print(f"Component {component_path.name} requires immediate split: {analysis['message']}")
            return self.execute_split(component_path, analysis)

        elif tier == 'split_recommended':
            # Split recommended but not urgent
            print(f"Component {component_path.name} should be split soon: {analysis['message']}")
            return {
                'action': 'split_recommended',
                'reason': f"Component near limits ({analysis['estimated_tokens']:,} tokens)",
                'tier': tier,
                'analysis': analysis,
                'recommendation': 'Plan split soon to avoid emergency split'
            }

        elif tier == 'monitor':
            # Monitor growth
            return {
                'action': 'monitor',
                'reason': f"Component within limits but growing ({analysis['estimated_tokens']:,} tokens)",
                'tier': tier,
                'analysis': analysis,
                'recommendation': 'Monitor growth, consider proactive splitting'
            }

        else:
            # Optimal size
            return {
                'action': 'none',
                'reason': 'Component within optimal size',
                'tier': tier,
                'analysis': analysis
            }

    def execute_split(self, component_path: Path, analysis: Dict) -> Dict:
        """
        Execute component split.

        Args:
            component_path: Path to component directory
            analysis: Component analysis from ContextWindowManager

        Returns:
            Dictionary containing split results
        """

        print(f"Planning split for {component_path.name}...")

        # Use Claude Code to plan the split if analyzer available
        if self.analyzer:
            split_plan = self.analyzer.analyze_with_claude_code(
                task="plan_split",
                target_path=component_path,
                additional_context={'current_analysis': analysis}
            )

            if not split_plan.get('completed'):
                return {
                    'action': 'failed',
                    'reason': 'Split planning failed',
                    'error': split_plan.get('error')
                }
        else:
            # Use simple default split plan
            split_plan = self._create_default_split_plan(component_path, analysis)

        # Execute the split plan
        result = self._execute_split_plan(component_path, split_plan)

        # Record in history
        self._record_split(component_path, split_plan, result)

        return result

    def _create_default_split_plan(self, component_path: Path, analysis: Dict) -> Dict:
        """
        Create a default split plan when Claude Code analyzer is not available.

        Args:
            component_path: Path to component directory
            analysis: Component analysis

        Returns:
            Simple split plan dictionary
        """

        # Simple 2-way split
        return {
            'completed': True,
            'strategy': 'horizontal',
            'new_components': [
                {
                    'name': f"{component_path.name}-core",
                    'purpose': 'Core functionality',
                    'estimated_tokens': analysis['estimated_tokens'] // 2
                },
                {
                    'name': f"{component_path.name}-utils",
                    'purpose': 'Utilities and helpers',
                    'estimated_tokens': analysis['estimated_tokens'] // 2
                }
            ],
            'shared': [],
            'steps': [
                'Create new component directories',
                'Move files to appropriate components',
                'Create API contracts',
                'Archive original component'
            ]
        }

    def _execute_split_plan(self, component_path: Path, plan: Dict) -> Dict:
        """
        Execute the split plan.

        Args:
            component_path: Path to component directory
            plan: Split plan dictionary

        Returns:
            Dictionary containing execution results
        """

        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')

        # Create backup
        backup_path = component_path.parent / f"_{component_path.name}_backup_{timestamp}"
        shutil.copytree(component_path, backup_path)

        # Create new component directories
        new_components = []
        for new_comp in plan.get('new_components', []):
            comp_name = new_comp['name']
            comp_path = component_path.parent / comp_name

            # Create component directory
            comp_path.mkdir(exist_ok=True)

            # Create CLAUDE.md for new component
            claude_md = self._generate_component_claude_md(comp_name, new_comp)
            (comp_path / "CLAUDE.md").write_text(claude_md)

            # Initialize git
            try:
                subprocess.run(['git', 'init'], cwd=comp_path, capture_output=True, check=False)
            except Exception:
                # Git initialization is optional
                pass

            new_components.append({
                'name': comp_name,
                'path': str(comp_path),
                'estimated_tokens': new_comp.get('estimated_tokens', 0)
            })

        # Move files according to plan
        self._migrate_files(component_path, new_components, plan)

        # Create contracts between new components
        self._create_contracts(new_components)

        # Archive original component
        archive_path = component_path.parent / f"_archived_{component_path.name}_{timestamp}"
        component_path.rename(archive_path)

        return {
            'action': 'split_completed',
            'original_component': component_path.name,
            'new_components': new_components,
            'backup_path': str(backup_path),
            'archive_path': str(archive_path),
            'timestamp': timestamp
        }

    def _generate_component_claude_md(self, name: str, component_info: Dict) -> str:
        """
        Generate CLAUDE.md for new split component.

        Args:
            name: Component name
            component_info: Component information dictionary

        Returns:
            CLAUDE.md content string
        """

        return f"""# {name.replace('-', ' ').title()} Component

This component was created from splitting a larger component.

## Component Responsibility
{component_info.get('purpose', 'Component responsibilities')}

## Your Boundaries
- You work ONLY in this directory: components/{name}/
- You CANNOT access other components' source code
- You can READ contracts from ../../contracts/
- You can READ shared libraries from ../../shared-libs/

## Token Budget
- Target size: ~40,000 tokens maximum
- Current estimate: {component_info.get('estimated_tokens', 0)} tokens
- Monitor growth to avoid future splits

## Development Guidelines
1. Maintain clean, modular code
2. Write comprehensive tests
3. Document all public APIs
4. Follow established patterns
5. Alert orchestrator if approaching token limits

## Integration Points
- Read your API contract from: ../../contracts/{name}-api.yaml
- Implement all defined endpoints
- Use shared libraries for common functionality
"""

    def _migrate_files(self, old_path: Path, new_components: List[Dict], plan: Dict):
        """
        Migrate files to new components.

        This is a simplified version. In practice, a Claude Code sub-agent
        would handle the complex file migration based on the split plan.

        Args:
            old_path: Original component path
            new_components: List of new component dictionaries
            plan: Split plan dictionary
        """

        print(f"Migrating files from {old_path.name} to new components...")

        # For now, just create placeholder structure
        for comp in new_components:
            comp_path = Path(comp['path'])
            (comp_path / 'src').mkdir(exist_ok=True)
            (comp_path / 'tests').mkdir(exist_ok=True)

            # Create README
            readme = f"""# {comp['name']}

Component split from {old_path.name}

## Setup
```bash
# Initialize development environment
```

## API
See ../../contracts/{comp['name']}-api.yaml
"""
            (comp_path / 'README.md').write_text(readme)

    def _create_contracts(self, components: List[Dict]):
        """
        Create API contracts between split components.

        Args:
            components: List of component dictionaries
        """

        contracts_dir = Path("contracts")
        contracts_dir.mkdir(exist_ok=True)

        for comp in components:
            contract = {
                'openapi': '3.0.0',
                'info': {
                    'title': f"{comp['name']} API",
                    'version': '1.0.0',
                    'description': f"API contract for {comp['name']} component"
                },
                'paths': {}
            }

            contract_file = contracts_dir / f"{comp['name']}-api.yaml"
            # Using JSON for now; would use yaml.dump in production
            contract_file.write_text(json.dumps(contract, indent=2))

    def _record_split(self, original_path: Path, plan: Dict, result: Dict):
        """
        Record split in history.

        Args:
            original_path: Original component path
            plan: Split plan dictionary
            result: Split result dictionary
        """

        history = []
        if self.split_history_file.exists():
            history = json.loads(self.split_history_file.read_text())

        history.append({
            'timestamp': datetime.now().isoformat(),
            'original_component': original_path.name,
            'split_plan': plan,
            'result': result
        })

        self.split_history_file.parent.mkdir(parents=True, exist_ok=True)
        self.split_history_file.write_text(json.dumps(history, indent=2))


class AutomaticSplitMonitor:
    """
    Monitor all components and trigger automatic splits.

    This class periodically checks all components in the project and
    automatically splits those that exceed size limits.

    Attributes:
        splitter: AdaptiveComponentSplitter instance
        context_manager: ContextWindowManager instance
    """

    def __init__(self):
        """Initialize AutomaticSplitMonitor."""
        from context_manager import ContextWindowManager

        self.splitter = AdaptiveComponentSplitter()
        self.context_manager = ContextWindowManager()

    def monitor_all_components(self, components_dir: Optional[Path] = None) -> List[Dict]:
        """
        Check all components and split as needed.

        Args:
            components_dir: Directory containing components (default: components/)

        Returns:
            List of monitoring results
        """

        if components_dir is None:
            components_dir = Path("components")

        results = []

        if not components_dir.exists():
            return results

        for component_path in components_dir.iterdir():
            if component_path.is_dir() and not component_path.name.startswith('_'):
                # Check component
                analysis = self.context_manager.analyze_component(component_path)

                if analysis['status'] in ['orange', 'red']:
                    # Needs splitting
                    print(f"Auto-splitting {component_path.name}...")
                    result = self.splitter.execute_split(component_path, analysis)
                    results.append(result)
                else:
                    results.append({
                        'component': component_path.name,
                        'action': 'monitored',
                        'status': analysis['status'],
                        'tokens': analysis['estimated_tokens']
                    })

        return results

    def get_split_recommendations(self, components_dir: Optional[Path] = None) -> List[Dict]:
        """
        Get recommendations for components approaching limits.

        Args:
            components_dir: Directory containing components (default: components/)

        Returns:
            List of recommendation dictionaries
        """

        if components_dir is None:
            components_dir = Path("components")

        recommendations = []

        if not components_dir.exists():
            return recommendations

        for component_path in components_dir.iterdir():
            if component_path.is_dir() and not component_path.name.startswith('_'):
                analysis = self.context_manager.analyze_component(component_path)

                if analysis['status'] == 'yellow':
                    recommendations.append({
                        'component': component_path.name,
                        'current_tokens': analysis['estimated_tokens'],
                        'percentage_used': analysis['percentage_used'],
                        'recommendation': 'Plan split strategy',
                        'urgency': 'medium'
                    })
                elif analysis['status'] in ['orange', 'red']:
                    recommendations.append({
                        'component': component_path.name,
                        'current_tokens': analysis['estimated_tokens'],
                        'percentage_used': analysis['percentage_used'],
                        'recommendation': 'Split immediately',
                        'urgency': 'high'
                    })

        return recommendations


if __name__ == "__main__":
    # CLI interface
    import sys

    if len(sys.argv) > 1:
        command = sys.argv[1]

        if command == "monitor":
            monitor = AutomaticSplitMonitor()
            results = monitor.monitor_all_components()
            print(f"\nMonitored {len(results)} components")
            for result in results:
                if result['action'] == 'split_completed':
                    print(f"  ✓ Split {result['original_component']} into {len(result['new_components'])} components")
                elif result['action'] == 'monitored':
                    print(f"  - {result['component']}: {result['status']}")

        elif command == "recommendations":
            monitor = AutomaticSplitMonitor()
            recs = monitor.get_split_recommendations()
            print(f"\nSplit Recommendations ({len(recs)} components):")
            for rec in recs:
                print(f"  {rec['urgency'].upper()}: {rec['component']} ({rec['current_tokens']:,} tokens)")
                print(f"    → {rec['recommendation']}")

        else:
            print(f"Unknown command: {command}")
            print("Available commands: monitor, recommendations")
    else:
        print("Usage: python component_splitter.py [monitor|recommendations]")
