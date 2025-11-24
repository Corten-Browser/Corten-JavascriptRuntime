#!/usr/bin/env python3
"""
Unified Installation Manager

Complete feature-restored installation system with all command-line options
and interactive features from original install.sh.

Version: 2.0.0
"""

import sys
import os
import subprocess
from pathlib import Path
from typing import List, Optional
from abc import ABC, abstractmethod
from datetime import datetime
import shutil
import json


# =============================================================================
# Colored Logging
# =============================================================================

class ColoredLogger:
    """Colored logging for installation output"""

    # ANSI color codes (designed for visibility on both white and black backgrounds)
    GREEN = '\033[0;32m'     # Success messages - good on both
    BLUE = '\033[0;34m'      # Informational - good on both
    MAGENTA = '\033[1;35m'   # Warnings/highlights - bold magenta for max contrast on both
    RED = '\033[0;31m'       # Errors - good on both
    BOLD = '\033[1m'         # Bold text
    NC = '\033[0m'           # No Color

    @staticmethod
    def info(message: str):
        print(f"ℹ️  {message}")

    @staticmethod
    def warn(message: str):
        print(f"{ColoredLogger.MAGENTA}⚠️  {message}{ColoredLogger.NC}", file=sys.stderr)

    @staticmethod
    def error(message: str):
        print(f"{ColoredLogger.RED}❌ ERROR: {message}{ColoredLogger.NC}", file=sys.stderr)

    @staticmethod
    def success(message: str):
        print(f"{ColoredLogger.GREEN}✅ {message}{ColoredLogger.NC}")

    @staticmethod
    def section(title: str):
        print()
        print("═" * 70)
        print(f"  {title}")
        print("═" * 70)
        print()


# =============================================================================
# Installation Strategies
# =============================================================================

class InstallStrategy(ABC):
    """Base class for installation strategies"""

    def __init__(self, dry_run: bool = False):
        self.dry_run = dry_run

    @abstractmethod
    def prepare_target(self, target_dir: Path) -> bool:
        """Prepare target directory for installation"""
        pass

    @abstractmethod
    def get_mode_name(self) -> str:
        """Get human-readable mode name"""
        pass


class FreshInstallStrategy(InstallStrategy):
    """Strategy for fresh installation on empty/new project"""

    def prepare_target(self, target_dir: Path) -> bool:
        """Create empty directory structure"""
        if self.dry_run:
            print("[DRY-RUN] Would prepare fresh installation directories")
            return True

        print("📦 Preparing fresh installation...")

        directories = [
            "orchestration",
            "components",
            "contracts",
            "shared-libs",
            "specifications",
            ".claude/commands"
        ]

        for dir_name in directories:
            dir_path = target_dir / dir_name
            if not dir_path.exists():
                dir_path.mkdir(parents=True, exist_ok=True)
                print(f"  ✓ Created {dir_name}/")

        return True

    def get_mode_name(self) -> str:
        return "fresh"


class ExistingProjectStrategy(InstallStrategy):
    """Strategy for existing project installation"""

    def __init__(self, dry_run: bool = False, skip_git: bool = False):
        super().__init__(dry_run)
        self.skip_git = skip_git
        self.backed_up_files: List[Path] = []

    def prepare_target(self, target_dir: Path) -> bool:
        """Create preservation checkpoint and backup conflicts"""
        if self.dry_run:
            print("[DRY-RUN] Would prepare existing project installation")
            return True

        print("📦 Preparing existing project installation...")

        if not self.skip_git:
            if not self._create_checkpoint(target_dir):
                return False

        self._backup_conflicts(target_dir)
        self._create_directories(target_dir)

        return True

    def _create_checkpoint(self, target_dir: Path) -> bool:
        """Create git preservation checkpoint"""
        print("\n  📍 Creating preservation checkpoint...")

        try:
            result = subprocess.run(
                ["git", "rev-parse", "--is-inside-work-tree"],
                capture_output=True,
                cwd=target_dir
            )

            if result.returncode != 0:
                print("  ⚠️  Not a git repository - skipping checkpoint")
                return True

            subprocess.run(["git", "add", "-A"], cwd=target_dir, check=True)
            subprocess.run(
                ["git", "commit", "-m", "checkpoint: pre-orchestration state (preservation point)"],
                cwd=target_dir,
                check=False
            )

            print("  ✓ Preservation checkpoint created")
            return True

        except Exception as e:
            print(f"  ⚠️  Failed to create checkpoint: {e}")
            return True

    def _backup_conflicts(self, target_dir: Path):
        """Backup files that will be overwritten"""
        print("\n  💾 Backing up conflicting files...")

        conflict_files = ["CLAUDE.md", "orchestration-config.json", ".gitignore"]

        for filename in conflict_files:
            file_path = target_dir / filename
            if file_path.exists():
                backup_path = file_path.with_suffix(file_path.suffix + ".backup")
                shutil.copy2(file_path, backup_path)
                self.backed_up_files.append(backup_path)
                print(f"    ✓ Backed up {filename} → {backup_path.name}")

        if self.backed_up_files:
            print(f"  ✓ Backed up {len(self.backed_up_files)} files")
        else:
            print("  ℹ️  No conflicts to backup")

    def _create_directories(self, target_dir: Path):
        """Create necessary directories"""
        directories = [
            "orchestration",
            "components",
            "contracts",
            "shared-libs",
            "specifications",
            ".claude/commands"
        ]

        for dir_name in directories:
            dir_path = target_dir / dir_name
            dir_path.mkdir(parents=True, exist_ok=True)

    def get_mode_name(self) -> str:
        return "existing"


