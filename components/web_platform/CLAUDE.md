# Component: web_platform

## Overview
- **Component Name**: web_platform
- **Type**: Feature
- **Version**: 0.1.0

## Description
Web platform APIs for JavaScript runtime, implementing browser-like features for server-side JavaScript execution.

## Tech Stack
- **Language**: Rust
- **Key Dependencies**:
  - tokio (async runtime)
  - serde/serde_json (serialization)
  - builtins (JavaScript built-in objects)
  - interpreter (JavaScript execution)
  - async_runtime (event loop and promises)

## Responsibility
This component provides web platform features:
- **Web Workers**: Multi-threaded JavaScript execution with message passing
- **SharedArrayBuffer**: Shared memory between workers
- **Atomics**: Thread-safe atomic operations
- **WebAssembly**: WASM compilation and execution
- **DevTools Protocol**: Chrome DevTools debugging support
- **Source Maps**: Map transpiled code to original source
- **Content Security Policy**: Script execution security controls

## Directory Structure
```
web_platform/
├── Cargo.toml           # Package configuration
├── src/
│   ├── lib.rs          # Module declarations and re-exports
│   ├── workers.rs      # Web Workers and SharedArrayBuffer
│   ├── wasm.rs         # WebAssembly integration
│   ├── devtools.rs     # Chrome DevTools Protocol
│   ├── source_maps.rs  # Source map support
│   └── csp.rs          # Content Security Policy
├── tests/              # Test suite
├── CLAUDE.md           # This file
└── README.md           # Component documentation
```

## Key APIs

### Workers
- `Worker::new(script)` - Create worker from script
- `Worker::post_message(msg)` - Send message to worker
- `SharedArrayBuffer::new(size)` - Create shared memory
- `Atomics::add/compare_exchange/wait/notify` - Thread synchronization

### WebAssembly
- `WebAssembly::compile(bytes)` - Compile WASM module
- `WebAssembly::instantiate(module, imports)` - Create instance
- `WasmInstance::call(name, args)` - Call exported function

### DevTools
- `DevToolsServer::new(port)` - Start debug server
- `DebugProtocol::set_breakpoint` - Set breakpoint
- `DebugProtocol::step_over/into/out` - Step control
- `DebugProtocol::evaluate` - Evaluate expression

### Source Maps
- `SourceMap::from_json(json)` - Parse source map
- `SourceMap::original_position_for` - Map to original source
- `SourceMap::generated_position_for` - Map to generated code

### Content Security Policy
- `ContentSecurityPolicy::from_header` - Parse CSP header
- `ContentSecurityPolicy::allows_eval` - Check eval permission
- `ContentSecurityPolicy::allows_script` - Check script source

## Development Guidelines

### Code Quality
- Follow TDD (Red-Green-Refactor)
- Minimum 80% test coverage
- All public APIs must have documentation
- Use proper error handling (Result types)
- Ensure thread safety for worker APIs

### Security Considerations
- CSP enforcement must be strict by default
- SharedArrayBuffer requires proper isolation
- WASM execution must be sandboxed
- DevTools access should be authenticated

### Performance
- Worker message passing should be efficient
- WASM execution should be near-native speed
- Source map lookups should be fast (consider caching)
- DevTools overhead should be minimal when not in use

## Integration Points
- **builtins**: Provides JavaScript value types
- **interpreter**: JavaScript execution engine
- **async_runtime**: Event loop for async operations

## Testing Strategy
- Unit tests for each module
- Integration tests for worker communication
- WASM conformance tests
- DevTools protocol compatibility tests
- CSP policy enforcement tests
