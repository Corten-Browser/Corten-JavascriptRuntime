# Extract Features from Specifications (LLM Supplement)

**Purpose**: Use Claude's intelligence to extract features that the automated regex patterns missed.

**How it works**:
1. Reads `orchestration/extraction_metadata.json` to find which spec files were processed
2. Detects implementation language from spec content (CRITICAL - stops if none detected)
3. Focuses on files with low automated extraction rates
4. Extracts features using LLM understanding (not regex)
5. Applies feature templates for detected language
6. Merges new features into existing task queue (UPDATE not REPLACE)
7. Preserves all existing features, IDs, statuses, timestamps
8. Auto-commits results to git

---

## Instructions for Claude

You are supplementing the automated feature extraction system.

### Step 1: Read Extraction Metadata

Read `orchestration/extraction_metadata.json` to determine:
- Which spec files were processed
- How many features were extracted per file (automated)
- Which files need LLM supplementation (low extraction count)

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

   Example for a browser project:

   ```markdown
   ## Implementation Language

   **Language**: Rust

   **Rationale**: Rust provides memory safety without garbage collection,
   critical for browser components handling untrusted web content. The
   strong type system and ownership model prevent common security
   vulnerabilities while maintaining C++-level performance.

   **Build System**: Cargo

   **Dependencies**: Available via crates.io
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

**Why this is critical**:
- Feature templates require knowing target language
- Library recommendations must match project language
- Cannot recommend Python libraries for Rust project
- Cannot recommend Rust crates for Python project

**Store detected language** in memory for template filtering in Step 4.

### Step 2: Read Existing Task Queue

Read `orchestration/tasks/queue_state.json` to see what features already exist.

Create a set of existing feature names (normalized to lowercase) to detect duplicates.

### Step 3: Process Each Spec File

For each spec file listed in the metadata's `spec_files` array:

1. **Read the specification file**

2. **Extract ALL implementable features**:
   - Milestone/phase deliverables
   - Checklist items (any format)
   - Explicit requirements
   - Deliverable lists
   - Goals requiring implementation
   - Features in validation checklists
   - Component specifications
   - Test requirements that imply features
   - **Deliverables**: sections
   - Phase/milestone goals

3. **For each feature found**:
   - Check if it already exists in the queue (fuzzy match on name)
   - If duplicate (> 85% similarity): SKIP (automation already found it)
   - If new: Add to extraction list

4. **Extract metadata**:
   - Feature name (concise, clear)
   - Description (detailed if available in spec)
   - Dependencies (if mentioned)
   - Phase/Milestone (if structured)
   - Priority (if mentioned)

**Duplicate Detection Algorithm**:

```python
def is_duplicate(llm_feature_name: str, existing_features: list[dict]) -> bool:
    """Check if LLM feature is duplicate of existing feature."""
    llm_normalized = llm_feature_name.lower().strip()

    for existing in existing_features:
        existing_normalized = existing["name"].lower().strip()

        # Exact match
        if llm_normalized == existing_normalized:
            return True

        # Remove common prefixes
        prefixes = ["implement ", "create ", "build ", "add ", "load ", "parse ", "wrap "]
        llm_clean = llm_normalized
        existing_clean = existing_normalized
        for prefix in prefixes:
            llm_clean = llm_clean.replace(prefix, "")
            existing_clean = existing_clean.replace(prefix, "")

        if llm_clean == existing_clean:
            return True

        # High similarity (> 85% character overlap)
        from difflib import SequenceMatcher
        similarity = SequenceMatcher(None, llm_clean, existing_clean).ratio()
        if similarity > 0.85:
            return True

    return False
