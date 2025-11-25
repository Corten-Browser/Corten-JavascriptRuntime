# Extract Features from Specifications (LLM Extraction)

**Purpose**: Use Claude's intelligence to extract ALL requirements from specifications, including features, constraints, compliance requirements, and performance targets.

**How it works**:
1. Reads `orchestration/extraction_metadata.json` to find which spec files were processed
2. Detects implementation language from spec content (CRITICAL - stops if none detected)
3. Extracts ALL requirements using LLM understanding (not regex)
4. **Categorizes by requirement type** (feature, compliance, constraint, performance, quality, integration)
5. **Resolves acronyms** using the built-in acronym database
6. **Generates anti-bypass clauses** for compliance requirements
7. **Adds source traceability** (file path, line number, verbatim text)
8. Applies feature templates for detected language
9. Merges new features into existing task queue (UPDATE not REPLACE)
10. Preserves all existing features, IDs, statuses, timestamps
11. Auto-commits results to git

---

## Instructions for Claude

You are the primary extraction system for requirements from specifications.

### Step 1: Read Extraction Metadata

Read `orchestration/extraction_metadata.json` to determine:
- Which spec files were processed
- How many features were extracted per file (automated)
- Which files need LLM extraction

If the file doesn't exist, display error:
```
❌ ERROR: extraction_metadata.json not found

Please run automated extraction first:
  python3 orchestration/auto_init.py

This will create the metadata file needed for /orch-extract-features.
```

### Step 1.5: Detect Implementation Language (CRITICAL)

**Before proceeding with feature extraction**, detect the implementation language:

**Language Detection Algorithm**:

1. **Read all specification files** and count language indicators:
   ```python
   language_indicators = {
       "rust": ["Cargo.toml", "cargo", ".rs", "crate", "use std::", "impl ", "pub fn"],
       "python": ["requirements.txt", "setup.py", ".py", "import ", "def ", "class "],
       "javascript": ["package.json", "npm", ".js", ".ts", "import { ", "export "],
       "go": ["go.mod", "go.sum", ".go", "package main", "func ", "import \""],
       "cpp": ["CMakeLists.txt", ".cpp", ".hpp", "#include", "namespace ", "class "],
   }
   ```

2. **Score each language** based on indicator frequency in spec content

3. **Check confidence level**:
   - If max_score == 0: confidence = "none"
   - If max_score < 3: confidence = "low"
   - If max_score < 10: confidence = "medium"
   - Else: confidence = "high"

4. **If confidence == "none" (NO LANGUAGE DETECTED)**:

   **STOP EXECUTION** and display this message:

   ```
   ❌ LANGUAGE DETECTION FAILED
   ══════════════════════════════════════════════════════════════

   No implementation language detected in specification files.

   Feature templates require knowing the target language to recommend
   appropriate libraries and tools.

   REQUIRED ACTION:
   Please add a language specification to one of your specification
   documents, then re-run /orch-extract-features.

   ══════════════════════════════════════════════════════════════
   RECOMMENDATION
   ══════════════════════════════════════════════════════════════

   Based on your project structure and specification content, I recommend:

   **Recommended Language**: [Analyze project domain and recommend]
     - Browser/systems components → Rust or C++
     - Web backend (performance) → Go
     - Web backend (rapid dev) → Python
     - Data science → Python
     - CLI tools (performance) → Rust
     - CLI tools (rapid dev) → Python

   **Reasoning**:
   [Analyze spec content for keywords like browser, API, DOM, CSS, etc.
    and provide specific reasoning for recommendation]

   **Recommended File**: [Select largest spec file or one with "architecture"/"overview" in name]

   **Suggested Text to Add**:

   Add this section near the top of the specification (after the title):

   ```markdown
   ## Implementation Language

   **Language**: [Recommended Language]

   **Rationale**: [Explain why this language is appropriate for this project]

   **Build System**: [Cargo for Rust, pip for Python, npm for JavaScript, etc.]
   ```

   ══════════════════════════════════════════════════════════════
   NEXT STEPS
   ══════════════════════════════════════════════════════════════

   1. Choose a language (use recommendation or select your own)
   2. Add the language specification section to: [recommended file]
   3. Re-run: /orch-extract-features

   ══════════════════════════════════════════════════════════════
   ```

   **After displaying this message**:
   - DO NOT proceed with feature extraction
   - DO NOT apply templates
   - DO NOT update queue_state.json
   - WAIT for user to update specifications and re-run command

