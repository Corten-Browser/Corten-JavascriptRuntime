#!/usr/bin/env python3
"""
Automatically initialize task queue from specification.
Called by git hooks, not by model.
No model decision required.
"""
import json
import sys
import re
from pathlib import Path
from datetime import datetime


def discover_specs() -> list[Path]:
    """Find spec files in standard locations."""
    project_root = Path.cwd()

    patterns = [
        "specs/*.yaml",
        "specs/*.yml",
        "specifications/*.yaml",
        "specifications/*.yml",
        "specifications/*.md",
        "docs/*-specification.md",
        "docs/*-spec.md",
    ]

    discovered = []
    for pattern in patterns:
        matches = list(project_root.glob(pattern))
        discovered.extend(matches)

    return sorted(set(discovered))


def extract_features_from_yaml(spec_file: Path) -> list[dict]:
    """Extract features from YAML spec."""
    try:
        import yaml
        with open(spec_file) as f:
            spec = yaml.safe_load(f)

        if "features" not in spec:
            return []

        features = []
        for feature in spec["features"]:
            if feature.get("required", True):
                features.append({
                    "id": feature["id"],
                    "name": feature["name"],
                    "description": feature.get("description", feature["name"]),
                    "dependencies": feature.get("dependencies", [])
                })

        return features
    except ImportError:
        print("Warning: PyYAML not installed. Cannot parse YAML specs.")
        return []
    except Exception as e:
        print(f"Warning: Failed to parse YAML spec: {e}")
        return []


def extract_features_from_markdown(spec_file: Path) -> list[dict]:
    """Extract features from markdown spec."""
    content = spec_file.read_text()
    lines = content.split('\n')

    features = []
    feature_id = 0

    for i, line in enumerate(lines):
        # Pattern 1: ## Feature: X
        match = re.match(r'^#+\s*(?:Feature|Component|Module):\s*(.+)$', line, re.I)
        if match:
            feature_id += 1
            features.append({
                "id": f"FEAT-{feature_id:03d}",
                "name": match.group(1).strip(),
                "description": match.group(1).strip(),
                "dependencies": []
            })
            continue

        # Pattern 2: - [ ] Implement X
        match = re.match(r'^[\s-]*\[\s*\]\s*(?:Implement|Create|Build|Add)\s+(.+)$', line, re.I)
        if match:
            feature_id += 1
            features.append({
                "id": f"FEAT-{feature_id:03d}",
                "name": match.group(1).strip(),
                "description": match.group(1).strip(),
                "dependencies": []
            })
            continue

        # Pattern 3: MUST implement X
        match = re.match(r'^.*(?:MUST|SHALL|REQUIRED)[:\s]+(?:implement|support|provide)\s+(.+?)(?:\.|$)', line, re.I)
        if match:
            feature_id += 1
            features.append({
                "id": f"FEAT-{feature_id:03d}",
                "name": match.group(1).strip(),
                "description": match.group(1).strip(),
                "dependencies": []
            })

    return features


def create_tasks_from_features(features: list[dict]) -> list[dict]:
    """Convert features to tasks."""
    tasks = []

    for feature in features:
        task = {
            "id": f"TASK-{feature['id']}",
            "name": f"Implement {feature['name']}",
            "description": feature["description"],
            "feature_id": feature["id"],
            "dependencies": [f"TASK-{dep}" for dep in feature.get("dependencies", [])],
            "status": "pending",
            "started_at": None,
            "completed_at": None,
            "verification_result": None
        }
        tasks.append(task)

    return tasks


def initialize_queue_from_spec(spec_file: Path) -> bool:
    """Initialize task queue from specification file."""
    if not spec_file.exists():
        print(f"Error: Spec file not found: {spec_file}")
        return False

    # Determine spec type and extract features
    if spec_file.suffix in [".yaml", ".yml"]:
        features = extract_features_from_yaml(spec_file)
    else:
        features = extract_features_from_markdown(spec_file)

    if not features:
        print(f"Warning: No features found in {spec_file}")
        return False

    # Create tasks
    tasks = create_tasks_from_features(features)

    # Save to queue state
    queue_state = {
        "tasks": tasks,
        "completed_order": [],
        "last_updated": datetime.now().isoformat(),
        "initialized": True,
        "spec_file": str(spec_file),
        "total_features": len(features)
    }

    queue_file = Path("orchestration/task_queue/queue_state.json")
    queue_file.parent.mkdir(parents=True, exist_ok=True)
    queue_file.write_text(json.dumps(queue_state, indent=2))

    print(f"Initialized queue with {len(tasks)} tasks from {spec_file.name}")

    # Update manifest
    manifest_file = Path("orchestration/spec_manifest.json")
    if manifest_file.exists():
        manifest = json.loads(manifest_file.read_text())
    else:
        manifest = {}

    manifest["queue_initialized"] = True
    manifest["spec_file"] = str(spec_file)
    manifest["last_sync"] = datetime.now().isoformat()
    manifest["task_count"] = len(tasks)

    manifest_file.write_text(json.dumps(manifest, indent=2))

    return True


def auto_initialize():
    """Main auto-initialization function."""
    print("=" * 60)
    print("AUTO-INITIALIZING TASK QUEUE")
    print("=" * 60)
    print("")

    # Check if already initialized
    queue_file = Path("orchestration/task_queue/queue_state.json")
    if queue_file.exists():
        try:
            state = json.loads(queue_file.read_text())
            if state.get("initialized") and state.get("tasks"):
                print("Queue already initialized")
                print(f"  Tasks: {len(state['tasks'])}")
                completed = sum(1 for t in state["tasks"] if t.get("status") == "completed")
                print(f"  Completed: {completed}/{len(state['tasks'])}")
                return True
        except Exception:
            pass

    # Find spec file
    manifest_file = Path("orchestration/spec_manifest.json")
    spec_file = None

    if manifest_file.exists():
        manifest = json.loads(manifest_file.read_text())
        if manifest.get("spec_file"):
            spec_file = Path(manifest["spec_file"])

    if not spec_file or not spec_file.exists():
        # Auto-discover
        discovered = discover_specs()
        if discovered:
            spec_file = discovered[0]
            print(f"Auto-discovered spec: {spec_file}")
        else:
            print("No specification file found")
            print("Queue will remain empty until spec is provided")
            return False

    # Initialize from spec
    success = initialize_queue_from_spec(spec_file)

    if success:
        print("")
        print("Queue initialized successfully")
    else:
        print("")
        print("Failed to initialize queue")

    print("=" * 60)

    return success


if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Specific spec file provided
        spec_file = Path(sys.argv[1])
        success = initialize_queue_from_spec(spec_file)
    else:
        # Auto-discover and initialize
        success = auto_initialize()

    sys.exit(0 if success else 1)
