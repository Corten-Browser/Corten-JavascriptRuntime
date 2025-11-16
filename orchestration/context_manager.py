"""
Context Window Manager

Monitors and manages context window usage across all components in the
orchestration system. Ensures no component exceeds Claude's context window
limits and provides automatic split recommendations.

Uses a two-tier limit system:
- Soft limits based on best practices and human factors
- Hard limits based on technical constraints

Classes:
    ContextWindowManager: Calculate and analyze component token usage
    TokenTracker: Track token usage across all components
"""

from pathlib import Path
from typing import Dict, List, Optional
import json
from datetime import datetime


class ContextWindowManager:
    """
    Ensure components never exceed Claude's context window.

    This class calculates safe component size limits based on Claude's
    200,000 token context window and analyzes existing components to
    determine if they are within safe limits.

    Uses a two-tier limit system:
    - Soft limits (best practices): Based on human factors and Claude performance
    - Hard limits (technical): Based on context window constraints

    Attributes:
        MAX_CONTEXT_TOKENS: Claude's maximum context window size (200k)
        OVERHEAD_TOKENS: Token overhead breakdown
        SOFT_LIMITS: Best practice limits for optimal development
        HARD_LIMITS: Technical constraint limits
    """

    # Claude's context window limit
    MAX_CONTEXT_TOKENS = 200_000

    # Token overhead breakdown (reduced from 32k to 21k)
    OVERHEAD_TOKENS = {
        'claude_md': 1_500,          # CLAUDE.md instructions
        'work_instructions': 500,     # Task/work instructions
        'system_prompts': 2_000,      # System and role prompts
        'contracts': 2_000,           # Relevant API contracts
        'response_buffer': 15_000,    # Response generation buffer
        'total': 21_000               # Total overhead
    }

    # Soft limits (best practices - human factors and Claude performance)
    SOFT_LIMITS = {
        'optimal_min': 0,
        'optimal_max': 120_000,       # 60% of available (sweet spot)
        'optimal_lines': 10_000,      # Human cognitive limit for review
        'warning': 150_000,           # 75% of available (monitor growth)
        'warning_lines': 12_000,
    }

    # Hard limits (technical constraints - context window based)
    HARD_LIMITS = {
        'split_trigger': 170_000,     # 85% of available (plan split immediately)
        'split_trigger_lines': 17_000,
        'emergency': 180_000,         # 90% of context window (force split)
        'emergency_lines': 18_000,
        'absolute_max': 200_000       # Never exceed (context window size)
    }

    def calculate_component_limits(self, max_context_tokens: int = 200_000) -> Dict:
        """
        Calculate component limits for given context window size.

        This method supports future-proofing for larger context windows (1M, 2M+ tokens).

        Args:
            max_context_tokens: Maximum context window size (default: 200,000)

        Returns:
            Dictionary containing:
                - optimal_tokens: Optimal component size (soft limit)
                - optimal_lines: Optimal lines of code
                - warning_tokens: Warning threshold
                - warning_lines: Warning line count
                - split_trigger_tokens: Immediate split planning required (hard limit)
                - split_trigger_lines: Split trigger line count
                - emergency_tokens: Emergency split required
                - emergency_lines: Emergency line count
                - max_context_tokens: Total context window
                - overhead_tokens: Total overhead
                - available_tokens: Available for source code
        """

        # Calculate overhead (scales slightly for larger contexts)
        base_overhead = self.OVERHEAD_TOKENS['total']
        scaling_overhead = int(max_context_tokens * 0.01) if max_context_tokens > 200_000 else 0
        total_overhead = base_overhead + scaling_overhead

        # Available tokens for source code
        available_tokens = max_context_tokens - total_overhead

        # Calculate limits as percentages of available tokens
        optimal_tokens = int(available_tokens * 0.6)      # 60% - plenty of headroom
        warning_tokens = int(available_tokens * 0.75)     # 75% - monitor growth
        split_trigger_tokens = int(available_tokens * 0.85)  # 85% - plan split
        emergency_tokens = int(available_tokens * 0.9)    # 90% - force split

        # Line count recommendations (may stay constant for human factors)
        optimal_lines = 10_000  # Human cognitive limit
        maximum_lines = min(int(split_trigger_tokens / 10), 25_000)  # Cap at 25k for review

        return {
            # Soft limits (best practices)
            'optimal_tokens': optimal_tokens,
            'optimal_lines': optimal_lines,
            'warning_tokens': warning_tokens,
            'warning_lines': 12_000,

            # Hard limits (technical constraints)
            'split_trigger_tokens': split_trigger_tokens,
            'split_trigger_lines': maximum_lines,
            'emergency_tokens': emergency_tokens,
            'emergency_lines': int(emergency_tokens / 10),

            # Metadata
            'max_context_tokens': max_context_tokens,
            'overhead_tokens': total_overhead,
            'available_tokens': available_tokens,
        }

    def analyze_component(self, component_path: Path) -> Dict:
        """
        Analyze if a component fits within context limits.

        Uses four-tier status system:
        - green (optimal): < 120k tokens
        - yellow (monitor): 120-150k tokens
        - orange (split recommended): 150-170k tokens
        - red (emergency): > 170k tokens

        Args:
            component_path: Path to component directory

        Returns:
            Dictionary containing:
                - component_name: Name of the component
                - line_count: Total lines of code
                - estimated_tokens: Estimated total tokens
                - percentage_used: Percentage of context window used
                - status: 'green', 'yellow', 'orange', or 'red'
                - tier: 'optimal', 'monitor', 'split_recommended', or 'emergency'
                - message: Human-readable status message
                - recommended_action: Recommended action to take
                - limits: Component limits (from calculate_component_limits)
                - file_details: List of file details
                - timestamp: ISO timestamp of analysis
        """

        total_tokens = 0
        line_count = 0
        file_details = []

        # Count source code
        for file in component_path.glob("**/*"):
            if file.is_file() and file.suffix in ['.py', '.rs', '.js', '.ts', '.tsx', '.jsx', '.java', '.cpp', '.c', '.h', '.go']:
                try:
                    content = file.read_text()
                    lines = len(content.splitlines())
                    line_count += lines
                    # Rough estimate: ~10 tokens per line
                    file_tokens = lines * 10
                    total_tokens += file_tokens

                    file_details.append({
                        'path': str(file.relative_to(component_path)),
                        'lines': lines,
                        'estimated_tokens': file_tokens
                    })
                except Exception:
                    # Skip files that can't be read
                    pass

        # Add overhead (new reduced overhead: 21k)
        total_tokens += self.OVERHEAD_TOKENS['total']

        # Get limits
        limits = self.calculate_component_limits()

        # Four-tier status system
        if total_tokens < limits['optimal_tokens']:
            status = 'green'
            tier = 'optimal'
            message = '🟢 Optimal size - well within limits'
            action = 'none'
        elif total_tokens < limits['warning_tokens']:
            status = 'yellow'
            tier = 'monitor'
            message = '🟡 Monitor growth - approaching recommended size'
            action = 'consider_split'
        elif total_tokens < limits['split_trigger_tokens']:
            status = 'orange'
            tier = 'split_recommended'
            message = '🟠 Split recommended - near hard limits'
            action = 'plan_split'
        else:
            status = 'red'
            tier = 'emergency'
            message = '🔴 IMMEDIATE SPLIT REQUIRED - exceeds safe limits'
            action = 'split_now'

        return {
            'component_name': component_path.name,
            'line_count': line_count,
            'estimated_tokens': total_tokens,
            'percentage_used': (total_tokens / self.MAX_CONTEXT_TOKENS) * 100,
            'status': status,
            'tier': tier,
            'message': message,
            'recommended_action': self._get_split_recommendation(line_count, total_tokens),
            'limits': limits,
            'file_details': file_details,
            'timestamp': datetime.now().isoformat()
        }

    def _get_split_recommendation(self, lines: int, tokens: int) -> str:
        """
        Recommend splitting strategy based on size.

        Uses new token limits:
        - < 120k: No action
        - 120-150k: Monitor
        - 150-170k: Plan split
        - > 170k: Split immediately

        Args:
            lines: Total lines of code
            tokens: Estimated total tokens

        Returns:
            Recommendation string
        """

        limits = self.calculate_component_limits()

        if tokens < limits['optimal_tokens']:
            return "No action needed - optimal size"
        elif tokens < limits['warning_tokens']:
            return "Monitor growth, prepare split strategy if growth continues"
        elif tokens < limits['split_trigger_tokens']:
            return "Plan split soon - recommend 2-3 components based on natural boundaries"
        elif tokens < limits['emergency_tokens']:
            return "SPLIT IMMEDIATELY - component exceeds safe limits"
        else:
            return "URGENT: EMERGENCY SPLIT REQUIRED - split into 3+ components immediately"

    def save_analysis(self, component_path: Path, output_dir: Optional[Path] = None) -> Path:
        """
        Save analysis to JSON file.

        Args:
            component_path: Path to component directory
            output_dir: Output directory for analysis file (default: orchestration/token-tracking)

        Returns:
            Path to saved analysis file
        """

        analysis = self.analyze_component(component_path)

        if output_dir is None:
            output_dir = Path("orchestration/token-tracking")

        output_dir.mkdir(parents=True, exist_ok=True)

        output_file = output_dir / f"{component_path.name}_analysis.json"
        with open(output_file, 'w') as f:
            json.dump(analysis, f, indent=2)

        return output_file

    def validate_concurrent_agents(self, requested_count: int, config_path: Optional[Path] = None) -> Dict:
        """
        Validate requested concurrent agent count against configured limits.

        Provides warnings and recommendations based on configured thresholds:
        - max_parallel_agents: Default maximum (typically 5)
        - warn_above: Performance warning threshold (typically 7)
        - recommended_max: Performance sweet spot (typically 10)
        - absolute_max: Hard cap (typically 15)

        Args:
            requested_count: Number of agents requested to run concurrently
            config_path: Path to orchestration-config.json (default: orchestration/orchestration-config.json)

        Returns:
            Dictionary containing:
                - valid: Boolean indicating if request is valid
                - level: 'ok', 'info', 'warning', or 'error'
                - message: Human-readable status message
                - recommendation: Recommended action
                - limits: Dictionary of configured limits
                - requested: Requested agent count
        """

        # Load configuration
        if config_path is None:
            config_path = Path("orchestration/orchestration-config.json")

        try:
            with open(config_path, 'r') as f:
                config = json.load(f)

            orchestration = config.get('orchestration', {})
            max_parallel = orchestration.get('max_parallel_agents', 5)
            warn_above = orchestration.get('warn_above', 7)
            recommended_max = orchestration.get('recommended_max', 10)
            absolute_max = orchestration.get('absolute_max', 15)
        except Exception as e:
            # Fallback to defaults if config can't be read
            max_parallel = 5
            warn_above = 7
            recommended_max = 10
            absolute_max = 15

        limits = {
            'max_parallel_agents': max_parallel,
            'warn_above': warn_above,
            'recommended_max': recommended_max,
            'absolute_max': absolute_max
        }

        # Validate agent count
        if requested_count > absolute_max:
            return {
                'valid': False,
                'level': 'error',
                'message': f'❌ Cannot launch {requested_count} agents (absolute maximum: {absolute_max})',
                'recommendation': (
                    f'Reduce to {absolute_max} agents or queue remaining work. '
                    f'Beyond {absolute_max} agents, the orchestrator experiences:\n'
                    f'  • Cognitive overload (loses track of agent progress)\n'
                    f'  • Git retry storms (5-10 minute delays)\n'
                    f'  • Integration conflicts become undebuggable\n'
                    f'  • Coordination overhead negates parallelism benefits'
                ),
                'limits': limits,
                'requested': requested_count
            }
        elif requested_count > recommended_max:
            return {
                'valid': True,
                'level': 'warning',
                'message': f'⚠️  Launching {requested_count} agents (above recommended maximum: {recommended_max})',
                'recommendation': (
                    f'Performance may degrade above {recommended_max} agents. Consider:\n'
                    f'  • Ensure components are extremely well-isolated\n'
                    f'  • Expect increased git conflicts requiring manual resolution\n'
                    f'  • Debugging failures becomes significantly harder\n'
                    f'  • Monitor orchestrator cognitive load closely'
                ),
                'limits': limits,
                'requested': requested_count
            }
        elif requested_count > warn_above:
            return {
                'valid': True,
                'level': 'info',
                'message': f'ℹ️  Launching {requested_count} agents (within acceptable range)',
                'recommendation': (
                    f'Using {requested_count} agents (above typical {max_parallel} but acceptable).\n'
                    f'  • Ensure components have clear boundaries\n'
                    f'  • Expect occasional git conflicts (retry wrapper will handle)\n'
                    f'  • Progress tracking becomes more complex'
                ),
                'limits': limits,
                'requested': requested_count
            }
        elif requested_count > max_parallel:
            return {
                'valid': True,
                'level': 'info',
                'message': f'ℹ️  Launching {requested_count} agents (above default {max_parallel})',
                'recommendation': (
                    f'Using {requested_count} agents is safe but above default {max_parallel}. '
                    f'Good for well-isolated components.'
                ),
                'limits': limits,
                'requested': requested_count
            }
        else:
            return {
                'valid': True,
                'level': 'ok',
                'message': f'✅ Launching {requested_count} agents (within default maximum)',
                'recommendation': f'Agent count is optimal for most projects.',
                'limits': limits,
                'requested': requested_count
            }

    def get_agent_limit_recommendation(self, project_component_count: int, config_path: Optional[Path] = None) -> str:
        """
        Get recommendation for max_parallel_agents based on project size.

        Args:
            project_component_count: Total number of components in project
            config_path: Path to orchestration-config.json

        Returns:
            Recommendation string
        """

        if project_component_count <= 3:
            return (
                "Recommended: max_parallel_agents = 3\n"
                "Small project (≤3 components) - default is sufficient"
            )
        elif project_component_count <= 10:
            return (
                "Recommended: max_parallel_agents = 5\n"
                "Medium project (4-10 components) - moderate parallelism"
            )
        elif project_component_count <= 30:
            return (
                "Recommended: max_parallel_agents = 7-10\n"
                "Large project (11-30 components) - high parallelism with good isolation"
            )
        else:
            return (
                "Recommended: max_parallel_agents = 10-15\n"
                "Very large project (30+ components) - maximum parallelism\n"
                "Ensure excellent component isolation and contract discipline"
            )


