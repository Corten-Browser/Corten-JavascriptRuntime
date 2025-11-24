"""
Orchestration Context System

This module provides shared context files and utilities for the
orchestration system. Key components:

- component-rules.md: Generic TDD/BDD rules for all components
- orchestration-rules.md: Orchestrator-level coordination rules
- ComponentYamlGenerator: Auto-generate component.yaml from context
"""

from .component_yaml_generator import ComponentYamlGenerator

__all__ = ["ComponentYamlGenerator"]
