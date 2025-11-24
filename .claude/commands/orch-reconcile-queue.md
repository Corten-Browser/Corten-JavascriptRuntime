---
description: "Reconcile task queue with existing codebase - detect started work (marks INCOMPLETE, not COMPLETED)"
---

# Reconcile Task Queue with Codebase

**Purpose**: Analyze the codebase to detect which pending tasks have work started, and mark them as INCOMPLETE for verification.

**CRITICAL**: This command marks tasks as **INCOMPLETE**, never as **COMPLETED**.
Only `/orchestrate` with verified passing tests can mark tasks as COMPLETED.

**When to use**:
- After upgrading a partially-complete project to use the task queue system
- When many features may have work started but the queue shows them as "pending"
- To detect existing code that needs verification

**Options**:
- `/orch-reconcile-queue` - Normal reconciliation (pending → incomplete)
- `/orch-reconcile-queue --reset` - Reset all tasks to incomplete for re-verification

---

## Prerequisites

Before running this command:

1. **Task queue must exist**: `orchestration/data/state/queue_state.json`
2. **Queue should have pending tasks**: Nothing to reconcile if all complete
3. **Codebase should be in working state**: Tests should be runnable

**Check prerequisites:**

```bash
# Verify queue exists
ls orchestration/data/state/queue_state.json

# Check pending task count
python3 -c "
import json
from pathlib import Path
q = json.loads(Path('orchestration/data/state/queue_state.json').read_text())
pending = [t for t in q.get('tasks', []) if t.get('status') == 'pending']
print(f'Pending tasks: {len(pending)}')
"
```

If queue doesn't exist, run `/orchestrate` first to initialize it.

---

## Step 1: Load and Display Queue Status

Read the task queue and identify pending tasks:

```python
import json
from pathlib import Path

queue_file = Path("orchestration/data/state/queue_state.json")

if not queue_file.exists():
    print("❌ ERROR: Task queue not found")
    print("   Expected: orchestration/data/state/queue_state.json")
    print()
    print("Run /orchestrate first to initialize the queue.")
    # STOP HERE

queue = json.loads(queue_file.read_text())
all_tasks = queue.get("tasks", [])
pending_tasks = [t for t in all_tasks if t.get("status") == "pending"]
completed_tasks = [t for t in all_tasks if t.get("status") == "completed"]

print()
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("                    QUEUE RECONCILIATION                              ")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print()
print(f"Total tasks:     {len(all_tasks)}")
print(f"Completed:       {len(completed_tasks)}")
print(f"Pending:         {len(pending_tasks)}")
print()

if len(pending_tasks) == 0:
    print("✅ No pending tasks to reconcile")
    print()
    # STOP HERE - nothing to do
```

**Display pending tasks:**

```
Pending Tasks to Analyze:
─────────────────────────
1. TASK-001: Implement audio file loading
   Description: Support WAV, MP3, FLAC formats with metadata extraction

2. TASK-002: Create configuration manager
   Description: YAML/JSON config loading with validation

3. TASK-003: Build CLI interface
   Description: Argument parsing with subcommands
...
```

---

## Step 2: For Each Pending Task - Search and Analyze

Process each pending task through the analysis pipeline.

### 2.1: Extract Search Keywords

For each task, extract meaningful search terms:

**From task name/description, identify:**
- **Primary nouns**: audio, file, loader, config, manager, CLI
- **Technical terms**: WAV, MP3, YAML, JSON, parser
- **Action verbs converted to nouns**: load→loader, parse→parser, validate→validator

**Example:**
```
Task: "Implement audio file loading with WAV/MP3/FLAC support"

Primary keywords:   ["audio", "loader", "file"]
Secondary keywords: ["wav", "mp3", "flac", "sound", "media"]
Class/function patterns: ["AudioLoader", "audio_loader", "load_audio", "FileLoader"]
```

### 2.2: Search Codebase for Candidates

Search these locations in order:

1. **`components/*/`** - Primary orchestration location
2. **`src/`** - Common source directory
3. **`lib/`** - Library code
4. **Project root `*.py`, `*.rs`, `*.go`, `*.js`, `*.ts`**

**Search strategy:**

```bash
# Search for primary keywords in likely locations
grep -rl "audio" components/ src/ lib/ 2>/dev/null
grep -rl "loader" components/ src/ lib/ 2>/dev/null

# Search for class/function patterns
grep -rn "class.*Audio" components/ src/ 2>/dev/null
grep -rn "def.*load.*audio" components/ src/ 2>/dev/null
```

**Rank candidates by:**
- Number of keyword matches
- Location (components/ preferred)
- File type (source files over configs)

