#!/usr/bin/env python3
"""
Semantic Correctness Verification

Goes beyond syntax to verify business logic completeness, error handling,
and data flow correctness. Part of v0.4.0 quality enhancement system - Batch 2.

This module provides:
- Business logic completeness verification
- Error handling completeness checking
- Data flow validation
- Security implementation verification
- PII leak detection
- SQL injection vulnerability detection
"""

import ast
from pathlib import Path
from typing import List, Dict, Optional, Set, Tuple
from dataclasses import dataclass, asdict
import yaml
import re
import json


@dataclass
class SemanticIssue:
    """A semantic correctness issue."""
    file_path: str
    line_number: int
    issue_type: str
    severity: str  # "critical", "warning", "info"
    description: str
    requirement_id: Optional[str]
    suggestion: str

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)


@dataclass
class VerificationResult:
    """Result of semantic verification."""
    component_path: str
    passed: bool
    issues: List[SemanticIssue]
    business_logic_complete: bool
    error_handling_complete: bool
    data_flow_valid: bool
    security_implemented: bool

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return {
            'component_path': self.component_path,
            'passed': self.passed,
            'issues': [i.to_dict() for i in self.issues],
            'business_logic_complete': self.business_logic_complete,
            'error_handling_complete': self.error_handling_complete,
            'data_flow_valid': self.data_flow_valid,
            'security_implemented': self.security_implemented
        }


