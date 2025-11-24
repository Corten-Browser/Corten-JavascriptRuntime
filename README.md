# Claude Code Orchestration System

A complete multi-agent orchestration system for Claude Code that enables single developers to build and maintain large-scale software projects (thousands to millions of lines of code) through intelligent component isolation, automated splitting, and context window management.

## Overview

This project provides an installable orchestration layer that transforms any software project into a multi-agent development environment. It enforces strict context window limits, automates component splitting when size thresholds are exceeded, and coordinates concurrent Claude Code sub-agents (configurable, default: 5) working in isolation.

## Key Features

- **🎯 Context Window Management**: Two-tier limit system (optimal: 70k tokens, split: 110k tokens - adjusted for large CLAUDE.md overhead)
- **🎯 Autonomous Orchestration**: Built-in `/orchestrate` slash commands for hands-free project development
- **🤖 Task Tool-Based Agent Orchestration**: Built-in parallel execution using Claude Code's Task tool (zero configuration)
- **🔒 Component Isolation**: Each sub-agent works in isolated directories with enforced boundaries
- **✂️ Automatic Splitting**: Components automatically split when approaching size limits
- **📋 Contract-Based Communication**: Components communicate through OpenAPI/YAML contracts
- **🔄 Project Migration**: Tools for migrating existing codebases to orchestrated architecture
- **💰 Zero API Costs**: Uses Claude Code exclusively (no API token consumption)
- **🚀 Built-in Recursion**: dispatch_agent tool for spawning sub-agents recursively
- **⚡ Breaking Changes Encouraged**: Pre-release policy prevents unnecessary backwards compatibility (0.x.x = break freely)
- **✅ Zero Tolerance Integration Policy**: 100% integration test pass rate required (prevents Music Analyzer-type catastrophes)
- **📖 README Accuracy Testing** [v0.12.0]: Documentation examples validated (shell + Python + doctest)
- **🎯 Feature Coverage Testing** [v0.13.0]: 13-check completion system ensures ALL declared features tested (prevents undocumented command failures)
- **🔒 Programmatic Enforcement** [v0.14.0]: Phase gates block progression until requirements met (prevents stopping at 83.3% test pass rate)
- **📦 Distribution-First Design** [v0.15.0]: Package structure standard prevents hardcoded path failures (ensures software works when installed to any directory)
- **🌐 Multi-Language Support** [v0.16.0]: Distribution-ready validation for Python, JavaScript/TypeScript, Go, and Rust (plugin-based architecture)

## Model Strategy (v1.15.0+)

The orchestration system uses a **unified model approach** where sub-agents inherit the orchestrator's model by default:

### Model Selection

**Single control point** via `/model` command:
- **Sonnet 4.5** (default): Best for well-defined specifications
  - Excellent coding performance (77.2% on SWE-bench)
  - 30+ hour coherent autonomy
  - Cost-effective ($3/$15 per 1M input/output tokens)
  - Optimal for `/orchestrate` (runs without intervention)
- **Opus 4.5**: Best for complex/ambiguous specifications
  - Superior reasoning (70.6% on GPQA Diamond)
  - Best for coding AND architectural decisions
  - Higher cost ($5/$25 per 1M tokens, ~1.67x more)
  - Use when specifications are vague or highly complex

Use `/model sonnet` or `/model opus` to switch models.

### How It Works

**Model inheritance:**
- You select a model → all sub-agents automatically inherit that model
- No explicit model specification needed in code
- Consistent model usage throughout the project
- Easy to switch models based on current task complexity

### Cost Comparison

For a typical project with 5 components:

| Configuration | Estimated Cost | Use Case |
|--------------|----------------|----------|
| **Sonnet throughout** | **~$2.66** | **Recommended: well-defined specs** |
| Opus throughout | ~$4.44 | Complex/ambiguous specs |
| **Difference** | **+$1.78 (67%)** | **Moderate cost increase** |

**Note**: Sub-agents inherit your model choice. Cost difference is ~67% (Opus vs Sonnet), which is acceptable for complex specifications that benefit from better reasoning.

**Prior to v1.15.0**: The system enforced `model="sonnet"` for all sub-agents to prevent 5x cost overruns with Opus 4.1. Opus 4.5's improved pricing (1.67x vs 5x) made enforcement unnecessary, so v1.15.0+ uses simple model inheritance instead.

## Quick Start

### Installation

**One command to install:**

```bash
# Clone the orchestration system
git clone https://github.com/your-org/claude-orchestration.git

# Install to your project (from the repo root)
claude-orchestration/install.sh [OPTIONS] /path/to/your/project

# Ready to use!
cd /path/to/your/project
claude code
```

**What happens during installation:**

