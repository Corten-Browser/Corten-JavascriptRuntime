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

# Add parent to path for standalone script execution
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Import canonical spec discovery (single source of truth)
from orchestration.cli.spec_discovery import discover_all_specs
from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def to_relative_path(path: Path) -> str:
    """
    Convert a path to relative (from current working directory) for portability.

    Paths stored in JSON files should be relative so they work after:
    - Git checkout to different machines
    - Projects moved to different directories

    Args:
        path: Absolute or relative Path object

    Returns:
        Relative path string, or original string if conversion fails
    """
    try:
        # Try to make relative to current working directory
        return str(path.relative_to(Path.cwd()))
    except ValueError:
        # If path is not under cwd, try to make it relative to its own parent
        # This handles edge cases but shouldn't normally happen
        return str(path)


def discover_specs() -> list[Path]:
    """
    Find spec files in standard locations.

    This function delegates to the canonical discover_all_specs() function
    in spec_discovery.py, which is the single source of truth for spec discovery.

    Returns:
        Sorted list of unique specification file paths.
    """
    return discover_all_specs()


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
    """Extract features from markdown spec (ENHANCED with bare checklists and deliverables)."""
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
                "dependencies": [],
                "extraction_method": "automated-header"
            })
            continue

        # Pattern 2a: Checklist with action verbs (EXPANDED)
        match_with_verb = re.match(
            r'^[\s-]*\[\s*\]\s*(?:Implement|Create|Build|Add|Load|Parse|Wrap|Extract|'
            r'Setup|Configure|Install|Enable|Support|Provide|Handle|Process|'
            r'Integrate|Connect|Attach|Deploy|Initialize|Develop|Design|Write)\s+(.+)$',
            line, re.I
        )
        if match_with_verb:
            feature_id += 1
            features.append({
                "id": f"FEAT-{feature_id:03d}",
                "name": match_with_verb.group(1).strip(),
                "description": match_with_verb.group(1).strip(),
                "dependencies": [],
                "extraction_method": "automated-checklist-verb"
            })
            continue

        # Pattern 2b: Bare checklist items (NEW - no verb required)
        match_bare = re.match(r'^[\s-]*\[\s*\]\s+(.+)$', line, re.I)
        if match_bare:
            feature_text = match_bare.group(1).strip()

            # Filter out meta-items and short entries
            skip_patterns = [
                r'^\*\*(Tests?|Testing|Validation|Performance|Note|Example|WPT)\*\*:',
                r'^Tests?:',
                r'^Validation:',
                r'^Note:',
                r'^Performance:',
                r'^Example:',
            ]

            # Only add if not a meta-item and has reasonable length
            if len(feature_text) > 10 and not any(re.match(p, feature_text, re.I) for p in skip_patterns):
                feature_id += 1
                features.append({
                    "id": f"FEAT-{feature_id:03d}",
                    "name": feature_text,
                    "description": feature_text,
                    "dependencies": [],
                    "extraction_method": "automated-checklist-bare"
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
                "dependencies": [],
                "extraction_method": "automated-must-shall"
            })

    # Extract deliverables and merge
    deliverable_features = extract_deliverables_from_markdown(spec_file)

    # Merge and deduplicate by feature name (case-insensitive)
    all_features = features + deliverable_features
    seen = set()
    unique_features = []

    for feature in all_features:
        name_lower = feature["name"].lower().strip()
        if name_lower not in seen and len(name_lower) > 10:
            seen.add(name_lower)
            unique_features.append(feature)

    # Renumber IDs sequentially
    for i, feature in enumerate(unique_features, 1):
        feature["id"] = f"FEAT-{i:03d}"

    return unique_features


