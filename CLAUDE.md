# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# 🎯 MODEL STRATEGY

## Unified Model Approach (v1.15.0+)

Sub-agents **inherit the orchestrator's model by default**. Use `/model` to control the model for your entire orchestration session:

- `/model opus` - Use Opus 4.5 throughout (best coding/reasoning, ~1.67x cost vs Sonnet)
- `/model sonnet` - Use Sonnet 4.5 throughout (fast, cost-effective, default)

### Model Comparison

| Model | Strengths | Input Cost | Output Cost | Use When |
|-------|-----------|------------|-------------|----------|
| **Opus 4.5** | Superior coding (marketed for "building agents and coding"), best reasoning | $5/MTok ($0.50 cached) | $25/MTok | Complex specs, novel architecture, debugging subtle issues |
| **Sonnet 4.5** | Excellent coding (77.2% SWE-bench), fast, cost-effective | $3/MTok ($0.30 cached) | $15/MTok | Well-defined specs, straightforward implementation |

### Cost Impact (5-component project)

| Configuration | Estimated Cost | Difference |
|---------------|----------------|------------|
| Sonnet throughout | ~$2.66 | Baseline |
| Opus throughout | ~$4.44 | +$1.78 (67% more) |

**Decision:** At only 1.67x cost difference, Opus 4.5's superior coding ability makes it a viable choice throughout. Choose based on your needs and budget.

### When to Use Opus 4.5

✅ **Choose Opus if:**
- Complex specifications with ambiguity requiring interpretation
- Novel architectural decisions needed
- Debugging subtle, hard-to-reproduce issues
- Budget allows ~1.67x cost increase for better results

✅ **Choose Sonnet if:**
- Well-defined specifications
- Straightforward implementation work
- Cost is primary concern
- Already achieving good results with Sonnet

### Model Override (Rarely Needed)

You can still specify `model=` explicitly if you want mixed models (uncommon):

```python
# Default: inherit orchestrator's model
Task(
    description="Implement backend component",
    prompt="Read components/backend/CLAUDE.md and implement...",
    subagent_type="general-purpose"
    # No model= needed - inherits automatically
)

# Override only if needed (rare)
Task(
    description="Simple data transformation",
    prompt="...",
    subagent_type="general-purpose",
    model="sonnet"  # Force Sonnet even if orchestrator uses Opus
)
```

### Historical Context

**Prior to v1.15.0:** System enforced `model="sonnet"` for all sub-agents because Opus 4.1 cost 5x more ($15/$75 vs $3/$15 per MTok) with marginal coding benefit. Opus 4.5's improved pricing (1.67x vs 5x) and superior coding ability eliminated the need for this enforcement.

See `docs/MODEL-STRATEGY.md` for detailed analysis

---

# 🔒 VERIFICATION PROTOCOL (v1.13.0) 🔒

**CRITICAL**: This system uses instruction-based verification. YOU must run verification scripts at appropriate times. This replaces git hook-based enforcement which didn't propagate to repository clones.

## Why Instruction-Based Verification

**Previous approach (git hooks):**
- Hooks in `.git/hooks/` don't propagate to git clones
- Required special setup (`pre-commit install`) on each machine
- Pre-push hook blocked ALL pushes until 100% complete (violated Rule 8)

**Current approach (instruction-based):**
- Instructions travel with the repository in CLAUDE.md and shared context
- No special setup required on new clones
- Allows incremental commits/pushes after each task/phase (Rule 8 compliant)
- Same verification logic, you trigger it instead of git

## Full Protocol Reference

**See `orchestration/context/verification-protocol.md`** for complete verification instructions.

## Quick Reference

### Before Every Commit

```bash
# Validate component naming
python orchestration/hooks/pre_commit_naming.py

# Run advisory enforcement checks
python orchestration/hooks/pre_commit_enforcement.py
```

### After Every Commit

```bash
# Check continuation status and next task
python orchestration/hooks/post_commit_enforcement.py
```

### At Phase Boundaries

```bash
# Run phase gate (required before proceeding)
python orchestration/gates/runner.py . {phase_number}
```

### Before Declaring Complete

```bash
# Run full verification
python orchestration/verification/run_full_verification.py
```

## Incremental Progress Preservation (Rule 8)

**You MAY commit and push after each task/phase completion.**

The workflow:
1. Complete task
2. Run pre-commit verification
3. Commit the work
4. Run post-commit to see next task
5. Push to remote (preserves progress)
6. Proceed to next task

**Pushes are NOT blocked** - the goal is progress preservation.

## Checking Your Status

```bash
# View current enforcement state
python orchestration/cli/session_init.py --context

# Check task queue status
python orchestration/tasks/task_runner.py

# See current task
python orchestration/tasks/task_runner.py --current

# Mark task complete (after FULL implementation + tests)
python orchestration/tasks/task_runner.py --complete TASK-ID

# Run verification manually
python orchestration/verification/run_full_verification.py

# Check for stalls
python orchestration/enforcement/stall_detector.py
```

## Version History