The `install.sh` script automatically:
1. ✅ Validates prerequisites (Python 3.8+, Git, write permissions)
2. ✅ Copies `` to your project
3. ✅ Installs slash commands to `.claude/commands/`
4. ✅ Creates `orchestration/config/orchestration.json` (with your max-agents setting)
5. ✅ Creates directory structure (`components/`, `contracts/`, `shared-libs/`)
6. ✅ Generates master `CLAUDE.md` from template (with MODEL STRATEGY section)
7. ✅ Initializes git repository and configures for orchestration
8. ✅ Validates installation (checks all critical files and Python imports)
9. ✅ Auto-commits installation to git with descriptive message

**Result:** Full orchestration system ready in your project directory with git initialized!

**Installation options:**

```bash
# Set max parallel agents (default: 5)
./install.sh --max-agents 7 /path/to/your/project
./install.sh -m 10 /path/to/your/project  # Short form (up to 15 max)

# Preview installation without making changes
./install.sh --dry-run /path/to/your/project

# Install without git operations
./install.sh --skip-git /path/to/your/project

# Overwrite existing installation
./install.sh --force /path/to/your/project

# Install without auto-commit (git init still runs)
./install.sh --no-commit /path/to/your/project

# Keep installer directory after installation
./install.sh --keep /path/to/your/project

# Combine flags
./install.sh --max-agents 10 --no-commit --keep /path/to/your/project

# Show all options
./install.sh --help
```

**Cloning an orchestrated project:**

```bash
# If the project already has orchestration installed:
git clone https://github.com/user/orchestrated-project.git
cd orchestrated-project
claude code
# Everything ready - slash commands work immediately!
```

### Three Ways to Start

**Option 1: Adaptive Orchestration (Recommended)**

For any task, from simple fixes to major features:

```
/orchestrate
```

This automatically scales the orchestration level based on task complexity:
- **Level 1 (Direct)**: Single file changes → 2-5 minutes
- **Level 2 (Feature)**: Multi-component features → 15-30 minutes
- **Level 3 (Full)**: Major enhancements from specs → 1-3 hours

The command analyzes your request and applies the appropriate orchestration level automatically. You can override with `--level=direct|feature|full` if needed.

**Continuous execution - no restarts required.**

**Option 2: Force Full Orchestration**

If you have complete project specifications and want to force full orchestration (skip auto-detection):

```
/orchestrate --level=full
```

This will autonomously:
1. Read your specifications
2. Design architecture and create all components
3. Launch parallel component agents using Task tool
4. Build complete project with coordinated parallel development
5. Deliver complete, tested, documented system

### Resume Interrupted Orchestrations

If orchestration stops prematurely (crash, timeout, user interrupt):

```bash
/orchestrate --resume
```

The system automatically:
- ✅ Detects previous state (checkpoint or discovery)
- ✅ Shows progress summary (completed/remaining phases, blocking issues)
- ✅ Skips completed phases automatically
- ✅ Continues from last incomplete phase

**How it works:**
1. **Checkpoint mode** (preferred): Loads from `orchestration-checkpoint.json` - knows exact state, original request, test results
2. **Discovery mode** (fallback): Scans project files, runs tests, analyzes git history if no checkpoint exists

**Resume with level override:**
```bash
/orchestrate --resume --level=full  # Force restart from Phase 1
```

See `.claude/commands/orchestrate.md` (help text with `--help` flag) for all resume options.

**Option 3: Manual Setup**

For manual component creation:
```
Please read and execute the instructions in prompts/setup-project.md
```

### Component Execution (Zero Configuration)

Component agents are launched dynamically using Claude Code's built-in Task tool.

**No configuration files needed!**

When the orchestrator creates components:
1. Creates component directory structure
2. Generates CLAUDE.md with component instructions
3. Components are immediately ready for use
4. No restart required

**How it works:**
```
User/You: "Build the complete system from specifications"

Orchestrator:
  → Analyzes specs, designs architecture
  → Creates all components
  → Launches parallel agents using Task tool
  → Each agent reads its CLAUDE.md and implements features
  → Coordinates integration and quality verification
  → Delivers complete system

No restarts, no configuration, continuous execution.
```

**Isolation:**
- Each agent instructed to work ONLY in its component directory
- Component CLAUDE.md reinforces boundaries
- Same isolation as previous MCP approach (instruction-based)

**Configuration:**
Max parallel agents set in `orchestration/config/orchestration.json` (default: 5, recommended max: 10, absolute max: 15)

### What Gets Installed

After installation, your project will have:

