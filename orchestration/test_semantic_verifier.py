#!/usr/bin/env python3
"""
Tests for Semantic Correctness Verification

Comprehensive test suite for semantic_verifier.py
Part of v0.4.0 quality enhancement system - Batch 2.
"""

import unittest
from pathlib import Path
import tempfile
import shutil
from semantic_verifier import (
    SemanticVerifier,
    SemanticIssue,
    VerificationResult
)


class TestSemanticVerifier(unittest.TestCase):
    """Test SemanticVerifier class."""

    def setUp(self):
        """Create temporary directory for test files."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.component_dir = self.test_dir / "components" / "test_component"
        self.component_dir.mkdir(parents=True, exist_ok=True)

        # Create orchestration directory for rules
        self.orch_dir = self.test_dir / "orchestration"
        self.orch_dir.mkdir(parents=True, exist_ok=True)

        self.verifier = SemanticVerifier(self.test_dir)

    def tearDown(self):
        """Clean up temporary directory."""
        shutil.rmtree(self.test_dir)

    def test_initialization(self):
        """Test SemanticVerifier initialization."""
        self.assertIsNotNone(self.verifier)
        self.assertEqual(self.verifier.project_root, self.test_dir)
        self.assertIsNotNone(self.verifier.rules)

    def test_default_rules_loaded(self):
        """Test that default rules are loaded."""
        rules = self.verifier._default_rules()

        self.assertIn('business_logic_patterns', rules)
        self.assertIn('error_handling_requirements', rules)
        self.assertIn('security_patterns', rules)

        # Check password reset pattern
        self.assertIn('password_reset', rules['business_logic_patterns'])
        password_reset = rules['business_logic_patterns']['password_reset']
        self.assertIn('token_generation', password_reset['required_elements'])

    def test_password_reset_complete(self):
        """Test password reset with all required elements."""
        code = '''
def reset_password(email):
    """Complete password reset implementation."""
    # Token generation
    token = secrets.token_urlsafe(32)

    # Token storage
    db.session.add(ResetToken(email=email, token=token))

    # Expiry check
    expires_at = datetime.now() + timedelta(hours=1)

    # Invalidation after use
    old_tokens = ResetToken.query.filter_by(email=email).all()
    for old_token in old_tokens:
        db.session.delete(old_token)

    # Rate limiting
    attempts = get_reset_attempts(email)
    if attempts > 3:
        raise RateLimitError()

    return token
'''
        test_file = self.component_dir / "password.py"
        test_file.write_text(code)

        issues = self.verifier.verify_business_logic_completeness(self.component_dir)
        password_issues = [i for i in issues if 'password_reset' in i.file_path.lower()]

        # Should have no critical issues
        critical_issues = [i for i in password_issues if i.severity == "critical"]
        self.assertEqual(len(critical_issues), 0)

    def test_password_reset_incomplete(self):
        """Test password reset missing required elements."""
        code = '''
def reset_password(email):
    """Incomplete password reset - missing several elements."""
    token = "simple_token"
    return token
'''
        test_file = self.component_dir / "password.py"
        test_file.write_text(code)

        issues = self.verifier.verify_business_logic_completeness(self.component_dir)
        password_issues = [i for i in issues if 'Password reset' in i.description]

        # Should have multiple critical issues
        self.assertGreater(len(password_issues), 0)
        self.assertTrue(any('token_storage' in i.description for i in password_issues))

    def test_user_registration_complete(self):
        """Test user registration with all required elements."""
        code = '''
def register_user(email, password):
    """Complete user registration."""
    # Email uniqueness check
    existing = User.query.filter_by(email=email).first()
    if existing:
        raise DuplicateEmailError()

    # Password hashing
    hashed_password = bcrypt.hashpw(password.encode(), bcrypt.gensalt())

    # Activation email
    send_activation_email(email)

    # Duplicate prevention
    try:
        user = User(email=email, password=hashed_password)
        db.session.add(user)
        db.session.commit()
    except IntegrityError:
        db.session.rollback()
        raise DuplicateEmailError()

    return user
'''
        test_file = self.component_dir / "registration.py"
        test_file.write_text(code)

        issues = self.verifier.verify_business_logic_completeness(self.component_dir)
        registration_issues = [i for i in issues if 'registration' in i.description.lower()]

        # Should have no critical issues
        critical_issues = [i for i in registration_issues if i.severity == "critical"]
        self.assertEqual(len(critical_issues), 0)

    def test_authentication_incomplete(self):
        """Test authentication missing account lockout."""
        code = '''
def login(username, password):
    """Incomplete authentication - missing lockout."""
    user = User.query.filter_by(username=username).first()

    # Password verification
    if not user or not bcrypt.checkpw(password.encode(), user.password):
        return None

    # Session creation
    session = create_session(user)
    return session
'''
        test_file = self.component_dir / "auth.py"
        test_file.write_text(code)

        issues = self.verifier.verify_business_logic_completeness(self.component_dir)
        auth_issues = [i for i in issues if 'Authentication' in i.description]

        # Should have issues for missing elements
        self.assertGreater(len(auth_issues), 0)
        self.assertTrue(any('account_lockout' in i.description for i in auth_issues))

    def test_database_operation_without_error_handling(self):
        """Test database operation without try/except."""
        code = '''
def get_user(user_id):
    """Database operation without error handling."""
    result = db.execute("SELECT * FROM users WHERE id = ?", (user_id,))
    return result
'''
        test_file = self.component_dir / "database.py"
        test_file.write_text(code)

        issues = self.verifier.verify_error_handling_completeness(self.component_dir)
        db_issues = [i for i in issues if 'Database operation' in i.description]

        # Should have critical issue
        self.assertGreater(len(db_issues), 0)
        self.assertEqual(db_issues[0].severity, "critical")

    def test_database_operation_with_error_handling(self):
        """Test database operation with proper error handling."""
        code = '''
def get_user(user_id):
    """Database operation with error handling."""
    try:
        result = db.execute("SELECT * FROM users WHERE id = ?", (user_id,))
        return result
    except ConnectionError as e:
        logger.error(f"Database connection failed: {e}")
        raise
    except TimeoutError as e:
        logger.error(f"Database timeout: {e}")
        raise
'''
        test_file = self.component_dir / "database.py"
        test_file.write_text(code)

        issues = self.verifier.verify_error_handling_completeness(self.component_dir)
        db_issues = [i for i in issues if 'Database operation' in i.description]

        # Should have no issues
        self.assertEqual(len(db_issues), 0)

    def test_api_call_without_error_handling(self):
        """Test external API call without error handling."""
        code = '''
import requests

def fetch_data(url):
    """API call without error handling."""
    response = requests.get(url)
    return response.json()
'''
        test_file = self.component_dir / "api_client.py"
        test_file.write_text(code)

        issues = self.verifier.verify_error_handling_completeness(self.component_dir)
        api_issues = [i for i in issues if 'API call' in i.description]

        # Should have critical issue
        self.assertGreater(len(api_issues), 0)
        self.assertEqual(api_issues[0].severity, "critical")

    def test_api_call_with_error_handling(self):
        """Test external API call with proper error handling."""
        code = '''
import requests

def fetch_data(url):
    """API call with error handling."""
    try:
        response = requests.get(url, timeout=30)
        response.raise_for_status()
        return response.json()
    except requests.Timeout:
        logger.error("Request timed out")
        raise
    except requests.ConnectionError:
        logger.error("Connection failed")
        raise
'''
        test_file = self.component_dir / "api_client.py"
        test_file.write_text(code)

        issues = self.verifier.verify_error_handling_completeness(self.component_dir)
        api_issues = [i for i in issues if 'API call' in i.description]

        # Should have no issues
        self.assertEqual(len(api_issues), 0)

    def test_pii_in_logging(self):
        """Test PII detection in log statements."""
        code = '''
def process_user(user):
    """Function that logs PII."""
    logger.info(f"Processing user with password: {user.password}")
    logger.debug(f"User SSN: {user.ssn}")
    logger.info(f"Credit card: {user.credit_card}")
'''
        test_file = self.component_dir / "user_processor.py"
        test_file.write_text(code)

        issues = self.verifier.verify_data_flow_completeness(self.component_dir)
        pii_issues = [i for i in issues if 'PII leak' in i.description]

        # Should have multiple critical issues
        self.assertGreaterEqual(len(pii_issues), 2)
        for issue in pii_issues:
            self.assertEqual(issue.severity, "critical")

    def test_safe_logging(self):
        """Test safe logging without PII."""
        code = '''
def process_user(user):
    """Function with safe logging."""
    logger.info(f"Processing user ID: {user.id}")
    logger.debug(f"User email domain: {user.email.split('@')[1]}")
'''
        test_file = self.component_dir / "user_processor.py"
        test_file.write_text(code)

        issues = self.verifier.verify_data_flow_completeness(self.component_dir)
        pii_issues = [i for i in issues if 'PII leak' in i.description]

        # Should have no PII issues
        self.assertEqual(len(pii_issues), 0)

    def test_sql_injection_vulnerable(self):
        """Test SQL injection vulnerability detection."""
        code = '''
def get_user_by_name(name):
    """Vulnerable to SQL injection."""
    query = f"SELECT * FROM users WHERE name = '{name}'"
    result = db.execute(query)
    return result
'''
        test_file = self.component_dir / "vulnerable.py"
        test_file.write_text(code)

        issues = self.verifier.verify_security_implementation(self.component_dir)
        sql_issues = [i for i in issues if 'SQL injection' in i.description]

        # Should have critical issue
        self.assertGreater(len(sql_issues), 0)
        self.assertEqual(sql_issues[0].severity, "critical")

    def test_sql_injection_safe(self):
        """Test safe parameterized query."""
        code = '''
def get_user_by_name(name):
    """Safe parameterized query."""
    query = "SELECT * FROM users WHERE name = ?"
    result = db.execute(query, (name,))
    return result
'''
        test_file = self.component_dir / "safe.py"
        test_file.write_text(code)

        issues = self.verifier.verify_security_implementation(self.component_dir)
        sql_issues = [i for i in issues if 'SQL injection' in i.description]

        # Should have no issues
        self.assertEqual(len(sql_issues), 0)

    def test_endpoint_without_authentication(self):
        """Test endpoint without authentication decorator."""
        code = '''
from flask import Flask

app = Flask(__name__)

@app.route('/api/users', methods=['POST'])
def create_user():
    """Endpoint without authentication."""
    return {"status": "created"}
'''
        test_file = self.component_dir / "api.py"
        test_file.write_text(code)

        issues = self.verifier.verify_security_implementation(self.component_dir)
        auth_issues = [i for i in issues if 'authentication decorator' in i.description]

        # Should have warning
        self.assertGreater(len(auth_issues), 0)
        self.assertEqual(auth_issues[0].severity, "warning")

    def test_endpoint_with_authentication(self):
        """Test endpoint with authentication decorator."""
        code = '''
from flask import Flask

app = Flask(__name__)

@app.route('/api/users', methods=['POST'])
@require_auth
def create_user():
    """Endpoint with authentication."""
    return {"status": "created"}
'''
        test_file = self.component_dir / "api.py"
        test_file.write_text(code)

        issues = self.verifier.verify_security_implementation(self.component_dir)
        auth_issues = [i for i in issues if 'authentication decorator' in i.description]

        # Should have no issues
        self.assertEqual(len(auth_issues), 0)

    def test_health_endpoint_no_auth_required(self):
        """Test that health endpoints don't require auth."""
        code = '''
from flask import Flask

app = Flask(__name__)

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint - no auth required."""
    return {"status": "ok"}
'''
        test_file = self.component_dir / "api.py"
        test_file.write_text(code)

        issues = self.verifier.verify_security_implementation(self.component_dir)
        auth_issues = [i for i in issues if 'authentication decorator' in i.description]

        # Should have no issues for health endpoint
        self.assertEqual(len(auth_issues), 0)

    def test_verification_result_structure(self):
        """Test VerificationResult structure."""
        result = VerificationResult(
            component_path="/test/path",
            passed=True,
            issues=[],
            business_logic_complete=True,
            error_handling_complete=True,
            data_flow_valid=True,
            security_implemented=True
        )

        self.assertTrue(result.passed)
        self.assertEqual(len(result.issues), 0)

        # Test to_dict
        result_dict = result.to_dict()
        self.assertIsInstance(result_dict, dict)
        self.assertIn('component_path', result_dict)
        self.assertIn('passed', result_dict)

    def test_semantic_issue_structure(self):
        """Test SemanticIssue structure."""
        issue = SemanticIssue(
            file_path="test.py",
            line_number=42,
            issue_type="test_issue",
            severity="critical",
            description="Test description",
            requirement_id="REQ-001",
            suggestion="Test suggestion"
        )

        self.assertEqual(issue.file_path, "test.py")
        self.assertEqual(issue.line_number, 42)
        self.assertEqual(issue.severity, "critical")

        # Test to_dict
        issue_dict = issue.to_dict()
        self.assertIsInstance(issue_dict, dict)
        self.assertEqual(issue_dict['line_number'], 42)

    def test_complete_component_verification(self):
        """Test complete component verification."""
        # Create a simple component with some issues
        code = '''
def reset_password(email):
    """Incomplete password reset."""
    token = "simple"
    return token

def fetch_data(url):
    """API call without error handling."""
    import requests
    return requests.get(url).json()
'''
        test_file = self.component_dir / "main.py"
        test_file.write_text(code)

        result = self.verifier.verify_component(self.component_dir)

        # Should fail due to critical issues
        self.assertFalse(result.passed)
        self.assertGreater(len(result.issues), 0)

        # Check component path
        self.assertIn('test_component', result.component_path)

    def test_report_generation(self):
        """Test report generation."""
        issues = [
            SemanticIssue(
                file_path="test.py",
                line_number=10,
                issue_type="business_logic_incomplete",
                severity="critical",
                description="Missing validation",
                requirement_id=None,
                suggestion="Add validation"
            ),
            SemanticIssue(
                file_path="test.py",
                line_number=20,
                issue_type="error_handling_missing",
                severity="warning",
                description="Missing error handler",
                requirement_id=None,
                suggestion="Add try/except"
            )
        ]

        result = VerificationResult(
            component_path=str(self.component_dir),
            passed=False,
            issues=issues,
            business_logic_complete=False,
            error_handling_complete=False,
            data_flow_valid=True,
            security_implemented=True
        )

        report = self.verifier.generate_report(result)

        # Check report content
        self.assertIn("SEMANTIC VERIFICATION", report)
        self.assertIn("FAILED", report)
        self.assertIn("CRITICAL ISSUES", report)
        self.assertIn("WARNINGS", report)
        self.assertIn("Missing validation", report)

    def test_empty_component(self):
        """Test verification of empty component."""
        result = self.verifier.verify_component(self.component_dir)

        # Empty component should pass (no code, no issues)
        self.assertTrue(result.passed)
        self.assertEqual(len(result.issues), 0)

    def test_payment_processing_complete(self):
        """Test payment processing with all required elements."""
        code = '''
def process_payment(amount, transaction_id):
    """Complete payment processing."""
    # Amount validation
    if amount <= 0 or amount > 1000000:
        raise ValueError("Invalid amount")

    # Idempotency check
    existing = Transaction.query.filter_by(transaction_id=transaction_id).first()
    if existing:
        return existing.result

    # Transaction logging
    log_transaction(amount, transaction_id)

    # Rollback mechanism
    try:
        charge_card(amount)
        update_balance(amount)
        db.session.commit()
    except Exception as e:
        db.session.rollback()
        raise

    # Fraud check (optional but recommended)
    fraud_score = check_fraud_risk(amount)
    if fraud_score > 0.8:
        flag_transaction(transaction_id)

    return {"status": "success"}
'''
        test_file = self.component_dir / "payment.py"
        test_file.write_text(code)

        issues = self.verifier.verify_business_logic_completeness(self.component_dir)
        payment_issues = [i for i in issues if 'Payment processing' in i.description]

        # Should have no critical issues
        critical_issues = [i for i in payment_issues if i.severity == "critical"]
        self.assertEqual(len(critical_issues), 0)

    def test_input_validation_missing(self):
        """Test detection of missing input validation."""
        code = '''
def process_data(user_id, data, options):
    """Function without input validation."""
    result = data * 2
    return result
'''
        test_file = self.component_dir / "processor.py"
        test_file.write_text(code)

        issues = self.verifier.verify_data_flow_completeness(self.component_dir)
        validation_issues = [i for i in issues if 'input validation' in i.description.lower()]

        # Should have warning about missing validation
        self.assertGreater(len(validation_issues), 0)

    def test_input_validation_present(self):
        """Test detection of present input validation."""
        code = '''
def process_data(user_id, data, options):
    """Function with input validation."""
    # Validate inputs
    if not isinstance(user_id, int):
        raise ValueError("user_id must be integer")
    if data is None:
        raise ValueError("data is required")
    if not isinstance(options, dict):
        raise ValueError("options must be dict")

    result = data * 2
    return result
'''
        test_file = self.component_dir / "processor.py"
        test_file.write_text(code)

        issues = self.verifier.verify_data_flow_completeness(self.component_dir)
        validation_issues = [i for i in issues if 'input validation' in i.description.lower()]

        # Should have no issues
        self.assertEqual(len(validation_issues), 0)

    def test_get_element_suggestion(self):
        """Test suggestion generation for missing elements."""
        suggestion = self.verifier._get_element_suggestion('token_generation', 'password_reset')

        self.assertIsInstance(suggestion, str)
        self.assertGreater(len(suggestion), 10)
        self.assertIn('token', suggestion.lower())

    def test_json_serialization(self):
        """Test JSON serialization of results."""
        import json

        issue = SemanticIssue(
            file_path="test.py",
            line_number=42,
            issue_type="test",
            severity="critical",
            description="Test",
            requirement_id=None,
            suggestion="Fix it"
        )

        result = VerificationResult(
            component_path="/test",
            passed=False,
            issues=[issue],
            business_logic_complete=True,
            error_handling_complete=False,
            data_flow_valid=True,
            security_implemented=True
        )

        # Should be JSON serializable
        result_dict = result.to_dict()
        json_str = json.dumps(result_dict, indent=2)
        self.assertIsInstance(json_str, str)

        # Parse back
        parsed = json.loads(json_str)
        self.assertEqual(parsed['component_path'], '/test')
        self.assertFalse(parsed['passed'])


