"""
Rust language support for distribution-ready validation.

Detects hardcoded paths, validates Cargo.toml structure, and verifies
cargo build/install works correctly.

Part of v0.16.0 multi-language distribution-first redesign.
"""

import re
import shutil
import subprocess
from pathlib import Path
from typing import List, Tuple, Optional

from .base import LanguageSupport, LanguageInfo, HardcodedPath


class RustSupport(LanguageSupport):
    """Rust distribution support."""

    @property
    def language_name(self) -> str:
        return "rust"

    @property
    def display_name(self) -> str:
        return "Rust"

    @property
    def file_extensions(self) -> List[str]:
        return [".rs"]

    @property
    def package_files(self) -> List[str]:
        return ["Cargo.toml", "Cargo.lock"]

    def detect(self, project_root: Path) -> Optional[LanguageInfo]:
        """Detect Rust project."""
        cargo_toml = project_root / "Cargo.toml"

        # Collect Rust source files
        source_files = self._collect_source_files(project_root)

        if not cargo_toml.exists() and not source_files:
            return None

        # Build package files list
        package_files_found = []
        if cargo_toml.exists():
            package_files_found.append(cargo_toml)

        cargo_lock = project_root / "Cargo.lock"
        if cargo_lock.exists():
            package_files_found.append(cargo_lock)

        # Confidence: 1.0 if has Cargo.toml, 0.6 if just .rs files
        confidence = 1.0 if cargo_toml.exists() else 0.6

        # Note: Rust tests are typically in the same files as code (#[cfg(test)])
        # So we don't separate test_files

        return LanguageInfo(
            name=self.language_name,
            display_name=self.display_name,
            version=self._detect_rust_version(),
            package_files=package_files_found,
            source_files=source_files,
            test_files=[],  # Tests are inline in Rust
            confidence=confidence
        )

    def find_hardcoded_paths(self, project_root: Path) -> List[HardcodedPath]:
        """Find hardcoded absolute paths in Rust files and Cargo.toml."""
        hardcoded = []

        # Patterns to detect
        patterns = [
            r'/workspaces/',
            r'/home/',
            r'/Users/',
            r'/root/',
            r'/opt/',
            r'C:\\\\',
            r'D:\\\\',
        ]

        # Check Cargo.toml for absolute path dependencies
        cargo_toml = project_root / "Cargo.toml"
        if cargo_toml.exists():
            try:
                with open(cargo_toml, 'r', encoding='utf-8') as f:
                    for line_num, line in enumerate(f, 1):
                        # Look for: path = "/absolute/path"
                        if re.search(r'path\s*=\s*"/', line):
                            path_match = re.search(r'path\s*=\s*"([^"]+)"', line)
                            path_value = path_match.group(1) if path_match else "unknown"

                            hardcoded.append(HardcodedPath(
                                file_path=Path("Cargo.toml"),
                                line_number=line_num,
                                line_content=line.strip()[:80],
                                path_value=path_value,
                                severity="critical"
                            ))
            except Exception:
                pass

        # Scan Rust source files
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
                                path_match = re.search(r'"([^"]+)"', line)
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
        """Check Cargo.toml structure."""
        cargo_toml = project_root / "Cargo.toml"
        issues = []

        if not cargo_toml.exists():
            return False, "Cargo.toml not found", ["Missing Cargo.toml file"]

        # Try to use toml library if available, fallback to text parsing
        try:
            import toml
            use_toml_lib = True
        except ImportError:
            use_toml_lib = False

        try:
            with open(cargo_toml, 'r', encoding='utf-8') as f:
                content = f.read()

            if use_toml_lib:
                # Parse with toml library
                try:
                    cargo_data = toml.loads(content)

                    if "package" not in cargo_data:
                        issues.append("Cargo.toml missing [package] section")
                    elif "name" not in cargo_data["package"]:
                        issues.append("Cargo.toml missing package.name")

                    # Check dependencies for absolute paths
                    for dep_section in ["dependencies", "dev-dependencies", "build-dependencies"]:
                        if dep_section in cargo_data:
                            deps = cargo_data[dep_section]
                            if not isinstance(deps, dict):
                                continue

                            for name, spec in deps.items():
                                if isinstance(spec, dict) and "path" in spec:
                                    if spec["path"].startswith("/") or (
                                        len(spec["path"]) > 1 and spec["path"][1] == ":"
                                    ):
                                        issues.append(
                                            f"Cargo.toml {dep_section}.{name} has absolute path: {spec['path']}"
                                        )
                except Exception as e:
                    # toml parsing failed, fall back to text check
                    use_toml_lib = False

            if not use_toml_lib:
                # Text-based checking (fallback)
                if "[package]" not in content:
                    issues.append("Cargo.toml missing [package] section")

                if not re.search(r'name\s*=\s*"[^"]+"', content):
                    issues.append("Cargo.toml missing package name")

                # Check for absolute paths in dependencies
                abs_path_deps = re.findall(r'path\s*=\s*"(/[^"]+)"', content)
                for abs_path in abs_path_deps:
                    issues.append(f"Cargo.toml has dependency with absolute path: {abs_path}")

                # Check for Windows absolute paths
                win_path_deps = re.findall(r'path\s*=\s*"([A-Z]:[^"]+)"', content)
                for win_path in win_path_deps:
                    issues.append(f"Cargo.toml has dependency with absolute path: {win_path}")

            if issues:
                return False, f"Found {len(issues)} issue(s) in Cargo.toml", issues

            return True, "Cargo.toml structure is valid", []

        except Exception as e:
            return False, "Failed to read Cargo.toml", [str(e)]

    def check_import_patterns(self, project_root: Path) -> List[str]:
        """Check for problematic use/mod patterns."""
        issues = []

        # Rust doesn't typically have hardcoded path issues in imports because:
        # - use statements reference modules, not filesystem paths
        # - mod statements use relative module paths
        # - Cargo.toml defines the crate structure

        return issues

    def verify_installability(self, project_root: Path) -> Tuple[bool, str, Optional[str]]:
        """Verify cargo build works."""
        if not self.get_package_manager_available():
            return False, "cargo not available", "cargo command not found in PATH"

        cargo_toml = project_root / "Cargo.toml"
        if not cargo_toml.exists():
            return False, "No Cargo.toml found", None

        # Try to build
        try:
            result = subprocess.run(
                ["cargo", "build"],
                cwd=project_root,
                capture_output=True,
                text=True,
                timeout=600  # Rust builds can be slow
            )

            if result.returncode != 0:
                return False, "cargo build failed", result.stderr

            return True, "cargo build successful", None

        except subprocess.TimeoutExpired:
            return False, "cargo build timed out", "Build took longer than 10 minutes"
        except Exception as e:
            return False, "cargo build error", str(e)

    def verify_deployment(
        self,
        project_root: Path,
        deployment_dir: Path,
        test_imports: Optional[List[str]] = None
    ) -> Tuple[bool, str, List[str]]:
        """Verify package builds in different directory."""
        failures = []

        if not self.get_package_manager_available():
            return False, "cargo not available", ["cargo command not found in PATH"]

        # Copy source to deployment directory
        try:
            # Create src subdirectory in deployment dir
            src_dir = deployment_dir / "src"
            if src_dir.exists():
                shutil.rmtree(src_dir)

            # Copy all Rust files and Cargo.toml
            shutil.copytree(
                project_root,
                src_dir,
                ignore=shutil.ignore_patterns(
                    '__pycache__', '*.pyc', 'node_modules', '.git',
                    'target', 'bin'
                )
            )

            # Try to build in deployment directory
            result = subprocess.run(
                ["cargo", "build"],
                cwd=src_dir,
                capture_output=True,
                text=True,
                timeout=600
            )

            if result.returncode != 0:
                return False, "cargo build failed in deployment directory", [result.stderr]

            return True, "Deployment verification passed", []

        except subprocess.TimeoutExpired:
            return False, "cargo build timed out", ["Build took longer than 10 minutes"]
        except Exception as e:
            return False, "Deployment verification error", [str(e)]

    def get_package_manager_available(self) -> bool:
        """Check if cargo is available."""
        return shutil.which('cargo') is not None

    def _detect_rust_version(self) -> Optional[str]:
        """Detect Rust version."""
        try:
            result = subprocess.run(
                ["rustc", "--version"],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                # Output: "rustc 1.73.0 (cc66ad468 2023-10-03)"
                match = re.search(r'rustc\s+(\d+\.\d+\.\d+)', result.stdout)
                if match:
                    return match.group(1)
        except Exception:
            pass
        return None

    def _collect_source_files(self, project_root: Path) -> List[Path]:
        """Collect all Rust source files, excluding target directory."""
        source_files = []

        # Directories to exclude
        exclude_dirs = {"target", ".git", "node_modules"}

        for rs_file in project_root.glob("**/*.rs"):
            # Check if file is in excluded directory
            if any(excluded in rs_file.parts for excluded in exclude_dirs):
                continue
            source_files.append(rs_file)

        return source_files