**✅ Committed to Git (part of your repository):**
```
your-project/
├── .claude/                # Slash commands (tracked in git)
│   └── commands/
│       └── orchestrate.md  # Adaptive orchestration command
├── orchestration/          # Management tools (tracked in git)
│   ├── git_retry.py        # Concurrent commit handling
│   ├── git_status.py       # Git status helper
│   ├── templates.py        # Embedded templates (no external files!)
│   ├── self_check.py       # Installation verification
│   ├── uninstall.py        # Uninstall script
│   ├── context_manager.py  # Token management
│   ├── version_guard.py    # Version protection
│   └── orchestration-config.json
├── components/             # Component workspaces (created empty)
├── contracts/              # API contracts (created empty)
├── shared-libs/            # Shared code (created empty)
├── CLAUDE.md              # Master orchestrator instructions
└── .gitignore             # Updated with orchestration entries
```

**🗑️ NOT in Repository (auto-removed by installer):**
```
  # Installer directory (removed after setup)
```

**📝 Gitignored (runtime files, not committed):**
```
orchestration/agent-registry.json    # Runtime tracking
orchestration/token-tracker.json     # Token usage
orchestration/logs/                  # Log files
components/_archived/                # Archived components
```

When you clone an orchestrated project from GitHub/GitLab, everything needed is already there!

## How to Specify System Specification Documents

The orchestration system uses specification documents to automatically populate the task queue. These documents define the features and requirements that need to be implemented.

### Supported Formats

- **YAML**: `specifications/my-app.yaml`
- **Markdown**: `specifications/my-app.md`

### Specification File Locations

The system automatically discovers specifications in these locations (case-insensitive):

- `specifications/*.{md,yaml,yml}`
- `specs/*.{md,yaml,yml}`
- `docs/*-specification.md`
- `docs/*-specifications.md` (plural)
- `docs/*-spec.md`
- `docs/*-specs.md` (plural)

**Note:** All searches are case-insensitive (`Specification.md` = `specification.md` = `SPECIFICATION.MD`).

### Option A: Move Specification to Standard Location

If your specification is in the project root or another location:

```bash
# Create specifications directory
mkdir -p specifications

# Move your specification file
mv my-project-spec.yaml specifications/

# Verify discovery
python3 orchestration/cli/auto_init.py --list

# Should show:
# Found specification files:
#   - specifications/my-project-spec.yaml
```

### Option B: Initialize from Specific File

You can initialize the queue from any file location:

```bash
# Initialize from specific file (any location)
python3 orchestration/cli/auto_init.py /path/to/my-spec.yaml

# Or relative path
python3 orchestration/cli/auto_init.py docs/project-requirements.md
```

This updates `orchestration/spec_manifest.json` to remember the file location.

### Verification

After setting up your specification, verify it worked:

```bash
# Check that specs are discoverable
python3 orchestration/cli/auto_init.py --check
echo $?  # Should be 0 if specs found

# List discovered specifications
python3 orchestration/cli/auto_init.py --list

# View populated queue
cat orchestration/tasks/queue_state.json | python3 -m json.tool

# Should show tasks extracted from your specification
```

### Example Specification (YAML)

```yaml
# specifications/my-app.yaml
features:
  - id: FEAT-001
    name: User Authentication
    description: Implement login/logout functionality
    required: true
    dependencies: []

  - id: FEAT-002
    name: User Dashboard
    description: Display user metrics and data
    required: true
    dependencies: [FEAT-001]
```

### Example Specification (Markdown)

```markdown
# My Application Specification

## Feature: User Authentication

Implement user login and logout with session management.

## Feature: User Dashboard

MUST implement a dashboard that displays:
- User profile information
- Activity metrics
- Export functionality

- [ ] Implement dashboard layout
- [ ] Implement metrics display
- [ ] Implement export feature
```

### Troubleshooting

**Queue remains empty after setup:**

```bash
# Re-run initialization
python3 orchestration/cli/auto_init.py

# Check for errors
python3 orchestration/cli/auto_init.py specifications/my-spec.yaml

# Verify queue populated
cat orchestration/tasks/queue_state.json
```

**Specifications not found:**

```bash
# Check current search paths
python3 orchestration/cli/auto_init.py --list

# Verify file exists and has correct extension
ls -la specifications/

# Check case sensitivity (Unix/Linux)
# File: Specification.md vs specification.md
```

For more details, see `orchestration/cli/auto_init.py` source code.

### Advanced Usage

#### Multiple Specification Files

The system can process multiple specification files simultaneously, merging all features into a single task queue:

```bash
# System automatically discovers and processes ALL specs
python3 orchestration/cli/auto_init.py

# Example: If you have both specifications/backend.yaml and specifications/frontend.yaml,
# the system will merge features from both files
```

