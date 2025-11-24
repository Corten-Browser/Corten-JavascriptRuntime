"""
JavaScript/TypeScript language support for distribution-ready validation.

Detects hardcoded paths, validates package.json structure, and verifies
npm installability and deployment.

Part of v0.16.0 multi-language distribution-first redesign.
"""

import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import List, Tuple, Optional

from .base import LanguageSupport, LanguageInfo, HardcodedPath


class JavaScriptSupport(LanguageSupport):
    """JavaScript/TypeScript distribution support."""

    @property
    def language_name(self) -> str:
        return "javascript"

    @property
    def display_name(self) -> str:
        return "JavaScript/TypeScript"

    @property
    def file_extensions(self) -> List[str]:
        return [".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"]

    @property
    def package_files(self) -> List[str]:
        return ["package.json", "tsconfig.json"]

    def detect(self, project_root: Path) -> Optional[LanguageInfo]:
        """Detect JavaScript/TypeScript project."""
        package_json = project_root / "package.json"
        tsconfig = project_root / "tsconfig.json"

        # Check for package.json (strong indicator)
        has_package_json = package_json.exists()

        # If no package.json, check for source files
        if not has_package_json:
            source_files = self._collect_source_files(project_root)
            if not source_files:
                return None
            # Has JS/TS files but no package.json - low confidence
            confidence = 0.6
        else:
            source_files = self._collect_source_files(project_root)
            confidence = 1.0

        # Filter test files
        test_files = [
            f for f in source_files
            if any(pattern in f.name for pattern in ["test", "spec", "__tests__"])
        ]

        # Build package files list
        package_files_found = []
        if has_package_json:
            package_files_found.append(package_json)
        if tsconfig.exists():
            package_files_found.append(tsconfig)

        return LanguageInfo(
            name=self.language_name,
            display_name=self.display_name,
            version=self._detect_node_version(),
            package_files=package_files_found,
            source_files=source_files,
            test_files=test_files,
            confidence=confidence
        )

    def find_hardcoded_paths(self, project_root: Path) -> List[HardcodedPath]:
        """Find hardcoded absolute paths in JavaScript/TypeScript files."""
        hardcoded = []

        # Patterns to detect (Unix and Windows)
        patterns = [
            r'/workspaces/',
            r'/home/',
            r'/Users/',
            r'/root/',
            r'/opt/',
            r'C:\\\\',
            r'D:\\\\',
        ]

        # Scan all source files
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
                        if stripped.startswith('//') or stripped.startswith('*'):
                            continue

                        for pattern in patterns:
                            if re.search(pattern, line):
                                # Extract the path value from string
                                path_match = re.search(r'["\']([^"\']+)["\']', line)
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
        """Check package.json structure."""
        package_json = project_root / "package.json"
        issues = []

        if not package_json.exists():
            return False, "package.json not found", ["Missing package.json file"]

        try:
            with open(package_json, 'r', encoding='utf-8') as f:
                pkg_data = json.load(f)
        except json.JSONDecodeError as e:
            return False, "package.json is invalid JSON", [f"JSON parse error: {str(e)}"]
        except Exception as e:
            return False, "Failed to read package.json", [str(e)]

        # Check required fields
        if "name" not in pkg_data:
            issues.append("package.json missing 'name' field")

        if "version" not in pkg_data:
            issues.append("package.json missing 'version' field")

        # Check for workspace-relative paths in dependencies
        for dep_type in ["dependencies", "devDependencies", "peerDependencies"]:
            if dep_type in pkg_data:
                deps = pkg_data[dep_type]
                if not isinstance(deps, dict):
                    issues.append(f"package.json {dep_type} is not an object")
                    continue

                for name, spec in deps.items():
                    if isinstance(spec, str):
                        # Check for absolute file paths
                        if spec.startswith("file:/"):
                            path = spec[5:]  # Remove "file:"
                            if path.startswith("/") or (len(path) > 1 and path[1] == ":"):
                                issues.append(
                                    f"Absolute file path in {dep_type}: {name} -> {spec}"
                                )

        if issues:
            return False, f"Found {len(issues)} issue(s) in package.json", issues

        return True, "package.json structure is valid", []

    def check_import_patterns(self, project_root: Path) -> List[str]:
        """Check for problematic require()/import patterns."""
        issues = []

        for source_file in self._collect_source_files(project_root):
            try:
                with open(source_file, 'r', encoding='utf-8') as f:
                    content = f.read()

                rel_path = source_file.relative_to(project_root)

                # Check for absolute path in require()
                if re.search(r'require\s*\(\s*["\']/', content):
                    issues.append(
                        f"{rel_path}: Uses absolute path in require()"
                    )

                # Check for absolute path in import
                if re.search(r'import\s+.*\s+from\s+["\']/', content):
                    issues.append(
                        f"{rel_path}: Uses absolute path in import statement"
                    )

                # Check for dynamic import with absolute path
                if re.search(r'import\s*\(\s*["\']/', content):
                    issues.append(
                        f"{rel_path}: Uses absolute path in dynamic import()"
                    )

            except Exception:
                pass

        return issues

    def verify_installability(self, project_root: Path) -> Tuple[bool, str, Optional[str]]:
        """Verify npm install works."""
        if not self.get_package_manager_available():
            return False, "npm not available", "npm command not found in PATH"

        package_json = project_root / "package.json"
        if not package_json.exists():
            return False, "No package.json found", None

        # Run npm install in test directory (without actually installing node_modules to project)
        with tempfile.TemporaryDirectory() as tmpdir:
            test_dir = Path(tmpdir)

            # Copy package.json and package-lock.json if exists
            shutil.copy(package_json, test_dir / "package.json")

            package_lock = project_root / "package-lock.json"
            if package_lock.exists():
                shutil.copy(package_lock, test_dir / "package-lock.json")

            # Run npm install
            try:
                result = subprocess.run(
                    ["npm", "install", "--no-audit", "--no-fund"],
                    cwd=test_dir,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                if result.returncode != 0:
                    return False, "npm install failed", result.stderr

                return True, "npm install successful", None

            except subprocess.TimeoutExpired:
                return False, "npm install timed out", "npm install took longer than 5 minutes"
            except Exception as e:
                return False, "npm install error", str(e)

    def verify_deployment(
        self,
        project_root: Path,
        deployment_dir: Path,
        test_imports: Optional[List[str]] = None
    ) -> Tuple[bool, str, List[str]]:
        """Verify package works in different directory."""
        failures = []

        if not self.get_package_manager_available():
            return False, "npm not available", ["npm command not found in PATH"]

        package_json = project_root / "package.json"
        if not package_json.exists():
            return False, "No package.json found", []

        # Install package to deployment directory
        try:
            result = subprocess.run(
                ["npm", "install", "--no-audit", "--no-fund", str(project_root)],
                cwd=deployment_dir,
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode != 0:
                return False, "Failed to install to deployment directory", [result.stderr]

        except subprocess.TimeoutExpired:
            return False, "npm install timed out", ["Installation took longer than 5 minutes"]
        except Exception as e:
            return False, "npm install error", [str(e)]

        # Test imports if specified
        if test_imports:
            for module_name in test_imports:
                test_code = f'require("{module_name}")'

                try:
                    result = subprocess.run(
                        ["node", "-e", test_code],
                        cwd=deployment_dir,
                        capture_output=True,
                        text=True,
                        timeout=30,
                        env={"NODE_PATH": str(deployment_dir / "node_modules")}
                    )

                    if result.returncode != 0:
                        failures.append(f"Import '{module_name}' failed: {result.stderr}")

                except subprocess.TimeoutExpired:
                    failures.append(f"Import '{module_name}' timed out")
                except Exception as e:
                    failures.append(f"Import '{module_name}' error: {str(e)}")

        if failures:
            return False, f"{len(failures)} import(s) failed in deployment", failures

        return True, "Deployment verification passed", []

    def get_package_manager_available(self) -> bool:
        """Check if npm is available."""
        return shutil.which('npm') is not None

    def _detect_node_version(self) -> Optional[str]:
        """Detect Node.js version."""
        try:
            result = subprocess.run(
                ["node", "--version"],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except Exception:
            pass
        return None

    def _collect_source_files(self, project_root: Path) -> List[Path]:
        """Collect all JavaScript/TypeScript source files, excluding build directories."""
        source_files = []

        # Directories to exclude
        exclude_dirs = {
            "node_modules",
            "dist",
            "build",
            "out",
            ".next",
            ".nuxt",
            "coverage",
            ".cache",
            "public",
            "static"
        }

        for ext in self.file_extensions:
            for file_path in project_root.glob(f"**/*{ext}"):
                # Check if file is in excluded directory
                if any(excluded in file_path.parts for excluded in exclude_dirs):
                    continue
                source_files.append(file_path)

        return source_files