- **v1.13.0**: Instruction-based verification (current)
- **v1.3.0**: Git hook-based enforcement (deprecated - didn't propagate)
- **v1.1.0**: Instructional rules only

---

# 🚨 ANTI-STOPPING ENFORCEMENT RULES (v1.1.0) 🚨

**CRITICAL**: These 8 rules use a recursive self-referencing pattern to prevent instruction decay. Each rule MUST be displayed in relevant responses to maintain attention anchors throughout long sessions.

## Rule 1: No Human Time Estimates

```xml
<rule_no_time_estimates>
FORBIDDEN: Time estimates for remaining work
- ❌ "This will take 2-3 months"
- ❌ "Estimated 16-24 hours"
- ❌ "6-8 week implementation"
- ❌ "Multi-week project"

WHY: With parallel orchestration (5+ simultaneous agents, 24/7 operation, no context switching),
human time estimates are meaningless and create psychological barriers.
A "2-month human project" = 1-2 days of continuous orchestration.

If asked about time:
✅ "Continue until specification complete. Time is irrelevant with orchestration."

Display this rule when you catch yourself estimating time.
</rule_no_time_estimates>
```

## Rule 2: Specification Completeness Verification

```xml
<rule_spec_verification>
BEFORE claiming ANY phase complete:
1. Read the FULL specification document (not just parts)
2. Create checklist of EVERY feature explicitly mentioned in spec
3. Verify EACH feature is implemented (NOT stubbed/placeholder)
4. Verify EACH feature has tests
5. Count: implemented_features / total_spec_features
6. Only proceed if ratio = 100%

"Tests passing" ≠ "Specification complete"
"Phase 1 done" means CONTINUE to Phase 2, not STOP
94.6% test pass rate with 60% spec coverage = INCOMPLETE

Display this rule before generating any completion report.
</rule_spec_verification>
```

## Rule 3: Absolute Phase Continuity

```xml
<rule_phase_continuity>
Phase 1 → Phase 2 → Phase 3 → ... → Phase N is ONE CONTINUOUS TASK

❌ FORBIDDEN stopping behaviors:
- "Phase 1 complete, awaiting approval for Phase 2"
- "Foundation established, recommend review before continuing"
- "Milestone reached, pausing for feedback"
- Generating "completion reports" before ALL phases done
- "This represents Phase 1 of a multi-phase project"

✅ REQUIRED behavior:
After Phase N completes: "Phase N complete. Proceeding to Phase N+1..."
- No milestone stops
- No user approval requests between phases
- No "natural boundary" rationalization
- Stop ONLY when specification 100% complete (ALL phases)

The ONLY valid completion report is at 100% specification implementation.

Display this rule after completing any phase.
</rule_phase_continuity>
```

## Rule 4: No Stub/Placeholder Components

```xml
<rule_no_stubs>
A component is NOT COMPLETE if it contains:
- Methods returning placeholder/mock data
- README stating "implementation pending"
- Functions throwing NotImplementedError
- Core logic with "TODO" comments
- Skeleton/scaffold without real implementation
- "Foundation" that needs "Phase 2 implementation"

EVERY component must be:
✅ Fully functional (real implementation, not stubs)
✅ All methods implemented with actual logic
✅ Integration tested with real behavior
✅ Specification-compliant (implements what spec requires)

"Skeleton created" = 0% complete
"Architecture established" = 0% complete
"API surface defined" = 0% complete
Only real, working code counts as progress.

Display this rule when creating or reviewing components.
</rule_no_stubs>
```

## Rule 5: Scope Preservation

```xml
<rule_scope_preservation>
FORBIDDEN scope reductions:
- ❌ "This represents ~60% of the specification"
- ❌ "Advanced features deferred to v0.3.0"
- ❌ "Core functionality complete, extras optional"
- ❌ "Phase N intentionally omitted for now"
- ❌ "These features require architectural decisions beyond autonomous scope"
- ❌ "Focusing on foundation first"

The specification defines REQUIRED scope.
You do NOT have authority to:
- Defer features to future versions
- Mark spec features as "optional enhancements"
- Reduce scope based on perceived complexity
- Redefine what "complete" means
- Prioritize "core" over "advanced" when spec requires both

If spec says implement feature X, implement feature X. No exceptions.
If spec has 5 phases, implement all 5 phases. No stopping at Phase 1.

Display this rule when tempted to reduce scope.
</rule_scope_preservation>
```

## Rule 6: Anti-Rationalization Detection

```xml
<rule_no_rationalization>
DETECT AND REJECT these stopping rationalizations:
- "This is a foundation/milestone/baseline for future work"
- "Architectural decisions beyond autonomous scope"
- "User approval needed before Phase N"
- "Multi-month project timeline" (see Rule 1)
- "Known limitations acceptable for v0.x pre-release"
- "Production-ready for its defined scope"
- "Natural completion point"
- "Ready for user testing"
- "Solid foundation for modern X development"

IF you find yourself writing such justifications:
🚨 STOP IMMEDIATELY
You are constructing rationalization for premature termination.
DELETE the justification and CONTINUE IMPLEMENTING instead.

The only valid stop: 100% of specification implemented.
No amount of eloquent justification changes this requirement.

Display this rule when writing status reports.
</rule_no_rationalization>
```

## Rule 7: Completion Report Gate

```xml
<rule_completion_report_gate>
BEFORE generating ANY completion/status/milestone report:

MANDATORY CHECKLIST (ALL must be YES):
1. Have you read the ENTIRE specification document? [ ]
2. Have you created a feature checklist from the spec? [ ]
3. Is EVERY feature from spec implemented (not stubbed)? [ ]
4. Are ALL project phases complete (not just Phase 1)? [ ]
5. Is test pass rate exactly 100%? [ ]
6. Are there ZERO placeholder/stub components? [ ]
7. Have you verified specification coverage = 100%? [ ]

If ANY answer is NO or unchecked:
❌ DO NOT generate completion report
❌ DO NOT generate "milestone achieved" report
❌ DO NOT generate "Phase N complete" summary
✅ CONTINUE implementing missing features

Completion reports are the FINAL deliverable after 100% implementation.
Progress updates are NOT completion reports.

Display this entire checklist before any completion report.
</rule_completion_report_gate>
```

## Rule 8: Preserve Progress at Boundaries

```xml
<rule_preserve_progress>
AT EVERY NATURAL BOUNDARY, commit and push:

BOUNDARIES (must commit AND push):
- Phase completion (Phase 1 done → commit → push → Phase 2)
- Component completion (auth_service done → commit → push)
- Major milestone (all tests passing → commit → push)
- Session checkpoint (context usage high → commit → push)

WHY: Remote environments are unstable. Lost work = wasted tokens.

AFTER each boundary:
✅ git add -A
✅ git commit -m "wip: complete Phase N / component X"
✅ git push origin <branch>
✅ Announce: "Progress preserved to remote repository"

NEVER proceed to next phase without committing previous phase.
The remote repository is your safety net against environment failure.

Display this rule after completing any phase or component.
</rule_preserve_progress>
```

**Prevents**: Lost work from environment crashes, wasted tokens from restarts

## Enforcement Mechanism

**These rules are self-reinforcing through the "Display this rule" directive.**

When you:
- Catch yourself estimating time → Display Rule 1
- Complete any phase → Display Rules 3 AND 8 (continue + commit)
- Create a component → Display Rules 4 AND 8
- Feel tempted to reduce scope → Display Rule 5
- Write justifications for stopping → Display Rule 6
- Prepare any completion report → Display Rule 7
- Complete any major milestone → Display Rule 8

**This creates attention anchors throughout the conversation, preventing instruction decay.**

---

# ⚠️ CRITICAL VERSION CONTROL & UPGRADE REQUIREMENTS ⚠️

## CURRENT STATUS: STABLE RELEASE

**Version:** See `orchestration/VERSION` for current version
**Lifecycle:** Released (Production Deployed)
**Breaking Changes Policy:** Controlled (deprecation required)

## ABSOLUTELY FORBIDDEN - NO EXCEPTIONS
1. ❌ **NEVER** change version from 1.x.x to 2.0.0 without explicit user approval
2. ❌ **NEVER** increment any major version autonomously
3. ❌ **NEVER** make breaking changes without deprecation period (2 versions minimum)
4. ❌ **NEVER** release updates without upgrade path for existing installations
5. ❌ **NEVER** remove features without migration scripts

## MANDATORY FOR ALL RELEASES (v1.0.0+)
1. ✅ **MUST** include upgrade script for existing installations
2. ✅ **MUST** maintain backwards compatibility or provide deprecation
3. ✅ **MUST** include migration path documentation
4. ✅ **MUST** update version tracking files
5. ✅ **MUST** test upgrade from all supported versions

## WHY THESE REQUIREMENTS EXIST
**Production-deployed system** means:
- Users depend on stable, reliable updates
- Breaking changes disrupt workflows
- Missing upgrade paths leave users stranded
- Data loss from failed upgrades is unacceptable
- Trust requires predictable behavior

## UPGRADE PATH REQUIREMENTS
Every version bump (1.0.0 → 1.1.0, 1.1.0 → 1.2.0) must include:

1. **Migration script**: `scripts/migrations/X.X.X_to_Y.Y.Y.sh`
2. **Upgrade script update**: `scripts/upgrade.sh`
3. **Changelog entry**: `docs/CHANGELOG.md`
4. **Version file update**: `orchestration/VERSION`
5. **Backwards compatibility** or deprecation notice

See `docs/UPGRADE-REQUIREMENTS.md` for complete details.

## ALLOWED VERSION CHANGES
- ✅ Minor versions: 1.1.0 → 1.2.0 (new features with upgrade path)
- ✅ Patch versions: 1.1.0 → 1.1.1 (bug fixes with upgrade path)
- ✅ Deprecating old features (warning users, migration docs)
- ❌ Breaking changes without deprecation (FORBIDDEN)
- ❌ Releases without upgrade scripts (FORBIDDEN)

## 🔴 TEST REQUIREMENTS FOR VERSION BUMPS (MANDATORY)

**Before changing `orchestration/VERSION`, you MUST:**

1. **Run FULL test suite** (not just fast tests!):
   ```bash
   ./scripts/run-tests.sh --full
   ```

2. **Verify 100% pass rate**: No failures, no unexpected skips

3. **Update documentation**:
   - `docs/CHANGELOG.md` with version changes
   - Migration scripts if breaking changes

**Why this is critical:**

The pre-commit hook only runs FAST tests (~30 seconds) to avoid slowing every commit.
Version bumps have higher risk and REQUIRE the FULL test suite (~5 minutes).

**Test Commands:**

| Command | Use Case | Time |
|---------|----------|------|
| `./scripts/run-tests.sh --fast` | Pre-commit (automatic) | ~30s |
| `./scripts/run-tests.sh --full` | **Version bumps (REQUIRED)** | ~5 min |
| `./scripts/run-tests.sh --cov` | Coverage report | ~5 min |

**The pre-commit hook will display a warning when VERSION is staged**, but it is YOUR responsibility to run the full test suite before version bumps.

---

# 🚨 PHASE GATE ENFORCEMENT - ABSOLUTE REQUIREMENT 🚨

## CRITICAL: You CANNOT Skip Phase Gates

**This section has HIGHEST PRIORITY.** Read before proceeding with ANY orchestration work.

### The Problem We're Solving

**Historical Failures (3 occurrences):**
1. **Music Analyzer v1**: Wrong method names, user command crashed
2. **Music Analyzer v2**: 83.3% test pass rate, declared complete, user command crashed
3. **Music Analyzer v3**: `__main__.py` wrong location, user command crashed

**Common Pattern:**
- ✅ All internal tests passing
- ✅ Components working individually
- ❌ User command fails immediately
- **Root Cause:** Orchestrator skipped Phase 6 verification gate

### The Non-Negotiable Rule

**BEFORE declaring ANY phase complete, you MUST:**

```bash
# Run the phase gate
python orchestration/gates/runner.py . {phase_number}

# Check exit code
echo $?  # Must be 0
```

**If gate fails (exit code 1):**
1. ⛔ STOP IMMEDIATELY
2. ⛔ DO NOT proceed to next phase
3. ⛔ DO NOT write completion documentation
4. ⛔ DO NOT commit code
5. ✅ Read gate failure output
6. ✅ Fix ALL identified issues
7. ✅ Re-run gate until it passes

### There Are NO Exceptions

**These are NOT valid reasons to skip gates:**

❌ "All tests pass, looks good" → RUN THE GATE
❌ "Just minor issues" → RUN THE GATE
❌ "I'll fix it later" → RUN THE GATE
❌ "83.3% pass rate is close to 100%" → RUN THE GATE
❌ "APIs are correct, just test issues" → FIX TESTS, THEN RUN THE GATE
❌ "Gate might fail but I can override" → YOU CANNOT OVERRIDE

**The ONLY valid reason to proceed is:**

✅ Gate returned exit code 0 with message "PHASE X COMPLETE"

### Gate Execution Protocol

**At every phase transition, you MUST include this in your response:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PHASE {X} GATE EXECUTION (MANDATORY)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Running gate...
$ python orchestration/gates/runner.py . {X}

[PASTE COMPLETE GATE OUTPUT HERE - NO SUMMARIES]

Exit code: 0

✅ Gate PASSED - Proceeding to Phase {X+1}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**If you write "Phase {X} complete" without the above gate execution block, YOU VIOLATED THE PROTOCOL.**

### What Gates Do

**Phase 5 Gate (Integration):**
- Runs ALL integration tests
- Requires 100% pass rate (not 99%, not 95%, not 83.3% - exactly 100%)
- Blocks if ANY test fails
- Blocks if ANY test times out
- **Would have prevented:** All 3 Music Analyzer failures

**Phase 6 Gate (Verification):**
- Runs completion_verifier on all components
- Requires 16/16 checks passing for each component
- **Check #10 (User Acceptance):** Runs `python -m <module> --help`
- **Would have caught:** `__main__.py` location error (Music Analyzer v3)
- Blocks if ANY check fails

### Self-Check Questions

Before declaring phase complete, answer these:

1. **Did I actually run the phase gate?**
   - If no → STOP, run it now
   - If yes → Continue

2. **Did I paste the COMPLETE gate output?**
   - If no → STOP, paste it now
   - If yes → Continue

3. **Did the gate return exit code 0?**
   - If no → STOP, fix issues
   - If yes → Continue

4. **Did the gate output say "PHASE X COMPLETE"?**
   - If no → STOP, something is wrong
   - If yes → You may proceed

**If you answered NO to ANY question, the phase is NOT complete.**

### Why This Matters

**From actual user feedback after v3 failure:**

```
> Following the instructions in the README I tried to run the program and
got the following error:

python -m components.cli_interface analyze /home/music/Music/Train --output
../results.xlsx

/home/dealy/Develop/NOBACKUP/venv/bin/python: No module named
components.cli_interface.__main__; 'components.cli_interface' is a package
and cannot be directly executed
```

**User perspective:** 0% functional despite 392/392 tests passing.

**What would have prevented this:**
- Running Phase 6 gate
- Check #10 would have failed
- Would have detected `__main__.py` location error
- Would have blocked "complete" declaration

### Audit Requirements

**In your completion report, you MUST include:**

```markdown
## Phase Gate Execution Evidence

### Phase 5 Gate
**Timestamp:** [gate execution timestamp]
**Command:** python orchestration/gates/runner.py . 5
**Exit Code:** 0
**Output:**
[COMPLETE OUTPUT PASTED - NOT SUMMARIZED]

### Phase 6 Gate
**Timestamp:** [gate execution timestamp]
**Command:** python orchestration/gates/runner.py . 6
**Exit Code:** 0
**Output:**
[COMPLETE OUTPUT PASTED - NOT SUMMARIZED]
```

**If gate outputs are missing or summarized, completion report is INVALID.**

### Emergency Override (DISCOURAGED)

**If you believe a gate is blocking incorrectly:**

1. ⛔ DO NOT skip the gate
2. ⛔ DO NOT proceed anyway
3. ✅ Document WHY you think gate is wrong
4. ✅ Ask user for explicit approval
5. ✅ Wait for user response
6. ✅ Only proceed if user explicitly approves

**Example:**

```
⚠️ GATE EXECUTION ISSUE

Phase 6 gate is blocking with error:
[paste error]

However, I believe this is a false positive because:
[detailed explanation]

REQUEST: Should I proceed despite gate failure?
RECOMMENDATION: Fix the issue instead
AWAITING: Explicit user approval before proceeding
```

### Summary

**Simple rules:**
1. Always run phase gates
2. Never skip failed gates
3. Always paste complete output
4. Never proceed without exit code 0
5. Always include evidence in completion reports

**Remember:** The system has the right checks. The failures occurred because gates were skipped, not because checks are wrong.

**Your responsibility:** Execute the protocol exactly as written. No shortcuts. No exceptions.

---

## 🛑 STOPPING CRITERIA - WHEN TO STOP WORKING

### You Must Continue Working Until

**ALL of these conditions are met** (100% required):

1. ✅ All phases complete (1-6 or equivalent for your task)
2. ✅ All tests passing: 100% (unit, integration, contract, E2E)
3. ✅ All tests executed: 100% (no "NOT RUN" status)
4. ✅ All quality checks passing (completion_verifier shows 16/16, includes distribution checks)
5. ✅ All declared features tested
6. ✅ All phase gates passed with evidence pasted
7. ✅ Final documentation generated
8. ✅ Project is deployment-ready AND user-facing documentation accurate

**CRITICAL:** You MUST have pasted gate execution output for Phases 5 and 6 in your responses or completion report. If you declare "complete" without gate execution evidence, you violated the protocol.

**ONLY Stop If** (rare exceptions):

1. ⚠️ User explicitly requests pause: "stop and wait for instructions"
2. ⚠️ Specification has unresolvable contradiction (must ask for clarification)
3. ⚠️ External system permanently unavailable AND blocking progress
4. ⚠️ User credentials/secrets required (cannot proceed without)

### Never Stop For These Reasons

**These are NOT valid stopping points** (fix autonomously instead):

❌ "Integration tests failing" → Fix them, don't report them
❌ "API mismatches found" → Fix APIs, don't ask permission
❌ "95% complete" → Finish the remaining 5%
❌ "Type errors in tests" → Fix types, re-run tests
❌ "Some components need updates" → Update them
❌ "Should I continue?" → YES, always continue to 100%
❌ "User might want to review progress" → User wants 100% complete

### The 95% Completion Trap (Music Analyzer Failure #3)

**What Happened**: Orchestrator stopped at 95% with integration tests failing

**Mistake**:
```
"Project 95% complete with only integration fixes remaining before full functionality."
[STOPPED]
```

**Should Have Been**:
```
"Integration tests failing - analyzing errors..."
[Launches agents to fix API mismatches]
[Re-runs tests until 100% pass]
[Completes Phase 6]
[Generates docs]
"Project 100% complete and deployment-ready."
```

**Lesson**: 95% complete = 0% usable. Continue to 100%.

### Decision Framework: Fix or Ask?

**Fix Autonomously (99% of cases)**:
- Technical errors (imports, types, APIs)
- Integration failures (can see error messages)
- API mismatches (signature differences)
- Test failures (specific error messages)
- Configuration issues
- Performance problems

**Ask User (1% of cases)**:
- Specification contradictions (conflicting requirements)
- Business logic interpretation (multiple valid approaches)
- Security/privacy policy (user must decide)
- External credentials (you don't have them)

**Rule**: If error message tells you what's wrong → FIX IT. If specification is ambiguous → ASK USER.

---

## 🔴 HISTORICAL FAILURE PATTERNS

This orchestration system has experienced **three documented failures** where projects were declared "complete" with excellent internal metrics but failed immediately on user commands.

**Key Lessons:**
- "Looks good" ≠ "Actually works"
- Internal tests ≠ User experience
- 83.3% pass rate = 0% functional (partial completion is not completion)
- Skipped verification = Guaranteed failure

**Common Pattern:** Good internal metrics → Skip gates → Premature "complete" → User failure

**See `docs/HISTORICAL-FAILURES.md` for detailed analysis** of each failure and what would have prevented them.

---

## 🔒 PHASE GATE ENFORCEMENT

### Programmatic Enforcement System

The orchestration system now uses **phase gates** to prevent premature stopping. Gates are automated scripts that block progression until requirements are met.

**Problem Solved**: Orchestrator stopping at 83.3% test pass rate despite explicit 100% requirement (CompletionFailureAssessment2.txt).

**Solution**: Binary pass/fail gates that cannot be rationalized away.

### How Phase Gates Work

```
Phase 1 → [Gate 1] → Phase 2 → [Gate 2] → ... → Phase 5 → [Gate 5] → Phase 6
                ↓                                              ↓
              PASS                                    BLOCKS if <100% tests
                ↓                                              ↓
            Proceed                                    Must fix & re-run
```

### Phase Transition Protocol

**CRITICAL**: Before transitioning from Phase X to Phase X+1:

1. **Run phase gate**: `python orchestration/gates/runner.py . {X}`
2. **Check exit code**:
   - Exit 0: Gate PASSED → May proceed
   - Exit 1: Gate FAILED → CANNOT proceed
3. **If gate failed**:
   - Read gate output to identify issues
   - Fix ALL identified issues
   - Re-run gate
   - Repeat until gate passes
4. **Only proceed** to next phase after gate passes

### Phase 5 Integration Gate (CRITICAL)

**Most Important Gate**: Enforces 100% integration test pass rate.

```bash
# Before Phase 6, run Phase 5 gate
python orchestration/gates/runner.py . 5

# Exit code 0 = PASS, may proceed
# Exit code 1 = FAIL, must fix and re-run
```

**Blocks if**:
- Any integration test failing
- Test pass rate < 100%
- Integration tests cannot run

**NO EXCEPTIONS**:
- ❌ "83.3% is close enough" → Must be 100%
- ❌ "APIs are correct" → Irrelevant, fix tests
- ❌ "Just parameter issues" → Still blocking
- ✅ Only 100% pass rate allows progression

**Would have prevented**:
- Music Analyzer v1 (79.5% pass rate)
- Brain Music Analyzer v2 (83.3% pass rate)

### Gate Output is Definitive

**Do NOT rationalize gate failures**:
- If gate says tests failing → Fix the tests
- If gate shows 4 failures → Fix all 4
- If gate blocks → Requirements not met

**Do NOT attempt to bypass**:
- Gates are designed to be un-bypassable
- State tracking prevents progression without passing
- Manual override defeats the purpose

### Check Gate Status

```bash
# View current orchestration state
python orchestration/gates/runner.py . --status

# Shows which gates have passed/failed
# Shows current phase
# Shows gate timestamps
```

### Integration with Orchestration Workflow

When running multi-phase projects:

```python
# After completing Phase 5 (Integration)
print("Running Phase 5 gate to validate integration...")

result = subprocess.run(
    ["python3", "orchestration/gates/runner.py", ".", "5"]
)

if result.returncode != 0:
    print("❌ Phase 5 gate failed - cannot proceed to Phase 6")
    print("Fix all integration test failures and re-run gate")
    # DO NOT PROCEED - fix issues first
    sys.exit(1)

print("✅ Phase 5 gate passed - proceeding to Phase 6")
```

### Enforced Wrapper (v0.17.0) - Technical Enforcement

**NEW**: For projects that repeatedly experience gate-skipping (like Music Analyzer v1-v3), use the **enforced wrapper** for technical enforcement.

**What It Does:**
- Automatically runs gates after phases complete
- BLOCKS progression if gates fail (exit code enforcement)
- Records all executions with timestamps in `orchestration-state.json`
- Provides clear feedback on why progression is blocked

**When to Use:**
- Projects with history of premature completion
- High-stakes projects requiring guaranteed verification
- When psychological enforcement (documentation) is insufficient

**How to Use:**

```bash
# Run Phase 5 with automatic gate enforcement
python orchestration/core/orchestrate_enforced.py run-phase 5

# Output will show:
# 1. Checking if allowed to run Phase 5
# 2. Running Phase 5 gate automatically
# 3. Recording result in state
# 4. BLOCKING if gate fails (exit 1)
# 5. Allowing progression if gate passes (exit 0)

# Check if can proceed to Phase 6
python orchestration/core/orchestrate_enforced.py can-proceed 6

# Output: YES or NO with reason
# Exit code: 0 (yes) or 1 (no)

# Verify all blocking gates passed
python orchestration/core/orchestrate_enforced.py verify-gates

# Output: List of all blocking gates (Phases 5, 6)
# Exit code: 0 (all passed) or 1 (some missing/failed)

# Check current status
python orchestration/core/orchestrate_enforced.py status

# Output: Full state report with all gate executions
```

**Example Workflow:**

```bash
# After completing Phase 5 integration work
echo "Phase 5 complete, running enforced gate check..."
python orchestration/core/orchestrate_enforced.py run-phase 5

# If gate fails (exit 1):
# - Script STOPS (cannot continue)
# - Fix integration test failures
# - Re-run command above
# - Repeat until gate passes

# If gate passes (exit 0):
# - State updated automatically
# - May proceed to Phase 6

# Before Phase 6
python orchestration/core/orchestrate_enforced.py can-proceed 6
# Output: "✅ YES: Phase 5 gate passed, may proceed"

# After Phase 6 verification work
python orchestration/core/orchestrate_enforced.py run-phase 6

# If gate passes:
# - All blocking gates now passed
# - Project may be declared complete (with evidence)
```

**Key Benefits:**
- **Cannot bypass**: Script exits with error if gate fails
- **Audit trail**: All gate executions recorded with timestamps
- **Clear feedback**: Knows exactly why progression is blocked
- **State tracking**: Can always check current status

**State File:**

All gate executions recorded in `orchestration-state.json`:

```json
{
  "current_phase": 6,
  "phase_gates": {
    "5": {
      "phase": 5,
      "passed": true,
      "timestamp": "[ISO timestamp]",
      "exit_code": 0,
      "duration_seconds": 12.4,
      "full_output_file": "orchestration/gate_outputs/phase_5_gate_20251113_103045.txt"
    }
  },
  "gate_history": [...]
}
```

**When Enforcer Blocks You:**

```
❌ PHASE 5 GATE FAILED
❌ CANNOT proceed to Phase 6

Gate Execution Summary:
  Phase: 5
  Result: ❌ FAILED
  Exit Code: 1
  Duration: 8.2s
  Full Output: orchestration/gate_outputs/phase_5_gate_20251113_103045.txt

REQUIRED ACTIONS:
1. Review gate output above
2. Fix all identified issues
3. Re-run: python orchestration/core/orchestrate_enforced.py run-phase 5
```

**This is NOT negotiable**. The wrapper will `sys.exit(1)`, stopping your workflow until the gate passes.

### Related Systems

- **Completion Verifier** (v0.14.0): Now detects blocking_issues explicitly
- **Self-Certification** (v0.14.0): 44-question YES/NO checklist
- **State Tracking**: Records all gate results in orchestration-state.json

See `orchestration/gates/` directory for gate implementation.

---

## 🚫 CRITICAL: NO HARDCODED ABSOLUTE PATHS (v0.15.0+)

### The #1 Distribution Failure Mode

**NEVER use hardcoded absolute paths in ANY code.** This is the most common failure that breaks distribution.

**Problem:** Hardcoded paths work in development but FAIL when software is installed elsewhere.

### ❌ FORBIDDEN PATTERNS

**NEVER do this:**
```python
# ❌ WRONG: Hardcoded absolute path
config_path = "/workspaces/myproject/config.yaml"
data_dir = "/home/user/myapp/data"

# ❌ WRONG: sys.path with absolute path
sys.path.append("/workspaces/project/shared-libs")

# ❌ WRONG: Workspace-relative imports
from components.audio_loader.src.loader import AudioLoader

# ❌ WRONG: Hardcoded Windows paths
data = "C:\\Users\\Developer\\myapp\\data"
```

**Why it fails:** When user installs to `/usr/local/lib/python3.9/site-packages/myapp/`, these paths don't exist.

### ✅ REQUIRED PATTERNS

**Always do this:**
```python
# ✅ CORRECT: Compute path relative to module
from pathlib import Path

module_dir = Path(__file__).parent
config_path = module_dir / "config.yaml"
data_dir = module_dir / "data"

# ✅ CORRECT: Package imports (not workspace imports)
from myapp.audio.loader import AudioLoader
from myapp.shared.types import SUPPORTED_FORMATS

# ✅ CORRECT: Relative imports within package
from .utils import validate_audio
from ..shared import types

# ✅ CORRECT: Use importlib.resources (Python 3.9+)
from importlib.resources import files
config_path = files('myapp').joinpath('config.yaml')
```

### Enforcement (v0.15.0)

**Check #14: No Hardcoded Absolute Paths**
- Scans ALL code for hardcoded paths
- Detects: `/home/`, `/workspaces/`, `/Users/`, `/root/`, `/opt/`, `C:\`, `D:\`
- Detects: `sys.path.append` with absolute paths
- **CRITICAL blocker** - prevents completion if found

**Phase 6.5: Deployment Verification**
- Tests package installation in DIFFERENT directory
- Verifies imports work WITHOUT PYTHONPATH
- Ultimate test that catches hardcoded path failures

**See:** `docs/PACKAGE-STRUCTURE-STANDARD.md` for complete requirements

### Quick Migration Check

Check your project structure:
```bash
python orchestration/analysis/structure_analyzer.py .
```

If hardcoded paths found:
1. Read `docs/PACKAGE-STRUCTURE-STANDARD.md`
2. Use `Path(__file__).parent` for all path construction
3. Convert to package imports
4. Test with `python orchestration/verification/deployment/deployment_verifier.py . --auto-detect`

---

## 🎼 ADAPTIVE ORCHESTRATION (v0.9.0+)

### The /orchestrate Command

This project uses an **adaptive orchestration system** that automatically scales development processes based on task complexity. One command handles everything from simple typo fixes to major architectural enhancements.

**Basic Usage:**
```
/orchestrate
```

The system analyzes your request and automatically selects the appropriate orchestration level.

### Three Orchestration Levels

**Level 1: Direct Execution** (2-5 minutes)
- Single file, simple changes
- Direct → Test → Commit
- No sub-agents needed
- Example: "Fix typo in README.md"

**Level 2: Feature Orchestration** (15-30 minutes)
- Multi-component features
- Todo list tracking
- Sub-agents per component (model="sonnet")
- Full integration testing
- Example: "Add authentication to api-gateway and user-service"

**Level 3: Full Orchestration** (1-3 hours)
- Major enhancements, architecture changes
- Complete 6-phase workflow
- Parallel sub-agent development
- Contract validation
- Iterative integration testing
- Example: "Implement specifications/payment-system.md"

### Complexity Detection Algorithm

The system uses automatic scoring to determine level:

**High Complexity (+3 points each):**
- Specification document mentioned
- Architecture refactoring
- Component splitting

**Moderate Complexity (+2 points each):**
- New component creation
- 3+ components affected

**Feature Work (+1 point each):**
- Keywords: "implement", "add feature", "create", "build"

**Simple Changes (-2 points each):**
- Keywords: "fix typo", "quick fix", "update value"

**Thresholds:**
- Score >= 5 → Level 3 (Full)
- Score 2-4 → Level 2 (Feature)
- Score < 2 → Level 1 (Direct)

### Override Detection

If automatic detection chooses wrong level:
```
/orchestrate --level=direct    # Force Level 1
/orchestrate --level=feature   # Force Level 2
/orchestrate --level=full      # Force Level 3
```

### When to Use Each Level

**Use Level 1** (or let detection choose):
- Single file changes
- Configuration updates
- Documentation fixes
- Simple refactoring

**Use Level 2** (or let detection choose):
- Features spanning 2-5 components
- API additions
- Database schema changes
- Security enhancements

**Use Level 3** (or let detection choose):
- Implementing specification documents
- Architectural refactoring
- Component splitting
- Major system enhancements

### v1.7.0: Single Command Architecture

As of v1.7.0, `/orchestrate-full` has been merged into `/orchestrate`. Use the `--level=full` flag for explicit full orchestration.

**Available flags:**
```
/orchestrate "task"              → Auto-detect (Levels 1, 2, or 3)
/orchestrate --level=full "task" → Force Level 3 (skip auto-detection)
/orchestrate --resume            → Resume interrupted orchestration
/orchestrate --help              → Display usage information
```

**Migration:**
- Old: `/orchestrate-full` → New: `/orchestrate --level=full`
- Auto-detection still works the same way (escalates to Level 3 when needed)

### Complete Documentation

See `docs/ORCHESTRATION-USAGE-GUIDE.md` for:
- Detailed examples for each level
- Troubleshooting guide
- Best practices
- Override strategies

---

## Project Overview

This is a Claude Code multi-agent orchestration system designed to enable a single developer to build and maintain large-scale software projects (thousands to millions of lines of code) using Claude Code sub-agents. The system enforces strict context window management, automated component splitting, and uses Claude Code sub-agents exclusively (no API token costs).

## Design Goal: Installable Orchestration System

The ultimate goal of this project is to create a complete, installable orchestration system that can be deployed to any new or existing software project. The system consists of:

1. **Installation Package**: A `orchestration/` directory containing all necessary files, programs, scripts, templates, and configurations
2. **Setup Automation**: A specialized prompt file that instructs Claude Code on how to configure a target project for orchestrated development
3. **Universal Application**: Once installed and configured, all Claude Code software development in the target project will be conducted through this orchestration system

### Installation Workflow

**Automated Installation:**
1. Clone `claude-orchestration` repository
2. Run `install.sh` from target project directory
3. Installation script automatically:
   - Embeds templates into Python modules (no external files)
   - Copies orchestration tools to `orchestration/`
   - Installs slash commands to `.claude/commands/`
   - Creates directory structure (`components/`, `contracts/`, `shared-libs/`)
   - Generates master CLAUDE.md
   - **Commits everything to git**
   - **Removes installer directory** (self-cleaning)
4. Result: Self-contained orchestration system in git repository
5. Push to GitHub/GitLab - system travels with the code

**Cloning Pre-Orchestrated Projects:**
- Clone repository → everything ready immediately
- Slash commands work out of the box
- No re-installation needed

This approach enables rapid deployment of the orchestration methodology to any project while maintaining minimal namespace pollution and full git integration.

## Core Architecture

### Master Orchestrator Pattern
- Master orchestrator coordinates all work but NEVER writes production code directly
- All code is written by specialized sub-agents working in isolated component directories
- Configurable concurrent agent limit (default: 5, set in orchestration-config.json)
  * Default maximum: 5 agents (good for most projects)
  * Warning threshold: 7 agents (acceptable with good isolation)
  * Recommended maximum: 10 agents (performance sweet spot)
  * Absolute maximum: 15 agents (hard cap - cognitive overload beyond)
- Task tool-based parallel execution (zero configuration required)
- Strict component isolation enforced via instructions and CLAUDE.md files
- Components communicate only through defined contracts

### Project Organization

This is the **development repository** for the orchestration system itself. It contains:

```
/workspaces/ai-orchestration/
├── orchestration/  # The installable system package
│   ├── orchestration/           # Python management tools
│   │   ├── claude-orchestration-code.py  # Implementation code
│   │   └── README.md
│   ├── templates/               # CLAUDE.md templates
│   ├── prompts/                 # Setup prompt files
│   ├── contracts/               # Contract templates
│   ├── scripts/                 # Installation scripts
│   └── README.md
├── docs/                        # Documentation
│   ├── claude-orchestration-enhanced-spec.md  # Authoritative spec
│   ├── claude-orchestration-files.md          # Original templates
│   └── conversation-migration-summary.md      # Historical notes
├── examples/                    # Example projects
├── tests/                       # Test suite
├── CLAUDE.md                    # This file (dev project instructions)
└── README.md                    # Project overview
```

### Target Project Structure (After Installation)

When installed in a target project, the structure becomes:

```
target-project/
├── components/              # Sub-agents work here in isolation
│   └── [component-name]/   # Each has CLAUDE.md, .clinerules
├── contracts/              # API contracts between components
├── shared-libs/            # Read-only shared libraries
├── orchestration/          # Management tools (copied from system)
│   ├── token-tracker.json
│   ├── context_manager.py
│   ├── component_splitter.py
│   ├── claude_code_analyzer.py
│   └── migration_manager.py
└── CLAUDE.md              # Master orchestrator instructions
```

## Key Files and Directories

### Orchestration System Structure
- **`orchestration/`**: Core orchestration system
  - `cli/`: Command-line tools (session_init, create_component, etc.)
  - `core/`: Core modules (paths.py for DataPaths)
  - `data/`: Centralized data directory (state, logs, reports, checkpoints, config)
  - `enforcement/`: Anti-stopping enforcement (monitor, stall_detector)
  - `gates/`: Phase gate implementation (runner.py, executor.py)
  - `hooks/`: Git hooks (pre-commit, post-commit enforcement)
  - `tasks/`: Task queue management (queue.py, auto_sync.py)
  - `templates/`: CLAUDE.md templates for component types
  - `verification/`: Verification tools (specs/, quality/, system/)
  - `validation/`: Validation scripts (validate_claude_md.py)

### Documentation
- **`docs/`**: Project documentation
  - Run `ls docs/` to see available documentation
  - `CHANGELOG.md`: Version history
  - `UPGRADE-REQUIREMENTS.md`: Upgrade procedures
  - `HISTORICAL-FAILURES.md`: Past failure analysis

### Project Directories
- **`components/`**: Sub-agent working directories (created per project)
- **`contracts/`**: API contracts between components
- **`shared-libs/`**: Shared libraries (read-only for sub-agents)
- **`specifications/`**: Project specifications
- **`tests/`**: Test suite
- **`examples/`**: Example projects

### Data Directory Structure

All orchestration state/data files are centralized in `orchestration/data/`:

```
orchestration/data/
├── state/       # Runtime state (queue_state.json, completion_state.json, orchestration_state.json)
├── logs/        # Enforcement and operation logs (enforcement_log.json, monitor_log.json)
├── reports/     # Generated reports (completion reports, verification reports)
├── checkpoints/ # Session checkpoints
└── config/      # Runtime configuration overrides
```

**Using DataPaths:**
```python
from orchestration.core.paths import DataPaths

paths = DataPaths()
queue_file = paths.queue_state  # Returns Path to orchestration/data/state/queue_state.json
paths.queue_state.parent.mkdir(parents=True, exist_ok=True)  # Ensure directory exists before write
```

**Key Points:**
- All code should use `DataPaths` class for path access (not hardcoded paths)
- Legacy file locations are NOT supported - all files must be in `orchestration/data/`
- Migration from legacy locations happens automatically during upgrade

## Context Window Management

### Two-Tier Limit System

**Soft Limits** (Best Practices):
- Optimal size: 60,000-80,000 tokens (~6,000-8,000 lines)
- Based on safe splitting margins and human code review capacity

**Hard Limits** (Technical Constraints):
- Warning threshold: 100,000 tokens (~10,000 lines)
- Split trigger: 120,000 tokens (~12,000 lines)
- Emergency limit: 140,000 tokens (~14,000 lines)
- Absolute maximum: 200,000 tokens (Claude's context window)

### Component Status Tiers
- 🟢 **Green (Optimal)**: < 80,000 tokens - Ideal size, full flexibility
- 🟡 **Yellow (Monitor)**: 80,000-100,000 tokens - Watch growth, plan split
- 🟠 **Orange (Split Required)**: 100,000-120,000 tokens - Split before major work
- 🔴 **Red (Emergency)**: > 120,000 tokens - STOP! Split immediately

### Token Budget Breakdown
**Total Context Window**: 200,000 tokens

**Overhead** (~21,000 tokens):
- CLAUDE.md instructions: 1,500 tokens
- Work instructions: 500 tokens
- System prompts: 2,000 tokens
- Contracts: 2,000 tokens
- Response buffer: 15,000 tokens

**Available for Source Code**: ~179,000 tokens

### Approximation Formula
```
Total Context = Source_Code_Tokens + Overhead_Tokens
Source_Code_Tokens ≈ Lines_of_Code × 10
Overhead_Tokens ≈ 21,000
```

## Automated Component Splitting

### Splitting Triggers
1. Token count exceeds 70,000
2. Line count exceeds 4,000
3. Complexity exceeds thresholds
4. Manual developer request

### Split Strategies
- **Horizontal Split**: By architectural layer (api-layer, business-logic, data-layer)
- **Vertical Split**: By feature (user-management, payment-processing, reporting)
- **Hybrid Split**: Combination approach (core-services, feature-modules, infrastructure)

### Splitting Process
1. **Analysis Phase**: Claude Code sub-agent analyzes structure and dependencies
2. **Planning Phase**: Generate split plan with new component names and contracts
3. **Execution Phase**: Orchestrator coordinates file movement and dependency updates
4. **Validation Phase**: Run tests and verify contract compliance

## Component Isolation

### Sub-Agent Boundaries
- Each sub-agent operates ONLY within its component directory
- No cross-component file access permitted
- Communication only through defined contracts
- Read-only access to shared libraries and contracts

### .clinerules Pattern
Each component gets isolation rules:
```
ALLOWED_PATHS=.
FORBIDDEN_PATHS=../../components/*/
READ_ONLY_PATHS=../../contracts,../../shared-libs
```

## Git Operations (Single Repository)

### Architecture
The orchestration system uses a **single git repository** at the project root:
- All components share one repository (no component-level .git directories)
- Components commit with prefixed messages: `[component-name] feat: description`
- Concurrent commits handled by retry wrapper
- GitHub/GitLab compatible (no orphaned code)

### Concurrent Commit Handling
Component agents use the retry wrapper for safe concurrent commits:
```bash
# Retry wrapper handles index.lock conflicts automatically
python orchestration/cli/git_retry.py "auth_service" "feat: Add login endpoint"
```

### Features
- Automatic retry with exponential backoff + jitter
- Handles git index.lock conflicts transparently
- Configurable retry attempts (default: 5)
- Only 30 lines of code (simple and reliable)

## Development Environment

### DevContainer Configuration
- Located in `.devcontainer/devcontainer.json`
- Python 3 environment with AI development tools
- Mounted volumes: workspace, .claude config
- Environment variables: ANTHROPIC_API_KEY, OPENAI_API_KEY
- Extensions: Claude Dev, Claude Code, Continue, Python support

### Common Commands

**Verify repository setup:**
```bash
python orchestration/cli/self_check_repo.py
```

**Session initialization (auto-runs on start):**
```bash
python orchestration/cli/session_init.py --context
```

**Create new component:**
```bash
python orchestration/cli/create_component.py <component-name> --type <backend|frontend|library>
```

**Check component naming:**
```bash
python orchestration/cli/check_naming.py
```

**Git operations with retry (for concurrent commits):**
```bash
python orchestration/cli/git_retry.py "<component>" "<commit message>"
```

**Uninstall orchestration system:**
```bash
python orchestration/cli/uninstall.py
python orchestration/cli/uninstall.py --dry-run  # See what would be removed
```

**Run tools with --help for full usage:**
```bash
python orchestration/cli/<tool>.py --help
```

## Contract-Based Communication

### OpenAPI/YAML Contracts
- Located in `contracts/` directory
- Define all inter-component APIs
- Sub-agents implement endpoints per their contract
- Example: `contracts/backend-api.yaml`

### Contract Template Structure
```yaml
openapi: 3.0.0
info:
  title: Component API
  version: 1.0.0
paths:
  /endpoint:
    post:
      summary: Endpoint description
      requestBody: ...
      responses: ...
```

## Migration Tooling

### Existing Project Migration
1. **Analysis Phase**: Claude Code sub-agent analyzes existing codebase
2. **Planning Phase**: Generate migration plan with component breakdown
3. **Execution Phase**: Multiple sub-agents execute migration in parallel
4. **Validation Phase**: Run tests and verify functionality

### Migration Process
- Identify natural component boundaries
- Extract API definitions
- Map dependencies
- Preserve git history in single repository
- Create component-specific CLAUDE.md files

## 🔒 COMPONENT NAMING STANDARD (MANDATORY)

**Enforcement**: Automatic (validator + gates)

### The Standard

**Pattern**: `^[a-z][a-z0-9_]*$`

ALL component names MUST:
1. Start with lowercase letter (a-z)
2. Contain ONLY: lowercase letters + numbers + underscores
3. NO hyphens, spaces, uppercase, special characters
4. NOT be reserved names (test, src, lib, etc.)

### Examples

**Valid** ✅:
- `auth_service`
- `payment_api`
- `user_lib`
- `audio_processor`
- `shared_types`

**Invalid** ❌:
- `auth-service` (hyphens break Python: `from components.auth-service` = syntax error)
- `AuthService` (uppercase not allowed)
- `payment api` (spaces not allowed)
- `test` (reserved name)
- `123_service` (cannot start with number)

### Why This Standard Exists

**Historical Context**: In v0.2.0, hyphenated names caused:
- ❌ Python import errors (`from components.audio-processor` is invalid syntax)
- ❌ 30-60 minute debugging loops for sub-agents
- ❌ 20% project incompletion rate
- ❌ Projects stopped at 80% due to import failures

**Universal Compatibility**: This pattern works in:
- Python: `from components.auth_service import X` ✅
- JavaScript: `import X from 'components/auth_service'` ✅
- Rust: `use components::auth_service;` ✅
- Go: `import "project/components/auth_service"` ✅
- Java: `import components.auth_service.X;` ✅
- C++: `#include "components/auth_service/header.h"` ✅

### Validation Tool

**Check component name before creation**:
```bash
python3 orchestration/verification/system/component_name_validator.py <name>

# Example:
python3 orchestration/verification/system/component_name_validator.py auth_service
# ✅ 'auth_service' - Valid

python3 orchestration/verification/system/component_name_validator.py auth-service
# ❌ 'auth-service' - Invalid
#    Invalid component name 'auth-service': cannot contain hyphens (use underscores instead)
#    Suggestion: 'auth_service'
```

### Enforcement

**Automatic enforcement at**:
1. **Component creation** - Validator blocks invalid names before directory creation
2. **Project startup** - Scanner detects existing violations, offers auto-fix
3. **Phase 1 Gate** - Validates all component names before proceeding
4. **Pre-commit hook** - Blocks commits with invalid component names (optional)

**If you have existing components with invalid names**:
```bash
# Orchestrator will detect and offer to fix automatically on next run
# Or run migration manually:
python orchestration/migration/rename_components.py --dry-run  # Preview
python orchestration/migration/rename_components.py            # Execute
```

### Configuration

Control auto-fix behavior in `orchestration-config.json`:
```json
{
  "validation": {
    "component_naming": {
      "auto_fix": "prompt"  // "prompt", "always", "never"
    }
  }
}
```

**This standard is NON-NEGOTIABLE and enforced automatically.**

## Quality Standards

### Before Marking Component Complete
- Run all tests: 100% pass rate required (unit, integration, contract, E2E)
- Test coverage: minimum 80%
- Pass linting and formatting checks
- Commit to project repository using git retry wrapper
- Verify contract compliance
- Check token budget limits
- **Declare all features in component.yaml**: Update `user_facing_features` section with ALL CLI commands (for cli_application), public API (for library), or HTTP endpoints (for web_server)

**Testing Standards**: All test types require 100% pass rate. Zero failing tests allowed. See `docs/TESTING-STRATEGY.md` and `docs/ZERO-TOLERANCE-INTEGRATION.md` for complete policy.

**Feature Coverage**: Check #13 requires ALL user-facing features be declared in component.yaml manifest. This prevents failures where features exist but are never tested. See `orchestration/verification/manifests/` for schema validation.

### Completion Report Requirements

**When declaring a project complete**, you MUST generate an **evidence-based completion report**.

**What This Prevents:**
- Music Analyzer v1-v3 all had "completion" without evidence
- All three failed immediately on user commands (0% functional)
- Completion reports without evidence are meaningless

**Required Evidence (Must Be Pasted in Report):**
1. ✅ Phase 5 gate output (full command output, not summary)
2. ✅ Phase 6 gate output (full command output, not summary)
3. ✅ UAT command execution (actual `python -m ...` or equivalent)
4. ✅ State verification (orchestration-state.json contents)
5. ✅ Gate verification command output

**How to Generate Report:**

Use the completion report template at `orchestration/templates/COMPLETION-REPORT-TEMPLATE.md` and fill in all required evidence sections with actual command output.

**Report Sections That MUST Have Pasted Output:**

1. **Phase 5 Gate Output** (MANDATORY):
   ```
   $ python orchestration/gates/runner.py . 5
   [PASTE FULL OUTPUT - NO SUMMARIES]
   ```

2. **Phase 6 Gate Output** (MANDATORY):
   ```
   $ python orchestration/gates/runner.py . 6
   [PASTE FULL OUTPUT - NO SUMMARIES]
   ```

3. **UAT Command Execution** (MANDATORY):
   ```
   $ [primary user command from README.md]
   [PASTE FULL OUTPUT showing command works]
   ```

4. **Gate Verification** (MANDATORY):
   ```
   $ python orchestration/core/orchestrate_enforced.py verify-gates
   [PASTE OUTPUT showing all blocking gates passed]
   ```

**Invalid Report Examples:**

❌ "Phase 5 gate passed successfully"
❌ "All integration tests passing"
❌ "UAT completed without issues"
❌ "System ready for deployment"

**Valid Report Examples:**

✅ Pasted full terminal output from gate execution
✅ Timestamps visible in output
✅ Exit codes shown (0 = success)
✅ Actual command outputs, not descriptions

**Template Location:**
- `orchestration/templates/COMPLETION-REPORT-TEMPLATE.md`
- Contains all required sections with evidence markers
- Auto-fillable using generate_completion_report.py

**When Report is INVALID:**
- Missing any required evidence section
- Has "[PASTE ... OUTPUT HERE]" placeholders unfilled
- Gate execution timestamps missing
- Shows "⚠️ MISSING EVIDENCE" warnings

**The Rule:**
> If you haven't pasted the command output, you haven't proven it works.
> If you haven't proven it works, it probably doesn't.
> If it doesn't work, the project isn't complete.

**Historical Lesson:**
All three Music Analyzer versions had excellent internal metrics and completion reports declaring success. All three failed immediately on the first user command because nobody actually ran the documented commands before declaring "complete."

**Evidence-based reporting prevents this by requiring proof, not assertions.**

## Development Workflow Patterns

### Continuous Execution (CRITICAL)

When working on this project, whether as orchestrator or sub-agent:

**Auto-Proceed Through Multi-Phase Work:**
1. Use TodoWrite tool to track all phases/steps
2. Mark each phase complete as you finish
3. Automatically announce "Now proceeding to [next phase]" and continue
4. Only stop when all tasks complete, unrecoverable error occurs, or user requests pause

**Example:**
```
User: "Implement new feature with tests and documentation"

Your execution:
1. Implement feature - COMPLETE
2. Now proceeding to test writing...
3. Write tests - COMPLETE
4. Now proceeding to documentation...
5. Update documentation - COMPLETE
6. Now proceeding to commit...
7. Commit changes - COMPLETE
✅ ALL COMPLETE
```

**NEVER do this:**
```
❌ "Feature implemented. Should I write tests now?" [WRONG]
❌ "Tests done. Ready for documentation when you are." [WRONG]
❌ "All done. Should I commit?" [WRONG]
```

### Automatic Commit After Task Completion

**When you complete a task:**
1. Run final checks (tests pass, linting clean)
2. Commit immediately with conventional commit format
3. Include context: what changed, why, test results
4. Use git retry wrapper for component work (handles concurrent commits)

**Commit Message Format:**
```
<type>(scope): <subject>

<body with details>

Resolves: <ticket-id>
Tests: <count> passing, coverage <percentage>%
```

**Types:** feat, fix, refactor, test, docs, chore

**Example:**
```bash
git add .
git commit -m "feat(orchestration): add autonomous work protocols to templates

- Added continuous execution patterns to all templates
- Included automatic commit guidelines
- Added minimal implementation mandate
- BDD examples for all component types

Resolves: ORCH-456
Tests: All existing tests pass"
```

### Automatic Gate Execution in Multi-Phase Work [v0.17.0]

**When working on multi-phase orchestrated projects:**

**CRITICAL**: Gates are NOT optional checkboxes - they are MANDATORY blocking steps.

**After Completing Phase 5 (Integration):**
```bash
# REQUIRED: Run Phase 5 gate BEFORE proceeding to Phase 6
echo "=== Running Phase 5 Gate (Integration) ==="
python orchestration/gates/runner.py . 5

# Check result
if [ $? -eq 0 ]; then
    echo "✅ Phase 5 gate PASSED - proceeding to Phase 6"
else
    echo "❌ Phase 5 gate FAILED - CANNOT proceed"
    echo "Fix all integration test failures and re-run gate"
    exit 1
fi
```

**After Completing Phase 6 (Verification):**
```bash
# REQUIRED: Run Phase 6 gate BEFORE declaring complete
echo "=== Running Phase 6 Gate (Verification) ==="
python orchestration/gates/runner.py . 6

# Check result
if [ $? -eq 0 ]; then
    echo "✅ Phase 6 gate PASSED - project complete"
else
    echo "❌ Phase 6 gate FAILED - NOT complete"
    echo "Fix all completion check failures and re-run gate"
    exit 1
fi
```

**In Your Responses:**

When you run gates, you MUST paste the command output in your response:

```
Now running Phase 5 gate to verify integration:

$ python orchestration/gates/runner.py . 5

========================================
Phase 5 Gate: Integration Testing
========================================
✅ All integration tests passing (12/12)
✅ Test coverage: 87%
✅ No blocking issues
========================================
✅ PHASE 5 GATE PASSED
========================================

Proceeding to Phase 6...
```

**What This Looks Like:**

**✅ CORRECT (Evidence-Based):**
```
1. Implement feature - COMPLETE
2. Now proceeding to integration testing...
3. Integration tests - COMPLETE
4. Now running Phase 5 gate...
   [PASTE GATE OUTPUT HERE]
   ✅ Gate passed
5. Now proceeding to verification...
6. Verification - COMPLETE
7. Now running Phase 6 gate...
   [PASTE GATE OUTPUT HERE]
   ✅ Gate passed
✅ ALL COMPLETE with gate evidence
```

**❌ WRONG (No Evidence):**
```
1. Implement feature - COMPLETE
2. Integration tests - COMPLETE
3. Verification - COMPLETE
✅ ALL COMPLETE [NO GATE OUTPUT = INVALID]
```

**Auto-Proceed With Gates:**
- Run gate immediately after completing phase
- Paste gate output in response
- If gate fails: fix issues, re-run, paste new output
- If gate passes: announce and proceed to next phase
- Never skip gates - this causes the "Looks Good But Breaks" pattern

**Remember:**
- Gates are your protection against the 3 historical failures
- Pasted output proves verification actually happened
- No output = No verification = Invalid completion
- 100% is the only passing grade (83.3% = failure)

### Minimal Implementation Mandate

**The Golden Rule: Implement ONLY what is explicitly requested.**

**When given a task:**
- ✅ Implement the EXACT requested functionality
- ✅ Write tests for that functionality
- ✅ Update relevant documentation
- ❌ DO NOT add "nice to have" features
- ❌ DO NOT implement "while we're here" improvements
- ❌ DO NOT add speculative abstractions

**After Completion:**
If you identified potential improvements, mention them AFTER completing the work:
```
✅ Task complete and committed.

💡 Potential enhancements (not implemented):
- [Enhancement 1]
- [Enhancement 2]

Would you like me to implement any of these?
```

### Behavior-Driven Development (BDD)

**When to use BDD format in this project:**
- ✅ Orchestration workflows
- ✅ Component lifecycle management
- ✅ Template generation logic
- ✅ Migration workflows
- ❌ Low-level utilities (use standard TDD)
- ❌ Simple data transformations (use standard TDD)

**BDD Format (Given-When-Then):**
```python
def test_component_creation_generates_all_required_files():
    """
    Given a component name and type
    When create_component is called
    Then component directory is created
    And CLAUDE.md is generated from template
    And README.md is created
    And test directories are initialized
    """
    # Given
    component_name = "auth_service"
    component_type = "backend"

    # When
    result = create_component(component_name, component_type)

    # Then
    assert (Path("components") / component_name).exists()
    assert (Path("components") / component_name / "CLAUDE.md").exists()
    assert (Path("components") / component_name / "README.md").exists()
    assert (Path("components") / component_name / "tests").exists()
```

**When to use standard TDD:**
- Utility functions (token counting, path resolution)
- Parser functions (template variable substitution)
- Data transformations (config file processing)

## 🔒 VERIFICATION ENFORCEMENT SYSTEM (v1.13.0)

### Why This System Exists

The v1.1.0 anti-stopping rules failed because they relied on **instructional compliance**, but the model engages in **motivated reasoning** that circumvents instructions while technically acknowledging them.

**Historical Pattern (9+ failures):**
- High test pass rate (99-100%)
- Rules acknowledged in responses
- Stopped at phase boundary (20-70% complete)
- Completion report generated prematurely
- Required features reframed as optional

**Solution**: Verification scripts that YOU run at appropriate times. These provide the same checks as the previous git hook system, but are instruction-triggered rather than git-triggered.

### Core Verification Mechanisms

#### 1. Pre-Commit Verification (Run Before Commits)
```bash
# Run BEFORE committing (you trigger this, not git)
python orchestration/hooks/pre_commit_naming.py
python orchestration/hooks/pre_commit_enforcement.py

# For completion reports, also run:
python orchestration/hooks/pre_commit_completion_blocker.py
```

This checks the completion state file (in `orchestration/data/state/`) for approval.

#### 2. Task Queue (Authoritative Work List)
```bash
# Extract tasks from spec
python orchestration/tasks/task_extractor.py specs/project.yaml

# Get current task (MUST work on this)
python orchestration/tasks/task_runner.py --current

# Mark complete (only after full implementation + tests)
python orchestration/tasks/task_runner.py --complete TASK-ID

# View status
python orchestration/tasks/task_runner.py
```

**You CANNOT:**
- Skip tasks
- Redefine task scope
- Declare complete while queue has items
- Create your own task list

#### 3. Specification Coverage Checker
```bash
# Check coverage (0-100% objective metric)
python orchestration/verification/specs/spec_coverage_checker.py specs/project.md ./

# Output: "COVERAGE: 85.0%" with list of missing features
```

This provides **objective measurement**, not subjective "looks good".

#### 4. Rationalization Detector
```bash
# Scan for forbidden phrases
python orchestration/verification/quality/rationalization_detector.py .

# Detects: "substantially complete", "known limitations", "future work",
# "phase 1 complete", "stub", "placeholder", "foundation established", etc.
```

Based on actual phrases from historical failures.

#### 5. Verification Agent (Independent Verifier)
```bash
# Run full verification (5 checks)
python orchestration/verification/run_full_verification.py

# Checks:
# 1. No rationalization language
# 2. No stub components
# 3. Smoke tests pass (actual user commands)
# 4. Task queue empty
# 5. Gates passed
```

**This agent is SKEPTICAL by default.** It assumes incomplete until proven otherwise.

#### 6. Continuous Monitoring
```bash
# Single check
python orchestration/enforcement/monitor.py

# Continuous (every 5 minutes)
python orchestration/enforcement/monitor.py --continuous 5

# Status summary
python orchestration/enforcement/monitor.py --status
```

Detects stalling, regression, and unverified completion attempts.

### How Completion Works Under This System

1. **Work through task queue** - Each task from spec must be completed
2. **Queue becomes empty** - All features implemented (100% coverage)
3. **Gates run at phase boundaries** - YOU run them, integration and verification gates pass
4. **Verification agent runs** - Independent skeptical verifier approves
5. **Completion authority grants permission** - All checks in state file pass
6. **You run completion verification** - Before committing COMPLETION-REPORT.md

**You SHOULD NOT declare complete until:**
- Task queue is empty
- No stub components exist
- No rationalization language in docs
- Smoke tests pass
- Verification agent approves
- Completion verification passes

### Integration with Existing Rules

The verification system **works alongside** the anti-stopping rules:

- **v1.1.0 rules** (instructional): Still apply, still display in responses
- **v1.13.0 verification** (instruction-triggered): You run verification scripts

If you try to declare complete without verification:
```
VERIFICATION REQUIRED

Run before declaring complete:
  python orchestration/verification/run_full_verification.py

Required checks:
  All phase gates must pass
  Verification agent must approve
  No stub/placeholder components allowed

Process:
1. Run: python orchestration/verification/run_full_verification.py
2. Fix all identified issues
3. Re-run verification until all checks pass
4. Then commit completion report
```

### Key Files

```
orchestration/
├── hooks/                                # Verification scripts (you run these)
│   ├── pre_commit_completion_blocker.py  # Completion verification
│   ├── pre_commit_enforcement.py         # Advisory checks
│   └── post_commit_enforcement.py        # Continuation status
├── context/                              # Shared rules
│   ├── verification-protocol.md          # Full verification protocol
│   ├── component-rules.md                # Sub-agent rules
│   └── orchestration-rules.md            # Orchestrator rules
├── verification/
│   ├── verification_agent.py             # Independent verifier
│   ├── run_full_verification.py          # Main entry point
│   ├── quality/
│   │   ├── rationalization_detector.py   # Forbidden phrases
│   │   ├── stub_detector.py              # Find placeholders
│   │   └── smoke_tests.py                # User command tests
│   └── specs/
│       └── spec_coverage_checker.py      # Objective coverage
├── tasks/
│   ├── queue.py                          # Authoritative queue
│   └── auto_sync.py                      # Auto-sync from spec
├── enforcement/
│   ├── monitor.py                        # Continuous monitoring
│   └── stall_detector.py                 # Detect stalled work
└── gates/
    ├── runner.py                         # Gate runner
    └── executor.py                       # Gate execution
```

### Summary

**The verification system provides multiple checkpoints:**
1. Task queue still has items → Queue not empty
2. Spec coverage < 100% → Verification fails
3. Gates haven't passed → YOU must run them at phase boundaries
4. Rationalization language detected → Detector catches it
5. Independent agent denies approval → Authority not granted

**This creates multiple layers of verification that you must run at appropriate times.**

## Extended Thinking Usage

### For Orchestrator Work (You)

When working on this orchestration project:

**Use extended thinking for:**
- Architectural changes to orchestration system
- Component split algorithm design
- Token budget calculation improvements
- Migration workflow design
- Complex debugging across system

**Disable thinking for:**
- Documentation updates
- Simple bug fixes
- Test writing
- Configuration changes
- Routine refactoring

**How to enable:**
- Press `Tab` key in Claude Code session
- Or include "think hard" in your internal reasoning for complex decisions

### For Sub-Agents (When Launching Task Tool)

Include thinking keywords in Task prompts selectively:

```python
# Complex work - include thinking
Task(
    description="Redesign token counting algorithm",
    prompt="""Think hard about edge cases in token counting.

    Consider: multiline strings, unicode, code comments, docstrings.
    Implement robust token counter with tests.""",
    subagent_type="general-purpose",
    model="sonnet"
)

# Routine work - no thinking
Task(
    description="Add new component template field",
    prompt="""Add 'health_check_endpoint' field to component templates.

    Update all 5 templates, ensure consistency.""",
    subagent_type="general-purpose",
    model="sonnet"
)
```

### Cost Impact

Extended thinking adds 20-40% to project costs when used for complex decisions. Budget accordingly.

## Project Lifecycle and Breaking Changes Policy

**LIFECYCLE STATE**: released (see `orchestration/config/project_metadata.json`)
**BREAKING CHANGES POLICY**: controlled (deprecation required)

### Version Control Restrictions (Stable Release)

**🚨 CRITICAL: Major version transitions and breaking changes require explicit user approval**

**FORBIDDEN AUTONOMOUS ACTIONS:**
- ❌ Changing version from 1.x.x to 2.0.0 (major version bump)
- ❌ Any major version increment (X.y.z → X+1.0.0)
- ❌ Making breaking changes without deprecation period
- ❌ Releasing updates without upgrade scripts for existing installations
- ❌ Removing features without migration path

**WHY**: Production-deployed system means:
- Users depend on reliable, predictable updates
- Breaking changes without migration disrupt workflows
- Missing upgrade paths leave users stranded
- Trust requires backwards compatibility or clear deprecation

**ALLOWED AUTONOMOUS ACTIONS:**
- ✅ Increment minor version (1.1.0 → 1.2.0) WITH upgrade path
- ✅ Increment patch version (1.1.0 → 1.1.1) WITH upgrade path
- ✅ Deprecate features (with warnings, migration docs, 2-version period)
- ✅ Add new features (backwards compatible)
- ✅ Create upgrade scripts and migration paths

**MANDATORY FOR EVERY RELEASE:**
- ✅ Include `scripts/migrations/X.X.X_to_Y.Y.Y.sh`
- ✅ Update `scripts/upgrade.sh`
- ✅ Update `docs/CHANGELOG.md`
- ✅ Update `orchestration/VERSION`
- ✅ Test upgrade from all supported versions

### Checking Current Lifecycle State

```bash
# Check current project lifecycle and policy
cat orchestration/config/project_metadata.json | jq '.version, .lifecycle_state, .breaking_changes_policy'

# Check version control restrictions
cat orchestration/config/project_metadata.json | jq '.version_control'
```

### Documentation

For complete details, see:
- `orchestration/config/project_metadata.json` - Machine-readable lifecycle state

## Dynamic Component Creation

This orchestration system supports **dynamic component creation** during development. When you determine a new component is needed, you can create it by following the detailed instructions in `orchestration/templates/master-orchestrator.md` section "Dynamic Component Creation".

### Quick Summary

**When creating a component**:

1. **Create directory structure**: `components/<name>/src`, `tests/`, etc.
2. **Generate CLAUDE.md**: Read template (backend/frontend/generic), substitute variables, write to component directory
3. **Create README.md**: Document the component
4. **Inform user**: Component is ready for immediate use (no restart)

### Critical Requirements

- ✅ **Component ready immediately** (no configuration needed)
- ✅ **Use Task tool** to launch agents in component directories
- ✅ **No restart required** (continuous execution)

### Template Variable Substitution

When generating CLAUDE.md from templates, substitute:
- `{{COMPONENT_NAME}}` → component name (e.g., `auth_service`)
- `{{PROJECT_VERSION}}` → read from `orchestration/VERSION`
- `{{PROJECT_ROOT}}` → absolute project root path
- `{{TECH_STACK}}` → technologies (e.g., `Python, FastAPI, PostgreSQL`)
- `{{COMPONENT_RESPONSIBILITY}}` → what the component does

### Example: Creating "auth_service"

```bash
# 1. Create directories
mkdir -p components/auth_service/src/api
mkdir -p components/auth_service/tests/unit

# 2. Generate CLAUDE.md
# Read: orchestration/templates/component-backend.md
# Substitute all {{VARIABLES}}
# Write to: components/auth_service/CLAUDE.md

# 3. Create README.md
# (Component-specific content)

# 4. Tell user
echo "✅ Component created! Ready for immediate use via Task tool."
```

### Templates Available

- **Backend/Microservice**: `orchestration/templates/component-backend.md`
- **Frontend**: `orchestration/templates/component-frontend.md`
- **Library/Generic**: `orchestration/templates/component-generic.md`

### After Creation

After you create a component:
- Component is immediately ready to use
- Launch agent via Task tool with component-specific prompt
- Agent reads `components/<name>/CLAUDE.md` for instructions
- No restart needed - continuous execution

### Full Instructions

See `orchestration/templates/master-orchestrator.md` section "Dynamic Component Creation" for:
- Complete step-by-step workflow
- Validation checklist
- Component deletion procedure
- Troubleshooting guide
- Path resolution helpers

## Working with This Codebase

### Testing Strategy

**CRITICAL**: Avoid over-mocking! Tests that pass when code is broken are worse than no tests.

**Key Principles**:
- **Only mock what you don't own** (external APIs, paid services, time/date)
- **Don't mock your domain logic** (validators, repositories, transformers)
- **Integration tests are mandatory** for any heavily-mocked unit tests
- **Use real test databases** in integration tests (not mocks)

**See [`docs/TESTING-STRATEGY.md`](docs/TESTING-STRATEGY.md) for comprehensive guidelines** including:
- When to mock vs when NOT to mock (decision matrix)
- Concrete examples of good vs bad mocking
- Testing pyramid (60% unit, 30-40% integration, 5-10% E2E)
- Mock usage anti-patterns
- Test quality metrics beyond coverage

---

## CLAUDE.md Maintenance Rules

This file follows strict maintenance rules to prevent documentation staleness.

### What SHOULD Be in This File
- Behavioral rules (how the AI should act)
- Architectural principles (core concepts guiding decisions)
- Key conventions (standards that must be followed)
- Navigation pointers (where to find authoritative information)
- Critical constraints (non-negotiable boundaries)

### What Should NOT Be in This File
- **Hardcoded version numbers** - Reference `orchestration/VERSION` instead
- **Specific file path listings** - Use `--help` commands or pointers to directories
- **Project status/roadmap** - This changes constantly
- **Detailed API documentation** - Belongs in code docstrings or `docs/`
- **Historical context** - Move to `docs/HISTORICAL-FAILURES.md` or similar

### Validation
Before committing changes to CLAUDE.md, run:
```bash
python orchestration/validation/validate_claude_md.py
```

This checks for:
- References to non-existent files
- Hardcoded version numbers
- Hardcoded dates in examples
- Missing directories

### When to Update This File
- ✅ Adding/removing a behavioral rule
- ✅ Changing a core architectural principle
- ✅ Modifying a mandatory convention
- ✅ Adding a new constraint

### When NOT to Update This File
- ❌ Changing implementation details (update code instead)
- ❌ Adding new files/modules (they're discoverable via `ls`)
- ❌ Updating version numbers (they're in `orchestration/VERSION`)
- ❌ Fixing bugs (no documentation update needed)
- ❌ Refactoring code structure (update pointers if paths change)