```

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

3. **Filter libraries by detected language**:
   ```python
   # Only include libraries for detected language
   if "recommended_libraries" in template:
       if detected_language in template["recommended_libraries"]:
           libraries = template["recommended_libraries"][detected_language]
       else:
           libraries = None  # No libraries for this language
   ```

4. **Apply template metadata** to feature:
   ```python
   feature["template_id"] = template["template_id"]
   feature["template_name"] = template["template_name"]
   feature["difficulty"] = template["difficulty"]
   feature["estimated_loc"] = template["estimated_loc"]
   feature["recommended_libraries"] = libraries  # Language-filtered
   feature["acceptance_criteria"] = template["acceptance_criteria"]
   feature["test_strategy"] = template["test_strategy"]
   feature["implementation_hints"] = template.get("implementation_hints", [])
   feature["common_pitfalls"] = template.get("common_pitfalls", [])
   feature["detected_language"] = detected_language
   ```

5. **If no template matches** (all scores < 2.0):
   ```python
   feature["template_id"] = None
   feature["template_name"] = "No template match"
   feature["detected_language"] = detected_language
   feature["recommendation"] = "Manual implementation planning required"
   ```

**Why Language Filtering is Critical**:
- Prevents recommending Python libraries for Rust projects
- Prevents recommending Rust crates for Python projects
- Ensures build commands match project language
- Provides relevant ecosystem guidance

**Template Categories**:
- networking: HTTP, WebSocket, WebRTC, TLS
- rendering: CSS, layout, paint, graphics
- dom: DOM tree, elements, events
- fonts: Font loading, shaping, rasterization
- javascript: JS runtime, V8, WASM
- media: Video/audio codecs, streaming
- devtools: Inspector, debugger, profiler
- shell: Window management, tabs, UI chrome

**Example Template Application**:

```python
# Feature: "HTTP/2 protocol support"
# Matched template: networking/http-client.yaml (score: 4.5)
# Detected language: Rust

feature = {
    "id": "FEAT-074",
    "name": "HTTP/2 protocol support",
    "template_id": "networking-http-client",
    "template_name": "HTTP Protocol Client",
    "difficulty": "medium",
    "estimated_loc": "5000-8000",
    "detected_language": "Rust",
    "recommended_libraries": [
        {
            "name": "hyper",
            "version": "1.0",
            "required": True,
            "purpose": "Low-level HTTP implementation",
            "registry_url": "https://crates.io/crates/hyper"
        },
        {
            "name": "h2",
            "version": "0.4",
            "required": False,
            "purpose": "HTTP/2 protocol support",
            "registry_url": "https://crates.io/crates/h2"
        }
    ],
    "acceptance_criteria": [
        "GET requests return correct response body",
        "Connection pooling reuses connections (> 80% reuse rate)",
        "Performance: 1000 req/sec on localhost loopback"
    ],
    "test_strategy": {
        "unit_tests": ["Request builder with GET/POST/PUT/DELETE"],
        "integration_tests": ["Full request/response with test HTTP server"],
        "public_test_suites": [
            {
                "name": "WPT Fetch API Tests",
                "url": "https://github.com/web-platform-tests/wpt/tree/master/fetch",
                "target_pass_rate": 85
            }
        ]
    },
    "implementation_hints": [
        "Use hyper::Client with connection pooling enabled",
        "Follow RFC 7230-7235 for HTTP/1.1"
    ],
    "common_pitfalls": [
        {
            "issue": "Not handling chunked encoding",
            "solution": "hyper handles this automatically, use Body stream"
        }
    ],
    "extraction_method": "llm",
    "status": "pending"
}
```

**Template Matching Report**:

After template application, display summary:
```
Template Matching Results:
  ✅ 45 features matched to templates (score > 2.0)
  ⚠️  12 features without template match
  📚 Library recommendations: 67 total
  🧪 Test strategies: 45 features with test suites

Top matched templates:
  - networking/http-client.yaml: 8 features
  - rendering/css-parser.yaml: 6 features
  - dom/dom-tree.yaml: 5 features
```

### Step 5: Merge with Existing Queue

**CRITICAL**: This is an UPDATE operation, not a REPLACE.

1. **Load existing queue_state.json**

2. **Preserve existing features**:
   - Keep all existing feature IDs (FEAT-001, FEAT-002, etc.)
   - Keep all statuses (pending/in_progress/completed)
   - Keep all timestamps (started_at, completed_at)
   - Keep all verification results
   - Keep all extraction_method fields

3. **Add new features**:
   - Assign next sequential IDs (if last was FEAT-073, start at FEAT-074)
   - Set status to "pending"
   - Set timestamps to null
   - Add source field: `"extraction_method": "llm"`

4. **Enrich existing features** (OPTIONAL):
   - If automated feature has minimal description, add detailed description from spec
   - Add phase/milestone context if missing
   - Add dependencies if detected
   - Mark as `"enriched_by_llm": true`

5. **Save updated queue_state.json**

### Step 6: Update Extraction Metadata

Update `orchestration/data/state/extraction_metadata.json`:
- Increment `llm_features` by number added
- Update `extraction_method` to "hybrid" (was "automated")
- Update `last_extraction` timestamp
- **CRITICAL**: Set `llm_extraction_complete: true` (prevents re-running on subsequent Level 3 orchestrations)
- **CRITICAL**: Set `llm_extraction_timestamp` to current ISO timestamp

**Example metadata update:**
```python
from datetime import datetime

