#!/usr/bin/env python3
"""
Test suite for test_quality_checker.py

Tests all detection patterns:
- Over-mocking detection (@patch('src.'), excessive patches)
- Integration test verification
- Skipped test detection
- Mock usage analysis
- Statistics calculation
"""

import pytest
import tempfile
import shutil
from pathlib import Path
from test_quality_checker import (
    TestQualityChecker,
    OverMockingDetector,
    IntegrationTestVerifier,
    SkippedTestDetector,
    Severity,
    Issue
)


class TestOverMockingDetector:
    """Test suite for OverMockingDetector class"""

    @pytest.fixture
    def temp_component(self):
        """Create temporary component directory"""
        temp_dir = Path(tempfile.mkdtemp())
        component_dir = temp_dir / 'test_component'
        component_dir.mkdir()

        # Create test directories
        (component_dir / 'tests' / 'unit').mkdir(parents=True)
        (component_dir / 'tests' / 'integration').mkdir(parents=True)
        (component_dir / 'src').mkdir()

        yield component_dir

        # Cleanup
        shutil.rmtree(temp_dir)

    def test_detects_patch_overmocking(self, temp_component):
        """Verify detection of @patch('src.') patterns"""
        # Create test file with over-mocking
        test_file = temp_component / 'tests' / 'unit' / 'test_example.py'
        test_file.write_text('''
from unittest.mock import patch

@patch('src.cli.MusicAnalyzer')
def test_analyze_command(mock_analyzer):
    execute_analyze_command(args)
    mock_analyzer.assert_called_once()
''')

        detector = OverMockingDetector(temp_component, 'test_component')
        issues = detector.analyze()

        # Should detect critical over-mocking
        assert len(issues) > 0
        critical_issues = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical_issues) == 1
        assert critical_issues[0].pattern == 'own_source_code'
        assert 'src.cli.MusicAnalyzer' in critical_issues[0].code

    def test_detects_component_overmocking(self, temp_component):
        """Verify detection of @patch('components.') patterns"""
        test_file = temp_component / 'tests' / 'unit' / 'test_example.py'
        test_file.write_text('''
from unittest.mock import patch

@patch('components.test_component.validators.EmailValidator')
def test_validation(mock_validator):
    pass
''')

        detector = OverMockingDetector(temp_component, 'test_component')
        issues = detector.analyze()

        critical_issues = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical_issues) == 1
        assert 'components.test_component' in critical_issues[0].code

    def test_ignores_external_service_mocks(self, temp_component):
        """Verify external service mocks are NOT flagged"""
        test_file = temp_component / 'tests' / 'unit' / 'test_example.py'
        test_file.write_text('''
from unittest.mock import patch

@patch('requests.get')
@patch('time.sleep')
@patch('boto3.client')
def test_external_calls(mock_boto, mock_sleep, mock_requests):
    # These are all external - should NOT be flagged
    pass
''')

        detector = OverMockingDetector(temp_component, 'test_component')
        issues = detector.analyze()

        # Should have NO issues (all mocks are external)
        critical_issues = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical_issues) == 0

    def test_detects_excessive_patches(self, temp_component):
        """Verify detection of > 3 @patch decorators"""
        test_file = temp_component / 'tests' / 'unit' / 'test_example.py'
        test_file.write_text('''
from unittest.mock import patch

@patch('requests.get')
@patch('time.sleep')
@patch('os.environ')
@patch('subprocess.run')
def test_many_mocks(mock1, mock2, mock3, mock4):
    # 4 patches - should trigger warning
    pass
''')

        detector = OverMockingDetector(temp_component, 'test_component')
        issues = detector.analyze()

        # Should have WARNING for excessive patches
        warnings = [i for i in issues if i.severity == Severity.WARNING]
        excessive_warnings = [w for w in warnings if w.pattern == 'excessive_patches']
        assert len(excessive_warnings) == 1
        assert '4' in excessive_warnings[0].message

    def test_detects_mock_spec_overmocking(self, temp_component):
        """Verify detection of Mock(spec=OwnClass) patterns"""
        test_file = temp_component / 'tests' / 'unit' / 'test_example.py'
        test_file.write_text('''
from unittest.mock import Mock
from src.models import User

def test_user_processing():
    mock_user = Mock(spec=User)
    # Should trigger warning
    pass
''')

        detector = OverMockingDetector(temp_component, 'test_component')
        issues = detector.analyze()

        # Should have WARNING for mocking own class
        warnings = [i for i in issues if i.severity == Severity.WARNING]
        spec_warnings = [w for w in warnings if w.pattern == 'mock_spec_own_class']
        assert len(spec_warnings) == 1
        assert 'User' in spec_warnings[0].message

    def test_handles_syntax_errors_gracefully(self, temp_component):
        """Verify syntax errors are reported as warnings"""
        test_file = temp_component / 'tests' / 'unit' / 'test_broken.py'
        test_file.write_text('''
def test_broken(
    # Syntax error - missing closing paren
''')

        detector = OverMockingDetector(temp_component, 'test_component')
        issues = detector.analyze()

        # Should report syntax error as warning
        warnings = [i for i in issues if i.category == 'syntax_error']
        assert len(warnings) == 1

    def test_no_issues_with_good_tests(self, temp_component):
        """Verify good tests pass without issues"""
        test_file = temp_component / 'tests' / 'unit' / 'test_good.py'
        test_file.write_text('''
from unittest.mock import patch

@patch('requests.get')  # External - OK
def test_api_call(mock_get):
    # Use real domain objects
    analyzer = RealAnalyzer()  # Not mocked
    result = analyzer.analyze()
    assert result is not None
''')

        detector = OverMockingDetector(temp_component, 'test_component')
        issues = detector.analyze()

        # Should have NO critical issues
        critical_issues = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical_issues) == 0


