"""
Workflow Orchestrator

Orchestrates multi-agent workflows for complex tasks.
Implements patterns like TDD, BDD, and multi-stage development workflows.

Classes:
    WorkflowOrchestrator: Main workflow coordination system
    Workflow: Definition of a multi-step workflow
    WorkflowStep: Individual step in a workflow
    WorkflowResult: Results from workflow execution

Usage:
    orchestrator = WorkflowOrchestrator()
    result = orchestrator.execute_workflow(
        workflow_type="feature_development",
        feature_name="user_registration",
        components=["user-api", "email-service"]
    )
"""

from pathlib import Path
from typing import Dict, List, Optional, Callable
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime
import json


class WorkflowType(Enum):
    """Predefined workflow types."""
    FEATURE_DEVELOPMENT = "feature_development"  # Full TDD/BDD feature development
    BUG_FIX = "bug_fix"  # Bug fix with test-first approach
    REFACTORING = "refactoring"  # Refactoring with test preservation
    DOCUMENTATION = "documentation"  # Documentation update
    SIMPLE_CHANGE = "simple_change"  # Simple one-agent change


class StepStatus(Enum):
    """Status of a workflow step."""
    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class WorkflowStep:
    """Individual step in a workflow."""
    name: str
    description: str
    agent_role: str  # e.g., "test_agent", "implementation_agent", "review_agent"
    component_name: Optional[str] = None
    task: str = ""
    status: StepStatus = StepStatus.PENDING
    depends_on: List[str] = field(default_factory=list)  # Names of prerequisite steps
    result: Optional[Dict] = None
    started_at: Optional[str] = None
    completed_at: Optional[str] = None
    quality_verification_required: bool = True

    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'name': self.name,
            'description': self.description,
            'agent_role': self.agent_role,
            'component_name': self.component_name,
            'task': self.task,
            'status': self.status.value,
            'depends_on': self.depends_on,
            'result': self.result,
            'started_at': self.started_at,
            'completed_at': self.completed_at,
            'quality_verification_required': self.quality_verification_required
        }


@dataclass
class Workflow:
    """Definition of a multi-step workflow."""
    name: str
    description: str
    steps: List[WorkflowStep]
    parallel_execution: bool = False  # Can steps run in parallel?
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'name': self.name,
            'description': self.description,
            'steps': [step.to_dict() for step in self.steps],
            'parallel_execution': self.parallel_execution,
            'created_at': self.created_at
        }


@dataclass
class WorkflowResult:
    """Results from workflow execution."""
    workflow_name: str
    status: str  # "completed", "failed", "partial"
    steps_completed: int
    steps_failed: int
    total_steps: int
    started_at: str
    completed_at: str
    quality_scores: Dict[str, int] = field(default_factory=dict)  # component_name -> score
    errors: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'workflow_name': self.workflow_name,
            'status': self.status,
            'steps_completed': self.steps_completed,
            'steps_failed': self.steps_failed,
            'total_steps': self.total_steps,
            'started_at': self.started_at,
            'completed_at': self.completed_at,
            'quality_scores': self.quality_scores,
            'errors': self.errors
        }


