---
description: "Internal: Resume workflow preparation (called by /orchestrate)"
---

# Internal: Resume Workflow Preparation

**This is an internal command called by `/orchestrate` via subagent.**
**Do not invoke directly unless you understand the orchestration workflow.**

## Purpose

Prepare for resuming interrupted orchestration by loading checkpoint or discovering
state from the project. This command implements Phase -1 of the orchestration workflow.

## Prerequisites

This command expects:
- Project has been partially orchestrated (some phases complete)
- Either checkpoint file exists OR project state can be discovered
- `orchestration/` directory exists

## When This Command Runs

This command is called by `/orchestrate` when:
- User specifies `--resume` flag
- Orchestration was previously interrupted

---

## Step 1: Check for Resume Context

```python
from pathlib import Path
import subprocess
import json

context_file = Path("orchestration-resume-context.json")

if context_file.exists():
    print("✅ Resume context found")
    print("   Loading existing context...")
    print()
else:
    print("⚠️  No resume context found")
    print("   Running state analysis to generate context...")
    print()

    # Run resume_orchestration.py to create context
    result = subprocess.run(
        ["python", "orchestration/checkpoints/resume_orchestration.py", "."],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print("❌ Resume analysis failed")
        print(result.stderr)
        print()
        print("Cannot resume. Please check orchestration state.")
        # Report failure back to orchestrator
        exit(1)

    print("✅ Resume context generated")
    print()
```

---

## Step 2: Load Resume Context

```python
try:
    with open(context_file) as f:
        resume_ctx = json.load(f)
except json.JSONDecodeError:
    print("❌ Resume context file is corrupted")
    print(f"   Location: {context_file}")
    print()
    print("Options:")
    print(f"  1. Delete and regenerate: rm {context_file} && /orchestrate --resume")
    print("  2. Manually fix the JSON")
    # Report failure back to orchestrator
    exit(1)

# Extract resume information
source = resume_ctx["source"]  # "checkpoint" or "discovery"
current_phase = resume_ctx["current_phase"]
original_request = resume_ctx["original_request"]
components = resume_ctx["components"]
blocking_issues = resume_ctx.get("blocking_issues", [])
phases = resume_ctx.get("phases", [])
```

---

## Step 3: Display Resume Summary

```python
print()
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("  RESUME ORCHESTRATION")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print()
print(f"Source: {source.upper()}")
if source == "checkpoint":
    print("  (Loaded from saved checkpoint)")
else:
    print("  (Discovered from project state)")
print()
print("Original Request:")
user_prompt = original_request.get('user_prompt', 'Unknown')
print(f"  {user_prompt}")
print()
print(f"Current Phase: {current_phase}")
print()

# Show completed phases
if phases:
    completed = [p for p in phases if p.get("status") == "completed"]
    incomplete = [p for p in phases if p.get("status") != "completed"]

    if completed:
        print("Completed Phases:")
        for phase in completed:
            print(f"  ✅ Phase {phase['phase_number']}: {phase['name']}")

    if incomplete:
        print()
        print("Remaining Phases:")
        for phase in incomplete:
            status_icon = "🔄" if phase.get("status") == "in_progress" else "⏸️"
            print(f"  {status_icon} Phase {phase['phase_number']}: {phase['name']}")

print()
if components:
    print(f"Components: {', '.join(components)}")
    print()

# Show blocking issues
if blocking_issues:
    print("⚠️  Blocking Issues Detected:")
    for issue in blocking_issues:
        print(f"  • {issue}")
    print()
    print("These issues may need to be resolved before proceeding.")
    print()

print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print()
```

---

## Step 4: Determine Execution Path

```python
# Check if --level flag also specified (passed from orchestrator)
level_override = None  # This will be passed from /orchestrate if specified

# Determine which phase to resume from
if level_override:
    print(f"⚠️  Level override detected: --level={level_override}")
    print(f"   Ignoring resume context phase ({current_phase})")
    print(f"   Will execute {level_override} orchestration")
    print()

    # Map level to execution
    if level_override == "direct":
        recommended_action = "Execute Level 1 (Direct)"
        resume_phase = None  # Level 1 doesn't use phases
    elif level_override == "feature":
        recommended_action = "Execute Level 2 (Feature)"
        resume_phase = None  # Level 2 doesn't use phases
    elif level_override == "full":
        recommended_action = "Execute Level 3 from Phase 1"
        resume_phase = 1
else:
    print(f"Resuming from Phase {current_phase}...")
    recommended_action = f"Resume Level 3 from Phase {current_phase}"
    resume_phase = current_phase
    print()
```

