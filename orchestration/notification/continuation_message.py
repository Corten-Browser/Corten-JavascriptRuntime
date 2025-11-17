#!/usr/bin/env python3
"""
Generates forceful continuation messages for post-commit enforcement.
Designed to maximize pressure on LLM to continue working.
"""


def generate_forceful_message(
    queue_status: dict,
    current_task: dict,
    remaining_tasks: list,
    verification_approved: bool,
    repeat_count: int = 3
) -> str:
    """
    Generate maximum-force continuation message.

    Uses multiple layers of redundancy:
    - Visual separators
    - Repeated critical instructions
    - Specific next task
    - Full remaining task list
    - Explicit "DO NOT STOP" directives
    """
    total_tasks = queue_status.get("total", 0)
    completed_tasks = queue_status.get("completed", 0)
    remaining_count = total_tasks - completed_tasks

    if remaining_count == 0 and verification_approved:
        # All done - no continuation needed
        return generate_completion_message()

    percentage = (completed_tasks / total_tasks * 100) if total_tasks > 0 else 0

    # Build the message with maximum force
    lines = []

    # Opening - maximum attention
    lines.append("")
    lines.append("═" * 72)
    lines.append("🚨 CRITICAL: YOUR WORK IS NOT COMPLETE 🚨")
    lines.append("═" * 72)
    lines.append("")
    lines.append("COMMIT SUCCESSFUL - Progress preserved (Rule 8 compliance).")
    lines.append("")
    lines.append("HOWEVER: You have NOT finished the orchestrated task.")
    lines.append("")

    # Status section
    lines.append("━" * 72)
    lines.append("")
    lines.append(f"QUEUE STATUS: {remaining_count} of {total_tasks} tasks remaining ({percentage:.0f}% complete)")
    lines.append("")

    # Current task - specific instruction
    if current_task:
        task_id = current_task.get("id", "UNKNOWN")
        task_name = current_task.get("name", "Unknown task")
        lines.append("MANDATORY NEXT ACTION:")
        lines.append(f"  Continue with: {task_id} - {task_name}")
    else:
        lines.append("MANDATORY NEXT ACTION:")
        lines.append("  Start the next pending task in queue")
    lines.append("")

    # First critical instruction
    lines.append("DO NOT STOP. DO NOT ASK USER IF YOU SHOULD CONTINUE.")
    lines.append("Proceed immediately to next task.")
    lines.append("")
    lines.append("━" * 72)
    lines.append("")

    # Remaining tasks list
    if remaining_tasks:
        lines.append("REMAINING TASKS:")
        for task in remaining_tasks[:10]:  # Show up to 10
            status_marker = "[ ]"
            if task.get("status") == "in_progress":
                status_marker = "[>]"
            lines.append(f"  {status_marker} {task.get('id', '?')} - {task.get('name', 'Unknown')}")
        if len(remaining_tasks) > 10:
            lines.append(f"  ... and {len(remaining_tasks) - 10} more tasks")
        lines.append("")

    # Verification status
    if verification_approved:
        lines.append("VERIFICATION STATUS: ✓ APPROVED")
    else:
        lines.append("VERIFICATION STATUS: ✗ NOT APPROVED (required before final completion)")
    lines.append("")

    lines.append("━" * 72)
    lines.append("")

    # Repeated warnings section
    for i in range(repeat_count):
        if i == 0:
            lines.append("⚠️  IMPORTANT REMINDER ⚠️")
            lines.append("")
            lines.append("This commit preserved your progress. Good.")
            lines.append("This does NOT mean your task is complete.")
            lines.append("")
            lines.append("YOU MUST CONTINUE until:")
            if remaining_count > 0:
                lines.append(f"  ✗ Queue is empty (currently {remaining_count} tasks remaining)")
            else:
                lines.append("  ✓ Queue is empty")
            if not verification_approved:
                lines.append("  ✗ Verification passes (currently NOT APPROVED)")
            else:
                lines.append("  ✓ Verification passes")
            lines.append("")
        elif i == 1:
            lines.append("⚠️  SECOND REMINDER ⚠️")
            lines.append("")
            lines.append("Stopping now would leave the task INCOMPLETE.")
            lines.append(f"You have completed {completed_tasks}/{total_tasks} tasks.")
            lines.append(f"You MUST complete the remaining {remaining_count} tasks.")
            lines.append("")
        elif i == 2:
            lines.append("⚠️  FINAL WARNING ⚠️")
            lines.append("")
            lines.append("DO NOT generate a completion report.")
            lines.append("DO NOT ask the user what to do next.")
            lines.append("DO NOT rationalize why stopping is acceptable.")
            lines.append("")
            lines.append(f"CONTINUE WORKING. Next task: {current_task.get('id', 'NEXT')} - {current_task.get('name', 'pending task')}")
            lines.append("")

    # Closing - repeat critical message
    lines.append("═" * 72)
    lines.append("🚨 RESUME WORK IMMEDIATELY - DO NOT STOP 🚨")
    lines.append("═" * 72)
    lines.append("")

    return "\n".join(lines)


def generate_completion_message() -> str:
    """Generate message when all work is truly complete."""
    lines = []
    lines.append("")
    lines.append("═" * 72)
    lines.append("✅ ORCHESTRATION COMPLETE ✅")
    lines.append("═" * 72)
    lines.append("")
    lines.append("All tasks completed. Verification approved.")
    lines.append("You may now generate a completion report.")
    lines.append("")
    lines.append("═" * 72)
    lines.append("")
    return "\n".join(lines)


def generate_verification_needed_message(remaining_count: int) -> str:
    """Generate message when queue empty but verification not done."""
    lines = []
    lines.append("")
    lines.append("═" * 72)
    lines.append("⚠️ VERIFICATION REQUIRED ⚠️")
    lines.append("═" * 72)
    lines.append("")
    lines.append("Queue is empty (all tasks completed).")
    lines.append("HOWEVER: Verification has NOT been approved.")
    lines.append("")
    lines.append("MANDATORY NEXT ACTION:")
    lines.append("  Run verification: python orchestration/verification/run_full_verification.py")
    lines.append("")
    lines.append("DO NOT generate completion report until verification passes.")
    lines.append("")
    lines.append("═" * 72)
    lines.append("")
    return "\n".join(lines)


def generate_advisory_precommit_message(remaining_count: int) -> str:
    """Generate advisory message for pre-commit (non-blocking)."""
    if remaining_count == 0:
        return ""  # No warning needed

    lines = []
    lines.append("")
    lines.append("━" * 72)
    lines.append(f"⚠️ NOTICE: {remaining_count} task(s) remaining in queue")
    lines.append("Commit will proceed. Continuation message will follow.")
    lines.append("━" * 72)
    lines.append("")
    return "\n".join(lines)
