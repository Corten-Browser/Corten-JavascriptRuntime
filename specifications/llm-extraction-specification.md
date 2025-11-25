# LLM Extraction Specification

**Version:** 1.0.0
**Status:** Draft
**Created:** 2025-11-25
**Purpose:** Define comprehensive requirements for extracting ALL specification requirements into actionable, verifiable tasks

---

## 1. Executive Summary

This specification defines how the orchestration system extracts requirements from specification documents and converts them into actionable, verifiable tasks. The extraction system must:

1. **Capture ALL requirements** - Not just features matching patterns, but every requirement statement
2. **Fully disambiguate terms** - Expand acronyms, resolve external references, clarify ambiguous language
3. **Prevent bypass** - Make requirements specific enough that mock/substitute implementations are detectable violations
4. **Maintain traceability** - Link every task to source file, line number, and verbatim text
5. **Distinguish internal from external** - Clearly identify when requirements reference external standards, test suites, or specifications

---

## 2. Problem Statement

### 2.1 Historical Failure Pattern

The orchestration system experienced a critical failure where:

1. A specification required "85% pass rate on WPT CSS tests"
2. The extraction system created a vague task: "WPT Test Harness Integration"
3. The implementation created internal "WPT-style" tests (not actual WPT)
4. Internal tests achieved 100% pass rate
5. Actual WPT compliance: 0% (never measured)
6. Project declared "complete" despite violating specification

### 2.2 Root Causes

| Gap | Description |
|-----|-------------|
| Pattern-based extraction | Only captured features matching regex patterns |
| Acronym ambiguity | "WPT" not expanded to authoritative source URL |
| No external/internal distinction | System couldn't differentiate real WPT from internal tests |
| Missing acceptance thresholds | Pass rate requirements not attached to tasks |
| Verification conflation | "Tests pass" equated with "requirement met" |

### 2.3 Design Goals

This specification ensures the extraction system:

- Extracts **100% of specification requirements** (not just pattern-matched features)
- **Expands all terms** with authoritative definitions and URLs
- **Attaches verification methods** that cannot be bypassed with internal substitutes
- **Tracks source location** for every extracted requirement
- **Distinguishes requirement types** (feature, constraint, compliance, performance, etc.)

---

## 3. Requirement Types

The extraction system MUST recognize and handle these requirement types:

### 3.1 Feature Requirements

**Definition:** Functionality that must be implemented.

**Recognition patterns:**
- "Implement X"
- "Create Y"
- "Build Z"
- "The system shall provide..."
- Checklist items: `- [ ] Feature name`
- Headers: `## Feature: Name`

**Example:**
```
Spec: "Implement CSS selector parsing for class, ID, and attribute selectors"

Extracted:
  type: feature
  name: "CSS Selector Parsing"
  scope: ["class selectors", "ID selectors", "attribute selectors"]
```

### 3.2 Constraint Requirements

**Definition:** Limitations or boundaries on implementation.

**Recognition patterns:**
- "Must not exceed..."
- "Limited to..."
- "Maximum of..."
- "Constrained by..."

**Example:**
```
Spec: "Memory usage must not exceed 100MB for documents under 10,000 nodes"

Extracted:
  type: constraint
  metric: memory_usage
  threshold: 100MB
  condition: "documents under 10,000 nodes"
```

### 3.3 Compliance Requirements

**Definition:** Conformance to external standards, specifications, or test suites.

**Recognition patterns:**
- "X% pass rate on [external suite]"
- "Compliant with [standard]"
- "Conforming to [specification]"
- "Compatible with [external system]"
- References to: WPT, POSIX, RFC, W3C, ECMA, ISO, IEEE

**Example:**
```
Spec: "85% pass rate on WPT CSS tests"

Extracted:
  type: compliance
  external_reference:
    name: "Web Platform Tests"
    acronym: "WPT"
    url: "https://github.com/web-platform-tests/wpt"
    type: "conformance_test_suite"
  threshold:
    metric: "pass_rate"
    value: 0.85
    operator: ">="
  test_paths:
    - css/CSS2/
    - css/css-cascade/
    - css/css-color/
```

### 3.4 Performance Requirements

**Definition:** Speed, throughput, latency, or efficiency targets.

**Recognition patterns:**
- "Must complete in under X ms"
- "Throughput of at least X/second"
- "Latency below X"
- "Performance target: X"

**Example:**
```
Spec: "Style computation must complete in under 16ms for 60fps rendering"

Extracted:
  type: performance
  operation: "style_computation"
  threshold:
    metric: "latency"
    value: 16
    unit: "ms"
  rationale: "60fps rendering"
```

### 3.5 Quality Requirements

**Definition:** Code quality, test coverage, or maintainability targets.

**Recognition patterns:**
- "Test coverage of X%"
- "Code review required"
- "Documentation for all public APIs"

**Example:**
```
Spec: "Minimum 80% test coverage for all modules"

Extracted:
  type: quality
  metric: "test_coverage"
  threshold: 0.80
  scope: "all modules"
```

### 3.6 Integration Requirements

**Definition:** How components must work together or with external systems.

**Recognition patterns:**
- "Must integrate with..."
- "Compatible with..."
- "Interoperable with..."

**Example:**
```
Spec: "Must integrate with the DOM module via the StyleResolver trait"

Extracted:
  type: integration
  target: "DOM module"
  interface: "StyleResolver trait"
  direction: "outbound"
```

---

## 4. Extraction Process

### 4.1 Phase 1: Specification Discovery

**Input:** Project root directory
**Output:** List of specification files with metadata

**Process:**
1. Search canonical locations:
   - `specifications/` (recursive, all files)
   - `specs/` (recursive, all files)
   - `docs/*-spec*.md`, `docs/*_spec*.md` (pattern match)
   - `*.specification.md`, `*.spec.yaml` (root level)

