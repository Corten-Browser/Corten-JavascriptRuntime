# Orchestration System Help

Welcome to the Claude Code Multi-Agent Orchestration System!

## Quick Reference

### Available Commands

**`/orchestrate`** - Main orchestration command (automatic complexity scaling)
- `/orchestrate "task description"` - Auto-detect complexity level
- `/orchestrate --level=full "implement spec"` - Force full orchestration
- `/orchestrate --resume` - Resume interrupted work
- `/orchestrate --help` - Show orchestrate command help

**`/orch-onboard`** - Onboard existing projects
- `/orch-onboard` - Start onboarding an existing codebase

**`/orch-extract-features`** - Extract features from specifications
- `/orch-extract-features` - LLM-based feature extraction

**`/orch-reconcile-queue`** - Reconcile queue with existing codebase
- `/orch-reconcile-queue` - Detect already-implemented features

**`/orch-help`** - Display this help
- `/orch-help` - Show this documentation (you are here)

---

## /orchestrate - Adaptive Orchestration Command

**Purpose:** Autonomously orchestrate any task with automatic complexity scaling

### Automatic Complexity Detection

The system automatically determines the right orchestration level:

| Level | Duration | Use Case | Example |
|-------|----------|----------|---------|
| **Level 1 (Direct)** | 2-5 min | Single file, simple changes | "Fix typo in README" |
| **Level 2 (Feature)** | 15-30 min | Multi-component features | "Add user authentication" |
| **Level 3 (Full)** | 1-3 hours | Spec implementation, architecture | "Implement project-spec.md" |

You don't need to choose—the system analyzes your request and scales appropriately.

### Common Usage

```bash
# Simple fix (auto-detects Level 1)
/orchestrate "fix typo in README.md"

# Multi-component feature (auto-detects Level 2)
/orchestrate "add JWT authentication to API and frontend"

# Full specification implementation (auto-detects Level 3)
/orchestrate "implement specifications/payment-system.md"

# Force specific level
/orchestrate --level=full "build entire project from spec"

# Resume interrupted orchestration
/orchestrate --resume
```

### Available Flags

- **`--help`** - Display orchestrate command help and stop
- **`--level=direct|feature|full`** - Force specific orchestration level (skip auto-detection)
- **`--resume`** - Resume interrupted orchestration from checkpoint

**Flag precedence:**
1. `--help` (highest - displays help and stops)
2. `--resume` (resumes from saved state)
3. `--level=X` (forces specific level)

### Level 3 Restart Requirement

**ONLY Level 3 (Full Orchestration)** requires a restart:

**Why restart is needed:**
- Phase 2 creates new components
- Components need MCP server registration
- Restart loads new MCP servers

**Workflow:**
1. `/orchestrate "implement spec"` - Starts Level 3
2. **Phase 1**: Analysis & architecture
3. **Phase 2**: Component creation
4. **⚠️ RESTART REQUIRED** - System pauses and tells you to restart
5. Close and reopen Claude Code
6. Say: "Restarted. Continue."
7. **Phase 3-6**: Contracts, development, integration, verification

**Levels 1 and 2 do NOT require restart** (no component creation).

### Execution Model

All orchestration levels use **autonomous execution**:
- ✅ No approval gates (except Level 3 restart)
- ✅ Makes all architectural decisions
- ✅ Implements everything per specifications
- ✅ Only asks questions if specs are ambiguous
- ✅ Enforces quality gates automatically
- ✅ Runs all tests and verification

---

## /orch-onboard - Onboard Existing Project

**Purpose:** Onboard an EXISTING codebase to use orchestration infrastructure

### When to Use

✅ **Use /orch-onboard when:**
- Installing orchestration on existing code
- After running `install_existing.sh`
- Migrating a project to orchestrated development
- Project has existing source code (not created by orchestration)

❌ **Do NOT use /orch-onboard for:**
- New projects created by orchestration system (use `/orchestrate`)
- Projects that already have orchestration installed (use `/orchestrate`)

