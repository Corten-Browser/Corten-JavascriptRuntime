# Component Types and Responsibilities

This document defines the different types of components in the orchestration system and their interaction patterns.

## Overview

Components are **composable libraries** that form applications. They can import and use each other's public APIs. Token limits determine component granularity, not architectural isolation.

## Component Hierarchy

Components are organized in levels, where higher levels can depend on lower levels:

```
Level 4: Application Components (entry points)
           └─> Level 3: Integration Components (orchestrators)
                        └─> Level 2: Feature Libraries (domain features)
                                     └─> Level 1: Core Libraries (fundamental logic)
                                                  └─> Level 0: Base Libraries (utilities)
```

---

## Level 0: Base Libraries

**Purpose**: Provide fundamental utilities with no dependencies

**Characteristics:**
- No dependencies on other components
- Stateless, pure functions
- Highly reusable across the system
- Stable interfaces

**Examples:**
- `string-utils` - String manipulation functions
- `math-helpers` - Mathematical operations
- `date-time` - Date/time utilities
- `validation` - Input validation functions

**Token Limit:** Strict (< 40,000 tokens)

**Dependencies:** None

**Example Structure:**
```python
# components/string-utils/src/__init__.py
"""Public API for string utilities"""

def slugify(text: str) -> str:
    """Convert text to URL-safe slug."""
    pass

def truncate(text: str, length: int) -> str:
    """Truncate text to specified length."""
    pass

__all__ = ['slugify', 'truncate']
```

---

## Level 1: Core Libraries

**Purpose**: Provide core domain functionality

**Characteristics:**
- Depend only on base libraries
- Implement fundamental domain logic
- Focused on single responsibility
- Well-defined public APIs

**Examples:**
- `parser` - Parse domain-specific formats
- `tokenizer` - Tokenization logic
- `ast-builder` - Abstract syntax tree construction
- `audio-codec` - Audio encoding/decoding

**Token Limit:** Strict (< 60,000 tokens)

**Dependencies:** Base libraries only

**Example Structure:**
```python
# components/parser/src/__init__.py
"""Public API for parser"""

from .api import Parser, ParseResult
from .errors import ParseError

__all__ = ['Parser', 'ParseResult', 'ParseError']

# Can import base libraries
from components.string_utils import slugify
```

---

## Level 2: Feature Libraries

**Purpose**: Implement specific features

**Characteristics:**
- Depend on core and base libraries
- Implement complete features
- May have moderate complexity
- Expose feature-specific APIs

**Examples:**
- `html-parser` - HTML parsing using parser library
- `css-engine` - CSS processing using parser and tokenizer
- `js-interpreter` - JavaScript interpretation
- `audio-analyzer` - Audio analysis using codec library

**Token Limit:** Strict (< 80,000 tokens)

**Dependencies:** Core libraries, base libraries

**Example Structure:**
```python
# components/audio-analyzer/src/__init__.py
"""Public API for audio analyzer"""

from .api import AudioAnalyzer, AnalysisResult

__all__ = ['AudioAnalyzer', 'AnalysisResult']

# Can import from core and base
from components.audio_codec import decode
from components.math_helpers import fft
```

---

## Level 3: Integration Components

**Purpose**: Orchestrate multiple libraries into workflows

**Characteristics:**
- **EXPECTED to import many components**
- Coordinate workflows across libraries
- Implement business logic spanning features
- Provide unified interfaces
- **Flexible token limits** (can be larger)

**Examples:**
- `browser-engine` - Orchestrates HTML, CSS, JS, rendering
- `music-analyzer` - Coordinates scanning, analysis, rating, playlists
- `build-system` - Orchestrates compilation, linking, packaging

**Token Limit:** Flexible (< 100,000 tokens acceptable)

**Dependencies:** Feature libraries, core libraries, base libraries

**Example Structure:**
```python
# components/music-analyzer/src/__init__.py
"""Integration component that orchestrates music analysis workflow"""

from components.audio_analyzer import AudioAnalyzer
from components.benefit_scorer import BenefitScorer
from components.playlist_generator import PlaylistGenerator
from components.data_store import DataStore

class MusicAnalyzer:
    """
    Integration component that coordinates the entire music analysis workflow.

    This component's JOB is to import and orchestrate other components.
    """

    def __init__(self):
        # Initialize all required components
        self.analyzer = AudioAnalyzer()
        self.scorer = BenefitScorer()
        self.generator = PlaylistGenerator()
        self.storage = DataStore()

    def process_directory(self, directory_path: Path) -> AnalysisReport:
        """
        Orchestrate complete workflow across multiple components.
        """
        # Step 1: Analyze audio files
        audio_results = self.analyzer.analyze_directory(directory_path)

        # Step 2: Calculate benefit scores
        scores = self.scorer.calculate_scores(audio_results)

        # Step 3: Store results
        self.storage.save_analysis(audio_results, scores)

        # Step 4: Generate playlists
        playlists = self.generator.create_playlists(scores)

        return AnalysisReport(audio_results, scores, playlists)

__all__ = ['MusicAnalyzer']
```

**Special Rules for Integration Components:**
- ✅ Import from multiple feature libraries
- ✅ Larger token budgets allowed
- ✅ Focus on workflow, not implementation
- ❌ Don't reimplement feature logic

---

## Level 4: Application Components

**Purpose**: Entry points that bootstrap the application

**Characteristics:**
- **User-facing entry points**
- Bootstrap and configure application
- Wire components together
- Handle command-line args, configuration
- **Minimal code** (mostly wiring)

