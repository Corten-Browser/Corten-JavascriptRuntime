# Claude Code Slash Commands for Orchestration

This directory contains Claude Code slash commands for autonomous project orchestration.

## Installation

When you install the orchestration system, copy this `.claude/` directory to your project root:

```bash
cp -r claude-orchestration-system/.claude /path/to/your/project/
```

After copying, these commands will be available in Claude Code.

## Available Commands

### `/orchestrate` - Compact Autonomous Orchestration

**Quick, concise command for experienced users.**

```
/orchestrate
```

What it does:
1. Reads all specification documents in project root
2. Plans architecture and creates all components
3. Tells you to restart Claude Code
4. After restart: coordinates all component agents to build complete project
5. Enforces quality gates and delivers working system

**Use when**: You want minimal verbosity and trust the orchestrator completely.

---

### `/orchestrate-full` - Detailed Autonomous Orchestration

**Detailed, step-by-step execution with progress reporting.**

```
/orchestrate-full
```

What it does:
1. **Phase 1**: Analysis & Architecture - reads specs, designs architecture, documents plan
2. **Phase 2**: Component Creation - creates all components, updates MCP config
3. **[RESTART REQUIRED]** - tells you to restart Claude Code
4. **Phase 3**: Contracts & Setup - creates API contracts, shared libraries
5. **Phase 4**: Parallel Development - coordinates all agents to implement features
6. **Phase 5**: Integration & Quality - runs tests, verifies quality gates
7. **Phase 6**: Documentation & Completion - generates docs, deployment guide

**Use when**: You want detailed progress reporting and explicit phase markers.

---

## How to Use

### Prerequisites

1. **Project specifications** in root directory as markdown files:
   - `project-spec.md`
   - `api-requirements.md`
   - `user-stories.md`
   - `architecture.md`
   - etc.

2. **Orchestration system installed** in your project

3. **Claude Code running** in project root directory

### Workflow

**Step 1: Prepare Specifications**
```bash
# Create comprehensive specification documents
vim project-spec.md
vim api-requirements.md
# etc.
```

**Step 2: Launch Orchestration**
```
/orchestrate-full
```
OR
```
/orchestrate
```

**Step 3: Wait for Component Creation**
Orchestrator will:
- Read your specs
- Design architecture
- Create all components
- Tell you: "⚠️ RESTART REQUIRED"

**Step 4: Restart Claude Code**
- Close Claude Code
- Reopen in same directory
- MCP servers now loaded

**Step 5: Continue Orchestration**
Say:
```
Restarted. Continue with Phase 3.
```
(for `/orchestrate-full`)

OR
```
Restarted. Continue.
```
(for `/orchestrate`)

**Step 6: Monitor Progress**
Orchestrator will:
- Coordinate all component agents
- Implement features in parallel
- Run quality verification
- Deliver complete system

---

## Execution Model

Both commands use **autonomous execution**:
- ✅ No approval gates (except restart)
- ✅ Makes all architectural decisions
- ✅ Implements everything per specs
- ✅ Only asks questions if specs are ambiguous

**Single stop point**: Restart after component creation

---

## Differences Between Commands

| Aspect | `/orchestrate` | `/orchestrate-full` |
|--------|----------------|---------------------|
| **Verbosity** | Minimal | Detailed |
| **Phase markers** | Implicit | Explicit |
| **Progress reporting** | Concise | Comprehensive |
| **Best for** | Experienced users | First-time users |
| **Execution** | Same workflow | Same workflow |

Both execute the same orchestration workflow - only the reporting differs.

---

## Tips for Best Results

**1. Complete Specifications**
The more detailed your specs, the better the results:
- Detailed feature requirements
- API endpoint specifications
- Data models and schemas
- User flows and stories
- Non-functional requirements (performance, security)

**2. Tech Stack Guidance**
Include technology preferences in specs:
```markdown
## Technology Stack
- Backend: Python 3.11, FastAPI, PostgreSQL
- Frontend: React 18, TypeScript, TailwindCSS
- Caching: Redis
- Queue: Celery
```

**3. Architectural Constraints**
Specify any architectural requirements:
```markdown
## Architecture
- Microservices architecture
- RESTful APIs with OpenAPI specs
- Event-driven communication for async operations
- CQRS pattern for read/write separation
```

**4. Quality Standards**
Orchestration system enforces by default:
- TDD (tests before code)
- 80%+ test coverage
- Quality gates (linting, formatting, complexity)
- Breaking changes encouraged (0.x.x pre-release)

---

## Example Session

```
$ claude code
Claude Code 2.0.24

You: /orchestrate-full

Orchestrator:
  PHASE 1: ANALYSIS & ARCHITECTURE

  Reading specifications:
  - project-spec.md (15,000 tokens)
  - api-requirements.md (8,000 tokens)

  [Detailed analysis...]

  ARCHITECTURE PLAN:
  - auth-service (backend)
  - user-api (backend)
  - payment-api (backend)
  - frontend (React)
  - shared-utils (library)

  PROCEEDING TO PHASE 2...

  PHASE 2: COMPONENT CREATION

  Creating auth-service... ✅
  Creating user-api... ✅
  Creating payment-api... ✅
  Creating frontend... ✅
  Creating shared-utils... ✅

  ⚠️  RESTART REQUIRED

  Components created: auth-service, user-api, payment-api, frontend, shared-utils

  Please restart Claude Code now to activate MCP servers.

  After restart, respond with: "Restarted. Continue with Phase 3."

  I will then autonomously complete Phases 3-6 (contracts, implementation, quality, documentation).

[RESTART CLAUDE CODE]

You: Restarted. Continue with Phase 3.

Orchestrator:
  PHASE 3: CONTRACTS & SETUP
  Creating API contracts... ✅

  PHASE 4: PARALLEL DEVELOPMENT

  @auth-service, implement authentication system...
  @user-api, implement user management...
  @payment-api, implement Stripe integration...
  @frontend, implement React dashboard...

  [Development proceeds autonomously...]

  PHASE 5: INTEGRATION & QUALITY
  Running integration tests... ✅
  Quality verification... ✅

  PHASE 6: DOCUMENTATION & COMPLETION

  PROJECT COMPLETE ✅

  Components: 5
  Lines of Code: ~35,000
  Test Coverage: 89%
  Documentation: docs/
```

---

## Troubleshooting

**Command not found**
- Ensure `.claude/` directory is in project root
- Restart Claude Code to load commands

**Orchestrator asks too many questions**
- Make specifications more detailed
- Explicitly state "make decisions autonomously"

**Components not available after restart**
- Check `.claude.json` exists in project root
- Verify component paths are absolute
- Confirm restart was complete

**Quality gates failing**
- Review failing tests
- Check test coverage reports
- Orchestrator will attempt fixes automatically

---

## Advanced Usage

**Resume after interruption**:
```
Resume orchestration from Phase 4. All components created, restart completed.
```

**Focus on specific phase**:
```
Execute only Phase 1 (analysis) and Phase 2 (component creation), then stop.
```

**Override defaults**:
```
/orchestrate-full

Override: Use 5-agent concurrency instead of 3.
```

---

## See Also

- `docs/CLAUDE-CODE-ONLY-IMPLEMENTATION-PLAN.md` - Implementation details
- `docs/COMPONENT-CREATION-QUICK-REFERENCE.md` - Component creation reference
- `CLAUDE.md` - Master orchestrator instructions
- `README.md` - Orchestration system overview

---

**Happy Orchestrating!** 🎯
