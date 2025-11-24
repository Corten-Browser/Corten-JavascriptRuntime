---
description: "Autonomously orchestrate any change - automatically scales to task complexity"
---

# Adaptive Orchestration Command

Execute any task with automatic complexity scaling from simple fixes to major enhancements.

## Overview

This command automatically determines the right level of orchestration based on your request:
- **Level 1 (Direct)**: Single file, < 100 lines → 2-5 minutes
- **Level 2 (Feature)**: 2-5 components, new feature → 15-30 minutes
- **Level 3 (Full)**: Spec docs, architecture changes → 1-3 hours

**You don't need to choose** - the system analyzes your request and scales appropriately.

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
   - Use Task tool with `model="sonnet"`
   - Each agent reads component's CLAUDE.md
   - Each agent applies TDD
   - Each agent runs component tests before completing

   **Sub-Agent Pattern**:
   ```python
   Task(
       description="Implement [feature] in [component]",
       prompt="""Read components/[component]/CLAUDE.md.

       Implement [specific functionality] with strict TDD:
       - Write tests first
       - Implement code to pass tests
       - Achieve 80%+ coverage
       - Run all component tests (100% pass required)

       Commit when done: feat([component]): [description]""",
       subagent_type="general-purpose",
       model="sonnet"
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
    prompt="""Read components/auth-service/CLAUDE.md.

    Implement password reset endpoint with TDD:
    - POST /auth/reset-password (request reset)
    - POST /auth/confirm-reset (confirm with token)
    - Generate secure tokens
    - 80%+ coverage
    - All tests passing

    Commit: feat(auth): add password reset endpoint""",
    subagent_type="general-purpose",
    model="sonnet"
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

**Process**: Complete orchestration workflow (all phases from orchestrate-full)

### Execution Approach

**This level uses the FULL orchestration workflow from `/orchestrate-full`** but applied incrementally to existing project.

### Key Differences from orchestrate-full

| Aspect | /orchestrate-full | /orchestrate (Level 3) |
|--------|-------------------|------------------------|
| **Starting Point** | Empty project | Existing project |
| **Phase 1** | Design from scratch | Analyze current + plan changes |
| **Phase 2** | Create all components | Create NEW + update existing |
| **Scope** | Entire project | Spec document or major enhancement |

### Execution Steps

Follow the complete workflow:

**Phase 1: Analysis & Architecture**
- Read specification document (if provided)
- Analyze current architecture
- Identify components to create/update/split
- Plan contract changes
- Document architecture decisions

**Phase 2: Component Creation/Updates**
- Create new components if needed
- Split oversized components (>100K tokens)
- Update existing component CLAUDE.md files
- Create component directories and structure

**Phase 3: Contracts & Setup**
- Create/update API contracts in contracts/
- Update shared libraries in shared-libs/
- Validate all component configurations

**Phase 4: Parallel Development**
- Launch sub-agents for ALL affected components (new + updated)
- Use `model="sonnet"` for all sub-agents
- Enforce strict TDD
- Monitor context limits
- Each agent commits their work
- Validate agent count against configured limits:
  * Read orchestration/orchestration-config.json
  * Default: max_parallel_agents = 5
  * Warning threshold: warn_above = 7
  * Recommended maximum: recommended_max = 10
  * Absolute maximum: absolute_max = 15
  * Use ContextWindowManager.validate_concurrent_agents() for validation
  * If above absolute_max (15): ERROR - reduce count or queue work
  * If above recommended_max (10): WARNING - show performance concerns
  * If above warn_above (7): INFO - acceptable but monitor closely

**Phase 4.5: Contract Validation**
- Run contract tests for all components
- Require 100% pass rate
- Fix any API mismatches before integration

**Phase 5: Integration Testing (Iterative)**
- Run full integration test suite
- Require 100% execution rate (no "NOT RUN")
- Require 100% pass rate
- If ANY failure: fix, re-run ENTIRE suite, verify 100%
- Use integration_coverage_checker.py to verify

**Phase 6: Completion Verification**
- Run completion_verifier for ALL components (11/11 checks)
- Verify project-type-specific UAT
- Check integration execution coverage (100%)
- Verify deployment readiness

**Complete Workflow Reference**: See `claude-orchestration-system/.claude/commands/orchestrate-full.md` for full details on each phase.

### Example Execution

**User**: `/orchestrate "Implement social_features_spec.md"`

**Your Response**:
```
Scope Analysis: Full Orchestration (Level 3)
Complexity Score: 6
Estimated Time: 1-3 hours

