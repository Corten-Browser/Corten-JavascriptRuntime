# Create New Component

## Context

You are creating a new component in an orchestrated project. This component will be developed by a dedicated sub-agent working in isolation.

## Input Required

Before proceeding, gather this information:

- **Component Name**: `[COMPONENT_NAME]` (e.g., "user-api", "payment-service")
- **Component Type**: `[COMPONENT_TYPE]` (backend, frontend, data, ml, auth, or generic)
- **Tech Stack**: `[TECH_STACK]` (e.g., "Python, FastAPI, PostgreSQL")
- **Responsibilities**: `[RESPONSIBILITIES]` (brief description of what this component does)

## Steps

### Step 1: Check Agent Capacity

Check current agent status:

```python
import sys
sys.path.insert(0, 'orchestration')

from agent_launcher import AgentLauncher

launcher = AgentLauncher()
status = launcher.get_status_summary()

print(f"\nAgent Status:")
print(f"  Active: {status['active']}/{status['max_concurrent']}")
print(f"  Queued: {status['queued']}")

if status['active'] >= status['max_concurrent']:
    print(f"\n⚠ Warning: At maximum concurrent agents")
    print(f"  Component will be queued when launched")
else:
    print(f"\n✓ Agent slot available")
```

### Step 2: Create Component Directory

Create the component directory structure:

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name

mkdir -p components/$COMPONENT_NAME/{src,tests}
echo "Created component directory: components/$COMPONENT_NAME"
```

**Validation**:
```bash
ls -la components/$COMPONENT_NAME/
```

Should show `src/` and `tests/` directories.

### Step 3: Generate Component CLAUDE.md

Use template engine to create component-specific instructions:

```python
import sys
sys.path.insert(0, 'orchestration')

from pathlib import Path
from template_engine import TemplateEngine

engine = TemplateEngine(Path("claude-orchestration-system/templates"))

# Component information
component_name = "[COMPONENT_NAME]"  # Replace
component_type = "[COMPONENT_TYPE]"  # backend, frontend, data, ml, auth, or generic
tech_stack = "[TECH_STACK]"  # e.g., "Python, FastAPI, PostgreSQL"
responsibilities = "[RESPONSIBILITIES]"  # Brief description

# Select appropriate template
template_map = {
    "backend": "component-backend.md",
    "frontend": "component-frontend.md",
    "data": "component-generic.md",
    "ml": "component-generic.md",
    "auth": "component-generic.md",
    "generic": "component-generic.md"
}

template_name = template_map.get(component_type, "component-generic.md")

# Prepare variables
variables = {
    "COMPONENT_NAME": component_name,
    "TECH_STACK": tech_stack,
    "COMPONENT_RESPONSIBILITY": responsibilities,
    "CURRENT_TOKENS": "0",
    "LINT_COMMAND": "# Add lint command for your tech stack",
    "FORMAT_COMMAND": "# Add format command for your tech stack",
    "TEST_COMMAND": "# Add test command for your tech stack",
    "COVERAGE_COMMAND": "# Add coverage command",
    "TYPECHECK_COMMAND": "tsc --noEmit",  # For TypeScript
    "BACKEND_API": component_name,  # For frontend components
    "STATE_MANAGEMENT": "Context API / Redux / Zustand",
    "STYLING_APPROACH": "CSS Modules / Tailwind / Styled Components",
    "ADDITIONAL_INSTRUCTIONS": ""
}

# Render template
try:
    content = engine.render(template_name, variables)
    output_path = Path(f"components/{component_name}/CLAUDE.md")
    output_path.write_text(content)
    print(f"✓ Created {output_path}")
except Exception as e:
    print(f"✗ Error rendering template: {e}")
    print(f"  Creating basic CLAUDE.md instead")

    # Fallback basic template
    basic_template = f"""# {component_name} Component

You are building the {component_name} component.

## Boundaries
- Work ONLY in components/{component_name}/
- Read contracts from ../../contracts/
- Read shared libs from ../../shared-libs/

## Tech Stack
{tech_stack}

## Responsibility
{responsibilities}

## Workflow
1. Write tests
2. Implement features
3. Run tests
4. Commit to git
"""
    Path(f"components/{component_name}/CLAUDE.md").write_text(basic_template)
    print(f"✓ Created basic CLAUDE.md")
```

### Step 4: Create .clinerules for Isolation

Create isolation rules:

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name

cat > components/$COMPONENT_NAME/.clinerules << 'EOF'
# Component Isolation Rules

# Allowed paths (this component only)
ALLOWED_PATHS=.

# Forbidden paths (other components)
FORBIDDEN_PATHS=../*/

# Read-only paths (contracts and shared libraries)
READ_ONLY_PATHS=../../contracts,../../shared-libs

# Maximum file size
MAX_FILE_SIZE=1MB
EOF

echo "✓ Created .clinerules"
```

### Step 5: Initialize Local Git Repository

Initialize git for the component:

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name

cd components/$COMPONENT_NAME
git init
git add .
git commit -m "Initial component setup for $COMPONENT_NAME"
cd ../..

echo "✓ Initialized git repository"
```

### Step 6: Create API Contract

Generate contract template:

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name

cat > contracts/${COMPONENT_NAME}-api.yaml << EOF
openapi: 3.0.0
info:
  title: ${COMPONENT_NAME} API
  version: 1.0.0
  description: API contract for ${COMPONENT_NAME} component

servers:
  - url: http://localhost:8000
    description: Development server

paths:
  # Add your API endpoints here
  /health:
    get:
      summary: Health check
      responses:
        '200':
          description: Service is healthy
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                    example: "healthy"

components:
  schemas:
    # Add your schemas here
    Error:
      type: object
      properties:
        message:
          type: string
        code:
          type: string
EOF

echo "✓ Created API contract: contracts/${COMPONENT_NAME}-api.yaml"
```

