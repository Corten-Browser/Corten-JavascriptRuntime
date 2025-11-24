#!/usr/bin/env python3
"""
Specification Completeness Analyzer

Analyzes specifications for ambiguities, missing requirements,
and generates clarification needs BEFORE coding begins.

Part of v0.4.0 quality enhancement system.
"""

from pathlib import Path
from typing import List, Dict, Optional, Set
from dataclasses import dataclass, field
import re
import yaml
from datetime import datetime


@dataclass
class Ambiguity:
    """An ambiguous term or requirement."""
    location: str  # Line number or section
    term: str
    context: str
    reason: str
    suggested_clarification: str
    severity: str = "warning"  # "critical", "warning", "info"


@dataclass
class MissingScenario:
    """A missing error or edge case scenario."""
    operation: str
    scenario_type: str  # "error", "edge_case", "validation"
    description: str
    suggested_handling: str


@dataclass
class MissingValidation:
    """A missing input validation requirement."""
    field: str
    context: str
    validation_type: str
    suggested_rule: str


@dataclass
class SpecificationAnalysis:
    """Complete specification analysis result."""
    has_critical_gaps: bool
    ambiguities: List[Ambiguity]
    missing_scenarios: List[MissingScenario]
    missing_validations: List[MissingValidation]
    completeness_score: float  # 0-100
    spec_source: str = "user_input"

    def generate_clarification_document(self) -> str:
        """Generate SPEC_CLARIFICATIONS.md content."""
        doc_lines = [
            "# Specification Clarifications Needed",
            "",
            f"**Analysis Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"**Source**: {self.spec_source}",
            f"**Completeness Score**: {self.completeness_score:.1f}/100",
            "",
            "## Summary",
            "",
            f"- **Critical Gaps**: {'Yes - MUST address before coding' if self.has_critical_gaps else 'No'}",
            f"- **Ambiguities Found**: {len(self.ambiguities)}",
            f"- **Missing Error Scenarios**: {len(self.missing_scenarios)}",
            f"- **Missing Validations**: {len(self.missing_validations)}",
            ""
        ]

        # Ambiguities section
        if self.ambiguities:
            doc_lines.extend([
                "## Ambiguous Terms and Requirements",
                "",
                "The following terms are vague and need specific, measurable definitions:",
                ""
            ])

            # Group by severity
            critical = [a for a in self.ambiguities if a.severity == "critical"]
            warnings = [a for a in self.ambiguities if a.severity == "warning"]
            info = [a for a in self.ambiguities if a.severity == "info"]

            if critical:
                doc_lines.extend([
                    "### Critical (MUST Fix)",
                    ""
                ])
                for amb in critical:
                    doc_lines.extend(self._format_ambiguity(amb))

            if warnings:
                doc_lines.extend([
                    "### Warnings (Should Fix)",
                    ""
                ])
                for amb in warnings:
                    doc_lines.extend(self._format_ambiguity(amb))

            if info:
                doc_lines.extend([
                    "### Informational (Consider Fixing)",
                    ""
                ])
                for amb in info:
                    doc_lines.extend(self._format_ambiguity(amb))

        # Missing scenarios section
        if self.missing_scenarios:
            doc_lines.extend([
                "## Missing Error and Edge Case Scenarios",
                "",
                "The following error scenarios need to be specified:",
                ""
            ])

            # Group by type
            errors = [s for s in self.missing_scenarios if s.scenario_type == "error"]
            edge_cases = [s for s in self.missing_scenarios if s.scenario_type == "edge_case"]
            validations = [s for s in self.missing_scenarios if s.scenario_type == "validation"]

            if errors:
                doc_lines.extend([
                    "### Error Handling",
                    ""
                ])
                for scenario in errors:
                    doc_lines.extend(self._format_scenario(scenario))

            if edge_cases:
                doc_lines.extend([
                    "### Edge Cases",
                    ""
                ])
                for scenario in edge_cases:
                    doc_lines.extend(self._format_scenario(scenario))

            if validations:
                doc_lines.extend([
                    "### Input Validation",
                    ""
                ])
                for scenario in validations:
                    doc_lines.extend(self._format_scenario(scenario))

        # Missing validations section
        if self.missing_validations:
            doc_lines.extend([
                "## Missing Input Validation Requirements",
                "",
                "The following fields need validation specifications:",
                ""
            ])

            for validation in self.missing_validations:
                doc_lines.extend(self._format_validation(validation))

        # Recommendations section
        doc_lines.extend([
            "## Recommendations",
            "",
            "To improve this specification:",
            ""
        ])

        if self.completeness_score < 50:
            doc_lines.append("1. **Start with a template**: Use an API specification template (e.g., OpenAPI) to ensure completeness")
        if len(self.ambiguities) > 5:
            doc_lines.append("1. **Replace vague terms**: Convert all subjective terms to specific, measurable requirements")
        if len(self.missing_scenarios) > 5:
            doc_lines.append("1. **Add error scenarios**: Document handling for network failures, timeouts, validation errors, and concurrent access")
        if len(self.missing_validations) > 5:
            doc_lines.append("1. **Define input constraints**: Specify format, length, range, and required/optional for all inputs")

        doc_lines.extend([
            "",
            "## Next Steps",
            "",
            "1. Address all **Critical** items before starting implementation",
            "2. Clarify **Warning** items to avoid rework during development",
            "3. Consider **Informational** items for improved specification quality",
            "4. Re-run specification analyzer after updates to verify improvements",
            ""
        ])

        return "\n".join(doc_lines)

    def _format_ambiguity(self, amb: Ambiguity) -> List[str]:
        """Format a single ambiguity for markdown."""
        return [
            f"#### {amb.term}",
            "",
            f"**Location**: {amb.location}",
            f"**Context**: {amb.context}",
            "",
            f"**Problem**: {amb.reason}",
            "",
            f"**Suggested Clarification**:",
            f"> {amb.suggested_clarification}",
            ""
        ]

    def _format_scenario(self, scenario: MissingScenario) -> List[str]:
        """Format a single missing scenario for markdown."""
        return [
            f"#### {scenario.operation}: {scenario.description}",
            "",
            f"**Suggested Handling**:",
            f"> {scenario.suggested_handling}",
            ""
        ]

    def _format_validation(self, validation: MissingValidation) -> List[str]:
        """Format a single missing validation for markdown."""
        return [
            f"#### Field: `{validation.field}`",
            "",
            f"**Context**: {validation.context}",
            f"**Missing**: {validation.validation_type}",
            "",
            f"**Suggested Rule**:",
            f"> {validation.suggested_rule}",
            ""
        ]