**Features:**
- ✅ Validates each spec file independently
- ✅ Merges features from all valid specs
- ✅ Detects duplicate feature IDs across files
- ✅ Reports validation errors per file

**Queue state structure:**
```json
{
  "spec_file": "specifications/backend.yaml",
  "spec_files": ["specifications/backend.yaml", "specifications/frontend.yaml"],
  "total_features": 25,
  "tasks": [...]
}
```

#### Force Re-initialization (--init)

Use `--init` to clear the existing queue and re-initialize from specs:

```bash
# Force re-initialization (prompts for confirmation)
python3 orchestration/cli/auto_init.py --init
```

**What it does:**
- Shows current queue status (task count, completed count)
- Prompts for confirmation (prevents accidental data loss)
- Clears existing queue and manifest
- Re-discovers and processes all specs
- Creates fresh queue with all tasks as "pending"

**Use cases:**
- Specification structure changed significantly
- Need to reset all task statuses
- Corrupted queue state
- Starting a new development iteration

**Warning:** This clears ALL task progress (completed, in_progress, timestamps).

#### Update Queue (--update)

Use `--update` to update the queue while preserving existing task statuses:

```bash
# Update queue from specs (preserves statuses)
python3 orchestration/cli/auto_init.py --update
```

**What it preserves:**
- ✅ Task status (pending/in_progress/completed)
- ✅ Timestamps (started_at, completed_at)
- ✅ Verification results
- ✅ Completed task order

**What it updates:**
- ✅ Task names (if changed in spec)
- ✅ Task descriptions (if changed in spec)
- ✅ Dependencies (if changed in spec)

**What it adds:**
- ✅ New features from specs (added as "pending")

**Example output:**
```
Update summary:
  Total tasks: 28
  New features: 3
    + FEAT-010
    + FEAT-011
    + FEAT-012
  Updated features: 2
    * FEAT-003 (details changed)
    * FEAT-007 (details changed)
  Preserved tasks: 25
```

**Use cases:**
- Adding new features to existing project
- Updating feature descriptions without losing progress
- Syncing queue with updated specs
- Continuing work after spec changes

#### Specification Validation

All spec files are validated before processing:

**YAML requirements:**
```yaml
# Must have 'features' key at root level
features:
  - id: FEAT-001        # Required: unique ID
    name: Feature Name  # Required: display name
    description: ...    # Optional
    dependencies: []    # Optional
    required: true      # Optional (default: true)
```

**Markdown requirements:**
```markdown
# Must have recognizable feature patterns:

## Feature: User Authentication
## Component: Payment Processing
## Module: Report Generator

- [ ] Implement login functionality
- [ ] Implement user registration

MUST implement password reset functionality
SHALL support two-factor authentication
```

**Validation errors are reported clearly:**
```
❌ backend-spec.yaml: Missing required 'features' key at root level
❌ frontend-spec.md: No recognizable feature patterns found
✅ api-spec.yaml: Valid (12 features)
```

#### List Available Specs

```bash
# List all discoverable specification files
python3 orchestration/cli/auto_init.py --list

# Output:
# Found specification files:
#   - specifications/backend.yaml
#   - specifications/frontend.yaml
#   - docs/api-specification.md
```

#### Check Spec Availability

```bash
# Check if any specs exist (silent, exit code only)
python3 orchestration/cli/auto_init.py --check
echo $?  # 0 = specs found, 1 = no specs
```

**Use in scripts:**
```bash
if python3 orchestration/cli/auto_init.py --check; then
    echo "Specs available"
else
    echo "No specs found - please add specification documents"
    exit 1
fi
```

## Architecture

### Master Orchestrator Pattern

The system uses a master orchestrator that:
- Coordinates all work but **never writes production code directly**
- Manages sub-agents and enforces concurrency limits
- Monitors component sizes continuously
- Triggers automatic splitting when thresholds exceeded
- Resolves integration conflicts

### Component Isolation

Each component:
- Lives in its own isolated directory
- Has its own CLAUDE.md with specific instructions
- Commits to the shared project git repository (with retry wrapper for concurrency)
- Cannot access other components' source code
- Communicates only through defined contracts

### Git Operations (Single Repository)

The orchestration system uses a **single git repository** at the project root to ensure GitHub/GitLab compatibility:

**Architecture:**
- All components share the project-level git repository
- Each component commits with prefixed messages: `[component-name] feat: description`
- Concurrent commits handled by retry wrapper (`orchestration/cli/git_retry.py`)
- No component-level `.git` directories (avoids orphaned code on GitHub)

**Concurrent Commit Handling:**
```bash
# Component agents use the retry wrapper for safe concurrent commits
python orchestration/cli/git_retry.py "auth-service" "feat: Add login endpoint"
```