def extract_deliverables_from_markdown(spec_file: Path) -> list[dict]:
    """
    Extract features from **Deliverables**: sections (NEW function).

    Pattern:
        **Deliverables**:
        - Feature 1
        - Feature 2
        - Feature 3
    """
    content = spec_file.read_text()
    lines = content.split('\n')

    features = []
    feature_id = 0
    in_deliverables = False
    current_phase = None

    for i, line in enumerate(lines):
        # Track current phase/milestone for context
        phase_match = re.match(r'^###?\s+(Phase|Milestone)\s+(\d+):\s*(.+)', line, re.I)
        if phase_match:
            current_phase = f"{phase_match.group(1)} {phase_match.group(2)}: {phase_match.group(3)}"

        # Detect **Deliverables**: heading (with optional 's')
        if re.match(r'^\s*\*\*Deliverables?\*\*:\s*$', line, re.I):
            in_deliverables = True
            continue

        # Stop at next major heading or section
        if in_deliverables:
            # Stop conditions
            if re.match(r'^#+\s', line):  # New markdown heading
                in_deliverables = False
            elif re.match(r'^\*\*[A-Z][a-z]+\*\*:', line):  # New **Section**:
                in_deliverables = False
            elif line.strip() == '' and i < len(lines) - 1 and re.match(r'^\*\*', lines[i+1]):
                # Blank line before new section
                in_deliverables = False

        # Extract bullet points while in deliverables section
        if in_deliverables:
            match = re.match(r'^[\s-]*[\-\*\+]\s+(.+)$', line)
            if match:
                feature_text = match.group(1).strip()

                # Filter out meta-items
                skip_patterns = [
                    r'^\*\*(Tests?|Testing|Validation|Performance|WPT)\*\*:',
                    r'^Tests?:',
                    r'^Validation:',
                    r'^Coverage:',
                    r'^Performance:',
                ]

                if not any(re.match(p, feature_text, re.I) for p in skip_patterns) and len(feature_text) > 10:
                    feature_id += 1
                    feature_data = {
                        "id": f"FEAT-{feature_id:03d}",
                        "name": feature_text,
                        "description": feature_text,
                        "dependencies": [],
                        "extraction_method": "automated-deliverables"
                    }

                    # Add phase context if available
                    if current_phase:
                        feature_data["phase"] = current_phase

                    features.append(feature_data)

    return features


def validate_yaml_spec(spec_file: Path) -> tuple[bool, str]:
    """
    Validate YAML specification has required structure.

    Returns:
        (True, "Valid (N features)") if valid
        (False, "Error message") if invalid
    """
    try:
        import yaml

        # Check 1: Parse YAML
        content = spec_file.read_text()
        if not content.strip():
            return False, "File is empty"

        spec = yaml.safe_load(content)

        # Check 2: Root structure
        if not isinstance(spec, dict):
            return False, "YAML must be a dictionary, not a list or scalar"

        # Check 3: Has 'features' key
        if "features" not in spec:
            return False, "Missing required 'features' key at root level"

        # Check 4: features is a list
        if not isinstance(spec["features"], list):
            return False, "'features' must be a list, not a dictionary"

        # Check 5: Not empty
        if len(spec["features"]) == 0:
            return False, "Features list is empty (no features defined)"

        # Check 6: Each feature has required fields
        errors = []
        for i, feature in enumerate(spec["features"]):
            if not isinstance(feature, dict):
                errors.append(f"Feature {i}: must be a dictionary")
                continue

            if "id" not in feature:
                errors.append(f"Feature {i}: missing required 'id' field")
            if "name" not in feature:
                errors.append(f"Feature {i}: missing required 'name' field")

        if errors:
            return False, "; ".join(errors)

        # Valid!
        feature_count = len(spec["features"])
        return True, f"Valid ({feature_count} feature{'s' if feature_count != 1 else ''})"

    except Exception as e:
        if "yaml" in str(type(e).__name__).lower():
            return False, f"Invalid YAML syntax: {str(e).split(':')[0]}"
        elif e.__class__.__name__ == "ModuleNotFoundError":
            return False, "PyYAML not installed (cannot validate YAML specs)"
        else:
            return False, f"Validation error: {type(e).__name__}: {e}"


def validate_markdown_spec(spec_file: Path) -> tuple[bool, str]:
    """
    Validate Markdown specification has extractable features.

    Returns:
        (True, "Valid (N features)") if valid
        (False, "Error message") if invalid
    """
    try:
        # Check 1: Not empty
        content = spec_file.read_text()
        if not content.strip():
            return False, "File is empty"

        # Check 2: Has recognizable feature patterns
        patterns = [
            r'##\s*Feature:',           # ## Feature: Auth
            r'##\s*Component:',         # ## Component: Auth
            r'##\s*Module:',            # ## Module: Auth
            r'\[\s*\]\s*(?:Implement|Create|Build|Add)',  # - [ ] Implement X
            r'(?:MUST|SHALL|REQUIRED)[:\s]+(?:implement|support|provide)',  # MUST implement X
        ]

        has_features = any(
            re.search(pattern, content, re.I)
            for pattern in patterns
        )

        if not has_features:
            return False, (
                "No recognizable feature patterns found. "
                "Expected: '## Feature:', '- [ ] Implement', or 'MUST implement'"
            )

        # Check 3: Try to extract features
        features = extract_features_from_markdown(spec_file)
        if not features:
            return False, (
                "Feature patterns found but extraction failed. "
                "Check that features have proper formatting."
            )

        # Valid!
        return True, f"Valid ({len(features)} feature{'s' if len(features) != 1 else ''})"

    except Exception as e:
        return False, f"Validation error: {type(e).__name__}: {e}"


