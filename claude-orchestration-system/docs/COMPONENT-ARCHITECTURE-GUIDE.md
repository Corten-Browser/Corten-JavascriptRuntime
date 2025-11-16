# Component Architecture Guide

**Version**: 0.5.0
**Last Updated**: 2025-11-11

This guide explains the updated component architecture for the Claude Code orchestration system, including component types, dependencies, naming conventions, and integration patterns.

## Table of Contents

- [Overview](#overview)
- [Component Naming Convention](#component-naming-convention)
- [Component Type Hierarchy](#component-type-hierarchy)
- [Component Interaction Model](#component-interaction-model)
- [Dependency Management](#dependency-management)
- [Specification-Driven Development](#specification-driven-development)
- [Development Workflow](#development-workflow)
- [Examples](#examples)
- [Tools Reference](#tools-reference)

## Overview

The orchestration system supports building software of any architecture - monolithic, microservices, or mixed - by using **composable libraries** that can import from each other's public APIs.

### Key Principles

1. **Components are libraries**, not isolated services
2. **Token limits force granularity**, not architectural isolation
3. **Libraries can import other libraries** via public APIs
4. **Component type determines complexity**, not isolation level
5. **Dependencies are explicit** in `component.yaml` manifests

### What Changed (v0.2.0)

- ✅ Components can now import from each other's public APIs
- ✅ Added component type hierarchy (Base → Core → Feature → Integration → Application)
- ✅ Added dependency management system
- ✅ Added specification analyzer to detect intended architecture
- ✅ Added specification verifier to ensure implementation matches spec
- ✅ Updated quality verification to check dependencies and spec compliance

### What Changed (v0.3.0)

- ✅ **Universal underscore naming convention** - all components now use `component_name` format
- ✅ Component name validation to prevent import issues
- ✅ Completion verification system - guarantees 100% project completion
- ✅ Checkpoint/resume capability for long-running components
- ✅ Dynamic time allocation based on component complexity
- ✅ Import path standardization with auto-generated `__init__.py` files

### What Changed (v0.4.0)

- ✅ **Quality-first code generation** - prevents bugs BEFORE they occur
- ✅ **Specification completeness analysis** - detects ambiguities and missing requirements
- ✅ **Contract-first development** - all APIs designed before implementation
- ✅ **Defensive programming enforcement** - automatic null safety, bounds checking, timeout validation
- ✅ **Semantic verification** - validates business logic correctness beyond syntax
- ✅ **Requirements traceability** - every line of code linked to requirements (REQ-XXX)
- ✅ **Integration failure prediction** - identifies mismatches before testing
- ✅ **Shared standards library** - consistent ErrorCodes, TimeoutDefaults, ValidationRules
- ✅ **System-wide validation** - 11-check deployment readiness (up from 8 checks)
- ✅ **Enhanced templates** - all templates updated with quality-first workflow

**Result**: Designed for multi-million line codebases with zero debugging budget.

## Component Naming Convention

**Version**: 0.5.0

### The Problem

In v0.2.0, components used hyphenated names like `audio-processor` and `shared-types`. This caused critical failures:

- ❌ **Python import errors**: `from components.audio-processor import X` is a syntax error
- ❌ **Agent debugging loops**: Sub-agents spent 30-60 minutes debugging imports instead of implementing features
- ❌ **20% incompletion rate**: Projects stopped at 80% completion due to import issues

### The Solution: Universal Underscore Convention

**All component names MUST use underscores only.**

**Format**: `[a-z][a-z0-9_]*`

**Rules**:
- Start with lowercase letter
- Contain only lowercase letters, numbers, and underscores
- No hyphens, spaces, or special characters
- Maximum 50 characters

### Examples

✅ **Correct**:
- `audio_processor`
- `shared_types`
- `user_management`
- `data_access`
- `app_orchestrator`

❌ **Incorrect**:
- `audio-processor` (hyphens break Python imports)
- `AudioProcessor` (PascalCase not compatible)
- `audioProcessor` (camelCase not compatible)
- `audio processor` (spaces not allowed)

### Cross-Language Compatibility

This convention works in **ALL** programming languages:

| Language | Syntax | Status |
|----------|--------|--------|
| **Python** | `from components.audio_processor import X` | ✅ Works |
| **JavaScript/TypeScript** | `import { X } from '../audio_processor'` | ✅ Works |
| **Rust** | `use components::audio_processor::X` | ✅ Works |
| **Go** | `import "project/components/audio_processor"` | ✅ Works |
| **Java** | `import components.audio_processor.X` | ✅ Works |
| **C++** | `#include "components/audio_processor.h"` | ✅ Works |

### Validation

Before creating any component, validate the name:

```bash
python orchestration/component_name_validator.py <component_name>
```

**Example**:
```bash
$ python orchestration/component_name_validator.py audio_processor
✅ 'audio_processor' - Valid

$ python orchestration/component_name_validator.py audio-processor
❌ 'audio-processor' - Invalid
   Invalid component name 'audio-processor': cannot contain hyphens (use underscores instead)
   Suggestion: 'audio_processor'
```

### Migration from v0.2.0

Existing projects with hyphenated names should:

1. Rename component directories: `audio-processor` → `audio_processor`
2. Update all imports in Python code
3. Update `component.yaml` manifests
4. Update contracts and documentation
5. Run validator on all components

## Component Type Hierarchy

Components are organized in a 5-level hierarchy based on dependencies and complexity:

### Level 0: Base Components

**Purpose**: Foundational utilities, data types, protocols

**Token Limit**: 40,000 tokens (~4,000 lines)

**Dependencies**: None (cannot depend on other components)

**Examples**:
- `shared_types`: Common data models
- `validation_utils`: Input validation functions
- `protocol_definitions`: Shared interfaces and protocols

**When to use**:
- Pure, self-contained functionality
- No dependencies on other components
- Used by many other components

```python
# components/shared_types/src/api.py
from dataclasses import dataclass
from typing import Optional

@dataclass
class User:
    """Shared user data type."""
    id: str
    email: str
    name: str
    country: Optional[str] = None
```

### Level 1: Core Components

**Purpose**: Core business logic, domain services

**Token Limit**: 60,000 tokens (~6,000 lines)

**Dependencies**: Base components only (1-3 dependencies typical)

**Examples**:
- `auth_core`: Authentication logic
- `data_access`: Database access layer
- `business_rules`: Core business rule engine

**When to use**:
- Reusable business logic
- Domain-specific functionality
- Foundation for feature components

```python
# components/auth_core/src/api.py
from components.shared_types import User
from components.validation_utils import validate_email

class Authenticator:
    """Core authentication logic."""

    def authenticate(self, email: str, password: str) -> User:
        """Authenticate user and return user data."""
        validate_email(email)
        # Authentication logic here
        return user
```

### Level 2: Feature Components

**Purpose**: Feature implementations, API endpoints, workflows

**Token Limit**: 80,000 tokens (~8,000 lines)

**Dependencies**: Base and Core components (2-5 dependencies typical)

**Examples**:
- `user_management`: User CRUD operations
- `payment_processing`: Payment workflows
- `reporting`: Report generation

**When to use**:
- Specific feature implementation
- API endpoints
- Feature-specific workflows

```python
# components/user-management/src/api.py
from components.shared_types import User
from components.auth_core.api import Authenticator
from components.data_access import Repository

class UserManager:
    """User management feature."""

    def __init__(self):
        self.auth = Authenticator()
        self.repo = Repository()

    def create_user(self, email: str, name: str, password: str) -> User:
        """Create new user account."""
        # Use core authentication
        # Use data access for storage
        pass
```

### Level 3: Integration Components

**Purpose**: Orchestrate multiple components, coordinate workflows

**Token Limit**: 100,000 tokens (~10,000 lines)

**Dependencies**: Base, Core, and Feature components (5-15+ dependencies expected)

**Examples**:
- `app_orchestrator`: Main application orchestrator
- `workflow_manager`: Complex workflow coordination
- `api_gateway`: API routing and aggregation

**When to use**:
- Coordinating multiple components
- Implementing cross-component workflows
- Application-level orchestration

**Important**: Integration components are **expected to have many imports** - this is correct and by design!

```python
# components/app-orchestrator/src/orchestrator.py
from components.user_management.api import UserManager
from components.payment_processing.api import PaymentProcessor
from components.notification.api import Notifier
from components.data_access import Repository
from components.reporting.api import ReportGenerator

class ApplicationOrchestrator:
    """
    Integration component that coordinates the entire application workflow.

    This component has many imports - this is correct!
    Integration components orchestrate other components.
    """

    def __init__(self, config):
        # Initialize all required components
        self.users = UserManager(config.user_db)
        self.payments = PaymentProcessor(config.payment_gateway)
        self.notifier = Notifier(config.email)
        self.repository = Repository(config.db)
        self.reports = ReportGenerator(config.reporting)

    def process_user_purchase(self, user_id, item_id):
        """Orchestrate complete purchase workflow across multiple components."""
        # Coordinate between components - don't reimplement their logic
        user = self.users.get_user(user_id)
        payment = self.payments.process_payment(user, item_id)
        self.repository.save_transaction(payment)
        self.notifier.send_receipt(user, payment)
        self.reports.record_sale(payment)
        return payment
```

### Level 4: Application Components

**Purpose**: Entry points (CLI, API server, GUI)

**Token Limit**: 20,000 tokens (~2,000 lines) - minimal code

**Dependencies**: Integration components primarily (1-3 dependencies)

**Examples**:
- `cli`: Command-line interface
- `api_server`: REST API server
- `desktop_app`: GUI application

**When to use**:
- Application entry points
- Command-line argument parsing
- Bootstrapping and configuration
- User interface layer

**Important**: Application components should be **minimal** - just wiring and bootstrapping!

```python
# components/cli/src/main.py
import argparse
from components.app_orchestrator import ApplicationOrchestrator
from components.config_manager import ConfigManager

def main():
    """
    CLI entry point - MINIMAL code only.

    Application components should NOT contain business logic.
    They bootstrap and wire together integration components.
    """
    # Parse arguments
    parser = argparse.ArgumentParser(description='My Application')
    parser.add_argument('command', help='Command to run')
    args = parser.parse_args()

    # Load configuration
    config = ConfigManager.load('config.yaml')

    # Create orchestrator (integration component)
    orchestrator = ApplicationOrchestrator(config)

    # Run command
    if args.command == 'process':
        result = orchestrator.process()
        print(f"Complete: {result}")

    return 0
```

## Component Interaction Model

### Public API vs Private Implementation

Components expose **public APIs** and hide **private implementation**:

```
components/my-component/
├── src/
│   ├── api.py              # PUBLIC - other components can import this
│   ├── types.py            # PUBLIC - shared types
│   ├── _internal/          # PRIVATE - implementation details
│   │   └── helpers.py
│   └── private/            # PRIVATE - internal modules
│       └── algorithms.py
```

### What Components CAN Do

✅ **Import other components' PUBLIC APIs**
```python
from components.audio_processor.api import AudioAnalyzer
from components.data_manager import DataStore
from components.shared_types import UserModel
```

✅ **Use other components as libraries/dependencies**
```python
analyzer = AudioAnalyzer()
result = analyzer.process(file)
```

✅ **Compose multiple components to build features**
```python
class AdvancedFeature:
    def __init__(self):
        self.component_a = ComponentA()
        self.component_b = ComponentB()
        self.component_c = ComponentC()
```

✅ **Create integration layers that orchestrate components**

✅ **Link compiled libraries from other components** (Rust, C++, etc.)

✅ **Call public functions/classes/modules from other components**

### What Components CANNOT Do

❌ **Access other components' PRIVATE implementation details**
```python
# ❌ FORBIDDEN - importing from _internal/ or private/
from components.audio_processor._internal.secrets import key
from components.data_manager.private.algorithms import helper
```

❌ **Modify files in other components' directories**
```python
# ❌ FORBIDDEN - writing to another component's files
with open("components/auth_service/config.json", "w") as f:
    f.write(data)
```

❌ **Import from _internal/ or private/ subdirectories**

❌ **Depend on implementation specifics not in public API**

❌ **Break encapsulation boundaries**

### Dependency Rules

1. **Components can only depend on same or lower level components**
   - Base (L0) → cannot depend on anything
   - Core (L1) → can depend on Base (L0)
   - Feature (L2) → can depend on Core (L1) and Base (L0)
   - Integration (L3) → can depend on Feature (L2), Core (L1), Base (L0)
   - Application (L4) → can depend on Integration (L3) primarily

2. **No circular dependencies**
   - If A depends on B, B cannot depend on A
   - Use dependency injection or event patterns to break cycles

3. **Dependencies must be explicit**
   - Declare all dependencies in `component.yaml`
   - Use semantic versioning for dependencies

## Dependency Management

### Component Manifest (component.yaml)

Every component must have a `component.yaml` file declaring its dependencies:

```yaml
# components/user-management/component.yaml
name: user-management
version: 1.0.0
type: feature  # base|core|feature|integration|application

description: User account management and authentication

language: python

# Public API definition
public_api:
  modules:
    - api
    - types
  classes:
    - UserManager
  functions:
    - create_user
    - get_user

# Component dependencies
dependencies:
  imports:
    - name: shared-types
      version: "^1.0.0"
      import_from: "components.shared_types"
      uses:
        - User
        - ValidationError

    - name: auth-core
      version: "^1.0.0"
      import_from: "components.auth_core.api"
      uses:
        - Authenticator
        - TokenValidator

    - name: data-access
      version: "^1.0.0"
      import_from: "components.data_access"
      uses:
        - Repository

# Token budget tracking
tokens:
  limit: 80000
  current: 45200
  status: green

# Quality standards
quality:
  min_coverage: 80
  tdd_required: true
  linting_required: true
```

### Using the Dependency Manager

The dependency manager validates dependencies and generates build order:

```bash
# Check all component dependencies
python orchestration/dependency_manager.py

# Get build order (topological sort)
python orchestration/dependency_manager.py --show-build-order

# Verify specific component
python orchestration/dependency_manager.py --verify user-management

# Generate dependency report
python orchestration/dependency_manager.py > dependency-report.txt
```

**Example output:**
```
============================================================
COMPONENT DEPENDENCY REPORT
============================================================

Components: 8

Components by Type:
  Base: 2
    - shared-types (0 dependencies)
    - validation-utils (0 dependencies)
  Core: 2
    - auth-core (1 dependencies)
    - data-access (1 dependencies)
  Feature: 3
    - user-management (3 dependencies)
    - payment-processing (3 dependencies)
    - reporting (2 dependencies)
  Integration: 1
    - app-orchestrator (7 dependencies)

Validation:
  ✅ No circular dependencies
  ✅ All dependencies respect level hierarchy
  ✅ All dependencies satisfied

Build Order:
   1. shared-types (base)
   2. validation-utils (base)
   3. auth-core (core)
   4. data-access (core)
   5. user-management (feature)
   6. payment-processing (feature)
   7. reporting (feature)
   8. app-orchestrator (integration)
============================================================
```

## Specification-Driven Development

### Analyzing Specifications

Use the specification analyzer to detect the intended architecture:

```bash
# Analyze specification
python orchestration/specification_analyzer.py project-spec.md

# Generate JSON output
python orchestration/specification_analyzer.py project-spec.md --json

# Verbose mode
python orchestration/specification_analyzer.py project-spec.md --verbose
```

**Example output:**
```
======================================================================
SPECIFICATION ANALYSIS
======================================================================

Architecture:
  Type: modular_monolith
  Confidence: 85%
  Integration Style: library_imports

Characteristics:
  Integrated: Yes
  Isolated: No
  Layered: Yes
  Modular: Yes

Components Found: 5
  - audio-processor (feature)
  - playlist-generator (feature)
  - music-analyzer (integration)
  - data-store (core)
  - cli (application)

Tech Stack: python, fastapi, postgresql

Recommendations:
  ✅ Use library-style components with direct imports
  ✅ Components can import from each other's public APIs
  ✅ Focus on clear public/private boundaries
  ✅ Enforce layer hierarchy (base → core → feature → integration → application)
======================================================================
```

### Verifying Implementation

Use the specification verifier to ensure implementation matches the spec:

```bash
# Verify implementation against specification
python orchestration/specification_verifier.py /project/root project-spec.md

# Save report to file
python orchestration/specification_verifier.py /project/root project-spec.md --output verification-report.txt
```

**Example output:**
```
======================================================================
SPECIFICATION VERIFICATION REPORT
======================================================================

✅ VERIFICATION PASSED

Specification: project-spec.md
Project Root: /workspaces/my-project
Total Checks: 12
Passed: 10
Failed: 0
Warnings: 2

WARNINGS:
  ⚠️ Found 2 components not in specification
    Extra: test-utils, benchmarks
    💡 Update specification to document these components

ALL CHECKS:
  ✅ Architecture: Direct imports
  ✅ Architecture: Integrated system
  ✅ Architecture: Layer hierarchy
  ✅ Components: All specified components present
  ✅ Components: No extra components
  ✅ Component types: Match specification
  ✅ Dependencies: No circular dependencies
  ✅ Dependencies: All dependencies exist
  ✅ Integration: Library imports
  ✅ Tech stack: Specified technologies

======================================================================
```

## Development Workflow

### Creating a New Component

1. **Determine component type** (base/core/feature/integration/application)
2. **Identify dependencies** (what will this import from?)
3. **Validate dependency levels** (no higher-level dependencies)
4. **Create directory structure**:
   ```bash
   mkdir -p components/my-component/src/api
   mkdir -p components/my-component/tests
   ```
5. **Create `component.yaml`** with dependencies
6. **Implement public API** in `src/api.py`
7. **Write tests** following TDD
8. **Verify dependencies**:
   ```bash
   python orchestration/dependency_manager.py --verify my-component
   ```

### Development Phases

When building a project, follow this phased approach:

**Phase 1: Planning**
- Analyze requirements
- Identify component boundaries
- Create dependency graph
- Validate no circular dependencies
- Generate build order

**Phase 2: Base Layer (Level 0)**
- Implement foundational types and utilities
- No dependencies on other components
- Can develop all base components in parallel

**Phase 3: Core Layer (Level 1)**
- Implement core business logic
- Depend only on base components
- Can develop core components in parallel

**Phase 4: Feature Layer (Level 2)**
- Implement features
- Depend on base and core
- Can develop features in parallel

**Phase 4.5: Integration Layer (Level 3)**
- Implement orchestrators
- Coordinate multiple lower-level components
- High import count is expected

**Phase 5: Application Layer (Level 4)**
- Implement entry points (CLI, API, GUI)
- Minimal code - just bootstrapping
- Depend primarily on integration components

**Phase 6: Verification**
- Run dependency validation
- Verify specification compliance
- Run integration tests
- Generate quality dashboard

### Quality Verification

The quality verifier now includes dependency and specification checks:

```bash
# Run quality verification
python orchestration/quality_verifier.py components/my-component
```

**New checks added:**
- **Dependency Validation**: Checks dependencies exist, no circular deps, level hierarchy respected
- **Specification Compliance**: Verifies implementation matches specification (warning-level)

## Examples

### Example 1: Music Analyzer (Integrated Architecture)

**Specification says**: "Create an integrated music analysis system where components work together by importing each other's functionality."

**Component Structure**:
```
components/
├── shared-types/          # L0: Base - Data models
├── audio-codec/           # L1: Core - Audio file handling
├── feature-extractor/     # L1: Core - Audio feature extraction
├── audio-processor/       # L2: Feature - Audio processing
├── benefit-scorer/        # L2: Feature - Scoring logic
├── playlist-generator/    # L2: Feature - Playlist creation
├── data-store/            # L2: Feature - Data persistence
├── music-analyzer/        # L3: Integration - Orchestrates everything
└── cli/                   # L4: Application - Command-line interface
```

**music-analyzer (Integration) imports from multiple components:**
```python
# components/music-analyzer/src/api.py
from components.audio_processor.api import AudioProcessor
from components.benefit_scorer.api import BenefitScorer
from components.playlist_generator.api import PlaylistGenerator
from components.data_store.api import DataStore

class MusicAnalyzer:
    """Integration component - orchestrates the workflow."""

    def __init__(self):
        self.processor = AudioProcessor()
        self.scorer = BenefitScorer()
        self.generator = PlaylistGenerator()
        self.storage = DataStore()

    def analyze_directory(self, path):
        """Orchestrate analysis across multiple components."""
        audio_results = self.processor.process_directory(path)
        scores = self.scorer.calculate_scores(audio_results)
        playlists = self.generator.create_playlists(scores)
        self.storage.save(audio_results, scores, playlists)
        return playlists
```

**CLI (Application) imports from integration:**
```python
# components/cli/src/main.py
from components.music_analyzer.api import MusicAnalyzer

def main():
    """Minimal CLI - just bootstrapping."""
    analyzer = MusicAnalyzer()
    result = analyzer.analyze_directory('/music')
    print(f"Created {len(result)} playlists")
```

### Example 2: E-Commerce Platform (Mixed Architecture)

**Specification says**: "Core services are isolated microservices communicating via REST APIs. Frontend is a monolithic React app importing feature modules."

**Component Structure**:
```
Backend (microservices):
├── user-service/         # L4: Application - REST API
├── payment-service/      # L4: Application - REST API
├── order-service/        # L4: Application - REST API

Frontend (integrated):
├── ui-components/        # L0: Base - Shared UI components
├── state-management/     # L1: Core - Redux store
├── user-module/          # L2: Feature - User features
├── cart-module/          # L2: Feature - Shopping cart
├── checkout-module/      # L2: Feature - Checkout
├── app-shell/            # L3: Integration - App orchestrator
└── web-app/              # L4: Application - React app entry
```

**Backend uses REST APIs** (isolated):
```python
# Components communicate via HTTP, not imports
# user-service makes HTTP call to payment-service
```

**Frontend uses imports** (integrated):
```javascript
// components/app-shell/src/App.jsx
import { UserModule } from 'components/user-module';
import { CartModule } from 'components/cart-module';
import { CheckoutModule } from 'components/checkout-module';

function App() {
  return (
    <div>
      <UserModule />
      <CartModule />
      <CheckoutModule />
    </div>
  );
}
```

## Tools Reference

### Dependency Manager

**Location**: `orchestration/dependency_manager.py`

**Commands**:
```bash
# Load and validate all manifests
python orchestration/dependency_manager.py

# Get build order
python orchestration/dependency_manager.py --show-build-order

# Verify specific component
python orchestration/dependency_manager.py --verify component-name

# JSON output
python orchestration/dependency_manager.py --json
```

**Python API**:
```python
from orchestration.dependency_manager import DependencyManager

manager = DependencyManager(project_root)
manager.load_all_manifests()

# Get build order
build_order = manager.get_build_order()

# Check for issues
cycles = manager.check_circular_dependencies()
violations = manager.validate_dependency_levels()
missing = manager.verify_dependencies('my-component')
```

### Specification Analyzer

**Location**: `orchestration/specification_analyzer.py`

**Commands**:
```bash
# Analyze specification
python orchestration/specification_analyzer.py spec.md

# JSON output
python orchestration/specification_analyzer.py spec.md --json

# Verbose output
python orchestration/specification_analyzer.py spec.md --verbose
```

**Python API**:
```python
from orchestration.specification_analyzer import SpecificationAnalyzer

analyzer = SpecificationAnalyzer()
result = analyzer.analyze_specification('spec.md')

print(f"Architecture: {result.architecture.architecture_type}")
print(f"Components: {len(result.suggested_components)}")
print(f"Integration style: {result.integration_style}")
```

### Specification Verifier

**Location**: `orchestration/specification_verifier.py`

**Commands**:
```bash
# Verify implementation
python orchestration/specification_verifier.py /project/root spec.md

# Save report
python orchestration/specification_verifier.py /project/root spec.md --output report.txt
```

**Python API**:
```python
from orchestration.specification_verifier import SpecificationVerifier

verifier = SpecificationVerifier(project_root)
verification = verifier.verify_against_specification('spec.md')

if verification.passed:
    print("✅ Implementation matches specification")
else:
    print(f"❌ {verification.checks_failed} checks failed")
    for error in verification.errors:
        print(f"  - {error.message}")
```

### Quality Verifier

**Location**: `orchestration/quality_verifier.py`

**Commands**:
```bash
# Run quality verification
python orchestration/quality_verifier.py components/my-component
```

**New Checks (v0.2.0)**:
- Dependency Validation: Checks dependencies exist, no cycles, level hierarchy
- Specification Compliance: Verifies against spec if present

## Summary

The updated component architecture enables:

1. **Flexible architecture support** - monolithic, microservices, or mixed
2. **Composable libraries** - components can import from each other
3. **Clear component hierarchy** - 5 levels from base to application
4. **Explicit dependencies** - declared in component.yaml
5. **Specification-driven development** - analyze spec, verify implementation
6. **Comprehensive quality checks** - including dependency and spec validation

This allows building systems of any size and architecture while maintaining quality standards and token limits.

---

**Questions or Issues?**

See the main documentation in `claude-orchestration-system/README.md` or refer to templates in `claude-orchestration-system/templates/`.
