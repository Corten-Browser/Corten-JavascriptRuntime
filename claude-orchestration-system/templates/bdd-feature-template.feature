# Feature: {{FEATURE_NAME}}
#
# This is a Gherkin feature file for Behavior-Driven Development (BDD).
#
# BDD describes behavior from the user's perspective using Given-When-Then scenarios.
# Each scenario should:
# - Be independent (can run in any order)
# - Test one specific behavior
# - Be readable by non-technical stakeholders
#
# Gherkin Keywords:
# - Feature: High-level description of the feature
# - Background: Steps that run before each scenario
# - Scenario: A specific test case
# - Given: Preconditions/setup
# - When: Actions/triggers
# - Then: Expected outcomes
# - And/But: Additional steps
# - Scenario Outline: Template for multiple test cases with examples
#
# Replace placeholders:
# - {{FEATURE_NAME}}: Name of the feature (e.g., "User Registration")
# - {{FEATURE_DESCRIPTION}}: Brief description of the feature
# - {{BENEFIT}}: Why this feature matters to users
# - {{USER_ROLE}}: Who uses this feature (e.g., "new user", "admin")
#

Feature: {{FEATURE_NAME}}
  {{FEATURE_DESCRIPTION}}

  As a {{USER_ROLE}}
  I want to {{FEATURE_ACTION}}
  So that {{BENEFIT}}

  # Background runs before each scenario
  # Use for common setup steps
  Background:
    Given the system is running
    And the database is seeded with test data

  # ══════════════════════════════════════════════════════════════════
  # Happy Path Scenarios
  # ══════════════════════════════════════════════════════════════════

  Scenario: {{HAPPY_PATH_SCENARIO_NAME}}
    Given {{PRECONDITION_1}}
    And {{PRECONDITION_2}}
    When {{ACTION_1}}
    And {{ACTION_2}}
    Then {{EXPECTED_OUTCOME_1}}
    And {{EXPECTED_OUTCOME_2}}

  # Example: User Registration (Happy Path)
  @happy-path @critical
  Scenario: Successful user registration with valid data
    Given I am on the registration page
    And no user exists with email "newuser@example.com"
    When I enter email "newuser@example.com"
    And I enter name "John Doe"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see "Registration successful"
    And I should be redirected to the dashboard
    And I should receive a welcome email at "newuser@example.com"
    And a user account should exist with email "newuser@example.com"

  # ══════════════════════════════════════════════════════════════════
  # Error/Validation Scenarios
  # ══════════════════════════════════════════════════════════════════

  Scenario: {{ERROR_SCENARIO_NAME}}
    Given {{ERROR_PRECONDITION}}
    When {{ERROR_ACTION}}
    Then {{ERROR_OUTCOME}}
    And {{ERROR_MESSAGE}}

  # Example: Registration with Missing Data
  @validation @error
  Scenario: Registration fails when required fields are missing
    Given I am on the registration page
    When I enter email "user@example.com"
    And I leave the name field empty
    And I click the "Register" button
    Then I should see an error message "Name is required"
    And I should remain on the registration page
    And no new user account should be created

  # Example: Registration with Duplicate Email
  @validation @error
  Scenario: Registration fails when email already exists
    Given a user exists with email "existing@example.com"
    And I am on the registration page
    When I enter email "existing@example.com"
    And I enter name "Duplicate User"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see an error message "Email already registered"
    And I should remain on the registration page
    And no new user account should be created

  # Example: Registration with Invalid Email Format
  @validation @error
  Scenario: Registration fails with invalid email format
    Given I am on the registration page
    When I enter email "invalid-email-format"
    And I enter name "Test User"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see an error message "Invalid email format"
    And the email field should be highlighted in red
    And I should remain on the registration page

  # ══════════════════════════════════════════════════════════════════
  # Scenario Outlines (Data-Driven Tests)
  # ══════════════════════════════════════════════════════════════════

  # Use Scenario Outline to test multiple similar cases with different data
  @validation @password
  Scenario Outline: Password validation with various invalid passwords
    Given I am on the registration page
    When I enter email "user@example.com"
    And I enter name "Test User"
    And I enter password "<password>"
    And I click the "Register" button
    Then I should see an error message "<error_message>"
    And I should remain on the registration page

    Examples:
      | password    | error_message                              |
      | short       | Password must be at least 8 characters     |
      | nodigit!!!  | Password must contain at least one digit   |
      | noupper123  | Password must contain an uppercase letter  |
      | NOLOWER123  | Password must contain a lowercase letter   |
      | NoSpecial1  | Password must contain a special character  |

  # ══════════════════════════════════════════════════════════════════
  # Edge Cases
  # ══════════════════════════════════════════════════════════════════

  @edge-case
  Scenario: {{EDGE_CASE_SCENARIO_NAME}}
    Given {{EDGE_CASE_PRECONDITION}}
    When {{EDGE_CASE_ACTION}}
    Then {{EDGE_CASE_OUTCOME}}

  # Example: Very Long Input
  @edge-case @validation
  Scenario: Registration handles very long name gracefully
    Given I am on the registration page
    When I enter email "user@example.com"
    And I enter a name with 256 characters
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see an error message "Name must be less than 100 characters"
    And the name field should be highlighted in red

  # Example: Special Characters in Name
  @edge-case
  Scenario: Registration accepts names with special characters
    Given I am on the registration page
    When I enter email "user@example.com"
    And I enter name "José María O'Brien-Smith"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see "Registration successful"
    And a user account should exist with name "José María O'Brien-Smith"

  # ══════════════════════════════════════════════════════════════════
  # Security Scenarios
  # ══════════════════════════════════════════════════════════════════

  @security @critical
  Scenario: {{SECURITY_SCENARIO_NAME}}
    Given {{SECURITY_PRECONDITION}}
    When {{SECURITY_ACTION}}
    Then {{SECURITY_OUTCOME}}

  # Example: XSS Prevention
  @security @xss
  Scenario: Registration prevents XSS attacks in name field
    Given I am on the registration page
    When I enter email "user@example.com"
    And I enter name "<script>alert('XSS')</script>"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see "Registration successful"
    And the stored name should be "&lt;script&gt;alert('XSS')&lt;/script&gt;"
    And no JavaScript should be executed

  # Example: SQL Injection Prevention
  @security @sql-injection
  Scenario: Registration prevents SQL injection in email field
    Given I am on the registration page
    When I enter email "user@example.com'; DROP TABLE users;--"
    And I enter name "Test User"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see an error message "Invalid email format"
    And the users table should still exist
    And no SQL should be executed

  # Example: Password Storage
  @security @password
  Scenario: Password is hashed before storage
    Given I am on the registration page
    When I enter email "user@example.com"
    And I enter name "Test User"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see "Registration successful"
    And the stored password should not equal "SecurePass123!"
    And the stored password should be a bcrypt hash

  # ══════════════════════════════════════════════════════════════════
  # Integration Scenarios
  # ══════════════════════════════════════════════════════════════════

  @integration @email
  Scenario: {{INTEGRATION_SCENARIO_NAME}}
    Given {{INTEGRATION_PRECONDITION}}
    When {{INTEGRATION_ACTION}}
    Then {{INTEGRATION_OUTCOME}}

  # Example: Email Service Integration
  @integration @email
  Scenario: Welcome email is sent after successful registration
    Given I am on the registration page
    And the email service is available
    When I enter email "user@example.com"
    And I enter name "Test User"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see "Registration successful"
    And a welcome email should be sent to "user@example.com"
    And the email should contain the user's name "Test User"
    And the email should contain a verification link

  # Example: Email Service Failure Handling
  @integration @email @error
  Scenario: Registration succeeds even if email service fails
    Given I am on the registration page
    And the email service is unavailable
    When I enter email "user@example.com"
    And I enter name "Test User"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see "Registration successful"
    And a user account should be created
    And an error should be logged about email failure
    But the registration should not fail

  # ══════════════════════════════════════════════════════════════════
  # Performance Scenarios
  # ══════════════════════════════════════════════════════════════════

  @performance
  Scenario: {{PERFORMANCE_SCENARIO_NAME}}
    Given {{PERFORMANCE_PRECONDITION}}
    When {{PERFORMANCE_ACTION}}
    Then {{PERFORMANCE_OUTCOME}}

  # Example: Registration Response Time
  @performance @non-functional
  Scenario: Registration completes within acceptable time
    Given I am on the registration page
    When I enter email "user@example.com"
    And I enter name "Test User"
    And I enter password "SecurePass123!"
    And I click the "Register" button
    Then I should see "Registration successful" within 2 seconds
    And the response time should be less than 500 milliseconds

  # ══════════════════════════════════════════════════════════════════
  # Accessibility Scenarios
  # ══════════════════════════════════════════════════════════════════

  @accessibility @a11y
  Scenario: {{ACCESSIBILITY_SCENARIO_NAME}}
    Given {{ACCESSIBILITY_PRECONDITION}}
    When {{ACCESSIBILITY_ACTION}}
    Then {{ACCESSIBILITY_OUTCOME}}

  # Example: Keyboard Navigation
  @accessibility @keyboard
  Scenario: Registration form is accessible via keyboard
    Given I am on the registration page
    When I press Tab to focus the email field
    And I type "user@example.com"
    And I press Tab to focus the name field
    And I type "Test User"
    And I press Tab to focus the password field
    And I type "SecurePass123!"
    And I press Tab to focus the Register button
    And I press Enter
    Then I should see "Registration successful"

  # Example: Screen Reader Support
  @accessibility @screen-reader
  Scenario: Registration form has proper ARIA labels
    Given I am on the registration page
    When I inspect the form with a screen reader
    Then the email field should have aria-label "Email address"
    And the name field should have aria-label "Full name"
    And the password field should have aria-label "Password"
    And the Register button should have aria-label "Register new account"
    And error messages should have role "alert"

  # ══════════════════════════════════════════════════════════════════
  # Notes for Implementation
  # ══════════════════════════════════════════════════════════════════

  # Tags Usage:
  # - @happy-path: Core functionality working correctly
  # - @critical: Must-pass scenarios for release
  # - @validation: Input validation tests
  # - @error: Error handling tests
  # - @edge-case: Boundary conditions
  # - @security: Security-related tests
  # - @xss: Cross-site scripting prevention
  # - @sql-injection: SQL injection prevention
  # - @integration: Tests involving multiple systems
  # - @performance: Performance/speed tests
  # - @accessibility: Accessibility compliance
  # - @wip: Work in progress (skip in CI)
  # - @smoke: Quick smoke tests
  #
  # Running specific tags:
  #   behave --tags=@critical
  #   behave --tags=@happy-path,@critical
  #   behave --tags=@validation --tags=~@wip  # validation but not wip
  #
  # Step Definitions:
  # - Implement steps in features/steps/{{FEATURE_NAME}}_steps.py
  # - Use shared steps from features/steps/common_steps.py
  # - Keep steps reusable across multiple scenarios
  #
  # Best Practices:
  # - Write scenarios before implementing features (BDD/TDD)
  # - Keep scenarios focused on behavior, not implementation
  # - Use business language, not technical jargon
  # - Make scenarios readable by non-developers
  # - Avoid detailed UI interactions in scenarios
  # - Use Background for common setup
  # - Use Scenario Outline for data-driven tests
  # - Tag scenarios for selective execution