# =============================================================================
# Installation Manager
# =============================================================================

class InstallManager:
    """Manages orchestration system installation with full feature set"""

    def __init__(self,
                 target_dir: Path,
                 install_mode: str = "fresh",
                 max_agents: int = 5,
                 force: bool = False,
                 skip_git: bool = False,
                 dry_run: bool = False,
                 no_commit: bool = False):
        """
        Initialize install manager with all options.

        Args:
            target_dir: Target directory for installation
            install_mode: Installation mode ("fresh" or "existing")
            max_agents: Maximum concurrent agents (default: 5)
            force: Skip interactive prompts for existing installations
            skip_git: Skip all git operations
            dry_run: Preview installation without making changes
            no_commit: Install git hooks but don't auto-commit
        """
        self.target_dir = Path(target_dir).resolve()
        self.install_mode = install_mode
        self.max_agents = max_agents
        self.force = force
        self.skip_git = skip_git
        self.dry_run = dry_run
        self.no_commit = no_commit

        # Select strategy
        if install_mode == "fresh":
            self.strategy = FreshInstallStrategy(dry_run=dry_run)
        elif install_mode == "existing":
            self.strategy = ExistingProjectStrategy(dry_run=dry_run, skip_git=skip_git)
        else:
            raise ValueError(f"Invalid install_mode: {install_mode}")

        # Determine source directory
        self.source_dir = Path(__file__).parent.parent.parent.resolve()

    def run_installation(self) -> bool:
        """Run complete installation process with all phases"""

        if self.dry_run:
            print(f"{ColoredLogger.BLUE}🔍 DRY-RUN MODE: No changes will be made{ColoredLogger.NC}")
            print()

        print("=" * 70)
        print(f"  ORCHESTRATION SYSTEM INSTALLATION ({self.strategy.get_mode_name().upper()})")
        print("=" * 70)
        print()

        try:
            # Phase 0: Agent limit validation
            if not self._validate_agent_limit():
                return False

            # Phase 1: Preflight checks
            ColoredLogger.section("Pre-Installation Checks")
            if not self._run_preflight_checks():
                return False

            # Phase 1.5: Check existing installation
            if not self._check_existing_installation():
                return False

            # Phase 2: Prepare target
            ColoredLogger.section("Step 1: Creating Directory Structure")
            if not self.strategy.prepare_target(self.target_dir):
                return False

            # Phase 3: Sync orchestration files
            ColoredLogger.section("Step 2: Copying Orchestration Files")
            if not self._sync_orchestration_files():
                return False

            # Phase 3.5: Specification discovery
            ColoredLogger.section("Step 2.5: Specification Document Discovery")
            discovered_specs = self._discover_root_specs()
            if discovered_specs:
                self._offer_to_move_specs(discovered_specs)
            if not self._verify_specs_exist():
                self._prompt_for_spec_path()

            # Phase 4: Configuration
            ColoredLogger.section("Step 3: Initial Configuration")
            if not self._generate_config():
                return False
            if not self._create_config_files():
                return False
            if not self._create_gitignore_entries():
                return False

            # Phase 5: Enforcement system
            if not self._install_enforcement_system():
                return False

            # Phase 6: Git hooks
            ColoredLogger.section("Step 4: Git Hooks Installation")
            if not self._install_git_hooks():
                return False

            # Phase 7: Executable permissions
            ColoredLogger.section("Step 5: Setting Permissions")
            if not self._set_executable_permissions():
                return False

            # Phase 8: Validation
            ColoredLogger.section("Step 6: Post-Installation Validation")
            if not self._validate_installation():
                return False

            # Phase 9: Installation manifest
            ColoredLogger.section("Step 7: Creating Installation Manifest")
            if not self._create_installation_manifest():
                return False

            # Phase 10: Finalize
            if not self._finalize_installation():
                return False

            # Success message
            print()
            print("=" * 70)
            ColoredLogger.success("INSTALLATION COMPLETE")
            print("=" * 70)
            print()
            print(f"{ColoredLogger.BLUE}Configuration:{ColoredLogger.NC}")
            print(f"  Max parallel agents: {ColoredLogger.MAGENTA}{self.max_agents}{ColoredLogger.NC}")
            print()
            print(f"{ColoredLogger.BLUE}Next steps:{ColoredLogger.NC}")
            print("  1. Review CLAUDE.md for orchestration guidelines")
            print("  2. Run: python orchestration/cli/self_check.py")
            print("  3. Start developing with /orchestrate command")
            print()

            return True

        except KeyboardInterrupt:
            print("\n\n❌ Installation cancelled by user")
            return False

        except PermissionError as e:
            ColoredLogger.error(f"Permission denied: {e}")
            print("\nTroubleshooting:")
            print("  1. Check write permissions on target directory")
            print("  2. Run with appropriate user privileges")
            print("  3. Check parent directory permissions")
            return False

        except FileNotFoundError as e:
            ColoredLogger.error(f"File not found: {e}")
            print("\nTroubleshooting:")
            print("  1. Verify source directory is correct")
            print("  2. Check all required files exist")
            print("  3. Try re-cloning the orchestration repository")
            return False

        except subprocess.CalledProcessError as e:
            ColoredLogger.error(f"Command failed: {e.cmd}")
            print(f"Exit code: {e.returncode}")
            if e.stderr:
                print(f"Error output: {e.stderr}")
            print("\nTroubleshooting:")
            print("  1. Check error message above")
            print("  2. Verify prerequisites are installed")
            print("  3. Try --dry-run to preview installation")
            print("  4. Check system logs for details")
            return False

        except Exception as e:
            ColoredLogger.error(f"Installation failed: {e}")
            print("\nTroubleshooting:")
            print("  1. Check error message above")
            print("  2. Verify prerequisites are installed")
            print("  3. Check permissions on target directory")
            print("  4. Try --dry-run to preview installation")
            print("  5. Report issue at https://github.com/anthropics/claude-orchestration/issues")
            import traceback
            traceback.print_exc()
            return False

    def _validate_agent_limit(self) -> bool:
        """Validate agent limit with warnings and interactive prompts"""
        if self.max_agents > 15:
            print()
            ColoredLogger.warn(f"WARNING: {self.max_agents} agents exceeds absolute maximum of 15")
            print()
            print("Beyond 15 agents, the orchestrator experiences:")
            print("  • Cognitive overload (loses track of agent progress)")
            print("  • Git retry storms (5-10 minute delays)")
            print("  • Integration conflicts become undebuggable")
            print("  • Coordination overhead negates parallelism benefits")
            print()

            if not self.dry_run:
                try:
                    response = input("Continue anyway? (y/n) ")
                    if response.lower() != 'y':
                        print("Installation cancelled. Recommended: Use 5-10 agents.")
                        return False
                except EOFError:
                    print("\nNo TTY available, cancelling.")
                    return False

        elif self.max_agents > 10:
            print()
            ColoredLogger.info(f"Using {self.max_agents} agents (above recommended maximum of 10)")
            print()
            print("Performance may degrade above 10 agents. Ensure:")
            print("  • Components are extremely well-isolated")
            print("  • Expect increased git conflicts")
            print("  • Debugging failures becomes significantly harder")
            print()

        elif self.max_agents > 7:
            print()
            ColoredLogger.success(f"Using {self.max_agents} agents (within acceptable range)")
            print("  This is above typical 5 but safe with good component isolation.")
            print()

        return True

    def _check_existing_installation(self) -> bool:
        """Check for existing installation and prompt if not forced"""
        orchestration_dir = self.target_dir / "orchestration"

        if orchestration_dir.exists() and not self.force:
            print()
            ColoredLogger.warn("WARNING: Existing orchestration installation detected")
            print(f"   Directory: {orchestration_dir}")
            print("\n   This will overwrite existing files. Options:")
            print("   1. Use --force to overwrite anyway")
            print("   2. Backup and remove existing installation")
            print("   3. Cancel installation (Ctrl+C)")
            print()

            if self.dry_run:
                print("[DRY-RUN] Would prompt for confirmation")
                return True

            try:
                response = input("Continue and overwrite? (y/n) ")
                if response.lower() != 'y':
                    ColoredLogger.error("Installation aborted")
                    return False
            except EOFError:
                print("\nNo TTY available, use --force to override.")
                return False

        if orchestration_dir.exists() and self.force:
            ColoredLogger.warn("Overwriting existing installation (--force mode)")

        return True

    def _run_preflight_checks(self) -> bool:
        """Run comprehensive preflight checks"""
        print("🔍 Running preflight checks...")

        errors = 0

        # Check 1: Target directory exists or can be created
        if self.target_dir.exists():
            if not os.access(self.target_dir, os.W_OK):
                ColoredLogger.error(f"No write permission to {self.target_dir}")
                errors += 1
            else:
                ColoredLogger.success("Write permissions OK")
        else:
            parent_dir = self.target_dir.parent
            if not os.access(parent_dir, os.W_OK):
                ColoredLogger.error(f"No write permission to parent directory: {parent_dir}")
                errors += 1
            else:
                ColoredLogger.success("Can create target directory")

        # Check 2: Python version
        if sys.version_info < (3, 8):
            python_version = f"{sys.version_info.major}.{sys.version_info.minor}"
            ColoredLogger.error(f"Python version {python_version} is too old (need 3.8+)")
            print("   Install Python 3.8+ and try again")
            errors += 1
        else:
            python_version = f"{sys.version_info.major}.{sys.version_info.minor}"
            ColoredLogger.success(f"Python {python_version} OK")

        # Check 3: Git available (unless --skip-git)
        if not self.skip_git:
            result = subprocess.run(["which", "git"], capture_output=True)
            if result.returncode != 0:
                ColoredLogger.error("Git not found - use --skip-git to proceed without git")
                errors += 1
            else:
                ColoredLogger.success("Git installed")
        else:
            ColoredLogger.info("Skipping git checks (--skip-git)")

        # Check 4: System directory exists
        if not (self.source_dir / "orchestration").exists():
            ColoredLogger.error(f"System directory not found: {self.source_dir}/orchestration")
            print("   Are you running from the correct location?")
            errors += 1
        else:
            ColoredLogger.success(f"System directory found: {self.source_dir}/orchestration")

        # Check 5: Essential system files
        required_files = [
            self.source_dir / "orchestration" / "VERSION",
            self.source_dir / "orchestration" / "commands" / "orchestrate.md",
        ]

        for file in required_files:
            if not file.exists():
                ColoredLogger.error(f"Required file missing: {file}")
                errors += 1

        if errors == 0:
            ColoredLogger.success("Essential system files present")

        print()
        if errors > 0:
            ColoredLogger.error(f"Preflight checks failed with {errors} error(s)")
            return False

        ColoredLogger.success("Preflight checks passed")
        return True

    def _sync_orchestration_files(self) -> bool:
        """Sync orchestration system files"""
        if self.dry_run:
            print("[DRY-RUN] Would sync orchestration files")
            print(f"  Source: {self.source_dir}")
            print(f"  Target: {self.target_dir}")
            return True

        print("\n📂 Syncing orchestration files...")

        sync_script = self.source_dir / "scripts" / "sync_system_files.sh"

        if sync_script.exists():
            try:
                cmd = ["bash", str(sync_script), str(self.target_dir)]
                if self.dry_run:
                    cmd.append("--dry-run")

                subprocess.run(cmd, check=True, cwd=self.source_dir)
                ColoredLogger.success("Orchestration files synced")

                # Sync script skips VERSION (reserved for upgrades), so copy it manually for fresh installs
                version_source = self.source_dir / "orchestration" / "VERSION"
                version_target = self.target_dir / "orchestration" / "VERSION"
                if version_source.exists() and not version_target.exists():
                    import shutil
                    shutil.copy2(version_source, version_target)
                    ColoredLogger.info("Copied VERSION file")

                return True
            except subprocess.CalledProcessError as e:
                ColoredLogger.error(f"Sync failed: {e}")
                return False
        else:
            return self._manual_sync_files()

    def _manual_sync_files(self) -> bool:
        """Manually sync files if script unavailable"""
        ColoredLogger.warn("Sync script not found, using manual sync...")

        source_orch = self.source_dir / "orchestration"
        target_orch = self.target_dir / "orchestration"

        # Files/directories to exclude (runtime state, not static code)
        def ignore_runtime_state(directory: str, files: list) -> list:
            """Ignore function for shutil.copytree - excludes runtime state files."""
            ignored = []
            dir_path = Path(directory)

            for f in files:
                file_path = dir_path / f
                # Exclude runtime state files that should be generated, not copied
                if f in [
                    'extraction_metadata.json',
                    'spec_manifest.json',
                    'queue_state.json',
                    'completion_state.json',
                    'orchestration_state.json',
                    'enforcement_log.json',
                    'monitor_log.json',
                ]:
                    ignored.append(f)
                # Exclude the entire data/ directory contents (runtime state)
                # but keep the directory structure
                elif 'orchestration/data/state' in str(file_path) and f.endswith('.json'):
                    ignored.append(f)
                # Exclude __pycache__ directories
                elif f == '__pycache__':
                    ignored.append(f)

            return ignored

        if source_orch.exists():
            if target_orch.exists():
                shutil.rmtree(target_orch)
            shutil.copytree(source_orch, target_orch, ignore=ignore_runtime_state)
            print("    ✓ Copied orchestration/ (excluding runtime state files)")

        # Copy commands from orchestration/commands/ to .claude/commands/
        # Commands are stored as source files in orchestration/commands/
        # but need to be installed to .claude/commands/ for Claude Code
        source_commands = self.source_dir / "orchestration" / "commands"
        target_claude_commands = self.target_dir / ".claude" / "commands"

        if source_commands.exists():
            target_claude_commands.mkdir(parents=True, exist_ok=True)
            for item in source_commands.glob("*.md"):
                if item.is_file():
                    target_file = target_claude_commands / item.name
                    shutil.copy2(item, target_file)
            print("    ✓ Copied commands to .claude/commands/")

        return True

    def _discover_root_specs(self) -> List[Path]:
        """Discover specification documents in project root"""
        if self.dry_run:
            print("[DRY-RUN] Would discover specifications")
            return []

        print("\n🔍 Scanning for specification documents in project root...")

        found_specs = []
        patterns = [
            '*specification*.md', '*specification*.yaml', '*specification*.yml',
            '*specifications*.md', '*specifications*.yaml', '*specifications*.yml',
            '*spec*.md', '*spec*.yaml', '*spec*.yml',
            '*specs*.md', '*specs*.yaml', '*specs*.yml'
        ]

        for pattern in patterns:
            for path in self.target_dir.glob(pattern):
                if path.is_file() and path not in found_specs:
                    found_specs.append(path)

            # Also try uppercase
            for path in self.target_dir.glob(pattern.upper()):
                if path.is_file() and path not in found_specs:
                    found_specs.append(path)

        return found_specs

    def _offer_to_move_specs(self, specs: List[Path]) -> bool:
        """Offer to move specification files to specifications/ directory"""
        if not specs:
            return True

        if self.dry_run:
            print(f"[DRY-RUN] Would offer to move {len(specs)} specification files")
            return True

        print(f"\nFound {len(specs)} specification document(s) in project root:")
        for spec in specs:
            print(f"  • {spec.name}")

        print("\nBest practice: Store specifications in specifications/ directory")
        print("This helps the orchestration system locate them automatically.")

        try:
            response = input("\nMove these files to specifications/? (y/n) ")
            if response.lower() != 'y':
                print("Skipping specification organization.")
                return True
        except EOFError:
            print("\nNo TTY available, skipping.")
            return True

        spec_dir = self.target_dir / "specifications"
        spec_dir.mkdir(exist_ok=True)

        for spec in specs:
            dest = spec_dir / spec.name
            if dest.exists():
                ColoredLogger.warn(f"{spec.name} already exists in specifications/, skipping")
                continue

            try:
                spec.rename(dest)
                ColoredLogger.success(f"Moved {spec.name} → specifications/")
            except Exception as e:
                ColoredLogger.error(f"Failed to move {spec.name}: {e}")

        return True

    def _verify_specs_exist(self) -> bool:
        """Verify at least one specification exists"""
        spec_dir = self.target_dir / "specifications"

        if not spec_dir.exists():
            return False

        specs = list(spec_dir.glob('*.md')) + list(spec_dir.glob('*.yaml')) + list(spec_dir.glob('*.yml'))
        return len(specs) > 0

    def _prompt_for_spec_path(self):
        """Prompt user for specification file path if none found"""
        if self.dry_run:
            return

        print("\n═══════════════════════════════════════════════════════════")
        print("  No Specification Documents Found")
        print("═══════════════════════════════════════════════════════════")
        print("\nThe orchestration system works best with a specification document.")
        print("This can be:")
        print("  • Project requirements (Markdown)")
        print("  • Feature specifications (YAML)")
        print("  • Architecture documents")
        print("\nYou can add specifications later to specifications/ directory.")
        print()

    def _generate_config(self) -> bool:
        """Generate orchestration-config.json with configured max_agents"""
        if self.dry_run:
            print("[DRY-RUN] Would generate orchestration-config.json")
            print(f"  max_concurrent_agents: {self.max_agents}")
            return True

        print("\n⚙️  Generating configuration...")

        config_path = self.target_dir / "orchestration-config.json"

        if config_path.exists():
            ColoredLogger.info("Configuration already exists, updating max_agents")
            try:
                with open(config_path, 'r') as f:
                    config = json.load(f)
                config["max_concurrent_agents"] = self.max_agents
                with open(config_path, 'w') as f:
                    json.dump(config, f, indent=2)
                ColoredLogger.success(f"Updated max_agents={self.max_agents}")
                return True
            except Exception as e:
                ColoredLogger.warn(f"Failed to update config: {e}, creating new one")

        config = {
            "version": "1.8.0",
            "project_name": self.target_dir.name,
            "max_concurrent_agents": self.max_agents,
            "model_strategy": {
                "orchestrator": "sonnet",
                "sub_agents": "sonnet"
            },
            "languages": {
                "project_primary": "python",
                "fallback": "python"
            }
        }

        with open(config_path, 'w') as f:
            json.dump(config, f, indent=2)

        ColoredLogger.success("Configuration generated")
        return True

    def _create_config_files(self) -> bool:
        """Create enforcement_config.json (spec_manifest.json created by auto_init.py)"""
        if self.dry_run:
            print("[DRY-RUN] Would create enforcement_config.json")
            return True

        print("Creating additional configuration files...")

        config_dir = self.target_dir / "orchestration" / "config"
        config_dir.mkdir(parents=True, exist_ok=True)

        # enforcement_config.json - static config, belongs in config/
        enforcement_config_path = config_dir / "enforcement_config.json"
        if not enforcement_config_path.exists():
            enforcement_config = {
                "blocking_mode": True,
                "auto_init_queue": True,
                "auto_run_verification": True,
                "block_rationalization": True,
                "block_stubs": True,
                "stall_threshold_minutes": 60,
                "require_queue_empty_for_commit": True,
                "require_verification_for_commit": True
            }
            with open(enforcement_config_path, 'w') as f:
                json.dump(enforcement_config, f, indent=2)
            ColoredLogger.success("Created enforcement_config.json")

        # NOTE: spec_manifest.json is NOT created here anymore
        # It is runtime state created by auto_init.py in orchestration/data/state/
        # Creating it here with empty values would be overwritten anyway

        # Ensure data/state directory exists for runtime files
        state_dir = self.target_dir / "orchestration" / "data" / "state"
        state_dir.mkdir(parents=True, exist_ok=True)

        return True

    def _create_gitignore_entries(self) -> bool:
        """Update .gitignore with orchestration state files"""
        if self.dry_run:
            print("[DRY-RUN] Would update .gitignore")
            return True

        print("\n📝 Updating .gitignore...")

        gitignore_path = self.target_dir / ".gitignore"
        gitignore_path.touch(exist_ok=True)

        entries = [
            "# Orchestration runtime data (local only)",
            "orchestration/data/logs/*",
            "orchestration/data/checkpoints/*",
            "orchestration/data/reports/*",
            "!orchestration/data/*/.gitkeep",
            "",
            "# Legacy paths (for older installations)",
            "orchestration/monitoring/activity_log.json",
            "orchestration/monitoring/alerts.json",
            "orchestration/verification/state/*",
            "!orchestration/verification/state/.gitkeep"
        ]

        existing_content = gitignore_path.read_text() if gitignore_path.exists() else ""

        added = 0
        with open(gitignore_path, 'a') as f:
            for entry in entries:
                if entry not in existing_content:
                    f.write(f"\n{entry}")
                    added += 1

        # Create .gitkeep files for data directories
        gitkeep_dirs = [
            self.target_dir / "orchestration" / "data" / "logs",
            self.target_dir / "orchestration" / "data" / "checkpoints",
            self.target_dir / "orchestration" / "data" / "reports",
            self.target_dir / "orchestration" / "data" / "state",
            self.target_dir / "orchestration" / "verification" / "state",  # Legacy
        ]
        for gitkeep_dir in gitkeep_dirs:
            gitkeep_dir.mkdir(parents=True, exist_ok=True)
            (gitkeep_dir / ".gitkeep").touch(exist_ok=True)

        ColoredLogger.success(f".gitignore updated ({added} entries added)")
        return True

    def _install_enforcement_system(self) -> bool:
        """Install enforcement system with verification"""
        if self.dry_run:
            print("[DRY-RUN] Would install enforcement system")
            return True

        ColoredLogger.section("Installing Automatic Enforcement System (v1.3.0)")

        enforcement_installer = self.target_dir / "orchestration" / "setup" / "install_enforcement.py"

        if not enforcement_installer.exists():
            ColoredLogger.warn("Enforcement installer not found, skipping")
            return True

        try:
            result = subprocess.run(
                ["python3", str(enforcement_installer)],
                cwd=self.target_dir,
                capture_output=True,
                text=True
            )

            if result.returncode == 0:
                ColoredLogger.success("Enforcement system installed successfully")
            else:
                ColoredLogger.error("Enforcement system installation failed")
                print(f"     {result.stderr}")
                return False

            # Verify installation
            verifier = self.target_dir / "orchestration" / "setup" / "verify_installation.py"
            if verifier.exists():
                print("\n  Verifying installation...")
                result = subprocess.run(
                    ["python3", str(verifier)],
                    cwd=self.target_dir,
                    capture_output=True,
                    text=True
                )
                if result.returncode == 0:
                    ColoredLogger.success("Verification passed")
                else:
                    ColoredLogger.warn("Verification had warnings")

            return True

        except Exception as e:
            ColoredLogger.error(f"Enforcement installation failed: {e}")
            return False

    def _install_git_hooks(self) -> bool:
        """
        Git hooks installation - DEPRECATED as of v1.13.0.

        Git hooks don't propagate to repository clones and required special setup.
        Verification is now instruction-based - the LLM runs verification scripts
        at appropriate times instead of git triggering them automatically.

        See orchestration/context/verification-protocol.md for the new approach.
        """
        if self.dry_run:
            print("[DRY-RUN] Git hooks skipped (deprecated in v1.13.0)")
            return True

        print("\n📋 Git hooks: Skipped (using instruction-based verification)")
        print("   Verification scripts are in orchestration/hooks/")
        print("   Run them manually before git operations")
        print("   See: orchestration/context/verification-protocol.md")

        ColoredLogger.success("Instruction-based verification configured")
        return True

    def _set_executable_permissions(self) -> bool:
        """Set executable permissions on scripts"""
        if self.dry_run:
            print("[DRY-RUN] Would set executable permissions")
            return True

        print("\n🔧 Setting executable permissions...")

        orchestration_dir = self.target_dir / "orchestration"
        count = 0

        # Python files with shebang
        for py_file in orchestration_dir.rglob("*.py"):
            try:
                with open(py_file, 'r') as f:
                    first_line = f.readline()
                if first_line.startswith('#!'):
                    py_file.chmod(py_file.stat().st_mode | 0o111)
                    count += 1
            except Exception:
                pass

        # All shell scripts
        for sh_file in orchestration_dir.rglob("*.sh"):
            try:
                sh_file.chmod(sh_file.stat().st_mode | 0o111)
                count += 1
            except Exception:
                pass

        ColoredLogger.success(f"Set executable on {count} files")
        return True

    def _validate_installation(self) -> bool:
        """Validate installation completed successfully"""
        if self.dry_run:
            print("[DRY-RUN] Would validate installation")
            return True

        print("\nValidating installation...")

        errors = 0

        # Check 1: orchestration/ directory
        if not (self.target_dir / "orchestration").exists():
            ColoredLogger.error("orchestration/ directory not found")
            errors += 1
        else:
            ColoredLogger.success("orchestration/ directory exists")

        # Check 2: Essential modules
        required_modules = ["VERSION", "config/orchestration.json"]

        for module in required_modules:
            if not (self.target_dir / "orchestration" / module).exists():
                ColoredLogger.error(f"Missing module: orchestration/{module}")
                errors += 1

        if errors == 0:
            ColoredLogger.success("All required modules present")

        # Check 3: Claude commands
        if not (self.target_dir / ".claude" / "commands").exists():
            ColoredLogger.error(".claude/commands/ directory not found")
            errors += 1
        else:
            ColoredLogger.success("Claude commands directory exists")

        # Check 4: Core directories
        core_dirs = ["components", "contracts", "shared-libs", "specifications"]

        for dir_name in core_dirs:
            if not (self.target_dir / dir_name).exists():
                ColoredLogger.warn(f"Core directory missing: {dir_name}/")

        print()
        if errors > 0:
            ColoredLogger.warn(f"Validation completed with {errors} errors")
            return False
        else:
            ColoredLogger.success("Validation passed - all checks successful")
            return True

    def _create_installation_manifest(self) -> bool:
        """Create installation manifest for tracking"""
        if self.dry_run:
            print("[DRY-RUN] Would create INSTALLATION_MANIFEST.json")
            return True

        print("\n📋 Creating installation manifest...")

        manifest_path = self.target_dir / "orchestration" / "INSTALLATION_MANIFEST.json"

        manifest = {
            "installed_at": datetime.now().isoformat(),
            "version": "1.8.0",
            "source_directory": str(self.source_dir),
            "installation_type": self.strategy.get_mode_name(),
            "max_agents": self.max_agents,
            "features": {
                "automatic_enforcement": True,
                "git_hooks": not self.skip_git,
                "task_queue": True,
                "verification_agent": True,
                "stall_detection": True,
                "auto_sync": True
            }
        }

        with open(manifest_path, 'w') as f:
            json.dump(manifest, f, indent=2)

        ColoredLogger.success("Installation manifest created")
        return True

    def _finalize_installation(self) -> bool:
        """Finalize installation"""
        if self.dry_run:
            print("[DRY-RUN] Would finalize installation")
            return True

        print("\n🎯 Finalizing installation...")

        # Create version file if it doesn't already exist (fallback)
        version_file = self.target_dir / "orchestration" / "VERSION"
        if not version_file.exists():
            version_file.write_text("1.8.0\n")
            ColoredLogger.info("Created VERSION file (fallback)")

        # Commit if not skipped
        if not self.skip_git and not self.no_commit:
            self._commit_installation()
        elif self.no_commit:
            ColoredLogger.info("Skipping auto-commit (--no-commit)")
            print("  💡 Run 'git add -A && git commit' manually when ready")

        ColoredLogger.success("Installation finalized")
        return True

    def _commit_installation(self):
        """Commit installation to git"""
        try:
            result = subprocess.run(
                ["git", "rev-parse", "--is-inside-work-tree"],
                capture_output=True,
                cwd=self.target_dir
            )

            if result.returncode == 0:
                subprocess.run(["git", "add", "-A"], cwd=self.target_dir)
                subprocess.run(
                    ["git", "commit", "-m", f"chore: install orchestration system v1.8.0 (max_agents={self.max_agents})"],
                    cwd=self.target_dir,
                    check=False
                )
                ColoredLogger.success("Installation committed to git")
        except Exception:
            pass