class TokenTracker:
    """
    Track token usage across all components.

    This class maintains a persistent tracker file that monitors all
    components in the project and identifies components that are
    approaching or exceeding size limits.

    Attributes:
        tracker_file: Path to token tracker JSON file
        context_manager: ContextWindowManager instance
    """

    def __init__(self, tracker_file: Optional[Path] = None):
        """
        Initialize TokenTracker.

        Args:
            tracker_file: Path to tracker file (default: orchestration/token-tracker.json)
        """
        if tracker_file is None:
            self.tracker_file = Path("orchestration/token-tracker.json")
        else:
            self.tracker_file = Path(tracker_file)
        self.context_manager = ContextWindowManager()

    def update_all_components(self, components_dir: Optional[Path] = None) -> Dict:
        """
        Update token tracking for all components.

        Args:
            components_dir: Directory containing components (default: components/)

        Returns:
            Dictionary containing tracking data
        """

        if components_dir is None:
            components_dir = Path("components")

        tracking_data = {
            'timestamp': datetime.now().isoformat(),
            'components': {},
            'summary': {
                'total_components': 0,
                'components_near_limit': 0,
                'components_over_limit': 0
            }
        }

        if not components_dir.exists():
            # No components directory yet
            self.tracker_file.parent.mkdir(parents=True, exist_ok=True)
            with open(self.tracker_file, 'w') as f:
                json.dump(tracking_data, f, indent=2)
            return tracking_data

        for component_path in components_dir.iterdir():
            if component_path.is_dir() and not component_path.name.startswith('_'):
                analysis = self.context_manager.analyze_component(component_path)

                tracking_data['components'][component_path.name] = {
                    'tokens': analysis['estimated_tokens'],
                    'lines': analysis['line_count'],
                    'status': analysis['status'],
                    'tier': analysis['tier'],
                    'percentage': analysis['percentage_used']
                }

                tracking_data['summary']['total_components'] += 1

                if analysis['status'] in ['orange', 'red']:
                    tracking_data['summary']['components_near_limit'] += 1

                if analysis['status'] == 'red':
                    tracking_data['summary']['components_over_limit'] += 1

        # Save tracking data
        self.tracker_file.parent.mkdir(parents=True, exist_ok=True)
        with open(self.tracker_file, 'w') as f:
            json.dump(tracking_data, f, indent=2)

        return tracking_data

    def get_critical_components(self) -> List[Dict]:
        """
        Get list of components that need immediate attention.

        Returns:
            List of dictionaries containing critical component information
        """

        if not self.tracker_file.exists():
            self.update_all_components()

        with open(self.tracker_file, 'r') as f:
            data = json.load(f)

        critical = []
        for name, info in data['components'].items():
            if info['status'] in ['orange', 'red']:
                critical.append({
                    'name': name,
                    'status': info['status'],
                    'tier': info.get('tier', 'unknown'),
                    'tokens': info['tokens'],
                    'action_required': 'SPLIT IMMEDIATELY' if info['status'] == 'red' else 'Plan split soon'
                })

        return critical


