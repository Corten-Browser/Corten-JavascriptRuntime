"""
Service runner utilities for cross-component integration testing.

This module provides utilities to start component services as subprocesses
and manage their lifecycle during integration tests.
"""

import subprocess
import time
import signal
import requests
from pathlib import Path
from typing import Dict, List, Tuple
import logging

logger = logging.getLogger(__name__)


class ServiceRunner:
    """Manages component services as subprocesses for integration testing."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.processes: List[Tuple[str, subprocess.Popen]] = []

    def start_service(self, component_name: str, port: int,
                     health_endpoint: str = "/health",
                     timeout: int = 30) -> str:
        """
        Start a component service as a subprocess.

        Args:
            component_name: Name of component (e.g., "auth-service")
            port: Port to run service on
            health_endpoint: Health check endpoint path
            timeout: Seconds to wait for service to be healthy

        Returns:
            Base URL of started service (e.g., "http://localhost:8001")

        Raises:
            RuntimeError: If service fails to start or become healthy
        """
        component_path = self.project_root / "components" / component_name

        if not component_path.exists():
            raise RuntimeError(f"Component not found: {component_path}")

        # Determine how to run the service based on what's present
        main_py = component_path / "main.py"
        app_py = component_path / "app.py"

        if main_py.exists():
            cmd = [
                'python', '-m', 'uvicorn',
                f'components.{component_name.replace("-", "_")}.main:app',
                '--port', str(port),
                '--host', '0.0.0.0',
                '--log-level', 'warning'
            ]
        elif app_py.exists():
            cmd = [
                'python', '-m', 'uvicorn',
                f'components.{component_name.replace("-", "_")}.app:app',
                '--port', str(port),
                '--host', '0.0.0.0',
                '--log-level', 'warning'
            ]
        else:
            raise RuntimeError(
                f"Cannot determine how to run {component_name}. "
                "Expected main.py or app.py"
            )

        logger.info(f"Starting {component_name} on port {port}...")

        # Start the service
        proc = subprocess.Popen(
            cmd,
            cwd=self.project_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )

        self.processes.append((component_name, proc))

        # Wait for service to be healthy
        base_url = f"http://localhost:{port}"
        health_url = f"{base_url}{health_endpoint}"

        try:
            self._wait_for_health(health_url, timeout)
            logger.info(f"{component_name} is healthy at {base_url}")
            return base_url
        except RuntimeError as e:
            self.stop_all()
            raise RuntimeError(f"Failed to start {component_name}: {e}")

    def _wait_for_health(self, health_url: str, timeout: int):
        """Wait for service health check to succeed."""
        start = time.time()
        last_error = None

        while time.time() - start < timeout:
            try:
                response = requests.get(health_url, timeout=2)
                if response.status_code == 200:
                    return
                last_error = f"HTTP {response.status_code}"
            except requests.ConnectionError as e:
                last_error = f"Connection error: {e}"
            except requests.Timeout:
                last_error = "Request timeout"

            time.sleep(0.5)

        raise RuntimeError(
            f"Service did not become healthy in {timeout}s. "
            f"Last error: {last_error}"
        )

    def stop_all(self):
        """Stop all running services."""
        for name, proc in self.processes:
            logger.info(f"Stopping {name}...")
            try:
                proc.terminate()
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                logger.warning(f"{name} did not stop gracefully, killing...")
                proc.kill()
                proc.wait()

        self.processes.clear()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop_all()
