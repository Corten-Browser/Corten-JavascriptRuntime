//! Web platform APIs for JavaScript runtime
//!
//! Implements Web Workers, WebAssembly, DevTools Protocol,
//! Same-Origin Policy, Structured Clone, Service Workers, and other web platform features.

pub mod workers;
pub mod wasm;
pub mod devtools;
pub mod source_maps;
pub mod csp;
pub mod same_origin;
pub mod structured_clone;
pub mod service_worker;

// Re-export main types
pub use workers::{Worker, SharedArrayBuffer, Atomics};
pub use wasm::{WebAssembly, WasmModule, WasmInstance};
pub use devtools::{DevToolsServer, DebugProtocol};
pub use source_maps::SourceMap;
pub use csp::ContentSecurityPolicy;
pub use same_origin::{Origin, OpaqueOrigin, SameOriginPolicy, OriginError};
pub use structured_clone::{StructuredClone, StructuredValue, CloneError, CloneOptions};
pub use service_worker::{
    ServiceWorker, ServiceWorkerState, ServiceWorkerRegistration, ServiceWorkerContainer,
    ServiceWorkerError, RegistrationOptions, UpdateViaCache,
    FetchRequest, FetchResponse, FetchEvent, RequestMethod, RequestMode, RequestDestination,
    ResponseType, Cache, CacheStorage, FetchEventHandler,
};