class SpecificationAnalyzer:
    """Analyzes specifications for completeness."""

    def __init__(self, patterns_file: Optional[Path] = None):
        # Patterns file is in config directory, not analysis directory
        config_dir = Path(__file__).parent.parent / "config"
        self.patterns_file = patterns_file or config_dir / "specification_patterns.yaml"
        self.patterns = self._load_patterns()

    def _load_patterns(self) -> dict:
        """Load ambiguity patterns from YAML."""
        if not self.patterns_file.exists():
            # Return minimal defaults if file doesn't exist
            return {
                "ambiguity_patterns": {},
                "error_scenario_categories": {},
                "validation_requirements": {},
                "operations_requiring_error_scenarios": []
            }

        with open(self.patterns_file, 'r', encoding='utf-8') as f:
            return yaml.safe_load(f)

    def analyze_specification(self, spec_text: str, spec_source: str = "user_input") -> SpecificationAnalysis:
        """
        Comprehensive specification analysis.
        Returns analysis with all detected issues.
        """
        if not spec_text or not spec_text.strip():
            return SpecificationAnalysis(
                has_critical_gaps=True,
                ambiguities=[],
                missing_scenarios=[],
                missing_validations=[],
                completeness_score=0.0,
                spec_source=spec_source
            )

        ambiguities = self.detect_ambiguous_terms(spec_text)
        missing_scenarios = self.detect_missing_error_scenarios(spec_text)
        missing_validations = self.detect_missing_validations(spec_text)

        # Has critical gaps if there are critical ambiguities (vague core requirements)
        # Missing scenarios are less critical since we can't enumerate all possibilities
        has_critical = (
            len([a for a in ambiguities if a.severity == "critical"]) > 0
        )

        completeness_score = self._calculate_completeness_score(
            ambiguities, missing_scenarios, missing_validations
        )

        return SpecificationAnalysis(
            has_critical_gaps=has_critical,
            ambiguities=ambiguities,
            missing_scenarios=missing_scenarios,
            missing_validations=missing_validations,
            completeness_score=completeness_score,
            spec_source=spec_source
        )

    def detect_ambiguous_terms(self, spec_text: str) -> List[Ambiguity]:
        """
        Find vague terms like:
        - "should handle errors appropriately"
        - "reasonable performance"
        - "user-friendly interface"
        - "scalable"
        - "fast"
        """
        ambiguities = []
        lines = spec_text.split('\n')

        ambiguity_patterns = self.patterns.get("ambiguity_patterns", {})

        for pattern_name, pattern_config in ambiguity_patterns.items():
            patterns = pattern_config.get("patterns", [])
            clarification = pattern_config.get("clarification_template", "Please specify more details")
            severity = pattern_config.get("severity", "warning")

            for pattern in patterns:
                # Case-insensitive search for pattern
                regex = re.compile(r'\b' + re.escape(pattern) + r'\b', re.IGNORECASE)

                for line_num, line in enumerate(lines, 1):
                    # Skip lines that are likely section headers
                    stripped = line.strip()
                    if (stripped.endswith(':') and len(stripped) < 40) or \
                       stripped.startswith('#'):
                        continue

                    matches = regex.finditer(line)
                    for match in matches:
                        # Extract context around the match
                        start = max(0, match.start() - 20)
                        end = min(len(line), match.end() + 20)
                        context = line[start:end].strip()
                        if start > 0:
                            context = "..." + context
                        if end < len(line):
                            context = context + "..."

                        ambiguities.append(Ambiguity(
                            location=f"Line {line_num}",
                            term=match.group(),
                            context=context,
                            reason=f"Vague term in {pattern_name.replace('_', ' ')}",
                            suggested_clarification=clarification,
                            severity=severity
                        ))

        return ambiguities

    def detect_missing_error_scenarios(self, spec_text: str) -> List[MissingScenario]:
        """
        For each operation, ensure error cases defined:
        - Network failures
        - Invalid inputs
        - Timeout scenarios
        - Concurrent access issues
        - Database failures
        """
        missing = []

        # Identify operations mentioned in spec
        operations = self._identify_operations(spec_text)

        error_categories = self.patterns.get("error_scenario_categories", {})

        for operation in operations:
            # Check which error scenarios are missing for this operation
            for category_name, category_config in error_categories.items():
                scenarios = category_config.get("scenarios", [])
                must_specify = category_config.get("must_specify", [])

                for scenario in scenarios:
                    # Check if this scenario is addressed in spec
                    if not self._is_scenario_addressed(spec_text, scenario, operation):
                        missing.append(MissingScenario(
                            operation=operation,
                            scenario_type="error",
                            description=f"{scenario}",
                            suggested_handling=f"Specify: {', '.join(must_specify)}"
                        ))

        return missing

    def detect_missing_validations(self, spec_text: str) -> List[MissingValidation]:
        """
        For each input, ensure validation specified:
        - Format requirements
        - Bounds/limits
        - Required vs optional
        - Allowed values
        """
        missing = []

        # Identify input fields mentioned in spec
        fields = self._identify_input_fields(spec_text)

        validation_requirements = self.patterns.get("validation_requirements", {})

        for field_name, field_type in fields.items():
            # Determine expected field type
            field_category = self._categorize_field(field_name, field_type)

            if field_category in validation_requirements:
                requirements = validation_requirements[field_category]
                must_specify = requirements.get("must_specify", [])

                # Handle both list of dicts and dict formats
                if isinstance(must_specify, list):
                    # List of single-key dicts
                    for item in must_specify:
                        if isinstance(item, dict):
                            for req_name, req_description in item.items():
                                # Check if this validation is specified
                                if not self._is_validation_specified(spec_text, field_name, req_name):
                                    missing.append(MissingValidation(
                                        field=field_name,
                                        context=f"Field type: {field_type}",
                                        validation_type=req_name.replace('_', ' '),
                                        suggested_rule=req_description
                                    ))
                elif isinstance(must_specify, dict):
                    # Dict format
                    for req_name, req_description in must_specify.items():
                        # Check if this validation is specified
                        if not self._is_validation_specified(spec_text, field_name, req_name):
                            missing.append(MissingValidation(
                                field=field_name,
                                context=f"Field type: {field_type}",
                                validation_type=req_name.replace('_', ' '),
                                suggested_rule=req_description
                            ))

        return missing

    def _calculate_completeness_score(self,
                                      ambiguities: List[Ambiguity],
                                      scenarios: List[MissingScenario],
                                      validations: List[MissingValidation]) -> float:
        """Calculate 0-100 completeness score."""
        # Start at 100, deduct for issues
        score = 100.0

        # Critical ambiguities cost more
        critical_ambiguities = len([a for a in ambiguities if a.severity == "critical"])
        warning_ambiguities = len([a for a in ambiguities if a.severity == "warning"])
        info_ambiguities = len([a for a in ambiguities if a.severity == "info"])

        # Critical ambiguities are severe - they indicate fundamental unclear requirements
        score -= critical_ambiguities * 20
        score -= warning_ambiguities * 8
        score -= info_ambiguities * 2

        # Missing scenarios - but cap the penalty to avoid over-penalizing
        # (Some missing scenarios are expected since we can't cover everything)
        scenario_penalty = min(len(scenarios) * 0.4, 12.0)  # Max 12 points for scenarios
        score -= scenario_penalty

        # Missing validations - also capped
        validation_penalty = min(len(validations) * 0.4, 8.0)  # Max 8 points for validations
        score -= validation_penalty

        return max(0.0, score)

    def _identify_operations(self, spec_text: str) -> List[str]:
        """Identify operations mentioned in specification."""
        operations = []
        spec_lower = spec_text.lower()

        operation_keywords = self.patterns.get("operations_requiring_error_scenarios", [])

        for keyword in operation_keywords:
            # Split multi-word keywords and check if all words appear
            words = keyword.lower().split()

            # For multi-word keywords, check if all words appear (in any order)
            if len(words) > 1:
                if all(word in spec_lower for word in words):
                    operations.append(keyword)
            else:
                # Single word - use word boundary matching
                pattern = re.compile(r'\b' + re.escape(keyword) + r'\b', re.IGNORECASE)
                if pattern.search(spec_text):
                    operations.append(keyword)

        # Also look for API endpoints
        endpoint_pattern = re.compile(r'(GET|POST|PUT|DELETE|PATCH)\s+(/[\w/\-{}]+)', re.IGNORECASE)
        for match in endpoint_pattern.finditer(spec_text):
            operations.append(f"{match.group(1)} {match.group(2)}")

        # Look for function/method definitions
        function_pattern = re.compile(r'(def|function|async)\s+(\w+)', re.IGNORECASE)
        for match in function_pattern.finditer(spec_text):
            operations.append(match.group(2))

        # Look for action verbs that indicate operations
        action_patterns = [
            r'\b(call|query|update|delete|insert|create|process|send|receive|fetch|store|retrieve)\b\s+\w+',
        ]
        for pattern_str in action_patterns:
            pattern = re.compile(pattern_str, re.IGNORECASE)
            for match in pattern.finditer(spec_text):
                operations.append(match.group(0))

        return operations

    def _is_scenario_addressed(self, spec_text: str, scenario: str, operation: str) -> bool:
        """Check if an error scenario is addressed in the specification."""
        # Look for keywords related to the scenario
        scenario_keywords = scenario.lower().split()

        # Check if spec contains discussion of this scenario
        spec_lower = spec_text.lower()

        # Check for explicit error handling sections
        has_error_section = any(section in spec_lower for section in [
            'error scenario',
            'error handling',
            'failure handling',
            'error cases',
            'exception handling'
        ])

        # More strict heuristic: require multiple keywords or specific error handling phrases
        keyword_matches = sum(1 for keyword in scenario_keywords if keyword in spec_lower)

        # HTTP status codes indicate error handling
        status_code_pattern = re.compile(r'\b(400|401|403|404|409|500|502|503|504)\b')
        has_status_codes = bool(status_code_pattern.search(spec_text))

        # Specific error handling actions
        error_handling_actions = [
            'retry',
            'fallback',
            'return error',
            'throw',
            'raise',
            'timeout',
            'unavailable',
            'invalid',
            'duplicate',
            'conflict',
            'failed',
            'failure'
        ]

        has_error_actions = any(action in spec_lower for action in error_handling_actions)

        # If there's an error scenarios section and the scenario keywords match
        if has_error_section and keyword_matches >= min(2, len(scenario_keywords)):
            return True

        # If status codes are present and scenario keywords match
        if has_status_codes and keyword_matches >= min(2, len(scenario_keywords)):
            return True

        # If explicit error handling actions mentioned with scenario keywords
        if has_error_actions and keyword_matches >= min(2, len(scenario_keywords)):
            return True

        return False

    def _identify_input_fields(self, spec_text: str) -> Dict[str, str]:
        """Identify input fields mentioned in specification."""
        fields = {}

        # Look for common field patterns
        field_patterns = [
            r'(\w+):\s*(required|optional)',  # "email: required"
            r'field[s]?:\s*(\w+)',  # "field: username"
            r'input[s]?:\s*(\w+)',  # "input: email"
            r'parameter[s]?:\s*(\w+)',  # "parameter: userId"
            r'(\w+)\s+field',  # "email field"
            r'accept\s+(?:user\s+)?(\w+)',  # "accept email", "accept user email"
            r'(\w+)\s+from\s+user',  # "email from user"
            r'-\s+(\w+):',  # "- email:"
        ]

        for pattern in field_patterns:
            matches = re.finditer(pattern, spec_text, re.IGNORECASE)
            for match in matches:
                field_name = match.group(1)
                # Skip common non-field words
                if field_name.lower() in ['user', 'fields', 'field', 'input', 'inputs', 'parameter', 'parameters']:
                    continue
                # Infer type from name
                field_type = self._infer_field_type(field_name)
                fields[field_name] = field_type

        return fields

    def _infer_field_type(self, field_name: str) -> str:
        """Infer field type from field name."""
        field_lower = field_name.lower()

        if 'email' in field_lower:
            return 'email'
        elif 'phone' in field_lower or 'tel' in field_lower:
            return 'phone'
        elif 'age' in field_lower or 'count' in field_lower or 'num' in field_lower:
            return 'numeric'
        elif 'date' in field_lower or 'time' in field_lower:
            return 'datetime'
        elif 'name' in field_lower or 'title' in field_lower or 'description' in field_lower:
            return 'string'
        elif 'status' in field_lower or 'type' in field_lower or 'role' in field_lower:
            return 'enum'
        elif 'file' in field_lower or 'upload' in field_lower or 'image' in field_lower:
            return 'file'
        elif 'list' in field_lower or 'array' in field_lower or field_lower.endswith('s'):
            return 'array'
        else:
            return 'string'

    def _categorize_field(self, field_name: str, field_type: str) -> str:
        """Categorize field to match validation requirements."""
        type_mapping = {
            'email': 'email_field',
            'phone': 'string_field',
            'numeric': 'numeric_field',
            'datetime': 'datetime_field',
            'string': 'string_field',
            'enum': 'enum_field',
            'file': 'file_upload_field',
            'array': 'array_field'
        }

        return type_mapping.get(field_type, 'string_field')

    def _is_validation_specified(self, spec_text: str, field_name: str, validation_type: str) -> bool:
        """Check if a specific validation is specified for a field."""
        # Look for field name and validation keywords nearby
        spec_lower = spec_text.lower()
        field_lower = field_name.lower()

        # Validation keywords to look for
        validation_keywords = {
            'format_validation': ['format', 'regex', 'pattern', 'rfc'],
            'max_length': ['max', 'maximum'],
            'min_length': ['min', 'minimum'],
            'min_max_values': ['min', 'max', 'range', 'between'],
            'required_optional': ['required', 'optional'],
            'allowed_values': ['enum', 'allowed', 'values', 'one of'],
            'precision': ['decimal', 'precision', 'places'],
            'timezone_handling': ['timezone', 'utc', 'tz'],
            'max_file_size': ['size', 'mb', 'gb', 'kb'],
            'allowed_mime_types': ['mime', 'type', 'jpeg', 'png', 'pdf'],
            'data_type': ['integer', 'decimal', 'float', 'string', 'number'],
            'min_max_dates': ['after', 'before', 'range'],
            'duplicates_allowed': ['unique', 'duplicate'],
            'min_items': ['min', 'minimum'],
            'max_items': ['max', 'maximum'],
            'item_type': ['type', 'string', 'integer']
        }

        keywords = validation_keywords.get(validation_type, [])

        # Check if field and validation keywords appear together
        field_pattern = re.compile(rf'\b{re.escape(field_lower)}\b')
        field_matches = list(field_pattern.finditer(spec_lower))

        if not field_matches:
            # Field not mentioned, so validation can't be specified
            return False

        for field_match in field_matches:
            # Look for validation keywords within 200 characters
            start = max(0, field_match.start() - 100)
            end = min(len(spec_lower), field_match.end() + 100)
            context = spec_lower[start:end]

            # Need to find at least one keyword
            matches = sum(1 for keyword in keywords if keyword in context)
            if matches > 0:
                return True

        return False