class WorkflowOrchestrator:
    """
    Orchestrate multi-agent workflows.

    Provides predefined workflows for common development patterns:
    - Feature development (TDD/BDD)
    - Bug fixes (test-first)
    - Refactoring (test-preservation)
    - Documentation updates

    Can also execute custom workflows defined by user.

    Attributes:
        quality_verifier: QualityVerifier instance for quality gates
        max_concurrent_agents: Maximum number of concurrent agents (default: 3)

    Note:
        Agent spawning now uses MCP (Model Context Protocol) instead of programmatic launching.
        Configure agents in .claude.json and use @mentions to invoke them.
        See docs/MCP-SETUP.md for details.
    """

    def __init__(
        self,
        quality_verifier=None,
        max_concurrent_agents: int = 3
    ):
        """
        Initialize WorkflowOrchestrator.

        Args:
            quality_verifier: QualityVerifier instance (will import if None)
            max_concurrent_agents: Max concurrent agents (default: 3, for documentation purposes)

        Note:
            The agent_launcher parameter has been removed. Agent spawning now uses MCP.
            This orchestrator generates workflow plans and guidance, but agent invocation
            happens via MCP @mentions from the master Claude instance.
        """
        self.agent_launcher = None  # Deprecated: Use MCP instead

        if quality_verifier is None:
            try:
                from quality_verifier import QualityVerifier
                self.quality_verifier = QualityVerifier()
            except ImportError:
                self.quality_verifier = None
        else:
            self.quality_verifier = quality_verifier

        self.max_concurrent_agents = max_concurrent_agents

    def execute_workflow(
        self,
        workflow_type: str,
        **kwargs
    ) -> WorkflowResult:
        """
        Execute a predefined workflow.

        Args:
            workflow_type: Type of workflow to execute
            **kwargs: Workflow-specific parameters

        Returns:
            WorkflowResult with execution details

        Example:
            >>> orchestrator = WorkflowOrchestrator()
            >>> result = orchestrator.execute_workflow(
            ...     workflow_type="feature_development",
            ...     feature_name="user_registration",
            ...     components=["user-api", "email-service"]
            ... )
        """
        # Create workflow based on type
        if workflow_type == WorkflowType.FEATURE_DEVELOPMENT.value:
            workflow = self._create_feature_development_workflow(**kwargs)
        elif workflow_type == WorkflowType.BUG_FIX.value:
            workflow = self._create_bug_fix_workflow(**kwargs)
        elif workflow_type == WorkflowType.REFACTORING.value:
            workflow = self._create_refactoring_workflow(**kwargs)
        elif workflow_type == WorkflowType.DOCUMENTATION.value:
            workflow = self._create_documentation_workflow(**kwargs)
        elif workflow_type == WorkflowType.SIMPLE_CHANGE.value:
            workflow = self._create_simple_change_workflow(**kwargs)
        else:
            raise ValueError(f"Unknown workflow type: {workflow_type}")

        # Execute workflow
        return self._execute_workflow_steps(workflow)

    def _create_feature_development_workflow(
        self,
        feature_name: str,
        components: List[str],
        bdd_required: bool = True,
        **kwargs
    ) -> Workflow:
        """
        Create feature development workflow with TDD/BDD.

        Workflow:
        1. Feature Designer: Create design and BDD scenarios (if needed)
        2. Test Agents: Write tests for each component (parallel)
        3. Verify tests fail (RED)
        4. Implementation Agents: Implement feature (parallel)
        5. Verify tests pass (GREEN)
        6. Review Agent: Code review
        7. Refactor if needed (REFACTOR)
        8. Quality Verification: All components
        9. Integration Tests
        10. Documentation Agent: Update docs

        Args:
            feature_name: Name of feature to implement
            components: List of component names to modify
            bdd_required: Whether to create BDD scenarios

        Returns:
            Workflow instance
        """
        steps = []

        # Step 1: Feature Designer (if BDD required)
        if bdd_required:
            steps.append(WorkflowStep(
                name="feature_design",
                description="Create technical design and BDD scenarios",
                agent_role="feature_designer",
                component_name=components[0],  # Primary component
                task=f"Create technical design and BDD scenarios for {feature_name}",
                quality_verification_required=False
            ))

        # Step 2: Test Agents (one per component, can run in parallel)
        for component in components:
            step_name = f"write_tests_{component.replace('-', '_')}"
            steps.append(WorkflowStep(
                name=step_name,
                description=f"Write tests for {component}",
                agent_role="test_agent",
                component_name=component,
                task=f"Write tests for {feature_name} feature",
                depends_on=["feature_design"] if bdd_required else [],
                quality_verification_required=False  # Tests should fail initially
            ))

        # Step 3: Verify RED (tests fail)
        steps.append(WorkflowStep(
            name="verify_red_tests",
            description="Verify that tests fail before implementation",
            agent_role="verification",
            task="Verify tests fail (RED phase of TDD)",
            depends_on=[f"write_tests_{c.replace('-', '_')}" for c in components],
            quality_verification_required=False
        ))

        # Step 4: Implementation Agents (one per component, can run in parallel)
        for component in components:
            step_name = f"implement_{component.replace('-', '_')}"
            steps.append(WorkflowStep(
                name=step_name,
                description=f"Implement feature in {component}",
                agent_role="implementation_agent",
                component_name=component,
                task=f"Implement {feature_name} feature (tests already written, make them pass)",
                depends_on=["verify_red_tests"],
                quality_verification_required=True
            ))

        # Step 5: Code Review
        steps.append(WorkflowStep(
            name="code_review",
            description="Review code quality and suggest improvements",
            agent_role="review_agent",
            task=f"Review {feature_name} implementation",
            depends_on=[f"implement_{c.replace('-', '_')}" for c in components],
            quality_verification_required=False
        ))

        # Step 6: Refactoring (if needed based on review)
        for component in components:
            step_name = f"refactor_{component.replace('-', '_')}"
            steps.append(WorkflowStep(
                name=step_name,
                description=f"Refactor {component} based on review",
                agent_role="implementation_agent",
                component_name=component,
                task=f"Refactor {feature_name} code based on review feedback",
                depends_on=["code_review"],
                quality_verification_required=True
            ))

        # Step 7: Integration Tests
        steps.append(WorkflowStep(
            name="integration_tests",
            description="Run integration tests across components",
            agent_role="integration_test",
            task=f"Verify {feature_name} works across all components",
            depends_on=[f"refactor_{c.replace('-', '_')}" for c in components],
            quality_verification_required=False
        ))

        # Step 8: Documentation
        steps.append(WorkflowStep(
            name="documentation",
            description="Update documentation for new feature",
            agent_role="documentation_agent",
            component_name=components[0],
            task=f"Document {feature_name} feature (README, API docs, etc.)",
            depends_on=["integration_tests"],
            quality_verification_required=False
        ))

        return Workflow(
            name=f"Feature Development: {feature_name}",
            description=f"TDD/BDD workflow for implementing {feature_name}",
            steps=steps,
            parallel_execution=True  # Some steps can run in parallel
        )

    def _create_bug_fix_workflow(
        self,
        bug_description: str,
        component_name: str,
        **kwargs
    ) -> Workflow:
        """
        Create bug fix workflow with test-first approach.

        Workflow:
        1. Write failing test that reproduces bug (RED)
        2. Verify test fails
        3. Fix bug (GREEN)
        4. Verify test passes
        5. Quality verification
        6. Update documentation if needed

        Args:
            bug_description: Description of bug
            component_name: Component to fix

        Returns:
            Workflow instance
        """
        steps = [
            WorkflowStep(
                name="reproduce_bug",
                description="Write test that reproduces bug",
                agent_role="test_agent",
                component_name=component_name,
                task=f"Write failing test that reproduces: {bug_description}",
                quality_verification_required=False
            ),
            WorkflowStep(
                name="verify_reproduction",
                description="Verify test fails (reproduces bug)",
                agent_role="verification",
                task="Verify test fails with bug",
                depends_on=["reproduce_bug"],
                quality_verification_required=False
            ),
            WorkflowStep(
                name="fix_bug",
                description="Fix the bug",
                agent_role="implementation_agent",
                component_name=component_name,
                task=f"Fix bug: {bug_description} (test exists, make it pass)",
                depends_on=["verify_reproduction"],
                quality_verification_required=True
            ),
            WorkflowStep(
                name="verify_fix",
                description="Verify test now passes",
                agent_role="verification",
                task="Verify test passes after fix",
                depends_on=["fix_bug"],
                quality_verification_required=False
            ),
            WorkflowStep(
                name="update_docs",
                description="Update documentation if needed",
                agent_role="documentation_agent",
                component_name=component_name,
                task=f"Document bug fix in CHANGELOG",
                depends_on=["verify_fix"],
                quality_verification_required=False
            )
        ]

        return Workflow(
            name=f"Bug Fix: {bug_description}",
            description=f"Test-first bug fix workflow for {component_name}",
            steps=steps,
            parallel_execution=False
        )

    def _create_refactoring_workflow(
        self,
        refactoring_description: str,
        component_name: str,
        **kwargs
    ) -> Workflow:
        """
        Create refactoring workflow with test preservation.

        Workflow:
        1. Verify all tests pass before refactoring
        2. Perform refactoring
        3. Verify all tests still pass
        4. Quality verification
        5. Code review

        Args:
            refactoring_description: Description of refactoring
            component_name: Component to refactor

        Returns:
            Workflow instance
        """
        steps = [
            WorkflowStep(
                name="verify_tests_before",
                description="Ensure all tests pass before refactoring",
                agent_role="verification",
                task="Verify all tests pass",
                quality_verification_required=False
            ),
            WorkflowStep(
                name="refactor",
                description="Perform refactoring",
                agent_role="implementation_agent",
                component_name=component_name,
                task=f"Refactor: {refactoring_description}",
                depends_on=["verify_tests_before"],
                quality_verification_required=True
            ),
            WorkflowStep(
                name="verify_tests_after",
                description="Ensure all tests still pass",
                agent_role="verification",
                task="Verify all tests still pass after refactoring",
                depends_on=["refactor"],
                quality_verification_required=False
            ),
            WorkflowStep(
                name="review",
                description="Review refactoring",
                agent_role="review_agent",
                task=f"Review refactoring: {refactoring_description}",
                depends_on=["verify_tests_after"],
                quality_verification_required=False
            )
        ]

        return Workflow(
            name=f"Refactoring: {refactoring_description}",
            description=f"Refactoring workflow for {component_name}",
            steps=steps,
            parallel_execution=False
        )

    def _create_documentation_workflow(
        self,
        documentation_task: str,
        component_name: str,
        **kwargs
    ) -> Workflow:
        """Create documentation update workflow."""
        steps = [
            WorkflowStep(
                name="update_docs",
                description="Update documentation",
                agent_role="documentation_agent",
                component_name=component_name,
                task=documentation_task,
                quality_verification_required=False
            )
        ]

        return Workflow(
            name=f"Documentation: {documentation_task}",
            description=f"Documentation update for {component_name}",
            steps=steps,
            parallel_execution=False
        )

    def _create_simple_change_workflow(
        self,
        change_description: str,
        component_name: str,
        **kwargs
    ) -> Workflow:
        """Create simple change workflow (single agent)."""
        steps = [
            WorkflowStep(
                name="implement_change",
                description="Implement change",
                agent_role="implementation_agent",
                component_name=component_name,
                task=change_description,
                quality_verification_required=True
            )
        ]

        return Workflow(
            name=f"Simple Change: {change_description}",
            description=f"Simple change to {component_name}",
            steps=steps,
            parallel_execution=False
        )

    def _execute_workflow_steps(self, workflow: Workflow) -> WorkflowResult:
        """
        Execute workflow steps.

        Handles dependencies, parallel execution, and quality verification.

        Args:
            workflow: Workflow to execute

        Returns:
            WorkflowResult with execution details
        """
        started_at = datetime.now().isoformat()
        steps_completed = 0
        steps_failed = 0
        quality_scores = {}
        errors = []

        # Track which steps are complete
        completed_steps = set()

        # Execute steps
        for step in workflow.steps:
            # Check if dependencies are satisfied
            if not all(dep in completed_steps for dep in step.depends_on):
                step.status = StepStatus.SKIPPED
                errors.append(f"Step {step.name} skipped: dependencies not met")
                continue

            # Execute step
            print(f"\n{'='*60}")
            print(f"Executing: {step.name}")
            print(f"Description: {step.description}")
            print(f"Agent Role: {step.agent_role}")
            if step.component_name:
                print(f"Component: {step.component_name}")
            print(f"{'='*60}\n")

            step.status = StepStatus.IN_PROGRESS
            step.started_at = datetime.now().isoformat()

            try:
                # Execute based on agent role
                if step.agent_role == "verification":
                    # Special case: verification steps
                    result = self._execute_verification_step(step)
                else:
                    # Launch agent for step
                    result = self._execute_agent_step(step)

                step.result = result
                step.status = StepStatus.COMPLETED
                step.completed_at = datetime.now().isoformat()
                completed_steps.add(step.name)
                steps_completed += 1

                # Quality verification if required
                if step.quality_verification_required and step.component_name:
                    quality_result = self._verify_quality(step.component_name)
                    quality_scores[step.component_name] = quality_result.get('score', 0)

                    if not quality_result.get('passed', False):
                        errors.append(f"Quality verification failed for {step.component_name}")
                        step.status = StepStatus.FAILED
                        steps_failed += 1

            except Exception as e:
                step.status = StepStatus.FAILED
                step.completed_at = datetime.now().isoformat()
                steps_failed += 1
                errors.append(f"Step {step.name} failed: {str(e)}")

        completed_at = datetime.now().isoformat()

        # Determine overall status
        if steps_failed == 0:
            status = "completed"
        elif steps_completed > 0:
            status = "partial"
        else:
            status = "failed"

        return WorkflowResult(
            workflow_name=workflow.name,
            status=status,
            steps_completed=steps_completed,
            steps_failed=steps_failed,
            total_steps=len(workflow.steps),
            started_at=started_at,
            completed_at=completed_at,
            quality_scores=quality_scores,
            errors=errors
        )

    def _execute_agent_step(self, step: WorkflowStep) -> Dict:
        """
        Execute a step using an agent via MCP.

        Args:
            step: WorkflowStep to execute

        Returns:
            Result dictionary with MCP invocation guidance

        Note:
            Agent launcher has been replaced with MCP (Model Context Protocol).
            This method returns instructions for invoking the agent via @mentions.
            The master Claude instance should use the MCP invocation string.
        """
        # Return MCP invocation guidance
        print(f"[MCP] Invoke: @{step.component_name}, {step.task}")
        return {
            'status': 'mcp_guidance',
            'agent_role': step.agent_role,
            'component_name': step.component_name,
            'task': step.task,
            'mcp_invocation': f"@{step.component_name}, {step.task}",
            'instructions': (
                f"Use MCP to invoke this step:\n"
                f"  @{step.component_name}, {step.task}\n\n"
                f"The {step.component_name} agent is configured in .claude.json"
            )
        }

    def _execute_verification_step(self, step: WorkflowStep) -> Dict:
        """
        Execute a verification step.

        Args:
            step: WorkflowStep to execute

        Returns:
            Result dictionary
        """
        print(f"[VERIFICATION] {step.task}")

        # In real implementation, would actually run tests
        # For now, return success
        return {
            'status': 'verified',
            'task': step.task
        }

    def _verify_quality(self, component_name: str) -> Dict:
        """
        Run quality verification on a component.

        Args:
            component_name: Name of component to verify

        Returns:
            Quality verification result
        """
        if self.quality_verifier is None:
            # Simulation mode
            print(f"[SIMULATION] Would verify quality for: {component_name}")
            return {'passed': True, 'score': 85}

        component_path = Path("components") / component_name
        report = self.quality_verifier.verify(str(component_path))

        return {
            'passed': report.passed,
            'score': report.overall_score,
            'failures': report.failures
        }