2. For each discovered file, record:
   ```yaml
   spec_file:
     path: "specifications/css-engine-specification.md"
     format: "markdown"  # or "yaml"
     size_bytes: 45230
     last_modified: "2025-11-20T14:30:00Z"
     hash: "sha256:abc123..."
   ```

3. Validate file accessibility and format

### 4.2 Phase 2: Exhaustive Requirement Identification

**Input:** Specification file content
**Output:** Raw requirement extractions with source locations

**CRITICAL:** This phase must extract ALL requirements, not just pattern-matched features.

**Process:**

#### 4.2.1 Structural Extraction

Extract requirements from document structure:

```python
STRUCTURAL_PATTERNS = [
    # Headers with requirement keywords
    r'^#{1,6}\s+(Feature|Component|Module|Requirement|Constraint):\s*(.+)$',

    # Checklist items
    r'^[-*]\s*\[[ x]\]\s*(.+)$',

    # Numbered requirements
    r'^\d+\.\s+(The system (shall|must|will)|Implement|Create|Build)\s+(.+)$',

    # Definition lists
    r'^([A-Z][a-zA-Z\s]+):\s*(.+)$',
]
```

#### 4.2.2 Semantic Extraction

Extract requirements from semantic indicators:

```python
SEMANTIC_PATTERNS = [
    # MUST/SHALL/REQUIRED statements (RFC 2119 keywords)
    r'(MUST|SHALL|REQUIRED|SHOULD|RECOMMENDED)\s+(.+?)(?:\.|$)',

    # Percentage/threshold requirements
    r'(\d+(?:\.\d+)?%)\s+(pass rate|coverage|compliance|accuracy)',

    # External standard references
    r'(compliant with|conforming to|compatible with|pass rate on)\s+([A-Z][A-Za-z0-9\s]+)',

    # Performance targets
    r'(under|below|within|at least|maximum of)\s+(\d+(?:\.\d+)?)\s*(ms|MB|GB|seconds?|minutes?)',
]
```

#### 4.2.3 Context Preservation

For each extraction, preserve:

```yaml
extraction:
  verbatim: "85% pass rate on WPT CSS tests"
  source:
    file: "specifications/css-engine-specification.md"
    line: 26
    section: "Target Implementation Goals"
    phase: "Phase 5"  # if within phase block
  context:
    preceding_text: "The implementation should achieve:"
    following_text: "This ensures compatibility with modern browsers."
```

#### 4.2.4 Exhaustive Scanning

**CRITICAL REQUIREMENT:** The extraction MUST NOT rely solely on patterns.

After pattern-based extraction, perform exhaustive scan:

1. **Sentence-by-sentence analysis:** Parse every sentence in the specification
2. **Requirement language detection:** Flag sentences containing requirement indicators:
   - Modal verbs: must, shall, will, should, may
   - Action verbs: implement, create, build, provide, support, handle
   - Threshold language: at least, minimum, maximum, under, below, within
   - Compliance language: compliant, conforming, compatible, passing
   - Percentage/numeric targets: X%, N units, ratios

3. **Human review queue:** Sentences with requirement indicators but no pattern match go to review queue

### 4.3 Phase 3: Term Resolution and Disambiguation

**Input:** Raw requirement extractions
**Output:** Fully disambiguated requirements

**CRITICAL:** Every acronym, external reference, and ambiguous term MUST be resolved.

#### 4.3.1 Acronym Expansion

**Process:**
1. Identify all acronyms (2+ consecutive capitals or known acronym patterns)
2. Look up in authoritative sources:
   - Specification glossary (if present)
   - Domain-specific acronym database
   - Web search for authoritative definition

3. Expand with full context:

```yaml
# Before
term: "WPT"

# After
term:
  acronym: "WPT"
  full_name: "Web Platform Tests"
  definition: "A cross-browser test suite for web platform standards"
  authoritative_source: "https://web-platform-tests.org/"
  repository_url: "https://github.com/web-platform-tests/wpt"
  type: "external_conformance_suite"
```

#### 4.3.2 External Reference Resolution

**Process:**
1. Identify references to external standards, specifications, or test suites
2. Resolve to authoritative URLs and versions
3. Capture specific sections/paths if mentioned

```yaml
# Before
reference: "CSS2 specification"

# After
reference:
  name: "Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification"
  short_name: "CSS 2.1"
  organization: "W3C"
  url: "https://www.w3.org/TR/CSS21/"
  type: "technical_specification"
  version: "2.1"
  status: "W3C Recommendation"
```

#### 4.3.3 Ambiguity Detection and Resolution

**Process:**
1. Flag ambiguous terms:
   - Terms with multiple meanings in context
   - Relative terms without baselines ("fast", "efficient", "scalable")
   - Scope ambiguities ("all", "some", "most")

2. Resolution strategies:
   - Check specification glossary
   - Check related requirements for context
   - Flag for human review if unresolvable

```yaml
# Ambiguous
term: "fast parsing"
ambiguity: "No quantitative definition of 'fast'"
resolution_required: true

# Resolved (from context)
term: "fast parsing"
resolved_definition: "Parsing completes in under 100ms for files up to 1MB"
resolution_source: "specifications/css-engine-specification.md:142"
```

### 4.4 Phase 4: Requirement Enrichment

**Input:** Disambiguated requirements
**Output:** Enriched requirements with verification methods

#### 4.4.1 Verification Method Assignment

For each requirement, determine how it will be verified:

```yaml
requirement:
  id: REQ-COMPLIANCE-001

  verification:
    method: "external_test_suite"

    # For external compliance requirements
    external_suite:
      name: "Web Platform Tests"
      url: "https://github.com/web-platform-tests/wpt"
      test_paths:
        - css/CSS2/
        - css/css-cascade/
        - css/css-color/
      harness: "wpt-harness"

    # How to measure
    measurement:
      metric: "pass_rate"
      numerator: "tests_passing"
      denominator: "total_tests_in_paths"
      threshold: 0.85
      operator: ">="

    # Evidence required
    evidence_required:
      - "WPT harness execution log"
      - "Test results JSON with per-test pass/fail"
      - "Timestamp of test execution"
      - "WPT repository commit hash used"
```