Signals detected:
  • Specification document mentioned (+3)
  • Feature keyword: 'implement' (+1)
  • Major feature set (from spec analysis) (+2)

Reasoning: Specification document indicates major enhancement requiring full orchestration

Proceeding with Full Orchestration...

Phase 1: Analyzing specification...
→ Reading social_features_spec.md
→ Features: friend lists, activity feed, notifications, messaging

Architecture analysis:
Current: 4 components (auth, user, db, notification)
Planned: Add 2 new components, update 2 existing

Architecture plan:
  NEW: social-graph component (friend relationships)
  NEW: activity-feed component (event stream)
  UPDATE: notification-service (social notifications)
  UPDATE: database-manager (new tables)

Phase 2: Creating new components...
→ Creating social-graph component
→ Creating activity-feed component
→ Components ready with CLAUDE.md files

Phase 3: Updating contracts...
→ Creating contracts/social-graph-api.yaml
→ Creating contracts/activity-feed-api.yaml
→ Updating contracts/notification-service-api.yaml

Phase 4: Launching parallel development (4 components)...
[Launch 3 agents initially, queue remaining work]

[Agent 1] social-graph:
Task(
    description="Implement social-graph component",
    prompt="""Read components/social-graph/CLAUDE.md.

    Implement complete social graph functionality per specification:
    - Friend request management
    - Friendship status tracking
    - Graph queries (mutual friends, suggestions)
    - Strict TDD, 80%+ coverage
    - Commit: feat(social-graph): implement friend management""",
    subagent_type="general-purpose",
    model="sonnet"
)

[Agents 2-4 for other components...]

[All agents complete]

Phase 4.5: Contract validation...
→ Running contract tests for all 4 components
→ 100% pass rate achieved

Phase 5: Integration testing...
→ Running complete integration test suite
→ First run: 18/20 passing (2 failures)
→ Analyzing failures: API mismatch in activity-feed
→ Fixing API mismatch...
→ Re-running ENTIRE suite: 20/20 passing (100%)
→ Verifying with integration_coverage_checker.py
→ ✅ 100% execution, 100% pass

Phase 6: Completion verification...
→ social-graph: 11/11 checks passing
→ activity-feed: 11/11 checks passing
→ notification-service: 11/11 checks passing
→ database-manager: 11/11 checks passing
→ Integration coverage: 100%
→ Test coverage: 85% (all components)

✅ Complete (2.3 hours)
Social features fully implemented per specification.
```

### Guidelines

- **Full workflow** (all phases, no shortcuts)
- **Architecture planning** (can create/split components)
- **Parallel sub-agents** (respect concurrency limits)
- **All quality gates** (100% everywhere)
- **Iterative integration testing** (fix → re-run ALL → verify)
- **Long duration** (1-3 hours for major features)
- **Reference orchestrate-full.md** for complete phase details

---

## EXECUTION RULES (ALL LEVELS)

These rules apply regardless of orchestration level:

### Continuous Execution

**Execute continuously to 100% completion** - Do NOT stop at:
- ❌ Integration test failures (fix them)
- ❌ API mismatches (fix them)
- ❌ "95% complete" (finish remaining 5%)
- ❌ "Should I continue?" (YES, always)

**ONLY stop if**:
- ✅ 100% complete (all tests passing, all quality checks pass)
- ✅ Specification contradiction (unresolvable ambiguity)
- ✅ Missing credentials (user must provide)
- ✅ User explicitly requests pause

### Quality Standards

**All levels must achieve**:
- 100% test pass rate (unit, integration, contract, E2E)
- 100% integration test execution rate (no "NOT RUN")
- TDD for new functionality (tests before code)
- Conventional commit messages
- Clean git status when complete

**Level 2 & 3 must also achieve**:
- 80%+ test coverage per component
- All completion_verifier checks passing (11/11)
- Contract validation passing (100%)
- Integration coverage verified

### Model Selection

**CRITICAL**: Always use `model="sonnet"` when launching Task tools:
```python
Task(
    description="...",
    prompt="...",
    subagent_type="general-purpose",
    model="sonnet"  # REQUIRED
)
```

Never omit the model parameter - sub-agents must use Sonnet for cost efficiency.

---

## EXAMPLES

### Example 1: Simple Fix (Level 1)

**User**: `/orchestrate "Fix typo in auth-service README: 'authetication' → 'authentication'"`

**Execution**:
```
Scope Analysis: Direct Execution (Level 1)
Score: -2 (simple change keyword)
Time: 2-5 minutes

