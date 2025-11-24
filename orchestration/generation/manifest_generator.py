#!/usr/bin/env python3
"""
Component Manifest Generator

Generates component.yaml manifest files (v2.0 schema) for components discovered
during onboarding. Manifests declare component type, features, entry points, and
dependencies - critical for feature coverage testing (Check #13, v0.13.0).

Usage:
    python orchestration/generation/manifest_generator.py generate <component_analysis.json>
    python orchestration/generation/manifest_generator.py validate <component_dir>

Schema: orchestration/manifest_validator.py (v2.0)
Docs: docs/COMPONENT-MANIFEST-SCHEMA.md
"""

import sys
import yaml
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass, asdict


@dataclass
class ComponentManifest:
    """Component manifest schema v2.0"""
    version: str = "2.0"
    name: str = ""
    type: str = ""  # cli_application, library, web_server, gui_application, generic
    description: str = ""
    entry_point: Optional[str] = None
    user_facing_features: List[Dict] = None
    dependencies: Dict = None
    metadata: Dict = None

    def __post_init__(self):
        if self.user_facing_features is None:
            self.user_facing_features = []
        if self.dependencies is None:
            self.dependencies = {"components": [], "external": []}
        if self.metadata is None:
            self.metadata = {"created_by": "onboarding", "auto_generated": True}


class ManifestGenerator:
    """Generates component.yaml manifests from component analysis"""

    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()

    def generate_from_analysis(self, component_data: Dict) -> ComponentManifest:
        """
        Generate manifest from component analysis data

        Expected format (from onboarding_planner.py):
        {
            "name": "component_name",
            "type": "cli_application",
            "directory": "path/to/component",
            "entry_point": "main.py",
            "responsibility": "Brief description",
            "features": ["feature 1", "feature 2"],
            "dependencies": ["other_component"],
            "estimated_loc": 500,
            "apis": [...]
        }
        """
        manifest = ComponentManifest(
            name=component_data.get("name", ""),
            type=component_data.get("type", "generic"),
            description=component_data.get("responsibility", ""),
            entry_point=component_data.get("entry_point")
        )

        # Convert features to user_facing_features format
        features = component_data.get("features", [])
        manifest.user_facing_features = self._convert_features(
            features,
            component_data.get("type"),
            component_data.get("apis", [])
        )

        # Set dependencies
        component_deps = component_data.get("dependencies", [])
        manifest.dependencies = {
            "components": component_deps,
            "external": []  # TODO: Extract from requirements.txt / package.json
        }

        # Metadata
        manifest.metadata = {
            "created_by": "onboarding",
            "auto_generated": True,
            "estimated_loc": component_data.get("estimated_loc", 0),
            "source_directory": component_data.get("directory", "")
        }

        return manifest

    def _convert_features(self, features: List[str], component_type: str, apis: List[Dict]) -> List[Dict]:
        """
        Convert simple feature list to user_facing_features schema

        Different formats based on component type:
        - cli_application: commands
        - library: api_methods
        - web_server: endpoints
        - gui_application: screens
        """
        converted = []

        if component_type == "cli_application":
            # CLI commands
            for feature in features:
                converted.append({
                    "name": feature,
                    "type": "cli_command",
                    "commands": [feature],  # Actual command string
                    "description": f"CLI command: {feature}"
                })

        elif component_type == "library":
            # Library API methods
            for api in apis:
                if api.get("type") == "function" or api.get("type") == "class":
                    converted.append({
                        "name": api.get("name", ""),
                        "type": "api_method",
                        "api_methods": [api.get("name", "")],
                        "description": api.get("description", "")
                    })

        elif component_type == "web_server":
            # HTTP endpoints
            for api in apis:
                if api.get("type") == "endpoint":
                    converted.append({
                        "name": api.get("name", ""),
                        "type": "api_endpoint",
                        "endpoints": [f"{api.get('method', 'GET')} {api.get('path', '/')}"],
                        "description": api.get("description", "")
                    })

        elif component_type == "gui_application":
            # GUI screens/dialogs
            for feature in features:
                converted.append({
                    "name": feature,
                    "type": "gui_screen",
                    "screens": [feature],
                    "description": f"GUI screen: {feature}"
                })

        else:
            # Generic - just list features
            for feature in features:
                converted.append({
                    "name": feature,
                    "type": "feature",
                    "description": feature
                })

        return converted

    def save_manifest(self, manifest: ComponentManifest, output_path: Path):
        """Save manifest as YAML"""
        # Convert to dict
        data = asdict(manifest)

        # Write YAML
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with output_path.open('w') as f:
            yaml.dump(data, f, default_flow_style=False, sort_keys=False, indent=2)

        print(f"✓ Manifest saved: {output_path}")

    def generate_all(self, component_analysis: Dict, output_dir: Optional[Path] = None) -> List[Path]:
        """
        Generate manifests for all components in analysis

        Args:
            component_analysis: Full analysis from onboarding_planner.py
            output_dir: Where to save manifests (default: components/<name>/)

        Returns:
            List of generated manifest paths
        """
        components = component_analysis.get("components", [])
        print(f"Generating manifests for {len(components)} components...")

        generated = []

        for comp_data in components:
            manifest = self.generate_from_analysis(comp_data)

            # Determine output path
            if output_dir:
                manifest_path = output_dir / f"{manifest.name}_component.yaml"
            else:
                # Save in component directory
                comp_dir = self.project_dir / "components" / manifest.name
                manifest_path = comp_dir / "component.yaml"

            self.save_manifest(manifest, manifest_path)
            generated.append(manifest_path)

        print(f"✓ Generated {len(generated)} manifests")
        return generated

    def validate_manifest(self, manifest_path: Path) -> Tuple[bool, List[str]]:
        """
        Validate manifest against schema
        Returns (is_valid, errors)
        """
        if not manifest_path.exists():
            return False, [f"Manifest not found: {manifest_path}"]

        try:
            with manifest_path.open() as f:
                data = yaml.safe_load(f)

            errors = []

            # Check required fields
            required = ['version', 'name', 'type', 'description']
            for field in required:
                if field not in data:
                    errors.append(f"Missing required field: {field}")

            # Check version
            if data.get('version') != '2.0':
                errors.append(f"Invalid version: {data.get('version')} (expected 2.0)")

            # Check type
            valid_types = ['cli_application', 'library', 'web_server', 'gui_application', 'generic']
            if data.get('type') not in valid_types:
                errors.append(f"Invalid type: {data.get('type')} (must be one of {valid_types})")

            # Check user_facing_features
            if 'user_facing_features' in data:
                features = data['user_facing_features']
                if not isinstance(features, list):
                    errors.append("user_facing_features must be a list")
                else:
                    for idx, feature in enumerate(features):
                        if 'name' not in feature:
                            errors.append(f"Feature {idx} missing 'name' field")
                        if 'type' not in feature:
                            errors.append(f"Feature {idx} missing 'type' field")

            # Check dependencies
            if 'dependencies' in data:
                deps = data['dependencies']
                if not isinstance(deps, dict):
                    errors.append("dependencies must be a dict")
                elif 'components' not in deps:
                    errors.append("dependencies missing 'components' field")

            is_valid = len(errors) == 0
            return is_valid, errors

        except yaml.YAMLError as e:
            return False, [f"YAML parsing error: {e}"]
        except Exception as e:
            return False, [f"Validation error: {e}"]