**Note**: Customize the contract with actual endpoints for this component.

### Step 7: Set Up Tech Stack

Based on the tech stack, initialize the development environment:

#### For Python Components:

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name

cd components/$COMPONENT_NAME
python3 -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install common dev dependencies
pip install pytest black mypy pylint

# Create requirements files
echo "pytest\nblack\nmypy\npylint" > requirements-dev.txt
echo "# Add project dependencies here" > requirements.txt

# Create pytest config
cat > pytest.ini << 'EOF'
[pytest]
testpaths = tests
python_files = test_*.py
python_classes = Test*
python_functions = test_*
EOF

cd ../..
echo "✓ Python environment setup complete"
```

#### For TypeScript/JavaScript Components:

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name

cd components/$COMPONENT_NAME
npm init -y

# Install dev dependencies
npm install --save-dev typescript @types/node jest ts-jest @types/jest

# Initialize TypeScript
npx tsc --init

# Create jest config
npx ts-jest config:init

cd ../..
echo "✓ TypeScript environment setup complete"
```

#### For Rust Components:

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name

cd components/$COMPONENT_NAME
cargo init --name $(echo $COMPONENT_NAME | tr '-' '_')

cd ../..
echo "✓ Rust environment setup complete"
```

### Step 8: Register Component

Update agent registry:

```python
import sys
sys.path.insert(0, 'orchestration')

from pathlib import Path
import json
from datetime import datetime

registry_path = Path("orchestration/agent-registry.json")
registry = json.loads(registry_path.read_text())

component_name = "[COMPONENT_NAME]"  # Replace with actual name

# Add to tracking (not queued yet, just registered)
print(f"✓ Component registered: {component_name}")
print(f"  Location: components/{component_name}/")
print(f"  Ready to launch sub-agent")
```

### Step 9: Create Component README

```bash
COMPONENT_NAME="[COMPONENT_NAME]"  # Replace with actual name
TECH_STACK="[TECH_STACK]"  # Replace
RESPONSIBILITIES="[RESPONSIBILITIES]"  # Replace

cat > components/$COMPONENT_NAME/README.md << EOF
# $COMPONENT_NAME

## Description
$RESPONSIBILITIES

## Tech Stack
$TECH_STACK

## API Contract
See ../../contracts/${COMPONENT_NAME}-api.yaml

## Development

### Setup
\`\`\`bash
# Setup instructions based on tech stack
\`\`\`

### Running Tests
\`\`\`bash
# Test commands
\`\`\`

### Building
\`\`\`bash
# Build commands
\`\`\`

## Integration Points
- API Contract: ../../contracts/${COMPONENT_NAME}-api.yaml
- Shared Libraries: ../../shared-libs/

## Token Budget
- Target: < 40,000 tokens
- Current: 0 tokens
- Status: Green
EOF

echo "✓ Created README.md"
```

### Step 10: Launch Sub-Agent (Optional)

If you want to launch the sub-agent immediately:

```python
import sys
sys.path.insert(0, 'orchestration')

from agent_launcher import AgentLauncher

launcher = AgentLauncher()
component_name = "[COMPONENT_NAME]"  # Replace with actual name

result = launcher.launch_agent(
    component_name=component_name,
    task="Initialize component and implement API contract",
    priority=0
)

if result['status'] == 'launched':
    print(f"\n✓ Sub-agent launched for {component_name}")
    print(f"  PID: {result['pid']}")
elif result['status'] == 'queued':
    print(f"\n⏳ Component queued (position {result['position']})")
    print(f"  Will launch when agent slot available")
else:
    print(f"\n✗ Error: {result.get('message', 'Unknown error')}")
```

## Validation Checklist

- [ ] Component directory created (`components/[COMPONENT_NAME]/`)
- [ ] CLAUDE.md generated from template
- [ ] .clinerules created for isolation
- [ ] Local git repository initialized
- [ ] API contract created (`contracts/[COMPONENT_NAME]-api.yaml`)
- [ ] Tech stack initialized (dependencies installed)
- [ ] Component registered
- [ ] README.md created
- [ ] Ready to launch sub-agent

## Next Steps

1. **Review component CLAUDE.md**: Verify instructions are correct
2. **Customize API contract**: Define specific endpoints
3. **Launch sub-agent**: Start development on this component
4. **Monitor progress**: Check agent status and component size

## Monitoring

Check agent status:
```bash
python orchestration/agent_launcher.py status
```

Check component size:
```bash
python orchestration/context_manager.py
```

## Success Message

```
================================================================================
COMPONENT CREATED: [COMPONENT_NAME]
================================================================================

Type: [COMPONENT_TYPE]
Tech Stack: [TECH_STACK]
Location: components/[COMPONENT_NAME]/

The component is ready for development!

Files Created:
  ✓ components/[COMPONENT_NAME]/CLAUDE.md
  ✓ components/[COMPONENT_NAME]/.clinerules
  ✓ components/[COMPONENT_NAME]/README.md
  ✓ contracts/[COMPONENT_NAME]-api.yaml

Next Steps:
  1. Review: components/[COMPONENT_NAME]/CLAUDE.md
  2. Define API: contracts/[COMPONENT_NAME]-api.yaml
  3. Launch sub-agent for this component

Launch Command:
  python -c "from orchestration.agent_launcher import AgentLauncher; \
  AgentLauncher().launch_agent('[COMPONENT_NAME]', 'Implement API contract')"

================================================================================
```