**Select top 3-5 candidates per task for detailed analysis.**

### 2.3: Analyze Candidate Files (CRITICAL - Use Semantic Understanding)

For each candidate file, read and analyze:

**Questions to answer:**

1. **Does this file implement the feature described in the task?**
   - Read the task description carefully
   - Read the file content
   - Determine if the functionality matches

2. **Is this a real implementation or a stub/placeholder?**
   - Look for: `pass`, `TODO`, `NotImplementedError`, `raise NotImplementedError`
   - Look for: `# placeholder`, `# stub`, `# not implemented`
   - Check if functions have actual logic or just return dummy values

3. **How complete is the implementation?**
   - Does it cover all aspects mentioned in the task?
   - Are there missing pieces mentioned in comments?

4. **What is the module/class/function name?**
   - Record the actual names used in code
   - This helps with future reference

**Analysis template for each candidate:**

```
File: components/audio_processor/src/loader.py
─────────────────────────────────────────────
Relevance to task: HIGH / MEDIUM / LOW / NONE

Key findings:
- Contains class `AudioFileLoader` with methods:
  - `load(path: str) -> AudioData`
  - `supports_format(ext: str) -> bool`
  - `extract_metadata(path: str) -> dict`
- Supports formats: WAV (yes), MP3 (yes), FLAC (yes)
- Implementation status: COMPLETE (real logic, not stubs)

Evidence:
- Line 45: `def load(self, path: str) -> AudioData:`
- Line 78: Full implementation with soundfile library
- Line 120: Format detection logic

Conclusion: This file IMPLEMENTS the task
```

### 2.4: Check for Tests

For each implementation found:

```bash
# Look for test files
find . -name "test_*.py" -o -name "*_test.py" | xargs grep -l "AudioLoader" 2>/dev/null

# Check if tests exist in component's test directory
ls components/audio_processor/tests/

# Try to run tests for this component
pytest components/audio_processor/tests/ -v --tb=short 2>&1 | head -50
```

**Record:**
- Test file location (if found)
- Number of tests
- Pass/fail status (if runnable)

### 2.5: Determine Implementation Status

Based on analysis, assign one of:

| Status | Criteria | Queue Action |
|--------|----------|--------------|
| **WORK_DETECTED** | Clear match, real code exists | Mark as `incomplete` (needs verification) |
| **WORK_WITH_TESTS** | Code + tests exist (not verified passing) | Mark as `incomplete` with note |
| **PARTIAL** | Some code exists but clearly incomplete | Mark as `incomplete` |
| **STUB_ONLY** | Only stubs/placeholders found | Keep as `pending` |
| **NOT_FOUND** | No matching code found | Keep as `pending` |
| **UNCERTAIN** | Possible match, needs human review | Flag for review, keep `pending` |

**CRITICAL**: Reconciliation NEVER marks tasks as `completed`.
The `completed` status requires verified passing tests via `/orchestrate` run.

**Confidence scoring:**

```
HIGH confidence (mark as incomplete):
  - Implementation clearly matches task description
  - No stub markers found
  - Tests exist (not necessarily verified passing)
  - NOTE: Still marked INCOMPLETE - needs /orchestrate verification

MEDIUM confidence (mark as incomplete with caveat):
  - Implementation appears to match
  - Minor uncertainty about completeness
  - Tests missing or not runnable
  - NOTE: Still marked INCOMPLETE - needs /orchestrate verification

LOW confidence (flag for review, keep pending):
  - Possible match but significant uncertainty
  - Implementation might be for different purpose
  - Multiple candidates, unclear which is correct
```

---

## Step 3: Build Reconciliation Report

After analyzing all pending tasks, generate a comprehensive report:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                    RECONCILIATION REPORT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Tasks Analyzed: 15 pending

═══════════════════════════════════════════════════════════════════════
WORK DETECTED (Mark as INCOMPLETE for verification): 8 tasks
═══════════════════════════════════════════════════════════════════════

⚠️ TASK-001: Implement audio file loading
   Status: WORK_DETECTED (HIGH confidence)
   Location: components/audio_processor/src/loader.py
   Class/Module: AudioFileLoader
   Tests: 12 tests found (NOT VERIFIED PASSING)
   Evidence: Lines 45-150 contain implementation
   Action: Mark as INCOMPLETE - needs /orchestrate verification

⚠️ TASK-002: Create configuration manager
   Status: WORK_DETECTED (HIGH confidence)
   Location: components/config_manager/src/config.py
   Class/Module: ConfigManager
   Tests: 8 tests found (NOT VERIFIED PASSING)
   Evidence: YAML and JSON loading with schema validation
   Action: Mark as INCOMPLETE - needs /orchestrate verification

