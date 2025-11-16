"""
Specification Analyzer

Analyzes project specifications to detect architecture patterns and component structure.

This tool helps the orchestration system understand the intended architecture
from specification documents, preventing misinterpretation like enforcing
microservices isolation when a monolithic architecture is specified.

Classes:
    ArchitecturePattern: Detected architecture characteristics
    ComponentSuggestion: Suggested component from specification
    SpecificationAnalyzer: Main analyzer class

Functions:
    analyze_specification: Analyze a specification file or directory

Usage:
    analyzer = SpecificationAnalyzer()
    result = analyzer.analyze_specification("project-spec.md")

    print(f"Architecture: {result.architecture_type}")
    print(f"Components: {len(result.suggested_components)}")
    for component in result.suggested_components:
        print(f"  - {component.name} ({component.type})")
"""

from dataclasses import dataclass, field
from typing import List, Dict, Set, Optional, Tuple
from pathlib import Path
import re
from enum import Enum


class ArchitectureType(Enum):
    """Types of architecture patterns."""
    MONOLITHIC = "monolithic"
    MICROSERVICES = "microservices"
    MODULAR_MONOLITH = "modular_monolith"
    MIXED = "mixed"
    UNKNOWN = "unknown"


class ComponentType(Enum):
    """Component type levels."""
    BASE = "base"
    CORE = "core"
    FEATURE = "feature"
    INTEGRATION = "integration"
    APPLICATION = "application"
    UNKNOWN = "unknown"


@dataclass
class ComponentSuggestion:
    """Suggested component from specification analysis."""
    name: str
    type: ComponentType
    responsibility: str
    dependencies: List[str] = field(default_factory=list)
    tech_stack: List[str] = field(default_factory=list)
    confidence: float = 0.0  # 0.0-1.0


@dataclass
class ArchitecturePattern:
    """Detected architecture characteristics."""
    architecture_type: ArchitectureType
    confidence: float  # 0.0-1.0
    reasoning: List[str]
    indicators: Dict[str, int]  # What patterns were detected

    # Architecture characteristics
    is_integrated: bool = False  # Components import each other directly
    is_isolated: bool = False    # Components communicate via APIs only
    is_layered: bool = False     # Clear layer separation
    is_modular: bool = False     # Modular but integrated

    # Communication patterns
    uses_direct_imports: bool = False
    uses_rest_apis: bool = False
    uses_message_queues: bool = False
    uses_shared_libraries: bool = False


@dataclass
class SpecificationAnalysis:
    """Complete specification analysis result."""
    architecture: ArchitecturePattern
    suggested_components: List[ComponentSuggestion]
    dependencies_detected: List[Tuple[str, str]]  # (from, to)
    tech_stack: Set[str]
    integration_style: str
    warnings: List[str]
    metadata: Dict


