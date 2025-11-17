//! Web platform APIs for JavaScript runtime
//!
//! Implements Web Workers, WebAssembly, DevTools Protocol,
//! and other web platform features.

pub mod workers;
pub mod wasm;
pub mod devtools;
pub mod source_maps;
pub mod csp;

// Re-export main types
pub use workers::{Worker, SharedArrayBuffer, Atomics};
pub use wasm::{WebAssembly, WasmModule, WasmInstance};
pub use devtools::{DevToolsServer, DebugProtocol};
pub use source_maps::SourceMap;
pub use csp::ContentSecurityPolicy;