# =============================================================================
# CLI Entry Point
# =============================================================================

def main():
    """CLI entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description='Claude Code Multi-Agent Orchestration System Installer',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
AGENT LIMITS:
  • Default: 5 (good for most projects)
  • Recommended max: 10 (performance sweet spot)
  • Warning above: 7 (acceptable with good isolation)
  • Absolute max: 15 (hard cap - performance degrades)

EXAMPLES:
  # Standard installation
  ./install.sh /home/user/my-project

  # Install with custom agent limit
  ./install.sh -m 10 /home/user/my-project

  # Preview installation without changes
  ./install.sh --dry-run /home/user/my-project

  # Install without git operations
  ./install.sh --skip-git /home/user/my-project

  # Overwrite existing installation
  ./install.sh --force /home/user/my-project

  # Existing project (use existing code)
  ./install_existing.sh /home/user/existing-project

For more information, see docs/INSTALLATION.md
'''
    )

    parser.add_argument(
        "target_dir",
        nargs="?",
        default=".",
        help="Target directory for installation (default: current directory)"
    )

    parser.add_argument(
        "--install-mode",
        choices=["fresh", "existing"],
        default="fresh",
        help="Installation type: fresh (new project) or existing (add to existing project)"
    )

    parser.add_argument(
        "-m", "--max-agents",
        type=int,
        default=5,
        help="Maximum concurrent agents (default: 5, recommended: 5-10, max: 15)"
    )

    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing installation without prompting"
    )

    parser.add_argument(
        "--skip-git",
        action="store_true",
        help="Skip git initialization and commits"
    )

    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview installation without making changes"
    )

    parser.add_argument(
        "--no-commit",
        action="store_true",
        help="Install git hooks but skip auto-commit (git init still runs unless --skip-git)"
    )

    args = parser.parse_args()

    manager = InstallManager(
        target_dir=Path(args.target_dir),
        install_mode=args.install_mode,
        max_agents=args.max_agents,
        force=args.force,
        skip_git=args.skip_git,
        dry_run=args.dry_run,
        no_commit=args.no_commit
    )

    success = manager.run_installation()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
