"""
Claude Code Analyzer

Generates analysis requests for the MCP code-analyzer agent.
Replaces the previous subprocess-based approach with MCP guidance pattern.

Classes:
    ClaudeCodeAnalyzer: Generate MCP analysis requests for specialized analysis tasks

Note:
    This module has been migrated from subprocess-based analysis to MCP.
    Instead of creating temporary workspaces and launching subprocess,
    it now generates MCP invocation guidance for the master Claude instance.
"""

import json
from pathlib import Path
from typing import Dict, Optional


class ClaudeCodeAnalyzer:
    """
    Generate MCP analysis requests for code analysis tasks.

    This class creates structured analysis requests that can be invoked via MCP.
    It replaces the previous subprocess-based approach with the simpler MCP pattern.

    Tasks supported:
        - identify_components: Analyze codebase to identify component boundaries
        - plan_split: Plan how to split an oversized component
        - migration_analysis: Analyze project for migration to orchestrated architecture
        - dependency_mapping: Map dependencies between components

    Usage:
        analyzer = ClaudeCodeAnalyzer()
        guidance = analyzer.analyze_with_claude_code("plan_split", component_path)

        # Then invoke via MCP:
        # @code-analyzer, [use guidance['mcp_invocation']]
    """

    def __init__(self):
        """
        Initialize ClaudeCodeAnalyzer.

        Note: No workspace directory needed anymore - analysis happens via MCP.
        """
        pass

    def analyze_with_claude_code(self, task: str, target_path: Path,
                                 additional_context: Optional[Dict] = None) -> Dict:
        """
        Generate MCP analysis request (replaces subprocess execution).

        This method now returns guidance for MCP invocation rather than
        executing analysis directly. The master Claude instance should use
        the returned mcp_invocation string to request analysis from the
        @code-analyzer MCP agent.

        Args:
            task: Analysis task type (identify_components, plan_split, etc.)
            target_path: Path to analyze
            additional_context: Additional context for the analysis

        Returns:
            Dictionary containing:
                - status: 'mcp_guidance' (indicates MCP invocation needed)
                - task: The analysis task type
                - target: Target path as string
                - mcp_invocation: Complete MCP invocation string
                - instructions: Human-readable instructions
                - task_description: Detailed task description
        """

        # Generate detailed task description
        task_description = self._get_task_description(task)

        # Build comprehensive analysis request
        analysis_request = f"""Analyze {target_path} for: {task}

Task Description:
{task_description}

Target Path: {target_path}
"""

        # Add context if provided
        if additional_context:
            analysis_request += f"\nAdditional Context:\n{json.dumps(additional_context, indent=2)}\n"

        # Add output format guidance
        analysis_request += """
Output Requirements:
- Provide structured, actionable analysis
- Identify specific components, files, or patterns
- Include size estimates where applicable
- Suggest concrete next steps
"""

        return {
            'status': 'mcp_guidance',
            'task': task,
            'target': str(target_path),
            'task_description': task_description,
            'mcp_invocation': f"@code-analyzer, {analysis_request}",
            'instructions': (
                f"Use MCP to invoke code analysis:\n\n"
                f"  @code-analyzer, {analysis_request}\n\n"
                f"The code-analyzer agent is configured in .claude.json and will\n"
                f"analyze {target_path} to provide structured results for {task}."
            )
        }

    def _get_task_description(self, task: str) -> str:
        """
        Get detailed task description.

        Args:
            task: Task type

        Returns:
            Task description string
        """

        descriptions = {
            "identify_components": """
Analyze the codebase to identify natural component boundaries.

Look for:
- Logical separation of concerns
- API boundaries and interfaces
- Data model groupings
- Feature modules and domains
- Shared utilities and libraries

Provide:
- List of identified components with names
- Size estimates (lines, files, estimated tokens)
- Responsibility description for each component
- Dependencies between components
- Recommendations for component structure""",

            "plan_split": """
Plan how to split an oversized component into smaller components.

Identify:
- Natural split points in the code
- Dependencies that need refactoring
- Shared code to extract into libraries
- New component names and responsibilities
- Files/modules for each new component

Provide:
- Splitting strategy (horizontal/vertical/hybrid)
- List of new components with responsibilities
- File movement plan
- Dependency updates needed
- Estimated complexity and risks""",

            "migration_analysis": """
Analyze existing project for migration to orchestrated architecture.

Identify:
- Current project structure and patterns
- Technology stack and frameworks
- Component candidates and boundaries
- Migration challenges and blockers
- Effort estimates and phases

Provide:
- Project overview (type, size, complexity)
- Suggested component breakdown
- Migration strategy and phases
- Estimated effort and timeline
- Potential risks and mitigation strategies""",

            "dependency_mapping": """
Map dependencies between components or modules.

Identify:
- Import relationships and patterns
- API calls between modules
- Shared data structures
- Circular dependencies
- External dependencies

Provide:
- Dependency graph or map
- Circular dependency issues
- Coupling analysis
- Recommendations for decoupling
- Shared code extraction opportunities"""
        }

        return descriptions.get(task, f"Analyze the codebase for: {task}")


if __name__ == "__main__":
    # CLI interface
    import sys

    if len(sys.argv) > 2:
        task = sys.argv[1]
        target = Path(sys.argv[2])

        analyzer = ClaudeCodeAnalyzer()
        guidance = analyzer.analyze_with_claude_code(task, target)

        print("=" * 60)
        print("ANALYSIS GUIDANCE (MCP)")
        print("=" * 60)
        print(f"\nTask: {guidance['task']}")
        print(f"Target: {guidance['target']}")
        print(f"\nStatus: {guidance['status']}")
        print("\nInvoke via MCP:")
        print("-" * 60)
        print(guidance['mcp_invocation'])
        print("-" * 60)
        print("\nOr use this command:")
        print(f"  @code-analyzer, analyze {target} for {task}")
        print("\n" + guidance['task_description'])
    else:
        print("Usage: python claude_code_analyzer.py <task> <target_path>")
        print("\nAvailable tasks:")
        print("  - identify_components: Find component boundaries")
        print("  - plan_split: Plan component splitting")
        print("  - migration_analysis: Analyze for migration")
        print("  - dependency_mapping: Map dependencies")
        print("\nExample:")
        print("  python claude_code_analyzer.py plan_split components/backend")
        print("\nNote: This generates MCP invocation guidance.")
        print("      Use @code-analyzer from master Claude instance to execute.")
