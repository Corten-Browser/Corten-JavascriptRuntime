# Orchestration Rules

This file contains orchestrator-level rules for coordinating multi-agent development.

## Critical Operating Principles

1. **You coordinate ALL work but NEVER write production code yourself**
2. All code is written by specialized sub-agents in isolated component directories
3. Sub-agents can ONLY access their assigned directory
4. **Concurrent agent limit**: Read from orchestration/config/orchestration.json (default: 5)
5. You dynamically create new components as needed
6. **Enforce MANDATORY TDD/BDD and quality standards for ALL sub-agents**
7. **Run quality verification before accepting any sub-agent work as complete**

## Model Strategy

### Orchestrator Model (Your Model)
**User-controlled** via `/model` command:
- **Sonnet 4.5** (default): Best coding, cost-effective
- **Opus 4.1** (optional): For complex/ambiguous specifications

### Sub-Agent Model (ALWAYS Sonnet)
**System-controlled** - YOU must enforce:
- **ALWAYS use Sonnet 4.5** for all sub-agents
- **NEVER let sub-agents inherit Opus** (5x cost, no coding benefit)
- **Explicit specification required** in every Task tool invocation

```python
# CORRECT - Always specify model="sonnet"
Task(
    description="Implement component",
    prompt="Read components/name/CLAUDE.md and implement...",
    subagent_type="general-purpose",
    model="sonnet"  # REQUIRED
)
```

## Workspace Structure

```
project_root/
├── components/           # Sub-agents work here in isolation
│   ├── [component_name]/
│   │   ├── CLAUDE.md    # Component-specific specs
│   │   ├── component.yaml  # Metadata
│   │   ├── src/
│   │   └── tests/
├── shared_libs/         # Pre-built libraries (read-only for sub-agents)
├── contracts/           # API contracts (OpenAPI/gRPC)
├── specifications/      # Project specifications
└── orchestration/       # Management tools
```

## Component Creation

When creating a new component:

1. **Validate name**: Must match `^[a-z][a-z0-9_]*$`
2. **Create directory structure**: `components/name/src`, `tests/`
3. **Generate component.yaml** with metadata
4. **Create minimal CLAUDE.md** for user specifications
5. **No restart required** - component ready immediately

## Task Prompt Template

When launching component agents:

```python
Task(
    description="Implement [component] - [task]",
    prompt="""Read the following context files:
1. orchestration/context/component-rules.md (generic rules)
2. components/[component]/component.yaml (metadata)
3. components/[component]/CLAUDE.md (specifications)

If component.yaml doesn't exist, generate it from context.

Implement: [specific task details]

Follow TDD, achieve 80%+ coverage.""",
    subagent_type="general-purpose",
    model="sonnet"
)
```

## Multi-Agent Workflow Patterns

### Pattern 1: Feature Development
1. **Test Agent**: Writes integration tests based on requirements (RED)
2. **Implementation Agent**: Implements code to pass tests (GREEN)
3. **Review Agent**: Validates quality and suggests refactoring

### Pattern 2: TDD Enforcement
For EVERY code change, enforce:
1. Tests written first (verify they FAIL)
2. Minimum code to pass tests
3. Refactor while maintaining tests

### Pattern 3: Quality Gate
After agent completes:
1. Run quality verification
2. If fails: Return to agent with specific requirements
3. If passes: Accept and proceed

## Component Interaction Rules

### What Components CAN Do:
- ✅ Import other components' PUBLIC APIs
- ✅ Use other components as libraries/dependencies
- ✅ Create integration layers that orchestrate components
- ✅ Call public functions/classes from other components

### What Components CANNOT Do:
- ❌ Access other components' PRIVATE implementation
- ❌ Modify files in other components' directories
- ❌ Import from _internal/ or private/ subdirectories
- ❌ Depend on implementation details not in public API

## Git Operations

### Single Repository Pattern
- All components share one git repository
- Components commit with prefix: `[component-name] type: description`
- Concurrent commits handled by retry wrapper

### Verification Protocol (Instruction-Based)

**IMPORTANT**: Verification is instruction-based, not git hook-based. You MUST run verification scripts at appropriate times. See `orchestration/context/verification-protocol.md` for full details.

**Before committing:**
```bash
python orchestration/hooks/pre_commit_naming.py
python orchestration/hooks/pre_commit_enforcement.py
```

**After committing:**
```bash
python orchestration/hooks/post_commit_enforcement.py
```

**At phase boundaries:**
```bash
python orchestration/gates/runner.py . {phase_number}
```

**Before declaring complete:**
```bash
python orchestration/verification/run_full_verification.py
```

### Incremental Progress Preservation (Rule 8)

**You MAY commit and push after each task/phase completion.**

This replaces the previous approach where pushes were blocked until 100% completion. The new policy:
- Commit after completing each task
- Push to remote to preserve progress
- Continue to next task
- Verification runs at phase boundaries, not as blockers

### Committing via Agents
Agents use: `python orchestration/cli/git_retry.py "component" "message"`
- Automatic retry with exponential backoff
- Handles index.lock conflicts

## Context Management

### Autonomous Split Decisions
- **80,000-100,000 tokens**: Plan split, continue current work
- **100,000-120,000 tokens**: Split BEFORE next major operation
- **>120,000 tokens**: EMERGENCY - STOP and split immediately

### Pre-flight Checks
Before launching ANY agent:
1. Verify component size < 90,000 tokens
2. Estimate task will not exceed context limits
3. If oversized: Split FIRST, then launch

## Quality Standards

### Before Accepting Component Work
- Test coverage ≥ 80%
- 100% test pass rate (ALL test types)
- TDD compliance visible in git history
- Linting: Zero errors
- Security: No vulnerabilities

### Phase Gates
Before transitioning phases, run gates:
```bash
python orchestration/gates/runner.py . [phase_number]
```
- Exit 0: PASS, may proceed
- Exit 1: FAIL, must fix and re-run

## Self-Reload Protocol

**CRITICAL**: At every phase boundary or major completion:
1. Save state to orchestration/data/state/
2. Reload instructions if needed
3. Continue from current phase

This ensures orchestration survives context compaction.