def main():
    """CLI interface."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: specification_analyzer.py <spec_file>")
        print("       specification_analyzer.py --stdin")
        sys.exit(1)

    if sys.argv[1] == "--stdin":
        spec_text = sys.stdin.read()
        spec_source = "stdin"
    else:
        spec_file = Path(sys.argv[1])
        if not spec_file.exists():
            print(f"Error: File not found: {spec_file}")
            sys.exit(1)
        spec_text = spec_file.read_text(encoding='utf-8')
        spec_source = str(spec_file)

    analyzer = SpecificationAnalyzer()
    analysis = analyzer.analyze_specification(spec_text, spec_source)

    print(f"Completeness Score: {analysis.completeness_score:.1f}/100")
    print(f"Critical Gaps: {'Yes' if analysis.has_critical_gaps else 'No'}")
    print(f"Ambiguities: {len(analysis.ambiguities)}")
    print(f"  - Critical: {len([a for a in analysis.ambiguities if a.severity == 'critical'])}")
    print(f"  - Warning: {len([a for a in analysis.ambiguities if a.severity == 'warning'])}")
    print(f"  - Info: {len([a for a in analysis.ambiguities if a.severity == 'info'])}")
    print(f"Missing Scenarios: {len(analysis.missing_scenarios)}")
    print(f"Missing Validations: {len(analysis.missing_validations)}")

    if analysis.has_critical_gaps or len(analysis.ambiguities) > 0 or len(analysis.missing_scenarios) > 0:
        clarification_doc = analysis.generate_clarification_document()

        if sys.argv[1] == "--stdin":
            output_file = Path("SPEC_CLARIFICATIONS.md")
        else:
            spec_file = Path(sys.argv[1])
            output_file = spec_file.parent / "SPEC_CLARIFICATIONS.md"

        output_file.write_text(clarification_doc, encoding='utf-8')
        print(f"\nGenerated: {output_file}")

    sys.exit(0 if not analysis.has_critical_gaps else 1)


if __name__ == '__main__':
    main()