metadata["llm_extraction_complete"] = True
metadata["llm_extraction_timestamp"] = datetime.now().isoformat()
metadata["extraction_method"] = "hybrid"
metadata["llm_features"] = metadata.get("llm_features", 0) + new_features_count
```

### Step 7: Commit Results to Git

After successfully updating the queue, commit changes:

```python
import subprocess
from pathlib import Path

if Path(".git").exists():
    try:
        subprocess.run(["git", "add",
                       "orchestration/tasks/queue_state.json",
                       "orchestration/extraction_metadata.json"],
                      check=True, capture_output=True)

        commit_msg = f"feat(queue): LLM extracted {new_features_count} features ({total_features} total)"

        subprocess.run(["git", "commit", "-m", commit_msg],
                      check=True, capture_output=True)

        print("\n✅ Changes committed to git")
    except Exception as e:
        print(f"\n⚠️  Git commit failed: {e}")
```

### Step 8: Report Results

Show a detailed report:

```
═══════════════════════════════════════════════════════════
  /orch-extract-features Results
═══════════════════════════════════════════════════════════

LANGUAGE DETECTION:
  ✅ Detected Language: [Language] (confidence: [high/medium/low])

Processed specification files:
  ✅ [file1.md] ([N] features found, [M] duplicates, [N-M] added)
  ✅ [file2.md] ([N] features found, [M] duplicates, [N-M] added)
  ...

Summary:
  Files processed: [N]
  Features found by LLM: [N]
  Duplicates (already in queue): [M]
  NEW features added: [N-M]
  Existing features enriched: [K]

Queue state updated:
  Previous total: [X] features (automated)
  New total: [Y] features ([N-M] LLM + [X] automated)

Coverage improvement:
  Before: [X]% ([X]/[Y] possible features)
  After: [Z]% ([Y]/[Y] features extracted)

Files updated:
  ✅ orchestration/tasks/queue_state.json
  ✅ orchestration/extraction_metadata.json

Git Integration:
  ✅ Changes committed: feat(queue): LLM extracted [N] features ([Y] total)
  📝 View history: git log --oneline orchestration/tasks/

═══════════════════════════════════════════════════════════
  Extraction Complete
═══════════════════════════════════════════════════════════

The task queue now contains ALL features from your specifications.

Next steps:
  - Review queue: cat orchestration/tasks/queue_state.json | jq
  - Start orchestration: /orchestrate
  - Check progress: cat orchestration/tasks/queue_state.json | jq '.tasks[] | select(.status=="completed") | .name'
```

---

## Error Handling

If errors occur during extraction:

1. **Spec file not found**: Skip file, log warning, continue with others
2. **Queue state corrupted**: Abort, show error, recommend running `auto_init.py --init` first
3. **Metadata missing**: Abort, show error, recommend running `auto_init.py` first
4. **JSON parse errors**: Show specific error, recommend manual fix
5. **Language detection failed**: Stop execution, show recommendation message (see Step 1.5)

**Never destructively modify the queue without backing it up first.**

---

## Performance Optimization

For large specification files (> 2000 lines):
- Process in chunks to avoid context window issues
- Extract features section by section
- Merge results at the end

For many spec files (> 10):
- Show progress indicator (Processing file 3/15...)
- Summarize results per file
- Show grand total at end

---

## Examples

### Example 1: Successful Extraction with Language Detection

```
/orch-extract-features

✅ Detected Language: Rust (confidence: high)
   - Found: Cargo.toml (10 occurrences)
   - Found: rust keywords (45 occurrences)

Processing 11 specification files...

✅ browser-shell-specification.md (30 features found, 0 duplicates, 30 added)
✅ css-engine-specification.md (25 features found, 0 duplicates, 25 added)
...

NEW features added: 203
Total features: 276

✅ Changes committed to git
```

### Example 2: Language Detection Failure

```
/orch-extract-features

❌ LANGUAGE DETECTION FAILED

No implementation language detected in specification files.

RECOMMENDATION:
  Recommended Language: Rust
  Reasoning: Browser components require memory safety...

  Add to: browser-shell-specification.md

  [Full recommendation message with markdown example]

NEXT STEPS:
  1. Add language specification to browser-shell-specification.md
  2. Re-run: /orch-extract-features
```

### Example 3: No New Features (All Captured by Automation)

```
/orch-extract-features

✅ Detected Language: Python (confidence: high)

Processing 5 specification files...

✅ All features already extracted by automation
   No new features found

The automated extraction has 100% coverage for these specs.
```
