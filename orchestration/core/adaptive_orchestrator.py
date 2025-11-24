#!/usr/bin/env python3
"""
Adaptive Orchestrator Helpers

Utility functions for executing different orchestration levels.

Part of the adaptive orchestration system (v0.9.0).
"""

from pathlib import Path
from typing import Dict, List, Optional
import subprocess


def execute_direct(task: str, target_files: Optional[List[Path]] = None) -> Dict:
    """
    Execute Level 1: Direct execution.

    Args:
        task: User's task description
        target_files: Optional list of files to modify

    Returns:
        Execution result dictionary
    """
    return {
        'level': 1,
        'approach': 'direct',
        'target_files': [str(f) for f in target_files] if target_files else [],
        'message': 'Executing directly with minimal ceremony',
        'guidance': """
Direct Execution Guidelines:
- Make changes directly to identified files
- Apply TDD if adding new functionality (not for fixes)
- Run affected tests only
- Commit with clear conventional commit message
- No sub-agents needed
- Expected time: 2-5 minutes
        """.strip()
    }


def execute_feature(task: str, affected_components: List[str]) -> Dict:
    """
    Execute Level 2: Feature orchestration.

    Args:
        task: User's task description
        affected_components: Components that need changes

    Returns:
        Execution result dictionary
    """
    return {
        'level': 2,
        'approach': 'feature',
        'components': affected_components,
        'message': f'Orchestrating feature across {len(affected_components)} component(s)',
        'guidance': """
Feature Orchestration Guidelines:
1. Create todo list for tracking phases
2. Analyze which components affected
3. For each component:
   - Launch sub-agent with TDD requirements
   - Implement feature following component's CLAUDE.md
   - Run component tests (100% pass required)
4. Run full integration test suite (100% pass required)
5. Verify quality using completion_verifier for affected components
6. Commit all changes with clear messages
7. Expected time: 15-30 minutes

Sub-Agent Launch Pattern:
Task(
    description="Implement [feature] in [component]",
    prompt=\"""Read components/[component]/CLAUDE.md.

    Implement [specific functionality].
    Follow TDD, achieve 80%+ coverage.\""",
    subagent_type="general-purpose",
    model="sonnet"
)
        """.strip()
    }


def execute_full(task: str, spec_document: Optional[Path] = None,
                 affected_components: Optional[List[str]] = None) -> Dict:
    """
    Execute Level 3: Full orchestration.

    Args:
        task: User's task description
        spec_document: Optional specification document path
        affected_components: Optional list of affected components

    Returns:
        Execution result dictionary
    """
    return {
        'level': 3,
        'approach': 'full',
        'spec': str(spec_document) if spec_document else None,
        'components': affected_components or [],
        'message': 'Using full orchestration workflow',
        'guidance': """
Full Orchestration Guidelines:
Follow the complete orchestration workflow with all phases:

Phase 1: Analysis & Architecture
- Read specification document (if provided)
- Analyze current architecture
- Plan changes (new components, splits, updates)
- Document architecture decisions

Phase 2: Component Creation/Updates
- Create new components if needed
- Split oversized components (> 100K tokens)
- Update component CLAUDE.md files

Phase 3: Contracts & Setup
- Update/create API contracts in contracts/
- Update shared libraries in shared-libs/
- Validate all configurations

Phase 4: Parallel Development
- Launch sub-agents for all affected components
- Use model="sonnet" for all sub-agents
- Enforce strict TDD (tests before code)
- Achieve 80%+ test coverage per component
- Monitor for context limits

Phase 4.5: Contract Validation
- Run contract tests for all components (100% pass)
- Verify API compliance before integration testing

Phase 5: Integration Testing (Iterative)
- Run full integration test suite
- Require 100% execution rate (no "NOT RUN" tests)
- Require 100% pass rate
- Fix failures and re-run ENTIRE suite
- Only proceed when 100%/100% achieved

Phase 6: Completion Verification
- Run completion_verifier for ALL components (11/11 checks)
- Verify project-type-specific UAT
- Check integration execution coverage (100%)
- Ensure deployment readiness

Reference: See .claude/commands/orchestrate.md
for complete workflow details (all phases included inline).

Expected time: 1-3 hours
        """.strip()
    }


def get_execution_guidance(level: int) -> str:
    """
    Get execution guidance based on orchestration level.

    Args:
        level: Orchestration level (1=Direct, 2=Feature, 3=Full)

    Returns:
        Guidance string for that level
    """
    if level == 1:
        return execute_direct("", [])['guidance']
    elif level == 2:
        return execute_feature("", [])['guidance']
    elif level == 3:
        return execute_full("")['guidance']
    else:
        return "Unknown orchestration level"


def run_affected_tests(component: str, project_root: Path) -> bool:
    """
    Run tests for a specific component.

    Args:
        component: Component name
        project_root: Project root directory

    Returns:
        True if all tests pass, False otherwise
    """
    component_path = project_root / "components" / component
    test_path = component_path / "tests"

    if not test_path.exists():
        return True  # No tests to run

    try:
        result = subprocess.run(
            ["pytest", str(test_path), "-v"],
            cwd=component_path,
            capture_output=True,
            text=True,
            timeout=300
        )
        return result.returncode == 0
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return False


def verify_component_quality(component: str, project_root: Path) -> Dict:
    """
    Run completion_verifier for a component.

    Args:
        component: Component name
        project_root: Project root directory

    Returns:
        Verification result dictionary
    """
    component_path = project_root / "components" / component
    verifier_path = project_root / "orchestration" / "completion_verifier.py"

    if not verifier_path.exists():
        return {
            'passed': False,
            'message': 'completion_verifier.py not found'
        }

    try:
        result = subprocess.run(
            ["python", str(verifier_path), str(component_path)],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=300
        )

        return {
            'passed': result.returncode == 0,
            'message': 'All checks passed' if result.returncode == 0 else 'Some checks failed',
            'output': result.stdout
        }
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        return {
            'passed': False,
            'message': f'Verification failed: {str(e)}'
        }


def check_integration_coverage(project_root: Path) -> Dict:
    """
    Check integration test execution coverage.

    Args:
        project_root: Project root directory

    Returns:
        Coverage result dictionary
    """
    checker_path = project_root / "orchestration" / "integration_coverage_checker.py"

    if not checker_path.exists():
        return {
            'ready': False,
            'message': 'integration_coverage_checker.py not found'
        }

    try:
        result = subprocess.run(
            ["python", str(checker_path), "--strict"],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=60
        )

        return {
            'ready': result.returncode == 0,
            'execution_rate': '100%' if result.returncode == 0 else 'Unknown',
            'message': 'All integration tests executed and passed' if result.returncode == 0
                      else 'Integration coverage check failed',
            'output': result.stdout
        }
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        return {
            'ready': False,
            'message': f'Coverage check failed: {str(e)}'
        }


def main():
    """CLI interface for testing helper functions."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: adaptive_orchestrator.py <level> [task]")
        print("  level: 1 (direct), 2 (feature), 3 (full)")
        sys.exit(1)

    level = int(sys.argv[1])
    task = sys.argv[2] if len(sys.argv) > 2 else "Example task"

    print(f"\n{'='*60}")
    print(f"Level {level} Orchestration Guidance")
    print(f"{'='*60}\n")

    print(get_execution_guidance(level))
    print(f"\n{'='*60}\n")


if __name__ == '__main__':
    main()
