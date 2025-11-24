#!/usr/bin/env python3
"""
Simple git retry wrapper for handling concurrent commits from multiple agents.
Git's index.lock provides safety; we just need to retry when conflicts occur.
"""

import subprocess
import time
import random
import sys

def git_commit_with_retry(component_name, message, max_retries=5):
    """
    Commit changes with automatic retry on index.lock conflicts.

    Args:
        component_name: Name of component (for commit prefix)
        message: Commit message
        max_retries: Maximum retry attempts (default 5)

    Returns:
        True if successful, False otherwise
    """
    full_message = f"[{component_name}] {message}"

    for attempt in range(max_retries):
        try:
            # Stage only component files
            subprocess.run(
                ['git', 'add', f'components/{component_name}/'],
                check=True,
                capture_output=True,
                text=True
            )

            # Attempt commit
            result = subprocess.run(
                ['git', 'commit', '-m', full_message],
                check=True,
                capture_output=True,
                text=True
            )

            print(f"✅ Commit successful: {full_message}")
            return True

        except subprocess.CalledProcessError as e:
            error_output = e.stderr if e.stderr else e.stdout
            if 'index.lock' in error_output and attempt < max_retries - 1:
                # Calculate delay with exponential backoff + jitter
                delay = (0.5 * (2 ** attempt)) + random.uniform(0, 0.5)
                print(f"⏳ Git lock detected, retrying in {delay:.1f}s (attempt {attempt + 1}/{max_retries})")
                time.sleep(delay)
            elif 'nothing to commit' in error_output:
                print(f"ℹ️  No changes to commit in {component_name}")
                return True
            else:
                print(f"❌ Commit failed: {error_output}")
                return False

    print(f"❌ Failed after {max_retries} attempts")
    return False

if __name__ == "__main__":
    # CLI usage: python git_retry.py <component> <message>
    if len(sys.argv) != 3:
        print("Usage: python git_retry.py <component-name> <commit-message>")
        sys.exit(1)

    success = git_commit_with_retry(sys.argv[1], sys.argv[2])
    sys.exit(0 if success else 1)