def validate_spec(spec_file: Path) -> tuple[bool, str]:
    """
    Validate specification file (YAML or Markdown).

    Returns:
        (True, "Valid (N features)") if valid
        (False, "Error message") if invalid
    """
    if not spec_file.exists():
        return False, f"File not found: {spec_file}"

    if not spec_file.is_file():
        return False, f"Not a file: {spec_file}"

    # Route to appropriate validator
    if spec_file.suffix.lower() in [".yaml", ".yml"]:
        return validate_yaml_spec(spec_file)
    elif spec_file.suffix.lower() == ".md":
        return validate_markdown_spec(spec_file)
    else:
        return False, f"Unsupported file type: {spec_file.suffix} (expected .yaml, .yml, or .md)"


def save_extraction_metadata(spec_files: list[Path], features_by_file: dict[str, int],
                             total_features: int, automated_success: bool = True) -> None:
    """
    Save metadata about what was extracted for /orch-extract-features to use.

    Args:
        spec_files: List of spec files processed
        features_by_file: Dict mapping file path to feature count
        total_features: Total features extracted
        automated_success: Whether automated extraction found any features
    """
    metadata = {
        "last_extraction": datetime.now().isoformat(),
        "extraction_method": "automated" if automated_success else "pending-llm",
        "spec_files": [
            {
                "path": to_relative_path(f),
                "features_extracted": features_by_file.get(str(f), 0),
                "extraction_method": "automated" if features_by_file.get(str(f), 0) > 0 else "none",
                "last_processed": datetime.now().isoformat()
            }
            for f in spec_files
        ],
        "total_features": total_features,
        "automated_features": total_features,
        "llm_features": 0,
        # LLM extraction must run at least once to validate/supplement automated extraction
        "llm_extraction_complete": False,
        "llm_extraction_timestamp": None,
        # Indicates whether automated regex patterns found any features
        "automated_extraction_success": automated_success
    }

    metadata_file = _paths.extraction_metadata
    metadata_file.parent.mkdir(parents=True, exist_ok=True)
    metadata_file.write_text(json.dumps(metadata, indent=2))


def auto_commit_extraction(extraction_type: str, features_added: int, features_total: int) -> bool:
    """
    Auto-commit extraction results to git if repository exists (NEW function).

    Args:
        extraction_type: "automated", "llm", or "hybrid"
        features_added: Number of features added in this extraction
        features_total: Total features in queue

    Returns:
        True if committed successfully, False otherwise
    """
    import subprocess

    # Check if this is a git repository
    if not Path(".git").exists():
        return False

    try:
        # Stage extraction files (only files that exist)
        candidate_files = [
            _paths.queue_state,
            _paths.extraction_metadata,
            _paths.spec_manifest,
        ]
        files_to_stage = [str(f) for f in candidate_files if f.exists()]

        if not files_to_stage:
            return False

        subprocess.run(
            ["git", "add"] + files_to_stage,
            check=True,
            capture_output=True
        )

        # Create commit message
        commit_msg = f"feat(queue): {extraction_type} extraction - {features_added} features added ({features_total} total)"

        # Commit
        result = subprocess.run(
            ["git", "commit", "-m", commit_msg],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            print("")
            print("✅ Changes committed to git:")
            print(f"   {commit_msg}")
            return True
        else:
            # Nothing to commit (no changes)
            return False

    except subprocess.CalledProcessError as e:
        error_msg = f"⚠️  Git commit failed: {e}"
        if e.stderr:
            stderr = e.stderr.decode() if isinstance(e.stderr, bytes) else e.stderr
            error_msg += f"\n   stderr: {stderr.strip()}"
        if e.stdout:
            stdout = e.stdout.decode() if isinstance(e.stdout, bytes) else e.stdout
            error_msg += f"\n   stdout: {stdout.strip()}"
        print(error_msg)
        return False


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
            "verification_result": None,
            "extraction_method": feature.get("extraction_method", "automated")
        }
        tasks.append(task)

    return tasks


