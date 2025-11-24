#!/usr/bin/env python3
"""
Pre-commit installation handler with PEP 668 compatibility.

Tries multiple installation methods in order of preference:
1. Already installed (check first)
2. pipx (isolated, recommended)
3. pip --user (user site-packages)
4. apt/dnf/pacman (system package manager)
5. Manual guidance if all fail

Usage:
    python3 orchestration/setup/install_precommit.py
    python3 orchestration/setup/install_precommit.py --verbose
"""

import subprocess
import sys
import shutil
from typing import Optional, Tuple


class PrecommitInstaller:
    """Handles pre-commit installation with multiple fallback methods."""

    def __init__(self, verbose: bool = False):
        self.verbose = verbose

    def log(self, message: str):
        """Print message if verbose mode enabled."""
        if self.verbose:
            print(message)

    def is_installed(self) -> bool:
        """Check if pre-commit is already available."""
        return shutil.which('pre-commit') is not None

    def get_version(self) -> Optional[str]:
        """Get installed pre-commit version."""
        try:
            result = subprocess.run(
                ['pre-commit', '--version'],
                capture_output=True,
                text=True,
                check=True
            )
            # Output: "pre-commit 3.5.0"
            return result.stdout.strip().split()[-1]
        except Exception:
            return None

    def try_pipx_install(self) -> Tuple[bool, str]:
        """Try installing via pipx (preferred)."""
        if not shutil.which('pipx'):
            return False, "pipx not available"

        self.log("Trying pipx install...")
        try:
            subprocess.run(
                ['pipx', 'install', 'pre-commit'],
                capture_output=not self.verbose,
                check=True
            )
            return True, "Installed via pipx"
        except subprocess.CalledProcessError as e:
            return False, f"pipx install failed: {e}"

    def try_pip_user_install(self) -> Tuple[bool, str]:
        """Try installing via pip --user."""
        self.log("Trying pip --user install...")
        try:
            subprocess.run(
                [sys.executable, '-m', 'pip', 'install', '--user', 'pre-commit'],
                capture_output=not self.verbose,
                check=True
            )
            return True, "Installed via pip --user"
        except subprocess.CalledProcessError as e:
            return False, f"pip --user install failed: {e}"

    def try_system_package(self) -> Tuple[bool, str]:
        """Try installing via system package manager."""
        # Detect package manager
        if shutil.which('apt-get'):
            cmd = ['sudo', 'apt-get', 'install', '-y', 'python3-pre-commit']
            pkg_mgr = 'apt-get'
        elif shutil.which('dnf'):
            cmd = ['sudo', 'dnf', 'install', '-y', 'python3-pre-commit']
            pkg_mgr = 'dnf'
        elif shutil.which('pacman'):
            cmd = ['sudo', 'pacman', '-S', '--noconfirm', 'python-pre-commit']
            pkg_mgr = 'pacman'
        else:
            return False, "No supported package manager found"

        self.log(f"Trying {pkg_mgr} install...")
        try:
            subprocess.run(cmd, check=True)
            return True, f"Installed via {pkg_mgr}"
        except subprocess.CalledProcessError as e:
            return False, f"System package install failed: {e}"

    def install(self) -> Tuple[bool, str]:
        """
        Install pre-commit using best available method.

        Returns:
            (success, message)
        """
        # Check if already installed
        if self.is_installed():
            version = self.get_version()
            return True, f"Already installed (v{version})"

        # Try installation methods in order
        methods = [
            ("pipx", self.try_pipx_install),
            ("pip --user", self.try_pip_user_install),
            ("system package", self.try_system_package),
        ]

        for method_name, method_func in methods:
            success, message = method_func()
            if success:
                return True, message

        # All methods failed
        return False, self._get_manual_instructions()

    def _get_manual_instructions(self) -> str:
        """Get manual installation instructions."""
        return """
Could not install pre-commit automatically.

Please install manually using ONE of these methods:

1. Using pipx (recommended):
   python3 -m pip install --user pipx
   pipx ensurepath
   pipx install pre-commit

2. Using pip --user:
   python3 -m pip install --user pre-commit
   # Add to PATH: export PATH="$HOME/.local/bin:$PATH"

3. Using system package manager:
   # Debian/Ubuntu
   sudo apt-get install python3-pre-commit

   # Fedora
   sudo dnf install python3-pre-commit

   # Arch
   sudo pacman -S python-pre-commit

4. Using virtual environment:
   python3 -m venv ~/.venv/pre-commit
   ~/.venv/pre-commit/bin/pip install pre-commit
   sudo ln -s ~/.venv/pre-commit/bin/pre-commit /usr/local/bin/

Then re-run the installation script.
"""


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Install pre-commit package with PEP 668 compatibility'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Show detailed installation output'
    )
    args = parser.parse_args()

    installer = PrecommitInstaller(verbose=args.verbose)

    print("Installing pre-commit package...")
    success, message = installer.install()

    if success:
        print(f"✅ {message}")
        sys.exit(0)
    else:
        print(f"❌ Installation failed")
        print(message)
        sys.exit(1)


if __name__ == "__main__":
    main()