**Examples:**
- `cli` - Command-line interface with main()
- `api-server` - REST API entry point
- `gui` - Graphical user interface
- `main` - Application entry point

**Token Limit:** Minimal (< 20,000 tokens)

**Dependencies:** Integration components, any component

**Example Structure:**
```python
# components/cli/src/main.py
"""Application entry point - CLI interface"""

import argparse
import sys
from pathlib import Path

# Import integration component
from components.music_analyzer import MusicAnalyzer
from components.config_manager import ConfigManager
from components.logger import setup_logger

def main():
    """Application entry point."""
    parser = argparse.ArgumentParser(
        description='Music Analysis System'
    )
    parser.add_argument('directory', type=Path,
                       help='Directory to analyze')
    parser.add_argument('--config', type=Path,
                       help='Configuration file')
    parser.add_argument('--output', type=Path,
                       help='Output directory')

    args = parser.parse_args()

    # Bootstrap application
    config = ConfigManager.load(args.config)
    logger = setup_logger(config.log_level)

    # Create main orchestrator
    analyzer = MusicAnalyzer(config=config, logger=logger)

    # Run application
    try:
        logger.info(f"Analyzing directory: {args.directory}")
        report = analyzer.process_directory(
            args.directory,
            output=args.output
        )
        print(f"✓ Analysis complete: {report.summary}")
        return 0
    except Exception as e:
        logger.error(f"Analysis failed: {e}", exc_info=True)
        return 1

if __name__ == '__main__':
    sys.exit(main())
```

**Special Rules for Application Components:**
- ✅ Import integration and feature components
- ✅ Minimal logic (mostly wiring)
- ✅ Handle user interaction
- ❌ No business logic here

---

## Cross-Cutting: Shared Types

**Purpose**: Type definitions used across components

**Characteristics:**
- Interface definitions
- Data structures
- Protocol/trait definitions
- No implementation logic

**Examples:**
- `shared-types` - Common data structures
- `interfaces` - Abstract interfaces
- `protocols` - Communication protocols

**Token Limit:** Minimal (< 10,000 tokens)

**Dependencies:** None (or only other type libraries)

**Example Structure:**
```python
# components/shared-types/src/__init__.py
"""Shared type definitions"""

from dataclasses import dataclass
from typing import Protocol
from pathlib import Path

@dataclass
class AudioFile:
    """Represents an audio file."""
    path: Path
    format: str
    duration: float
    sample_rate: int

class Analyzer(Protocol):
    """Protocol for audio analyzers."""
    def analyze(self, file: AudioFile) -> AnalysisResult:
        ...

__all__ = ['AudioFile', 'Analyzer']
```

---

## Dependency Rules

### Allowed Dependencies (By Level)

| Component Level | Can Depend On |
|----------------|---------------|
| Base (L0) | Nothing |
| Core (L1) | Base |
| Feature (L2) | Core, Base |
| Integration (L3) | Feature, Core, Base |
| Application (L4) | All levels |

### Forbidden Dependencies

- ❌ Lower levels depending on higher levels
- ❌ Circular dependencies between components
- ❌ Depending on private implementation details

---

## Token Management by Type

| Type | Optimal | Warning | Maximum | Action if Exceeded |
|------|---------|---------|---------|-------------------|
| Base | < 40k | 40-50k | 60k | Split into smaller utilities |
| Core | < 60k | 60-80k | 100k | Extract sub-libraries |
| Feature | < 80k | 80-100k | 120k | Split by sub-features |
| Integration | < 100k | 100-120k | 140k | Extract features to libraries |
| Application | < 20k | 20-30k | 40k | Simplify, move logic to integration |

**Key Principle:** Token limits force granular libraries, not architectural isolation.

---

## When to Create Each Type

### Create Base Library When:
- Need utilities used by multiple components
- Logic is stateless and general-purpose
- No dependencies required

### Create Core Library When:
- Implementing fundamental domain logic
- Need focused, single-responsibility component
- Will be used by multiple features

### Create Feature Library When:
- Implementing a complete feature
- Need to compose core libraries
- Feature is self-contained

### Create Integration Component When:
- Need to orchestrate multiple features
- Implementing workflows across components
- Business logic spans multiple libraries

### Create Application Component When:
- Need user-facing entry point
- Bootstrapping the application
- Providing CLI/API/GUI interface

---

## Public API Structure

Every component must clearly define its public API:

### Python
```python
# components/[name]/src/__init__.py
"""Public API exports"""

from .api import PublicClass, public_function
from .types import PublicType

__all__ = ['PublicClass', 'public_function', 'PublicType']

# Private implementation in _internal/
```

### Rust
```rust
// components/[name]/src/lib.rs
//! Public API

pub mod api;
pub use api::{PublicStruct, public_function};

// Private modules
mod internal;
```

### TypeScript
```typescript
// components/[name]/src/index.ts
// Public exports
export { PublicClass } from './api/PublicClass';
export { publicFunction } from './api/functions';
export type { PublicType } from './api/types';

// Internal not exported
```

---

## Summary

**Key Principles:**
1. Components are **composable libraries**, not isolated services
2. Higher-level components **can and should** import lower-level ones
3. Token limits determine **granularity**, not isolation
4. Integration components are **expected** to import many libraries
5. Public APIs enable composition, private details stay hidden
6. Build complex systems from simple, focused components

**The Goal:** Enable building massive applications (like web browsers) from hundreds of small, well-defined, composable components.