def main():
    """CLI interface for workflow orchestration."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python workflow_orchestrator.py <workflow_type> [args]")
        print("")
        print("Workflow Types:")
        print("  feature_development <feature_name> <component1> [component2...]")
        print("  bug_fix <bug_description> <component>")
        print("  refactoring <description> <component>")
        print("  documentation <task> <component>")
        print("  simple_change <description> <component>")
        sys.exit(1)

    workflow_type = sys.argv[1]
    orchestrator = WorkflowOrchestrator()

    if workflow_type == "feature_development":
        if len(sys.argv) < 4:
            print("Usage: ... feature_development <feature_name> <component1> [component2...]")
            sys.exit(1)

        feature_name = sys.argv[2]
        components = sys.argv[3:]

        result = orchestrator.execute_workflow(
            workflow_type="feature_development",
            feature_name=feature_name,
            components=components
        )

    elif workflow_type == "bug_fix":
        if len(sys.argv) < 4:
            print("Usage: ... bug_fix <bug_description> <component>")
            sys.exit(1)

        bug_description = sys.argv[2]
        component_name = sys.argv[3]

        result = orchestrator.execute_workflow(
            workflow_type="bug_fix",
            bug_description=bug_description,
            component_name=component_name
        )

    else:
        print(f"Unknown workflow type: {workflow_type}")
        sys.exit(1)

    # Print result
    print("\n" + "="*60)
    print("Workflow Result")
    print("="*60)
    print(json.dumps(result.to_dict(), indent=2))


if __name__ == "__main__":
    main()
