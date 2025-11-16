# CLAUDE.md Templates

This directory contains comprehensive CLAUDE.md templates for different component types in the orchestration system. All templates enforce rigorous quality standards including TDD/BDD, 80%+ test coverage, and complete documentation.

## Template Files

### master-orchestrator.md (900+ lines)
The master orchestrator CLAUDE.md that gets installed at the project root. This instructs Claude Code on:
- Coordinating all work without writing production code
- **Managing sub-agents with configurable concurrency limits** (default: 3) for token budget management
- **Quality verification workflows** - enforcing quality gates before accepting work
- **Multi-agent collaboration patterns** - TDD/BDD workflows, parallel execution
- **Quality metrics tracking** - dashboard generation, trend analysis
- **Architecture Decision Records** - documenting significant decisions
- Monitoring component sizes and triggering splits
- Resolving integration conflicts

### component-backend.md (1,100+ lines)
Comprehensive template for backend/API components with mandatory quality standards:
- **Mandatory TDD** - Red-Green-Refactor workflow enforced
- **Testing Standards** - 80% minimum coverage, 95% target
- **Backend Design Patterns** - Repository, Service Layer, Dependency Injection
- **API Contract Compliance** - OpenAPI/gRPC contract enforcement
- **Security Requirements** - SQL injection prevention, authentication, secrets management
- **Performance Guidelines** - Database optimization, caching, N+1 prevention
- **SOLID Principles** - with code examples
- **Definition of Done** - 25-point checklist

### component-frontend.md (1,000+ lines)
Comprehensive template for frontend/UI components with mandatory quality standards:
- **Mandatory TDD** - Component test-first development
- **Testing Standards** - Unit tests, integration tests, accessibility tests
- **Frontend Design Patterns** - Container/Presenter, Custom Hooks, Composition
- **Accessibility Requirements** - WCAG 2.1 AA compliance (mandatory)
- **Security Requirements** - XSS prevention, input validation, secrets management
- **Performance Guidelines** - Code splitting, lazy loading, memoization
- **User Experience Patterns** - Loading states, error handling, form validation
- **Definition of Done** - Comprehensive checklist including a11y verification

### component-generic.md (500+ lines)
Generic template for custom component types with comprehensive quality standards:
- **Mandatory TDD** - Red-Green-Refactor cycle with git history verification
- **Code Quality Standards** - Style, formatting, structure, naming conventions
- **Testing Standards** - 80% minimum coverage with all test types
- **Documentation Requirements** - README, docstrings, inline comments, examples
- **SOLID Principles** - Detailed explanations with code examples
- **Security Requirements** - Input validation, secrets management
- **Performance Guidelines** - Complexity documentation, optimization strategies
- **Definition of Done** - 25-point quality checklist

## Additional Templates

### pre-commit-hook.sh
Bash script that enforces 7 quality gates before allowing commits:
1. ✅ Tests Pass (100% pass rate)
2. ✅ Coverage ≥ 80%
3. ✅ Linting (zero errors)
4. ✅ Formatting (100% compliant)
5. ✅ No Debug Statements
6. ⚠️ TODO/FIXME Warning
7. ✅ No Secrets

Supports Python, JavaScript/TypeScript, Rust, and Go projects.

### bdd-feature-template.feature
Comprehensive Gherkin/BDD feature file template with examples for:
- Happy path scenarios
- Error/validation scenarios
- Scenario outlines (data-driven tests)
- Edge cases
- Security scenarios (XSS, SQL injection)
- Integration scenarios
- Performance scenarios
- Accessibility scenarios

## Template Variables

All templates support variable substitution:

- `{{COMPONENT_NAME}}` - Name of the component
- `{{COMPONENT_TYPE}}` - Type of component (backend, frontend, data, etc.)
- `{{TECH_STACK}}` - Technologies used (e.g., "Rust, actix-web, sqlx")
- `{{COMPONENT_RESPONSIBILITY}}` - Description of component's responsibilities
- `{{TOKEN_BUDGET}}` - Current estimated token count
- `{{API_CONTRACT}}` - Path to API contract file

## Usage

Templates are used by the orchestration tools when creating new components:

```python
from orchestration.create_subagent import create_subagent

# Automatically selects appropriate template
create_subagent(
    component_name="payment-api",
    component_type="backend",
    tech_stack=["python", "fastapi", "postgresql"]
)
```

## Customization

To create custom templates:

1. Copy an existing template
2. Modify the content for your specific needs
3. Save with a descriptive name (e.g., `component-blockchain.md`)
4. Reference in your orchestration scripts

## Template Structure

All templates follow this structure:

```markdown
# [Component Name] Component

## Your Boundaries
- Directory restrictions
- Access permissions
- Read-only paths

## Your Focus
- Responsibilities
- Quality standards
- Testing requirements

## Tech Stack
- Languages
- Frameworks
- Tools

## Integration Points
- API contracts
- Shared libraries
- Dependencies

## Development Workflow
- Step-by-step process
- Quality gates
- Commit requirements

## You MUST NOT
- Prohibited actions
- Boundary violations
```
