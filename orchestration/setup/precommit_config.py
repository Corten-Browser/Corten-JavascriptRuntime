#!/usr/bin/env python3
"""
Manages .pre-commit-config.yaml for orchestration hooks.

Handles:
- Detecting existing pre-commit configurations
- Merging our hooks with user's existing hooks
- Preserving user modifications
- Updating hook definitions during upgrades

Usage:
    python3 orchestration/setup/precommit_config.py /path/to/project
    python3 orchestration/setup/precommit_config.py . --verbose
"""

import sys
from pathlib import Path
from typing import Dict, List, Any, Tuple

# Try to import yaml, provide helpful error if not available
try:
    import yaml
except ImportError:
    print("Error: PyYAML not installed", file=sys.stderr)
    print("Install with: pip install --user PyYAML", file=sys.stderr)
    sys.exit(1)


class PrecommitConfigManager:
    """Manages .pre-commit-config.yaml for orchestration hooks."""

    def __init__(self, project_root: Path, verbose: bool = False):
        self.project_root = Path(project_root)
        self.config_path = self.project_root / '.pre-commit-config.yaml'
        self.verbose = verbose

    def log(self, message: str):
        """Print message if verbose mode enabled."""
        if self.verbose:
            print(message)

    def exists(self) -> bool:
        """Check if .pre-commit-config.yaml exists."""
        return self.config_path.exists()

    def load(self) -> Dict[str, Any]:
        """Load existing config or return empty template."""
        if self.exists():
            self.log(f"Loading existing config: {self.config_path}")
            with open(self.config_path) as f:
                config = yaml.safe_load(f) or {'repos': []}
                return config
        else:
            self.log("No existing config, will create new one")
            return {'repos': []}

    def get_orchestration_hooks(self) -> Dict[str, Any]:
        """Get our hook definitions for deployment to target projects.

        Note: This only includes ENFORCEMENT hooks that apply to all projects.
        Development-specific hooks (tests, version-check) are NOT included here
        as they are specific to the orchestration repo's development workflow.
        """
        return {
            'repo': 'local',
            'hooks': [
                {
                    'id': 'orchestration-naming',
                    'name': 'Component naming validation',
                    'entry': 'python3 orchestration/hooks/pre_commit_naming.py',
                    'language': 'system',
                    'pass_filenames': False,
                    'always_run': True,
                },
                {
                    'id': 'orchestration-enforcement',
                    'name': 'Orchestration enforcement',
                    'entry': 'python3 orchestration/hooks/pre_commit_enforcement.py',
                    'language': 'system',
                    'pass_filenames': False,
                    'always_run': True,
                },
                {
                    'id': 'orchestration-completion-blocker',
                    'name': 'Completion blocker',
                    'entry': 'python3 orchestration/hooks/pre_commit_completion_blocker.py',
                    'language': 'system',
                    'pass_filenames': False,
                    'always_run': True,
                },
            ]
        }

    def has_orchestration_hooks(self, config: Dict[str, Any]) -> bool:
        """Check if orchestration hooks are already configured."""
        for repo in config.get('repos', []):
            if repo.get('repo') == 'local':
                for hook in repo.get('hooks', []):
                    if hook.get('id', '').startswith('orchestration-'):
                        return True
        return False

    def remove_orchestration_hooks(self, config: Dict[str, Any]) -> Dict[str, Any]:
        """Remove existing orchestration hooks (for clean upgrade)."""
        repos_to_keep = []

        for repo in config.get('repos', []):
            if repo.get('repo') == 'local':
                # Filter out orchestration hooks
                non_orch_hooks = [
                    h for h in repo.get('hooks', [])
                    if not h.get('id', '').startswith('orchestration-')
                ]

                # Keep local repo only if it has non-orchestration hooks
                if non_orch_hooks:
                    repo['hooks'] = non_orch_hooks
                    repos_to_keep.append(repo)
                # Otherwise skip this local repo entirely
            else:
                # Keep non-local repos
                repos_to_keep.append(repo)

        config['repos'] = repos_to_keep
        return config

    def add_orchestration_hooks(self, config: Dict[str, Any]) -> Dict[str, Any]:
        """Add orchestration hooks to config."""
        orch_repo = self.get_orchestration_hooks()

        # Find existing local repo or create new
        local_repo = None
        for repo in config.get('repos', []):
            if repo.get('repo') == 'local':
                local_repo = repo
                break

        if local_repo:
            # Append to existing local hooks
            self.log("Adding to existing local repo")
            local_repo['hooks'].extend(orch_repo['hooks'])
        else:
            # Insert at beginning (run our hooks first)
            self.log("Creating new local repo at beginning")
            config.setdefault('repos', []).insert(0, orch_repo)

        return config

    def install_or_update(self) -> Tuple[bool, str]:
        """
        Install or update orchestration hooks in .pre-commit-config.yaml.

        Returns:
            (created_new, message)
        """
        config = self.load()
        was_new = not self.exists()

        if self.has_orchestration_hooks(config):
            self.log("Orchestration hooks found, removing old versions")

        # Remove old orchestration hooks if present
        config = self.remove_orchestration_hooks(config)

        # Add current orchestration hooks
        config = self.add_orchestration_hooks(config)

        # Write config
        self.log(f"Writing config to: {self.config_path}")
        with open(self.config_path, 'w') as f:
            yaml.safe_dump(config, f, default_flow_style=False, sort_keys=False)

        if was_new:
            return True, "Created .pre-commit-config.yaml with orchestration hooks"
        else:
            return False, "Updated .pre-commit-config.yaml with orchestration hooks"

    def get_user_hooks_count(self, config: Dict[str, Any]) -> int:
        """Count user's non-orchestration hooks."""
        count = 0
        for repo in config.get('repos', []):
            if repo.get('repo') == 'local':
                # Count non-orchestration hooks
                count += sum(
                    1 for h in repo.get('hooks', [])
                    if not h.get('id', '').startswith('orchestration-')
                )
            else:
                # All hooks in non-local repos are user's
                count += len(repo.get('hooks', []))
        return count


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Configure orchestration hooks in .pre-commit-config.yaml'
    )
    parser.add_argument(
        'project_root',
        help='Path to project root directory'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Show detailed output'
    )
    args = parser.parse_args()

    project_root = Path(args.project_root).resolve()
    if not project_root.is_dir():
        print(f"Error: {project_root} is not a directory", file=sys.stderr)
        sys.exit(1)

    manager = PrecommitConfigManager(project_root, verbose=args.verbose)

    # Load existing config to check for user hooks
    existing_config = manager.load()
    user_hooks_before = manager.get_user_hooks_count(existing_config)

    # Install or update
    created_new, message = manager.install_or_update()

    # Report results
    print(f"✅ {message}")

    if user_hooks_before > 0:
        print(f"   Preserved {user_hooks_before} existing user hook(s)")

    if created_new:
        print("\nNext steps:")
        print("  1. Review .pre-commit-config.yaml")
        print("  2. Run: pre-commit install")
    else:
        print("\nOrchestration hooks have been updated.")
        print("Run 'pre-commit install' if hooks aren't working.")

    sys.exit(0)


if __name__ == "__main__":
    main()