def extract_all_features(spec_files: list[Path]) -> tuple[list[dict], list[str]]:
    """
    Extract features from multiple specification files.

    Returns:
        (features, errors) - List of features and list of error messages
    """
    all_features = []
    errors = []
    seen_ids = set()

    for spec_file in spec_files:
        # Validate spec
        valid, message = validate_spec(spec_file)
        if not valid:
            errors.append(f"{spec_file.name}: {message}")
            continue

        # Extract features based on type
        if spec_file.suffix.lower() in [".yaml", ".yml"]:
            features = extract_features_from_yaml(spec_file)
        else:
            features = extract_features_from_markdown(spec_file)

        if not features:
            errors.append(f"{spec_file.name}: No features extracted")
            continue

        # Check for duplicate IDs
        for feature in features:
            feature_id = feature["id"]
            if feature_id in seen_ids:
                errors.append(
                    f"{spec_file.name}: Duplicate feature ID '{feature_id}' "
                    f"(already defined in another spec)"
                )
            else:
                seen_ids.add(feature_id)
                all_features.append(feature)

    return all_features, errors


def initialize_queue_from_multiple_specs(spec_files: list[Path]) -> bool:
    """Initialize task queue from multiple specification files (ENHANCED with metadata)."""
    if not spec_files:
        print("Error: No specification files provided")
        return False

    print(f"Processing {len(spec_files)} specification file(s)...")
    print("")

    # NEW: Track features per file for metadata
    features_by_file = {}
    for spec_file in spec_files:
        if spec_file.suffix.lower() in [".yaml", ".yml"]:
            file_features = extract_features_from_yaml(spec_file)
        else:
            file_features = extract_features_from_markdown(spec_file)
        features_by_file[str(spec_file)] = len(file_features)

    # Extract features from all specs
    all_features, errors = extract_all_features(spec_files)

    # Report validation results
    for spec_file in spec_files:
        valid, message = validate_spec(spec_file)
        if valid:
            print(f"✅ {spec_file.name}: {message}")
        else:
            print(f"❌ {spec_file.name}: {message}")

    if errors:
        print("")
        print("Errors encountered:")
        for error in errors:
            print(f"  - {error}")
        print("")

    # Track whether automated extraction found features
    automated_success = bool(all_features)

    if not all_features:
        print("")
        print("=" * 60)
        print("⚠️  NO FEATURES EXTRACTED BY AUTOMATED PATTERNS")
        print("=" * 60)
        print("")
        print("The automated regex patterns didn't match your spec format.")
        print("This is OK - LLM extraction can find features that regex missed.")
        print("")
        print("Creating empty queue for LLM extraction...")
        all_features = []  # Ensure it's an empty list
    else:
        print("")
        print(f"Extracted {len(all_features)} total features from {len(spec_files)} file(s)")

    # Create tasks (may be empty if no features found)
    tasks = create_tasks_from_features(all_features)

    # Save to queue state
    queue_state = {
        "tasks": tasks,
        "completed_order": [],
        "last_updated": datetime.now().isoformat(),
        "initialized": True,
        "spec_file": to_relative_path(spec_files[0]),  # Backwards compatibility
        "spec_files": [to_relative_path(f) for f in spec_files],  # Multiple specs
        "total_features": len(all_features)
    }

    queue_file = _paths.queue_state
    queue_file.parent.mkdir(parents=True, exist_ok=True)
    queue_file.write_text(json.dumps(queue_state, indent=2))

    print(f"Initialized queue with {len(tasks)} tasks")

    # Update manifest
    manifest_file = _paths.spec_manifest
    manifest_file.parent.mkdir(parents=True, exist_ok=True)
    if manifest_file.exists():
        manifest = json.loads(manifest_file.read_text())
    else:
        manifest = {}

    manifest["queue_initialized"] = True
    manifest["spec_file"] = to_relative_path(spec_files[0])  # Backwards compatibility
    manifest["spec_files"] = [to_relative_path(f) for f in spec_files]  # Multiple specs
    manifest["last_sync"] = datetime.now().isoformat()
    manifest["task_count"] = len(tasks)

    manifest_file.write_text(json.dumps(manifest, indent=2))

    # Save extraction metadata (pass automated_success flag)
    save_extraction_metadata(spec_files, features_by_file, len(all_features), automated_success)

    # Print reminder that LLM extraction is required/recommended
    print("")
    print("=" * 60)
    if automated_success:
        print("💡 LLM EXTRACTION RECOMMENDED")
        print("=" * 60)
        print("")
        print(f"Automated extraction found {len(all_features)} features.")
        print("LLM extraction can find additional features that regex patterns missed.")
    else:
        print("🔴 LLM EXTRACTION REQUIRED")
        print("=" * 60)
        print("")
        print("No features were found by automated extraction.")
        print("LLM extraction is REQUIRED to populate the task queue.")
    print("")
    print("Run: /orch-extract-features")
    print("=" * 60)

    # Auto-commit if in git repo
    auto_commit_extraction("automated", len(all_features), len(all_features))

    return True


