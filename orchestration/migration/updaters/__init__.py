"""
Language-specific updaters for component renaming.

Each updater handles a specific programming language's import syntax.
"""

from .base_updater import BaseUpdater
from .python_updater import PythonUpdater
from .js_ts_updater import JsTsUpdater
from .rust_updater import RustUpdater
from .go_updater import GoUpdater
from .cpp_updater import CppUpdater
from .java_updater import JavaUpdater

__all__ = [
    'BaseUpdater',
    'PythonUpdater',
    'JsTsUpdater',
    'RustUpdater',
    'GoUpdater',
    'CppUpdater',
    'JavaUpdater',
]

# Mapping of language names to updater classes
UPDATER_MAP = {
    'python': PythonUpdater,
    'javascript': JsTsUpdater,
    'typescript': JsTsUpdater,
    'rust': RustUpdater,
    'go': GoUpdater,
    'cpp': CppUpdater,
    'java': JavaUpdater,
}


def get_updater(language: str, old_name: str, new_name: str) -> BaseUpdater:
    """
    Get appropriate updater for a language.

    Args:
        language: Language name ('python', 'javascript', etc.)
        old_name: Old component name
        new_name: New component name

    Returns:
        Updater instance

    Raises:
        ValueError: If language not supported
    """
    updater_class = UPDATER_MAP.get(language.lower())
    if not updater_class:
        raise ValueError(f"Unsupported language: {language}")

    return updater_class(old_name, new_name)
