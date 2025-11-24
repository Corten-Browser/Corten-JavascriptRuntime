"""
Component migration system for safe renaming.

Provides:
- Language detection
- Backup management
- Multi-language import updating (Python, JS/TS, Rust, Go, C++, Java)
- Migration coordination
- CLI tools
"""

from .language_detector import LanguageDetector
from .backup_manager import BackupManager
from .migration_coordinator import MigrationCoordinator

__all__ = [
    'LanguageDetector',
    'BackupManager',
    'MigrationCoordinator',
]
