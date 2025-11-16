# Shared Libraries

This directory contains read-only shared code accessible to all components.

## Purpose

Components can import utilities from this directory, but CANNOT modify files here. This enforces component isolation while allowing code reuse.

## Structure

```
shared-libs/
├── README.md (this file)
└── [utility modules]
```

## Usage

From a component:
```python
import sys
sys.path.insert(0, '../../shared-libs')
from some_utility import helper_function
```

## Rules

- ✅ Components can READ files from shared-libs/
- ❌ Components CANNOT WRITE to shared-libs/
- ❌ Components CANNOT modify shared-libs/ files
- ✅ Only master orchestrator can update shared-libs/

## Adding Shared Code

Only the master orchestrator can add code here. If a component needs shared functionality:

1. Request via contract or communication channel
2. Master orchestrator evaluates request
3. Master orchestrator adds to shared-libs/ if appropriate
4. All components can then use the new utility

This prevents components from polluting shared namespace and maintains strict isolation.