The retry wrapper:
- Handles git index.lock conflicts automatically
- Uses exponential backoff with jitter
- Retries up to 5 times (configurable)
- Only 30 lines of code (vs 1,500 lines for complex coordination)

### Token Budget

Components use a two-tier limit system:

**Soft Limits** (Best Practices):
- **Optimal size**: 60,000-80,000 tokens (~6,000-8,000 lines)

**Hard Limits** (Technical Constraints):
- **Warning threshold**: 100,000 tokens (~10,000 lines)
- **Split trigger**: 120,000 tokens (~12,000 lines)
- **Emergency limit**: 140,000 tokens (~14,000 lines)

## Project Lifecycle and Breaking Changes Policy

**Current Version**: 0.1.0 (Pre-Release)
**Lifecycle State**: pre-release
**Breaking Changes Policy**: encouraged

### Core Principle

This project follows **semantic versioning** (semver.org):
- **Version 0.y.z** = Initial development, **anything MAY change at any time**
- **Version 1.0.0+** = Stable release, backwards compatibility required

### Breaking Changes in Pre-Release (0.x.x)

During pre-release development, **breaking changes are encouraged and preferred**:
- ✅ Clean, simple code over backwards compatibility
- ✅ Remove deprecated code immediately
- ✅ Break and improve APIs freely
- ✅ Refactor to better patterns without hesitation
- ❌ No deprecation warnings for unreleased features
- ❌ No maintaining old API signatures "just in case"
- ❌ No compatibility layers during development

**Rationale**: Claude Code (and LLMs generally) are trained on production codebases where backwards compatibility is critical. This causes them to add unnecessary deprecation warnings, compatibility layers, and versioned APIs even in unreleased software. The breaking changes policy explicitly signals that this project is in active development and breaking changes are not only allowed but encouraged.

### Version Control Restrictions

**Major version transitions require explicit user approval**:

**FORBIDDEN** (automatic):
- ❌ Transitioning from 0.x.x to 1.0.0
- ❌ Any major version increment (X.y.z → X+1.0.0)
- ❌ Changing lifecycle_state without approval

**ALLOWED** (automatic):
- ✅ Minor version bumps (0.1.x → 0.2.0)
- ✅ Patch version bumps (0.1.0 → 0.1.1)
- ✅ Breaking changes in 0.x.x versions

**WHY**: Major version transitions (especially 0.x.x → 1.0.0) are business decisions, not technical decisions. They involve legal implications (SLAs), user communication, complete documentation, comprehensive testing, and business readiness.

### Transition to 1.0.0

When the project reaches version 1.0.0:
- Breaking changes policy changes to "controlled"
- Deprecation process required before removal
- Backwards compatibility becomes important
- API contracts are locked
- Semantic versioning rules tighten

**Until 1.0.0: Break freely, improve constantly.**

### Implementation

The policy is enforced through multiple layers:
1. **Project Metadata**: `orchestration/config/project_metadata.json` - Machine-readable lifecycle state
2. **Templates**: All CLAUDE.md templates include breaking changes policy
3. **Master Orchestrator**: Coordinates breaking changes across components
4. **Optional Tooling**: `breaking_changes_detector.py` (optional) - Scans for accidental backwards compatibility patterns

For complete details, see [`docs/BREAKING-CHANGES-POLICY.md`](docs/BREAKING-CHANGES-POLICY.md) (1,340 lines).

## Project Structure

```
claude-orchestration/
├──   # Installable system
│   ├── orchestration/           # Python management tools
│   ├── templates/               # CLAUDE.md templates
│   ├── prompts/                 # Setup prompt files
│   ├── contracts/               # Contract templates
│   └── scripts/                 # Installation scripts
├── docs/                        # Documentation
│   ├── claude-orchestration-enhanced-spec.md
│   ├── claude-orchestration-files.md
│   └── conversation-migration-summary.md
├── examples/                    # Example projects
├── tests/                       # Test suite
├── CLAUDE.md                    # Project-specific instructions
└── README.md                    # This file
```

## Documentation

- **[Enhanced Specification](docs/claude-orchestration-enhanced-spec.md)**: Complete system specification (authoritative)
- **[Orchestration Usage Guide](docs/ORCHESTRATION-USAGE-GUIDE.md)**: Comprehensive guide to adaptive `/orchestrate` command ⭐ **NEW (v0.9.0)**
- **[Slash Commands Guide](.claude/commands/README.md)**: `/orchestrate` commands for autonomous development
- **[Breaking Changes Policy](docs/BREAKING-CHANGES-POLICY.md)**: Pre-release development policy (1,340 lines)
- **[Task Tool Architecture](docs/TASK-TOOL-ARCHITECTURE.md)**: Parallel execution design
- **[Orchestration Configuration](docs/ORCHESTRATION-CONFIG.md)**: Configuring max agents, quality standards
- **[Component Creation Quick Reference](docs/COMPONENT-CREATION-QUICK-REFERENCE.md)**: Quick reference for dynamic component creation
- **[Installation Guide](README.md)**: Detailed installation instructions
- **[Component Development](docs/)**: Guide to developing components
- **[Migration Guide](docs/)**: Migrating existing projects

