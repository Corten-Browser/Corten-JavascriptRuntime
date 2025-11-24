#!/usr/bin/env python3
"""
E2E Test Template for Web Services/APIs

This template tests web services from the user's perspective by:
1. Starting service as subprocess (not importing)
2. Making real HTTP requests
3. Verifying complete workflows

Usage:
    1. Copy this template to your component's tests/e2e/ directory
    2. Replace all <PLACEHOLDERS> with your values
    3. Run: pytest tests/e2e/ -v
"""

import subprocess
import time
import requests
import pytest
from pathlib import Path
import signal
import psutil

# ============================================================================
# CONFIGURATION - REPLACE THESE PLACEHOLDERS
# ============================================================================

# Replace with your service module path (e.g., 'components.api_server')
MODULE_PATH = '<MODULE_PATH>'

# Replace with your service port (e.g., 8000)
SERVICE_PORT = '<PORT>'

# ============================================================================
# FIXTURES
# ============================================================================

@pytest.fixture(scope="module")
def running_service():
    """
    Start web service as subprocess.

    This is how users run the service, not how developers test it.
    """
    # Start service
    process = subprocess.Popen(
        ['python', '-m', MODULE_PATH],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # Wait for service to be ready
    max_wait = 30
    service_url = f'http://localhost:{SERVICE_PORT}'

    for _ in range(max_wait):
        try:
            response = requests.get(f'{service_url}/health', timeout=1)
            if response.status_code == 200:
                break
        except requests.exceptions.RequestException:
            time.sleep(1)
    else:
        process.terminate()
        stdout, stderr = process.communicate(timeout=5)
        pytest.fail(
            f"Service failed to start within {max_wait} seconds\n"
            f"STDOUT: {stdout}\n"
            f"STDERR: {stderr}"
        )

    yield service_url

    # Cleanup: terminate service
    try:
        # Get all child processes
        parent = psutil.Process(process.pid)
        children = parent.children(recursive=True)

        # Terminate children first
        for child in children:
            child.terminate()

        # Terminate parent
        process.terminate()
        process.wait(timeout=5)
    except Exception:
        process.kill()


# ============================================================================
# E2E TESTS - WEB SERVICE
# ============================================================================

class TestWebServiceEndToEnd:
    """E2E tests for web service."""

    def test_service_health_check(self, running_service):
        """Verify service starts and responds to health check."""
        response = requests.get(f'{running_service}/health')
        assert response.status_code == 200

    def test_documented_api_workflow(self, running_service):
        """
        Execute complete API workflow from documentation.

        This is the user's perspective - making HTTP requests.

        TODO: Customize this workflow for your API.
        Example below shows: Register → Login → Access protected resource
        """
        # Step 1: Register
        register_response = requests.post(
            f'{running_service}/register',
            json={
                'email': 'test@example.com',
                'password': 'SecurePass123!'
            }
        )
        assert register_response.status_code == 201, (
            f"Registration failed: {register_response.status_code}\n"
            f"Response: {register_response.text}"
        )

        # Step 2: Login
        login_response = requests.post(
            f'{running_service}/login',
            json={
                'email': 'test@example.com',
                'password': 'SecurePass123!'
            }
        )
        assert login_response.status_code == 200, (
            f"Login failed: {login_response.status_code}\n"
            f"Response: {login_response.text}"
        )
        token = login_response.json()['token']

        # Step 3: Access protected resource
        profile_response = requests.get(
            f'{running_service}/profile',
            headers={'Authorization': f'Bearer {token}'}
        )
        assert profile_response.status_code == 200
        assert profile_response.json()['email'] == 'test@example.com'

    def test_all_documented_endpoints_exist(self, running_service):
        """
        Verify all documented endpoints are accessible.

        Catches: Documentation mentioning endpoints that don't exist.

        TODO: If you have an OpenAPI/contract file, uncomment and customize
        """
        # import yaml
        # api_spec = Path('contracts/<service>-api.yaml')
        #
        # if not api_spec.exists():
        #     pytest.skip("No API contract found")
        #
        # with open(api_spec) as f:
        #     spec = yaml.safe_load(f)
        #
        # # Test each endpoint
        # for path, methods in spec['paths'].items():
        #     for method in methods.keys():
        #         response = requests.request(
        #             method.upper(),
        #             f'{running_service}{path}',
        #             headers={'Content-Type': 'application/json'}
        #         )
        #
        #         # Should not be 404 (endpoint must exist)
        #         assert response.status_code != 404, (
        #             f"Documented endpoint {method.upper()} {path} returns 404"
        #         )
        pass


# ============================================================================
# USAGE INSTRUCTIONS
# ============================================================================

"""
To use this template:

1. Replace placeholders at top of file:
   - MODULE_PATH: e.g., 'components.api_server'
   - SERVICE_PORT: e.g., 8000

2. Customize test_documented_api_workflow():
   - Implement your actual API workflow
   - Test complete user journeys (not just single endpoints)

3. Optionally enable test_all_documented_endpoints_exist():
   - Uncomment code
   - Point to your API contract/spec file

4. Run tests:
   ```bash
   pytest tests/e2e/ -v
   ```

These E2E tests catch:
✅ Service startup failures
✅ Broken endpoint routing
✅ Authentication/authorization issues
✅ Database persistence failures
✅ Documentation inaccuracies

REMEMBER: E2E tests start the service as a subprocess and make
real HTTP requests. This is how users interact with your API.
"""
