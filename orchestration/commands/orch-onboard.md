# Onboard Existing Project

**Purpose**: Onboard an EXISTING software project (not created by orchestration system) so it can use all orchestration features.

**For**: Projects with existing source code that need orchestration infrastructure

**When to use**:
- Installing orchestration on an existing codebase
- Migrating a project to orchestrated development
- After running `install_existing.sh`

**When NOT to use**:
- New projects created by orchestration system (use `/orchestrate` instead)
- Projects that already have orchestration installed (run `/orchestrate`)

---

## What This Command Does

This command performs a comprehensive 12-phase onboarding process:

### Phase 0: Preflight Checks (Automated)
- Validates project is ready for onboarding
- Checks git repository, Python version, conflicts
- Ensures no existing orchestration installation

### Phase 1: Analysis & Discovery (LLM + Automated)
- Detects programming languages and project structure
- Discovers logical components in codebase
- Identifies entry points, APIs, features
- Maps dependencies between modules

### Phase 2: Planning (LLM-Heavy)
- Creates detailed reorganization plan
- Plans file movements (preserving git history)
- Identifies import path updates needed
- Determines migration strategy

### Phase 3: Installation (Automated)
- Runs `install_existing.sh` if not already done
- Installs orchestration system files
- Creates directory structure
- Sets up git hooks and enforcement

### Phase 4: Structure Preparation (User Review Required)
- Reviews reorganization plan with user
- Gets approval for file movements
- Creates backup commit before changes

### Phase 5: File Migration (Automated with Git History)
- Moves files using `git mv` (preserves history)
- Reorganizes code into component structure
- Creates incremental commits per phase

### Phase 6: Import Path Updates (LLM-Assisted)
- Fixes broken imports after file moves
- Updates relative and absolute import paths
- Validates Python syntax

### Phase 7: Manifest & Contract Generation (LLM + Templates)
- Generates `component.yaml` for each component
- Creates OpenAPI contracts for API boundaries
- Validates manifests against schema

### Phase 8: Testing Infrastructure (Template-Based + LLM)
- Sets up test directories per component
- Generates test templates
- Identifies test coverage gaps

### Phase 9: Specification Generation (LLM-Heavy)
- Extracts specification from code and docs
- Creates `specifications/extracted_system_spec.yaml`
- Documents all features for test coverage

### Phase 10: Task Queue Initialization (Automated + LLM Subagent)
- Runs `auto_init.py` for regex-based feature extraction
- Launches `/orch-extract-features` subagent for LLM extraction
- Populates task queue from extracted specification

### Phase 11: Queue Reconciliation (LLM Subagent)
- Launches `/orch-reconcile-queue` subagent
- Matches existing implementations to queue tasks
- Marks already-implemented features as complete
- Results in accurate queue state reflecting actual progress

### Phase 12: Validation & Verification (Automated)
- Runs `onboarding_verifier.py`
- Checks all components have manifests
- Validates contracts exist
- Ensures import health
- Final smoke tests

---

## Prerequisites

Before running this command, you must:

1. **Have orchestration system installed** via `install_existing.sh`:
   ```bash
   /path/to/ai-orchestration/scripts/install_existing.sh /path/to/your/project
   ```

2. **Be in your existing project directory**:
   ```bash
   cd /path/to/your/existing/project
   ```

3. **Have a clean git working tree** (or be ready to commit changes)

4. **Have Python 3.8+ installed**

---

## Usage

Simply type:

```
/orch-onboard
```

The command will guide you through each phase with approval gates for critical steps.

---

## Execution Instructions

You are Claude Code. When the user types `/orch-onboard`, follow this precise workflow:

### Step 1: Verify Prerequisites

Run preflight checks:

```bash
python3 orchestration/cli/onboard_preflight.py .
```

**If preflight fails:**
- Display error messages
- Ask user to resolve issues
- STOP - do not proceed

**If preflight passes:**
- Continue to Step 2

### Step 2: Run Automated Analysis

Execute structure analyzer:

```bash
python3 orchestration/migration/onboarding_planner.py analyze . --output analysis_report.md
```

Display summary to user:
- Languages detected
- Source file count
- Lines of code
- Existing components found
- Entry points discovered

### Step 3: LLM-Guided Component Discovery

Read the analysis report, then generate LLM analysis prompt:

```bash
python3 orchestration/migration/onboarding_planner.py llm-analyze .
```

This outputs a detailed prompt. You should:

1. **Execute the prompt yourself** (you are the LLM)
2. Analyze the project codebase
3. Discover logical components
4. Identify all features and APIs
5. Map component boundaries

Generate output as JSON:

```json
{
  "components": [
    {
      "name": "cli_interface",
      "type": "cli_application",
      "directory": "src/cli",
      "entry_point": "main.py",
      "responsibility": "Command-line interface for user interaction",
      "features": ["analyze command", "export command"],
      "dependencies": ["core_engine"],
      "estimated_loc": 500,
      "apis": []
    }
  ],
  "api_boundaries": [
    {
      "from_component": "cli_interface",
      "to_component": "core_engine",
      "interface_type": "function_call",
      "methods": ["analyze_file", "get_results"]
    }
  ],
  "testing_status": {...},
  "recommendations": [...]
}
```

Save this as `component_analysis.json`

### Step 4: User Review - Component Discovery

**CRITICAL**: Present component discovery to user for approval.

Display:
- Number of components discovered
- Component names, types, responsibilities
- API boundaries identified

Ask user:
```
Discovered X components:

1. cli_interface (cli_application) - Command-line interface
2. core_engine (library) - Analysis engine
3. file_processor (library) - File handling

API Boundaries:
- cli_interface → core_engine (function calls)
- core_engine → file_processor (function calls)

Does this match your understanding of the project structure?
Options:
A) Yes, proceed with this structure
B) No, let me provide corrections
C) Cancel onboarding
```

**If user chooses B:** Collect feedback, re-analyze, repeat Step 3-4

**If user chooses C:** STOP

**If user chooses A:** Continue to Step 5

### Step 5: Generate Reorganization Plan

Use component analysis to generate reorganization plan:

```bash
python3 orchestration/migration/onboarding_planner.py llm-plan . --component-analysis component_analysis.json
```

This outputs a detailed prompt. You should:

1. **Execute the prompt yourself**
2. Plan all file movements
3. Identify import updates needed
4. Assess risks
5. Choose migration strategy

Generate output as JSON (see onboarding_planner.py for schema)

Save as `reorganization_plan.json`

### Step 6: User Review - Reorganization Plan

**CRITICAL**: Get user approval for file movements.

Display:
- Migration strategy (minimal/moderate/full)
- Number of files to move
- Risk assessment
- Sample of file moves (first 10)

Ask user:
```
Reorganization Plan Summary:

Strategy: moderate (preserve internal structure, move to components/)
Files to move: 47
Import updates needed: 23 files
Estimated duration: 45 minutes

High-risk moves: 2 (entry points)
Medium-risk moves: 15 (core modules)
Low-risk moves: 30 (utilities)

Sample moves:
- src/cli/main.py → components/cli_interface/src/main.py
- src/engine/analyzer.py → components/core_engine/src/analyzer.py

Ready to proceed with reorganization?
Options:
A) Yes, proceed (creates backup commit first)
B) Show full plan first
C) Modify plan
D) Cancel
```

**If user chooses B:** Display full reorganization_plan.json, then re-ask

**If user chooses C:** Collect modifications, update plan, re-ask

**If user chooses D:** STOP

**If user chooses A:** Continue to Step 7

### Step 7: Create Backup Commit

Before ANY file movements:

```bash
cd . && git add -A && git commit -m "checkpoint: pre-reorganization backup

This commit preserves project state before orchestration reorganization.

Rollback command if needed:
  git reset --hard HEAD

Onboarding phase: file migration preparation
Timestamp: $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
```

Confirm backup created successfully.

### Step 8: Execute File Migration

**AUTOMATED PHASE** - No user interaction

For each file move in reorganization_plan.json:

1. Create destination directory: `mkdir -p <dest_dir>`
2. Move file: `git mv <source> <destination>`
3. Log progress

After all moves complete:

```bash
git commit -m "refactor(structure): reorganize into orchestration component structure

Moved 47 files to component directories:
- 15 files → components/cli_interface/
- 20 files → components/core_engine/
- 12 files → components/file_processor/

Preserves git history via 'git mv'.

Phase: file migration (5/12)
Reorganization plan: reorganization_plan.json"
```

### Step 9: Fix Import Paths

**AUTOMATED PHASE**

Generate import fix plan:

```bash
python3 orchestration/migration/import_updater.py fix . --plan reorganization_plan.json --output import_fix_plan.json
```

Review plan:
- Display number of import fixes
- Show sample fixes (first 5)

Apply fixes:

```bash
python3 orchestration/migration/import_updater.py apply . --plan import_fix_plan.json
```

Commit:

```bash
git add -A && git commit -m "fix: update import paths after reorganization

Updated import paths in 23 files to reflect new component structure.

Old: from src.engine import analyzer
New: from components.core_engine.src import analyzer

Phase: import path updates (6/12)
Fix plan: import_fix_plan.json"
```

### Step 10: Generate Component Manifests

**AUTOMATED PHASE**

```bash
python3 orchestration/generation/manifest_generator.py generate-all component_analysis.json
```

This creates `component.yaml` in each component directory.

Commit:

```bash
git add components/*/component.yaml && git commit -m "feat: add component manifests

Generated component.yaml for all X components.

Manifests declare:
- Component type (cli_application, library, etc.)
- User-facing features (for test coverage)
- Dependencies
- Entry points

Phase: manifest generation (7/12)
Schema: v2.0"
```

### Step 11: Generate API Contracts

**AUTOMATED PHASE**

```bash
python3 orchestration/generation/contract_generator.py generate-all component_analysis.json
```

This creates OpenAPI YAML files in `contracts/` directory.

Commit:

```bash
git add contracts/*.yaml && git commit -m "feat: add API contracts between components

Generated OpenAPI contracts for Y API boundaries:
- cli_interface → core_engine
- core_engine → file_processor

Contracts define:
- Method signatures
- Request/response schemas
- Error scenarios

Phase: contract generation (7/10)"
```

### Step 12: Set Up Testing Infrastructure

**AUTOMATED PHASE**

For each component:

1. Create `tests/` directory
2. Create `tests/unit/` and `tests/integration/`
3. Generate test template based on component type
4. Create `__init__.py` files

Commit:

```bash
git add components/*/tests/ && git commit -m "test: add testing infrastructure for all components

Created test directories and templates for X components.

Structure:
- tests/unit/ (isolated tests)
- tests/integration/ (component integration)
- tests/e2e/ (for CLI/web components)

Phase: testing infrastructure (8/12)"
```

### Step 13: Extract Specification

**LLM-HEAVY PHASE**

Generate specification extraction prompt:

```bash
python3 orchestration/migration/onboarding_planner.py llm-extract-spec . --component-analysis component_analysis.json
```

This outputs a detailed prompt. You should:

1. **Execute the prompt yourself**
2. Read codebase, README, docs
3. Extract all features
4. Document architecture
5. List dependencies

Generate output as YAML (see prompt for schema)

Save as `specifications/extracted_system_spec.yaml`

Commit:

```bash
git add specifications/ && git commit -m "docs: add extracted system specification

Extracted formal specification from existing codebase.

Includes:
- System architecture
- All X features with IDs
- Component responsibilities
- API contracts
- Testing requirements

This spec serves as authoritative reference for orchestration.

Phase: specification extraction (9/12)"
```

### Step 14: Task Queue Initialization

**AUTOMATED + LLM SUBAGENT PHASE**

Now that the specification has been extracted, initialize the task queue:

**Step 14a: Run automated extraction**

```bash
python3 orchestration/cli/auto_init.py
```

This performs regex-based feature extraction from the specification files.

**Step 14b: Launch LLM feature extraction subagent**

Use the Task tool to launch `/orch-extract-features`:

```python
Task(
    description="Extract features from specifications",
    prompt="""Execute the /orch-extract-features command.

Read and follow .claude/commands/orch-extract-features.md completely.

This will supplement the automated extraction with LLM intelligence,
catching features that regex patterns missed.

The specification was just extracted in Phase 9, so focus on:
- specifications/extracted_system_spec.yaml
- Any other spec files in specifications/

Report back with:
{
    "features_extracted": <number>,
    "queue_total": <number>,
    "success": true/false
}""",
    subagent_type="general-purpose"
)
```

**After subagent returns:**

Display status:
```
✅ Task queue initialized
   - Automated extraction: X features
   - LLM extraction: Y features
   - Total queue size: Z tasks (all pending)

Proceeding to reconciliation...
```

Commit:
```bash
git add orchestration/data/state/queue_state.json
git add orchestration/extraction_metadata.json
git commit -m "feat(queue): initialize task queue from specification

Extracted features from specifications into task queue.
- Automated extraction: X features
- LLM extraction: Y features
- Total: Z tasks

Phase: task queue initialization (10/12)"
```

### Step 15: Queue Reconciliation

**LLM SUBAGENT PHASE**

Since this is an existing codebase, many features may already be implemented.
Reconcile the queue with existing code:

```python
Task(
    description="Reconcile queue with existing implementations",
    prompt="""Execute the /orch-reconcile-queue command.

Read and follow .claude/commands/orch-reconcile-queue.md completely.

This is an existing codebase being onboarded, so many tasks in the queue
may already be implemented. Analyze the codebase to detect which features
are already complete.

IMPORTANT:
- Be thorough - this codebase existed before orchestration
- Check components/, src/, lib/ for implementations
- Look for tests that verify features work
- When uncertain, keep as pending (conservative)

Report back with:
{
    "tasks_analyzed": <number>,
    "marked_complete": <number>,
    "kept_pending": <number>,
    "reconciliation_complete": true/false
}""",
    subagent_type="general-purpose"
)
```

**After subagent returns:**

Display status:
```
✅ Queue reconciled with existing codebase
   - Tasks analyzed: X
   - Already implemented: Y (marked complete)
   - Pending work: Z tasks

Queue now accurately reflects project state.
```