class SemanticVerifier:
    """Verifies semantic correctness of implementations."""

    def __init__(self, project_root: Path):
        self.project_root = Path(project_root)
        self.rules_file = self.project_root / "orchestration" / "semantic_rules.yaml"
        self.rules = self._load_rules()

    def _load_rules(self) -> dict:
        """Load semantic verification rules."""
        if self.rules_file.exists():
            with open(self.rules_file, 'r') as f:
                return yaml.safe_load(f)
        return self._default_rules()

    def _default_rules(self) -> dict:
        """Default semantic rules."""
        return {
            'business_logic_patterns': {
                'password_reset': {
                    'required_elements': [
                        'token_generation',
                        'token_storage',
                        'expiry_check',
                        'invalidation_after_use',
                        'rate_limiting'
                    ],
                    'detection_keywords': ['reset_password', 'password_reset', 'forgot_password']
                },
                'user_registration': {
                    'required_elements': [
                        'email_uniqueness_check',
                        'password_hashing',
                        'activation_email',
                        'duplicate_prevention'
                    ],
                    'detection_keywords': ['register', 'signup', 'create_user']
                },
                'authentication': {
                    'required_elements': [
                        'password_verification',
                        'session_creation',
                        'failed_attempt_tracking',
                        'account_lockout'
                    ],
                    'detection_keywords': ['login', 'authenticate', 'sign_in']
                },
                'payment_processing': {
                    'required_elements': [
                        'amount_validation',
                        'idempotency_check',
                        'transaction_logging',
                        'rollback_mechanism',
                        'fraud_check'
                    ],
                    'detection_keywords': ['payment', 'charge', 'process_payment', 'transaction']
                }
            },
            'error_handling_requirements': {
                'database_operations': {
                    'must_handle': [
                        'connection_failure',
                        'timeout',
                        'constraint_violation',
                        'deadlock'
                    ],
                    'detection_patterns': [
                        r'\.execute\(',
                        r'\.query\(',
                        r'\.insert\(',
                        r'\.update\(',
                        r'\.delete\(',
                        r'\.commit\('
                    ]
                },
                'external_api_calls': {
                    'must_handle': [
                        'timeout',
                        'connection_refused',
                        'invalid_response',
                        'rate_limit_exceeded',
                        'authentication_failure'
                    ],
                    'detection_patterns': [
                        r'requests\.',
                        r'httpx\.',
                        r'urllib\.',
                        r'http\.client',
                        r'aiohttp\.'
                    ]
                },
                'file_operations': {
                    'must_handle': [
                        'file_not_found',
                        'permission_denied',
                        'disk_full'
                    ],
                    'detection_patterns': [
                        r'open\(',
                        r'\.read\(',
                        r'\.write\(',
                        r'Path\('
                    ]
                }
            },
            'security_patterns': {
                'pii_fields': [
                    'password', 'ssn', 'social_security', 'credit_card',
                    'card_number', 'cvv', 'pin', 'secret', 'token',
                    'api_key', 'private_key'
                ],
                'sql_injection_vulnerable': [
                    r'execute\s*\(\s*f["\']',
                    r'execute\s*\(\s*["\'].*\+',
                    r'query\s*\(\s*f["\']',
                    r'query\s*\(\s*["\'].*\+'
                ],
                'required_authentication_decorators': [
                    'require_auth', 'login_required', 'authenticated',
                    'require_permission', 'authorize'
                ]
            }
        }

    def verify_component(self, component_path: Path) -> VerificationResult:
        """Comprehensive semantic verification of component."""
        component_path = Path(component_path)
        issues = []

        # Run all verification checks
        issues.extend(self.verify_business_logic_completeness(component_path))
        issues.extend(self.verify_error_handling_completeness(component_path))
        issues.extend(self.verify_data_flow_completeness(component_path))
        issues.extend(self.verify_security_implementation(component_path))

        # Determine overall pass/fail
        critical_issues = [i for i in issues if i.severity == "critical"]
        passed = len(critical_issues) == 0

        return VerificationResult(
            component_path=str(component_path),
            passed=passed,
            issues=issues,
            business_logic_complete=len([i for i in issues if "business_logic" in i.issue_type]) == 0,
            error_handling_complete=len([i for i in issues if "error_handling" in i.issue_type]) == 0,
            data_flow_valid=len([i for i in issues if "data_flow" in i.issue_type]) == 0,
            security_implemented=len([i for i in issues if "security" in i.issue_type]) == 0
        )

    def verify_business_logic_completeness(self, component_path: Path) -> List[SemanticIssue]:
        """
        Verify business logic is complete.

        For each business rule pattern:
        1. Find relevant code sections
        2. Check all required elements present
        3. Verify logic matches specification
        """
        issues = []

        # Get all Python files
        for py_file in component_path.rglob("*.py"):
            # Skip test files and pycache
            if "__pycache__" in str(py_file):
                continue
            if py_file.name.startswith("test_") or "/tests/" in str(py_file):
                continue

            tree = self._parse_file(py_file)
            if not tree:
                continue

            # Check for business logic patterns
            issues.extend(self._check_password_reset_logic(py_file, tree))
            issues.extend(self._check_registration_logic(py_file, tree))
            issues.extend(self._check_authentication_logic(py_file, tree))
            issues.extend(self._check_payment_logic(py_file, tree))

        return issues

    def _check_password_reset_logic(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check password reset implementation completeness."""
        issues = []
        pattern = self.rules['business_logic_patterns']['password_reset']

        # Look for password reset function
        reset_functions = self._find_functions_matching(
            tree,
            pattern['detection_keywords']
        )

        if not reset_functions:
            return issues  # No password reset in this file

        for func in reset_functions:
            required_elements = {element: False for element in pattern['required_elements']}

            # Analyze function body
            func_code = self._get_function_source(func)
            func_code_lower = func_code.lower()

            # Check for token generation (must have generation function, not just variable)
            if any(term in func_code_lower for term in ['secrets.token', 'uuid.uuid', 'random.', 'generate_token', 'create_token']):
                required_elements['token_generation'] = True

            # Check for token storage
            if any(term in func_code_lower for term in ['db.session.add', 'save', 'store', '.insert', 'database', 'create(']):
                required_elements['token_storage'] = True

            # Check for expiry check
            if any(term in func_code_lower for term in ['expir', 'timeout', 'valid_until', 'created_at', 'ttl', 'lifetime', 'timedelta']):
                required_elements['expiry_check'] = True

            # Check for invalidation
            if any(term in func_code_lower for term in ['invalidate', '.delete', 'used', 'consumed', 'revoke', 'remove']):
                required_elements['invalidation_after_use'] = True

            # Check for rate limiting
            if any(term in func_code_lower for term in ['rate_limit', 'throttle', 'attempts', 'cooldown', 'backoff', 'max_attempts']):
                required_elements['rate_limiting'] = True

            # Report missing elements
            for element, present in required_elements.items():
                if not present:
                    issues.append(SemanticIssue(
                        file_path=str(file_path.relative_to(self.project_root)),
                        line_number=func.lineno,
                        issue_type="business_logic_incomplete",
                        severity="critical",
                        description=f"Password reset missing: {element}",
                        requirement_id=None,
                        suggestion=self._get_element_suggestion(element, 'password_reset')
                    ))

        return issues

    def _check_registration_logic(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check user registration implementation completeness."""
        issues = []
        pattern = self.rules['business_logic_patterns']['user_registration']

        registration_functions = self._find_functions_matching(
            tree,
            pattern['detection_keywords']
        )

        if not registration_functions:
            return issues

        for func in registration_functions:
            required_elements = {element: False for element in pattern['required_elements']}
            func_code = self._get_function_source(func)
            func_code_lower = func_code.lower()

            # Check for email uniqueness
            if any(term in func_code_lower for term in ['.filter_by', '.exists', 'check_email', 'get_by_email', '.first()']):
                required_elements['email_uniqueness_check'] = True

            # Check for password hashing
            if any(term in func_code_lower for term in ['bcrypt', 'hashpw', 'pbkdf2', 'argon', 'scrypt', 'hash_password']):
                required_elements['password_hashing'] = True

            # Check for activation email
            if any(term in func_code_lower for term in ['send_email', 'send_activation', 'activation_email', 'verify_email']):
                required_elements['activation_email'] = True

            # Check for duplicate prevention
            if any(term in func_code_lower for term in ['integrityerror', 'unique', 'constraint', 'rollback']):
                required_elements['duplicate_prevention'] = True

            for element, present in required_elements.items():
                if not present:
                    issues.append(SemanticIssue(
                        file_path=str(file_path.relative_to(self.project_root)),
                        line_number=func.lineno,
                        issue_type="business_logic_incomplete",
                        severity="critical",
                        description=f"User registration missing: {element}",
                        requirement_id=None,
                        suggestion=self._get_element_suggestion(element, 'user_registration')
                    ))

        return issues

    def _check_authentication_logic(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check authentication implementation completeness."""
        issues = []
        pattern = self.rules['business_logic_patterns']['authentication']

        auth_functions = self._find_functions_matching(
            tree,
            pattern['detection_keywords']
        )

        if not auth_functions:
            return issues

        for func in auth_functions:
            required_elements = {element: False for element in pattern['required_elements']}
            func_code = self._get_function_source(func)
            func_code_lower = func_code.lower()

            # Check for password verification
            if any(term in func_code_lower for term in ['checkpw', 'verify_password', 'check_password', 'validate_password', 'password.verify']):
                required_elements['password_verification'] = True

            # Check for session creation
            if any(term in func_code_lower for term in ['create_session', 'generate_token', 'jwt.encode', 'set_cookie']):
                required_elements['session_creation'] = True

            # Check for failed attempt tracking
            if any(term in func_code_lower for term in ['failed_attempts', 'login_attempts', 'increment', 'track_attempt']):
                required_elements['failed_attempt_tracking'] = True

            # Check for account lockout (more specific patterns to avoid matching comments)
            if any(term in func_code_lower for term in ['user.locked', 'account.locked', 'is_locked', 'disable', 'suspend', 'max_attempts', 'account_lock', '.locked =']):
                required_elements['account_lockout'] = True

            for element, present in required_elements.items():
                if not present:
                    issues.append(SemanticIssue(
                        file_path=str(file_path.relative_to(self.project_root)),
                        line_number=func.lineno,
                        issue_type="business_logic_incomplete",
                        severity="critical",
                        description=f"Authentication missing: {element}",
                        requirement_id=None,
                        suggestion=self._get_element_suggestion(element, 'authentication')
                    ))

        return issues

    def _check_payment_logic(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check payment processing implementation completeness."""
        issues = []
        pattern = self.rules['business_logic_patterns']['payment_processing']

        payment_functions = self._find_functions_matching(
            tree,
            pattern['detection_keywords']
        )

        if not payment_functions:
            return issues

        for func in payment_functions:
            required_elements = {element: False for element in pattern['required_elements']}
            func_code = self._get_function_source(func)
            func_code_lower = func_code.lower()

            # Check for amount validation
            if any(term in func_code_lower for term in ['validate', 'amount', 'positive', 'range', 'decimal']):
                required_elements['amount_validation'] = True

            # Check for idempotency
            if any(term in func_code_lower for term in ['idempotent', 'duplicate', 'unique_id', 'transaction_id']):
                required_elements['idempotency_check'] = True

            # Check for transaction logging
            if any(term in func_code_lower for term in ['log', 'audit', 'record', 'history']):
                required_elements['transaction_logging'] = True

            # Check for rollback
            if any(term in func_code_lower for term in ['rollback', 'revert', 'undo', 'compensate']):
                required_elements['rollback_mechanism'] = True

            # Check for fraud check
            if any(term in func_code_lower for term in ['fraud', 'risk', 'verify', 'suspicious']):
                required_elements['fraud_check'] = True

            for element, present in required_elements.items():
                if not present:
                    # Fraud check is warning, others are critical
                    severity = "warning" if element == "fraud_check" else "critical"
                    issues.append(SemanticIssue(
                        file_path=str(file_path.relative_to(self.project_root)),
                        line_number=func.lineno,
                        issue_type="business_logic_incomplete",
                        severity=severity,
                        description=f"Payment processing missing: {element}",
                        requirement_id=None,
                        suggestion=self._get_element_suggestion(element, 'payment_processing')
                    ))

        return issues

    def verify_error_handling_completeness(self, component_path: Path) -> List[SemanticIssue]:
        """
        Verify error handling is complete.

        For every risky operation:
        - Database calls: timeout, connection error, constraint violation
        - External API: timeout, connection error, invalid response
        - File operations: not found, permission denied
        """
        issues = []

        for py_file in component_path.rglob("*.py"):
            # Skip test files and pycache
            if "__pycache__" in str(py_file):
                continue
            if py_file.name.startswith("test_") or "/tests/" in str(py_file):
                continue

            tree = self._parse_file(py_file)
            if not tree:
                continue

            # Check database operations
            issues.extend(self._check_database_error_handling(py_file, tree))

            # Check external API calls
            issues.extend(self._check_api_error_handling(py_file, tree))

            # Check file operations
            issues.extend(self._check_file_error_handling(py_file, tree))

        return issues

    def _check_database_error_handling(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check database operations have proper error handling."""
        issues = []

        class DatabaseCallVisitor(ast.NodeVisitor):
            def __init__(self):
                self.db_calls = []

            def visit_Call(self, node):
                # Look for database calls
                if hasattr(node.func, 'attr'):
                    if node.func.attr in ['execute', 'query', 'insert', 'update', 'delete', 'commit', 'rollback']:
                        self.db_calls.append(node)
                elif hasattr(node.func, 'id'):
                    if node.func.id in ['execute', 'query']:
                        self.db_calls.append(node)
                self.generic_visit(node)

        visitor = DatabaseCallVisitor()
        visitor.visit(tree)

        for call in visitor.db_calls:
            # Check if wrapped in try/except
            if not self._is_in_try_block(tree, call):
                issues.append(SemanticIssue(
                    file_path=str(file_path.relative_to(self.project_root)),
                    line_number=call.lineno,
                    issue_type="error_handling_missing",
                    severity="critical",
                    description="Database operation without error handling",
                    requirement_id=None,
                    suggestion="Wrap database call in try/except to handle connection/timeout/constraint errors"
                ))

        return issues

    def _check_api_error_handling(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check external API calls have proper error handling."""
        issues = []

        class APICallVisitor(ast.NodeVisitor):
            def __init__(self):
                self.api_calls = []

            def visit_Call(self, node):
                # Look for HTTP library calls
                if hasattr(node.func, 'value'):
                    if hasattr(node.func.value, 'id'):
                        if node.func.value.id in ['requests', 'httpx', 'urllib', 'aiohttp']:
                            self.api_calls.append(node)
                self.generic_visit(node)

        visitor = APICallVisitor()
        visitor.visit(tree)

        for call in visitor.api_calls:
            if not self._is_in_try_block(tree, call):
                issues.append(SemanticIssue(
                    file_path=str(file_path.relative_to(self.project_root)),
                    line_number=call.lineno,
                    issue_type="error_handling_missing",
                    severity="critical",
                    description="External API call without error handling",
                    requirement_id=None,
                    suggestion="Wrap API call in try/except to handle timeout/connection/response errors"
                ))

        return issues

    def _check_file_error_handling(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check file operations have proper error handling."""
        issues = []

        class FileOpVisitor(ast.NodeVisitor):
            def __init__(self):
                self.file_ops = []

            def visit_Call(self, node):
                # Look for open() calls
                if hasattr(node.func, 'id'):
                    if node.func.id == 'open':
                        self.file_ops.append(node)
                self.generic_visit(node)

            def visit_With(self, node):
                # Look for with open() as f:
                for item in node.items:
                    if hasattr(item.context_expr, 'func'):
                        if hasattr(item.context_expr.func, 'id'):
                            if item.context_expr.func.id == 'open':
                                self.file_ops.append(item.context_expr)
                self.generic_visit(node)

        visitor = FileOpVisitor()
        visitor.visit(tree)

        for op in visitor.file_ops:
            # with statements have implicit error handling, but check if explicit too
            if not self._is_in_try_block(tree, op) and not self._is_in_with_block(tree, op):
                issues.append(SemanticIssue(
                    file_path=str(file_path.relative_to(self.project_root)),
                    line_number=op.lineno,
                    issue_type="error_handling_missing",
                    severity="warning",
                    description="File operation without explicit error handling",
                    requirement_id=None,
                    suggestion="Add try/except to handle FileNotFoundError and PermissionError"
                ))

        return issues

    def verify_data_flow_completeness(self, component_path: Path) -> List[SemanticIssue]:
        """
        Verify data flow through component.

        Checks:
        - Input validation at entry points
        - Data transformations applied correctly
        - Output formatting correct
        - No data leaks (PII not logged)
        """
        issues = []

        for py_file in component_path.rglob("*.py"):
            # Skip test files and pycache
            if "__pycache__" in str(py_file):
                continue
            if py_file.name.startswith("test_") or "/tests/" in str(py_file):
                continue

            tree = self._parse_file(py_file)
            if not tree:
                continue

            # Check for input validation
            issues.extend(self._check_input_validation(py_file, tree))

            # Check for PII in logs
            issues.extend(self._check_pii_logging(py_file, tree))

        return issues

    def _check_input_validation(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check that functions validate their inputs."""
        issues = []

        class FunctionVisitor(ast.NodeVisitor):
            def __init__(self):
                self.functions = []

            def visit_FunctionDef(self, node):
                # Look for public functions (not starting with _)
                if not node.name.startswith('_') and len(node.args.args) > 0:
                    self.functions.append(node)
                self.generic_visit(node)

        visitor = FunctionVisitor()
        visitor.visit(tree)

        for func in visitor.functions:
            # Check if function validates inputs
            has_validation = False
            func_code = self._get_function_source(func)

            # Look for validation patterns
            validation_keywords = ['validate', 'check', 'verify', 'assert', 'raise', 'isinstance', 'if not']
            if any(keyword in func_code.lower() for keyword in validation_keywords):
                has_validation = True

            if not has_validation and len(func.args.args) > 1:  # Ignore self/cls
                issues.append(SemanticIssue(
                    file_path=str(file_path.relative_to(self.project_root)),
                    line_number=func.lineno,
                    issue_type="data_flow_validation_missing",
                    severity="warning",
                    description=f"Function '{func.name}' lacks input validation",
                    requirement_id=None,
                    suggestion="Add input validation at the beginning of the function"
                ))

        return issues

    def _check_pii_logging(self, file_path: Path, tree: ast.AST) -> List[SemanticIssue]:
        """Check that PII is not logged."""
        issues = []
        pii_fields = self.rules['security_patterns']['pii_fields']

        class LoggingVisitor(ast.NodeVisitor):
            def __init__(self):
                self.log_calls = []

            def visit_Call(self, node):
                # Look for logging calls
                if hasattr(node.func, 'attr'):
                    if node.func.attr in ['log', 'info', 'debug', 'warning', 'error', 'critical']:
                        self.log_calls.append(node)
                elif hasattr(node.func, 'value'):
                    if hasattr(node.func.value, 'id'):
                        if node.func.value.id in ['logger', 'log', 'logging']:
                            self.log_calls.append(node)
                self.generic_visit(node)

        visitor = LoggingVisitor()
        visitor.visit(tree)

        # Get source code
        try:
            with open(file_path, 'r') as f:
                source_lines = f.readlines()
        except:
            return issues

        for call in visitor.log_calls:
            # Get the line of code
            if call.lineno <= len(source_lines):
                line = source_lines[call.lineno - 1]

                # Check for PII fields in the log statement
                for pii_field in pii_fields:
                    if pii_field in line.lower():
                        issues.append(SemanticIssue(
                            file_path=str(file_path.relative_to(self.project_root)),
                            line_number=call.lineno,
                            issue_type="data_flow_pii_leak",
                            severity="critical",
                            description=f"Potential PII leak: '{pii_field}' in log statement",
                            requirement_id=None,
                            suggestion=f"Remove or mask '{pii_field}' from log output"
                        ))

        return issues

    def verify_security_implementation(self, component_path: Path) -> List[SemanticIssue]:
        """
        Verify security requirements implemented.

        Checks:
        - Authentication checked
        - Authorization enforced
        - Input sanitization
        - SQL injection prevention
        - XSS prevention
        """
        issues = []

        for py_file in component_path.rglob("*.py"):
            # Skip test files and pycache
            if "__pycache__" in str(py_file):
                continue
            if py_file.name.startswith("test_") or "/tests/" in str(py_file):
                continue

            # Check SQL injection vulnerabilities
            issues.extend(self._check_sql_injection(py_file))

            # Check authentication on endpoints
            issues.extend(self._check_authentication_decorators(py_file))

        return issues

    def _check_sql_injection(self, file_path: Path) -> List[SemanticIssue]:
        """Check for SQL injection vulnerabilities."""
        issues = []

        try:
            with open(file_path, 'r') as f:
                source = f.read()
        except:
            return issues

        # Check for f-strings or string concatenation in SQL
        # Pattern 1: f-string with SQL keywords
        f_string_pattern = r'f["\'].*(?:SELECT|INSERT|UPDATE|DELETE|FROM|WHERE).*\{.*\}.*["\']'
        matches = re.finditer(f_string_pattern, source, re.IGNORECASE | re.DOTALL)
        for match in matches:
            line_number = source[:match.start()].count('\n') + 1
            issues.append(SemanticIssue(
                file_path=str(file_path.relative_to(self.project_root)),
                line_number=line_number,
                issue_type="security_sql_injection",
                severity="critical",
                description="Potential SQL injection vulnerability: f-string with SQL query",
                requirement_id=None,
                suggestion="Use parameterized queries with placeholders (?, %s) instead of f-strings"
            ))

        # Pattern 2: String concatenation with SQL keywords
        concat_pattern = r'["\'].*(?:SELECT|INSERT|UPDATE|DELETE|FROM|WHERE).*["\'].*\+.*["\']'
        matches = re.finditer(concat_pattern, source, re.IGNORECASE)
        for match in matches:
            line_number = source[:match.start()].count('\n') + 1
            issues.append(SemanticIssue(
                file_path=str(file_path.relative_to(self.project_root)),
                line_number=line_number,
                issue_type="security_sql_injection",
                severity="critical",
                description="Potential SQL injection vulnerability: string concatenation in SQL query",
                requirement_id=None,
                suggestion="Use parameterized queries with placeholders (?, %s) instead of string formatting"
            ))

        # Pattern 3: .format() with SQL
        format_pattern = r'["\'].*(?:SELECT|INSERT|UPDATE|DELETE|FROM|WHERE).*["\']\.format\('
        matches = re.finditer(format_pattern, source, re.IGNORECASE)
        for match in matches:
            line_number = source[:match.start()].count('\n') + 1
            issues.append(SemanticIssue(
                file_path=str(file_path.relative_to(self.project_root)),
                line_number=line_number,
                issue_type="security_sql_injection",
                severity="critical",
                description="Potential SQL injection vulnerability: .format() with SQL query",
                requirement_id=None,
                suggestion="Use parameterized queries with placeholders (?, %s) instead of .format()"
            ))

        return issues

    def _check_authentication_decorators(self, file_path: Path) -> List[SemanticIssue]:
        """Check that endpoints have authentication decorators."""
        issues = []

        tree = self._parse_file(file_path)
        if not tree:
            return issues

        class EndpointVisitor(ast.NodeVisitor):
            def __init__(self):
                self.endpoints = []

            def visit_FunctionDef(self, node):
                # Look for route decorators
                has_route = False
                has_auth = False

                for decorator in node.decorator_list:
                    decorator_name = ""
                    if hasattr(decorator, 'attr'):
                        decorator_name = decorator.attr
                    elif hasattr(decorator, 'id'):
                        decorator_name = decorator.id
                    elif hasattr(decorator, 'func'):
                        if hasattr(decorator.func, 'attr'):
                            decorator_name = decorator.func.attr

                    if decorator_name in ['route', 'get', 'post', 'put', 'delete', 'patch']:
                        has_route = True
                    if any(auth in decorator_name.lower() for auth in ['auth', 'login', 'require', 'permission']):
                        has_auth = True

                if has_route and not has_auth:
                    self.endpoints.append(node)

                self.generic_visit(node)

        visitor = EndpointVisitor()
        visitor.visit(tree)

        for endpoint in visitor.endpoints:
            # Some endpoints like /health, /metrics don't need auth
            if endpoint.name in ['health', 'metrics', 'ping', 'status']:
                continue

            issues.append(SemanticIssue(
                file_path=str(file_path.relative_to(self.project_root)),
                line_number=endpoint.lineno,
                issue_type="security_missing_authentication",
                severity="warning",
                description=f"Endpoint '{endpoint.name}' lacks authentication decorator",
                requirement_id=None,
                suggestion="Add authentication decorator (@require_auth, @login_required, etc.) to endpoint"
            ))

        return issues

    def _parse_file(self, file_path: Path) -> Optional[ast.AST]:
        """Parse Python file to AST."""
        try:
            with open(file_path, 'r') as f:
                return ast.parse(f.read())
        except:
            return None

    def _find_functions_matching(self, tree: ast.AST, patterns: List[str]) -> List[ast.FunctionDef]:
        """Find functions matching name patterns."""
        functions = []

        class FunctionVisitor(ast.NodeVisitor):
            def visit_FunctionDef(self, node):
                if any(pattern in node.name.lower() for pattern in patterns):
                    functions.append(node)
                self.generic_visit(node)

        visitor = FunctionVisitor()
        visitor.visit(tree)
        return functions

    def _get_function_source(self, func_node: ast.FunctionDef) -> str:
        """Get source code of a function."""
        try:
            return ast.unparse(func_node)
        except:
            return ""

    def _is_in_try_block(self, tree: ast.AST, target_node: ast.AST) -> bool:
        """Check if node is within a try/except block."""
        # Build parent map
        parent_map = {}

        class ParentMapper(ast.NodeVisitor):
            def visit(self, node):
                for child in ast.iter_child_nodes(node):
                    parent_map[child] = node
                self.generic_visit(node)

        ParentMapper().visit(tree)

        # Walk up from target node to check if any parent is a Try node
        current = target_node
        while current in parent_map:
            parent = parent_map[current]
            if isinstance(parent, ast.Try):
                # Check if target is in the try body (not in handlers/else/finally)
                if current in parent.body or any(self._node_in_tree(current, child) for child in parent.body):
                    return True
            current = parent

        return False

    def _node_in_tree(self, needle: ast.AST, haystack: ast.AST) -> bool:
        """Check if needle node is somewhere in haystack tree."""
        if needle is haystack:
            return True
        for child in ast.iter_child_nodes(haystack):
            if self._node_in_tree(needle, child):
                return True
        return False

    def _is_in_with_block(self, tree: ast.AST, target_node: ast.AST) -> bool:
        """Check if node is within a with block."""
        class WithVisitor(ast.NodeVisitor):
            def __init__(self):
                self.has_with = False

            def visit_With(self, node):
                self.has_with = True
                self.generic_visit(node)

        visitor = WithVisitor()
        visitor.visit(tree)
        return visitor.has_with

    def _get_element_suggestion(self, element: str, pattern_type: str) -> str:
        """Get suggestion for missing element."""
        suggestions = {
            'password_reset': {
                'token_generation': "Generate a cryptographically secure random token using secrets.token_urlsafe(32)",
                'token_storage': "Store token in database with user_id, created_at, expires_at columns",
                'expiry_check': "Check if current time is before token.expires_at (typically 1 hour expiry)",
                'invalidation_after_use': "Delete or mark token as used after successful password reset",
                'rate_limiting': "Limit password reset requests to 3 per hour per email"
            },
            'user_registration': {
                'email_uniqueness_check': "Query database for existing user with email before creating",
                'password_hashing': "Use bcrypt.hashpw() or argon2 to hash password before storing",
                'activation_email': "Send email with activation link/token to verify email ownership",
                'duplicate_prevention': "Use database unique constraint on email column and handle exception"
            },
            'authentication': {
                'password_verification': "Use bcrypt.checkpw() or password_hash.verify() to verify password",
                'session_creation': "Create session token/JWT after successful authentication",
                'failed_attempt_tracking': "Increment failed_attempts counter on each failed login",
                'account_lockout': "Lock account for 30 minutes after 5 failed attempts"
            },
            'payment_processing': {
                'amount_validation': "Validate amount is positive decimal with max 2 decimal places",
                'idempotency_check': "Check if transaction_id already exists before processing",
                'transaction_logging': "Log all transaction details to audit table",
                'rollback_mechanism': "Implement database transaction with rollback on failure",
                'fraud_check': "Integrate fraud detection service or implement basic risk scoring"
            }
        }

        return suggestions.get(pattern_type, {}).get(element, f"Implement {element.replace('_', ' ')}")

    def generate_report(self, result: VerificationResult) -> str:
        """Generate formatted report."""
        report = []
        report.append("="*70)
        report.append(f"SEMANTIC VERIFICATION: {Path(result.component_path).name}")
        report.append("="*70)
        report.append("")

        if result.passed:
            report.append("✅ PASSED - All semantic checks passed")
        else:
            report.append("❌ FAILED - Semantic issues found")

        report.append("")
        report.append(f"Business Logic Complete: {'✅' if result.business_logic_complete else '❌'}")
        report.append(f"Error Handling Complete: {'✅' if result.error_handling_complete else '❌'}")
        report.append(f"Data Flow Valid: {'✅' if result.data_flow_valid else '❌'}")
        report.append(f"Security Implemented: {'✅' if result.security_implemented else '❌'}")

        if result.issues:
            report.append("")
            report.append(f"Issues Found: {len(result.issues)}")

            # Group by severity
            critical = [i for i in result.issues if i.severity == "critical"]
            warnings = [i for i in result.issues if i.severity == "warning"]
            info = [i for i in result.issues if i.severity == "info"]

            if critical:
                report.append("")
                report.append(f"CRITICAL ISSUES ({len(critical)}):")
                report.append("-" * 70)
                for issue in critical:
                    report.append(f"  {issue.description}")
                    report.append(f"    File: {issue.file_path}:{issue.line_number}")
                    report.append(f"    Type: {issue.issue_type}")
                    report.append(f"    Suggestion: {issue.suggestion}")
                    report.append("")

            if warnings:
                report.append("")
                report.append(f"WARNINGS ({len(warnings)}):")
                report.append("-" * 70)
                for issue in warnings:
                    report.append(f"  {issue.description}")
                    report.append(f"    File: {issue.file_path}:{issue.line_number}")
                    report.append(f"    Suggestion: {issue.suggestion}")
                    report.append("")

            if info:
                report.append("")
                report.append(f"INFO ({len(info)}):")
                report.append("-" * 70)
                for issue in info:
                    report.append(f"  {issue.description}")
                    report.append(f"    File: {issue.file_path}:{issue.line_number}")
                    report.append("")

        report.append("="*70)
        return "\n".join(report)

    def save_report(self, result: VerificationResult, output_path: Path) -> None:
        """Save verification result to JSON file."""
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, 'w') as f:
            json.dump(result.to_dict(), f, indent=2)


def main():
    """CLI interface."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: semantic_verifier.py <component_path> [--json output.json]")
        print("")
        print("Examples:")
        print("  semantic_verifier.py components/auth-service/")
        print("  semantic_verifier.py components/payment/ --json results.json")
        sys.exit(1)

    component_path = Path(sys.argv[1])
    if not component_path.exists():
        print(f"Error: Component path does not exist: {component_path}")
        sys.exit(1)

    verifier = SemanticVerifier(Path.cwd())
    result = verifier.verify_component(component_path)

    # Generate and print report
    print(verifier.generate_report(result))

    # Save JSON if requested
    if '--json' in sys.argv:
        json_index = sys.argv.index('--json')
        if json_index + 1 < len(sys.argv):
            output_path = Path(sys.argv[json_index + 1])
            verifier.save_report(result, output_path)
            print(f"\nResults saved to: {output_path}")

    sys.exit(0 if result.passed else 1)


if __name__ == '__main__':
    main()