### What It Does

Comprehensive 10-phase onboarding process:

1. **Preflight Checks** - Validates project readiness
2. **Analysis & Discovery** - Detects languages, structure, components
3. **Component Manifest** - Documents existing components
4. **Directory Setup** - Creates orchestration directories
5. **Contract Generation** - Generates API contracts
6. **Test Infrastructure** - Sets up testing framework
7. **Migration Planning** - Creates migration plan
8. **Migration Execution** - Executes migrations (if safe)
9. **Validation** - Verifies onboarding success
10. **Documentation** - Generates onboarding report

### Usage

```bash
/orch-onboard
```

The command runs autonomously and asks questions only when it needs clarification.

### After Onboarding

Once onboarding completes:
1. ✅ Use `/orchestrate` for all future development
2. ✅ Orchestration system fully integrated
3. ✅ All enforcement and quality gates active
4. ✅ Component-based development enabled

---

## /orch-extract-features - LLM-Based Feature Extraction

**Purpose:** Supplement automated feature extraction using Claude's intelligence

### When to Use

Use `/orch-extract-features` when:
- Automated extraction missed features from specifications
- Specifications are complex or narrative-style
- Task queue needs more granular tasks
- Features are implicit rather than explicit

### What It Does

1. Reads `orchestration/extraction_metadata.json` (processed spec files)
2. Detects implementation language from spec content
3. Focuses on files with low automated extraction rates
4. Extracts features using LLM understanding (not regex)
5. Applies feature templates for detected language
6. **Merges** new features into existing queue (UPDATE not REPLACE)
7. Preserves all existing features, IDs, statuses, timestamps
8. Auto-commits results to git

### Usage

```bash
/orch-extract-features
```

### Important Notes

- **Merges** (does not replace) existing task queue
- Preserves feature IDs and statuses
- Language detection is CRITICAL (stops if none detected)
- Only processes specs with low extraction rates
- Automatically commits to git when done

---

## /orch-reconcile-queue - Reconcile Queue with Codebase

**Purpose:** Detect which pending tasks are already implemented and update the queue

### When to Use

Use `/orch-reconcile-queue` when:
- Upgrading a partially-complete project to use the task queue system
- The queue shows tasks as "pending" but features are already implemented
- After initializing a queue on an existing codebase
- You need to sync queue state with actual implementation status

### The Problem It Solves

When you initialize a task queue on an existing project:
1. `auto_init.py` extracts features → creates tasks as **"pending"**
2. `/orch-extract-features` supplements → more tasks as **"pending"**
3. But many features may already be implemented!

A Python script can't determine if code implements a feature—that requires
semantic understanding. This command uses LLM intelligence to analyze the
codebase and match implementations to task descriptions.

### What It Does

1. **Loads pending tasks** from the queue
2. **For each task:**
   - Extracts keywords from task description
   - Searches codebase for candidate files
   - Reads and analyzes candidates with semantic understanding
   - Checks for stubs/placeholders (TODO, NotImplementedError)
   - Verifies if tests exist and pass
3. **Assigns confidence levels:**
   - **HIGH**: Clear implementation + tests pass
   - **MEDIUM**: Implementation exists, tests missing
   - **LOW**: Possible match, needs verification
   - **NONE**: No matching code found
4. **Generates detailed report** with evidence
5. **Requires user confirmation** before any changes
6. **Updates queue** with confirmed completions
7. **Creates audit log** for future reference

### Usage

```bash
/orch-reconcile-queue
```

### Important Safeguards

- ⚠️ **Never auto-marks tasks** - always requires user confirmation
- ⚠️ **Conservative matching** - when uncertain, keeps as pending
- ⚠️ **Preserves evidence** - logs what was found and why
- ⚠️ **Reversible** - queue can be manually adjusted after

### Example Output

