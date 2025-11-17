# Web Platform APIs

Web platform APIs for the Corten JavaScript Runtime, providing browser-like features for server-side execution.

## Features

### Web Workers
Multi-threaded JavaScript execution with isolated contexts and message passing.
- Worker creation from scripts
- Message passing between main thread and workers
- SharedArrayBuffer for shared memory
- Atomics for thread synchronization

### WebAssembly
Full WebAssembly support for high-performance code execution.
- Module compilation and validation
- Instance creation with imports
- Function calls with type checking
- Memory management

### DevTools Protocol
Chrome DevTools Protocol support for debugging.
- Remote debugging via WebSocket
- Breakpoint management
- Step-by-step execution control
- Expression evaluation
- Call stack inspection

### Source Maps
Source map support for debugging transpiled code.
- VLQ-encoded mapping parsing
- Original position lookup
- Generated position lookup
- Source content embedding

### Content Security Policy
Security controls for script execution.
- CSP header parsing
- eval() restriction
- Inline script control
- Source allowlisting
- Violation reporting

## Usage

```rust
use web_platform::{
    Worker, SharedArrayBuffer, Atomics,
    WebAssembly, WasmModule,
    DevToolsServer,
    SourceMap,
    ContentSecurityPolicy,
};

// Create a Web Worker
let worker = Worker::new("self.onmessage = (e) => postMessage(e.data * 2)");
worker.post_message("21");

// Compile WebAssembly
let wasm_bytes = include_bytes!("module.wasm");
let module = WebAssembly::compile(wasm_bytes)?;
let instance = WebAssembly::instantiate(&module, &imports)?;
let result = instance.call("add", &[1.into(), 2.into()])?;

// Start DevTools server
let server = DevToolsServer::new(9229);
server.start().await;

// Parse source map
let source_map = SourceMap::from_json(source_map_json)?;
let original = source_map.original_position_for(10, 5);

// Check CSP
let csp = ContentSecurityPolicy::from_header("script-src 'self'")?;
if csp.allows_eval() {
    // eval is allowed
}
```

## Architecture

```
web_platform/
├── workers.rs      # Web Workers, SharedArrayBuffer, Atomics
├── wasm.rs         # WebAssembly compilation and execution
├── devtools.rs     # Chrome DevTools Protocol
├── source_maps.rs  # Source map parsing and lookup
└── csp.rs          # Content Security Policy enforcement
```

## Dependencies

- **builtins**: JavaScript built-in types
- **interpreter**: JavaScript execution engine
- **async_runtime**: Event loop and promises
- **tokio**: Async runtime for networking
- **serde**: Serialization for protocol messages

## Development Status

This component is in early development. The module structure and public APIs are defined, but implementations are not yet complete.

### TODO
- [ ] Implement Web Worker thread management
- [ ] Implement SharedArrayBuffer memory sharing
- [ ] Implement Atomics operations
- [ ] Integrate WebAssembly runtime
- [ ] Implement DevTools Protocol messages
- [ ] Implement VLQ decoding for source maps
- [ ] Implement CSP directive evaluation

## Testing

```bash
cargo test -p web_platform
```

## License

Part of the Corten JavaScript Runtime project.
