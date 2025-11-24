"""
Language Support Plugin System

Auto-detects which programming languages are used in a project and applies
appropriate distribution-ready validation for each language.

Supported Languages:
- Python (setup.py, pip install)
- JavaScript/TypeScript (package.json, npm install)
- Go (go.mod, go build)
- Rust (Cargo.toml, cargo build)

Part of v0.16.0 multi-language distribution-first redesign.
"""

from typing import List, Tuple
from pathlib import Path

from .base import LanguageSupport, LanguageInfo, HardcodedPath

# Import language implementations
# We'll import these as we create them
try:
    from .python import PythonSupport
except ImportError:
    PythonSupport = None

try:
    from .javascript import JavaScriptSupport
except ImportError:
    JavaScriptSupport = None

try:
    from .go import GoSupport
except ImportError:
    GoSupport = None

try:
    from .rust import RustSupport
except ImportError:
    RustSupport = None


def _build_registry() -> List[LanguageSupport]:
    """
    Build registry of available language support plugins.

    Returns:
        List of LanguageSupport instances for all available languages
    """
    registry = []

    if PythonSupport is not None:
        registry.append(PythonSupport())

    if JavaScriptSupport is not None:
        registry.append(JavaScriptSupport())

    if GoSupport is not None:
        registry.append(GoSupport())

    if RustSupport is not None:
        registry.append(RustSupport())

    return registry


# Global registry of all supported languages
_LANGUAGE_SUPPORT_REGISTRY: List[LanguageSupport] = _build_registry()


def get_all_language_support() -> List[LanguageSupport]:
    """
    Get all registered language support instances.

    Returns:
        List of LanguageSupport instances (empty if no languages available)
    """
    return _LANGUAGE_SUPPORT_REGISTRY.copy()


def detect_project_languages(project_root: Path) -> List[Tuple[LanguageSupport, LanguageInfo]]:
    """
    Detect all programming languages used in a project.

    Runs detection for all registered languages and returns those with
    confidence >= 0.5.

    Args:
        project_root: Absolute path to project root directory

    Returns:
        List of (LanguageSupport, LanguageInfo) tuples, sorted by confidence (highest first)

    Example:
        >>> languages = detect_project_languages(Path("/path/to/project"))
        >>> for lang_support, lang_info in languages:
        ...     print(f"{lang_info.display_name}: {lang_info.confidence}")
        Python: 1.0
        JavaScript/TypeScript: 0.9
    """
    detected = []

    for lang_support in _LANGUAGE_SUPPORT_REGISTRY:
        lang_info = lang_support.detect(project_root)

        # Only include languages with reasonable confidence
        if lang_info and lang_info.confidence >= 0.5:
            detected.append((lang_support, lang_info))

    # Sort by confidence (highest first), then by language name for stability
    detected.sort(key=lambda x: (-x[1].confidence, x[1].name))

    return detected


def get_language_support(language_name: str) -> LanguageSupport:
    """
    Get language support instance by name.

    Args:
        language_name: Language identifier ("python", "javascript", "go", "rust")

    Returns:
        LanguageSupport instance for the language

    Raises:
        ValueError: If language not supported or not available
    """
    for lang_support in _LANGUAGE_SUPPORT_REGISTRY:
        if lang_support.language_name == language_name:
            return lang_support

    available = [ls.language_name for ls in _LANGUAGE_SUPPORT_REGISTRY]
    raise ValueError(
        f"Language '{language_name}' not supported. "
        f"Available: {', '.join(available) if available else 'none'}"
    )


def is_language_supported(language_name: str) -> bool:
    """
    Check if a language is supported.

    Args:
        language_name: Language identifier ("python", "javascript", "go", "rust")

    Returns:
        True if language is supported and available
    """
    try:
        get_language_support(language_name)
        return True
    except ValueError:
        return False


__all__ = [
    # Base classes
    'LanguageSupport',
    'LanguageInfo',
    'HardcodedPath',

    # Language implementations (when available)
    'PythonSupport',
    'JavaScriptSupport',
    'GoSupport',
    'RustSupport',

    # Registry functions
    'get_all_language_support',
    'detect_project_languages',
    'get_language_support',
    'is_language_supported',
]