class TestIntegrationTestVerifier:
    """Test suite for IntegrationTestVerifier class"""

    @pytest.fixture
    def temp_component(self):
        """Create temporary component directory"""
        temp_dir = Path(tempfile.mkdtemp())
        component_dir = temp_dir / 'test_component'
        component_dir.mkdir()
        (component_dir / 'tests').mkdir()

        yield component_dir

        shutil.rmtree(temp_dir)

    def test_detects_missing_integration_directory(self, temp_component):
        """Verify detection of missing tests/integration/ directory"""
        verifier = IntegrationTestVerifier(temp_component)
        issues, result = verifier.verify()

        # Should report CRITICAL issue
        assert len(issues) > 0
        critical = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical) == 1
        assert 'missing' in critical[0].message.lower()
        assert result['directory_exists'] is False

    def test_detects_empty_integration_directory(self, temp_component):
        """Verify detection of empty integration directory"""
        # Create directory but no test files
        (temp_component / 'tests' / 'integration').mkdir(parents=True)

        verifier = IntegrationTestVerifier(temp_component)
        issues, result = verifier.verify()

        # Should report CRITICAL issue for no tests
        critical = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical) == 1
        assert 'no integration test functions' in critical[0].message.lower()
        assert result['directory_exists'] is True
        assert result['test_count'] == 0

    def test_passes_with_integration_tests(self, temp_component):
        """Verify component with integration tests passes"""
        int_dir = temp_component / 'tests' / 'integration'
        int_dir.mkdir(parents=True)

        # Create test file with actual tests
        test_file = int_dir / 'test_integration.py'
        test_file.write_text('''
def test_full_workflow():
    # Real integration test
    result = run_full_workflow()
    assert result.success

def test_database_integration():
    # Real database test
    assert True
''')

        verifier = IntegrationTestVerifier(temp_component)
        issues, result = verifier.verify()

        # Should have NO critical issues
        critical = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical) == 0
        assert result['directory_exists'] is True
        assert result['tests_present'] is True
        assert result['test_count'] == 2
        assert result['file_count'] == 1

    def test_detects_overmocking_in_integration_tests(self, temp_component):
        """Verify over-mocking in integration tests is CRITICAL"""
        int_dir = temp_component / 'tests' / 'integration'
        int_dir.mkdir(parents=True)

        # Create integration test with over-mocking
        test_file = int_dir / 'test_integration.py'
        test_file.write_text('''
from unittest.mock import patch

@patch('src.database.UserRepository')  # Mocking own code in integration test!
def test_user_flow(mock_repo):
    # This defeats the purpose of integration testing
    pass
''')

        verifier = IntegrationTestVerifier(temp_component)
        issues, result = verifier.verify()

        # Should detect over-mocking as CRITICAL
        critical = [i for i in issues if i.severity == Severity.CRITICAL and i.category == 'overmocking']
        assert len(critical) == 1
        assert 'integration test mocking own code' in critical[0].message.lower()
        assert result['own_code_mocking'] is True