#### 4.4.2 Anti-Bypass Clauses

**CRITICAL:** For compliance and external requirements, add explicit anti-bypass clauses.

```yaml
requirement:
  id: REQ-COMPLIANCE-001

  not_satisfied_by:
    - description: "Internal tests mimicking WPT naming conventions"
      rationale: "Must run actual WPT suite, not approximations"

    - description: "Subset of WPT tests"
      rationale: "Must run all tests in specified paths"

    - description: "Mocked or stubbed test harness"
      rationale: "Must execute tests against real engine output"

    - description: "Self-reported pass rates without execution evidence"
      rationale: "Must provide execution logs and results JSON"

    - description: "Tests run against different/older WPT version"
      rationale: "Must document WPT commit hash and justify if not latest"
```

#### 4.4.3 Acceptance Criteria Generation

Generate explicit, measurable acceptance criteria:

```yaml
requirement:
  id: REQ-COMPLIANCE-001

  acceptance_criteria:
    - criterion: "WPT repository cloned"
      measurable: true
      measurement: "Directory exists with valid .git"

    - criterion: "WPT harness configured for engine"
      measurable: true
      measurement: "wpt-config.yml exists with engine paths"

    - criterion: "All specified test paths executed"
      measurable: true
      measurement: "Test results JSON includes all 9 paths"

    - criterion: "Pass rate >= 85%"
      measurable: true
      measurement: "passing_tests / total_tests >= 0.85"

    - criterion: "Results reproducible"
      measurable: true
      measurement: "Re-run produces same results within 1% variance"
```

### 4.5 Phase 5: Task Generation

**Input:** Enriched requirements
**Output:** Task queue entries

#### 4.5.1 Requirement-to-Task Mapping

Each requirement generates one or more tasks:

```yaml
# Requirement
requirement:
  id: REQ-COMPLIANCE-001
  name: "WPT CSS Compliance"
  type: compliance
  threshold: 0.85

# Generated Tasks
tasks:
  - id: TASK-REQ-COMPLIANCE-001-SETUP
    name: "Set up WPT test infrastructure"
    requirement_id: REQ-COMPLIANCE-001
    phase: "setup"
    acceptance_criteria:
      - "WPT repository cloned"
      - "Harness configured"

  - id: TASK-REQ-COMPLIANCE-001-EXECUTE
    name: "Execute WPT CSS test suite"
    requirement_id: REQ-COMPLIANCE-001
    phase: "execution"
    depends_on: [TASK-REQ-COMPLIANCE-001-SETUP]
    acceptance_criteria:
      - "All test paths executed"
      - "Results JSON generated"

  - id: TASK-REQ-COMPLIANCE-001-VERIFY
    name: "Verify WPT pass rate >= 85%"
    requirement_id: REQ-COMPLIANCE-001
    phase: "verification"
    depends_on: [TASK-REQ-COMPLIANCE-001-EXECUTE]
    acceptance_criteria:
      - "Pass rate >= 85%"
      - "Evidence documented"
```

#### 4.5.2 Task Schema

Complete task schema with all required fields:

```yaml
task:
  # Identity
  id: "TASK-REQ-001-001"
  name: "Human-readable task name"
  description: "Detailed description of what must be done"

  # Traceability
  requirement_id: "REQ-001"
  source:
    file: "specifications/spec.md"
    line: 42
    verbatim: "Original requirement text"

  # Classification
  type: "feature|constraint|compliance|performance|quality|integration"
  phase: "setup|implementation|testing|verification"

  # Dependencies
  depends_on: ["TASK-REQ-001-000"]
  blocks: ["TASK-REQ-002-001"]

  # External references (if applicable)
  external_reference:
    name: "Full name of external standard/suite"
    acronym: "ACRONYM"
    url: "https://authoritative-url.com"
    type: "conformance_suite|specification|standard"
    test_paths: ["path1/", "path2/"]  # if test suite

  # Verification
  verification:
    method: "unit_test|integration_test|external_suite|manual_review"
    threshold:
      metric: "pass_rate|coverage|latency|memory"
      value: 0.85
      operator: ">=|<=|==|<|>"
      unit: "percent|ms|MB|count"
    evidence_required:
      - "Description of required evidence"

  # Anti-bypass (for external requirements)
  not_satisfied_by:
    - description: "What does NOT satisfy this requirement"
      rationale: "Why this approach is invalid"

  # Acceptance criteria
  acceptance_criteria:
    - criterion: "Specific, measurable criterion"
      measurable: true
      measurement: "How to measure"

  # Status tracking
  status: "pending|incomplete|in_progress|completed|blocked"
  started_at: null
  completed_at: null
  verification_result:
    verified: false
    evidence: []
    pass_rate: null
```

### 4.6 Phase 6: Validation and Quality Assurance

**Input:** Generated task queue
**Output:** Validated task queue ready for execution

#### 4.6.1 Completeness Validation

Verify all requirements are captured:

```python
def validate_completeness(spec_file, task_queue):
    # Re-scan spec for requirement indicators
    requirement_sentences = extract_requirement_sentences(spec_file)

    # Check each sentence maps to a task
    unmapped = []
    for sentence in requirement_sentences:
        if not any(task.source.verbatim in sentence for task in task_queue):
            unmapped.append(sentence)

    if unmapped:
        raise ExtractionIncomplete(
            f"Found {len(unmapped)} requirements without corresponding tasks",
            unmapped_requirements=unmapped
        )
```

#### 4.6.2 Disambiguation Validation

Verify no ambiguous terms remain:

```python
def validate_disambiguation(task_queue):
    issues = []

    for task in task_queue:
        # Check for unresolved acronyms
        acronyms = find_acronyms(task.description)
        for acronym in acronyms:
            if not task.has_expanded_term(acronym):
                issues.append(f"Task {task.id}: Unresolved acronym '{acronym}'")

        # Check external references have URLs
        if task.type == "compliance" and not task.external_reference.url:
            issues.append(f"Task {task.id}: Compliance task missing external URL")

        # Check thresholds have units
        if task.verification.threshold and not task.verification.threshold.unit:
            issues.append(f"Task {task.id}: Threshold missing unit")

    if issues:
        raise DisambiguationIncomplete(issues)
```

#### 4.6.3 Anti-Bypass Validation

Verify compliance tasks have anti-bypass clauses:

```python
def validate_anti_bypass(task_queue):
    issues = []

    for task in task_queue:
        if task.type == "compliance":
            if not task.not_satisfied_by:
                issues.append(
                    f"Task {task.id}: Compliance task missing anti-bypass clauses"
                )
            if not task.external_reference.url:
                issues.append(
                    f"Task {task.id}: Compliance task missing authoritative URL"
                )
            if not task.verification.evidence_required:
                issues.append(
                    f"Task {task.id}: Compliance task missing evidence requirements"
                )

    if issues:
        raise AntiBypassIncomplete(issues)
```

---

## 5. Output Schemas

### 5.1 Requirement Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Requirement",
  "type": "object",
  "required": ["id", "name", "type", "source", "verification"],
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^REQ-[A-Z]+-\\d{3}$"
    },
    "name": {
      "type": "string",
      "minLength": 5
    },
    "description": {
      "type": "string"
    },
    "type": {
      "type": "string",
      "enum": ["feature", "constraint", "compliance", "performance", "quality", "integration"]
    },
    "source": {
      "type": "object",
      "required": ["file", "line", "verbatim"],
      "properties": {
        "file": { "type": "string" },
        "line": { "type": "integer", "minimum": 1 },
        "verbatim": { "type": "string" },
        "section": { "type": "string" },
        "phase": { "type": "string" }
      }
    },
    "external_reference": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "acronym": { "type": "string" },
        "url": { "type": "string", "format": "uri" },
        "type": {
          "type": "string",
          "enum": ["conformance_suite", "specification", "standard", "library", "tool"]
        },
        "test_paths": {
          "type": "array",
          "items": { "type": "string" }
        },
        "version": { "type": "string" }
      }
    },
    "verification": {
      "type": "object",
      "required": ["method"],
      "properties": {
        "method": {
          "type": "string",
          "enum": ["unit_test", "integration_test", "external_suite", "manual_review", "benchmark", "static_analysis"]
        },
        "threshold": {
          "type": "object",
          "properties": {
            "metric": { "type": "string" },
            "value": { "type": "number" },
            "operator": {
              "type": "string",
              "enum": [">=", "<=", "==", ">", "<"]
            },
            "unit": { "type": "string" }
          }
        },
        "evidence_required": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "not_satisfied_by": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["description", "rationale"],
        "properties": {
          "description": { "type": "string" },
          "rationale": { "type": "string" }
        }
      }
    },
    "acceptance_criteria": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["criterion", "measurable"],
        "properties": {
          "criterion": { "type": "string" },
          "measurable": { "type": "boolean" },
          "measurement": { "type": "string" }
        }
      }
    }
  }
}
```

### 5.2 Task Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Task",
  "type": "object",
  "required": ["id", "name", "requirement_id", "source", "type", "status", "acceptance_criteria"],
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^TASK-REQ-[A-Z]+-\\d{3}-\\d{3}$"
    },
    "name": {
      "type": "string",
      "minLength": 5
    },
    "description": {
      "type": "string"
    },
    "requirement_id": {
      "type": "string",
      "pattern": "^REQ-[A-Z]+-\\d{3}$"
    },
    "source": {
      "type": "object",
      "required": ["file", "line", "verbatim"],
      "properties": {
        "file": { "type": "string" },
        "line": { "type": "integer", "minimum": 1 },
        "verbatim": { "type": "string" }
      }
    },
    "type": {
      "type": "string",
      "enum": ["feature", "constraint", "compliance", "performance", "quality", "integration"]
    },
    "phase": {
      "type": "string",
      "enum": ["setup", "implementation", "testing", "verification"]
    },
    "depends_on": {
      "type": "array",
      "items": { "type": "string" }
    },
    "external_reference": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "acronym": { "type": "string" },
        "url": { "type": "string", "format": "uri" },
        "type": { "type": "string" },
        "test_paths": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "verification": {
      "type": "object",
      "required": ["method"],
      "properties": {
        "method": { "type": "string" },
        "threshold": {
          "type": "object",
          "properties": {
            "metric": { "type": "string" },
            "value": { "type": "number" },
            "operator": { "type": "string" },
            "unit": { "type": "string" }
          }
        },
        "evidence_required": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "not_satisfied_by": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "description": { "type": "string" },
          "rationale": { "type": "string" }
        }
      }
    },
    "acceptance_criteria": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["criterion", "measurable"],
        "properties": {
          "criterion": { "type": "string" },
          "measurable": { "type": "boolean" },
          "measurement": { "type": "string" }
        }
      }
    },
    "status": {
      "type": "string",
      "enum": ["pending", "incomplete", "in_progress", "completed", "blocked"]
    },
    "started_at": {
      "type": ["string", "null"],
      "format": "date-time"
    },
    "completed_at": {
      "type": ["string", "null"],
      "format": "date-time"
    },
    "verification_result": {
      "type": ["object", "null"],
      "properties": {
        "verified": { "type": "boolean" },
        "evidence": {
          "type": "array",
          "items": { "type": "string" }
        },
        "pass_rate": { "type": ["number", "null"] },
        "timestamp": { "type": "string", "format": "date-time" }
      }
    }
  }
}
```