## Usage Examples

### Adaptive Orchestration (Recommended)

**For any task - automatically scales to the right level:**

```
/orchestrate
```

**Example tasks:**

**Simple fix (Level 1 - Direct):**
```
User: Fix typo in README.md line 42
→ Executes directly, minimal ceremony (2-5 min)
```

**Multi-component feature (Level 2 - Feature):**
```
User: Add user authentication across api-gateway and user-service
→ Launches sub-agents, full TDD, integration tests (15-30 min)
```

**Major enhancement (Level 3 - Full):**
```
User: Implement specifications/new-payment-system.md
→ Full orchestration workflow with architecture planning (1-3 hours)
```

**Override detection:**
```
User: /orchestrate --level=full
[Your task description]
→ Forces Level 3 even if task seems simpler
```

See docs/ORCHESTRATION-USAGE-GUIDE.md for detailed examples.

### Full Orchestration (Legacy)

**Complete hands-free development from specifications:**

```
/orchestrate "implement specifications/project_name.md"
```

Or force full orchestration:
```
/orchestrate --level=full
```

What happens:
1. Reads all specification documents in project root
2. Designs architecture, creates all components
3. Launches parallel component agents using Task tool
4. Builds complete project with coordinated parallel development
5. Delivers tested, documented, production-ready system

**Resume after interruption:**
```
/orchestrate --resume
```

See `.claude/commands/orchestrate.md` for detailed workflow and all flags.

### Creating a Single Component

**Manual component creation:**

Instruct the orchestrator:
```
Create a new backend component called "payment-api" using Python, FastAPI, and PostgreSQL
```

The orchestrator will:
1. Create directory structure
2. Generate CLAUDE.md from template
3. Update .claude.json automatically
4. Tell you to restart Claude Code
5. After restart: component available as `@payment-api`

### Monitoring Component Sizes

```bash
# Check all component sizes
./orchestration/scripts/check-sizes.sh

# Or use Python tool
python orchestration/context_manager.py
```

### Working with Components

**Autonomous Orchestration:**
Use `/orchestrate` slash command (with optional `--level=direct|feature|full` flag). Orchestrator launches all component agents automatically using Task tool. Supports resume with `--resume` flag.

**Manual Component Work:**

Option 1 - Through orchestrator:
```
You: "Add authentication to backend-api component"
Orchestrator: [Uses Task tool to launch agent in components/backend-api/]
```

Option 2 - Direct work:
```bash
cd components/backend-api/
claude code
# Work directly in component (reads local CLAUDE.md)
```

Both approaches work. Choose based on preference.

## Development Workflow

1. **Receive Requirements**: User provides feature specifications
2. **Decompose**: Orchestrator breaks down into component tasks
3. **Assign**: Tasks assigned to sub-agents (max 3 concurrent)
4. **Develop**: Sub-agents work in isolation
5. **Integrate**: Orchestrator coordinates integration via contracts
6. **Validate**: Run integration tests
7. **Deploy**: Coordinated deployment across components

## Component Splitting

When a component approaches size limits:

1. **Detection**: Context manager identifies oversized component
2. **Analysis**: Claude Code sub-agent analyzes split points
3. **Planning**: Generate split plan (horizontal/vertical/hybrid)
4. **Execution**: Create new components and migrate files
5. **Validation**: Run tests and verify contracts
6. **Archive**: Original component archived (git history preserved in main repository)

## System Requirements

- **Claude Code**: claude.ai/code subscription (Claude Max recommended)
- **Python**: 3.8 or higher
- **Git**: For component version control
- **OS**: Linux, macOS, or Windows (WSL)

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas for contribution:
- Additional component templates
- Example projects
- Documentation improvements
- Testing infrastructure
- Migration tools

## Roadmap

### Phase 1: Core Infrastructure ✅
- [x] Context window management
- [x] MCP-based agent orchestration ⭐ **MIGRATED FROM SUBPROCESS**
- [x] Basic orchestrator CLAUDE.md
- [x] Zero-code agent configuration

### Phase 2: Splitting Mechanism ✅
- [x] Component splitter design
- [x] Automated split execution
- [x] Split validation
- [x] Contract regeneration