Commit:
```bash
git add orchestration/data/state/queue_state.json
git add orchestration/data/logs/reconciliation_log.json
git commit -m "feat(queue): reconcile queue with existing implementations

Analyzed codebase and matched implementations to queue tasks.
- Tasks analyzed: X
- Marked complete: Y (already implemented)
- Remaining pending: Z

Phase: queue reconciliation (11/12)"
```

### Step 16: Final Verification

**AUTOMATED PHASE**

Run onboarding verifier:

```bash
python3 orchestration/verification/onboarding_verifier.py .
```

Display verification report:
- Directory structure: ✅
- Orchestration system: ✅
- Components: ✅ X found
- Manifests: ✅ All valid
- Contracts: ✅ Y found
- Specifications: ✅ Found
- Import health: ✅ No syntax errors
- Git status: ✅ All committed

**If verification passes:**
- Display success message
- Show next steps

**If verification fails:**
- Display failures
- Offer to fix automatically (where possible)
- Ask user how to proceed

### Step 17: Completion

**SUCCESS MESSAGE:**

```
═══════════════════════════════════════════════════════════
  ✅ ONBOARDING COMPLETE
═══════════════════════════════════════════════════════════

Your existing project has been successfully onboarded!

Changes made:
• X components discovered and isolated
• Y API contracts generated
• Z manifests created
• Import paths updated
• Testing infrastructure added
• Specification extracted
• Task queue initialized and reconciled

Task Queue Status:
• Total tasks: N
• Completed: M (already implemented)
• Pending: P (remaining work)
• Progress: [████████░░░░░░░░] M/N (XX%)

All changes committed to git with preservation of history.

Next steps:

1. Review the structure:
   tree components/

2. Run tests (if any exist):
   pytest

3. Check queue status:
   python3 orchestration/tasks/task_runner.py

4. Continue with remaining work:
   /orchestrate --resume "complete remaining features"

Documentation:
• Component structure: components/
• API contracts: contracts/
• Specifications: specifications/
• Task queue: orchestration/data/state/queue_state.json
• System docs: docs/ORCHESTRATION-USAGE-GUIDE.md

═══════════════════════════════════════════════════════════
```

---

## Error Handling

### If user's git tree is dirty:

```
⚠️  Git working tree has uncommitted changes.

Onboarding requires a clean state for safety. Options:

A) Commit changes now: git add -A && git commit -m "checkpoint: pre-onboarding"
B) Stash changes: git stash
C) Cancel onboarding (resolve manually)

What would you like to do?
```

### If onboarding fails mid-process:

```
❌ Onboarding failed at Phase X: [error message]

Your code is safe. All changes are committed incrementally.

Rollback options:

1. Undo last phase:
   git reset --hard HEAD~1

2. Undo all onboarding changes:
   git log --oneline --grep="Phase:"
   git reset --hard [checkpoint-commit-hash]

3. Continue from current state:
   /orch-onboard --resume

What would you like to do?
```

### If analysis discovers no components:

```
⚠️  No distinct components discovered in project.

This may indicate:
- Project is a single module (monolithic)
- Components not clearly separated
- Detection needs refinement

Options:

A) Treat as single component (generic type)
B) Manually specify component boundaries
C) Cancel onboarding

What would you like to do?
```

---

## Important Notes

1. **Git History is Preserved**: All file moves use `git mv` to maintain history

2. **Incremental Commits**: Each phase commits separately for easy rollback

3. **User Approval Gates**: Critical steps (component discovery, reorganization plan) require explicit approval

4. **Backup Checkpoints**: Multiple backup commits created before risky operations

5. **Idempotent**: Can be re-run if it fails (skips completed phases)

6. **Language Agnostic**: Works with Python, JavaScript, Go, Rust, C++, Java projects

7. **No Data Loss**: Original structure preserved in git history

---

## Related Commands

- `/orchestrate` - After onboarding, use this for development
- `/orch-extract-features` - Extract features from specifications into task queue
- `/orch-reconcile-queue` - Match existing code to queue tasks (detect already-implemented features)
- `/review` - Review generated manifests and contracts

---

## For Orchestrator (You)

**Remember:**
- You are both orchestrator AND the LLM for analysis phases
- Execute LLM prompts yourself, don't just display them
- Get user approval at critical gates (Steps 4, 6)
- Commit after each phase for safety
- Display progress clearly (Phase X/12)
- Be thorough in component discovery (affects entire structure)
- Validate all outputs before proceeding
- If stuck, ask user for guidance (don't guess)

**Display this rule in your response after completion:**

> **Rule: After Completion**
> - Mark /orch-onboard todo as completed ✅
> - Announce next steps (run /orchestrate)
> - Do NOT start new work unless user requests it