class SpecificationAnalyzer:
    """
    Analyze project specifications to detect architecture and component structure.

    This analyzer prevents architectural mismatches by understanding what the
    specification actually describes, not imposing a predetermined pattern.
    """

    # Keywords indicating architectural patterns
    MONOLITHIC_INDICATORS = [
        "monolith", "single application", "integrated system",
        "import from", "shared codebase", "single deployment",
        "directly call", "shared modules", "single executable"
    ]

    MICROSERVICES_INDICATORS = [
        "microservice", "service", "api gateway", "rest api",
        "independent deployment", "service mesh", "container",
        "kubernetes", "docker", "separate deployment"
    ]

    MODULAR_MONOLITH_INDICATORS = [
        "modular", "modules", "libraries", "packages",
        "components", "layered", "bounded context",
        "internal modules", "library dependencies"
    ]

    INTEGRATION_PATTERNS = {
        "direct_imports": [
            "import from", "use library", "link to",
            "include module", "shared library", "static linking"
        ],
        "rest_apis": [
            "rest api", "http endpoint", "api call",
            "web service", "rest interface", "http request"
        ],
        "message_queue": [
            "message queue", "event bus", "pub/sub",
            "kafka", "rabbitmq", "message broker"
        ]
    }

    # Component type indicators
    COMPONENT_TYPE_KEYWORDS = {
        ComponentType.BASE: [
            "data type", "model", "utility", "helper",
            "protocol", "interface", "shared type"
        ],
        ComponentType.CORE: [
            "core logic", "business logic", "domain",
            "service layer", "business rule"
        ],
        ComponentType.FEATURE: [
            "feature", "endpoint", "handler",
            "controller", "api", "workflow"
        ],
        ComponentType.INTEGRATION: [
            "orchestrator", "coordinator", "integration",
            "workflow manager", "main logic", "ties together"
        ],
        ComponentType.APPLICATION: [
            "cli", "main entry", "application", "server",
            "frontend", "gui", "entry point"
        ]
    }

    def __init__(self):
        """Initialize the specification analyzer."""
        self.architecture_indicators = {
            "monolithic": 0,
            "microservices": 0,
            "modular_monolith": 0
        }
        self.integration_patterns = {
            "direct_imports": 0,
            "rest_apis": 0,
            "message_queue": 0
        }

    def analyze_specification(self, spec_path: Path) -> SpecificationAnalysis:
        """
        Analyze specification file(s) to detect architecture.

        Args:
            spec_path: Path to specification file or directory

        Returns:
            SpecificationAnalysis with detected patterns
        """
        spec_path = Path(spec_path)

        # Read specification content
        content = self._read_specification(spec_path)

        # Detect architecture pattern
        architecture = self._detect_architecture(content)

        # Extract component suggestions
        components = self._extract_components(content, architecture)

        # Detect dependencies
        dependencies = self._detect_dependencies(content, components)

        # Extract tech stack
        tech_stack = self._extract_tech_stack(content)

        # Determine integration style
        integration_style = self._determine_integration_style(architecture)

        # Generate warnings for potential issues
        warnings = self._generate_warnings(architecture, components)

        # Metadata
        metadata = {
            "spec_path": str(spec_path),
            "total_components": len(components),
            "architecture_confidence": architecture.confidence
        }

        return SpecificationAnalysis(
            architecture=architecture,
            suggested_components=components,
            dependencies_detected=dependencies,
            tech_stack=tech_stack,
            integration_style=integration_style,
            warnings=warnings,
            metadata=metadata
        )

    def _read_specification(self, spec_path: Path) -> str:
        """Read specification content from file or directory."""
        content = []

        if spec_path.is_file():
            with open(spec_path, 'r', encoding='utf-8') as f:
                content.append(f.read())
        elif spec_path.is_dir():
            # Read all markdown and text files
            for file_path in spec_path.glob("**/*.md"):
                with open(file_path, 'r', encoding='utf-8') as f:
                    content.append(f.read())
            for file_path in spec_path.glob("**/*.txt"):
                with open(file_path, 'r', encoding='utf-8') as f:
                    content.append(f.read())

        return "\n\n".join(content)

    def _detect_architecture(self, content: str) -> ArchitecturePattern:
        """Detect architecture pattern from specification content."""
        content_lower = content.lower()

        # Count indicators for each architecture type
        monolithic_score = sum(
            content_lower.count(indicator)
            for indicator in self.MONOLITHIC_INDICATORS
        )

        microservices_score = sum(
            content_lower.count(indicator)
            for indicator in self.MICROSERVICES_INDICATORS
        )

        modular_monolith_score = sum(
            content_lower.count(indicator)
            for indicator in self.MODULAR_MONOLITH_INDICATORS
        )

        # Count integration patterns
        direct_imports_score = sum(
            content_lower.count(pattern)
            for pattern in self.INTEGRATION_PATTERNS["direct_imports"]
        )

        rest_apis_score = sum(
            content_lower.count(pattern)
            for pattern in self.INTEGRATION_PATTERNS["rest_apis"]
        )

        message_queue_score = sum(
            content_lower.count(pattern)
            for pattern in self.INTEGRATION_PATTERNS["message_queue"]
        )

        # Determine architecture type
        scores = {
            ArchitectureType.MONOLITHIC: monolithic_score,
            ArchitectureType.MICROSERVICES: microservices_score,
            ArchitectureType.MODULAR_MONOLITH: modular_monolith_score
        }

        # Check for mixed architecture
        high_scores = [arch for arch, score in scores.items() if score > 5]
        if len(high_scores) > 1:
            arch_type = ArchitectureType.MIXED
            confidence = 0.7
        else:
            # Use highest score
            if max(scores.values()) == 0:
                arch_type = ArchitectureType.UNKNOWN
                confidence = 0.0
            else:
                arch_type = max(scores, key=scores.get)
                total_score = sum(scores.values())
                confidence = scores[arch_type] / max(total_score, 1)

        # Determine characteristics
        is_integrated = (
            direct_imports_score > 3 or
            modular_monolith_score > 5 or
            monolithic_score > 5
        )

        is_isolated = (
            microservices_score > 5 or
            rest_apis_score > 5
        )

        is_layered = any(
            keyword in content_lower
            for keyword in ["layer", "tier", "level", "hierarchy"]
        )

        is_modular = (
            modular_monolith_score > 3 or
            any(keyword in content_lower for keyword in ["module", "component", "package"])
        )

        # Build reasoning
        reasoning = []
        if monolithic_score > 0:
            reasoning.append(f"Monolithic indicators found: {monolithic_score}")
        if microservices_score > 0:
            reasoning.append(f"Microservices indicators found: {microservices_score}")
        if modular_monolith_score > 0:
            reasoning.append(f"Modular monolith indicators found: {modular_monolith_score}")
        if direct_imports_score > 0:
            reasoning.append(f"Direct import patterns found: {direct_imports_score}")
        if rest_apis_score > 0:
            reasoning.append(f"REST API patterns found: {rest_apis_score}")

        return ArchitecturePattern(
            architecture_type=arch_type,
            confidence=confidence,
            reasoning=reasoning,
            indicators={
                "monolithic": monolithic_score,
                "microservices": microservices_score,
                "modular_monolith": modular_monolith_score,
                "direct_imports": direct_imports_score,
                "rest_apis": rest_apis_score,
                "message_queue": message_queue_score
            },
            is_integrated=is_integrated,
            is_isolated=is_isolated,
            is_layered=is_layered,
            is_modular=is_modular,
            uses_direct_imports=(direct_imports_score > 2),
            uses_rest_apis=(rest_apis_score > 2),
            uses_message_queues=(message_queue_score > 2),
            uses_shared_libraries=(direct_imports_score > 2 or modular_monolith_score > 3)
        )

    def _extract_components(
        self,
        content: str,
        architecture: ArchitecturePattern
    ) -> List[ComponentSuggestion]:
        """Extract suggested components from specification."""
        components = []

        # Look for component/module/service definitions
        patterns = [
            r"(?:component|module|service|library):\s+([a-zA-Z0-9_-]+)",
            r"##\s+([A-Z][a-zA-Z0-9\s]+)(?:Component|Module|Service)",
            r"-\s+([a-zA-Z0-9_-]+)(?:\s+component|\s+module|\s+service)",
        ]

        found_components = set()

        for pattern in patterns:
            matches = re.finditer(pattern, content, re.IGNORECASE)
            for match in matches:
                component_name = match.group(1).strip()
                component_name = re.sub(r'\s+', '-', component_name.lower())
                found_components.add(component_name)

        # Classify each component
        for name in found_components:
            comp_type = self._classify_component(name, content)
            responsibility = self._extract_responsibility(name, content)
            dependencies = self._extract_component_dependencies(name, content)
            tech_stack = self._extract_component_tech_stack(name, content)

            components.append(ComponentSuggestion(
                name=name,
                type=comp_type,
                responsibility=responsibility,
                dependencies=dependencies,
                tech_stack=tech_stack,
                confidence=0.8
            ))

        return components

    def _classify_component(self, name: str, content: str) -> ComponentType:
        """Classify component type based on name and context."""
        name_lower = name.lower()

        # Search for component in context
        pattern = rf"(?:^|\n).*{re.escape(name)}.*?(?:\n|$)"
        matches = re.finditer(pattern, content, re.IGNORECASE | re.MULTILINE)
        context = " ".join(match.group(0) for match in matches).lower()

        # Score each type
        type_scores = {}
        for comp_type, keywords in self.COMPONENT_TYPE_KEYWORDS.items():
            score = sum(
                context.count(keyword) + name_lower.count(keyword)
                for keyword in keywords
            )
            type_scores[comp_type] = score

        # Return highest scoring type
        if max(type_scores.values()) > 0:
            return max(type_scores, key=type_scores.get)

        # Default based on name patterns
        if any(word in name_lower for word in ["type", "model", "util", "helper"]):
            return ComponentType.BASE
        elif any(word in name_lower for word in ["core", "service", "logic"]):
            return ComponentType.CORE
        elif any(word in name_lower for word in ["api", "handler", "controller", "feature"]):
            return ComponentType.FEATURE
        elif any(word in name_lower for word in ["orchestrator", "coordinator", "manager", "main"]):
            return ComponentType.INTEGRATION
        elif any(word in name_lower for word in ["cli", "app", "server", "frontend"]):
            return ComponentType.APPLICATION

        return ComponentType.UNKNOWN

    def _extract_responsibility(self, component_name: str, content: str) -> str:
        """Extract component responsibility from context."""
        # Look for responsibility near component name
        pattern = rf"{re.escape(component_name)}[:\s]+([^.\n]+)"
        match = re.search(pattern, content, re.IGNORECASE)
        if match:
            return match.group(1).strip()

        return "Component responsibility not specified"

    def _extract_component_dependencies(
        self,
        component_name: str,
        content: str
    ) -> List[str]:
        """Extract dependencies for a specific component."""
        dependencies = []

        # Look for dependency patterns near component name
        patterns = [
            rf"{re.escape(component_name)}.*?(?:depends on|uses|requires|imports).*?([a-zA-Z0-9_-]+)",
            rf"([a-zA-Z0-9_-]+).*?(?:used by|required by).*?{re.escape(component_name)}"
        ]

        for pattern in patterns:
            matches = re.finditer(pattern, content, re.IGNORECASE)
            for match in matches:
                dep_name = match.group(1).strip()
                if dep_name != component_name:
                    dependencies.append(dep_name)

        return list(set(dependencies))

    def _extract_component_tech_stack(
        self,
        component_name: str,
        content: str
    ) -> List[str]:
        """Extract tech stack for a specific component."""
        # Common technologies
        technologies = [
            "python", "rust", "javascript", "typescript", "go",
            "fastapi", "flask", "django", "react", "vue", "angular",
            "postgresql", "mysql", "mongodb", "redis",
            "docker", "kubernetes"
        ]

        tech_stack = []

        # Search near component name
        pattern = rf"(?:^|\n).*{re.escape(component_name)}.*?(?:\n|$)"
        matches = re.finditer(pattern, content, re.IGNORECASE | re.MULTILINE)
        context = " ".join(match.group(0) for match in matches).lower()

        for tech in technologies:
            if tech in context:
                tech_stack.append(tech)

        return tech_stack

    def _detect_dependencies(
        self,
        content: str,
        components: List[ComponentSuggestion]
    ) -> List[Tuple[str, str]]:
        """Detect dependencies between components."""
        dependencies = []

        for component in components:
            for dep_name in component.dependencies:
                # Check if dependency is in our component list
                if any(c.name == dep_name for c in components):
                    dependencies.append((component.name, dep_name))

        return dependencies

    def _extract_tech_stack(self, content: str) -> Set[str]:
        """Extract overall tech stack from specification."""
        content_lower = content.lower()

        technologies = {
            "python", "rust", "javascript", "typescript", "go", "java",
            "c++", "c#", "ruby", "php", "swift", "kotlin",
            "fastapi", "flask", "django", "express", "spring",
            "react", "vue", "angular", "svelte",
            "postgresql", "mysql", "mongodb", "redis", "elasticsearch",
            "docker", "kubernetes", "aws", "azure", "gcp"
        }

        found_tech = set()
        for tech in technologies:
            if tech in content_lower:
                found_tech.add(tech)

        return found_tech

    def _determine_integration_style(
        self,
        architecture: ArchitecturePattern
    ) -> str:
        """Determine recommended integration style."""
        if architecture.uses_direct_imports and architecture.is_integrated:
            return "library_imports"
        elif architecture.uses_rest_apis and architecture.is_isolated:
            return "rest_apis"
        elif architecture.uses_message_queues:
            return "message_queue"
        elif architecture.is_modular:
            return "modular_library"
        else:
            return "unknown"

    def _generate_warnings(
        self,
        architecture: ArchitecturePattern,
        components: List[ComponentSuggestion]
    ) -> List[str]:
        """Generate warnings for potential architectural issues."""
        warnings = []

        # Low confidence warning
        if architecture.confidence < 0.5:
            warnings.append(
                f"⚠️ Low confidence ({architecture.confidence:.0%}) in architecture detection. "
                "Specification may need clarification."
            )

        # Mixed patterns warning
        if architecture.architecture_type == ArchitectureType.MIXED:
            warnings.append(
                "⚠️ Mixed architecture detected. Ensure component boundaries are clear."
            )

        # No components found
        if len(components) == 0:
            warnings.append(
                "⚠️ No components detected in specification. "
                "Consider adding explicit component definitions."
            )

        # Integration style unclear
        if not architecture.uses_direct_imports and not architecture.uses_rest_apis:
            warnings.append(
                "⚠️ Integration style unclear. "
                "Specify how components should communicate (imports, APIs, etc.)."
            )

        return warnings


