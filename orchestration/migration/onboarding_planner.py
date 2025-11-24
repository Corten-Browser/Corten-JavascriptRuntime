#!/usr/bin/env python3
"""
Onboarding Planner - LLM-Guided Onboarding for Existing Projects

Generates structured LLM prompts for analyzing existing projects and planning
their integration with the orchestration system. This tool handles the
LLM-heavy phases of onboarding where human-level understanding is required.

Usage:
    python orchestration/migration/onboarding_planner.py analyze <project_dir>
    python orchestration/migration/onboarding_planner.py plan <project_dir>
    python orchestration/migration/onboarding_planner.py extract-spec <project_dir>

Outputs:
    - analysis_report.md (project structure analysis)
    - onboarding_plan.md (detailed reorganization plan)
    - extracted_spec.yaml (extracted specification)
"""

import sys
import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, field, asdict
from datetime import datetime


@dataclass
class ProjectAnalysis:
    """Results of automated project structure analysis"""
    project_dir: Path
    languages_detected: Dict[str, float]  # {language: confidence}
    source_file_count: int
    total_lines_of_code: int
    existing_components: List[str]
    main_entry_points: List[str]
    dependencies_files: List[str]
    has_tests: bool
    test_framework: Optional[str]
    package_structure: str  # "workspace" or "package" or "flat"
    potential_conflicts: List[str]


@dataclass
class ComponentDiscovery:
    """Discovered component from existing codebase"""
    name: str
    directory: str
    type: str  # cli_application, library, web_server, etc.
    entry_point: Optional[str]
    dependencies: List[str]
    estimated_loc: int
    features: List[str] = field(default_factory=list)


@dataclass
class OnboardingPlan:
    """Complete plan for onboarding an existing project"""
    timestamp: str
    project_name: str
    project_dir: str
    analysis: ProjectAnalysis
    components: List[ComponentDiscovery]
    reorganization_needed: bool
    migration_strategy: str  # "minimal", "moderate", "full_restructure"
    file_moves: List[Tuple[str, str]]  # [(source, destination)]
    import_updates: List[str]  # Files that need import path updates
    manifest_count: int
    contract_count: int
    estimated_duration_minutes: int
    risks: List[str]
    recommendations: List[str]