Executing directly...

File: components/auth-service/README.md
Line 12: authetication → authentication

No tests needed (documentation)

Committed: docs(auth): fix typo in README

✅ Complete (30 seconds)
```

---

### Example 2: Multi-Component Feature (Level 2)

**User**: `/orchestrate "Add two-factor authentication"`

**Execution**:
```
Scope Analysis: Feature Orchestration (Level 2)
Score: 2 (feature keyword + moderate complexity)
Time: 15-30 minutes

Affected: auth-service, user-service, email-service

[Create todo list with 5 phases]

Phase 1: Analysis complete
Phase 2: Launching 3 sub-agents (parallel)
[Agents implement 2FA in each component]
Phase 3: Integration tests 100% pass
Phase 4: Quality verified (11/11 all components)

✅ Complete (28 minutes)
```

---

### Example 3: Major Enhancement (Level 3)

**User**: `/orchestrate "Implement payment_system_spec.md"`

**Execution**:
```
Scope Analysis: Full Orchestration (Level 3)
Score: 7 (spec doc + major scope)
Time: 1-3 hours

Reading spec: Payment processing, subscriptions, refunds

Phase 1: Architecture (create 3 new components)
Phase 2: Component creation
Phase 3: Contracts setup
Phase 4: Parallel development (6 components)
Phase 4.5: Contract validation (100%)
Phase 5: Integration testing (iterative to 100%)
Phase 6: Completion verification (all pass)

✅ Complete (2.1 hours)
```

---

## TROUBLESHOOTING

### Wrong Level Detected

**User can override**:
```
/orchestrate --level=direct "..."
/orchestrate --level=feature "..."
/orchestrate --level=full "..."
```

### Unclear Request

**If scope is ambiguous, ask ONE clarifying question**:
```
"Your request could be interpreted as [interpretation 1] or [interpretation 2].

Which did you mean?
1. [Option 1] - Would use Level [X]
2. [Option 2] - Would use Level [Y]
"
```

### Integration Tests Failing (Level 2 & 3)

**Never proceed with failures**:
1. Analyze error messages
2. Fix issues autonomously
3. Re-run ENTIRE integration suite
4. Verify 100% execution + 100% pass
5. Only then proceed

### Component Too Large (Level 3)

**If component exceeds 100K tokens**:
1. Use component splitting workflow
2. Create new components from oversized one
3. Update contracts
4. Re-run integration tests

---

## WHEN TO USE /orchestrate vs /orchestrate-full

| Situation | Command | Reason |
|-----------|---------|--------|
| **New project from scratch** | `/orchestrate-full` | Creates entire project structure |
| **Add feature to existing project** | `/orchestrate` | Adapts to existing architecture |
| **Fix bug** | `/orchestrate` | Auto-detects as Level 1 |
| **Implement spec doc** | `/orchestrate` | Auto-detects as Level 3 |
| **"Just build X"** | `/orchestrate-full` | Starting from nothing |
| **"Add X to the project"** | `/orchestrate` | Working with existing code |

---

BEGIN EXECUTION NOW

1. Analyze user request (Phase 0)
2. Determine orchestration level
3. Announce analysis result
4. Execute with appropriate level's process
5. Continue to 100% completion