### 5.3 Extraction Metadata Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ExtractionMetadata",
  "type": "object",
  "required": ["extraction_timestamp", "spec_files", "requirements_extracted", "tasks_generated"],
  "properties": {
    "extraction_timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "extraction_version": {
      "type": "string",
      "description": "Version of extraction specification used"
    },
    "spec_files": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["path", "hash", "requirements_count"],
        "properties": {
          "path": { "type": "string" },
          "hash": { "type": "string" },
          "requirements_count": { "type": "integer" },
          "last_processed": { "type": "string", "format": "date-time" }
        }
      }
    },
    "requirements_extracted": {
      "type": "integer"
    },
    "tasks_generated": {
      "type": "integer"
    },
    "disambiguation_stats": {
      "type": "object",
      "properties": {
        "acronyms_expanded": { "type": "integer" },
        "external_references_resolved": { "type": "integer" },
        "ambiguities_flagged": { "type": "integer" },
        "ambiguities_resolved": { "type": "integer" }
      }
    },
    "validation_results": {
      "type": "object",
      "properties": {
        "completeness_check": { "type": "boolean" },
        "disambiguation_check": { "type": "boolean" },
        "anti_bypass_check": { "type": "boolean" }
      }
    },
    "human_review_queue": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "source": {
            "type": "object",
            "properties": {
              "file": { "type": "string" },
              "line": { "type": "integer" }
            }
          },
          "text": { "type": "string" },
          "reason": { "type": "string" }
        }
      }
    }
  }
}
```

---

## 6. Acronym and Term Database

### 6.1 Built-in Acronym Database

The extraction system MUST include a built-in database of common technical acronyms:

```yaml
acronyms:
  # Web Platform
  WPT:
    full_name: "Web Platform Tests"
    url: "https://github.com/web-platform-tests/wpt"
    type: "conformance_suite"

  W3C:
    full_name: "World Wide Web Consortium"
    url: "https://www.w3.org/"
    type: "standards_organization"

  CSS:
    full_name: "Cascading Style Sheets"
    url: "https://www.w3.org/Style/CSS/"
    type: "specification"

  DOM:
    full_name: "Document Object Model"
    url: "https://dom.spec.whatwg.org/"
    type: "specification"

  HTML:
    full_name: "HyperText Markup Language"
    url: "https://html.spec.whatwg.org/"
    type: "specification"

  # Programming Standards
  POSIX:
    full_name: "Portable Operating System Interface"
    url: "https://pubs.opengroup.org/onlinepubs/9699919799/"
    type: "specification"

  RFC:
    full_name: "Request for Comments"
    url: "https://www.rfc-editor.org/"
    type: "specification_series"

  # JavaScript
  ECMAScript:
    full_name: "ECMAScript Language Specification"
    url: "https://tc39.es/ecma262/"
    type: "specification"
    aliases: ["ES", "ES6", "ES2015", "ES2020", "ES2021", "ES2022", "ES2023"]

  TC39:
    full_name: "Technical Committee 39"
    url: "https://tc39.es/"
    type: "standards_body"

  # Testing
  TDD:
    full_name: "Test-Driven Development"
    type: "methodology"

  BDD:
    full_name: "Behavior-Driven Development"
    type: "methodology"

  CI:
    full_name: "Continuous Integration"
    type: "practice"

  CD:
    full_name: "Continuous Deployment"
    type: "practice"

  # Security
  OWASP:
    full_name: "Open Web Application Security Project"
    url: "https://owasp.org/"
    type: "organization"

  CVE:
    full_name: "Common Vulnerabilities and Exposures"
    url: "https://cve.mitre.org/"
    type: "database"

  # Performance
  FPS:
    full_name: "Frames Per Second"
    type: "metric"

  TTI:
    full_name: "Time to Interactive"
    type: "metric"

  LCP:
    full_name: "Largest Contentful Paint"
    type: "metric"

  # Data Formats
  JSON:
    full_name: "JavaScript Object Notation"
    url: "https://www.json.org/"
    type: "format"

  YAML:
    full_name: "YAML Ain't Markup Language"
    url: "https://yaml.org/"
    type: "format"

  XML:
    full_name: "Extensible Markup Language"
    url: "https://www.w3.org/XML/"
    type: "format"
```

### 6.2 Domain-Specific Extensions

Projects MAY provide domain-specific acronym files:

```yaml
# specifications/glossary.yaml
acronyms:
  CSSOM:
    full_name: "CSS Object Model"
    url: "https://drafts.csswg.org/cssom/"
    type: "specification"

  BFC:
    full_name: "Block Formatting Context"
    type: "css_concept"

  IFC:
    full_name: "Inline Formatting Context"
    type: "css_concept"
```

### 6.3 Acronym Resolution Algorithm

```python
def resolve_acronym(acronym: str, context: str) -> ResolvedTerm:
    # 1. Check project-specific glossary
    if project_glossary and acronym in project_glossary:
        return project_glossary[acronym]

    # 2. Check specification glossary
    if spec_glossary and acronym in spec_glossary:
        return spec_glossary[acronym]

    # 3. Check built-in database
    if acronym in BUILTIN_ACRONYMS:
        return BUILTIN_ACRONYMS[acronym]

    # 4. Check aliases
    for entry in BUILTIN_ACRONYMS.values():
        if acronym in entry.get('aliases', []):
            return entry

    # 5. Web search (with caching)
    result = web_search_acronym(acronym, context)
    if result.confidence > 0.8:
        return result

    # 6. Flag for human review
    return UnresolvedTerm(
        acronym=acronym,
        context=context,
        needs_human_review=True
    )