```
═══════════════════════════════════════════════════════════════
                    RECONCILIATION REPORT
═══════════════════════════════════════════════════════════════

IMPLEMENTED (Recommend marking complete): 8 tasks

✅ TASK-001: Implement audio file loading
   Location: components/audio_processor/src/loader.py
   Confidence: HIGH
   Tests: 12 passing

✅ TASK-002: Create configuration manager
   Location: components/config_manager/src/config.py
   Confidence: HIGH
   Tests: 8 passing

PARTIAL (Keep as pending): 2 tasks

⚠️ TASK-005: Export to Excel format
   Location: components/exporter/src/excel.py
   Issue: Contains stub with TODO comment

NOT FOUND: 4 tasks

❌ TASK-010: User authentication
   Searched: components/, src/, lib/
   No matching implementation found

═══════════════════════════════════════════════════════════════
```

---

## Prerequisites

### For All Commands

1. **Orchestration system installed** in your project:
   ```bash
   bash scripts/install.sh /path/to/project
   ```

2. **Claude Code running** in project root directory

### For /orchestrate

**Specifications in project root** (for Level 3) or `specifications/` directory:
- Markdown: `project-spec.md`, `requirements.md`, `architecture.md`
- YAML: `api-spec.yaml`, `features.yaml`

**Specification best practices:**
```markdown
## Technology Stack
- Backend: Python 3.11, FastAPI, PostgreSQL
- Frontend: React 18, TypeScript, TailwindCSS

## Features
1. User authentication (JWT)
2. REST API with OpenAPI spec
3. PostgreSQL database with migrations
4. 80%+ test coverage

## Architecture
- Microservices architecture
- Event-driven communication
- CQRS for read/write separation
```

---

## Example Workflows

### Scenario 1: New Project from Specification

```bash
# 1. Create specification
vim specifications/my-project.md

# 2. Start orchestration (auto-detects Level 3)
/orchestrate "implement specifications/my-project.md"

# Output: Level 3 Full Orchestration detected
#   Phase 1: Analysis & Architecture (reading spec...)
#   Phase 2: Component Creation (creating components...)
#   ⚠️  RESTART REQUIRED

# 3. Restart Claude Code
# (close and reopen)

# 4. Continue after restart
Restarted. Continue.

# Output: Phase 3: Contracts & Setup...
#         Phase 4: Parallel Development...
#         Phase 5: Integration Testing...
#         Phase 6: Completion Verification
#         ✅ PROJECT COMPLETE
```

### Scenario 2: Add Feature to Existing Project

```bash
# Add authentication to existing project (auto-detects Level 2)
/orchestrate "add JWT authentication to API and frontend"

# Output: Level 2 Feature Orchestration detected
#   Creating feature plan...
#   @api-server, add authentication middleware...
#   @frontend, add login form...
#   Running integration tests... ✅
#   Feature complete!
```

### Scenario 3: Quick Fix

```bash
# Fix typo (auto-detects Level 1)
/orchestrate "fix typo in README: 'authentification' -> 'authentication'"

# Output: Level 1 Direct Execution
#   Fixed typo in README.md
#   Tests passing ✅
#   Committed changes
```

### Scenario 4: Onboard Existing Project

```bash
# After installing orchestration with install_existing.sh
/orch-onboard

# Output: Phase 1: Preflight checks... ✅
#         Phase 2: Analyzing codebase...
#         Detected: Python project with Flask
#         Found 5 logical components
#         Phase 3: Creating manifests...
#         ...
#         Phase 10: ✅ Onboarding complete!
#
#         Next: Use /orchestrate for development
```

### Scenario 5: Upgrade Partially-Complete Project

This workflow is for projects that:
- Were previously being built with orchestration
- Are not yet complete (orchestration stopped prematurely)
- Need to be upgraded to use the new task queue system
- Have many features already implemented

