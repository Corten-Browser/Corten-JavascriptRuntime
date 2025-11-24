#!/usr/bin/env python3
"""
Progress Reporting for Long Operations

Provides visual feedback during multi-phase operations like onboarding.

Version: 1.0.0
"""

import time
from typing import Optional


class ProgressReporter:
    """Reports progress for multi-phase operations"""

    def __init__(self, total_phases: int = 10):
        """
        Initialize progress reporter.

        Args:
            total_phases: Total number of phases
        """
        self.total_phases = total_phases
        self.current_phase = 0
        self.phase_start_time: Optional[float] = None
        self.overall_start_time: Optional[float] = None

    def start_overall(self):
        """Start overall timing"""
        self.overall_start_time = time.time()

    def start_phase(self, phase: int, description: str):
        """
        Start a phase.

        Args:
            phase: Phase number (1-indexed)
            description: Phase description
        """
        self.current_phase = phase
        self.phase_start_time = time.time()

        progress_pct = ((phase - 1) / self.total_phases) * 100

        print()
        print("=" * 70)
        print(f"Phase {phase}/{self.total_phases}: {description} ({progress_pct:.0f}%)")
        print("=" * 70)
        print()

    def update_progress(self, current: int, total: int, item: str):
        """
        Update progress within phase.

        Args:
            current: Current item number
            total: Total items
            item: Current item description
        """
        if total == 0:
            return

        pct = (current / total) * 100
        bar = self._format_progress_bar(current, total, width=30)

        print(f"\r{bar} {current}/{total} ({pct:.0f}%) - {item[:40]:<40}", end='', flush=True)

        if current == total:
            print()  # Newline after completion

    def complete_phase(self, duration: Optional[float] = None):
        """
        Complete current phase.

        Args:
            duration: Duration in seconds (calculated if not provided)
        """
        if duration is None and self.phase_start_time:
            duration = time.time() - self.phase_start_time

        time_str = self._format_time(duration) if duration else "unknown"
        print(f"\n✓ Phase {self.current_phase} complete ({time_str})\n")

    def complete_overall(self):
        """Complete overall operation"""
        if self.overall_start_time:
            duration = time.time() - self.overall_start_time
            time_str = self._format_time(duration)

            print()
            print("=" * 70)
            print(f"✅ ALL PHASES COMPLETE")
            print(f"Total Time: {time_str}")
            print("=" * 70)

    def report_error(self, error: str):
        """
        Report error during operation.

        Args:
            error: Error message
        """
        print()
        print("❌ ERROR:")
        print(f"   {error}")
        print()

    def _format_progress_bar(self, current: int, total: int, width: int = 20) -> str:
        """Format progress bar"""
        if total == 0:
            filled = 0
        else:
            filled = int((current / total) * width)

        empty = width - filled
        return f"[{'█' * filled}{'░' * empty}]"

    def _format_time(self, seconds: float) -> str:
        """Format time duration"""
        if seconds < 60:
            return f"{seconds:.1f}s"
        elif seconds < 3600:
            minutes = int(seconds / 60)
            secs = int(seconds % 60)
            return f"{minutes}m {secs}s"
        else:
            hours = int(seconds / 3600)
            minutes = int((seconds % 3600) / 60)
            return f"{hours}h {minutes}m"


# Example usage
if __name__ == "__main__":
    import time

    reporter = ProgressReporter(total_phases=3)
    reporter.start_overall()

    # Phase 1
    reporter.start_phase(1, "Analyzing project")
    for i in range(1, 11):
        reporter.update_progress(i, 10, f"File {i}")
        time.sleep(0.1)
    reporter.complete_phase()

    # Phase 2
    reporter.start_phase(2, "Installing components")
    for i in range(1, 21):
        reporter.update_progress(i, 20, f"Component {i}")
        time.sleep(0.05)
    reporter.complete_phase()

    # Phase 3
    reporter.start_phase(3, "Verification")
    for i in range(1, 6):
        reporter.update_progress(i, 5, f"Check {i}")
        time.sleep(0.2)
    reporter.complete_phase()

    reporter.complete_overall()