```

---

## 7. External Reference Types

### 7.1 Conformance Test Suites

External test suites that implementations must pass:

| Suite | Domain | URL | Common Paths |
|-------|--------|-----|--------------|
| WPT | Web Platform | https://github.com/web-platform-tests/wpt | css/, dom/, html/, fetch/ |
| Test262 | ECMAScript | https://github.com/tc39/test262 | test/language/, test/built-ins/ |
| POSIX Test Suite | POSIX | Various | - |
| Unicode Conformance | Unicode | https://unicode.org/Public/UCD/ | - |

### 7.2 Technical Specifications

Normative documents defining behavior:

| Spec | Organization | URL Pattern |
|------|--------------|-------------|
| CSS | W3C | https://www.w3.org/TR/CSS{n}/ |
| HTML | WHATWG | https://html.spec.whatwg.org/ |
| DOM | WHATWG | https://dom.spec.whatwg.org/ |
| ECMAScript | TC39 | https://tc39.es/ecma262/ |
| HTTP | IETF | https://httpwg.org/specs/ |

### 7.3 Reference Resolution

```yaml
# Example: Resolving "CSS2 specification"
input: "CSS2 specification"

resolved:
  name: "Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification"
  short_name: "CSS 2.1"
  url: "https://www.w3.org/TR/CSS21/"
  organization: "W3C"
  type: "technical_specification"
  status: "W3C Recommendation"

# Example: Resolving "WPT CSS tests"
input: "WPT CSS tests"

resolved:
  name: "Web Platform Tests - CSS Test Suite"
  url: "https://github.com/web-platform-tests/wpt"
  type: "conformance_suite"
  test_paths:
    - css/CSS2/
    - css/css-cascade/
    - css/css-color/
    - css/css-display/
    - css/css-flexbox/
    - css/css-grid/
    - css/css-pseudo/
    - css/css-variables/
    - css/selectors/
```

---

## 8. Verification Methods

### 8.1 Internal Verification

For feature and quality requirements:

```yaml
verification:
  method: "unit_test"
  location: "tests/unit/"
  framework: "pytest"
  coverage_required: 0.80
```

### 8.2 External Suite Verification

For compliance requirements:

```yaml
verification:
  method: "external_suite"

  suite:
    name: "Web Platform Tests"
    repository: "https://github.com/web-platform-tests/wpt"
    commit: "latest"  # or specific commit hash

  execution:
    harness: "wpt run"
    adapter: "path/to/engine/adapter"
    test_paths:
      - css/CSS2/
      - css/css-cascade/

  measurement:
    metric: "pass_rate"
    threshold: 0.85

  evidence:
    - "wpt-results.json"
    - "execution-log.txt"
    - "wpt-commit-hash.txt"
```

### 8.3 Benchmark Verification

For performance requirements:

```yaml
verification:
  method: "benchmark"

  benchmark:
    tool: "criterion"  # or "hyperfine", "pytest-benchmark"
    suite: "benches/style_computation.rs"

  measurement:
    metric: "latency_p99"
    threshold: 16
    unit: "ms"

  conditions:
    document_size: "10000 nodes"
    iterations: 100
    warmup: 10
```

### 8.4 Manual Review Verification

For subjective or complex requirements:

```yaml
verification:
  method: "manual_review"

  reviewer_requirements:
    - "Domain expertise in CSS layout"
    - "Familiarity with browser rendering"

  checklist:
    - "Visual output matches reference browser"
    - "Edge cases handled correctly"
    - "No visual artifacts"

  evidence:
    - "Screenshots comparison"
    - "Reviewer sign-off"
```

---

## 9. Anti-Bypass Patterns

### 9.1 Common Bypass Attempts

The extraction system must generate anti-bypass clauses that prevent:

| Bypass Pattern | How Detected | Anti-Bypass Clause |
|----------------|--------------|-------------------|
| Internal test suite mimicking external | Test files don't match external repo structure | "Must run actual [suite] from [url], not internal approximations" |
| Subset of external tests | Test count doesn't match expected | "Must run all tests in specified paths, not cherry-picked subset" |
| Mocked/stubbed harness | No real execution evidence | "Must provide execution logs showing real test runs" |
| Self-reported metrics | No independent verification | "Metrics must come from [suite] harness, not self-reported" |
| Outdated external suite | Version/commit mismatch | "Must use [suite] version/commit within 6 months of latest" |
| Modified external tests | Test hashes don't match | "Must run unmodified tests from official repository" |

### 9.2 Anti-Bypass Clause Generation

```python
def generate_anti_bypass_clauses(requirement):
    clauses = []

    if requirement.type == "compliance":
        ref = requirement.external_reference

        # Clause 1: Must use actual external suite
        clauses.append({
            "description": f"Internal tests mimicking {ref.name} naming conventions",
            "rationale": f"Must run actual {ref.name} from {ref.url}, not approximations"
        })

        # Clause 2: Must run all specified tests
        if ref.test_paths:
            clauses.append({
                "description": f"Subset of {ref.name} tests",
                "rationale": f"Must run all tests in paths: {', '.join(ref.test_paths)}"
            })

        # Clause 3: Must have execution evidence
        clauses.append({
            "description": "Self-reported pass rates without execution evidence",
            "rationale": f"Must provide {ref.name} harness execution logs and results"
        })

        # Clause 4: Must use current version
        clauses.append({
            "description": f"Outdated {ref.name} version",
            "rationale": "Must document version/commit used; should be within 6 months of latest"
        })

    return clauses
```

---

## 10. Integration Points

### 10.1 Integration with Task Queue

```python
# After extraction, merge with existing queue
def merge_extracted_tasks(extracted_tasks, existing_queue):
    for task in extracted_tasks:
        existing = existing_queue.find_by_requirement(task.requirement_id)

        if existing:
            # Update existing task with enriched data
            existing.update(
                external_reference=task.external_reference,
                verification=task.verification,
                not_satisfied_by=task.not_satisfied_by,
                acceptance_criteria=task.acceptance_criteria
            )
        else:
            # Add new task
            existing_queue.add(task)

    return existing_queue
