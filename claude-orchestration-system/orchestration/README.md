# Orchestration Tools

This directory contains the Python management tools for the Claude Code orchestration system.

## Core Modules

### context_manager.py
**Classes**: `ContextWindowManager`, `TokenTracker`

Monitors and manages context window usage across all components:
- Calculate token counts for components
- Track usage against limits
- Trigger alerts when thresholds approached
- Provide split recommendations

**Key Methods**:
- `calculate_component_limits(task_type)` - Calculate safe component size limits
- `analyze_component(component_path)` - Analyze if a component fits within limits
- `get_critical_components()` - Get list of components needing immediate attention

### agent_launcher.py
**Class**: `AgentLauncher`

Launch and manage Claude Code sub-agents:
- Enforce configurable concurrency limit (default: 3) for token budget management
- Queue management with priority
- Monitor agent activity
- Terminate idle agents

**Key Methods**:
- `launch_agent(component_name, task, priority)` - Launch a sub-agent
- `check_agent_status(component_name)` - Check status of an agent
- `terminate_agent(component_name)` - Terminate a running agent
- `get_status_summary()` - Get summary of all agents

### component_splitter.py
**Classes**: `AdaptiveComponentSplitter`, `AutomaticSplitMonitor`

Automatically detect and split oversized components:
- Identify natural split boundaries
- Create split plans using Claude Code analysis
- Execute splits with minimal disruption
- Update contracts and dependencies

**Key Methods**:
- `check_and_split(component_path)` - Check and split if necessary
- `execute_split(component_path, analysis)` - Execute component split
- `get_split_recommendations()` - Get recommendations for components approaching limits

### claude_code_analyzer.py
**Class**: `ClaudeCodeAnalyzer`

Use Claude Code sub-agents for analysis tasks:
- Component identification
- Split planning
- Migration analysis
- Dependency mapping

**Key Methods**:
- `analyze_with_claude_code(task, target_path, additional_context)` - Launch analysis sub-agent

### migration_manager.py
**Class**: `MigrationManager`

Migrate existing projects to orchestrated architecture:
- Multi-phase migration process
- Preserve git history
- Generate contracts
- Create component CLAUDE.md files

**Key Methods**:
- `migrate_project(source_path, target_path)` - Execute full migration

## Data Files

- **agent-registry.json** - Tracks active, queued, and completed agents
- **token-tracker.json** - Tracks token usage across components
- **split-history.json** - Records all component splits
- **migration-state.json** - Tracks migration progress

## Usage

These tools are automatically used by the master orchestrator. For manual usage:

```python
from orchestration.context_manager import ContextWindowManager

manager = ContextWindowManager()
analysis = manager.analyze_component(Path("components/my-component"))
print(analysis)
```

## Dependencies

- Python 3.8+
- Standard library only (no external dependencies for core functionality)