class TestSkippedTestDetector:
    """Test suite for SkippedTestDetector class"""

    @pytest.fixture
    def temp_component(self):
        """Create temporary component directory"""
        temp_dir = Path(tempfile.mkdtemp())
        component_dir = temp_dir / 'test_component'
        component_dir.mkdir()
        (component_dir / 'tests' / 'unit').mkdir(parents=True)
        (component_dir / 'tests' / 'integration').mkdir(parents=True)

        yield component_dir

        shutil.rmtree(temp_dir)

    def test_detects_skip_decorator_in_unit_tests(self, temp_component):
        """Verify detection of @pytest.mark.skip in unit tests"""
        test_file = temp_component / 'tests' / 'unit' / 'test_example.py'
        test_file.write_text('''
import pytest

@pytest.mark.skip(reason="TODO: implement")
def test_feature():
    pass
''')

        detector = SkippedTestDetector(temp_component)
        issues = detector.detect()

        # Should be WARNING for unit tests
        warnings = [i for i in issues if i.severity == Severity.WARNING]
        assert len(warnings) == 1
        assert 'skipped test in unit tests' in warnings[0].message.lower()

    def test_detects_skip_call_in_integration_tests(self, temp_component):
        """Verify detection of pytest.skip() in integration tests"""
        test_file = temp_component / 'tests' / 'integration' / 'test_integration.py'
        test_file.write_text('''
import pytest

def test_full_workflow():
    pytest.skip("needs all dependencies")
    # Test code never runs
''')

        detector = SkippedTestDetector(temp_component)
        issues = detector.detect()

        # Should be CRITICAL for integration tests
        critical = [i for i in issues if i.severity == Severity.CRITICAL]
        assert len(critical) == 1
        assert 'integration' in critical[0].message.lower()

    def test_no_issues_with_no_skips(self, temp_component):
        """Verify no issues when tests aren't skipped"""
        test_file = temp_component / 'tests' / 'unit' / 'test_good.py'
        test_file.write_text('''
def test_feature():
    assert True  # Not skipped

def test_another_feature():
    assert True  # Not skipped
''')

        detector = SkippedTestDetector(temp_component)
        issues = detector.detect()

        # Should have NO issues
        assert len(issues) == 0


class TestTestQualityChecker:
    """Test suite for TestQualityChecker coordinator class"""

    @pytest.fixture
    def temp_component_good(self):
        """Create component with good test quality"""
        temp_dir = Path(tempfile.mkdtemp())
        component_dir = temp_dir / 'good_component'
        component_dir.mkdir()

        # Create directories
        (component_dir / 'tests' / 'unit').mkdir(parents=True)
        (component_dir / 'tests' / 'integration').mkdir(parents=True)
        (component_dir / 'src').mkdir()

        # Create good unit test
        unit_test = component_dir / 'tests' / 'unit' / 'test_unit.py'
        unit_test.write_text('''
from unittest.mock import patch

@patch('requests.get')  # External - OK
def test_api_call(mock_get):
    # Use real domain logic
    result = real_function()
    assert result
''')

        # Create good integration test
        int_test = component_dir / 'tests' / 'integration' / 'test_integration.py'
        int_test.write_text('''
def test_full_workflow():
    # Real integration test
    result = run_workflow()
    assert result.success
''')

        yield component_dir

        shutil.rmtree(temp_dir)

    @pytest.fixture
    def temp_component_bad(self):
        """Create component with bad test quality"""
        temp_dir = Path(tempfile.mkdtemp())
        component_dir = temp_dir / 'bad_component'
        component_dir.mkdir()

        # Create directories
        (component_dir / 'tests' / 'unit').mkdir(parents=True)
        (component_dir / 'src').mkdir()

        # Create bad test with over-mocking
        bad_test = component_dir / 'tests' / 'unit' / 'test_bad.py'
        bad_test.write_text('''
from unittest.mock import patch

@patch('src.cli.MusicAnalyzer')  # Over-mocking!
def test_analyze(mock_analyzer):
    mock_analyzer.return_value.analyze.return_value = {}
    result = execute_command()
    assert result
''')

        # No integration tests directory - critical issue

        yield component_dir

        shutil.rmtree(temp_dir)

    def test_passes_with_good_tests(self, temp_component_good):
        """Verify good tests pass all checks"""
        checker = TestQualityChecker(temp_component_good)
        report = checker.check()

        assert report.status == "PASSED"
        assert report.exit_code == 0
        assert report.has_critical_issues() is False

    def test_fails_with_bad_tests(self, temp_component_bad):
        """Verify bad tests fail checks"""
        checker = TestQualityChecker(temp_component_bad)
        report = checker.check()

        assert report.status == "FAILED"
        assert report.exit_code == 1
        assert report.has_critical_issues() is True
        assert len(report.blocking_issues) > 0

    def test_summary_statistics_accurate(self, temp_component_bad):
        """Verify summary statistics are calculated correctly"""
        checker = TestQualityChecker(temp_component_bad)
        report = checker.check()

        # Should have at least 2 critical issues:
        # 1. Over-mocking
        # 2. Missing integration tests
        assert report.summary['critical_count'] >= 2
        assert report.summary['total_issues'] >= 2

    def test_mock_analysis_calculation(self, temp_component_bad):
        """Verify mock usage analysis is correct"""
        checker = TestQualityChecker(temp_component_bad)
        report = checker.check()

        # Should detect own code mocks
        assert report.mock_analysis['own_code_mocks'] >= 1
        assert report.mock_analysis['mock_ratio'] > 0

    def test_human_report_generation(self, temp_component_good):
        """Verify human-readable report is generated"""
        checker = TestQualityChecker(temp_component_good)
        report = checker.check()

        human_report = checker.generate_human_report(report)

        # Should contain key sections
        assert "Test Quality Check" in human_report
        assert "Over-Mocking Detection" in human_report
        assert "Integration Test Verification" in human_report
        assert "Summary" in human_report

    def test_json_report_generation(self, temp_component_good):
        """Verify JSON report is valid"""
        checker = TestQualityChecker(temp_component_good)
        report = checker.check()

        json_report = checker.generate_json_report(report)

        # Should be valid JSON
        import json
        data = json.loads(json_report)

        # Should have required fields
        assert 'component' in data
        assert 'status' in data
        assert 'summary' in data
        assert 'exit_code' in data

    def test_detects_user_scenario_exactly(self, temp_component_bad):
        """Verify detection of exact user scenario from bug report"""
        # This is the exact pattern from the user's bug report
        checker = TestQualityChecker(temp_component_bad)
        report = checker.check()

        # Should detect over-mocking as CRITICAL
        overmocking_issues = [i for i in report.issues
                             if i.pattern == 'own_source_code'
                             and i.severity == Severity.CRITICAL]
        assert len(overmocking_issues) >= 1

        # Check the specific pattern is detected
        found_music_analyzer = any(
            'src.cli.MusicAnalyzer' in (issue.code or '')
            for issue in overmocking_issues
        )
        assert found_music_analyzer


