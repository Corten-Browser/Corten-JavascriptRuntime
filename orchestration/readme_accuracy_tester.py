#!/usr/bin/env python3
"""
README Accuracy Tester

Standalone utility to test README examples (shell + Python + doctest).

Usage:
    python orchestration/readme_accuracy_tester.py components/my-component
    python orchestration/readme_accuracy_tester.py components/my-component --json
    python orchestration/readme_accuracy_tester.py components/my-component --verbose
"""

import sys
import argparse
import subprocess
import doctest
from pathlib import Path
from typing import Dict, List
import json

from markdown_parser import MarkdownParser


class Colors:
    """ANSI color codes."""
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    BOLD = '\033[1m'
    RESET = '\033[0m'


class ReadmeAccuracyTester:
    """Test README examples for accuracy."""

    def __init__(self, component_path: Path, use_colors: bool = True, verbose: bool = False):
        self.component_path = Path(component_path)
        self.readme_path = self.component_path / "README.md"
        self.project_root = self.component_path.parent.parent
        self.use_colors = use_colors
        self.verbose = verbose

    def c(self, color: str, text: str) -> str:
        """Apply color if colors enabled."""
        if self.use_colors:
            return f"{color}{text}{Colors.RESET}"
        return text

    def run_all_tests(self) -> Dict:
        """Run all README tests and return results."""
        if not self.readme_path.exists():
            return {
                'error': f'README.md not found at {self.readme_path}',
                'overall_pass': False
            }

        results = {
            'component': self.component_path.name,
            'readme': str(self.readme_path),
            'shell_commands': self.test_shell_commands(),
            'python_examples': self.test_python_examples(),
            'doctest': self.test_doctest(),
            'overall_pass': True
        }

        # Determine overall pass/fail
        for category in ['shell_commands', 'python_examples', 'doctest']:
            if results[category]['failed'] > 0:
                results['overall_pass'] = False

        return results

    def test_shell_commands(self) -> Dict:
        """Test shell commands."""
        result = {
            'total': 0,
            'passed': 0,
            'failed': 0,
            'failures': []
        }

        try:
            parser = MarkdownParser(self.readme_path)
            shell_blocks = parser.extract_code_blocks(
                languages=['bash', 'shell', 'sh', 'console']
            )

            result['total'] = len(shell_blocks)

            for block in shell_blocks:
                if block.skip:
                    continue

                # Extract individual commands
                commands = [line.strip() for line in block.content.split('\n') if line.strip()]

                for cmd in commands:
                    # Skip comments
                    if cmd.startswith('#'):
                        continue

                    try:
                        proc_result = subprocess.run(
                            cmd,
                            shell=True,
                            cwd=self.project_root,
                            capture_output=True,
                            text=True,
                            timeout=10
                        )

                        if proc_result.returncode != 0:
                            result['failed'] += 1
                            result['failures'].append({
                                'line': block.line_number,
                                'command': cmd,
                                'error': proc_result.stderr[:300]
                            })
                        else:
                            result['passed'] += 1

                    except subprocess.TimeoutExpired:
                        result['failed'] += 1
                        result['failures'].append({
                            'line': block.line_number,
                            'command': cmd,
                            'error': 'Timeout (>10s)'
                        })
                    except Exception as e:
                        result['failed'] += 1
                        result['failures'].append({
                            'line': block.line_number,
                            'command': cmd,
                            'error': str(e)
                        })

        except Exception as e:
            result['error'] = str(e)

        return result

    def test_python_examples(self) -> Dict:
        """Test Python examples."""
        result = {
            'total': 0,
            'passed': 0,
            'failed': 0,
            'failures': []
        }

        try:
            parser = MarkdownParser(self.readme_path)
            python_blocks = parser.extract_code_blocks(
                languages=['python', 'py']
            )

            result['total'] = len(python_blocks)

            for block in python_blocks:
                if block.skip:
                    continue

                try:
                    # Create isolated namespace
                    namespace = {
                        '__name__': '__main__',
                        '__file__': str(self.readme_path),
                        '__builtins__': __builtins__
                    }

                    # Add project to path temporarily
                    original_path = sys.path.copy()
                    sys.path.insert(0, str(self.project_root))

                    try:
                        # Execute code example
                        exec(block.content, namespace)
                        result['passed'] += 1

                    finally:
                        # Restore path
                        sys.path = original_path

                except ImportError as e:
                    result['failed'] += 1
                    result['failures'].append({
                        'line': block.line_number,
                        'code': block.content[:100],
                        'error_type': 'ImportError',
                        'error': str(e)
                    })
                except TypeError as e:
                    result['failed'] += 1
                    result['failures'].append({
                        'line': block.line_number,
                        'code': block.content[:100],
                        'error_type': 'TypeError',
                        'error': str(e),
                        'hint': 'Check API signature matches documentation'
                    })
                except AssertionError as e:
                    result['failed'] += 1
                    result['failures'].append({
                        'line': block.line_number,
                        'code': block.content[:100],
                        'error_type': 'AssertionError',
                        'error': str(e)
                    })
                except Exception as e:
                    result['failed'] += 1
                    result['failures'].append({
                        'line': block.line_number,
                        'code': block.content[:100],
                        'error_type': type(e).__name__,
                        'error': str(e)
                    })

        except Exception as e:
            result['error'] = str(e)

        return result

    def test_doctest(self) -> Dict:
        """Test doctest examples."""
        result = {
            'total': 0,
            'passed': 0,
            'failed': 0,
            'failures': []
        }

        try:
            # Check if README contains doctest format
            with open(self.readme_path, 'r') as f:
                content = f.read()

            if '>>>' not in content:
                # No doctest examples
                return result

            # Run doctest on README
            doctest_result = doctest.testfile(
                str(self.readme_path),
                module_relative=False,
                verbose=False,
                optionflags=doctest.ELLIPSIS | doctest.NORMALIZE_WHITESPACE
            )

            result['total'] = doctest_result.attempted
            result['passed'] = doctest_result.attempted - doctest_result.failed
            result['failed'] = doctest_result.failed

            if doctest_result.failed > 0:
                result['failures'].append({
                    'error': f'{doctest_result.failed}/{doctest_result.attempted} doctest examples failed',
                    'hint': f'Run: python -m doctest {self.readme_path} -v'
                })

        except Exception as e:
            result['error'] = str(e)

        return result

    def print_results(self, results: Dict):
        """Print results with colors."""
        print()
        print(self.c(Colors.BOLD, f"README Accuracy Test: {results['component']}"))
        print(self.c(Colors.BLUE, f"README: {results['readme']}"))
        print()

        # Overall result
        if results.get('error'):
            print(self.c(Colors.RED, f"❌ ERROR: {results['error']}"))
            return

        if results['overall_pass']:
            print(self.c(Colors.GREEN, "✅ PASS: All README examples work"))
        else:
            print(self.c(Colors.RED, "❌ FAIL: Some README examples failed"))

        print()

        # Shell commands
        self._print_category_results(
            "Shell Commands",
            results['shell_commands']
        )

        # Python examples
        self._print_category_results(
            "Python Examples",
            results['python_examples']
        )

        # Doctest
        self._print_category_results(
            "Doctest",
            results['doctest']
        )

        print()

    def _print_category_results(self, category: str, result: Dict):
        """Print results for one category."""
        total = result.get('total', 0)
        passed = result.get('passed', 0)
        failed = result.get('failed', 0)

        if total == 0:
            print(f"{self.c(Colors.BLUE, f'{category}:')} No examples found")
            return

        if failed == 0:
            print(f"{self.c(Colors.GREEN, f'✓ {category}:')} {passed}/{total} passed")
        else:
            print(f"{self.c(Colors.RED, f'✗ {category}:')} {passed}/{total} passed, {failed} failed")

        # Print failures if verbose or if there are failures
        if (self.verbose or failed > 0) and result.get('failures'):
            for failure in result['failures'][:10]:  # Limit to first 10
                self._print_failure(failure)

            if len(result['failures']) > 10:
                remaining = len(result['failures']) - 10
                print(f"  ... and {remaining} more failure(s)")

        print()

    def _print_failure(self, failure: Dict):
        """Print a single failure."""
        if 'line' in failure:
            print(f"  Line {failure['line']}:")

        if 'command' in failure:
            print(f"    Command: {failure['command']}")
            print(f"    Error: {failure['error']}")
        elif 'code' in failure:
            print(f"    Code: {failure['code']}...")
            print(f"    Error ({failure.get('error_type', 'Unknown')}): {failure['error']}")
            if 'hint' in failure:
                print(f"    Hint: {failure['hint']}")
        else:
            print(f"    Error: {failure.get('error', 'Unknown error')}")
            if 'hint' in failure:
                print(f"    Hint: {failure['hint']}")


def main():
    parser = argparse.ArgumentParser(
        description="Test README examples for accuracy"
    )
    parser.add_argument(
        'component_path',
        type=Path,
        help="Path to component directory"
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help="Output results as JSON"
    )
    parser.add_argument(
        '--no-color',
        action='store_true',
        help="Disable colored output"
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help="Show all test details, including passing tests"
    )

    args = parser.parse_args()

    if not args.component_path.exists():
        print(f"Error: Component path not found: {args.component_path}")
        sys.exit(1)

    tester = ReadmeAccuracyTester(
        args.component_path,
        use_colors=not args.no_color,
        verbose=args.verbose
    )

    results = tester.run_all_tests()

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        tester.print_results(results)

    sys.exit(0 if results.get('overall_pass', False) else 1)


if __name__ == '__main__':
    main()
