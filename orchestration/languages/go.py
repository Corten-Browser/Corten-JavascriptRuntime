"""
Go language support for distribution-ready validation.

Detects hardcoded paths, validates go.mod structure, and verifies
go build/install works correctly.

Part of v0.16.0 multi-language distribution-first redesign.
"""

import re
import shutil
import subprocess
from pathlib import Path
from typing import List, Tuple, Optional

from .base import LanguageSupport, LanguageInfo, HardcodedPath


class GoSupport(LanguageSupport):
    """Go distribution support."""

    @property
    def language_name(self) -> str:
        return "go"

    @property
    def display_name(self) -> str:
        return "Go"

    @property
    def file_extensions(self) -> List[str]:
        return [".go"]

    @property
    def package_files(self) -> List[str]:
        return ["go.mod", "go.sum"]

    def detect(self, project_root: Path) -> Optional[LanguageInfo]:
        """Detect Go project."""
        go_mod = project_root / "go.mod"

        # Collect Go source files
        source_files = self._collect_source_files(project_root)

        if not go_mod.exists() and not source_files:
            return None

        # Filter test files
        test_files = [f for f in source_files if f.name.endswith("_test.go")]

        # Build package files list
        package_files_found = []
        if go_mod.exists():
            package_files_found.append(go_mod)

        go_sum = project_root / "go.sum"
        if go_sum.exists():
            package_files_found.append(go_sum)

        # Confidence: 1.0 if has go.mod, 0.6 if just .go files
        confidence = 1.0 if go_mod.exists() else 0.6

        return LanguageInfo(
            name=self.language_name,
            display_name=self.display_name,
            version=self._detect_go_version(),
            package_files=package_files_found,
            source_files=source_files,
            test_files=test_files,
            confidence=confidence
        )

    def find_hardcoded_paths(self, project_root: Path) -> List[HardcodedPath]:
        """Find hardcoded absolute paths in Go files."""
        hardcoded = []

        # Patterns to detect (primarily Unix, Go is less common on Windows for servers)
        patterns = [
            r'/workspaces/',
            r'/home/',
            r'/Users/',
            r'/root/',
            r'/opt/',
        ]

        # Check go.mod for absolute replace directives
        go_mod = project_root / "go.mod"
        if go_mod.exists():
            try:
                with open(go_mod, 'r', encoding='utf-8') as f:
                    for line_num, line in enumerate(f, 1):
                        # Look for: replace module => /absolute/path
                        if line.strip().startswith('replace'):
                            # Match replace directives with absolute paths
                            match = re.search(r'=>\s+(/[^\s]+)', line)
                            if match:
                                abs_path = match.group(1)
                                hardcoded.append(HardcodedPath(
                                    file_path=Path("go.mod"),
                                    line_number=line_num,
                                    line_content=line.strip()[:80],
                                    path_value=abs_path,
                                    severity="critical"
                                ))
            except Exception:
                pass

        # Scan Go source files
        for source_file in self._collect_source_files(project_root):
            try:
                with open(source_file, 'r', encoding='utf-8', errors='ignore') as f:
                    in_block_comment = False
                    for line_num, line in enumerate(f, 1):
                        # Skip comments
                        stripped = line.strip()

                        # Handle block comments
                        if '/*' in line and '*/' not in line:
                            in_block_comment = True
                            continue
                        if in_block_comment:
                            if '*/' in line:
                                in_block_comment = False
                            continue
                        # Skip single-line block comments
                        if stripped.startswith('/*') and stripped.endswith('*/'):
                            continue
                        if stripped.startswith('//'):
                            continue

                        for pattern in patterns:
                            if re.search(pattern, line):
                                # Extract path from string literal
                                # Go uses both backticks and quotes
                                path_match = re.search(r'["`]([^"`]+)["`]', line)
                                path_value = path_match.group(1) if path_match else "unknown"

                                hardcoded.append(HardcodedPath(
                                    file_path=source_file.relative_to(project_root),
                                    line_number=line_num,
                                    line_content=line.strip()[:80],
                                    path_value=path_value,
                                    severity="critical"
                                ))
                                break  # One per line
            except Exception:
                pass

        return hardcoded

    def check_package_structure(self, project_root: Path) -> Tuple[bool, str, List[str]]:
        """Check go.mod structure."""
        go_mod = project_root / "go.mod"
        issues = []

        if not go_mod.exists():
            return False, "go.mod not found", [
                "Missing go.mod file",
                "Run: go mod init <module-name>"
            ]

        try:
            with open(go_mod, 'r', encoding='utf-8') as f:
                content = f.read()

            # Check for module declaration
            if not re.search(r'^module\s+\S+', content, re.MULTILINE):
                issues.append("go.mod missing module declaration")

            # Check for replace directives with absolute paths
            # Match both single-line: replace foo => /abs/path
            # And block format: foo => /abs/path (inside replace block)
            replace_abs = re.findall(r'^replace\s+\S+\s+=>\s+(/[^\s]+)', content, re.MULTILINE)
            # Also check for block format where lines inside "replace (" block have paths
            replace_block_abs = re.findall(r'^\s+\S+\s+=>\s+(/[^\s]+)', content, re.MULTILINE)
            for abs_path in replace_abs + replace_block_abs:
                issues.append(
                    f"go.mod has replace directive with absolute path: {abs_path}"
                )

            if issues:
                return False, f"Found {len(issues)} issue(s) in go.mod", issues

            return True, "go.mod structure is valid", []

        except Exception as e:
            return False, "Failed to read go.mod", [str(e)]

    def check_import_patterns(self, project_root: Path) -> List[str]:
        """Check for problematic import patterns."""
        issues = []

        # Go doesn't typically have this issue because:
        # - Go imports use module paths (github.com/user/repo)
        # - No filesystem path imports allowed
        # - go.mod defines the module root

        # We could check for GOPATH assumptions in comments/docs, but that's low priority

        return issues

    def verify_installability(self, project_root: Path) -> Tuple[bool, str, Optional[str]]:
        """Verify go build works."""
        if not self.get_package_manager_available():
            return False, "go not available", "go command not found in PATH"

        go_mod = project_root / "go.mod"
        if not go_mod.exists():
            return False, "No go.mod found", "Run 'go mod init <module-name>' first"

        # Try to build all packages
        try:
            result = subprocess.run(
                ["go", "build", "./..."],
                cwd=project_root,
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode != 0:
                return False, "go build failed", result.stderr

            return True, "go build successful", None

        except subprocess.TimeoutExpired:
            return False, "go build timed out", "Build took longer than 5 minutes"
        except Exception as e:
            return False, "go build error", str(e)

    def verify_deployment(
        self,
        project_root: Path,
        deployment_dir: Path,
        test_imports: Optional[List[str]] = None
    ) -> Tuple[bool, str, List[str]]:
        """Verify package builds in different directory."""
        failures = []

        if not self.get_package_manager_available():
            return False, "go not available", ["go command not found in PATH"]

        # Copy source to deployment directory
        try:
            # Create src subdirectory in deployment dir
            src_dir = deployment_dir / "src"
            if src_dir.exists():
                shutil.rmtree(src_dir)

            # Copy all Go files and go.mod
            shutil.copytree(
                project_root,
                src_dir,
                ignore=shutil.ignore_patterns(
                    '__pycache__', '*.pyc', 'node_modules', '.git',
                    'vendor', 'bin', 'pkg'
                )
            )

            # Try to build in deployment directory
            result = subprocess.run(
                ["go", "build", "./..."],
                cwd=src_dir,
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode != 0:
                return False, "go build failed in deployment directory", [result.stderr]

            return True, "Deployment verification passed", []

        except subprocess.TimeoutExpired:
            return False, "go build timed out", ["Build took longer than 5 minutes"]
        except Exception as e:
            return False, "Deployment verification error", [str(e)]

    def get_package_manager_available(self) -> bool:
        """Check if go is available."""
        return shutil.which('go') is not None

    def _detect_go_version(self) -> Optional[str]:
        """Detect Go version."""
        try:
            result = subprocess.run(
                ["go", "version"],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                # Output: "go version go1.21.0 linux/amd64"
                match = re.search(r'go(\d+\.\d+\.\d+)', result.stdout)
                if match:
                    return match.group(1)
        except Exception:
            pass
        return None

    def _collect_source_files(self, project_root: Path) -> List[Path]:
        """Collect all Go source files, excluding vendor directory."""
        source_files = []

        # Directories to exclude
        exclude_dirs = {"vendor", ".git", "node_modules"}

        for go_file in project_root.glob("**/*.go"):
            # Check if file is in excluded directory
            if any(excluded in go_file.parts for excluded in exclude_dirs):
                continue
            source_files.append(go_file)

        return source_files
