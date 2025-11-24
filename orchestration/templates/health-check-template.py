"""
Health check endpoint template.

Every component MUST have a /health endpoint for integration testing.
Add this to your main.py or app.py file.
"""

# For FastAPI
from fastapi import FastAPI

app = FastAPI()

@app.get("/health")
async def health_check():
    """
    Health check endpoint.

    Required for integration testing - allows test fixtures to verify
    that service is ready before running tests.

    Returns:
        dict: Status information
    """
    return {
        "status": "healthy",
        "service": "{{COMPONENT_NAME}}",
        "version": "{{PROJECT_VERSION}}"
    }


# For Flask
from flask import Flask, jsonify

app = Flask(__name__)

@app.route("/health")
def health_check():
    """
    Health check endpoint.

    Required for integration testing - allows test fixtures to verify
    that service is ready before running tests.

    Returns:
        JSON response with status information
    """
    return jsonify({
        "status": "healthy",
        "service": "{{COMPONENT_NAME}}",
        "version": "{{PROJECT_VERSION}}"
    }), 200


# For Express (Node.js)
"""
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    service: '{{COMPONENT_NAME}}',
    version: '{{PROJECT_VERSION}}'
  });
});
"""