5. **If confidence >= "low"**: Proceed with extraction using detected language

**Store detected language** in memory for template filtering in Step 4.

### Step 1.6: Load Acronym Database (NEW in v1.16.0)

**Read the built-in acronym database** from `orchestration/data/acronyms/builtin.yaml`.

This database contains 50+ technical acronyms with:
- **Full name**: Expanded form (e.g., "WPT" → "Web Platform Tests")
- **URL**: Authoritative source URL
- **Type**: `conformance_suite`, `specification`, `standard`, `protocol`
- **Test paths**: For conformance suites, common test directory paths

**Use this database to**:
1. Expand acronyms found in specifications
2. Add authoritative URLs to external references
3. Identify compliance requirements that reference external test suites

**Example acronyms in database**:
| Acronym | Full Name | Type | URL |
|---------|-----------|------|-----|
| WPT | Web Platform Tests | conformance_suite | https://github.com/web-platform-tests/wpt |
| CSS | Cascading Style Sheets | specification | https://www.w3.org/Style/CSS/ |
| DOM | Document Object Model | specification | https://dom.spec.whatwg.org/ |
| RFC | Request for Comments | standard | https://www.rfc-editor.org/ |

### Step 2: Read Existing Task Queue

Read `orchestration/tasks/queue_state.json` to see what features already exist.

Create a set of existing feature names (normalized to lowercase) to detect duplicates.

### Step 3: Process Each Spec File (ENHANCED)

For each spec file listed in the metadata's `spec_files` array:

1. **Read the specification file**

2. **Extract ALL requirements by TYPE**:

   **Requirement Type Taxonomy** (CRITICAL - categorize every requirement):

   | Type | Recognition Patterns | Example |
   |------|---------------------|---------|
   | `feature` | "Implement X", "Create Y", checklist items, deliverables | "Implement CSS selector parsing" |
   | `constraint` | "Must not exceed", "Limited to", "Maximum of" | "Memory must not exceed 100MB" |
   | `compliance` | "X% pass rate on [suite]", "Compliant with [standard]", references to WPT/RFC/W3C | "85% pass rate on WPT CSS tests" |
   | `performance` | "Under X ms", "Throughput of X/sec", "Latency below" | "Style computation under 16ms" |
   | `quality` | "Test coverage of X%", "Documentation required" | "80% test coverage for all modules" |
   | `integration` | "Must integrate with", "Compatible with" | "Integrate with DOM via StyleResolver" |

3. **For each requirement found**:

   a. **Determine requirement type** using the taxonomy above

   b. **Extract source traceability** (NEW in v1.16.0):
      ```json
      "source": {
        "file": "specifications/css-engine-spec.md",
        "line": 245,
        "verbatim": "The CSS engine MUST achieve 85% pass rate on WPT CSS tests"
      }
      ```

   c. **Check for acronyms** and resolve using the database:
      - If acronym found (e.g., "WPT"), look up in `builtin.yaml`
      - Add `external_reference` with authoritative URL

   d. **For COMPLIANCE requirements** (CRITICAL - prevents bypass):
      - Identify the external standard/suite being referenced
      - Look up in acronym database for authoritative URL
      - Extract threshold (e.g., "85% pass rate")
      - **Generate anti-bypass clauses** (see Step 3.5)

   e. **Check if it already exists** in the queue (fuzzy match on name)
      - If duplicate (> 85% similarity): SKIP
      - If new: Add to extraction list

4. **Extract metadata**:
   - Feature name (concise, clear)
   - Description (detailed if available in spec)
   - Dependencies (if mentioned)
   - Phase/Milestone (if structured)
   - Priority (if mentioned)
   - **Requirement type** (feature/constraint/compliance/performance/quality/integration)
   - **Source location** (file, line, verbatim text)

### Step 3.5: Generate Anti-Bypass Clauses (NEW in v1.16.0)