def main():
    """Main entry point"""
    import argparse
    import json

    parser = argparse.ArgumentParser(
        description="Generate component.yaml manifests"
    )
    parser.add_argument(
        'command',
        choices=['generate', 'generate-all', 'validate'],
        help='Command to run'
    )
    parser.add_argument(
        'input',
        help='Component analysis JSON (for generate) or manifest path (for validate)'
    )
    parser.add_argument(
        '--project-dir',
        default='.',
        help='Project directory (default: current directory)'
    )
    parser.add_argument(
        '--output',
        help='Output directory for manifests'
    )
    parser.add_argument(
        '--name',
        help='Component name (for single generate)'
    )

    args = parser.parse_args()

    project_dir = Path(args.project_dir).resolve()
    generator = ManifestGenerator(project_dir)

    if args.command == 'generate':
        # Generate single manifest
        analysis_data = json.loads(Path(args.input).read_text())

        # If analysis has multiple components and --name specified, filter
        if "components" in analysis_data and args.name:
            components = analysis_data["components"]
            component_data = next((c for c in components if c["name"] == args.name), None)
            if not component_data:
                print(f"ERROR: Component '{args.name}' not found in analysis")
                sys.exit(1)
        else:
            component_data = analysis_data

        manifest = generator.generate_from_analysis(component_data)

        output_path = Path(args.output) if args.output else project_dir / "components" / manifest.name / "component.yaml"
        generator.save_manifest(manifest, output_path)

    elif args.command == 'generate-all':
        # Generate manifests for all components
        analysis_data = json.loads(Path(args.input).read_text())

        output_dir = Path(args.output) if args.output else None
        generated = generator.generate_all(analysis_data, output_dir)

        print(f"\n✓ Generated {len(generated)} manifests:")
        for manifest_path in generated:
            print(f"  {manifest_path}")

    elif args.command == 'validate':
        # Validate manifest
        manifest_path = Path(args.input).resolve()

        is_valid, errors = generator.validate_manifest(manifest_path)

        if is_valid:
            print(f"✅ Manifest is valid: {manifest_path}")
            sys.exit(0)
        else:
            print(f"❌ Manifest validation FAILED: {manifest_path}")
            print(f"\nErrors:")
            for error in errors:
                print(f"  • {error}")
            sys.exit(1)


if __name__ == '__main__':
    from typing import Tuple
    main()