if __name__ == "__main__":
    # CLI interface for quick checks
    import sys

    if len(sys.argv) > 1:
        component_path = Path(sys.argv[1])
        manager = ContextWindowManager()
        analysis = manager.analyze_component(component_path)

        print(f"\nComponent: {analysis['component_name']}")
        print(f"Lines: {analysis['line_count']:,}")
        print(f"Estimated tokens: {analysis['estimated_tokens']:,}")
        print(f"Status: {analysis['status'].upper()} ({analysis['tier']})")
        print(f"Message: {analysis['message']}")
        print(f"Recommendation: {analysis['recommended_action']}")
        print(f"\nLimits:")
        print(f"  Optimal: {analysis['limits']['optimal_tokens']:,} tokens ({analysis['limits']['optimal_lines']:,} lines)")
        print(f"  Warning: {analysis['limits']['warning_tokens']:,} tokens ({analysis['limits']['warning_lines']:,} lines)")
        print(f"  Split Trigger: {analysis['limits']['split_trigger_tokens']:,} tokens ({analysis['limits']['split_trigger_lines']:,} lines)")
        print(f"  Emergency: {analysis['limits']['emergency_tokens']:,} tokens\n")
    else:
        tracker = TokenTracker()
        data = tracker.update_all_components()

        print(f"\nToken Tracker Summary")
        print(f"Total components: {data['summary']['total_components']}")
        print(f"Components near limit: {data['summary']['components_near_limit']}")
        print(f"Components over limit: {data['summary']['components_over_limit']}\n")

        if data['components']:
            print("Component Status:")
            for name, info in data['components'].items():
                status_symbol = {
                    'green': '✓',
                    'yellow': '⚠',
                    'orange': '⚠⚠',
                    'red': '✗'
                }.get(info['status'], '?')

                print(f"  {status_symbol} {name}: {info['tokens']:,} tokens ({info['status']})")