**For COMPLIANCE requirements only**, generate anti-bypass clauses that prevent mock/substitute implementations.

**Why this is critical**:
The Corten-CSSEngine failure occurred because:
- Spec required "85% WPT CSS pass rate"
- Implementation created internal "WPT-style" tests
- Internal tests passed, but actual WPT was never run
- Project declared "complete" despite 0% actual WPT compliance

**Anti-bypass clause generation**:

For each compliance requirement with an external reference:

1. **Identify the external suite/standard**:
   - Name: from spec (e.g., "WPT")
   - URL: from acronym database
   - Type: `conformance_suite`, `specification`, etc.

2. **Generate NOT_SATISFIED_BY clauses**:

   ```json
   "not_satisfied_by": [
     {
       "description": "Internal tests mimicking WPT naming conventions",
       "rationale": "Must run actual Web Platform Tests from https://github.com/web-platform-tests/wpt"
     },
     {
       "description": "Subset of WPT tests",
       "rationale": "Must run all tests in specified paths, not cherry-picked subset"
     },
     {
       "description": "Mocked or stubbed test harness",
       "rationale": "Must execute tests against real implementation output"
     },
     {
       "description": "Self-reported pass rates without execution evidence",
       "rationale": "Must provide WPT harness execution logs and results"
     }
   ]
   ```

3. **Add verification specification**:

   ```json
   "verification": {
     "method": "external_suite",
     "suite_name": "Web Platform Tests",
     "suite_url": "https://github.com/web-platform-tests/wpt",
     "test_paths": ["css/CSS2/", "css/css-cascade/", "css/css-color/"],
     "threshold": {
       "metric": "pass_rate",
       "value": 0.85,
       "operator": ">="
     },
     "evidence_required": [
       "WPT harness execution log",
       "Results JSON from wpt run",
       "Screenshot of failing tests (if any)"
     ]
   }
   ```

**Bypass patterns to always include**:

| Pattern | Description | Applies To |
|---------|-------------|------------|
| `internal_test_mimicry` | Internal tests with similar names | All conformance suites |
| `test_subset` | Running only some tests | All conformance suites |
| `mocked_harness` | Fake test runner | All conformance suites |
| `self_reported` | Pass rates without logs | All external verification |

### Step 4: Apply Feature Templates (Language-Aware)

**Purpose**: Enrich features with language-specific libraries, test strategies, and implementation guidance.

**Template Loading**:
1. Load all templates from `orchestration/templates/{category}/*.yaml`
2. Parse YAML and create template database
3. Build keyword index for efficient matching

**Template Matching Algorithm**:

For each extracted feature:

1. **Calculate match scores** for all templates:
   ```python
   def calculate_match_score(feature_text: str, template: dict) -> float:
       score = 0.0
       feature_lower = feature_text.lower()

       # Keyword matching (1.0 point per keyword found)
       for keyword in template["keywords"]:
           if keyword.lower() in feature_lower:
               score += 1.0

       # Name similarity (0-2.0 points)
       from difflib import SequenceMatcher
       name_similarity = SequenceMatcher(
           None,
           feature_lower,
           template["template_name"].lower()
       ).ratio()
       score += name_similarity * 2.0

       # Category bonus (0.5 points if category in feature text)
       if template["category"] in feature_lower:
           score += 0.5

       return score
   ```

2. **Select best template** (score > 2.0 required for match)

3. **Filter libraries by detected language**

4. **Apply template metadata** to feature

**Template Categories**:
- networking: HTTP, WebSocket, WebRTC, TLS
- rendering: CSS, layout, paint, graphics
- dom: DOM tree, elements, events
- fonts: Font loading, shaping, rasterization
- javascript: JS runtime, V8, WASM
- media: Video/audio codecs, streaming
- devtools: Inspector, debugger, profiler
- shell: Window management, tabs, UI chrome

### Step 5: Build Complete Task Objects (NEW in v1.16.0)

For each extracted requirement, build a complete task object with all new fields:

```json
{
  "id": "FEAT-074",
  "name": "WPT CSS Test Compliance",
  "description": "Achieve 85% pass rate on Web Platform Tests CSS test suite",
  "type": "compliance",
  "status": "pending",
  "extraction_method": "llm",

  "source": {
    "file": "specifications/css-engine-spec.md",
    "line": 245,
    "verbatim": "The CSS engine MUST achieve 85% pass rate on WPT CSS tests"
  },

  "external_reference": {
    "name": "Web Platform Tests",
    "acronym": "WPT",
    "url": "https://github.com/web-platform-tests/wpt",
    "type": "conformance_suite",
    "test_paths": ["css/CSS2/", "css/css-cascade/", "css/css-color/"]
  },

  "verification": {
    "method": "external_suite",
    "suite_name": "Web Platform Tests",
    "suite_url": "https://github.com/web-platform-tests/wpt",
    "threshold": {
      "metric": "pass_rate",
      "value": 0.85,
      "operator": ">="
    },
    "evidence_required": [
      "WPT harness execution log",
      "Results JSON from wpt run"
    ]
  },

  "not_satisfied_by": [
    {
      "description": "Internal 'WPT-style' tests",
      "rationale": "Must run actual WPT from official repository"
    },
    {
      "description": "Subset of WPT tests",
      "rationale": "Must run all tests in specified paths"
    }
  ],

  "acceptance_criteria": [
    "WPT CSS tests executed against implementation",
    "Pass rate >= 85% verified via official WPT harness",
    "Failing tests documented with issue tracking"
  ]
}
```

**Field requirements by type**:

| Requirement Type | Required Fields |
|------------------|-----------------|
| `feature` | name, description, source, acceptance_criteria |
| `compliance` | ALL fields including external_reference, verification, not_satisfied_by |
| `constraint` | name, description, source, threshold (if numeric) |
| `performance` | name, description, source, threshold, verification method |
| `quality` | name, description, source, threshold |
| `integration` | name, description, source, target system |

### Step 6: Merge with Existing Queue

**CRITICAL**: This is an UPDATE operation, not a REPLACE.

1. **Load existing queue_state.json**

2. **Preserve existing features**:
   - Keep all existing feature IDs (FEAT-001, FEAT-002, etc.)
   - Keep all statuses (pending/in_progress/completed)
   - Keep all timestamps (started_at, completed_at)
   - Keep all verification results

3. **Add new features**:
   - Assign next sequential IDs (if last was FEAT-073, start at FEAT-074)
   - Set status to "pending"
   - Set timestamps to null
   - Add source field: `"extraction_method": "llm"`

4. **Enrich existing features** (if missing fields):
   - Add source traceability if missing
   - Add requirement type if missing
   - Add external_reference for compliance requirements
   - Add not_satisfied_by for compliance requirements
   - Mark as `"enriched_by_llm": true`

5. **Save updated queue_state.json**

### Step 7: Update Extraction Metadata

Update `orchestration/data/state/extraction_metadata.json`:
- Increment `llm_features` by number added
- Update `extraction_method` to "hybrid" (was "automated")
- Update `last_extraction` timestamp
- **CRITICAL**: Set `llm_extraction_complete: true`
- **CRITICAL**: Set `llm_extraction_timestamp` to current ISO timestamp

### Step 8: Commit Results to Git

After successfully updating the queue, commit changes:

```bash
git add orchestration/tasks/queue_state.json orchestration/data/state/extraction_metadata.json
git commit -m "feat(queue): LLM extracted [N] requirements ([Y] total, [C] compliance with anti-bypass)"
```

### Step 9: Report Results (ENHANCED)

Show a detailed report:

```
═══════════════════════════════════════════════════════════
  /orch-extract-features Results
═══════════════════════════════════════════════════════════

LANGUAGE DETECTION:
  ✅ Detected Language: [Language] (confidence: [high/medium/low])

ACRONYM RESOLUTION:
  📚 Acronyms found: [N]
  ✅ Resolved via database: [M]
  ⚠️  Unresolved (manual review needed): [K]

Processed specification files:
  ✅ [file1.md] ([N] requirements found, [M] duplicates, [N-M] added)
  ✅ [file2.md] ([N] requirements found, [M] duplicates, [N-M] added)
  ...

Requirements by Type:
  📋 Features: [N]
  🔒 Compliance: [N] (with anti-bypass clauses)
  ⚡ Performance: [N]
  📏 Constraints: [N]
  🔗 Integration: [N]
  ✅ Quality: [N]

Anti-Bypass Protection:
  🛡️  Compliance requirements with anti-bypass clauses: [N]
  📋 Total NOT_SATISFIED_BY clauses generated: [M]
  🔍 External suites referenced: [K]

Summary:
  Files processed: [N]
  Requirements found by LLM: [N]
  Duplicates (already in queue): [M]
  NEW requirements added: [N-M]
  Existing features enriched: [K]

Queue state updated:
  Previous total: [X] features
  New total: [Y] requirements ([N-M] new + [X] existing)

Files updated:
  ✅ orchestration/tasks/queue_state.json
  ✅ orchestration/data/state/extraction_metadata.json

Git Integration:
  ✅ Changes committed

═══════════════════════════════════════════════════════════
  Extraction Complete
═══════════════════════════════════════════════════════════

The task queue now contains ALL requirements from your specifications,
with anti-bypass protection for compliance requirements.

Next steps:
  - Review queue: cat orchestration/tasks/queue_state.json | jq
  - Start orchestration: /orchestrate
  - Verify compliance: python orchestration/gates/external_suite_gate.py . --all
```

---

## Error Handling

If errors occur during extraction:

1. **Spec file not found**: Skip file, log warning, continue with others
2. **Queue state corrupted**: Abort, show error, recommend running `auto_init.py --init` first
3. **Metadata missing**: Abort, show error, recommend running `auto_init.py` first
4. **JSON parse errors**: Show specific error, recommend manual fix
5. **Language detection failed**: Stop execution, show recommendation message (see Step 1.5)
6. **Acronym not in database**: Flag for human review, continue extraction

**Never destructively modify the queue without backing it up first.**

---

## Examples

### Example 1: Compliance Requirement Extraction

**Spec text**:
```markdown
The CSS engine MUST achieve 85% pass rate on WPT CSS tests.
```

**Extracted requirement**:
```json
{
  "id": "FEAT-042",
  "name": "WPT CSS Test Compliance",
  "type": "compliance",
  "source": {
    "file": "specifications/css-engine-spec.md",
    "line": 245,
    "verbatim": "The CSS engine MUST achieve 85% pass rate on WPT CSS tests"
  },
  "external_reference": {
    "name": "Web Platform Tests",
    "acronym": "WPT",
    "url": "https://github.com/web-platform-tests/wpt",
    "type": "conformance_suite"
  },
  "verification": {
    "method": "external_suite",
    "threshold": {"metric": "pass_rate", "value": 0.85, "operator": ">="}
  },
  "not_satisfied_by": [
    {"description": "Internal 'WPT-style' tests", "rationale": "Must run actual WPT"}
  ]
}
```

### Example 2: Performance Requirement Extraction

**Spec text**:
```markdown
Style computation must complete in under 16ms for 60fps rendering.
```

**Extracted requirement**:
```json
{
  "id": "FEAT-043",
  "name": "Style Computation Performance",
  "type": "performance",
  "source": {
    "file": "specifications/css-engine-spec.md",
    "line": 312,
    "verbatim": "Style computation must complete in under 16ms for 60fps rendering"
  },
  "verification": {
    "method": "benchmark",
    "threshold": {"metric": "latency_ms", "value": 16, "operator": "<"}
  },
  "acceptance_criteria": [
    "Benchmark shows p99 latency < 16ms",
    "No frame drops during continuous style updates"
  ]
}
```

### Example 3: Feature Requirement Extraction

**Spec text**:
```markdown
- [ ] Implement CSS selector parsing for class, ID, and attribute selectors
```

**Extracted requirement**:
```json
{
  "id": "FEAT-044",
  "name": "CSS Selector Parsing",
  "type": "feature",
  "source": {
    "file": "specifications/css-engine-spec.md",
    "line": 89,
    "verbatim": "Implement CSS selector parsing for class, ID, and attribute selectors"
  },
  "acceptance_criteria": [
    "Class selectors (.classname) parse correctly",
    "ID selectors (#id) parse correctly",
    "Attribute selectors ([attr=value]) parse correctly",
    "Unit tests cover all selector types"
  ]
}
```
