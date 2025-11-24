"""
Base classes for language-specific distribution support.

Provides abstract interfaces that each language plugin must implement
to support distribution-ready validation across multiple programming languages.

Part of v0.16.0 multi-language distribution-first redesign.
"""

from abc import ABC, abstractmethod
from pathlib import Path
from typing import List, Tuple, Optional
from dataclasses import dataclass, field


@dataclass
class LanguageInfo:
    """Information about detected language in project."""

    name: str                      # Language identifier (lowercase): "python", "javascript", "go", "rust"
    display_name: str              # Human-readable name: "Python", "JavaScript/TypeScript", "Go", "Rust"
    version: Optional[str]         # Detected language/runtime version (e.g., "3.11.2", "v18.0.0")
    package_files: List[Path]      # Package config files found (setup.py, package.json, go.mod, Cargo.toml)
    source_files: List[Path]       # All source files for this language (*.py, *.js, *.go, *.rs)
    test_files: List[Path]         # Test files identified
    confidence: float              # 0.0-1.0, confidence this language is actually used in project

    def __post_init__(self):
        """Validate confidence is in valid range."""
        if not 0.0 <= self.confidence <= 1.0:
            raise ValueError(f"Confidence must be 0.0-1.0, got {self.confidence}")


@dataclass
class HardcodedPath:
    """A hardcoded absolute path found in source code."""

    file_path: Path                # File containing the hardcoded path (relative to project root)
    line_number: int               # Line number where path was found
    line_content: str              # Line content (truncated to 80 chars for display)
    path_value: str                # The actual hardcoded path extracted
    severity: str = "critical"     # "critical" or "warning"

    def __post_init__(self):
        """Validate severity."""
        if self.severity not in ["critical", "warning"]:
            raise ValueError(f"Severity must be 'critical' or 'warning', got {self.severity}")


