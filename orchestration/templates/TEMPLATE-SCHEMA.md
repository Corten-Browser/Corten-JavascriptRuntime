# Feature Template Schema v1.0

This document defines the schema for feature templates used by the /orch-extract-features command.

## Template Structure

```yaml
template_id: "unique-template-identifier"
template_name: "Human-Readable Template Name"
category: "networking|rendering|dom|fonts|javascript|media|devtools|shell"
difficulty: "easy|medium|hard|expert"
estimated_loc: "Range in lines of code (e.g., 500-1000)"

# Keywords for matching features to this template
keywords:
  - "http"
  - "client"
  - "request"
  - "response"

# Template parameters extracted from feature context
parameters:
  - name: "protocol_version"
    type: "string|array|boolean|integer"
    required: true|false
    options: ["HTTP/1.0", "HTTP/1.1", "HTTP/2", "HTTP/3"]  # For enum types
    default: "HTTP/1.1"
    description: "Which protocol version to implement"

# Core interfaces/APIs to implement
core_interfaces:
  - name: "InterfaceName"
    description: "What this interface does"
    methods:
      - "async fn request(req: Request) -> Response"
      - "fn close(&mut self)"

# Language-specific library recommendations
recommended_libraries:
  rust:
    - name: "hyper"
      version: "1.0"
      required: true
      purpose: "HTTP client/server"
      registry_url: "https://crates.io/crates/hyper"
  python:
    - name: "httpx"
      version: "0.25"
      required: true
      purpose: "HTTP client"
      registry_url: "https://pypi.org/project/httpx/"

# Detailed acceptance criteria (testable requirements)
acceptance_criteria:
  - "GET requests complete successfully with 2xx status"
  - "POST requests send request body correctly"
  - "Connection pooling reuses existing connections"
  - "Handles redirects up to 10 hops"
  - "Performance: 1000 req/sec baseline on localhost"

# Test strategy
test_strategy:
  unit_tests:
    - "Request builder with various methods"
    - "Header parsing and serialization"
    - "URL parsing and validation"
  integration_tests:
    - "Full request/response cycle with test server"
    - "Connection pooling behavior"
    - "Error handling for network failures"
  benchmarks:
    - "Request throughput (req/sec)"
    - "Latency percentiles (p50, p95, p99)"
    - "Memory usage per connection"
  public_test_suites:
    - name: "Web Platform Tests (WPT)"
      url: "https://github.com/web-platform-tests/wpt"
      test_paths: ["fetch/api/", "fetch/http-cache/"]
      target_pass_rate: 90  # Percentage

# Implementation hints
implementation_hints:
  - "Use connection pooling for performance"
  - "Implement timeout handling for all network operations"
  - "Follow HTTP RFCs 7230-7235 for HTTP/1.1"
  - "Support both HTTP and HTTPS (TLS)"

# Common pitfalls
common_pitfalls:
  - issue: "Not handling partial reads from socket"
    solution: "Use buffered reader and loop until complete"
  - issue: "Connection leaks when errors occur"
    solution: "Use RAII pattern (Drop trait in Rust) for cleanup"
  - issue: "Blocking on network I/O in async context"
    solution: "Use async/await throughout the stack"

# Dependencies (other templates this depends on)
depends_on:
  - "template://networking/tls-client"  # If HTTPS required
  - "template://core/async-runtime"     # If async required

# Related templates (alternatives or extensions)
related_templates:
  - "template://networking/http-server"  # Server-side equivalent
  - "template://networking/http2-support"  # Protocol upgrade
```

## Field Descriptions

### Required Fields

- **template_id**: Unique identifier using `category-name` format
- **template_name**: Human-readable name shown to users
- **category**: One of the 8 defined categories
- **keywords**: List of terms for matching features to templates
- **recommended_libraries**: At least one language must be defined

### Optional Fields

- **difficulty**: Complexity level (default: "medium")
- **estimated_loc**: Lines of code estimate
- **parameters**: Template configuration extracted from context
- **core_interfaces**: Key APIs/interfaces to implement
- **acceptance_criteria**: Testable requirements
- **test_strategy**: Testing approach by test type
- **implementation_hints**: Best practices
- **common_pitfalls**: Known issues and solutions
- **depends_on**: Template dependencies
- **related_templates**: Alternative/extension templates

## Template Matching

Templates are matched to features using a scoring algorithm:

1. **Keyword matching** (1.0 point per keyword found in feature text)
2. **Name similarity** (0-2.0 points based on template name vs feature name similarity)
3. **Category match** (0.5 bonus points if categories align)

**Threshold**: Score > 2.0 required for template application

## Language Filtering

When applying templates, ONLY libraries for the detected project language are included.

Example:
- Project language: Rust
- Template has rust and python libraries
- Only rust libraries are included in feature enrichment

This prevents recommending Python libraries for Rust projects.

## Creating New Templates

1. Copy an existing template from the same category
2. Update all fields to match the new feature
3. Ensure keywords cover common variations
4. Include language-specific libraries from official registries
5. Test with sample features to verify matching
6. Add to appropriate category directory

## Template Categories

- **networking**: HTTP, WebSocket, WebRTC, protocols
- **rendering**: CSS, layout, paint, graphics
- **dom**: DOM tree, elements, traversal, manipulation
- **fonts**: Font loading, shaping, rasterization
- **javascript**: JS runtime, V8, WASM, bindings
- **media**: Video/audio codecs, streaming, playback
- **devtools**: Inspector, debugger, profiler
- **shell**: Window management, tabs, UI chrome
