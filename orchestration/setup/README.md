# Automatic Enforcement System Setup (v1.3.0)

This directory contains the installation and verification scripts for the automatic enforcement system.

## What This Does

The automatic enforcement system prevents premature stopping by generating **forceful continuation messages** after every commit. Git hooks run automatically:

1. **Pre-commit hook** (advisory): Warns about incomplete work but ALLOWS commit
2. **Post-commit hook** (enforcement): Generates maximum-force continuation message
3. **Result**: Progress preserved (Rule 8) AND model receives clear instruction to continue

**Key insight:** Blocking commits punishes the user (work not saved). Allowing commits with forceful reminders preserves work while maximizing pressure to continue.

## Installation

### Fresh Installation

```bash
# From your project directory
bash path/to/orchestration-system/scripts/install.sh .

# This automatically:
# 1. Copies orchestration files
# 2. Installs git hooks
# 3. Sets up state directories
# 4. Configures enforcement
```

### Upgrade from v1.2.0

```bash
# From your project directory
bash path/to/orchestration-system/scripts/upgrade.sh .

# Migration 1.2.0 → 1.3.0 automatically:
# 1. Adds auto-initialization system
# 2. Installs git hooks
# 3. Sets up monitoring
# 4. Configures automatic enforcement
```

## Files in This Directory

### install_enforcement.py

Main installation script that:
- Installs git pre-commit, pre-push, post-checkout hooks
- Creates state directories (task_queue, monitoring, verification/state)
- Initializes configuration files
- Auto-discovers specification files
- Sets up activity logging

Usage:
```bash
python orchestration/setup/install_enforcement.py
```

### verify_installation.py

Validates that enforcement system is properly installed:
- Checks all required files exist
- Verifies git hooks are executable
- Confirms state directories created
- Validates configuration

Usage:
```bash
python orchestration/setup/verify_installation.py
```

## How It Works

### Pre-Commit Hook (Advisory Warnings)

Every `git commit` automatically triggers advisory checks:

1. **Auto-Init**: Initializes queue from specs if not done
2. **Queue Check**: Warns if tasks remain (but ALLOWS commit)
3. **Rationalization Detection**: Warns about forbidden phrases
4. **Stub Detection**: Warns about placeholder code

The hook **always exits with code 0** (allowing commit for Rule 8 compliance).

### Post-Commit Hook (Primary Enforcement)

After every successful commit:

1. **Queue Status**: Checks if tasks remain
2. **Message Generation**: Creates maximum-force continuation message
3. **Redundant Warnings**: Repeats "DO NOT STOP" multiple times
4. **Specific Instructions**: Names exact next task to work on
5. **Visible Output**: Message appears in terminal (LLM context)

Example output when queue not empty:

```
════════════════════════════════════════════════════════════════════════
🚨 CRITICAL: YOUR WORK IS NOT COMPLETE 🚨
════════════════════════════════════════════════════════════════════════

COMMIT SUCCESSFUL - Progress preserved.

HOWEVER: You have NOT finished the orchestrated task.

QUEUE STATUS: 5 of 10 tasks remaining (50% complete)

MANDATORY NEXT ACTION:
  Continue with: TASK-006 - Implement authentication service

DO NOT STOP. DO NOT ASK USER IF YOU SHOULD CONTINUE.

════════════════════════════════════════════════════════════════════════
🚨 RESUME WORK IMMEDIATELY - DO NOT STOP 🚨
════════════════════════════════════════════════════════════════════════
```

### Auto-Initialization

When a specification file is discovered:

1. Scans `specifications/`, `specs/`, `docs/` directories
2. Extracts features from YAML or Markdown specs
3. Creates task queue automatically
4. No model action required

### Stall Detection

Background monitoring that:

1. Tracks activity timestamps
2. Detects inactivity with pending work
3. Identifies tasks stuck in_progress too long
4. Logs alerts for visibility

### Auto-Sync

Automatically detects completed tasks by:

1. Scanning for implementation code with keywords
2. Checking for corresponding tests
3. Marking tasks complete when evidence found
4. Conservative approach (requires both code AND tests)

## Configuration

Edit `orchestration/enforcement_config.json`:

```json
{
  "blocking_mode": true,          // Enable/disable blocking
  "auto_init_queue": true,        // Auto-initialize from specs
  "auto_run_verification": true,  // Auto-run verification
  "block_rationalization": true,  // Block rationalization language
  "block_stubs": true,            // Block stub/placeholder code
  "stall_threshold_minutes": 60,  // Stall detection threshold
  "require_queue_empty_for_commit": true,
  "require_verification_for_commit": true
}
```

## Key Difference from v1.2.0

**v1.2.0**: Hooks existed but required manual installation. Model had to cooperate.

**v1.3.0**: Hooks are installed automatically by install.sh/upgrade.sh. Model cannot bypass enforcement because hooks are external to model control.

## Troubleshooting

### Commits are blocked

Check what's blocking:
```bash
python orchestration/session_init.py --json
```

This shows:
- Queue status (is it empty?)
- Verification status (is it approved?)
- Remaining tasks
- Next required action

### Queue won't initialize

Ensure specification file exists:
```bash
ls specifications/
ls specs/
ls docs/*-spec*
```

Queue auto-discovers from these locations.

### Verification failing

Run manually with details:
```bash
python orchestration/verification/run_full_verification.py
```

This shows exactly what checks are failing.

### Stall detection alerts

Check monitoring status:
```bash
python orchestration/enforcement/stall_detector.py
```

This shows if work has stalled and recommendations.

## Uninstalling

To remove automatic enforcement:

1. Remove git hooks:
   ```bash
   rm .git/hooks/pre-commit
   rm .git/hooks/pre-push
   rm .git/hooks/post-checkout
   ```

2. Disable blocking mode:
   ```bash
   echo '{"blocking_mode": false}' > orchestration/enforcement_config.json
   ```

## Why This Approach?

Historical failures showed that instructional guidance (telling the model not to stop) doesn't work because models use motivated reasoning to rationalize around rules. Technical enforcement via git hooks helps because:

1. Hooks run externally to model control
2. Messages are highly visible (not buried in instructions)
3. Explicit "DO NOT STOP" is clearer than implicit rules
4. Model sees exact next task to work on
5. Enforcement is automatic, not optional

**Why post-commit notification instead of pre-commit blocking?**

Blocking commits punishes the USER when model stops:
- Work not saved = higher risk of data loss
- Environment crashes cause more damage
- User gets no partial results

Post-commit notification preserves work while providing same pressure:
- Work saved = Rule 8 compliance
- User gets partial results even if model stops
- Message is just as visible as block error
- Model still sees explicit instruction to continue

Both approaches are ultimately advisory (neither can FORCE continuation). The difference is blocking harms the user while notification preserves progress.

This is **technical enforcement with progress preservation**.