```bash
# 1. Upgrade orchestration system
cd /path/to/project
bash /path/to/ai-orchestration/scripts/upgrade.sh

# 2. Ensure specifications are in place
ls specifications/
# my-project-spec.md, feature-list.yaml, etc.

# 3. Initialize task queue (extracts features from specs)
python3 orchestration/cli/auto_init.py

# Output: Discovered 2 spec files
#         Extracted 15 features
#         Queue initialized: 15 pending tasks

# 4. Supplement with LLM extraction (catches what regex missed)
/orch-extract-features

# Output: Analyzing specifications...
#         Found 5 additional features
#         Queue now has 20 pending tasks

# 5. Reconcile queue with existing codebase
#    This detects which tasks are already implemented!
/orch-reconcile-queue

# Output: ═══════════════════════════════════════════════════════
#                     RECONCILIATION REPORT
#         ═══════════════════════════════════════════════════════
#         IMPLEMENTED: 12 tasks (recommend marking complete)
#         PARTIAL: 3 tasks (keep as pending)
#         NOT FOUND: 5 tasks (keep as pending)
#
#         Proceed with marking 12 as complete? [Y/n]: y
#
#         ✅ Marked 12 tasks as completed
#         Queue status: 12 completed, 8 pending

# 6. Resume orchestration for remaining work
/orchestrate --resume "complete remaining features"

# Output: Resuming orchestration...
#         8 tasks remaining
#         Proceeding with implementation...
```

**Key insight:** Without `/orch-reconcile-queue`, you'd have to manually
identify and mark each completed task. The reconcile command uses LLM
intelligence to analyze your codebase and match implementations to tasks.

---

## Orchestration System Features

### Automatic Enforcement

The orchestration system enforces quality standards automatically:

✅ **Test-Driven Development (TDD)**
- Tests written before implementation
- 80%+ test coverage required
- Unit, integration, and contract tests

✅ **Quality Gates**
- Linting (code style)
- Formatting (consistent style)
- Complexity limits
- Phase gates (integration, verification)

✅ **Contract-Based Communication**
- OpenAPI/YAML contracts between components
- Contract validation before integration
- Breaking change detection

✅ **Component Isolation**
- Each component has dedicated sub-agent
- Strict boundaries (no cross-component access)
- Communication only through contracts

✅ **Progress Preservation**
- Git commits at phase boundaries
- Restart-safe workflows
- Resume capability

### Adaptive Concurrency

- Default: 5 parallel agents (good for most projects)
- Configurable: 5-10 agents (performance sweet spot)
- Maximum: 15 agents (hard cap)
- Set in `orchestration-config.json`

---

## Tips for Best Results

### 1. Complete Specifications

The more detailed your specs, the better the results:
- ✅ Detailed feature requirements
- ✅ API endpoint specifications
- ✅ Data models and schemas
- ✅ User flows and stories
- ✅ Non-functional requirements (performance, security)

### 2. Technology Stack Guidance

Include technology preferences:
```markdown
## Technology Stack
- Backend: Python 3.11, FastAPI, PostgreSQL
- Frontend: React 18, TypeScript, TailwindCSS
- Caching: Redis
- Queue: Celery
```

### 3. Architectural Constraints

Specify architectural requirements:
```markdown
## Architecture
- Microservices architecture
- RESTful APIs with OpenAPI specs
- Event-driven communication for async operations
- CQRS pattern for read/write separation
```

### 4. State Intent Explicitly

For autonomous execution, state clearly:
```markdown
## Execution Mode
Execute autonomously. Make architectural decisions
without asking for approval. Only ask questions if
requirements are ambiguous or contradictory.
```

---

## Troubleshooting

### Commands Not Found

**Symptom:** Claude Code doesn't recognize `/orchestrate` or `/orch-help`

**Solution:**
1. Ensure `.claude/commands/` directory exists in project root
2. Restart Claude Code to load commands
3. Check file permissions: `ls -la .claude/commands/`

### Orchestrator Asks Too Many Questions

**Symptom:** Gets approval for every decision

**Solution:**
- Make specifications more detailed
- Explicitly state "make decisions autonomously"
- Include technology stack and architecture constraints

### Level 3 Restart - Components Not Available

**Symptom:** After restart, components not found

