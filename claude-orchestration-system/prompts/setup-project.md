# Setup Claude Code Orchestration System

## Context

You are setting up the Claude Code multi-agent orchestration system in this project. This is an autonomous installation process that will configure the project for orchestrated development.

## Prerequisites

Verify these exist:
- [ ] Python 3.8 or higher installed
- [ ] Git installed
- [ ] Write access to current directory
- [ ] `claude-orchestration-system/` directory exists in current location

## Installation Steps

### Step 1: Verify Location

Check that you are in the project root directory and that `claude-orchestration-system/` exists:

```bash
ls -la claude-orchestration-system/
```

**Expected output**: You should see `orchestration/`, `templates/`, `prompts/`, `contracts/`, `scripts/` directories.

### Step 2: Create Directory Structure

Create the orchestration directory structure in the project root:

```bash
mkdir -p components
mkdir -p contracts
mkdir -p shared-libs
mkdir -p orchestration
```

**Validation**:
```bash
ls -ld components contracts shared-libs orchestration
```

All four directories should now exist.

### Step 3: Install Python Tools

Copy the orchestration tools from the installation package:

```bash
cp -r claude-orchestration-system/orchestration/*.py orchestration/
```

**Validation**:
```bash
ls orchestration/*.py
```

**Expected files**:
- `context_manager.py`
- `agent_launcher.py`
- `component_splitter.py`
- `claude_code_analyzer.py`
- `migration_manager.py`
- `template_engine.py`
- `__init__.py`

### Step 4: Initialize Tracking Files

Create the agent registry file:

```bash
cat > orchestration/agent-registry.json << 'EOF'
{
  "max_concurrent": 3,
  "active": [],
  "queued": [],
  "completed": []
}
EOF
```

Create the token tracker file:

```bash
cat > orchestration/token-tracker.json << 'EOF'
{
  "last_updated": "",
  "components": {},
  "summary": {
    "total_components": 0,
    "components_near_limit": 0,
    "components_over_limit": 0
  }
}
EOF
```

**Validation**:
```bash
cat orchestration/agent-registry.json
cat orchestration/token-tracker.json
```

Both files should contain valid JSON.

### Step 5: Generate Master Orchestrator CLAUDE.md

Use Python to render the master orchestrator template:

```python
import sys
sys.path.insert(0, 'orchestration')

from pathlib import Path
from template_engine import TemplateEngine

# Initialize template engine
engine = TemplateEngine(Path("claude-orchestration-system/templates"))

# Get project name (from current directory)
project_name = Path.cwd().name

# Render master orchestrator template
variables = {
    "PROJECT_NAME": project_name,
    "PROJECT_ROOT": str(Path.cwd()),
    "ADDITIONAL_INSTRUCTIONS": ""
}

try:
    content = engine.render("master-orchestrator.md", variables)

    # Write to project root
    Path("CLAUDE.md").write_text(content)
    print(f"✓ Created CLAUDE.md for project: {project_name}")
except FileNotFoundError:
    # Fallback if template not found
    basic_claude_md = f"""# Master Orchestrator - {project_name}

You are the orchestrator for this multi-agent development project.

See claude-orchestration-system/README.md for full documentation.

## Your Role
- Coordinate all work but NEVER write production code
- Maximum 3 concurrent sub-agents
- Strict component isolation
- Contract-based communication

## Monitoring
- Check sizes: `python orchestration/context_manager.py`
- View agents: `python orchestration/agent_launcher.py status`
"""
    Path("CLAUDE.md").write_text(basic_claude_md)
    print(f"✓ Created basic CLAUDE.md for project: {project_name}")
```

**Validation**:
```bash
head -20 CLAUDE.md
```

Should show the master orchestrator configuration.

### Step 6: Initialize Git Repository

Check if git repository exists, initialize if needed:

```bash
if [ ! -d ".git" ]; then
    git init
    echo "✓ Initialized git repository"
else
    echo "✓ Git repository already exists"
fi
```

Create or update `.gitignore`:

```bash
cat >> .gitignore << 'EOF'

# Claude Code Orchestration
orchestration/agent-registry.json
orchestration/token-tracker.json
orchestration/split-history.json
orchestration/migration-state.json
orchestration/analysis_workspace/
components/_archived/
*.pyc
__pycache__/
.venv/
EOF
```

**Validation**:
```bash
grep "Claude Code Orchestration" .gitignore
```

Should show the orchestration entries in `.gitignore`.

### Step 7: Create Shared Library Structure

Set up the shared libraries directory:

```bash
mkdir -p shared-libs/common
```

Create shared-libs README:

```bash
cat > shared-libs/README.md << 'EOF'
# Shared Libraries

Code shared across multiple components.

## Usage
- Components can READ (but not modify) shared libraries
- All shared code must be versioned
- Document all public APIs

## Structure
- `common/` - Common utilities
- `auth/` - Authentication libraries
- `validation/` - Input validation
- `errors/` - Error handling
EOF
```

**Validation**:
```bash
cat shared-libs/README.md
```

### Step 8: Create Contracts Directory Structure

Set up contracts directory:

```bash
mkdir -p contracts
```

Create contracts README:

```bash
cat > contracts/README.md << 'EOF'
# API Contracts

OpenAPI/YAML contracts defining APIs between components.

## Usage
- Components implement endpoints defined in their contract
- All contracts must be valid OpenAPI 3.0
- Update contracts before implementing changes

## Naming Convention
`{component-name}-api.yaml`

## Example
See claude-orchestration-system/contracts/ for templates.
EOF
```

**Validation**:
```bash
cat contracts/README.md
```

### Step 9: Validate Installation

Run validation checks using Python:

```python
from pathlib import Path

# Check directories exist
required_dirs = ['components', 'contracts', 'shared-libs', 'orchestration']
all_exist = True

for dir_name in required_dirs:
    if Path(dir_name).exists():
        print(f"✓ {dir_name}/")
    else:
        print(f"✗ {dir_name}/ MISSING")
        all_exist = False

# Check Python modules exist
required_modules = [
    'orchestration/context_manager.py',
    'orchestration/agent_launcher.py',
    'orchestration/component_splitter.py',
]

for module in required_modules:
    if Path(module).exists():
        print(f"✓ {module}")
    else:
        print(f"✗ {module} MISSING")
        all_exist = False

# Check CLAUDE.md exists
if Path("CLAUDE.md").exists():
    print(f"✓ CLAUDE.md")
else:
    print(f"✗ CLAUDE.md MISSING")
    all_exist = False

# Check registry files exist
if Path("orchestration/agent-registry.json").exists():
    print(f"✓ orchestration/agent-registry.json")
else:
    print(f"✗ orchestration/agent-registry.json MISSING")
    all_exist = False

if all_exist:
    print("\n✓ Installation validated successfully!")
else:
    print("\n✗ Installation validation failed - missing components")
```

### Step 10: Initial Git Commit

Commit the orchestration setup:

```bash
git add .
git commit -m "Setup Claude Code orchestration system

- Install orchestration tools
- Create component directories
- Configure master orchestrator
- Initialize tracking files"
```

## Validation Checklist

After completing all steps, verify:

- [ ] Directory structure created (`components/`, `contracts/`, `shared-libs/`, `orchestration/`)
- [ ] Python tools installed in `orchestration/`
- [ ] Tracking files created (`agent-registry.json`, `token-tracker.json`)
- [ ] Master orchestrator `CLAUDE.md` created at project root
- [ ] Git repository initialized (or updated)
- [ ] `.gitignore` updated with orchestration entries
- [ ] Shared library structure created
- [ ] Contracts directory created
- [ ] Validation script passed
- [ ] Initial commit created

## Troubleshooting

**Issue**: Python modules not found
**Solution**: Ensure `claude-orchestration-system/orchestration/` exists and contains `.py` files

**Issue**: Permission denied when creating directories
**Solution**: Check write permissions in current directory

**Issue**: Template rendering fails
**Solution**: Verify `claude-orchestration-system/templates/` contains `.md` files

**Issue**: Git commands fail
**Solution**: Install git or skip git-related steps (not recommended)

## Next Steps

After successful installation:

1. **Review the generated CLAUDE.md** in the project root
2. **Create your first component** using the create-component prompt:
   ```
   Read and execute: claude-orchestration-system/prompts/create-component.md
   ```
3. **Begin development** with orchestrated architecture

## Success Message

When complete, you should see:

```
================================================================================
CLAUDE CODE ORCHESTRATION SYSTEM - INSTALLATION COMPLETE
================================================================================

Your project is now configured for multi-agent orchestrated development!

Directory Structure:
  ✓ components/     - Sub-agent workspaces
  ✓ contracts/      - API contracts
  ✓ shared-libs/    - Shared code
  ✓ orchestration/  - Management tools

Next Steps:
  1. Review CLAUDE.md for orchestration guidelines
  2. Create your first component
  3. Start building!

Monitoring Commands:
  - Check component sizes: python orchestration/context_manager.py
  - View agent status:    python orchestration/agent_launcher.py status
  - Get recommendations:  python orchestration/component_splitter.py recommendations

For help: See claude-orchestration-system/README.md
================================================================================
```

Display this success message after validation passes.