def analyze_specification(spec_path: Path) -> SpecificationAnalysis:
    """
    Convenience function to analyze a specification.

    Args:
        spec_path: Path to specification file or directory

    Returns:
        SpecificationAnalysis result
    """
    analyzer = SpecificationAnalyzer()
    return analyzer.analyze_specification(spec_path)


def main():
    """CLI entry point for specification analyzer."""
    import sys
    import argparse
    import json

    parser = argparse.ArgumentParser(
        description="Analyze project specifications to detect architecture"
    )
    parser.add_argument(
        'spec_path',
        type=Path,
        help='Path to specification file or directory'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output as JSON'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Verbose output'
    )

    args = parser.parse_args()

    if not args.spec_path.exists():
        print(f"Error: Specification path not found: {args.spec_path}")
        return 1

    # Analyze specification
    analyzer = SpecificationAnalyzer()
    result = analyzer.analyze_specification(args.spec_path)

    if args.json:
        # JSON output
        output = {
            "architecture": {
                "type": result.architecture.architecture_type.value,
                "confidence": result.architecture.confidence,
                "characteristics": {
                    "integrated": result.architecture.is_integrated,
                    "isolated": result.architecture.is_isolated,
                    "layered": result.architecture.is_layered,
                    "modular": result.architecture.is_modular
                },
                "patterns": {
                    "direct_imports": result.architecture.uses_direct_imports,
                    "rest_apis": result.architecture.uses_rest_apis,
                    "message_queues": result.architecture.uses_message_queues
                }
            },
            "components": [
                {
                    "name": comp.name,
                    "type": comp.type.value,
                    "responsibility": comp.responsibility,
                    "dependencies": comp.dependencies,
                    "tech_stack": comp.tech_stack,
                    "confidence": comp.confidence
                }
                for comp in result.suggested_components
            ],
            "dependencies": [
                {"from": from_comp, "to": to_comp}
                for from_comp, to_comp in result.dependencies_detected
            ],
            "tech_stack": list(result.tech_stack),
            "integration_style": result.integration_style,
            "warnings": result.warnings
        }
        print(json.dumps(output, indent=2))
    else:
        # Human-readable output
        print("=" * 70)
        print("SPECIFICATION ANALYSIS")
        print("=" * 70)
        print()

        print("Architecture:")
        print(f"  Type: {result.architecture.architecture_type.value}")
        print(f"  Confidence: {result.architecture.confidence:.0%}")
        print(f"  Integration Style: {result.integration_style}")
        print()

        print("Characteristics:")
        print(f"  Integrated: {'Yes' if result.architecture.is_integrated else 'No'}")
        print(f"  Isolated: {'Yes' if result.architecture.is_isolated else 'No'}")
        print(f"  Layered: {'Yes' if result.architecture.is_layered else 'No'}")
        print(f"  Modular: {'Yes' if result.architecture.is_modular else 'No'}")
        print()

        if args.verbose:
            print("Reasoning:")
            for reason in result.architecture.reasoning:
                print(f"  - {reason}")
            print()

        print(f"Components Found: {len(result.suggested_components)}")
        if result.suggested_components:
            for comp in result.suggested_components:
                print(f"  - {comp.name} ({comp.type.value})")
                if args.verbose and comp.responsibility:
                    print(f"      {comp.responsibility}")
                if args.verbose and comp.dependencies:
                    print(f"      Dependencies: {', '.join(comp.dependencies)}")
        print()

        if result.tech_stack:
            print(f"Tech Stack: {', '.join(sorted(result.tech_stack))}")
            print()

        if result.warnings:
            print("Warnings:")
            for warning in result.warnings:
                print(f"  {warning}")
            print()

        print("=" * 70)

        # Recommendations
        print("\nRecommendations:")
        if result.architecture.is_integrated:
            print("  ✅ Use library-style components with direct imports")
            print("  ✅ Components can import from each other's public APIs")
            print("  ✅ Focus on clear public/private boundaries")
        if result.architecture.is_isolated:
            print("  ✅ Use service-style components with API contracts")
            print("  ✅ Define REST/gRPC contracts between components")
            print("  ✅ Focus on API versioning and compatibility")
        if result.architecture.is_layered:
            print("  ✅ Enforce layer hierarchy (base → core → feature → integration → application)")
            print("  ✅ Lower layers should not depend on higher layers")

    return 0


if __name__ == '__main__':
    import sys
    sys.exit(main())
