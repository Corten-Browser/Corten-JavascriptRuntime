"""Tests for state discovery."""

import pytest
from pathlib import Path
from orchestration.state_discovery import StateDiscovery


@pytest.fixture
def temp_project(tmp_path):
    """Create temporary project with mock structure."""
    # Create components
    (tmp_path / "components" / "comp1" / "src").mkdir(parents=True)
    (tmp_path / "components" / "comp1" / "CLAUDE.md").write_text("# Component 1")
    (tmp_path / "components" / "comp2" / "src").mkdir(parents=True)
    (tmp_path / "components" / "comp2" / "CLAUDE.md").write_text("# Component 2")

    return tmp_path


@pytest.fixture
def discovery(temp_project):
    """Create state discovery instance."""
    return StateDiscovery(temp_project)


def test_discover_components(discovery):
    """Test component discovery."""
    components = discovery._discover_components()
    assert len(components) == 2
    assert "comp1" in components
    assert "comp2" in components


def test_discover_components_empty(tmp_path):
    """Test discovery when no components exist."""
    discovery = StateDiscovery(tmp_path)
    components = discovery._discover_components()
    assert len(components) == 0


def test_discover_components_filters_hidden(tmp_path):
    """Test that hidden directories are filtered out."""
    # Create visible and hidden components
    (tmp_path / "components" / "comp1" / "src").mkdir(parents=True)
    (tmp_path / "components" / "comp1" / "CLAUDE.md").write_text("# Component 1")
    (tmp_path / "components" / ".hidden" / "src").mkdir(parents=True)

    discovery = StateDiscovery(tmp_path)
    components = discovery._discover_components()

    assert len(components) == 1
    assert "comp1" in components
    assert ".hidden" not in components


def test_discover_state_structure(discovery):
    """Test that discover_state returns proper structure."""
    state = discovery.discover_state()

    assert "discovery_timestamp" in state
    assert "discovery_method" in state
    assert "confidence" in state
    assert "discovered_state" in state
    assert "inferred_context" in state
    assert "missing_information" in state


def test_discover_state_finds_components(discovery):
    """Test that discover_state finds components."""
    state = discovery.discover_state()

    components = state["discovered_state"]["components_found"]
    assert len(components) == 2
    assert "comp1" in components
    assert "comp2" in components


def test_confidence_assessment(discovery):
    """Test confidence assessment logic."""
    state = discovery.discover_state()

    # With mock project, should have some missing info
    assert state["confidence"] in ["high", "medium", "low"]


def test_confidence_high(tmp_path):
    """Test high confidence when all info present."""
    # Create full project structure
    (tmp_path / "components" / "comp1" / "src").mkdir(parents=True)
    (tmp_path / "components" / "comp1" / "CLAUDE.md").write_text("# Component 1")

    # Create orchestration-state.json
    import json
    with open(tmp_path / "orchestration-state.json", 'w') as f:
        json.dump({"phase": 3}, f)

    discovery = StateDiscovery(tmp_path)
    state = discovery.discover_state()

    # Should have medium or high confidence
    assert state["confidence"] in ["high", "medium"]


def test_infer_context_with_components(discovery):
    """Test context inference when components exist."""
    state = discovery.discover_state()

    inferred = state["inferred_context"]
    assert "likely_current_phase" in inferred
    assert "likely_original_specs" in inferred
    assert "estimated_progress_percent" in inferred

    # Should infer at least phase 2 (components exist)
    assert inferred["likely_current_phase"] >= 2


def test_infer_context_no_components(tmp_path):
    """Test context inference with no components."""
    discovery = StateDiscovery(tmp_path)
    state = discovery.discover_state()

    inferred = state["inferred_context"]

    # Should infer phase 0 or 1 (nothing exists)
    assert inferred["likely_current_phase"] <= 1


def test_missing_information_tracking(discovery):
    """Test that missing information is tracked."""
    state = discovery.discover_state()

    missing = state["missing_information"]
    assert isinstance(missing, list)

    # Should note that orchestration-state.json is missing
    assert any("orchestration-state.json" in item for item in missing)


def test_discover_state_with_specs(tmp_path):
    """Test spec file discovery."""
    # Create specifications directory
    (tmp_path / "specifications").mkdir()
    (tmp_path / "specifications" / "spec1.md").write_text("# Spec 1")
    (tmp_path / "specifications" / "spec2.yaml").write_text("openapi: 3.0.0")

    # Create component to trigger discovery
    (tmp_path / "components" / "comp1" / "src").mkdir(parents=True)
    (tmp_path / "components" / "comp1" / "CLAUDE.md").write_text("# Component 1")

    discovery = StateDiscovery(tmp_path)
    state = discovery.discover_state()

    specs = state["inferred_context"]["likely_original_specs"]
    assert len(specs) >= 2
    assert any("spec1.md" in s for s in specs)
    assert any("spec2.yaml" in s for s in specs)


def test_load_orchestration_state(tmp_path):
    """Test loading orchestration-state.json."""
    import json

    # Create state file
    state_data = {"phase": 3, "components": ["comp1"]}
    with open(tmp_path / "orchestration-state.json", 'w') as f:
        json.dump(state_data, f)

    discovery = StateDiscovery(tmp_path)
    state = discovery._load_orchestration_state()

    assert state is not None
    assert state["phase"] == 3


def test_load_orchestration_state_missing(tmp_path):
    """Test graceful handling when state file missing."""
    discovery = StateDiscovery(tmp_path)
    state = discovery._load_orchestration_state()

    assert state is None


def test_estimate_progress(tmp_path):
    """Test progress estimation."""
    # Create components (phase 2)
    (tmp_path / "components" / "comp1" / "src").mkdir(parents=True)
    (tmp_path / "components" / "comp1" / "CLAUDE.md").write_text("# Component 1")

    discovery = StateDiscovery(tmp_path)
    state = discovery.discover_state()

    progress = state["inferred_context"]["estimated_progress_percent"]
    assert isinstance(progress, int)
    assert 0 <= progress <= 100

    # Should estimate at least phase 2 / 6 = 33%
    assert progress >= 33