```

### 10.2 Integration with Phase Gates

```python
# Phase gate checks verification requirements
def phase_gate_check(task, evidence):
    if task.type == "compliance":
        # Must have external suite results
        if not evidence.has_external_suite_results(task.external_reference):
            raise GateFailure(
                f"Task {task.id} requires {task.external_reference.name} results"
            )

        # Check anti-bypass clauses
        for clause in task.not_satisfied_by:
            if detect_bypass(evidence, clause):
                raise GateFailure(
                    f"Task {task.id} appears satisfied by forbidden method: {clause.description}"
                )

        # Check threshold
        actual = evidence.get_pass_rate(task.external_reference)
        required = task.verification.threshold.value

        if actual < required:
            raise GateFailure(
                f"Task {task.id}: Pass rate {actual:.1%} < required {required:.1%}"
            )
```

### 10.3 Integration with Verification Agent

```python
# Verification agent uses extracted requirements
def verify_requirement(requirement, implementation):
    # Get verification method
    method = requirement.verification.method

    if method == "external_suite":
        # Run external suite
        result = run_external_suite(
            suite_url=requirement.external_reference.url,
            test_paths=requirement.external_reference.test_paths,
            adapter=find_adapter(implementation)
        )

        # Check result against threshold
        return VerificationResult(
            requirement_id=requirement.id,
            passed=result.pass_rate >= requirement.verification.threshold.value,
            actual_value=result.pass_rate,
            expected_value=requirement.verification.threshold.value,
            evidence=result.to_evidence()
        )
```

---

## 11. Error Handling

### 11.1 Extraction Errors

| Error Type | Cause | Recovery |
|------------|-------|----------|
| `SpecificationNotFound` | No spec files discovered | Guide user to create spec or specify location |
| `AcronymUnresolved` | Acronym not in database, web search failed | Add to human review queue |
| `ExternalReferenceUnresolved` | Cannot find authoritative URL | Add to human review queue |
| `AmbiguousRequirement` | Multiple interpretations possible | Add to human review queue with options |
| `ThresholdMissing` | Compliance requirement without numeric target | Prompt for clarification |

### 11.2 Human Review Queue

Unresolved items go to human review:

```yaml
human_review_queue:
  - id: "REVIEW-001"
    type: "unresolved_acronym"
    item:
      acronym: "CSSWG"
      context: "Per CSSWG resolution, floats must..."
      source:
        file: "specifications/css-engine-specification.md"
        line: 156
    suggested_resolution:
      full_name: "CSS Working Group"
      url: "https://www.w3.org/Style/CSS/"
    confidence: 0.7

  - id: "REVIEW-002"
    type: "ambiguous_requirement"
    item:
      text: "Support for modern CSS features"
      source:
        file: "specifications/css-engine-specification.md"
        line: 42
    issue: "No specific list of 'modern CSS features'"
    suggested_action: "Request clarification or define scope"
```

---

## 12. Reporting

### 12.1 Extraction Report

After extraction, generate comprehensive report:

```markdown
# Extraction Report

**Timestamp:** 2025-11-25T10:30:00Z
**Specification Files:** 3
**Extraction Version:** 1.0.0

## Summary

| Metric | Count |
|--------|-------|
| Requirements Extracted | 47 |
| Tasks Generated | 62 |
| Acronyms Expanded | 23 |
| External References Resolved | 5 |
| Items for Human Review | 3 |

## Requirements by Type

| Type | Count | % |
|------|-------|---|
| Feature | 32 | 68% |
| Compliance | 5 | 11% |
| Performance | 4 | 8% |
| Quality | 3 | 6% |
| Constraint | 2 | 4% |
| Integration | 1 | 2% |

## External Compliance Requirements

| ID | Name | Suite | Target |
|----|------|-------|--------|
| REQ-COMPLIANCE-001 | WPT CSS Compliance | Web Platform Tests | 85% |
| REQ-COMPLIANCE-002 | CSS2.1 Conformance | W3C CSS2.1 Test Suite | 95% |

## Human Review Required

| ID | Type | Issue |
|----|------|-------|
| REVIEW-001 | Unresolved Acronym | "CSSWG" at line 156 |
| REVIEW-002 | Ambiguous Requirement | "modern CSS features" at line 42 |
| REVIEW-003 | Missing Threshold | "good performance" at line 89 |

## Validation Results

- [x] Completeness Check: PASSED
- [x] Disambiguation Check: PASSED (3 items in review queue)
- [x] Anti-Bypass Check: PASSED
```

### 12.2 Traceability Matrix

Generate requirement-to-task traceability:

```markdown
# Traceability Matrix

| Requirement | Source | Tasks | Status |
|-------------|--------|-------|--------|
| REQ-FEAT-001 | spec.md:26 | TASK-001, TASK-002 | Pending |
| REQ-COMPLIANCE-001 | spec.md:30 | TASK-003, TASK-004, TASK-005 | Pending |
| REQ-PERF-001 | spec.md:45 | TASK-006 | Pending |
```

---

## 13. Implementation Requirements

### 13.1 Extraction Engine

The extraction engine MUST:

1. **Support multiple spec formats:** Markdown, YAML, JSON
2. **Perform exhaustive extraction:** Not limited to patterns
3. **Resolve all acronyms:** Using multi-source lookup
4. **Generate anti-bypass clauses:** For all compliance requirements
5. **Maintain traceability:** Source file and line for every requirement
6. **Validate completeness:** Ensure no requirements missed
7. **Generate human review queue:** For unresolvable items

### 13.2 CLI Interface

```bash
# Extract requirements from all specs
python orchestration/cli/extract_requirements.py

# Extract from specific file
python orchestration/cli/extract_requirements.py --spec specifications/my-spec.md

# Generate report only (no queue modification)
python orchestration/cli/extract_requirements.py --report-only

# Validate existing extraction
python orchestration/cli/extract_requirements.py --validate