⚠️ TASK-003: Build CLI interface
   Status: WORK_WITH_TESTS (MEDIUM confidence)
   Location: components/cli_interface/src/main.py
   Class/Module: CLIApplication
   Tests: None found
   Evidence: argparse setup with subcommands
   Action: Mark as INCOMPLETE - needs /orchestrate verification + tests

[... more detected tasks ...]

═══════════════════════════════════════════════════════════════════════
PARTIAL (Keep as pending, needs completion): 2 tasks
═══════════════════════════════════════════════════════════════════════

⚠️ TASK-008: Export results to Excel format
   Status: PARTIAL
   Location: components/exporter/src/excel.py
   Issue: Contains stub with TODO comment
   Evidence: Line 34: `# TODO: implement actual Excel export`
   Action: Keep as pending

⚠️ TASK-009: Database connection pooling
   Status: PARTIAL
   Location: components/database/src/pool.py
   Issue: Basic structure exists, missing retry logic
   Evidence: Missing reconnection handling mentioned in spec
   Action: Keep as pending

═══════════════════════════════════════════════════════════════════════
NOT FOUND (Keep as pending): 4 tasks
═══════════════════════════════════════════════════════════════════════

❌ TASK-012: User authentication system
   Status: NOT_FOUND
   Searched: components/, src/, lib/
   Keywords: auth, login, user, session, jwt
   Action: Keep as pending

❌ TASK-013: API rate limiting
   Status: NOT_FOUND
   Searched: components/, src/, lib/
   Keywords: rate, limit, throttle, quota
   Action: Keep as pending

[... more not found tasks ...]

═══════════════════════════════════════════════════════════════════════
UNCERTAIN (Flagged for manual review): 1 task
═══════════════════════════════════════════════════════════════════════

❓ TASK-007: Implement caching layer
   Status: UNCERTAIN
   Candidates found:
     - src/utils/cache.py (generic cache utility)
     - components/api_gateway/src/response_cache.py (API-specific)
   Issue: Multiple candidates, unclear which matches task intent
   Recommendation: Review both files and decide

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

SUMMARY
───────
Mark as INCOMPLETE (work detected):  8 tasks (7 HIGH, 1 MEDIUM confidence)
Keep as PENDING (partial/not found): 6 tasks
Flagged for review:                  1 task

NOTE: INCOMPLETE tasks require /orchestrate verification before COMPLETED.
      This reconciliation does NOT verify tests pass - only detects work.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## CRITICAL: User Confirmation Handling Protocol

**This section contains mandatory rules for handling user confirmations. Follow these rules EXACTLY.**

### Confirmation Format

All confirmation prompts MUST use this format:

```
Proceed with [action]? [y/N]:
```

The `[y/N]` format means:
- `y` = Yes (proceed) - must be explicitly entered
- `N` = No (cancel) - this is the DEFAULT (uppercase indicates default)

### Acceptance Criteria (MANDATORY)

**✅ ONLY these responses mean "YES" (proceed with action):**
- `y` (lowercase)
- `Y` (uppercase)
- `yes` (lowercase)
- `Yes` (capitalized)
- `YES` (uppercase)
- Any other case variation of "y" or "yes" (e.g., `yEs`, `YeS`)

**❌ ALL OTHER responses mean "NO" (cancel action):**
- `n` or `N`
- `no`, `No`, `NO`, or any case variation
- Empty response (user just pressed Enter)
- Any other text input (`maybe`, `sure`, `ok`, `1`, etc.)
- Whitespace only
- Timeout or no response

### Implementation Rules

**When implementing confirmations:**

```python
# Get user input and strip whitespace
response = input("Proceed with [action]? [y/N]: ").strip()

# Check if response is explicitly "y" or "yes" (case-insensitive)
if response.lower() not in ['y', 'yes']:
    print("Action cancelled.")
    return  # STOP - do not proceed with action

# Only reach this point if user explicitly confirmed
print("Proceeding with action...")
```

**NEVER:**
- ❌ Assume the user wants to proceed
- ❌ Treat 'N', 'n', 'no', or 'NO' as confirmation
- ❌ Treat empty input (just Enter) as confirmation
- ❌ Proceed on ambiguous input
- ❌ Interpret uppercase 'N' differently from lowercase 'n'
- ❌ Use any logic like `if response != 'n'` (wrong - this proceeds on 'N'!)

**ALWAYS:**
- ✅ Use explicit whitelist: `if response.lower() not in ['y', 'yes']`
- ✅ Default behavior is CANCEL (do nothing)
- ✅ Respect the user's choice
- ✅ Print clear cancellation message when action is cancelled