---

## Step 5: Initialize Checkpoint System

```python
# If we're resuming Level 3 (Full Orchestration), verify checkpoint system
if resume_phase is not None and resume_phase >= 1 and resume_phase <= 6:
    checkpoint_file = Path("orchestration/checkpoints/orchestration_checkpoint.json")

    if checkpoint_file.exists():
        print("✅ Checkpoint file found")
        print(f"   Location: {checkpoint_file}")
    else:
        print("⚠️  No checkpoint file found")
        print("   Will rely on discovered state")

    print()
    print("Checkpoint system ready for Level 3 orchestration")
    print()
```

---

## Step 6: Route to Appropriate Phase

```python
# Determine routing based on phase
phase_routing = {
    0: "Phase 0 (Scope Analysis)",
    1: "Phase 1 (Analysis & Architecture)",
    2: "Phase 2 (Component Creation)",
    3: "Phase 3 (Contracts & Setup)",
    4: "Phase 4 (Parallel Development)",
    4.5: "Phase 4.5 (Contract Validation)",
    5: "Phase 5 (Integration Testing)",
    6: "Phase 6 (Completion Verification)"
}

if resume_phase is not None:
    phase_name = phase_routing.get(resume_phase, f"Phase {resume_phase}")
    print(f"Routing to: {phase_name}")
else:
    print("Routing to: Level-specific execution (not Phase-based)")

print()
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print()
```

---

## Resume Execution Notes

**When resuming:**
- Use `original_request` from resume context as the user's request
- Use `components` list from resume context
- Skip phases that are already completed (current_phase > X)
- Execute current phase and all subsequent phases
- Continue checkpoint updates as phases complete

**Phase Skipping Pattern:**

Each phase (1-6) should check at start:

```python
# If resuming and current_phase > X, skip this phase
if 'resuming' in globals() and resuming and current_phase > X:
    print(f"⏭️  Skipping Phase {X} (already completed)")
    # Continue to next phase
```

**Error Handling:**

If resume fails at any point, preserve state and allow user to retry:
- Checkpoint file preserved
- Resume context preserved
- User can fix issues and run `/orchestrate --resume` again

---

## Reporting Back to Orchestrator

**After completing all steps, report back with this structure:**

```json
{
    "resume_prepared": true,
    "source": "checkpoint",
    "current_phase": 3,
    "original_request": {
        "user_prompt": "implement payment-system.md",
        "timestamp": "2025-11-21T10:30:00Z"
    },
    "components": ["auth_service", "payment_api", "database_manager"],
    "completed_phases": [1, 2],
    "remaining_phases": [3, 4, 4.5, 5, 6],
    "blocking_issues": [],
    "level_override": null,
    "recommended_action": "Resume Level 3 from Phase 3",
    "resume_phase": 3,
    "checkpoint_available": true
}
```

**If resume preparation failed:**
```json
{
    "resume_prepared": false,
    "error": "Resume context file corrupted",
    "source": null,
    "current_phase": null,
    "original_request": null,
    "components": [],
    "completed_phases": [],
    "remaining_phases": [],
    "blocking_issues": ["Cannot parse orchestration-resume-context.json"],
    "level_override": null,
    "recommended_action": "Delete context file and retry: rm orchestration-resume-context.json",
    "resume_phase": null,
    "checkpoint_available": false
}
```

**If blocking issues detected:**
```json
{
    "resume_prepared": true,
    "source": "discovery",
    "current_phase": 5,
    "original_request": {
        "user_prompt": "implement audio-analyzer spec"
    },
    "components": ["audio_loader", "analyzer", "cli_interface"],
    "completed_phases": [1, 2, 3, 4, 4.5],
    "remaining_phases": [5, 6],
    "blocking_issues": [
        "Integration tests failing (3/8 pass)",
        "API mismatch in audio_loader.load_audio()"
    ],
    "level_override": null,
    "recommended_action": "Resume Level 3 from Phase 5 - address blocking issues",
    "resume_phase": 5,
    "checkpoint_available": true
}
```

**CRITICAL**: Include all blocking issues so the orchestrator can handle them appropriately.