def initialize_queue_from_spec(spec_file: Path) -> bool:
    """Initialize task queue from specification file."""
    if not spec_file.exists():
        print(f"Error: Spec file not found: {spec_file}")
        return False

    # Validate spec before processing
    print(f"Validating {spec_file.name}...")
    valid, message = validate_spec(spec_file)

    if not valid:
        print(f"❌ Validation failed: {message}")
        print(f"   File: {spec_file}")
        print("")
        print("Requirements:")
        print("  YAML specs:")
        print("    - Must have 'features' key at root level")
        print("    - Each feature must have 'id' and 'name' fields")
        print("  Markdown specs:")
        print("    - Must have '## Feature:', '- [ ] Implement', or 'MUST implement' patterns")
        return False

    print(f"✅ {message}")

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
        "spec_file": to_relative_path(spec_file),
        "total_features": len(features)
    }

    queue_file = _paths.queue_state
    queue_file.parent.mkdir(parents=True, exist_ok=True)
    queue_file.write_text(json.dumps(queue_state, indent=2))

    print(f"Initialized queue with {len(tasks)} tasks from {spec_file.name}")

    # Update manifest
    manifest_file = _paths.spec_manifest
    manifest_file.parent.mkdir(parents=True, exist_ok=True)
    if manifest_file.exists():
        manifest = json.loads(manifest_file.read_text())
    else:
        manifest = {}

    manifest["queue_initialized"] = True
    manifest["spec_file"] = to_relative_path(spec_file)
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
    queue_file = _paths.queue_state
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

    # Find spec files
    manifest_file = _paths.spec_manifest
    spec_files = []

    if manifest_file.exists():
        manifest = json.loads(manifest_file.read_text())
        # Check for spec_files (plural) first, then fallback to spec_file (backwards compat)
        if manifest.get("spec_files"):
            spec_files = [Path(f) for f in manifest["spec_files"] if Path(f).exists()]
        elif manifest.get("spec_file"):
            spec_file = Path(manifest["spec_file"])
            if spec_file.exists():
                spec_files = [spec_file]

    if not spec_files:
        # Auto-discover ALL specs (not just first one)
        discovered = discover_specs()
        if discovered:
            spec_files = discovered
            print(f"Auto-discovered {len(spec_files)} specification file(s):")
            for spec in spec_files:
                print(f"  - {spec}")
        else:
            print("No specification files found")
            print("Queue will remain empty until spec is provided")
            return False

    # Initialize from all specs
    success = initialize_queue_from_multiple_specs(spec_files)

    if success:
        print("")
        print("Queue initialized successfully")
    else:
        print("")
        print("Failed to initialize queue")

    print("=" * 60)

    return success


def force_reinit() -> bool:
    """
    Force re-initialization of task queue.
    Clears existing queue and re-initializes from discovered specs.
    """
    print("=" * 60)
    print("FORCE RE-INITIALIZATION")
    print("=" * 60)
    print("")
    print("⚠️  WARNING: This will clear the existing task queue")
    print("   All task statuses will be reset to 'pending'")
    print("")

    # Check if queue exists
    queue_file = _paths.queue_state
    if queue_file.exists():
        try:
            state = json.loads(queue_file.read_text())
            task_count = len(state.get("tasks", []))
            completed = sum(1 for t in state.get("tasks", []) if t.get("status") == "completed")
            print(f"Current queue: {task_count} tasks ({completed} completed)")
            print("")
        except Exception:
            pass

    # Prompt for confirmation
    response = input("Continue with re-initialization? [y/N]: ")
    if response.lower() not in ["y", "yes"]:
        print("Cancelled")
        return False

    print("")
    print("Clearing existing queue...")

    # Delete existing queue
    if queue_file.exists():
        queue_file.unlink()

    # Clear manifest
    manifest_file = _paths.spec_manifest
    if manifest_file.exists():
        manifest_file.unlink()

    print("Queue cleared")
    print("")

    # Re-initialize from specs
    return auto_initialize()


