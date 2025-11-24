"""
Pytest fixtures for cross-component integration testing.

This module provides fixtures that start component services as subprocesses
and make them available to integration tests.
"""

import pytest
from pathlib import Path
from .service_runner import ServiceRunner


@pytest.fixture(scope="session")
def project_root():
    """Path to project root directory."""
    return Path(__file__).parent.parent.parent


@pytest.fixture(scope="module")
def service_runner(project_root):
    """
    Service runner that manages component services as subprocesses.

    Usage in tests:
        def test_something(service_runner):
            auth_url = service_runner.start_service('auth-service', 8001)
            user_url = service_runner.start_service('user-service', 8002)
            # ... test using auth_url and user_url
    """
    runner = ServiceRunner(project_root)
    yield runner
    runner.stop_all()


@pytest.fixture(scope="module")
def running_services(service_runner):
    """
    Start all component services and return their URLs.

    This fixture automatically starts all services found in components/
    and assigns them sequential ports starting at 8001.

    Returns:
        dict: Mapping of component name to base URL
              e.g., {'auth-service': 'http://localhost:8001', ...}
    """
    from pathlib import Path

    component_dir = service_runner.project_root / "components"
    components = [d.name for d in component_dir.iterdir() if d.is_dir()]

    services = {}
    port = 8001

    for component in sorted(components):
        try:
            url = service_runner.start_service(component, port)
            services[component] = url
            port += 1
        except RuntimeError as e:
            pytest.skip(f"Could not start {component}: {e}")

    return services