### Phase 3: Quality & Language Support ✅
- [x] Quality verifier with 13 gates (v0.13.0: added feature coverage testing)
- [x] Programmatic enforcement via phase gates (v0.14.0: prevents premature stopping)
- [x] Python, TypeScript, Rust, Go support
- [x] Test coverage enforcement (80%+)
- [x] Linting and formatting automation

### Phase 4: Parallel Execution Architecture ✅  **COMPLETED 2025-11-09**
- [x] MCP analysis and removal (unnecessary complexity)
- [x] Task tool integration for parallel execution
- [x] Zero-configuration component launching
- [x] Continuous execution workflow (no restarts)
- [x] Configurable concurrency (orchestration-config.json)

### Phase 5: Advanced Features (In Progress)
- [x] Migration manager for existing projects
- [x] Breaking changes policy for pre-release development ⭐ **COMPLETED 2025-11-06**
- [x] Distribution-first redesign (v0.15.0) ⭐ **COMPLETED 2025-11-13**
  - [x] Package structure standard (PACKAGE-STRUCTURE-STANDARD.md)
  - [x] Hardcoded paths detection (Check #14)
  - [x] Package installability verification (Check #15)
  - [x] README comprehensiveness check (Check #16)
  - [x] README auto-generator (orchestration/generation/readme_generator.py)
  - [x] Deployment verifier (orchestration/verification/deployment/deployment_verifier.py)
  - [x] Structure analyzer (orchestration/structure_analyzer.py)
  - [x] Clean install tester integration (Phase 5.5)
  - [x] Deployment verification integration (Phase 6.5)
- [ ] Monitoring dashboard
- [ ] Performance optimization
- [ ] Enhanced orchestration patterns

## Testing

```bash
# Run test suite
pytest

# With coverage
pytest --cov=orchestration --cov-report=html

# Integration tests only
pytest tests/integration/
```

### Testing Standards

**All test types require 100% pass rate:**
- Unit tests: 100% pass
- Integration tests: 100% pass
- Contract tests: 100% pass
- E2E tests: 100% pass

**Test coverage**: ≥80% (coverage measures what has tests, pass rate measures what succeeds)

See [`docs/TESTING-STRATEGY.md`](docs/TESTING-STRATEGY.md) for complete testing guidelines and [`docs/ZERO-TOLERANCE-INTEGRATION.md`](docs/ZERO-TOLERANCE-INTEGRATION.md) for the zero-tolerance policy.

### Gate Enforcement System [v0.17.0]

**Problem Solved**: Prevents premature "completion" declarations despite failing verification steps.

**Historical Context**: Three Music Analyzer versions (v1-v3) were declared "complete" with good internal metrics but failed immediately on user commands. Root cause: orchestrator skipped mandatory verification gates.

**The "Looks Good But Breaks" Pattern:**
- ✅ High test pass rates (83.3%-100%)
- ✅ Good internal metrics
- ❌ **Critical verification steps skipped** (gates not run)
- 🔴 **User command failed immediately** (0% functional)

**Solution**: Multi-layered gate enforcement system that makes verification steps **mandatory**, not optional.

**Three Layers of Defense:**

1. **Enhanced Documentation** (Psychological Layer)
   - Ultra-priority warnings in `CLAUDE.md`
   - Historical failure examples as cautionary tales
   - Clear gate execution protocols
   - Self-check questions before declaring complete

2. **Enforced Wrapper** (Technical Layer)
   - `orchestrate_enforced.py` - Cannot bypass gates
   - Automatic gate execution after phases
   - Blocks progression if gates fail (exit code enforcement)
   - State tracking with timestamps in `orchestration-state.json`

3. **Evidence-Based Reporting** (Process Layer)
   - `COMPLETION-REPORT-TEMPLATE.md` - Requires pasted outputs
   - `generate_completion_report.py` - Auto-fills gate data
   - Validates completeness (flags missing evidence)
   - Report invalid without actual command outputs

**Critical Gates:**
- **Phase 5 Gate** (Integration Testing): Requires **100% integration test pass rate** (no exceptions)
- **Phase 6 Gate** (Verification): Requires **all 16 completion checks passing**

**Usage:**

```bash
# Check current state
python orchestration/orchestrate_enforced.py status

# Run Phase 5 with automatic gate enforcement
python orchestration/orchestrate_enforced.py run-phase 5
# Output: Gate runs automatically, BLOCKS if fails (exit 1)

# Check if can proceed to Phase 6
python orchestration/orchestrate_enforced.py can-proceed 6
# Output: YES (exit 0) or NO (exit 1) with reason

# Verify all blocking gates passed
python orchestration/orchestrate_enforced.py verify-gates
# Output: ✅ All gates passed OR ❌ Some gates missing/failed

# Generate evidence-based completion report
python orchestration/generate_completion_report.py

# Validate completion report
python orchestration/generate_completion_report.py --validate COMPLETION-REPORT.md
```

**Key Benefits:**
- ✅ **Cannot skip gates** (technical enforcement via exit codes)
- ✅ **Audit trail** (all gate executions timestamped in `orchestration-state.json`)
- ✅ **Evidence required** (completion reports invalid without pasted command outputs)
- ✅ **Clear blocking reasons** (know exactly why progression is blocked)

**What Gets Blocked:**
- Declaring Phase 6 complete without Phase 5 gate passing
- Declaring project complete without both Phase 5 & 6 gates passing
- Creating completion report without evidence (flagged as INVALID)

**Would Have Prevented:**
- Music Analyzer v2: 83.3% test pass rate declared "complete" → Blocked by Phase 5 gate
- Music Analyzer v3: `__main__.py` location error → Caught by Phase 6 UAT (Check #10)

**Documentation:**
- `CLAUDE.md`: Complete enforcement protocols for orchestrator
- `docs/GATE-ENFORCEMENT-MIGRATION.md`: Migration guide for existing projects
- `orchestration/templates/COMPLETION-REPORT-TEMPLATE.md`: Report template with evidence markers

**The Rule**: If you haven't pasted the command output, you haven't proven it works. If you haven't proven it works, it probably doesn't.

## License

[To be determined]

## Acknowledgments

Built for the Claude Code community to enable large-scale software development with AI assistance.

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/claude-orchestration/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/claude-orchestration/discussions)
- **Documentation**: [docs/](docs/)

## Version

**Current version: 0.16.0** (Pre-Release)

- **Lifecycle State**: pre-release
- **Breaking Changes Policy**: encouraged
- **API Stability**: Not guaranteed (0.x.x = anything may change)
- **Target Release Version**: 1.0.0 (when production-ready)

### Recent Changes (v0.16.0)

**Multi-Language Distribution Support** - Extends hardcoded path detection and distribution validation to all supported languages:

**New Language Plugins:**
- `orchestration/languages/python.py`: Python distribution validation
- `orchestration/languages/javascript.py`: JavaScript/TypeScript support
- `orchestration/languages/go.py`: Go support
- `orchestration/languages/rust.py`: Rust support

**Enhanced Tools:**
- `structure_analyzer.py`: Now detects and analyzes all languages in project
- All hardcoded path detection works across Python, JS/TS, Go, Rust
- Package structure validation for package.json, go.mod, Cargo.toml
- Deployment testing for npm, go build, cargo build

**Architecture:**
- Plugin-based language support system (`orchestration/languages/`)
- Auto-detection of project languages
- Graceful degradation when language tools unavailable
- 100% backward compatible with Python-only projects

**Problem Solved:** Distribution-ready validation now works for multi-language projects. JavaScript, Go, and Rust code is validated for hardcoded paths just like Python.

---

### Changes (v0.15.0)

**Distribution-First Redesign** - Prevents HardPathsFailureAssessment.txt failures:

**New Checks:**
- Check #14: No hardcoded absolute paths (`/workspaces/`, `/home/`, `C:\`)
- Check #15: Package is installable via `pip install`
- Check #16: README.md is comprehensive (500+ words, code examples)

**New Workflow Phases:**
- Phase 4.8: Package configuration generation (setup.py, requirements.txt)
- Phase 5.5: Clean install testing (isolated virtual environment)
- Phase 6.3: README generation (auto-generate comprehensive docs)
- Phase 6.5: Deployment verification (test in different directory)

**New Tools:**
- `orchestration/generation/readme_generator.py`: Auto-generates 500+ word READMEs
- `orchestration/verification/deployment/deployment_verifier.py`: Tests installation in different directories
- `orchestration/structure_analyzer.py`: Analyzes project structure, provides migration guidance
- `orchestration/package_generator.py`: Generates setup.py configuration
- `orchestration/clean_install_tester.py`: Tests installation in clean venv

**New Standards:**
- `docs/PACKAGE-STRUCTURE-STANDARD.md`: Official package structure standard
- Proper package imports instead of workspace imports
- Dynamic path construction (`Path(__file__).parent`)
- No PYTHONPATH manipulation

**Problem Solved:** Software that works in development now works when installed to any directory (prevents "it works on my machine" deployment failures).

---

This project follows semantic versioning (semver.org):
- Version 0.y.z = Initial development, breaking changes allowed
- Version 1.0.0 = First stable release (requires explicit user approval to transition)

Based on the enhanced specification with multi-agent coordination, automated component splitting, breaking changes policy, and distribution-first design.