def update_queue_from_specs() -> bool:
    """
    Update task queue from specs while preserving existing task statuses (ENHANCED with metadata).
    - Keeps status (pending/in_progress/completed) for existing features
    - Keeps timestamps (started_at, completed_at) for existing features
    - Updates task details (name, description) if spec changed
    - Adds new features as pending tasks
    """
    print("=" * 60)
    print("UPDATE QUEUE FROM SPECIFICATIONS")
    print("=" * 60)
    print("")

    # Check if queue exists
    queue_file = _paths.queue_state
    if not queue_file.exists():
        print("No existing queue found - running initial initialization instead")
        print("")
        return auto_initialize()

    # Load existing queue
    try:
        existing_state = json.loads(queue_file.read_text())
        existing_tasks = {t["feature_id"]: t for t in existing_state.get("tasks", [])}
        print(f"Existing queue: {len(existing_tasks)} tasks")
        completed = sum(1 for t in existing_tasks.values() if t.get("status") == "completed")
        print(f"  Completed: {completed}")
        print("")
    except Exception as e:
        print(f"Error reading existing queue: {e}")
        return False

    # Discover specs
    manifest_file = _paths.spec_manifest
    spec_files = []

    if manifest_file.exists():
        manifest = json.loads(manifest_file.read_text())
        if manifest.get("spec_files"):
            spec_files = [Path(f) for f in manifest["spec_files"] if Path(f).exists()]
        elif manifest.get("spec_file"):
            spec_file = Path(manifest["spec_file"])
            if spec_file.exists():
                spec_files = [spec_file]

    if not spec_files:
        discovered = discover_specs()
        if not discovered:
            print("No specification files found")
            return False
        spec_files = discovered

    print(f"Processing {len(spec_files)} specification file(s)...")
    print("")

    # NEW: Track features per file for metadata
    features_by_file = {}
    for spec_file in spec_files:
        if spec_file.suffix.lower() in [".yaml", ".yml"]:
            file_features = extract_features_from_yaml(spec_file)
        else:
            file_features = extract_features_from_markdown(spec_file)
        features_by_file[str(spec_file)] = len(file_features)

    # Extract features from all specs
    all_features, errors = extract_all_features(spec_files)

    # Report validation results
    for spec_file in spec_files:
        valid, message = validate_spec(spec_file)
        if valid:
            print(f"✅ {spec_file.name}: {message}")
        else:
            print(f"❌ {spec_file.name}: {message}")

    if errors:
        print("")
        print("Errors encountered:")
        for error in errors:
            print(f"  - {error}")
        print("")

    # Track whether automated extraction found features
    automated_success = bool(all_features)

    if not all_features:
        print("")
        print("⚠️  No features extracted from specs by automated patterns")
        print("   Existing queue will be preserved. Run /orch-extract-features for LLM extraction.")
        all_features = []  # Ensure it's an empty list
    else:
        print("")
        print(f"Extracted {len(all_features)} features from specs")
    print("")

    # Merge with existing tasks
    updated_tasks = []
    new_features = []
    updated_features = []

    for feature in all_features:
        feature_id = feature["id"]
        task_id = f"TASK-{feature_id}"

        if feature_id in existing_tasks:
            # Feature exists - preserve status and timestamps, update details
            existing_task = existing_tasks[feature_id]
            updated_task = {
                "id": task_id,
                "name": f"Implement {feature['name']}",  # Updated from spec
                "description": feature["description"],  # Updated from spec
                "feature_id": feature_id,
                "dependencies": [f"TASK-{dep}" for dep in feature.get("dependencies", [])],
                "status": existing_task.get("status", "pending"),  # PRESERVED
                "started_at": existing_task.get("started_at"),  # PRESERVED
                "completed_at": existing_task.get("completed_at"),  # PRESERVED
                "verification_result": existing_task.get("verification_result")  # PRESERVED
            }
            updated_tasks.append(updated_task)

            # Check if details changed
            if (existing_task.get("name") != updated_task["name"] or
                existing_task.get("description") != updated_task["description"]):
                updated_features.append(feature_id)
        else:
            # New feature - add as pending
            new_task = {
                "id": task_id,
                "name": f"Implement {feature['name']}",
                "description": feature["description"],
                "feature_id": feature_id,
                "dependencies": [f"TASK-{dep}" for dep in feature.get("dependencies", [])],
                "status": "pending",
                "started_at": None,
                "completed_at": None,
                "verification_result": None
            }
            updated_tasks.append(new_task)
            new_features.append(feature_id)

    # Report changes
    print("Update summary:")
    print(f"  Total tasks: {len(updated_tasks)}")
    print(f"  New features: {len(new_features)}")
    if new_features:
        for fid in new_features:
            print(f"    + {fid}")
    print(f"  Updated features: {len(updated_features)}")
    if updated_features:
        for fid in updated_features:
            print(f"    * {fid} (details changed)")
    preserved = len(updated_tasks) - len(new_features)
    print(f"  Preserved tasks: {preserved}")
    print("")

    # Save updated queue
    queue_state = {
        "tasks": updated_tasks,
        "completed_order": existing_state.get("completed_order", []),
        "last_updated": datetime.now().isoformat(),
        "initialized": True,
        "spec_file": to_relative_path(spec_files[0]),  # Backwards compatibility
        "spec_files": [to_relative_path(f) for f in spec_files],
        "total_features": len(all_features)
    }

    queue_file.write_text(json.dumps(queue_state, indent=2))

    # Update manifest
    if manifest_file.exists():
        manifest = json.loads(manifest_file.read_text())
    else:
        manifest = {}

    manifest["queue_initialized"] = True
    manifest["spec_file"] = to_relative_path(spec_files[0])
    manifest["spec_files"] = [to_relative_path(f) for f in spec_files]
    manifest["last_sync"] = datetime.now().isoformat()
    manifest["task_count"] = len(updated_tasks)

    manifest_file.write_text(json.dumps(manifest, indent=2))

    # Save extraction metadata (pass automated_success flag)
    save_extraction_metadata(spec_files, features_by_file, len(all_features), automated_success)

    # Print reminder that LLM extraction is required/recommended
    print("")
    print("=" * 60)
    if automated_success:
        print("💡 LLM EXTRACTION RECOMMENDED")
        print("=" * 60)
        print("")
        print(f"Automated extraction found {len(all_features)} features.")
        print("LLM extraction can find additional features that regex patterns missed.")
    else:
        print("🔴 LLM EXTRACTION REQUIRED")
        print("=" * 60)
        print("")
        print("No NEW features were found by automated extraction.")
        print("LLM extraction is REQUIRED to find features in your specs.")
    print("")
    print("Run: /orch-extract-features")
    print("=" * 60)

    # Auto-commit if in git repo
    auto_commit_extraction("automated", len(new_features), len(updated_tasks))

    print("Queue updated successfully")
    print("=" * 60)

    return True


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Auto-initialize task queue from specification documents"
    )
    parser.add_argument(
        "spec_file",
        nargs="?",
        help="Specific specification file to use (optional)"
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List discovered specification files and exit"
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check if specifications exist (exit 0=yes, 1=no)"
    )
    parser.add_argument(
        "--init",
        action="store_true",
        help="Force re-initialization (clears existing queue and re-initializes from specs)"
    )
    parser.add_argument(
        "--update",
        action="store_true",
        help="Update queue from specs (preserves existing task statuses, adds only new features)"
    )

    args = parser.parse_args()

    # --list: Show discovered specs
    if args.list:
        specs = discover_specs()
        if specs:
            print("Found specification files:")
            for spec in specs:
                print(f"  - {spec}")
            sys.exit(0)
        else:
            print("No specification files found")
            sys.exit(1)

    # --check: Verify specs exist (silent, exit code only)
    if args.check:
        specs = discover_specs()
        sys.exit(0 if specs else 1)

    # --init: Force re-initialization
    if args.init:
        success = force_reinit()
        sys.exit(0 if success else 1)

    # --update: Update queue while preserving statuses
    if args.update:
        success = update_queue_from_specs()
        sys.exit(0 if success else 1)

    # Normal initialization
    if args.spec_file:
        # Specific spec file provided
        spec_file = Path(args.spec_file)
        success = initialize_queue_from_spec(spec_file)
    else:
        # Auto-discover and initialize
        success = auto_initialize()

    sys.exit(0 if success else 1)
