# Orchestration Context System

This directory contains shared context files referenced by all components and orchestrators.

## Purpose

Instead of generating 2,000+ line CLAUDE.md files per component with redundant content, we now use:
1. **Shared rules** (this directory) - System-maintained, upgradeable
2. **component.yaml** - Component metadata, structured data
3. **CLAUDE.md** - User-owned component specifications

## Files

| File | Purpose | Used By |
|------|---------|---------|
| `component-rules.md` | Generic TDD/BDD/quality rules | All component agents |
| `orchestration-rules.md` | Orchestrator coordination rules | /orchestrate command |
| `README.md` | Documentation | Developers |

## Usage

### For Component Agents
Read context via Task prompt:
```python
prompt="""Read the following context files:
1. orchestration/context/component-rules.md (generic rules)
2. components/[name]/component.yaml (metadata)
3. components/[name]/CLAUDE.md (specifications)
..."""
```

### For Orchestrator
Read orchestration rules at startup:
```
Read orchestration/context/orchestration-rules.md for coordination patterns.
```

## Upgrades

These files are **system-owned** and updated via `upgrade.sh`.
- User customizations should go in component CLAUDE.md files
- component.yaml stores component-specific metadata
- These shared files can be safely replaced during upgrades

## Migration

For existing projects with old-style CLAUDE.md files:
1. Run migration script: `python orchestration/migration/migrate_to_context_system.py`
2. Old CLAUDE.md files continue working (backwards compatible)
3. New components automatically use new structure

## Benefits

1. **98% size reduction** - From 2,471 to ~50 lines per component
2. **Clean upgrades** - System rules update without touching user files
3. **No merge conflicts** - User content separate from system content
4. **Efficient context** - Generic rules loaded once, not duplicated