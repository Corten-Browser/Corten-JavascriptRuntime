"""
User confirmation interface for resume operations.
"""


class ResumeConfirmation:
    """Handle user confirmation for resume."""

    @staticmethod
    def ask_for_specs() -> list:
        """
        Ask user for specification files.

        Returns list of file paths.
        """
        print("\n⚠️ No checkpoint found - need additional context:")
        print("\n1. What was the original specification?")
        print("   [Enter spec file paths separated by commas, or press Enter if none]")
        print("   Example: specifications/music-analyzer.md,contracts/api.yaml")
        print()

        response = input("Specification files: ").strip()

        if not response:
            return []

        specs = [s.strip() for s in response.split(',')]
        return specs

    @staticmethod
    def ask_for_goal() -> str:
        """
        Ask user for original goal/request.

        Returns goal description.
        """
        print("\n2. Original request/goal?")
        print("   [Brief description of what you were implementing]")
        print()

        goal = input("Original goal: ").strip()
        return goal if goal else "Continue orchestration"

    @staticmethod
    def confirm_resume(
        status_display: str,
        plan_display: str
    ) -> bool:
        """
        Display status and plan, ask for confirmation.

        Returns True if user confirms, False otherwise.
        """
        print("\n" + "="*70)
        print(status_display)
        print(plan_display)
        print("="*70)
        print()
        print("Continue with this plan? [Y/n]", end=" ")

        response = input().strip().lower()

        # Default to Yes
        return response in ['', 'y', 'yes']