class TestSemanticVerifierIntegration(unittest.TestCase):
    """Integration tests for semantic verifier."""

    def setUp(self):
        """Create temporary directory for test files."""
        self.test_dir = Path(tempfile.mkdtemp())
        self.component_dir = self.test_dir / "components" / "test_service"
        self.component_dir.mkdir(parents=True, exist_ok=True)

        # Create src directory
        self.src_dir = self.component_dir / "src"
        self.src_dir.mkdir(parents=True, exist_ok=True)

        self.verifier = SemanticVerifier(self.test_dir)

    def tearDown(self):
        """Clean up temporary directory."""
        shutil.rmtree(self.test_dir)

    def test_realistic_component_structure(self):
        """Test verification of realistic component structure."""
        # Create models.py
        models_code = '''
from sqlalchemy import Column, Integer, String, DateTime
from database import Base

class User(Base):
    __tablename__ = 'users'

    id = Column(Integer, primary_key=True)
    email = Column(String(255), unique=True)
    password_hash = Column(String(255))
    created_at = Column(DateTime)
'''
        (self.src_dir / "models.py").write_text(models_code)

        # Create auth.py with issues
        auth_code = '''
def register_user(email, password):
    """User registration - incomplete."""
    user = User(email=email, password=password)  # Missing hashing!
    db.session.add(user)  # No error handling!
    db.session.commit()
    return user

def login(email, password):
    """Login - incomplete."""
    user = User.query.filter_by(email=email).first()
    if user and user.password == password:  # Plaintext comparison!
        return create_session(user)
    return None
'''
        (self.src_dir / "auth.py").write_text(auth_code)

        # Run verification
        result = self.verifier.verify_component(self.component_dir)

        # Should fail with multiple issues
        self.assertFalse(result.passed)
        self.assertGreater(len(result.issues), 0)

        # Should have business logic issues
        self.assertFalse(result.business_logic_complete)


def run_tests():
    """Run all tests."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    # Add all test classes
    suite.addTests(loader.loadTestsFromTestCase(TestSemanticVerifier))
    suite.addTests(loader.loadTestsFromTestCase(TestSemanticVerifierIntegration))

    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    return result.wasSuccessful()


if __name__ == '__main__':
    import sys
    success = run_tests()
    sys.exit(0 if success else 1)
