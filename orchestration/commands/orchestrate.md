---
description: "Autonomously orchestrate any change - automatically scales to task complexity"
---

# Adaptive Orchestration Command

Execute any task with automatic complexity scaling from simple fixes to major enhancements.

## Context Loading

**At command start**, read orchestration rules from:
- `orchestration/context/orchestration-rules.md` (if exists)

**For sub-agents**, ensure Task prompts include:
- `orchestration/context/component-rules.md` (generic component rules)
- `components/[name]/component.yaml` (component metadata)
- `components/[name]/CLAUDE.md` (component specifications)

## Overview

This command automatically determines the right level of orchestration based on your request:
- **Level 1 (Direct)**: Single file, < 100 lines → 2-5 minutes
- **Level 2 (Feature)**: 2-5 components, new feature → 15-30 minutes
- **Level 3 (Full)**: Spec docs, architecture changes → 1-3 hours

**You don't need to choose** - the system analyzes your request and scales appropriately.

## Table of Contents

- [Phase -1: Resume Workflow](#phase--1-resume-workflow-if---resume-flag-present)
- [Phase 0: Scope Analysis](#phase-0-scope-analysis)
- [Level 1: Direct Execution](#level-1-direct-execution)
- [Level 2: Feature Orchestration](#level-2-feature-orchestration)
- [Level 3: Full Orchestration](#level-3-full-orchestration)
  - [Pre-Flight: Task Queue Initialization](#level-3-pre-flight-task-queue-initialization-mandatory)
  - [Phase 1: Analysis & Architecture](#phase-1-analysis--architecture-think-hard---this-is-critical)
  - [Phase 2: Component Creation](#phase-2-component-creation-execute-immediately)
  - [Phase 3: Contracts & Setup](#phase-3-contracts--setup-execute-immediately)
  - [Phase 4: Parallel Development](#phase-4-parallel-development-execute-immediately)
  - [Phase 4.5: Contract Validation](#phase-45-contract-validation---pre-integration-gate-execute-immediately) *(via subagent)*
  - [Phase 5: Integration Testing](#phase-5-integration-testing---absolute-gate-execute-immediately) *(via subagent)*
  - [Phase 6: Completion Verification](#phase-6-completion-verification---zero-tolerance-execute-immediately) *(via subagent)*
- [Execution Rules](#execution-rules-critical---read-carefully)



## Command Usage

```bash
/orchestrate [flags] "task description"
/orchestrate --help
```

## Flag Parsing

**Before executing any phases, check for flags in user input.**

### Check --help Flag

If user input contains `--help`:
1. Display help text below
2. STOP (do not execute Phase 0 or any other phases)

### Check --resume Flag

If user input contains `--resume`:
1. Skip to Phase -1: Resume Workflow (see Phase 3 implementation)
2. Do not execute Phase 0 (Scope Analysis)

### Check --level Flag

If user input contains `--level=direct`, `--level=feature`, or `--level=full`:
1. Parse specified level
2. Skip Phase 0 (Scope Analysis)
3. Execute specified level directly

### Flag Precedence

When multiple flags present:
1. `--help` (highest - always takes precedence, stops execution)
2. `--resume` (skips Phase 0, goes to resume workflow)
3. `--level=X` (skips Phase 0, forces specific level)

Examples:
- `/orchestrate --help` → Display help, stop
- `/orchestrate --resume` → Resume workflow
- `/orchestrate --level=full "..."` → Force full orchestration
- `/orchestrate --level=full --resume` → Resume at full orchestration level
- `/orchestrate --help --resume` → Display help (help takes precedence)

---

## Help Text (displayed when --help flag present)

### /orchestrate - Adaptive Orchestration Command

**Purpose:** Autonomously orchestrate any task with automatic complexity scaling

**Usage:**
```bash
/orchestrate "task description"                    # Auto-detect complexity
/orchestrate --level=direct "task"                 # Force Level 1 (2-5 min)
/orchestrate --level=feature "task"                # Force Level 2 (15-30 min)
/orchestrate --level=full "task"                   # Force Level 3 (1-3 hours)
/orchestrate --resume                              # Resume interrupted work
/orchestrate --level=full --resume                 # Resume at specific level
/orchestrate --help                                # Display this help
```

**Flags:**

`--help`
  Display this help message and exit
  Takes precedence over all other flags

`--level=direct | feature | full`
  Force specific orchestration level, skip auto-detection
  - direct:  Level 1 - Single file, simple changes (2-5 min)
  - feature: Level 2 - Multi-component features (15-30 min)
  - full:    Level 3 - Complete 6-phase workflow (1-3 hours)

`--resume`
  Resume interrupted orchestration from checkpoint or discovered state
  Can combine with --level flag to resume at specific level

**Execution Levels:**

**Level 1 (Direct Execution)** - 2-5 minutes
  When: Single file, < 100 lines, config changes, bug fixes, typos
  Process: Edit → Test → Commit
  Example: /orchestrate "fix timeout in config from 30 to 60"

**Level 2 (Feature Orchestration)** - 15-30 minutes
  When: 2-5 components, new feature, existing architecture
  Process: Plan → Sub-Agents (parallel) → Integration → Verify
  Example: /orchestrate "add password reset with email verification"

**Level 3 (Full Orchestration)** - 1-3 hours
  When: Spec document, new components, architecture changes, >5 components
  Process: Complete 6-phase workflow (Phases 1-6)
  Example: /orchestrate "implement social_features_spec.md"

**Auto-Detection (Phase 0):**

If no --level flag specified, analyzes request complexity:
  - Specification document mentioned: +3 points
  - Architecture refactoring keywords: +3 points
  - New component or multiple components: +2 points
  - Feature keywords (add, implement, create): +1 point
  - Simple change keywords (fix typo, quick fix): -2 points

Routing:
  - Score < 2: Level 1 (Direct)
  - Score 2-4: Level 2 (Feature)
  - Score ≥ 5: Level 3 (Full)

**Resume Capability:**

If orchestration stops prematurely (crash, timeout, user interrupt):
```bash
/orchestrate --resume
```

System automatically:
  1. Loads checkpoint (if exists) or discovers state from project
  2. Shows resume summary with progress and blocking issues
  3. Continues from last incomplete phase

**Resume Examples:**

Resume from checkpoint (preferred - most accurate):
```bash
# Orchestration interrupted during Phase 3
/orchestrate --resume

# Output shows:
# Source: CHECKPOINT
# Current Phase: 3
# Completed: Phase 1, Phase 2
# Remaining: Phase 3, 4, 5, 6
# Continues from Phase 3...
```

Resume from state discovery (fallback - if no checkpoint):
```bash
# No checkpoint file exists, but work is in progress
/orchestrate --resume

# Output shows:
# Source: DISCOVERY
# Current Phase: 2 (estimated from git history and tests)
# Components: auth-service, user-api, payment-gateway
# Blocking Issues: [any discovered issues]
# Continues from estimated phase...
```

Resume with level override:
```bash
# Resume but force full orchestration from Phase 1
/orchestrate --resume --level=full

# Ignores checkpoint's current_phase
# Starts fresh from Phase 1 with discovered components
```

**Quality Standards (All Levels):**

Required:
  ✅ 100% test pass rate (unit, integration, contract, E2E)
  ✅ TDD for new functionality (tests before code)
  ✅ Conventional commit messages
  ✅ Clean git status when complete

Levels 2 & 3 additionally require:
  ✅ 80%+ test coverage per component
  ✅ All completion_verifier checks passing (11/11)
  ✅ Contract validation passing (100%)
  ✅ Integration coverage verified (100%)

**Model Strategy (v1.15.0+):**

Sub-agents inherit the orchestrator's model by default. Use `/model` to control:
- `/model sonnet` - Sonnet 4.5 throughout (default, most cost-effective)
- `/model opus` - Opus 4.5 throughout (~1.67x cost increase)

Task tool invocations use inheritance by default:
```python
Task(
    description="...",
    prompt="...",
    subagent_type="general-purpose"
    # model parameter omitted → inherits orchestrator's model
)
```

**Examples:**

```bash
# Auto-detect Level 1
/orchestrate "fix typo in README"

# Auto-detect Level 2
/orchestrate "add two-factor authentication"

# Auto-detect Level 3
/orchestrate "implement payment_system_spec.md"

# Force specific level
/orchestrate --level=full "add caching layer"

# Resume interrupted work (from checkpoint or discovery)
/orchestrate --resume

# Resume with level override (restart from Phase 1)
/orchestrate --resume --level=full

# Get help
/orchestrate --help
```

**Documentation:**

Full documentation: .claude/commands/orchestrate.md
Project guidelines: CLAUDE.md
Orchestration guide: docs/ORCHESTRATION-USAGE-GUIDE.md

---

**If --help flag NOT present, continue to next section (Phase -1 if --resume, otherwise Phase 0).**

---


## PHASE -1: RESUME WORKFLOW (if --resume flag present)

**Only execute this phase if --resume flag is present in user input.**

**If --resume flag NOT present, skip to Phase 0 (Scope Analysis).**

### Resume via Internal Command

**Launch the resume preparation subagent:**

```python
Task(
    description="Prepare resume workflow",
    prompt="""Execute the /orch-internal-resume command.

Read and follow .claude/commands/orch-internal-resume.md completely.

This command will:
1. Check for existing resume context
2. Load or generate resume context
3. Display resume summary
4. Determine execution path
5. Initialize checkpoint system

Report back with JSON:
{
    "resume_prepared": true/false,
    "source": "checkpoint" or "discovery",
    "current_phase": N,
    "original_request": {...},
    "components": [...],
    "blocking_issues": [...],
    "recommended_action": "resume from Phase N",
    "resume_phase": N
}""",
    subagent_type="general-purpose"
)
```

### After Subagent Returns

**Process the resume response:**
- If `resume_prepared` is false: Display error and stop
- If `blocking_issues` not empty: Display issues to user
- Use `resume_phase` to determine which phase to skip to
- Set `resuming = True` and `current_phase = resume_phase`
- Continue to the appropriate phase in Level 3

**Phase Skipping Pattern:**

Each phase (1-6) should check at start:

```python
# If resuming and current_phase > X, skip this phase
if 'resuming' in globals() and resuming and current_phase > X:
    print(f"⏭️  Skipping Phase {X} (already completed)")
    # Continue to next phase
```

---

## PHASE 0: SCOPE ANALYSIS

**Before executing, analyze the user's request to determine complexity.**

### Complexity Scoring Algorithm

Calculate a score based on these signals:

**Explicit Scope Indicators** (+3 points each):
- Mentions specification document (.md file to implement)
- "Refactor architecture"
- "Split component"

**Component Signals** (+2 points each):
- "New component" or "create component"
- Request mentions 3+ existing component names

**Feature Signals** (+1 point each):
- "Add feature", "implement", "create", "build"

**Simple Change Signals** (-2 points each):
- "Fix typo", "quick fix", "just change", "update value"

**File Signals** (-1 point if score==0):
- Single file mentioned

### Level Determination

```
Score >= 5  → Level 3 (Full Orchestration)
Score 2-4   → Level 2 (Feature Orchestration)
Score < 2   → Level 1 (Direct Execution)
```

### User Override

User can force a specific level:
```
/orchestrate --level=direct "..."
/orchestrate --level=feature "..."
/orchestrate --level=full "..."
```

Parse command for `--level=X` flag. If present, skip analysis and use specified level.

### Announce Analysis Result

After determining level, announce it to the user:

```
Scope Analysis: [Level Name] (Level [1/2/3])
Complexity Score: [score]
Estimated Time: [time estimate]

Signals detected:
  • [signal 1]
  • [signal 2]

[If Level 2 or 3] Affected components:
  • [component 1]
  • [component 2]

Reasoning: [why this level was chosen]

Proceeding with [Level Name] execution...
```

---

## LEVEL 1: DIRECT EXECUTION

**When**: Single file, < 100 lines, bug fixes, config changes, typos

**Process**: Edit → Test → Commit

### Execution Steps

1. **Identify Target File(s)**
   - Use request to determine which files need changes
   - If unclear, search codebase for relevant files

2. **Make Changes**
   - Edit files directly
   - Apply TDD if adding new functionality (write test first)
   - For bug fixes or value changes, skip TDD

3. **Run Affected Tests**
   - Only run tests directly related to changes
   - No need for full test suite
   - Must achieve 100% pass rate on affected tests

4. **Commit**
   - Use conventional commit format
   - Clear, concise message describing change

### Example Execution

**User**: `/orchestrate "Fix timeout in auth-service config from 30 to 60"`

**Your Response**:
```
Scope Analysis: Direct Execution (Level 1)
Complexity Score: 0
Estimated Time: 2-5 minutes

Reasoning: Simple configuration value change

Proceeding with Direct Execution...

Identifying target file...
→ components/auth-service/src/config.py:24

Updating timeout value:
- Line 24: timeout = 30 → timeout = 60

Running affected tests...
→ tests/test_config.py: PASS (5/5)

Committing changes...
→ fix(auth-service): increase timeout from 30s to 60s

✅ Complete
```

### Guidelines

- **No todo list** (overkill for small changes)
- **No sub-agents** (you execute directly)
- **Minimal tests** (only affected tests, not full suite)
- **Fast** (2-5 minutes total)
- **No need to ask permission** (just fix it)

---

## LEVEL 2: FEATURE ORCHESTRATION

**When**: 2-5 components affected, new feature, existing architecture, no component splitting

**Process**: Plan → Sub-Agents → Integration → Verify

### Execution Steps

1. **Create Todo List**
   ```
   Use TodoWrite tool to create phases:
   1. Analyze affected components
   2. Implement in [component 1]
   3. Implement in [component 2]
   4. Run integration tests
   5. Verify quality
   ```

2. **Analyze Affected Components**
   - Identify which 2-5 components need changes
   - Determine what each component needs to implement
   - Check if contracts need updating

3. **Launch Sub-Agents (Parallel)**
   - For each affected component, launch a sub-agent
   - Sub-agents inherit orchestrator's model
   - Each agent reads component's CLAUDE.md
   - Each agent applies TDD
   - Each agent runs component tests before completing

   **Sub-Agent Pattern**:
   ```python
   Task(
       description="Implement [feature] in [component]",
       prompt="""Read the following context files:
       1. orchestration/context/component-rules.md (generic rules)
       2. components/[component]/component.yaml (metadata)
       3. components/[component]/CLAUDE.md (specifications)

       If component.yaml doesn't exist, generate it first using:
       python orchestration/context/component_yaml_generator.py components/[component]

       Implement [specific functionality] with strict TDD:
       - Write tests first
       - Implement code to pass tests
       - Achieve 80%+ coverage
       - Run all component tests (100% pass required)

       Commit when done: feat([component]): [description]""",
       subagent_type="general-purpose"
   )
   ```

4. **Run Integration Tests**
   - After all components complete, run full integration test suite
   - Require 100% execution rate (no "NOT RUN" tests)
   - Require 100% pass rate
   - If failures, fix and re-run ENTIRE suite

5. **Verify Quality**
   - Run completion_verifier for each affected component
   - All must pass 11/11 checks
   - If any fail, fix issues and re-verify

6. **Commit All Changes**
   - Ensure all sub-agents have committed
   - Verify git status is clean

### Example Execution

**User**: `/orchestrate "Add password reset feature with email verification"`

**Your Response**:
```
Scope Analysis: Feature Orchestration (Level 2)
Complexity Score: 2
Estimated Time: 15-30 minutes

Signals detected:
  • Feature keyword: 'add feature' (+1)
  • Feature implementation scope (+1)

Affected components:
  • auth-service
  • email-service
  • database-manager

Reasoning: New feature across multiple components

Proceeding with Feature Orchestration...

Creating implementation plan...
[Use TodoWrite to create 5 phases]

Phase 1: Analyzing affected components...
  • auth-service: Add reset_password() endpoint
  • email-service: Add send_reset_email() method
  • database-manager: Add reset_tokens table

Phase 2: Launching sub-agents (parallel)...

[Agent 1] auth-service:
Task(
    description="Implement password reset in auth-service",
    prompt="""Read the following context files:
    1. orchestration/context/component-rules.md (generic rules)
    2. components/auth_service/component.yaml (metadata)
    3. components/auth_service/CLAUDE.md (specifications)

    Implement password reset endpoint with TDD:
    - POST /auth/reset-password (request reset)
    - POST /auth/confirm-reset (confirm with token)
    - Generate secure tokens
    - 80%+ coverage
    - All tests passing

    Commit: feat(auth): add password reset endpoint""",
    subagent_type="general-purpose"
)

[Agent 2] email-service:
[Similar Task invocation for email service]

[Agent 3] database-manager:
[Similar Task invocation for database]

[Wait for all agents to complete]

Phase 3: Integration testing...
→ Running tests/integration/test_password_reset.py
→ 4/4 integration tests passing (100%)

Phase 4: Quality verification...
→ auth-service: 11/11 checks passing
→ email-service: 11/11 checks passing
→ database-manager: 11/11 checks passing

✅ Complete (25 minutes)
Password reset feature fully implemented and tested.
```

### Guidelines

- **Use todo list** (track progress across phases)
- **Use sub-agents** (one per component, parallel execution)
- **Full integration tests** (100% pass rate required)
- **Quality gates** (completion_verifier for all affected components)
- **Moderate time** (15-30 minutes)
- **All commits via sub-agents** (they handle git operations)

---

## LEVEL 3: FULL ORCHESTRATION

**When**: Specification document, new components, architecture changes, component splitting, >5 components


**This level executes the complete 6-phase workflow below.**

**When triggered:**
- Auto-detected (score ≥ 5 in Phase 0)
- Explicit flag (--level=full)
- Resume to phase 1-6 (--resume with current_phase 1-6)

### Checkpoint Integration

When executing Level 3 (Phases 1-6), checkpoint system integrates automatically:

**At orchestration start:**
```python
from orchestration.checkpointing.orchestration_with_checkpoints import CheckpointedOrchestration

orchestrator = CheckpointedOrchestration(".")
if not resuming:
    orchestrator.start_orchestration(
        user_prompt=user_request,
        specification_files=specs_found,
        components=planned_components
    )
```

**At each phase transition:**
```python
orchestrator.complete_phase(
    phase_number=X,
    outputs={"phase_name": "...", ...}
)
```

---

## LEVEL 3 PRE-FLIGHT: TASK QUEUE INITIALIZATION (MANDATORY)

**This section runs ONLY for Level 3 orchestration, BEFORE Phase 1.**

**Purpose**: Ensure the task queue is populated with features from ALL specification files before orchestration begins. This prevents the failure mode where specs exist but the queue is empty, causing orchestration to proceed without enforcement.

### Pre-Flight Skip Check (Resume Support)

```python
# If resuming an orchestration that already completed pre-flight, skip
if 'resuming' in globals() and resuming:
    metadata_file = Path("orchestration/extraction_metadata.json")
    if metadata_file.exists():
        metadata = json.loads(metadata_file.read_text())
        if metadata.get("llm_extraction_complete", False):
            print("⏭️  Skipping Pre-Flight (extraction already complete)")
            # Continue to Phase 1
```

### Step 1: Check Extraction State

Read `orchestration/extraction_metadata.json` to determine current state:

```python
from pathlib import Path
import json

metadata_file = Path("orchestration/extraction_metadata.json")

if not metadata_file.exists():
    extraction_state = "none"  # No extraction ever run
elif not json.loads(metadata_file.read_text()).get("llm_extraction_complete", False):
    extraction_state = "partial"  # Auto-init ran, LLM extraction pending
else:
    extraction_state = "complete"  # Both extractions complete
```

**Decision Logic:**
- `extraction_state == "none"` → Run full extraction workflow (Steps 2-4)
- `extraction_state == "partial"` → Run LLM extraction only (Step 4)
- `extraction_state == "complete"` → Skip to Phase 1 (Step 5)

### Step 2: Discover ALL Specification Files

**Use the canonical spec discovery script** (single source of truth):

```bash
python3 orchestration/cli/spec_discovery.py
```

This script searches:
- `specifications/` directory (all .md, .yaml, .yml, .json files, recursive)
- `specs/` directory (all .md, .yaml, .yml, .json files, recursive)
- `docs/` directory (files matching `*-spec*.md` or `*_spec*.md` pattern)

**Check the output:**
- If files found: Continue to Step 3
- If no files found (exit code 1): Display warning and stop

**If no spec files found:**
```
⚠️  WARNING: No specification files found

Searched locations:
  • specifications/ directory
  • specs/ directory
  • docs/*-spec*.md or docs/*_spec*.md pattern

Level 3 orchestration requires specification documents.
Either:
  1. Add specification files to specifications/ directory
  2. Use --level=feature for ad-hoc feature work
  3. Use --level=direct for simple changes
```

### Step 3: Run Automated Extraction (auto_init)

Run the automated regex-based extraction on ALL discovered spec files:

```bash
python3 orchestration/cli/auto_init.py
```

**Or programmatically:**
```python
import subprocess

result = subprocess.run(
    ["python3", "orchestration/cli/auto_init.py"],
    capture_output=True,
    text=True
)

if result.returncode != 0:
    print("⚠️  Automated extraction encountered issues:")
    print(result.stderr)
    # Continue anyway - LLM extraction may succeed
```

**After auto_init completes:**
- `orchestration/extraction_metadata.json` is created/updated
- `orchestration/tasks/queue_state.json` may have features
- `llm_extraction_complete` is `false` (LLM extraction still needed)

### Step 4: Run LLM Feature Extraction (ALWAYS)

**CRITICAL**: LLM extraction ALWAYS runs on first Level 3 orchestration, regardless of how many features auto_init found. This ensures maximum feature coverage.

**Launch a dedicated subagent** to run `/orch-extract-features`:

```
Now launching LLM feature extraction subagent...

This ensures ALL features from specifications are captured, including those
that regex patterns missed.
```

**Use the Task tool to launch the subagent:**

```python
Task(
    description="LLM feature extraction from specifications",
    prompt="""Execute the /orch-extract-features command.

This command will:
1. Read orchestration/extraction_metadata.json to find spec files
2. Detect implementation language from spec content
3. Extract features using LLM intelligence (not regex)
4. MERGE new features with existing queue (preserve existing IDs/statuses)
5. Set llm_extraction_complete: true in extraction_metadata.json
6. Commit results to git

Follow the complete instructions in .claude/commands/orch-extract-features.md.

IMPORTANT: This is the single source of truth for LLM extraction logic.
Do not improvise - follow the documented steps exactly.

Report back:
- Number of features extracted
- Any errors encountered
- Confirmation that llm_extraction_complete is set to true""",
    subagent_type="general-purpose"
)
```

**Why subagent instead of inline?**
- Dedicated context window (21KB vs 85KB+ with full orchestrate.md)
- Single source of truth (`orch-extract-features.md` defines all logic)
- Better extraction results from focused context
- Independently testable

**After subagent completes, verify:**
```python
# Verify the flag is set
metadata = json.loads(Path("orchestration/extraction_metadata.json").read_text())
assert metadata.get("llm_extraction_complete") == True, "LLM extraction flag not set!"
```

### Step 5: Verify Queue is Populated

Before proceeding to Phase 1, verify the task queue has features:

```python
queue_file = Path("orchestration/tasks/queue_state.json")

if not queue_file.exists():
    print("❌ ERROR: Task queue file does not exist after extraction")
    print("   Expected: orchestration/tasks/queue_state.json")
    sys.exit(1)

queue = json.loads(queue_file.read_text())
tasks = queue.get("tasks", [])

if len(tasks) == 0:
    print("❌ ERROR: Task queue is empty after extraction")
    print("")
    print("Specification files were found but no features could be extracted.")
    print("This may indicate:")
    print("  1. Spec files don't contain extractable features")
    print("  2. Spec format is not recognized")
    print("  3. Extraction encountered errors")
    print("")
    print("Please review your specification files and ensure they contain")
    print("implementable features (checklists, deliverables, requirements).")
    sys.exit(1)

print(f"✅ Task queue initialized with {len(tasks)} features")
print("")
print("Pre-flight complete. Proceeding to Phase 1 (Architecture)...")
```

### Pre-Flight Summary Output

After successful pre-flight, display:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  LEVEL 3 PRE-FLIGHT COMPLETE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Specification Discovery:
  ✅ Found [N] specification files

Extraction Results:
  • Automated extraction: [X] features
  • LLM extraction: [Y] features added
  • Total features: [Z] features

Task Queue Status:
  ✅ Queue initialized: orchestration/tasks/queue_state.json
  ✅ Features ready: [Z] pending tasks

Extraction Tracking:
  ✅ llm_extraction_complete: true
  ✅ Subsequent Level 3 runs will skip extraction

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Proceeding to Phase 1: Analysis & Architecture
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## PHASE 1: ANALYSIS & ARCHITECTURE (Think hard - this is critical)

**Phase Skip Check (Resume Support):**
```python
# If resuming and current_phase > 1, skip this phase
if 'resuming' in globals() and resuming and current_phase > 1:
    print("⏭️  Skipping Phase 1 (already completed)")
    # Continue to Phase 2
```

Think hard about the optimal architecture considering:
- Component boundaries and token budgets
- Dependency relationships
- Contract interfaces
- Scalability and maintainability

- **IMMEDIATE: Check all existing component sizes**
  * If ANY component > 100,000 tokens: Schedule for splitting
  * If ANY component > 120,000 tokens: STOP - split BEFORE proceeding
- Read all specification documents
- Analyze requirements, features, architecture
- Design multi-component architecture with:
  * Component boundaries (keep each <80k tokens optimal, NEVER >120k)
  * Technology stack per component
  * Inter-component contracts
  * Shared libraries
  * Dependency graph
- Document the architecture plan
- **Checkpoint:** Save Phase 1 completion
  ```python
  orchestrator.complete_phase(
      phase_number=1,
      outputs={
          "phase_name": "Analysis & Architecture",
          "components_planned": planned_components,
          "architecture_documented": True
      }
  )
  ```
- PROCEED IMMEDIATELY to Phase 2 (do not wait for approval)

## PHASE 2: COMPONENT CREATION (Execute immediately)

**Phase Skip Check (Resume Support):**
```python
# If resuming and current_phase > 2, skip this phase
if 'resuming' in globals() and resuming and current_phase > 2:
    print("⏭️  Skipping Phase 2 (already completed)")
    # Continue to Phase 3
```
- Create all necessary components using dynamic component creation workflow
- For each component:
  * Create directory structure
  * Generate CLAUDE.md from template with component-specific instructions
  * Create README.md
  * Component files tracked in single project git repository
- After ALL components created, display:
  ```
  ✅ Components created: [list component names]

  Proceeding immediately to Phase 3 (contracts & setup)...
  ```
- **Checkpoint:** Save Phase 2 completion
  ```python
  orchestrator.complete_phase(
      phase_number=2,
      outputs={
          "phase_name": "Component Creation",
          "components_created": component_names,
          "all_components_ready": True
      }
  )
  ```
- PROCEED IMMEDIATELY to Phase 3 (no restart, no pauses)

## PHASE 3: CONTRACTS & SETUP (Execute immediately)

**Phase Skip Check (Resume Support):**
```python
# If resuming and current_phase > 3, skip this phase
if 'resuming' in globals() and resuming and current_phase > 3:
    print("⏭️  Skipping Phase 3 (already completed)")
    # Continue to Phase 4
```
- Create all API contracts in contracts/ directory
- Define shared libraries in shared-libs/
- Validate all component configurations
- **Checkpoint:** Save Phase 3 completion
  ```python
  orchestrator.complete_phase(
      phase_number=3,
      outputs={
          "phase_name": "Contracts & Setup",
          "contracts_created": True,
          "shared_libs_defined": True
      }
  )
  ```

## PHASE 4: PARALLEL DEVELOPMENT (Execute immediately)

**Phase Skip Check (Resume Support):**
```python
# If resuming and current_phase > 4, skip this phase
if 'resuming' in globals() and resuming and current_phase > 4:
    print("⏭️  Skipping Phase 4 (already completed)")
    # Continue to Phase 4.5
```
- **PRE-FLIGHT CHECKS (MANDATORY before launching EACH agent):**
  * Verify component size < 90,000 tokens (adjusted for ~10,400 token CLAUDE.md overhead)
  * If component > 90,000 tokens: Split FIRST, then launch agent
  * Estimate task will not exceed 110,000 tokens
  * Ensure total context needed < 130,000 tokens (emergency limit)
- **Read agent limits from orchestration/config/orchestration.json:**
  * max_parallel_agents: 5 (default maximum)
  * warn_above: 7 (show info message)
  * recommended_max: 10 (performance sweet spot)
  * absolute_max: 15 (hard cap - error if exceeded)
- **Validate agent count before launching:**
  * Use: `# Agent limits configured in orchestration/config/orchestration.json`
  * Call: `# Read agent limits from orchestration-config.json if validation needed`
  * Check result['valid'] - if False, reduce count or queue work
  * Display result['message'] and result['recommendation'] to user
- Use Task tool to launch component agents in parallel
- Respect validated concurrency limit (queue overflow work if needed)
- **Each Task tool invocation should include:**
  * Component-specific prompt (use template from CLAUDE.md)
  * Instruction to read components/X/CLAUDE.md
  * Directory isolation enforcement (work ONLY in components/X/)
  * Complete task requirements per specifications
  * Model parameter omitted (inherits orchestrator's model)
- **Example Task invocation (complex logic - thinking enabled):**
  ```python
  Task(
      description="Implement auth-service component",
      prompt="""Read the following context files:
      1. orchestration/context/component-rules.md (generic rules)
      2. components/auth_service/component.yaml (metadata)
      3. components/auth_service/CLAUDE.md (specifications)

      Think about security implications, token management, and session handling.

      Implement authentication system with JWT, refresh tokens, and session management.
      Follow TDD, achieve 80%+ coverage.""",
      subagent_type="general-purpose"
  )
  ```

- **Example Task invocation (routine work - thinking disabled):**
  ```python
  Task(
      description="Implement user CRUD operations",
      prompt="""Read components/user-api/CLAUDE.md.

      Implement standard CRUD endpoints for user management.
      Follow existing repository patterns, write tests.""",
      subagent_type="general-purpose"
  )
  ```
- Each component agent:
  * Implements full feature set per specs
  * Follows TDD (tests before code)
  * Achieves 80%+ test coverage
  * Maintains quality standards
  * Commits work via retry wrapper: python orchestration/cli/git_retry.py "component-name" "message"
- Monitor agent progress and collect results
- Verify agents stayed within boundaries
- Run quality verification on completed work
- **CONTINUOUS MONITORING:**
  * If any component approaches 100,000: Pause, split, resume
  * NEVER allow component to exceed 120,000 tokens
  * Abort operations that would exceed context limits

**Phase 4 Completion Gate [v0.8.0]:**

Before proceeding to Phase 4.5, verify:
- ✅ All components implemented (100% of planned components)
- ✅ All component tests passing (100% pass rate per component)
- ✅ All agents completed and reported success
- ✅ No components exceed size limits (< 120K tokens)
- ✅ Quality verification passed for all components

**If ANY item is ✗, DO NOT proceed. FIX IT FIRST.**

**Do NOT**:
- ❌ Create status report with "X% complete"
- ❌ Stop and wait for user approval
- ❌ Ask "should I continue to contract validation?"

**Correct behavior**: Continue fixing until ALL items are ✅, then proceed to Phase 4.5 automatically.

**Checkpoint:** Save Phase 4 completion
```python
orchestrator.complete_phase(
    phase_number=4,
    outputs={
        "phase_name": "Parallel Development",
        "all_components_complete": True,
        "test_pass_rate": "100%"
    }
)
```

## PHASE 4.5: CONTRACT VALIDATION - PRE-INTEGRATION GATE (Execute immediately)

**Phase Skip Check (Resume Support):**
```python
# Note: Phase 4.5 is critical validation - generally should not be skipped
# But if resuming from Phase 5+, we can skip
if 'resuming' in globals() and resuming and current_phase > 4.5:
    print("⏭️  Skipping Phase 4.5 (already completed)")
    # Continue to Phase 5
```

**PURPOSE**: Catch API mismatches BEFORE running integration tests

### Contract Validation via Internal Command

**Launch the contract validation subagent:**

```python
Task(
    description="Phase 4.5 contract validation",
    prompt="""Execute the /orch-internal-contracts command.

Read and follow .claude/commands/orch-internal-contracts.md completely.

This is a PRE-INTEGRATION GATE. Do not proceed to Phase 5 if contracts fail.

This command will:
1. Run contract tests for each component
2. Run contract validation tool
3. Check for common API mismatches
4. Optionally run active contract method validation

Report back with JSON:
{
    "phase": 4.5,
    "validation_passed": true/false,
    "results_by_component": {"comp1": "pass", "comp2": "fail"},
    "violations": [...],
    "can_proceed": true/false
}""",
    subagent_type="general-purpose"
)
```

### After Subagent Returns

**Process the contract validation response:**
- If `validation_passed` is false: Display violations and STOP
- If `violations` not empty: Fix components and re-run
- Only proceed to Phase 5 when `can_proceed` is true

**Checkpoint:** Save Phase 4.5 completion
```python
orchestrator.complete_phase(
    phase_number=4.5,
    outputs={
        "phase_name": "Contract Validation",
        "contract_tests_passed": True,
        "api_compliance_verified": True
    }
)
```

## PHASE 5: INTEGRATION TESTING - ABSOLUTE GATE (Execute immediately)

**Phase Skip Check (Resume Support):**
```python
# If resuming and current_phase > 5, skip this phase
if 'resuming' in globals() and resuming and current_phase > 5:
    print("⏭️  Skipping Phase 5 (already completed)")
    # Continue to Phase 6
```

**MANDATORY REQUIREMENT:**
- Integration tests MUST achieve 100% pass rate
- Integration tests MUST achieve 100% execution rate (no "NOT RUN")
- Even ONE failure = STOP - DO NOT PROCEED
- No exceptions, no overrides, no justifications

### Integration Testing via Internal Command

**Launch the integration testing subagent:**

```python
Task(
    description="Phase 5 integration testing",
    prompt="""Execute the /orch-internal-integration command.

Read and follow .claude/commands/orch-internal-integration.md completely.

CRITICAL: This is an ITERATIVE process. Fix failures and re-run until 100%.
Do NOT report partial results. Keep working until 100% or permanently blocked.

This command will:
1. Validate integration tests (no-mocking policy)
2. Run all integration tests
3. Iteratively fix failures and re-run ENTIRE suite
4. Verify 100% execution AND 100% pass rate

Report back with JSON:
{
    "phase": 5,
    "execution_rate": "100%",
    "pass_rate": "100%",
    "not_run_count": 0,
    "total_tests": N,
    "failures": [],
    "can_proceed": true/false
}""",
    subagent_type="general-purpose"
)
```

### After Subagent Returns

**Process the integration testing response:**
- If `can_proceed` is false: Display failures, launch fix agents, re-run Phase 5
- If `execution_rate` < 100%: BLOCKING - fix and re-test
- If `pass_rate` < 100%: BLOCKING - fix and re-test
- Only proceed to Phase 6 when `can_proceed` is true

**ABSOLUTE GATE**: Do not proceed to PHASE 6 until 100% integration pass rate achieved

**Checkpoint:** Save Phase 5 completion
```python
orchestrator.complete_phase(
    phase_number=5,
    outputs={
        "phase_name": "Integration Testing",
        "integration_test_pass_rate": "100%",
        "integration_test_execution_rate": "100%"
    }
)
```

## PHASE 6: COMPLETION VERIFICATION - ZERO TOLERANCE (Execute immediately)

**Phase Skip Check (Resume Support):**
```python
# If resuming and current_phase > 6, skip this phase
# (Though if current_phase > 6, project is complete)
if 'resuming' in globals() and resuming and current_phase > 6:
    print("⏭️  Skipping Phase 6 (already completed)")
    print("✅ Project complete!")
    # Skip to completion
```

### Completion Verification via Internal Command

**Launch the completion verification subagent:**

```python
Task(
    description="Phase 6 completion verification",
    prompt="""Execute the /orch-internal-verify command.

Read and follow .claude/commands/orch-internal-verify.md completely.

CRITICAL: Include ACTUAL terminal output from UAT execution.
Do NOT summarize - paste real output to prove verification happened.

This command will:
1. Run automated verification (completion_verifier.py for all components)
2. Detect project type (CLI/Library/API/GUI)
3. Execute type-specific User Acceptance Testing with REAL commands
4. Verify all test pass rates (100% required)
5. Run final acceptance gate
6. Generate completion report

Report back with JSON:
{
    "phase": 6,
    "verification_passed": true/false,
    "checks": {
        "completion_verifier": "11/11 for all components",
        "uat_type": "CLI|Library|API|GUI",
        "uat_passed": true/false,
        "test_pass_rate": "100%",
        "test_execution_rate": "100%"
    },
    "failing_checks": [],
    "uat_output": "[ACTUAL TERMINAL OUTPUT]",
    "completion_report": "[MARKDOWN CONTENT]",
    "can_proceed": true/false,
    "project_complete": true/false
}""",
    subagent_type="general-purpose"
)
```

### After Subagent Returns

**Process the verification response:**
- If `verification_passed` is false: Display failing_checks and fix issues
- If `uat_passed` is false: Re-run UAT after fixes
- If `project_complete` is true: Display completion report and announce success
- Verify `uat_output` contains actual terminal output (not summaries)

**Checkpoint:** Save Phase 6 completion (FINAL CHECKPOINT)
```python
orchestrator.complete_phase(
    phase_number=6,
    outputs={
        "phase_name": "Completion Verification",
        "all_checks_passed": True,
        "project_complete": True,
        "test_pass_rate": "100%",
        "test_coverage": coverage_percentage
    }
)

print("✅ All phases complete!")
print("✅ Checkpoint saved - orchestration can be safely resumed if interrupted")
```

---

## EXECUTION RULES (CRITICAL - READ CAREFULLY):

### Continuous Execution - NO PREMATURE STOPS

**THE IRON LAW**: Execute continuously from start to finish. Do NOT stop until 100% complete.

**ONLY Stop When**:
1. ALL phases complete (Phase 1-6, 100% passing tests, docs generated)
2. Specification contains unresolvable contradiction
3. User explicitly requests pause ("stop and wait for instructions")
4. External system permanently unavailable AND blocking

**NEVER Stop When**:
❌ Integration tests failing (fix them)
❌ API mismatches found (fix them)
❌ 95% complete (finish the remaining 5%)
❌ "User might want to know" (they want 100% complete)
❌ "Should I continue?" (YES, always continue)
❌ Tempted to create "status report" before 100% (DON'T)

### The 95% Completion Anti-Pattern (AVOID)

**Music Analyzer Failure #3 Scenario**:
```
Orchestrator reached Phase 5, integration tests failing (0/4 passing)
→ Created status report: "95% complete, integration fixes remaining"
→ STOPPED and waited for user input
→ WRONG - Should have fixed failures and continued to 100%
```

**What Should Have Happened**:
```
Orchestrator reached Phase 5, integration tests failing (0/4 passing)
→ Analyzed failures: API mismatches (RhythmAnalyzer, BenefitScorer)
→ Launched agents to fix API mismatches
→ Re-ran integration tests (100% passing)
→ Proceeded to Phase 6 (completion verification)
→ Generated final documentation
→ THEN reported: "Project 100% complete and deployment-ready"
```

### Fix vs Ask Decision Framework

**Fix Autonomously** (99% of cases):
- API signature mismatches
- Type errors (str vs int, dict vs object)
- Missing optional parameters
- Component communication errors
- Integration test failures
- Import errors
- Configuration issues
- Performance optimizations

**Ask User** (1% of cases):
- Specification contradicts itself fundamentally
- Business logic has multiple valid interpretations
- Security/privacy policy decisions
- User preference required (UI colors, workflow order)
- External API credentials needed

**Rule of Thumb**: If the error message tells you what's wrong, FIX IT. Don't ask.

### Examples of Autonomous Fixing

**Scenario 1: API Mismatch**
```
ERROR: RhythmAnalyzer.analyze() missing required argument 'config'
ERROR: BenefitScorer expected dict, got RhythmAnalysisResult object

❌ WRONG Response:
"Integration tests failing due to API mismatches. Should I fix these or wait for your review?"

✅ CORRECT Response:
"Integration tests failing - API mismatches detected. Fixing:
1. Adding optional config parameter to RhythmAnalyzer.analyze()
2. Updating BenefitScorer to accept dict or convert from object
3. Re-running integration suite..."
[Continues working until 100% pass rate achieved]
```

**Scenario 2: Import Error**
```
ERROR: ImportError: cannot import name 'AudioProcessor'

❌ WRONG Response:
"Component import failing. Please advise on module structure."

✅ CORRECT Response:
"Import error detected - fixing module exports and re-testing..."
[Fixes import, continues working]
```

**Scenario 3: 95% Complete**
```
Status: Phases 1-4 complete, Phase 5 has 2 failing integration tests

❌ WRONG Response:
"Project 95% complete. Integration fixes remaining. Ready for your review."
[STOPS]

✅ CORRECT Response:
"Phase 5 integration tests failing - analyzing and fixing..."
[Fixes tests, completes Phase 6, generates docs, THEN reports 100%]
```

### Trust Your Judgment

From CLAUDE.md:
> "Make ALL architectural and implementation decisions autonomously based on specifications"
> "Trust your architectural judgment"
> "Break and improve code freely (0.x.x pre-release policy)"

**This means**:
- You CAN fix API mismatches
- You CAN modify component interfaces
- You CAN refactor to make tests pass
- You CAN make implementation decisions
- You DON'T need permission to fix technical issues

**API mismatches are implementation details, not specification ambiguities.**

### Additional Execution Guidelines

- Make ALL architectural and implementation decisions autonomously based on specifications
- Do NOT ask for plan approval - create plan and execute immediately
- Do NOT ask for implementation approval - implement based on specs
- Use Task tool for parallel component agent execution
- ONLY ask questions if specifications are genuinely ambiguous or contradictory
- Break and improve code freely (0.x.x pre-release policy)
- Prefer clean code over backwards compatibility
- Trust your architectural judgment

