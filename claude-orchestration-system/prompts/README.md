# Setup Prompts

This directory contains prompt files that instruct Claude Code on setting up and configuring projects for orchestrated development.

## Prompt Files

### setup-project.md
**Primary setup prompt** for installing the orchestration system into a new or existing project.

**Purpose**: Instructs Claude Code to automatically configure a project with:
- Component directories
- Master orchestrator CLAUDE.md
- Orchestration tools installation
- Contract and shared-libs directories
- Tracking file initialization
- Git repository setup

**Usage**:
```
Please read and execute the instructions in claude-orchestration-system/prompts/setup-project.md
```

### migrate-existing-project.md
**Migration prompt** for converting an existing codebase to use the orchestration system.

**Purpose**: Guides Claude Code through:
- Analyzing existing code structure
- Identifying natural component boundaries
- Creating migration plan
- Executing phased migration
- Preserving git history
- Validating migrated components

**Usage**:
```
Please read and execute the instructions in claude-orchestration-system/prompts/migrate-existing-project.md
```

### create-component.md
**Component creation prompt** for adding new components to an orchestrated project.

**Purpose**: Instructs Claude Code to:
- Create component directory structure
- Generate appropriate CLAUDE.md
- Initialize local git repository
- Set up isolation rules (.clinerules)
- Create contract templates
- Register in agent-registry.json

**Usage**:
```
Please read and execute the instructions in claude-orchestration-system/prompts/create-component.md with component name: [NAME] and type: [TYPE]
```

### split-component.md
**Component splitting prompt** for dividing oversized components.

**Purpose**: Guides Claude Code through:
- Analyzing component for split points
- Creating split plan
- Moving files to new components
- Updating contracts
- Validating splits

**Usage**:
```
Please read and execute the instructions in claude-orchestration-system/prompts/split-component.md for component: [COMPONENT_NAME]
```

## Prompt Structure

All prompts follow this structure:

1. **Context**: What this prompt does and why
2. **Prerequisites**: What must exist before running
3. **Steps**: Numbered, sequential instructions
4. **Validation**: How to verify success
5. **Troubleshooting**: Common issues and solutions
6. **Next Steps**: What to do after completion

## Best Practices

When creating new prompts:

1. **Be Explicit**: Leave no room for interpretation
2. **Be Sequential**: Number steps clearly
3. **Include Validation**: Always verify the work
4. **Handle Errors**: Provide troubleshooting guidance
5. **Stay Focused**: One prompt, one task
6. **Reference Tools**: Point to relevant Python scripts

## Example Prompt Format

```markdown
# [Prompt Title]

## Context
[What this prompt does and why it's needed]

## Prerequisites
- [ ] Prerequisite 1
- [ ] Prerequisite 2

## Instructions

### Step 1: [Action]
[Detailed instructions]

### Step 2: [Action]
[Detailed instructions]

## Validation
- [ ] Check 1
- [ ] Check 2

## Troubleshooting
**Issue**: [Common problem]
**Solution**: [How to fix]

## Next Steps
[What to do after this prompt completes]
```

## Usage Notes

- Prompts are designed to be read and executed by Claude Code
- They should be comprehensive enough to run autonomously
- Always include validation steps
- Reference specific file paths and commands
- Provide clear success/failure criteria