class OnboardingPlanner:
    """Orchestrates LLM-guided onboarding analysis and planning"""

    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()
        self.analysis: Optional[ProjectAnalysis] = None
        self.plan: Optional[OnboardingPlan] = None

    def run_automated_analysis(self) -> ProjectAnalysis:
        """
        Run automated analysis (no LLM needed)
        Detects language, structure, entry points, etc.
        """
        print("Running automated project analysis...")

        # Detect languages
        languages = self._detect_languages()

        # Count source files
        source_files = self._find_source_files()
        total_loc = self._count_lines_of_code(source_files)

        # Find entry points
        entry_points = self._find_entry_points()

        # Detect existing components
        existing_components = self._detect_existing_components()

        # Find dependency files
        dependency_files = self._find_dependency_files()

        # Check for tests
        has_tests, test_framework = self._detect_tests()

        # Determine package structure
        package_structure = self._detect_package_structure()

        # Check for conflicts
        conflicts = self._check_conflicts()

        self.analysis = ProjectAnalysis(
            project_dir=self.project_dir,
            languages_detected=languages,
            source_file_count=len(source_files),
            total_lines_of_code=total_loc,
            existing_components=existing_components,
            main_entry_points=entry_points,
            dependencies_files=dependency_files,
            has_tests=has_tests,
            test_framework=test_framework,
            package_structure=package_structure,
            potential_conflicts=conflicts
        )

        print(f"✓ Analysis complete: {len(source_files)} source files, {total_loc:,} LOC")
        return self.analysis

    def generate_llm_analysis_prompt(self) -> str:
        """
        Generate LLM prompt for deep project analysis
        (component discovery, feature identification)
        """
        if not self.analysis:
            raise ValueError("Must run automated analysis first")

        prompt = f"""# Task: Analyze Existing Project for Orchestration Onboarding

You are analyzing an existing software project to onboard it into the Claude Code orchestration system.

## Project Information

**Location**: `{self.project_dir}`
**Languages**: {', '.join(f"{lang} ({conf*100:.0f}%)" for lang, conf in self.analysis.languages_detected.items())}
**Size**: {self.analysis.source_file_count} files, {self.analysis.total_lines_of_code:,} lines of code
**Package Structure**: {self.analysis.package_structure}
**Entry Points**: {len(self.analysis.main_entry_points)} found
**Has Tests**: {self.analysis.has_tests}

## Your Task

Analyze the project codebase and provide:

### 1. Component Discovery

Identify logical components in the codebase. For each component:
- **Name**: Component identifier (use snake_case)
- **Type**: One of: cli_application, library, web_server, gui_application, generic
- **Directory**: Current location in codebase
- **Entry Point**: Main file/module (if applicable)
- **Responsibility**: What this component does (1-2 sentences)
- **Features**: List of user-facing features (for testing)
- **Dependencies**: Other components it depends on

### 2. Feature Identification

For each component, list ALL user-facing features:
- CLI commands (for cli_application)
- Public API functions/classes (for library)
- HTTP endpoints (for web_server)
- UI screens/dialogs (for gui_application)

This is CRITICAL for ensuring complete test coverage.

### 3. API Boundaries

Identify where components communicate:
- Function calls between modules
- HTTP API endpoints
- Shared data structures
- Event systems

### 4. Testing Infrastructure

Analyze existing tests:
- Test framework used
- Test coverage (estimated %)
- Unit vs integration vs E2E tests
- Missing test types

## Analysis Guidelines

**Component Boundaries**: Look for:
- Separate directories/modules with distinct responsibilities
- Different entry points (CLI vs library vs web server)
- Clear API boundaries between modules
- Shared vs isolated code

**Component Naming**: Use snake_case, lowercase only:
- ✅ `user_service`, `payment_api`, `cli_interface`
- ❌ `UserService`, `payment-api`, `CLI Interface`

**Component Types**:
- **cli_application**: Has main entry point, runs from command line
- **library**: Imported by other code, no direct execution
- **web_server**: HTTP API, listens on port
- **gui_application**: Desktop GUI (Qt, GTK, etc.)
- **generic**: Other (tests, tools, utilities)

## Output Format

Provide your analysis as structured data:

```json
{{
  "components": [
    {{
      "name": "component_name",
      "type": "cli_application",
      "directory": "relative/path/from/project/root",
      "entry_point": "main.py",
      "responsibility": "Brief description",
      "features": ["feature 1", "feature 2"],
      "dependencies": ["other_component"],
      "estimated_loc": 500,
      "apis": [
        {{
          "type": "function",
          "name": "do_something",
          "description": "What it does"
        }}
      ]
    }}
  ],
  "api_boundaries": [
    {{
      "from_component": "component_a",
      "to_component": "component_b",
      "interface_type": "function_call",
      "methods": ["method1", "method2"]
    }}
  ],
  "testing_status": {{
    "has_unit_tests": true,
    "has_integration_tests": false,
    "has_e2e_tests": false,
    "framework": "pytest",
    "estimated_coverage": 60,
    "missing_test_types": ["integration", "e2e"]
  }},
  "recommendations": [
    "Recommendation 1",
    "Recommendation 2"
  ]
}}
```

## Files to Analyze

Key files to review:
{self._generate_file_list_for_llm()}

## Start Your Analysis

Begin by exploring the project structure, then provide your complete analysis in the JSON format above.
"""

        return prompt

    def generate_llm_planning_prompt(self, component_analysis: Dict) -> str:
        """
        Generate LLM prompt for reorganization planning
        (after component discovery is complete)
        """
        prompt = f"""# Task: Generate Onboarding Reorganization Plan

You have analyzed an existing project and discovered its components. Now create a detailed plan for reorganizing the code to work with the orchestration system.

## Project Information

**Location**: `{self.project_dir}`
**Components Discovered**: {len(component_analysis.get('components', []))}
**Current Structure**: {self.analysis.package_structure}

## Discovered Components

{self._format_components_for_prompt(component_analysis.get('components', []))}

## Orchestration System Requirements

The target structure must be:

```
project/
├── components/              # Isolated component directories
│   ├── component_name_1/   # Each component isolated here
│   │   ├── src/           # Source code
│   │   ├── tests/         # Component tests
│   │   ├── CLAUDE.md      # Component instructions
│   │   └── component.yaml # Manifest
│   └── component_name_2/
│       └── ...
├── contracts/              # API contracts between components
│   ├── component_a_api.yaml
│   └── component_b_api.yaml
├── shared-libs/           # Read-only shared libraries
│   └── shared_types/
├── specifications/        # Specification documents
│   └── system_spec.yaml
└── orchestration/         # System files (already installed)
```

## Your Task

Create a detailed reorganization plan with:

### 1. File Movements

For each file that needs to move:
- **Source**: Current path (relative to project root)
- **Destination**: New path in orchestration structure
- **Reason**: Why this move is necessary
- **Risk**: LOW / MEDIUM / HIGH
- **Git Command**: The exact `git mv` command

### 2. Import Path Updates

For each file that needs import path updates:
- **File**: Path to file
- **Old Imports**: List of import statements that will break
- **New Imports**: List of corrected import statements
- **Impact**: Number of files affected

### 3. Manifest Generation

For each component:
- **Component Name**: Identifier
- **Type**: cli_application / library / web_server / etc.
- **Entry Point**: Main file path
- **Features**: List of user-facing features
- **Dependencies**: Other components it depends on

### 4. Contract Generation

For each API boundary:
- **Contract Name**: Identifier (e.g., `user_service_api`)
- **Provider**: Component that provides API
- **Consumer**: Component that uses API
- **Endpoints**: List of functions/methods/routes

### 5. Migration Strategy

Choose one:
- **minimal**: Minimal moves, keep most structure, add orchestration layer
- **moderate**: Move code to components, preserve internal structure
- **full_restructure**: Complete reorganization into orchestration structure

### 6. Risk Assessment

Identify risks:
- Breaking changes to imports
- Test failures after reorganization
- Dependency conflicts
- User-facing command changes

## Output Format

Provide your plan as structured JSON:

```json
{{
  "migration_strategy": "moderate",
  "file_moves": [
    {{
      "source": "src/cli/main.py",
      "destination": "components/cli_interface/src/main.py",
      "reason": "CLI entry point to isolated component",
      "risk": "LOW",
      "git_command": "git mv src/cli/main.py components/cli_interface/src/main.py"
    }}
  ],
  "import_updates": [
    {{
      "file": "components/cli_interface/src/main.py",
      "old_import": "from src.core import engine",
      "new_import": "from components.core_engine.src import engine",
      "files_affected": 3
    }}
  ],
  "manifests_to_create": [
    {{
      "component": "cli_interface",
      "type": "cli_application",
      "entry_point": "src/main.py",
      "features": ["analyze command", "export command"],
      "dependencies": ["core_engine", "file_processor"]
    }}
  ],
  "contracts_to_create": [
    {{
      "name": "core_engine_api",
      "provider": "core_engine",
      "consumer": "cli_interface",
      "endpoints": [
        {{"method": "analyze_file", "params": ["file_path"], "returns": "AnalysisResult"}},
        {{"method": "get_results", "params": ["session_id"], "returns": "List[Result]"}}
      ]
    }}
  ],
  "shared_libraries": [
    {{
      "name": "shared_types",
      "source_files": ["src/types.py", "src/constants.py"],
      "destination": "shared-libs/shared_types/",
      "reason": "Used by multiple components"
    }}
  ],
  "risks": [
    "Import path changes will break existing tests",
    "CLI entry point changes may affect user scripts"
  ],
  "recommendations": [
    "Run full test suite after each phase",
    "Update documentation to reflect new structure"
  ],
  "estimated_duration_minutes": 120
}}
```

## Important Notes

- **Preserve git history**: Use `git mv`, not `rm + add`
- **Incremental commits**: One commit per phase
- **Component naming**: snake_case only (no hyphens, no uppercase)
- **Testing**: Plan for test updates after moves

## Start Your Planning

Provide the complete reorganization plan in the JSON format above.
"""

        return prompt

    def generate_spec_extraction_prompt(self, component_analysis: Dict) -> str:
        """
        Generate LLM prompt for specification extraction
        (create YAML spec from existing code/docs)
        """
        prompt = f"""# Task: Extract Specification from Existing Project

You are creating a formal specification document for an existing project that is being onboarded into the orchestration system.

## Project Information

**Location**: `{self.project_dir}`
**Components**: {len(component_analysis.get('components', []))}
**Languages**: {', '.join(self.analysis.languages_detected.keys())}

## Discovered Components

{self._format_components_for_prompt(component_analysis.get('components', []))}

## Your Task

Create a comprehensive YAML specification that documents the system as it currently exists. This spec will serve as the authoritative reference for the orchestration system.

### Specification Structure

```yaml
metadata:
  name: "Project Name"
  version: "0.1.0"  # Start at 0.1.0 for newly onboarded projects
  description: "Brief project description"
  repository: "git URL"
  language: "primary language"
  status: "existing_project_onboarded"

architecture:
  type: "multi-component"  # or "monolith" if single component
  components:
    - name: component_name
      type: cli_application | library | web_server | gui_application | generic
      responsibility: "What this component does"
      entry_point: "path/to/main.py"

features:
  - id: FEAT-001
    name: "Feature Name"
    description: "What the feature does"
    component: component_name
    type: cli_command | api_endpoint | library_function | gui_screen
    user_facing: true | false
    commands:  # For CLI features
      - "command --option"
    api_methods:  # For library features
      - "method_name(params)"
    endpoints:  # For web_server features
      - "GET /api/endpoint"

contracts:
  - name: component_a_to_b_api
    provider: component_a
    consumer: component_b
    interface:
      - method: method_name
        params:
          - name: param_name
            type: str
            required: true
        returns: ReturnType

testing_requirements:
  unit_test_coverage: 80
  integration_tests_required: true
  e2e_tests_required: true  # For CLI and web_server components
  test_framework: pytest | jest | go_test | cargo_test

quality_gates:
  phase_5:
    integration_test_pass_rate: 100
  phase_6:
    completion_checks_required: 16

dependencies:
  external:
    - name: package_name
      version: ">=1.0.0"
      purpose: "Why needed"
  internal:
    - from: component_a
      to: component_b
      type: api_call
```

### Extraction Guidelines

**From Code**:
- Component structure → architecture.components
- Function signatures → contracts.interface
- CLI argument parsers → features with type: cli_command
- HTTP routes → features with type: api_endpoint
- Test files → testing_requirements

**From Documentation**:
- README → metadata.description, features.description
- API docs → contracts, features
- User guides → features with user_facing: true

**From Package Files**:
- requirements.txt / package.json / go.mod → dependencies.external
- setup.py / pyproject.toml → metadata

## Output Format

Provide the complete specification as valid YAML:

```yaml
# (Complete YAML specification here)
```

## Important Notes

- **Be comprehensive**: List ALL features (for test coverage)
- **Be accurate**: Reflect what exists NOW, not what should be
- **Be specific**: Use actual command names, method signatures, endpoints
- **Feature IDs**: Sequential FEAT-001, FEAT-002, etc.

## Files to Review

Key documentation files:
{self._generate_doc_list_for_llm()}

## Start Your Extraction

Provide the complete YAML specification.
"""

        return prompt

    def save_analysis_report(self, output_path: Path):
        """Save automated analysis results to markdown"""
        if not self.analysis:
            raise ValueError("No analysis results to save")

        report = f"""# Existing Project Onboarding Analysis

**Generated**: {datetime.now().isoformat()}
**Project**: {self.project_dir}

## Summary

- **Languages**: {', '.join(f"{lang} ({conf*100:.0f}%)" for lang, conf in self.analysis.languages_detected.items())}
- **Source Files**: {self.analysis.source_file_count}
- **Lines of Code**: {self.analysis.total_lines_of_code:,}
- **Package Structure**: {self.analysis.package_structure}
- **Has Tests**: {self.analysis.has_tests} ({self.analysis.test_framework or 'N/A'})

## Discovered Components

{len(self.analysis.existing_components)} existing component directories found:

{self._format_list(self.analysis.existing_components)}

## Entry Points

{len(self.analysis.main_entry_points)} main entry points detected:

{self._format_list(self.analysis.main_entry_points)}

## Dependency Files

{self._format_list(self.analysis.dependencies_files)}

## Potential Conflicts

{self._format_list(self.analysis.potential_conflicts) if self.analysis.potential_conflicts else "None detected"}

## Next Steps

1. Review this analysis
2. Run LLM-guided component discovery:
   ```bash
   python orchestration/migration/onboarding_planner.py llm-analyze {self.project_dir}
   ```
3. Review component analysis output
4. Generate reorganization plan:
   ```bash
   python orchestration/migration/onboarding_planner.py llm-plan {self.project_dir}
   ```

---

*This is an automated analysis. LLM analysis required for component discovery and planning.*
"""

        output_path.write_text(report)
        print(f"✓ Analysis report saved: {output_path}")

    # Helper methods

    def _detect_languages(self) -> Dict[str, float]:
        """Detect programming languages (simple heuristic-based)"""
        extensions = {
            '.py': 'Python',
            '.js': 'JavaScript',
            '.ts': 'TypeScript',
            '.go': 'Go',
            '.rs': 'Rust',
            '.java': 'Java',
            '.cpp': 'C++',
            '.c': 'C',
            '.h': 'C/C++',
        }

        counts = {}
        for ext, lang in extensions.items():
            files = list(self.project_dir.rglob(f'*{ext}'))
            # Exclude common non-source directories
            files = [f for f in files if not any(
                excluded in f.parts
                for excluded in {'.git', 'node_modules', 'venv', '.venv', '__pycache__', 'dist', 'build'}
            )]
            if files:
                counts[lang] = len(files)

        if not counts:
            return {}

        total = sum(counts.values())
        return {lang: count / total for lang, count in counts.items()}

    def _find_source_files(self) -> List[Path]:
        """Find all source code files"""
        extensions = ['.py', '.js', '.ts', '.go', '.rs', '.java', '.cpp', '.c', '.h']
        source_files = []

        for ext in extensions:
            files = list(self.project_dir.rglob(f'*{ext}'))
            # Exclude non-source directories
            files = [f for f in files if not any(
                excluded in f.parts
                for excluded in {'.git', 'node_modules', 'venv', '.venv', '__pycache__', 'dist', 'build'}
            )]
            source_files.extend(files)

        return source_files

    def _count_lines_of_code(self, files: List[Path]) -> int:
        """Count total lines of code"""
        total = 0
        for file_path in files:
            try:
                total += len(file_path.read_text().splitlines())
            except (UnicodeDecodeError, PermissionError):
                pass
        return total

    def _find_entry_points(self) -> List[str]:
        """Find main entry points (main.py, __main__.py, index.js, etc.)"""
        entry_patterns = [
            'main.py', '__main__.py', 'cli.py',
            'index.js', 'index.ts', 'server.js',
            'main.go', 'main.rs'
        ]

        entry_points = []
        for pattern in entry_patterns:
            for file_path in self.project_dir.rglob(pattern):
                rel_path = file_path.relative_to(self.project_dir)
                entry_points.append(str(rel_path))

        return entry_points

    def _detect_existing_components(self) -> List[str]:
        """Detect directories that might be existing components"""
        # Look for common component directory names
        common_names = ['src', 'lib', 'components', 'services', 'api', 'cli', 'core', 'utils']

        existing = []
        for item in self.project_dir.iterdir():
            if item.is_dir() and item.name in common_names:
                existing.append(item.name)

        # Also check for components/ directory
        components_dir = self.project_dir / 'components'
        if components_dir.exists():
            for item in components_dir.iterdir():
                if item.is_dir():
                    existing.append(f"components/{item.name}")

        return existing

    def _find_dependency_files(self) -> List[str]:
        """Find dependency declaration files"""
        dep_files = [
            'requirements.txt', 'pyproject.toml', 'setup.py', 'Pipfile',
            'package.json', 'yarn.lock', 'package-lock.json',
            'go.mod', 'go.sum',
            'Cargo.toml', 'Cargo.lock',
            'pom.xml', 'build.gradle'
        ]

        found = []
        for dep_file in dep_files:
            file_path = self.project_dir / dep_file
            if file_path.exists():
                found.append(dep_file)

        return found

    def _detect_tests(self) -> Tuple[bool, Optional[str]]:
        """Detect if project has tests and what framework"""
        test_dirs = ['test', 'tests', 'spec', '__tests__']
        has_tests = any((self.project_dir / d).exists() for d in test_dirs)

        # Detect framework
        framework = None
        if (self.project_dir / 'pytest.ini').exists() or any(self.project_dir.rglob('test_*.py')):
            framework = 'pytest'
        elif (self.project_dir / 'jest.config.js').exists():
            framework = 'jest'
        elif any(self.project_dir.rglob('*_test.go')):
            framework = 'go test'
        elif any(self.project_dir.rglob('test_*.rs')):
            framework = 'cargo test'

        return has_tests, framework

    def _detect_package_structure(self) -> str:
        """Detect if workspace or package structure"""
        # Check for workspace structure (multiple top-level project dirs)
        top_level_projects = 0
        for item in self.project_dir.iterdir():
            if item.is_dir() and (item / 'src').exists():
                top_level_projects += 1

        if top_level_projects >= 2:
            return "workspace"

        # Check for package structure (single src/ directory)
        if (self.project_dir / 'src').exists():
            return "package"

        return "flat"

    def _check_conflicts(self) -> List[str]:
        """Check for directories that would conflict with orchestration"""
        conflicts = []
        conflict_dirs = ['components', 'contracts', 'shared-libs', 'specifications']

        for dir_name in conflict_dirs:
            dir_path = self.project_dir / dir_name
            if dir_path.exists():
                try:
                    contents = list(dir_path.iterdir())
                    if contents:
                        conflicts.append(f"{dir_name}/ exists with {len(contents)} items")
                except PermissionError:
                    conflicts.append(f"{dir_name}/ exists (permission denied)")

        return conflicts

    def _generate_file_list_for_llm(self) -> str:
        """Generate formatted file list for LLM prompt"""
        important_files = []

        # Entry points
        for entry in self.analysis.main_entry_points[:5]:
            important_files.append(f"- Entry point: {entry}")

        # Dependency files
        for dep_file in self.analysis.dependencies_files:
            important_files.append(f"- Dependencies: {dep_file}")

        # Documentation
        for doc in ['README.md', 'README.rst', 'docs/index.md']:
            if (self.project_dir / doc).exists():
                important_files.append(f"- Documentation: {doc}")

        return '\n'.join(important_files) if important_files else "- (explore project directory)"

    def _generate_doc_list_for_llm(self) -> str:
        """Generate list of documentation files for LLM"""
        docs = []
        doc_patterns = ['README.*', 'docs/**/*.md', 'API.md', 'ARCHITECTURE.md']

        for pattern in doc_patterns:
            for doc_file in self.project_dir.glob(pattern):
                rel_path = doc_file.relative_to(self.project_dir)
                docs.append(f"- {rel_path}")

        return '\n'.join(docs) if docs else "- (no documentation files found)"

    def _format_list(self, items: List[str]) -> str:
        """Format list as markdown bullet points"""
        if not items:
            return "- (none)"
        return '\n'.join(f"- {item}" for item in items)

    def _format_components_for_prompt(self, components: List[Dict]) -> str:
        """Format component list for LLM prompt"""
        if not components:
            return "(No components discovered yet)"

        formatted = []
        for comp in components:
            formatted.append(f"""
**{comp['name']}**
- Type: {comp['type']}
- Directory: {comp['directory']}
- Entry Point: {comp.get('entry_point', 'N/A')}
- Estimated LOC: {comp.get('estimated_loc', '?')}
- Features: {len(comp.get('features', []))}
""")

        return '\n'.join(formatted)


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description="Onboarding planner for existing projects"
    )
    parser.add_argument(
        'command',
        choices=['analyze', 'llm-analyze', 'llm-plan', 'llm-extract-spec'],
        help='Command to run'
    )
    parser.add_argument(
        'project_dir',
        help='Project directory to analyze'
    )
    parser.add_argument(
        '--output',
        default='analysis_report.md',
        help='Output file path (default: analysis_report.md)'
    )
    parser.add_argument(
        '--component-analysis',
        help='JSON file with component analysis (for llm-plan)'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()
    planner = OnboardingPlanner(project_dir)

    if args.command == 'analyze':
        # Run automated analysis
        planner.run_automated_analysis()
        output_path = Path(args.output)
        planner.save_analysis_report(output_path)

    elif args.command == 'llm-analyze':
        # Generate LLM prompt for component discovery
        planner.run_automated_analysis()
        prompt = planner.generate_llm_analysis_prompt()
        print("\n" + "=" * 70)
        print("LLM PROMPT FOR COMPONENT DISCOVERY")
        print("=" * 70)
        print(prompt)
        print("=" * 70)
        print("\nCopy the prompt above and provide it to Claude Code.")
        print("Save the JSON response and use it with 'llm-plan' command.")

    elif args.command == 'llm-plan':
        # Generate LLM prompt for reorganization planning
        if not args.component_analysis:
            print("ERROR: --component-analysis required for llm-plan")
            sys.exit(1)

        component_analysis = json.loads(Path(args.component_analysis).read_text())
        planner.run_automated_analysis()
        prompt = planner.generate_llm_planning_prompt(component_analysis)
        print("\n" + "=" * 70)
        print("LLM PROMPT FOR REORGANIZATION PLANNING")
        print("=" * 70)
        print(prompt)
        print("=" * 70)
        print("\nCopy the prompt above and provide it to Claude Code.")

    elif args.command == 'llm-extract-spec':
        # Generate LLM prompt for specification extraction
        if not args.component_analysis:
            print("ERROR: --component-analysis required for llm-extract-spec")
            sys.exit(1)

        component_analysis = json.loads(Path(args.component_analysis).read_text())
        planner.run_automated_analysis()
        prompt = planner.generate_spec_extraction_prompt(component_analysis)
        print("\n" + "=" * 70)
        print("LLM PROMPT FOR SPECIFICATION EXTRACTION")
        print("=" * 70)
        print(prompt)
        print("=" * 70)
        print("\nCopy the prompt above and provide it to Claude Code.")
        print("Save the YAML response to specifications/extracted_system_spec.yaml")


if __name__ == '__main__':
    main()