class TestEndToEnd:
    """End-to-end integration tests"""

    def test_full_workflow_bad_component(self):
        """Test full workflow with component that should fail"""
        temp_dir = Path(tempfile.mkdtemp())
        try:
            component = temp_dir / 'failing_component'
            component.mkdir()
            (component / 'tests' / 'unit').mkdir(parents=True)
            (component / 'src').mkdir()

            # Create test with multiple issues
            test_file = component / 'tests' / 'unit' / 'test_cli.py'
            test_file.write_text('''
from unittest.mock import patch, Mock

@patch('src.cli.MusicAnalyzer')
@patch('src.cli.ProgressReporter')
def test_analyze_command_executes_with_resume(mock_progress, mock_analyzer):
    mock_analyzer_instance = Mock()
    mock_analyzer.return_value = mock_analyzer_instance

    execute_analyze_command(args)

    mock_analyzer.assert_called_once()
''')

            # No integration tests - should fail

            checker = TestQualityChecker(component)
            report = checker.check()

            # Should FAIL with multiple critical issues
            assert report.status == "FAILED"
            assert report.exit_code == 1
            assert report.critical_count >= 2  # Over-mocking + no integration tests

            # Verify human report provides actionable guidance
            human_report = checker.generate_human_report(report)
            assert "CRITICAL" in human_report
            assert "Fix" in human_report or "fix" in human_report

        finally:
            shutil.rmtree(temp_dir)

    def test_full_workflow_good_component(self):
        """Test full workflow with component that should pass"""
        temp_dir = Path(tempfile.mkdtemp())
        try:
            component = temp_dir / 'passing_component'
            component.mkdir()
            (component / 'tests' / 'unit').mkdir(parents=True)
            (component / 'tests' / 'integration').mkdir(parents=True)
            (component / 'src').mkdir()

            # Create good unit test
            unit_test = component / 'tests' / 'unit' / 'test_unit.py'
            unit_test.write_text('''
from unittest.mock import patch

@patch('external_api.fetch')
def test_fetch_data(mock_fetch):
    # Mock external API only
    analyzer = RealAnalyzer()  # Use real domain object
    result = analyzer.process()
    assert result
''')

            # Create good integration test
            int_test = component / 'tests' / 'integration' / 'test_integration.py'
            int_test.write_text('''
def test_end_to_end_workflow():
    # No mocks - real integration test
    result = run_full_workflow()
    assert result.success

def test_database_integration():
    # Real database test
    with TestDB() as db:
        result = db.query()
        assert result
''')

            checker = TestQualityChecker(component)
            report = checker.check()

            # Should PASS
            assert report.status == "PASSED"
            assert report.exit_code == 0
            assert report.critical_count == 0

            # Verify report shows success
            human_report = checker.generate_human_report(report)
            assert "✅ PASSED" in human_report or "PASSED" in human_report

        finally:
            shutil.rmtree(temp_dir)


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
