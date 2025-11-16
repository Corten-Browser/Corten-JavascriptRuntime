# Installation and Setup Scripts

This directory contains shell scripts for installing and managing the Claude Code orchestration system.

## Scripts

### install.sh
**Primary installation script** for setting up the orchestration system in a target project.

**Usage**:
```bash
./claude-orchestration-system/scripts/install.sh
```

**What it does**:
- Creates directory structure (components/, contracts/, shared-libs/, orchestration/)
- Copies orchestration Python tools
- Generates initial master orchestrator CLAUDE.md
- Initializes tracking files (agent-registry.json, token-tracker.json)
- Sets up git repository if not exists
- Creates .gitignore for orchestration-specific files

**Options**:
- `--force` - Overwrite existing installation
- `--skip-git` - Don't initialize git
- `--dry-run` - Show what would be installed without doing it

### init-component.sh
**Component initialization script** for creating new isolated component directories.

**Usage**:
```bash
./orchestration/scripts/init-component.sh <component-name> <component-type>
```

**Example**:
```bash
./orchestration/scripts/init-component.sh user-api backend
```

**What it does**:
- Creates component directory structure
- Generates component-specific CLAUDE.md
- Creates .clinerules for isolation
- Initializes local git repository
- Sets up testing framework
- Registers component in agent-registry.json

### check-sizes.sh
**Component size monitoring script** for checking token usage across all components.

**Usage**:
```bash
./orchestration/scripts/check-sizes.sh
```

**Output**:
```
Component Sizes Report
=====================
user-api:        3,247 lines (~32,470 tokens) ✓
payment-api:     5,891 lines (~58,910 tokens) ⚠️
notification:    1,203 lines (~12,030 tokens) ✓

Status: 1 component approaching limits
```

**Options**:
- `--json` - Output in JSON format
- `--watch` - Continuously monitor
- `--threshold <num>` - Set custom warning threshold

### migrate-project.sh
**Project migration script** for converting existing codebases to orchestrated architecture.

**Usage**:
```bash
./claude-orchestration-system/scripts/migrate-project.sh <source-path> [target-path]
```

**What it does**:
- Analyzes existing project structure
- Identifies component boundaries
- Creates migration plan
- Backs up original code
- Executes migration in phases
- Validates migrated components
- Preserves git history

**Options**:
- `--analyze-only` - Only analyze, don't migrate
- `--interactive` - Prompt for decisions
- `--preserve-history` - Use git subtree to preserve history

### status.sh
**System status script** for displaying current orchestration state.

**Usage**:
```bash
./orchestration/scripts/status.sh
```

**Output**:
```
Claude Code Orchestration Status
================================
Time: 2024-01-15 10:30:45

AGENT STATUS
  Active: 2/3
  Queued: 1
  Completed: 5

  Running:
  - user-api
  - payment-api

  Waiting:
  - notification-service

COMPONENT SIZES
  ✓ user-api:        32,470 tokens
  ⚠️ payment-api:    58,910 tokens
  ✓ notification:    12,030 tokens
  ✓ auth-service:    25,100 tokens
```

### cleanup.sh
**Cleanup script** for removing archived components and old logs.

**Usage**:
```bash
./orchestration/scripts/cleanup.sh
```

**What it does**:
- Removes archived components older than 30 days
- Cleans up completed agent logs
- Purges old tracking history
- Optimizes git repositories

**Options**:
- `--dry-run` - Show what would be deleted
- `--keep-days <num>` - Keep archives for N days
- `--aggressive` - Also clean local git objects

### validate-installation.sh
**Validation script** to verify orchestration system installation.

**Usage**:
```bash
./orchestration/scripts/validate-installation.sh
```

**Checks**:
- [ ] Directory structure exists
- [ ] Python tools are present
- [ ] Templates are installed
- [ ] Tracking files initialized
- [ ] Git repository configured
- [ ] Dependencies available

## Script Development Guidelines

When creating new scripts:

1. **Include Help**: Always support `--help` flag
2. **Error Handling**: Exit with non-zero code on errors
3. **Logging**: Use clear status messages
4. **Dry Run**: Support `--dry-run` for safety
5. **Idempotent**: Can run multiple times safely
6. **Variables**: Use descriptive variable names
7. **Comments**: Document complex logic

## Script Template

```bash
#!/bin/bash
set -euo pipefail  # Exit on error, undefined vars, pipe failures

# Script description
# Usage: ./script.sh [options]

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

# Main logic
main() {
    # Implementation
    :
}

# Run main
main "$@"
```

## Dependencies

Scripts require:
- Bash 4.0+
- Git
- Python 3.8+
- Standard Unix utilities (find, wc, grep, etc.)

## Testing Scripts

Test scripts in a safe environment:

```bash
# Create test project
mkdir /tmp/test-orchestration
cd /tmp/test-orchestration

# Test installation
./claude-orchestration-system/scripts/install.sh --dry-run

# Verify with validation script
./orchestration/scripts/validate-installation.sh
```
