"""
Claude Code Orchestration System - Core Modules

This package contains the core orchestration tools for managing
multi-agent Claude Code development environments.

Modules:
    context_manager: Monitor and manage context window usage
    component_splitter: Automatically split oversized components
    claude_code_analyzer: Use Claude Code for specialized analysis
    migration_manager: Migrate existing projects to orchestrated architecture
    template_engine: Render CLAUDE.md templates with variable substitution
    quality_verifier: Enforce quality gates on component code

Classes:
    ContextWindowManager: Calculate and analyze component token usage
    TokenTracker: Track token usage across all components
    AdaptiveComponentSplitter: Split oversized components
    AutomaticSplitMonitor: Monitor all components for splitting needs
    ClaudeCodeAnalyzer: Launch analysis sub-agents
    MigrationManager: Orchestrate project migrations
    TemplateEngine: Render templates with variable substitution
    TemplateValidator: Validate template files

Note:
    Agent launching now uses MCP (Model Context Protocol) instead of AgentLauncher.
    Configure agents in .claude.json and invoke with @mentions.
    See docs/MCP-SETUP.md for details.
"""

from .context_manager import ContextWindowManager, TokenTracker
from .component_splitter import AdaptiveComponentSplitter, AutomaticSplitMonitor
from .claude_code_analyzer import ClaudeCodeAnalyzer
from .migration_manager import MigrationManager
from .template_engine import TemplateEngine, TemplateValidator

__version__ = "2.0.0-mcp"
__author__ = "Claude Code Orchestration Project"

__all__ = [
    # Context Management
    "ContextWindowManager",
    "TokenTracker",

    # Component Splitting
    "AdaptiveComponentSplitter",
    "AutomaticSplitMonitor",

    # Analysis
    "ClaudeCodeAnalyzer",

    # Migration
    "MigrationManager",

    # Templates
    "TemplateEngine",
    "TemplateValidator",
]


# Convenience function for quick component analysis
def analyze_component(component_path):
    """
    Quick analysis of a component's token usage.

    Args:
        component_path: Path to component directory

    Returns:
        Analysis dictionary
    """
    from pathlib import Path
    manager = ContextWindowManager()
    return manager.analyze_component(Path(component_path))


# Convenience function for checking all components
def check_all_components():
    """
    Check token usage across all components.

    Returns:
        Tracking data dictionary
    """
    tracker = TokenTracker()
    return tracker.update_all_components()


# MCP-based agent management
def get_mcp_info():
    """
    Get information about MCP agent configuration.

    Returns:
        Dictionary with MCP setup information

    Note:
        Agent status is now managed through MCP. Agents are configured in
        .claude.json and invoked with @mentions. This function provides
        guidance on checking MCP configuration.
    """
    import json
    from pathlib import Path

    config_path = Path(".claude.json")
    if not config_path.exists():
        return {
            'configured': False,
            'message': 'No .claude.json found. See docs/MCP-SETUP.md to configure agents.',
            'agents': []
        }

    with open(config_path) as f:
        config = json.load(f)

    agents = list(config.get('mcpServers', {}).keys())
    return {
        'configured': True,
        'agent_count': len(agents),
        'agents': agents,
        'message': f'Found {len(agents)} configured MCP agents: {", ".join(agents)}',
        'usage': f'Invoke agents with: @{agents[0]}, <task>' if agents else 'No agents configured'
    }