# Process human review queue
python orchestration/cli/extract_requirements.py --review
```

### 13.3 Output Files

| File | Purpose |
|------|---------|
| `orchestration/data/state/requirements.json` | All extracted requirements |
| `orchestration/data/state/queue_state.json` | Task queue (enhanced) |
| `orchestration/data/state/extraction_metadata.json` | Extraction metadata |
| `orchestration/data/reports/extraction_report.md` | Human-readable report |
| `orchestration/data/state/human_review_queue.json` | Items needing review |

---

## 14. Acceptance Criteria for This Specification

This specification is complete when:

1. **Exhaustive Extraction:** System extracts 100% of requirements from spec documents
2. **Full Disambiguation:** All acronyms expanded with authoritative sources
3. **External Reference Resolution:** All external standards/suites have URLs
4. **Anti-Bypass Clauses:** All compliance requirements have anti-bypass clauses
5. **Source Traceability:** Every task links to source file:line
6. **Verification Methods:** Every requirement has defined verification method
7. **Validation Passing:** Completeness, disambiguation, and anti-bypass checks pass
8. **Human Review Queue:** Unresolvable items properly queued

---

## 15. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-25 | Initial specification |

---

## Appendix A: Example Extraction

### Input Specification

```markdown
# CSS Engine Specification

## Target Implementation Goals

- 85% pass rate on WPT CSS tests
- Sub-16ms style computation for 60fps rendering
- Full CSS2.1 selector support

## Phase 1: Core CSS2.1

### Features
- [ ] Implement CSS parser using cssparser crate
- [ ] Create selector matching engine
- [ ] Build cascade resolver

### Tests
- 60% CSS2 WPT pass rate
```

### Extracted Requirements

```yaml
requirements:
  - id: REQ-COMPLIANCE-001
    name: "WPT CSS Compliance"
    type: compliance
    source:
      file: "specifications/css-engine-specification.md"
      line: 5
      verbatim: "85% pass rate on WPT CSS tests"
    external_reference:
      name: "Web Platform Tests"
      acronym: "WPT"
      url: "https://github.com/web-platform-tests/wpt"
      type: "conformance_suite"
      test_paths:
        - css/CSS2/
        - css/css-cascade/
        - css/css-color/
        - css/css-display/
        - css/css-flexbox/
        - css/css-grid/
        - css/css-pseudo/
        - css/css-variables/
        - css/selectors/
    verification:
      method: "external_suite"
      threshold:
        metric: "pass_rate"
        value: 0.85
        operator: ">="
      evidence_required:
        - "WPT harness execution log"
        - "Test results JSON"
        - "WPT commit hash"
    not_satisfied_by:
      - description: "Internal 'WPT-style' tests"
        rationale: "Must run actual WPT suite from repository"
      - description: "Subset of WPT tests"
        rationale: "Must run all tests in specified paths"
      - description: "Self-reported pass rates"
        rationale: "Must provide harness execution evidence"
    acceptance_criteria:
      - criterion: "WPT repository cloned and configured"
        measurable: true
        measurement: "Directory exists with wpt-config.yml"
      - criterion: "All 9 test paths executed"
        measurable: true
        measurement: "Results JSON includes all paths"
      - criterion: "Pass rate >= 85%"
        measurable: true
        measurement: "passing_tests / total_tests >= 0.85"

  - id: REQ-PERF-001
    name: "Style Computation Performance"
    type: performance
    source:
      file: "specifications/css-engine-specification.md"
      line: 6
      verbatim: "Sub-16ms style computation for 60fps rendering"
    verification:
      method: "benchmark"
      threshold:
        metric: "latency_p99"
        value: 16
        operator: "<"
        unit: "ms"
      evidence_required:
        - "Benchmark results"
        - "Test document specification"
    acceptance_criteria:
      - criterion: "Style computation < 16ms at p99"
        measurable: true
        measurement: "criterion benchmark p99 < 16ms"

  - id: REQ-FEAT-001
    name: "CSS2.1 Selector Support"
    type: feature
    source:
      file: "specifications/css-engine-specification.md"
      line: 7
      verbatim: "Full CSS2.1 selector support"
    external_reference:
      name: "CSS 2.1 Specification - Selectors"
      url: "https://www.w3.org/TR/CSS21/selector.html"
      type: "specification"
    verification:
      method: "unit_test"
      threshold:
        metric: "test_pass_rate"
        value: 1.0
        operator: "=="
    acceptance_criteria:
      - criterion: "All CSS2.1 selector types implemented"
        measurable: true
        measurement: "Tests cover type, class, ID, attribute, pseudo selectors"
```

### Generated Tasks

```yaml
tasks:
  - id: TASK-REQ-COMPLIANCE-001-001
    name: "Set up WPT test infrastructure"
    requirement_id: REQ-COMPLIANCE-001
    phase: setup
    acceptance_criteria:
      - criterion: "WPT repository cloned"
        measurable: true
      - criterion: "Harness configured for engine"
        measurable: true

  - id: TASK-REQ-COMPLIANCE-001-002
    name: "Create WPT engine adapter"
    requirement_id: REQ-COMPLIANCE-001
    phase: implementation
    depends_on: [TASK-REQ-COMPLIANCE-001-001]

  - id: TASK-REQ-COMPLIANCE-001-003
    name: "Execute WPT CSS test suite"
    requirement_id: REQ-COMPLIANCE-001
    phase: testing
    depends_on: [TASK-REQ-COMPLIANCE-001-002]
    external_reference:
      name: "Web Platform Tests"
      url: "https://github.com/web-platform-tests/wpt"
      test_paths: [css/CSS2/, css/css-cascade/, ...]

  - id: TASK-REQ-COMPLIANCE-001-004
    name: "Verify WPT pass rate >= 85%"
    requirement_id: REQ-COMPLIANCE-001
    phase: verification
    depends_on: [TASK-REQ-COMPLIANCE-001-003]
    verification:
      method: external_suite
      threshold:
        metric: pass_rate
        value: 0.85
        operator: ">="
    not_satisfied_by:
      - description: "Internal 'WPT-style' tests"
        rationale: "Must run actual WPT suite"
```

---

*End of Specification*