**Solution:**
1. Check `.claude.json` exists in project root
2. Verify component paths are absolute (not relative)
3. Confirm restart was complete (not just window refresh)
4. Check MCP server logs for loading errors

### Quality Gates Failing

**Symptom:** Tests fail, verification fails

**Solution:**
- Review failing test output
- Check test coverage reports
- Orchestrator attempts fixes automatically
- If persistent, provide more context in specs

### /orch-onboard Fails on Existing Project

**Symptom:** Preflight checks fail

**Solution:**
1. Ensure git repository exists (`git init`)
2. Check Python version (3.8+)
3. Remove any conflicting orchestration installation
4. Check for write permissions

### Extraction Produces No Features

**Symptom:** `/orch-extract-features` completes but no features extracted

**Solution:**
1. Check `orchestration/extraction_metadata.json` exists
2. Verify specification files have explicit features
3. Ensure language is detectable in spec (Python, JavaScript, Go, Rust)
4. Check task queue: `cat orchestration/tasks/queue_state.json`

---

## Advanced Usage

### Resume After Interruption

If orchestration is interrupted (crash, timeout, etc.):

```bash
/orchestrate --resume
```

The system will:
- Read orchestration state
- Determine last completed phase
- Resume from next phase
- Preserve all progress

### Force Specific Level

Override auto-detection:

```bash
# Force Level 3 even for small tasks
/orchestrate --level=full "add login button"

# Force Level 1 even for complex tasks (not recommended)
/orchestrate --level=direct "implement entire auth system"
```

### Override Concurrency

Edit `orchestration-config.json`:

```json
{
  "max_concurrent_agents": 8
}
```

Then restart orchestration.

### Focus on Specific Phase

```bash
/orchestrate "execute only Phase 1 (analysis) and Phase 2 (component creation), then stop"
```

Orchestrator will respect phase boundaries when explicitly instructed.

---

## File Locations

**Commands:** `.claude/commands/`
- `orchestrate.md` - Main orchestration command
- `orch-onboard.md` - Onboarding command
- `orch-extract-features.md` - Feature extraction command
- `orch-reconcile-queue.md` - Queue reconciliation command
- `orch-help.md` - This help file

---

## Internal Commands

Commands prefixed with `/orch-internal-` are internal commands used by the orchestration
system. They are called automatically by `/orchestrate` via subagents and are **not
intended for direct user invocation**.

Internal commands provide focused context for specific orchestration phases:
- Contract validation (Phase 4.5)
- Integration testing (Phase 5)
- Completion verification (Phase 6)
- Resume workflow preparation

**Do not call `/orch-internal-*` commands directly** unless you understand the
orchestration workflow and have a specific reason to do so. These commands expect
specific orchestration context and may not work correctly when called directly.

**Configuration:** `orchestration-config.json`
- Max concurrent agents
- Model strategy
- Language settings

**State:** `orchestration/data/`
- `state/queue_state.json` - Task queue
- `logs/activity_log.json` - Execution log
- `logs/reconciliation_log.json` - Reconciliation audit log
- `state/completion_state.json` - Completion state
- `checkpoints/` - Session checkpoints

**Specifications:** `specifications/`
- Project specs (Markdown, YAML)
- Feature documents
- Architecture diagrams

---

## See Also

**Documentation:**
- `CLAUDE.md` - Master orchestrator instructions
- `README.md` - Orchestration system overview
- `docs/ORCHESTRATION-USAGE-GUIDE.md` - Comprehensive usage guide
- `docs/INSTALLATION.md` - Installation instructions

**Scripts:**
- `scripts/install.sh` - Install on new project
- `scripts/install_existing.sh` - Install on existing project
- `scripts/upgrade.sh` - Upgrade orchestration system

**Quality:**
- `orchestration/verification/run_full_verification.py` - Manual verification
- `orchestration/gates/runner.py` - Run phase gates manually

---

**Happy Orchestrating!** 🎯

---

**Version:** 1.11.2
**Last Updated:** 2025-11-23
**Command:** `/orch-help`