class LanguageSupport(ABC):
    """
    Abstract base class for language-specific distribution support.

    Each supported language (Python, JavaScript, Go, Rust, etc.) must implement
    this interface to provide:
    - Language detection
    - Hardcoded path detection
    - Package structure validation
    - Import/require pattern checking
    - Installability verification
    - Deployment verification (works in different directory)

    All methods must be implemented. Graceful degradation (returning empty results
    when language tools unavailable) is preferred over raising exceptions.
    """

    @property
    @abstractmethod
    def language_name(self) -> str:
        """
        Language identifier (lowercase, no spaces).

        Returns:
            "python", "javascript", "go", "rust", etc.
        """
        pass

    @property
    @abstractmethod
    def display_name(self) -> str:
        """
        Human-readable language name for display.

        Returns:
            "Python", "JavaScript/TypeScript", "Go", "Rust", etc.
        """
        pass

    @property
    @abstractmethod
    def file_extensions(self) -> List[str]:
        """
        File extensions for this language (including dot).

        Returns:
            [".py"], [".js", ".jsx", ".ts", ".tsx"], [".go"], [".rs"], etc.
        """
        pass

    @property
    @abstractmethod
    def package_files(self) -> List[str]:
        """
        Package configuration filenames that identify this language.

        Returns:
            ["setup.py", "pyproject.toml"], ["package.json"], ["go.mod"], ["Cargo.toml"], etc.
        """
        pass

    @abstractmethod
    def detect(self, project_root: Path) -> Optional[LanguageInfo]:
        """
        Detect if this language is used in the project.

        Checks for:
        - Package configuration files
        - Source files with matching extensions
        - Project structure patterns

        Args:
            project_root: Absolute path to project root directory

        Returns:
            LanguageInfo if language detected (confidence >= 0.5), None otherwise
        """
        pass

    @abstractmethod
    def find_hardcoded_paths(self, project_root: Path) -> List[HardcodedPath]:
        """
        Find hardcoded absolute paths in source files.

        Scans all source files for patterns indicating hardcoded absolute paths:
        - /workspaces/  (dev containers)
        - /home/        (Linux)
        - /Users/       (macOS)
        - /root/        (Linux root)
        - /opt/         (Linux optional)
        - C:\           (Windows C: drive)
        - D:\           (Windows D: drive)

        Language-specific contexts:
        - Python: sys.path.append('/abs/path'), config = "/abs/path"
        - JavaScript: require('/abs/path'), import '/abs/path'
        - Go: configPath := "/abs/path"
        - Rust: const CONFIG: &str = "/abs/path"

        Args:
            project_root: Absolute path to project root directory

        Returns:
            List of HardcodedPath objects (empty list if none found)
        """
        pass

    @abstractmethod
    def check_package_structure(self, project_root: Path) -> Tuple[bool, str, List[str]]:
        """
        Check if package structure is valid for distribution.

        Validates:
        - Package configuration file exists and is valid
        - Required fields are present
        - No absolute paths in dependencies
        - Structure follows language conventions

        Language-specific checks:
        - Python: setup.py/pyproject.toml valid, find_packages() works
        - JavaScript: package.json valid, no "file:/absolute/path" dependencies
        - Go: go.mod exists, no absolute replace directives
        - Rust: Cargo.toml valid, no absolute path dependencies

        Args:
            project_root: Absolute path to project root directory

        Returns:
            (is_valid, message, issues)
            - is_valid: True if structure is valid
            - message: Summary message
            - issues: List of specific issues found (empty if valid)
        """
        pass

    @abstractmethod
    def check_import_patterns(self, project_root: Path) -> List[str]:
        """
        Check for problematic import/require patterns.

        Detects:
        - Absolute path imports/requires
        - Workspace-relative imports that won't work when installed
        - sys.path manipulation (Python)
        - GOPATH assumptions (Go)

        Args:
            project_root: Absolute path to project root directory

        Returns:
            List of import issues found (empty list if none)
        """
        pass

    @abstractmethod
    def verify_installability(self, project_root: Path) -> Tuple[bool, str, Optional[str]]:
        """
        Verify package can be installed using language package manager.

        Tests:
        - Python: pip install . works
        - JavaScript: npm install works
        - Go: go build ./... works
        - Rust: cargo build works

        Graceful degradation:
        - If package manager not available, return (False, "tool not available", None)
        - Do not raise exceptions

        Args:
            project_root: Absolute path to project root directory

        Returns:
            (success, message, error_details)
            - success: True if installation succeeded
            - message: Summary message
            - error_details: Detailed error output if failed, None if succeeded
        """
        pass

    @abstractmethod
    def verify_deployment(
        self,
        project_root: Path,
        deployment_dir: Path,
        test_imports: Optional[List[str]] = None
    ) -> Tuple[bool, str, List[str]]:
        """
        Verify package works when deployed to different directory.

        This is the ULTIMATE test for distribution readiness:
        1. Install package to deployment_dir (different from project_root)
        2. Test imports/requires work WITHOUT PYTHONPATH or similar hacks
        3. Verify no hardcoded paths break runtime

        Tests:
        - Python: pip install to different dir, import modules
        - JavaScript: npm install, require() modules
        - Go: copy source, go build works
        - Rust: copy source, cargo build works

        Graceful degradation:
        - If package manager not available, return (False, "tool not available", [])

        Args:
            project_root: Original project location
            deployment_dir: Different directory to test deployment in
            test_imports: Modules/packages to test importing (optional)

        Returns:
            (success, message, failures)
            - success: True if deployment works
            - message: Summary message
            - failures: List of specific failures (empty if all passed)
        """
        pass

    @abstractmethod
    def get_package_manager_available(self) -> bool:
        """
        Check if language package manager/compiler is available.

        Checks:
        - Python: pip available
        - JavaScript: npm available
        - Go: go available
        - Rust: cargo available

        Returns:
            True if package manager/compiler is available on system
        """
        pass