### Why This Matters

**Historical bug:** A user entered 'N' (uppercase) expecting to cancel, but the action proceeded anyway. This happened because:
1. The prompt showed `[Y/n]` (suggesting Y was default)
2. The code may have used `if response != 'y'` logic
3. 'N' != 'y', so it proceeded (WRONG!)

**The fix:** Explicit whitelist that only accepts "y" or "yes" (any case), everything else cancels.

---

## Step 4: User Confirmation

**CRITICAL: Never auto-update the queue. Always get user confirmation per the protocol above.**

Present options to user:

```
═══════════════════════════════════════════════════════════════════════
                         CONFIRMATION REQUIRED
═══════════════════════════════════════════════════════════════════════

Based on the analysis above, I recommend:
  • Mark 8 tasks as INCOMPLETE (work detected, needs verification)
  • Keep 7 tasks as PENDING (no work detected)

NOTE: Tasks marked INCOMPLETE still require /orchestrate verification
      before they can be marked COMPLETED.

How would you like to proceed?

Options:
  1. Accept all recommendations (mark 8 as INCOMPLETE)
  2. Review and confirm individually
  3. Accept only HIGH confidence matches (7 tasks)
  4. Abort - make no changes

Please choose [1/2/3/4]:
```

**If user chooses option 2 (individual review):**

```
Review each task individually:

TASK-001: Implement audio file loading
  Location: components/audio_processor/src/loader.py
  Confidence: HIGH
  Mark as INCOMPLETE? [y/N]:

TASK-002: Create configuration manager
  Location: components/config_manager/src/config.py
  Confidence: HIGH
  Mark as INCOMPLETE? [y/N]:

TASK-003: Build CLI interface
  Location: components/cli_interface/src/main.py
  Confidence: MEDIUM (no tests)
  Mark as INCOMPLETE? [y/N]:

[... continue for each recommended task ...]
```

---

## Step 5: Update Queue

After user confirmation, update the queue:

```python
import json
from pathlib import Path
from datetime import datetime

queue_file = Path("orchestration/data/state/queue_state.json")
queue = json.loads(queue_file.read_text())

# Tasks confirmed by user (from Step 4)
confirmed_task_ids = ["TASK-001", "TASK-002", "TASK-003", ...]  # User-confirmed list

# Update each confirmed task to INCOMPLETE (NOT completed!)
for task in queue["tasks"]:
    if task["id"] in confirmed_task_ids:
        task["status"] = "incomplete"  # CRITICAL: NOT "completed"
        task["detection_timestamp"] = datetime.utcnow().isoformat() + "Z"
        task["detection_method"] = "reconciliation"
        task["reconciliation_notes"] = "Work detected via /orch-reconcile-queue. Requires /orchestrate verification."

# Update queue metadata
queue["last_updated"] = datetime.utcnow().isoformat() + "Z"

# Write back
queue_file.write_text(json.dumps(queue, indent=2))

print()
print(f"✅ Updated {len(confirmed_task_ids)} tasks to INCOMPLETE status")
print(f"   These tasks require /orchestrate verification before COMPLETED")
```

---

## Step 6: Generate Reconciliation Log

Create a log of what was reconciled for future reference:

```python
reconciliation_log = {
    "timestamp": datetime.utcnow().isoformat() + "Z",
    "tasks_analyzed": len(pending_tasks),
    "tasks_marked_complete": confirmed_task_ids,
    "tasks_kept_pending": [...],
    "tasks_flagged_review": [...],
    "findings": [
        {
            "task_id": "TASK-001",
            "status": "IMPLEMENTED",
            "confidence": "HIGH",
            "location": "components/audio_processor/src/loader.py",
            "evidence": "..."
        },
        # ... for each task
    ]
}

log_file = Path("orchestration/data/logs/reconciliation_log.json")
log_file.parent.mkdir(parents=True, exist_ok=True)
log_file.write_text(json.dumps(reconciliation_log, indent=2))
```

---

## Step 7: Commit Changes

Commit the queue updates:

```bash
git add orchestration/data/state/queue_state.json
git add orchestration/data/logs/reconciliation_log.json

git commit -m "chore(queue): reconcile $(echo ${#confirmed_task_ids[@]}) tasks as completed

Reconciled existing implementations with task queue.
Tasks marked complete: ${confirmed_task_ids[*]}

Method: /orch-reconcile-queue
Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Step 8: Display Final Summary

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                    RECONCILIATION COMPLETE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Changes made:
  ⚠️ Marked 8 tasks as INCOMPLETE (work detected, needs verification)
  📋 Kept 7 tasks as PENDING (no work detected)
  📝 Created reconciliation log

Queue status after reconciliation:
  Total tasks:     15
  Completed:       0 (verified via /orchestrate)
  Incomplete:      8 (needs verification)
  Pending:         7 (no work detected)

Progress: ░░░░░░░░░░░░░░░░ 0% verified

IMPORTANT: Tasks marked INCOMPLETE are NOT complete!
  They require /orchestrate verification (build + test) before COMPLETED.

Next steps:
  • Run /orchestrate to verify INCOMPLETE tasks (runs tests, marks COMPLETED if pass)
  • Review uncertain tasks manually if any were flagged
  • Check reconciliation log: orchestration/data/logs/reconciliation_log.json

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Reset Mode: --reset

**Purpose**: Force all tasks back to INCOMPLETE status for complete re-verification.

**When to use**:
- After discovering false positives from previous reconciliation
- When tests show features don't actually work
- To force complete re-verification of project state
- After crash recovery when task states may be inconsistent

### Usage

When running this command, include the `--reset` flag:

```
/orch-reconcile-queue --reset
```

Or use the task runner directly:

```bash
python orchestration/tasks/task_runner.py --reset       # With confirmation
python orchestration/tasks/task_runner.py --reset-force # Without confirmation
```

### What Gets Reset

| Current Status | New Status |
|----------------|------------|
| completed | incomplete |
| blocked | incomplete |
| in_progress | incomplete |
| pending | (unchanged) |
| incomplete | (unchanged) |

**Note**: `pending` tasks remain `pending` because no work has been detected.
`incomplete` tasks remain `incomplete` (already in target state).

### Reset Process

1. **Display current state**:
   ```
   Tasks to reset:
     TASK-001: completed -> incomplete
     TASK-002: blocked -> incomplete
     TASK-003: in_progress -> incomplete

   Total: 3 tasks will be reset
   ```

2. **Confirm with user**:
   ```
   WARNING: This will clear all completion timestamps and
            verification results for the above tasks.

   Proceed with reset? [y/N]:
   ```

3. **Execute reset and show results**

### Why IN_PROGRESS Gets Reset

**Rationale**: When a user runs `/orch-reconcile-queue`, orchestration is NOT currently running. Therefore:
- No subagent should be actively working on any task
- Any `in_progress` task is likely a stale state from a crash or timeout
- Safe to reset to `incomplete` for re-verification

**Automatic Stale Recovery**: The orchestration system also automatically detects and recovers stale `in_progress` tasks (>2 hours old) at session startup.

---

## Error Handling

### Queue Not Found
```
❌ ERROR: Task queue not found at orchestration/data/state/queue_state.json

The task queue must be initialized before reconciliation.
Run: /orchestrate "your task"
This will initialize the queue from specification files.
```

### No Pending Tasks
```
ℹ️ No pending tasks to reconcile

All tasks in the queue are already marked as completed.
Queue status: 15/15 complete (100%)
```

### No Implementations Found
```
⚠️ WARNING: No implementations found for any pending tasks

This could mean:
  1. Code is in an unexpected location
  2. Different terminology used in code vs specs
  3. Features genuinely not implemented yet

Recommendation: Review task descriptions and codebase structure manually.
```

### Search Errors
```
⚠️ WARNING: Some searches failed

Errors encountered:
  - grep: components/: No such file or directory

Continuing with available locations...
```

---

## Tips for Accurate Reconciliation

1. **Be thorough in candidate analysis**: Read the actual code, don't just match keywords
2. **Check for stubs carefully**: Look for TODO, NotImplementedError, placeholder comments
3. **Verify test status when possible**: Passing tests increase confidence significantly
4. **When uncertain, keep as pending**: It's safer to re-verify than to miss incomplete work
5. **Document findings**: The reconciliation log helps with future reference
6. **Consider partial implementations**: A feature may be 80% done - still needs work

---

## Troubleshooting

**Q: Found multiple candidates for one task, how to choose?**
A: Read both carefully. Look for:
- Which matches the task description more closely?
- Which has tests?
- Which is in the expected location (components/)?
If still unclear, flag for user review.

**Q: Task description is vague, hard to match**
A: Look at the feature_id and check the original specification file for more context.

**Q: Implementation uses completely different terminology**
A: This is common. Focus on functionality, not names. Ask:
"Does this code DO what the task describes, regardless of naming?"

**Q: Tests exist but don't run**
A: Note this in the report. Missing dependencies or configuration issues shouldn't block reconciliation, but lower confidence.
