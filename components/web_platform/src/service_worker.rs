//! Service Worker API implementation
//!
//! Implements the Service Worker API per the W3C specification:
//! https://www.w3.org/TR/service-workers/
//!
//! Service workers enable offline functionality, background sync,
//! and fetch interception for web applications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::same_origin::Origin;
use crate::structured_clone::{CloneError, StructuredValue};

// ============================================================================
// Service Worker State
// ============================================================================

/// Service Worker lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceWorkerState {
    /// Initial state after registration, script being parsed
    Parsed,
    /// Worker is installing (install event dispatched)
    Installing,
    /// Worker installed successfully, waiting to activate
    Installed,
    /// Worker is activating (activate event dispatched)
    Activating,
    /// Worker is active and controlling pages
    Activated,
    /// Worker has been replaced or unregistered
    Redundant,
}

impl ServiceWorkerState {
    /// Check if this state allows fetch interception
    pub fn can_intercept_fetch(&self) -> bool {
        matches!(self, ServiceWorkerState::Activated)
    }

    /// Check if the worker is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, ServiceWorkerState::Redundant)
    }
}

impl std::fmt::Display for ServiceWorkerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceWorkerState::Parsed => write!(f, "parsed"),
            ServiceWorkerState::Installing => write!(f, "installing"),
            ServiceWorkerState::Installed => write!(f, "installed"),
            ServiceWorkerState::Activating => write!(f, "activating"),
            ServiceWorkerState::Activated => write!(f, "activated"),
            ServiceWorkerState::Redundant => write!(f, "redundant"),
        }
    }
}

// ============================================================================
// Service Worker Errors
// ============================================================================

/// Errors that can occur during service worker operations
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceWorkerError {
    /// Invalid script URL
    InvalidScriptUrl(String),
    /// Invalid scope URL
    InvalidScopeUrl(String),
    /// Scope not within script directory
    ScopeOutsideScriptDirectory { scope: String, script: String },
    /// Security error (e.g., cross-origin)
    SecurityError(String),
    /// Registration not found
    RegistrationNotFound(String),
    /// Worker in invalid state for operation
    InvalidState {
        expected: String,
        actual: ServiceWorkerState,
    },
    /// Network error during script fetch
    NetworkError(String),
    /// Script evaluation error
    ScriptError(String),
    /// Cache operation error
    CacheError(String),
    /// Message posting error
    MessageError(String),
}

impl std::fmt::Display for ServiceWorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceWorkerError::InvalidScriptUrl(url) => {
                write!(f, "Invalid service worker script URL: {}", url)
            }
            ServiceWorkerError::InvalidScopeUrl(url) => {
                write!(f, "Invalid service worker scope URL: {}", url)
            }
            ServiceWorkerError::ScopeOutsideScriptDirectory { scope, script } => {
                write!(
                    f,
                    "Scope '{}' is outside of script directory '{}'",
                    scope, script
                )
            }
            ServiceWorkerError::SecurityError(msg) => {
                write!(f, "Security error: {}", msg)
            }
            ServiceWorkerError::RegistrationNotFound(scope) => {
                write!(
                    f,
                    "No service worker registration found for scope: {}",
                    scope
                )
            }
            ServiceWorkerError::InvalidState { expected, actual } => {
                write!(
                    f,
                    "Invalid service worker state: expected {}, got {}",
                    expected, actual
                )
            }
            ServiceWorkerError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            }
            ServiceWorkerError::ScriptError(msg) => {
                write!(f, "Script error: {}", msg)
            }
            ServiceWorkerError::CacheError(msg) => {
                write!(f, "Cache error: {}", msg)
            }
            ServiceWorkerError::MessageError(msg) => {
                write!(f, "Message error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ServiceWorkerError {}

// ============================================================================
// Service Worker
// ============================================================================

/// Represents a Service Worker instance
pub struct ServiceWorker {
    /// Unique identifier
    id: u64,
    /// Script URL
    script_url: String,
    /// Current state
    state: RwLock<ServiceWorkerState>,
    /// State change listeners
    state_listeners: Mutex<Vec<Box<dyn Fn(ServiceWorkerState) + Send + Sync>>>,
    /// Message queue for postMessage
    message_queue: Mutex<Vec<StructuredValue>>,
}

impl ServiceWorker {
    /// Create a new service worker instance
    fn new(script_url: String) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            script_url,
            state: RwLock::new(ServiceWorkerState::Parsed),
            state_listeners: Mutex::new(Vec::new()),
            message_queue: Mutex::new(Vec::new()),
        }
    }

    /// Get the worker's unique ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the script URL
    pub fn script_url(&self) -> &str {
        &self.script_url
    }

    /// Get the current state
    pub fn state(&self) -> ServiceWorkerState {
        *self.state.read().unwrap()
    }

    /// Transition to a new state
    pub(crate) fn set_state(&self, new_state: ServiceWorkerState) {
        let mut state = self.state.write().unwrap();
        let old_state = *state;
        if old_state != new_state {
            *state = new_state;
            drop(state);
            self.notify_state_change(new_state);
        }
    }

    /// Add a state change listener
    pub fn on_state_change<F>(&self, callback: F)
    where
        F: Fn(ServiceWorkerState) + Send + Sync + 'static,
    {
        let mut listeners = self.state_listeners.lock().unwrap();
        listeners.push(Box::new(callback));
    }

    /// Notify all listeners of state change
    fn notify_state_change(&self, new_state: ServiceWorkerState) {
        let listeners = self.state_listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(new_state);
        }
    }

    /// Post a message to the service worker
    pub fn post_message(&self, message: StructuredValue) -> Result<(), ServiceWorkerError> {
        if self.state().is_terminal() {
            return Err(ServiceWorkerError::InvalidState {
                expected: "non-redundant".to_string(),
                actual: self.state(),
            });
        }
        let mut queue = self.message_queue.lock().unwrap();
        queue.push(message);
        Ok(())
    }

    /// Receive pending messages (for worker thread)
    pub fn receive_messages(&self) -> Vec<StructuredValue> {
        let mut queue = self.message_queue.lock().unwrap();
        std::mem::take(&mut *queue)
    }
}

impl std::fmt::Debug for ServiceWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceWorker")
            .field("id", &self.id)
            .field("script_url", &self.script_url)
            .field("state", &self.state())
            .finish()
    }
}

// ============================================================================
// Service Worker Registration
// ============================================================================

/// Registration options
#[derive(Debug, Clone)]
pub struct RegistrationOptions {
    /// Scope URL for the registration
    pub scope: Option<String>,
    /// Update via cache mode
    pub update_via_cache: UpdateViaCache,
}

impl Default for RegistrationOptions {
    fn default() -> Self {
        Self {
            scope: None,
            update_via_cache: UpdateViaCache::Imports,
        }
    }
}

/// Update via cache modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateViaCache {
    /// Check cache for imports only
    Imports,
    /// Check cache for main script and imports
    All,
    /// Never check cache
    None,
}

/// Represents a Service Worker Registration
pub struct ServiceWorkerRegistration {
    /// Unique registration ID
    id: u64,
    /// Scope URL
    scope: String,
    /// Installing worker (if any)
    installing: RwLock<Option<Arc<ServiceWorker>>>,
    /// Waiting worker (if any)
    waiting: RwLock<Option<Arc<ServiceWorker>>>,
    /// Active worker (if any)
    active: RwLock<Option<Arc<ServiceWorker>>>,
    /// Update via cache mode
    update_via_cache: UpdateViaCache,
    /// Whether update is in progress
    update_pending: Mutex<bool>,
    /// Timestamp of last update check
    last_update_check: Mutex<Option<std::time::Instant>>,
}

impl ServiceWorkerRegistration {
    /// Create a new registration
    fn new(scope: String, update_via_cache: UpdateViaCache) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            scope,
            installing: RwLock::new(None),
            waiting: RwLock::new(None),
            active: RwLock::new(None),
            update_via_cache,
            update_pending: Mutex::new(false),
            last_update_check: Mutex::new(None),
        }
    }

    /// Get the registration ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the scope URL
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// Get the installing worker
    pub fn installing(&self) -> Option<Arc<ServiceWorker>> {
        self.installing.read().unwrap().clone()
    }

    /// Get the waiting worker
    pub fn waiting(&self) -> Option<Arc<ServiceWorker>> {
        self.waiting.read().unwrap().clone()
    }

    /// Get the active worker
    pub fn active(&self) -> Option<Arc<ServiceWorker>> {
        self.active.read().unwrap().clone()
    }

    /// Get the update via cache mode
    pub fn update_via_cache(&self) -> UpdateViaCache {
        self.update_via_cache
    }

    /// Check if navigation preload is enabled (stub)
    pub fn navigation_preload_enabled(&self) -> bool {
        false
    }

    /// Start the installation process
    pub(crate) fn start_install(&self, worker: Arc<ServiceWorker>) {
        worker.set_state(ServiceWorkerState::Installing);
        *self.installing.write().unwrap() = Some(worker);
    }

    /// Complete installation successfully
    pub(crate) fn complete_install(&self) -> Result<(), ServiceWorkerError> {
        let installing = self.installing.read().unwrap().clone();
        if let Some(worker) = installing {
            worker.set_state(ServiceWorkerState::Installed);

            // Move installing to waiting
            *self.installing.write().unwrap() = None;
            *self.waiting.write().unwrap() = Some(worker);
            Ok(())
        } else {
            Err(ServiceWorkerError::InvalidState {
                expected: "installing worker present".to_string(),
                actual: ServiceWorkerState::Parsed,
            })
        }
    }

    /// Fail installation
    pub(crate) fn fail_install(&self) {
        let installing = self.installing.write().unwrap().take();
        if let Some(worker) = installing {
            worker.set_state(ServiceWorkerState::Redundant);
        }
    }

    /// Start activation
    pub(crate) fn start_activate(&self) -> Result<(), ServiceWorkerError> {
        let waiting = self.waiting.read().unwrap().clone();
        if let Some(worker) = waiting {
            worker.set_state(ServiceWorkerState::Activating);
            Ok(())
        } else {
            Err(ServiceWorkerError::InvalidState {
                expected: "waiting worker present".to_string(),
                actual: ServiceWorkerState::Parsed,
            })
        }
    }

    /// Complete activation successfully
    pub(crate) fn complete_activate(&self) -> Result<(), ServiceWorkerError> {
        let waiting = self.waiting.read().unwrap().clone();
        if let Some(worker) = waiting {
            if worker.state() != ServiceWorkerState::Activating {
                return Err(ServiceWorkerError::InvalidState {
                    expected: "activating".to_string(),
                    actual: worker.state(),
                });
            }

            // Make the old active worker redundant
            if let Some(old_active) = self.active.read().unwrap().as_ref() {
                old_active.set_state(ServiceWorkerState::Redundant);
            }

            worker.set_state(ServiceWorkerState::Activated);

            // Move waiting to active
            *self.waiting.write().unwrap() = None;
            *self.active.write().unwrap() = Some(worker);
            Ok(())
        } else {
            Err(ServiceWorkerError::InvalidState {
                expected: "waiting worker present".to_string(),
                actual: ServiceWorkerState::Parsed,
            })
        }
    }

    /// Fail activation
    pub(crate) fn fail_activate(&self) {
        let waiting = self.waiting.write().unwrap().take();
        if let Some(worker) = waiting {
            worker.set_state(ServiceWorkerState::Redundant);
        }
    }

    /// Trigger an update check
    pub fn update(&self) -> Result<(), ServiceWorkerError> {
        let mut pending = self.update_pending.lock().unwrap();
        if *pending {
            return Ok(()); // Update already in progress
        }
        *pending = true;
        *self.last_update_check.lock().unwrap() = Some(std::time::Instant::now());
        // In a real implementation, this would fetch the script and compare
        Ok(())
    }

    /// Unregister this service worker
    pub fn unregister(&self) -> bool {
        // Make all workers redundant
        if let Some(w) = self.installing.write().unwrap().take() {
            w.set_state(ServiceWorkerState::Redundant);
        }
        if let Some(w) = self.waiting.write().unwrap().take() {
            w.set_state(ServiceWorkerState::Redundant);
        }
        if let Some(w) = self.active.write().unwrap().take() {
            w.set_state(ServiceWorkerState::Redundant);
        }
        true
    }
}

impl std::fmt::Debug for ServiceWorkerRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceWorkerRegistration")
            .field("id", &self.id)
            .field("scope", &self.scope)
            .field("installing", &self.installing())
            .field("waiting", &self.waiting())
            .field("active", &self.active())
            .finish()
    }
}

// ============================================================================
// Service Worker Container (navigator.serviceWorker)
// ============================================================================

/// The ServiceWorkerContainer interface (navigator.serviceWorker)
pub struct ServiceWorkerContainer {
    /// Origin for this container
    origin: Origin,
    /// All registrations by scope
    registrations: RwLock<HashMap<String, Arc<ServiceWorkerRegistration>>>,
    /// The controlling service worker for this context
    controller: RwLock<Option<Arc<ServiceWorker>>>,
    /// Whether service workers are supported/enabled
    enabled: bool,
}

impl ServiceWorkerContainer {
    /// Create a new service worker container
    pub fn new(origin: Origin) -> Self {
        Self {
            origin,
            registrations: RwLock::new(HashMap::new()),
            controller: RwLock::new(None),
            enabled: true,
        }
    }

    /// Create a disabled container (for contexts where SW isn't supported)
    pub fn disabled() -> Self {
        Self {
            origin: Origin::new("null", "", None),
            registrations: RwLock::new(HashMap::new()),
            controller: RwLock::new(None),
            enabled: false,
        }
    }

    /// Check if service workers are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the current controller
    pub fn controller(&self) -> Option<Arc<ServiceWorker>> {
        self.controller.read().unwrap().clone()
    }

    /// Set the controller (internal use)
    pub(crate) fn set_controller(&self, worker: Option<Arc<ServiceWorker>>) {
        *self.controller.write().unwrap() = worker;
    }

    /// Register a service worker
    pub fn register(
        &self,
        script_url: &str,
        options: Option<RegistrationOptions>,
    ) -> Result<Arc<ServiceWorkerRegistration>, ServiceWorkerError> {
        if !self.enabled {
            return Err(ServiceWorkerError::SecurityError(
                "Service workers are not supported in this context".to_string(),
            ));
        }

        let options = options.unwrap_or_default();

        // Validate script URL
        let script_origin = Origin::parse(script_url)
            .map_err(|_| ServiceWorkerError::InvalidScriptUrl(script_url.to_string()))?;

        // Check same-origin
        if !self.origin.is_same_origin(&script_origin) {
            return Err(ServiceWorkerError::SecurityError(format!(
                "Script URL '{}' is not same-origin with '{}'",
                script_url,
                self.origin.serialize()
            )));
        }

        // Determine scope
        let scope = options.scope.unwrap_or_else(|| {
            // Default scope is the directory containing the script
            let url = script_url;
            if let Some(idx) = url.rfind('/') {
                url[..=idx].to_string()
            } else {
                "/".to_string()
            }
        });

        // Validate scope
        let scope_origin = Origin::parse(&scope)
            .map_err(|_| ServiceWorkerError::InvalidScopeUrl(scope.clone()))?;

        if !self.origin.is_same_origin(&scope_origin) {
            return Err(ServiceWorkerError::SecurityError(format!(
                "Scope '{}' is not same-origin with '{}'",
                scope,
                self.origin.serialize()
            )));
        }

        // Validate scope is within script directory
        let script_dir = if let Some(idx) = script_url.rfind('/') {
            &script_url[..=idx]
        } else {
            "/"
        };

        if !scope.starts_with(script_dir) {
            return Err(ServiceWorkerError::ScopeOutsideScriptDirectory {
                scope: scope.clone(),
                script: script_url.to_string(),
            });
        }

        // Check for existing registration
        let mut registrations = self.registrations.write().unwrap();

        if let Some(existing) = registrations.get(&scope) {
            // Trigger update on existing registration
            existing.update()?;
            return Ok(Arc::clone(existing));
        }

        // Create new registration
        let registration = Arc::new(ServiceWorkerRegistration::new(
            scope.clone(),
            options.update_via_cache,
        ));

        // Create the service worker
        let worker = Arc::new(ServiceWorker::new(script_url.to_string()));

        // Start installation
        registration.start_install(Arc::clone(&worker));

        // Store registration
        registrations.insert(scope, Arc::clone(&registration));

        Ok(registration)
    }

    /// Get a registration by scope
    pub fn get_registration(&self, scope: Option<&str>) -> Option<Arc<ServiceWorkerRegistration>> {
        let registrations = self.registrations.read().unwrap();

        if let Some(scope) = scope {
            registrations.get(scope).cloned()
        } else {
            // Return the registration that controls this page
            // For now, return the first one
            registrations.values().next().cloned()
        }
    }

    /// Get all registrations
    pub fn get_registrations(&self) -> Vec<Arc<ServiceWorkerRegistration>> {
        self.registrations
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Find a registration by client URL
    pub fn match_registration(&self, client_url: &str) -> Option<Arc<ServiceWorkerRegistration>> {
        let registrations = self.registrations.read().unwrap();

        // Find the registration with the longest matching scope
        let mut best_match: Option<&Arc<ServiceWorkerRegistration>> = None;
        let mut best_length = 0;

        for (scope, registration) in registrations.iter() {
            if client_url.starts_with(scope) && scope.len() > best_length {
                best_match = Some(registration);
                best_length = scope.len();
            }
        }

        best_match.cloned()
    }

    /// Remove a registration (internal)
    pub(crate) fn remove_registration(&self, scope: &str) -> bool {
        self.registrations.write().unwrap().remove(scope).is_some()
    }

    /// Start controlling a client
    pub fn start_controlling(&self, client_url: &str) {
        if let Some(registration) = self.match_registration(client_url) {
            if let Some(active) = registration.active() {
                self.set_controller(Some(active));
            }
        }
    }
}

impl std::fmt::Debug for ServiceWorkerContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceWorkerContainer")
            .field("origin", &self.origin)
            .field("enabled", &self.enabled)
            .field(
                "registration_count",
                &self.registrations.read().unwrap().len(),
            )
            .finish()
    }
}

// ============================================================================
// Fetch Event for Service Workers
// ============================================================================

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl std::fmt::Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestMethod::Get => write!(f, "GET"),
            RequestMethod::Post => write!(f, "POST"),
            RequestMethod::Put => write!(f, "PUT"),
            RequestMethod::Delete => write!(f, "DELETE"),
            RequestMethod::Head => write!(f, "HEAD"),
            RequestMethod::Options => write!(f, "OPTIONS"),
            RequestMethod::Patch => write!(f, "PATCH"),
        }
    }
}

/// Request mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestMode {
    Navigate,
    SameOrigin,
    NoCors,
    Cors,
}

/// Request destination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestDestination {
    Document,
    Script,
    Style,
    Image,
    Font,
    Worker,
    ServiceWorker,
    SharedWorker,
    Audio,
    Video,
    Object,
    Embed,
    Frame,
    Iframe,
    Unknown,
}

/// A fetch request that can be intercepted by a service worker
#[derive(Debug, Clone)]
pub struct FetchRequest {
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: RequestMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<Vec<u8>>,
    /// Request mode
    pub mode: RequestMode,
    /// Request destination
    pub destination: RequestDestination,
    /// Client ID
    pub client_id: Option<String>,
    /// Whether this is a reload
    pub is_reload: bool,
}

impl FetchRequest {
    /// Create a new fetch request
    pub fn new(url: impl Into<String>, method: RequestMethod) -> Self {
        Self {
            url: url.into(),
            method,
            headers: HashMap::new(),
            body: None,
            mode: RequestMode::Cors,
            destination: RequestDestination::Unknown,
            client_id: None,
            is_reload: false,
        }
    }

    /// Create a navigation request
    pub fn navigate(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: RequestMethod::Get,
            headers: HashMap::new(),
            body: None,
            mode: RequestMode::Navigate,
            destination: RequestDestination::Document,
            client_id: None,
            is_reload: false,
        }
    }

    /// Clone the request
    pub fn clone_request(&self) -> Self {
        self.clone()
    }
}

/// A fetch response
#[derive(Debug, Clone)]
pub struct FetchResponse {
    /// Response status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
    /// Response type
    pub response_type: ResponseType,
    /// Response URL
    pub url: String,
    /// Whether the response was redirected
    pub redirected: bool,
}

/// Response types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseType {
    Basic,
    Cors,
    Default,
    Error,
    Opaque,
    OpaqueRedirect,
}

impl FetchResponse {
    /// Create a new response
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            status_text: status_text(status).to_string(),
            headers: HashMap::new(),
            body,
            response_type: ResponseType::Default,
            url: String::new(),
            redirected: false,
        }
    }

    /// Create an error response
    pub fn error() -> Self {
        Self {
            status: 0,
            status_text: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            response_type: ResponseType::Error,
            url: String::new(),
            redirected: false,
        }
    }

    /// Create a redirect response
    pub fn redirect(url: &str, status: u16) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), url.to_string());
        Self {
            status,
            status_text: status_text(status).to_string(),
            headers,
            body: Vec::new(),
            response_type: ResponseType::Default,
            url: String::new(),
            redirected: false,
        }
    }

    /// Check if the response is OK (status 200-299)
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Clone the response
    pub fn clone_response(&self) -> Self {
        self.clone()
    }
}

/// Get status text for common status codes
fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "",
    }
}

/// Fetch event that can be handled by a service worker
pub struct FetchEvent {
    /// The request being intercepted
    pub request: FetchRequest,
    /// Client ID
    pub client_id: Option<String>,
    /// Result slot ID
    pub result_id: Option<String>,
    /// Whether respondWith was called
    responded: Mutex<bool>,
    /// The response (if respondWith was called)
    response: Mutex<Option<FetchResponse>>,
}

impl FetchEvent {
    /// Create a new fetch event
    pub fn new(request: FetchRequest) -> Self {
        let client_id = request.client_id.clone();
        Self {
            request,
            client_id,
            result_id: None,
            responded: Mutex::new(false),
            response: Mutex::new(None),
        }
    }

    /// Respond to the fetch event with a custom response
    pub fn respond_with(&self, response: FetchResponse) -> Result<(), ServiceWorkerError> {
        let mut responded = self.responded.lock().unwrap();
        if *responded {
            return Err(ServiceWorkerError::InvalidState {
                expected: "respondWith not called".to_string(),
                actual: ServiceWorkerState::Activated,
            });
        }
        *responded = true;
        *self.response.lock().unwrap() = Some(response);
        Ok(())
    }

    /// Check if respondWith was called
    pub fn was_responded(&self) -> bool {
        *self.responded.lock().unwrap()
    }

    /// Get the response (if respondWith was called)
    pub fn get_response(&self) -> Option<FetchResponse> {
        self.response.lock().unwrap().clone()
    }
}

// ============================================================================
// Cache API
// ============================================================================

/// A single cache in the Cache Storage
pub struct Cache {
    /// Cache name
    name: String,
    /// Cached entries (URL -> Response)
    entries: RwLock<HashMap<String, FetchResponse>>,
}

impl Cache {
    /// Create a new cache
    fn new(name: String) -> Self {
        Self {
            name,
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Get the cache name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Match a request against the cache
    pub fn match_request(&self, request: &FetchRequest) -> Option<FetchResponse> {
        self.match_url(&request.url)
    }

    /// Match a URL against the cache
    pub fn match_url(&self, url: &str) -> Option<FetchResponse> {
        self.entries.read().unwrap().get(url).cloned()
    }

    /// Match all entries for a request
    pub fn match_all(&self, request: Option<&FetchRequest>) -> Vec<FetchResponse> {
        let entries = self.entries.read().unwrap();
        if let Some(req) = request {
            entries
                .iter()
                .filter(|(url, _)| url.starts_with(&req.url))
                .map(|(_, resp)| resp.clone())
                .collect()
        } else {
            entries.values().cloned().collect()
        }
    }

    /// Add a request/response pair to the cache
    pub fn put(
        &self,
        request: &FetchRequest,
        response: FetchResponse,
    ) -> Result<(), ServiceWorkerError> {
        if response.status == 206 {
            return Err(ServiceWorkerError::CacheError(
                "Cannot cache partial responses (206)".to_string(),
            ));
        }
        self.entries
            .write()
            .unwrap()
            .insert(request.url.clone(), response);
        Ok(())
    }

    /// Add a URL directly (will fetch and cache)
    pub fn add(&self, url: &str) -> Result<(), ServiceWorkerError> {
        // In a real implementation, this would fetch the URL
        // For now, we create a placeholder entry
        let response = FetchResponse::new(200, Vec::new());
        self.entries
            .write()
            .unwrap()
            .insert(url.to_string(), response);
        Ok(())
    }

    /// Add multiple URLs
    pub fn add_all(&self, urls: &[&str]) -> Result<(), ServiceWorkerError> {
        for url in urls {
            self.add(url)?;
        }
        Ok(())
    }

    /// Delete a cached entry
    pub fn delete(&self, request: &FetchRequest) -> bool {
        self.entries.write().unwrap().remove(&request.url).is_some()
    }

    /// Delete by URL
    pub fn delete_url(&self, url: &str) -> bool {
        self.entries.write().unwrap().remove(url).is_some()
    }

    /// Get all cached request URLs
    pub fn keys(&self) -> Vec<String> {
        self.entries.read().unwrap().keys().cloned().collect()
    }
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cache")
            .field("name", &self.name)
            .field("entry_count", &self.entries.read().unwrap().len())
            .finish()
    }
}

/// Cache Storage API
pub struct CacheStorage {
    /// All caches by name
    caches: RwLock<HashMap<String, Arc<Cache>>>,
    /// Origin for this storage
    origin: Origin,
}

impl CacheStorage {
    /// Create a new cache storage
    pub fn new(origin: Origin) -> Self {
        Self {
            caches: RwLock::new(HashMap::new()),
            origin,
        }
    }

    /// Open a cache (creates if doesn't exist)
    pub fn open(&self, name: &str) -> Arc<Cache> {
        let mut caches = self.caches.write().unwrap();
        if let Some(cache) = caches.get(name) {
            Arc::clone(cache)
        } else {
            let cache = Arc::new(Cache::new(name.to_string()));
            caches.insert(name.to_string(), Arc::clone(&cache));
            cache
        }
    }

    /// Check if a cache exists
    pub fn has(&self, name: &str) -> bool {
        self.caches.read().unwrap().contains_key(name)
    }

    /// Delete a cache
    pub fn delete(&self, name: &str) -> bool {
        self.caches.write().unwrap().remove(name).is_some()
    }

    /// Get all cache names
    pub fn keys(&self) -> Vec<String> {
        self.caches.read().unwrap().keys().cloned().collect()
    }

    /// Match a request against all caches
    pub fn match_request(&self, request: &FetchRequest) -> Option<FetchResponse> {
        for cache in self.caches.read().unwrap().values() {
            if let Some(response) = cache.match_request(request) {
                return Some(response);
            }
        }
        None
    }

    /// Match a URL against all caches
    pub fn match_url(&self, url: &str) -> Option<FetchResponse> {
        for cache in self.caches.read().unwrap().values() {
            if let Some(response) = cache.match_url(url) {
                return Some(response);
            }
        }
        None
    }
}

impl std::fmt::Debug for CacheStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheStorage")
            .field("origin", &self.origin)
            .field("cache_count", &self.caches.read().unwrap().len())
            .finish()
    }
}

// ============================================================================
// Fetch Event Handler
// ============================================================================

/// Handler for fetch events
pub struct FetchEventHandler {
    /// Service worker container
    container: Arc<ServiceWorkerContainer>,
    /// Cache storage
    cache_storage: Arc<CacheStorage>,
}

impl FetchEventHandler {
    /// Create a new fetch event handler
    pub fn new(container: Arc<ServiceWorkerContainer>, cache_storage: Arc<CacheStorage>) -> Self {
        Self {
            container,
            cache_storage,
        }
    }

    /// Handle a fetch event
    pub fn handle(&self, request: FetchRequest) -> Result<FetchResponse, ServiceWorkerError> {
        // Check if there's an active service worker that can handle this
        if let Some(controller) = self.container.controller() {
            if controller.state().can_intercept_fetch() {
                // Create fetch event
                let event = FetchEvent::new(request.clone());

                // In a real implementation, this would dispatch to the worker
                // and wait for respondWith to be called

                // For now, try cache first
                if let Some(cached) = self.cache_storage.match_request(&request) {
                    return Ok(cached);
                }
            }
        }

        // No interception, return a "not handled" indicator
        // In practice, this would fall through to the network
        Err(ServiceWorkerError::NetworkError(
            "Not handled by service worker".to_string(),
        ))
    }

    /// Check if a request would be intercepted
    pub fn would_intercept(&self, url: &str) -> bool {
        if let Some(controller) = self.container.controller() {
            if controller.state().can_intercept_fetch() {
                // Check if the URL is in scope
                if let Some(registration) = self.container.match_registration(url) {
                    return url.starts_with(registration.scope());
                }
            }
        }
        false
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_origin() -> Origin {
        Origin::new("https", "example.com", Some(443))
    }

    // Service Worker State Tests
    mod state_tests {
        use super::*;

        #[test]
        fn test_state_display() {
            assert_eq!(ServiceWorkerState::Parsed.to_string(), "parsed");
            assert_eq!(ServiceWorkerState::Installing.to_string(), "installing");
            assert_eq!(ServiceWorkerState::Installed.to_string(), "installed");
            assert_eq!(ServiceWorkerState::Activating.to_string(), "activating");
            assert_eq!(ServiceWorkerState::Activated.to_string(), "activated");
            assert_eq!(ServiceWorkerState::Redundant.to_string(), "redundant");
        }

        #[test]
        fn test_can_intercept_fetch() {
            assert!(!ServiceWorkerState::Parsed.can_intercept_fetch());
            assert!(!ServiceWorkerState::Installing.can_intercept_fetch());
            assert!(!ServiceWorkerState::Installed.can_intercept_fetch());
            assert!(!ServiceWorkerState::Activating.can_intercept_fetch());
            assert!(ServiceWorkerState::Activated.can_intercept_fetch());
            assert!(!ServiceWorkerState::Redundant.can_intercept_fetch());
        }

        #[test]
        fn test_is_terminal() {
            assert!(!ServiceWorkerState::Parsed.is_terminal());
            assert!(!ServiceWorkerState::Installing.is_terminal());
            assert!(!ServiceWorkerState::Installed.is_terminal());
            assert!(!ServiceWorkerState::Activating.is_terminal());
            assert!(!ServiceWorkerState::Activated.is_terminal());
            assert!(ServiceWorkerState::Redundant.is_terminal());
        }
    }

    // Service Worker Tests
    mod worker_tests {
        use super::*;

        #[test]
        fn test_create_worker() {
            let worker = ServiceWorker::new("https://example.com/sw.js".to_string());
            assert_eq!(worker.script_url(), "https://example.com/sw.js");
            assert_eq!(worker.state(), ServiceWorkerState::Parsed);
        }

        #[test]
        fn test_state_transitions() {
            let worker = ServiceWorker::new("https://example.com/sw.js".to_string());

            worker.set_state(ServiceWorkerState::Installing);
            assert_eq!(worker.state(), ServiceWorkerState::Installing);

            worker.set_state(ServiceWorkerState::Installed);
            assert_eq!(worker.state(), ServiceWorkerState::Installed);

            worker.set_state(ServiceWorkerState::Activating);
            assert_eq!(worker.state(), ServiceWorkerState::Activating);

            worker.set_state(ServiceWorkerState::Activated);
            assert_eq!(worker.state(), ServiceWorkerState::Activated);
        }

        #[test]
        fn test_post_message() {
            let worker = ServiceWorker::new("https://example.com/sw.js".to_string());
            worker.set_state(ServiceWorkerState::Activated);

            let msg = StructuredValue::String("hello".to_string());
            assert!(worker.post_message(msg).is_ok());

            let messages = worker.receive_messages();
            assert_eq!(messages.len(), 1);
        }

        #[test]
        fn test_post_message_redundant() {
            let worker = ServiceWorker::new("https://example.com/sw.js".to_string());
            worker.set_state(ServiceWorkerState::Redundant);

            let msg = StructuredValue::String("hello".to_string());
            assert!(worker.post_message(msg).is_err());
        }

        #[test]
        fn test_unique_ids() {
            let worker1 = ServiceWorker::new("https://example.com/sw1.js".to_string());
            let worker2 = ServiceWorker::new("https://example.com/sw2.js".to_string());
            assert_ne!(worker1.id(), worker2.id());
        }
    }

    // Registration Tests
    mod registration_tests {
        use super::*;

        #[test]
        fn test_create_registration() {
            let reg = ServiceWorkerRegistration::new(
                "https://example.com/".to_string(),
                UpdateViaCache::Imports,
            );
            assert_eq!(reg.scope(), "https://example.com/");
            assert!(reg.installing().is_none());
            assert!(reg.waiting().is_none());
            assert!(reg.active().is_none());
        }

        #[test]
        fn test_installation_lifecycle() {
            let reg = ServiceWorkerRegistration::new(
                "https://example.com/".to_string(),
                UpdateViaCache::Imports,
            );

            let worker = Arc::new(ServiceWorker::new("https://example.com/sw.js".to_string()));

            // Start install
            reg.start_install(Arc::clone(&worker));
            assert!(reg.installing().is_some());
            assert_eq!(worker.state(), ServiceWorkerState::Installing);

            // Complete install
            reg.complete_install().unwrap();
            assert!(reg.installing().is_none());
            assert!(reg.waiting().is_some());
            assert_eq!(worker.state(), ServiceWorkerState::Installed);
        }

        #[test]
        fn test_activation_lifecycle() {
            let reg = ServiceWorkerRegistration::new(
                "https://example.com/".to_string(),
                UpdateViaCache::Imports,
            );

            let worker = Arc::new(ServiceWorker::new("https://example.com/sw.js".to_string()));

            // Go through install
            reg.start_install(Arc::clone(&worker));
            reg.complete_install().unwrap();

            // Start activate
            reg.start_activate().unwrap();
            assert_eq!(worker.state(), ServiceWorkerState::Activating);

            // Complete activate
            reg.complete_activate().unwrap();
            assert!(reg.waiting().is_none());
            assert!(reg.active().is_some());
            assert_eq!(worker.state(), ServiceWorkerState::Activated);
        }

        #[test]
        fn test_unregister() {
            let reg = ServiceWorkerRegistration::new(
                "https://example.com/".to_string(),
                UpdateViaCache::Imports,
            );

            let worker = Arc::new(ServiceWorker::new("https://example.com/sw.js".to_string()));
            reg.start_install(Arc::clone(&worker));
            reg.complete_install().unwrap();
            reg.start_activate().unwrap();
            reg.complete_activate().unwrap();

            assert!(reg.unregister());
            assert!(reg.active().is_none());
            assert_eq!(worker.state(), ServiceWorkerState::Redundant);
        }

        #[test]
        fn test_install_failure() {
            let reg = ServiceWorkerRegistration::new(
                "https://example.com/".to_string(),
                UpdateViaCache::Imports,
            );

            let worker = Arc::new(ServiceWorker::new("https://example.com/sw.js".to_string()));
            reg.start_install(Arc::clone(&worker));

            reg.fail_install();
            assert!(reg.installing().is_none());
            assert_eq!(worker.state(), ServiceWorkerState::Redundant);
        }
    }

    // Container Tests
    mod container_tests {
        use super::*;

        #[test]
        fn test_create_container() {
            let container = ServiceWorkerContainer::new(test_origin());
            assert!(container.is_enabled());
            assert!(container.controller().is_none());
        }

        #[test]
        fn test_disabled_container() {
            let container = ServiceWorkerContainer::disabled();
            assert!(!container.is_enabled());
        }

        #[test]
        fn test_register_same_origin() {
            let container = ServiceWorkerContainer::new(test_origin());

            let result = container.register("https://example.com/sw.js", None);

            assert!(result.is_ok());
            let reg = result.unwrap();
            assert!(reg.scope().starts_with("https://example.com/"));
        }

        #[test]
        fn test_register_cross_origin_fails() {
            let container = ServiceWorkerContainer::new(test_origin());

            let result = container.register("https://other.com/sw.js", None);

            assert!(matches!(result, Err(ServiceWorkerError::SecurityError(_))));
        }

        #[test]
        fn test_register_with_scope() {
            let container = ServiceWorkerContainer::new(test_origin());

            let result = container.register(
                "https://example.com/app/sw.js",
                Some(RegistrationOptions {
                    scope: Some("https://example.com/app/".to_string()),
                    ..Default::default()
                }),
            );

            assert!(result.is_ok());
            let reg = result.unwrap();
            assert_eq!(reg.scope(), "https://example.com/app/");
        }

        #[test]
        fn test_get_registration() {
            let container = ServiceWorkerContainer::new(test_origin());
            container
                .register("https://example.com/sw.js", None)
                .unwrap();

            let reg = container.get_registration(Some("https://example.com/"));
            assert!(reg.is_some());
        }

        #[test]
        fn test_match_registration() {
            let container = ServiceWorkerContainer::new(test_origin());
            container
                .register(
                    "https://example.com/app/sw.js",
                    Some(RegistrationOptions {
                        scope: Some("https://example.com/app/".to_string()),
                        ..Default::default()
                    }),
                )
                .unwrap();

            let matched = container.match_registration("https://example.com/app/page.html");
            assert!(matched.is_some());

            let not_matched = container.match_registration("https://example.com/other/");
            assert!(not_matched.is_none());
        }
    }

    // Cache Tests
    mod cache_tests {
        use super::*;

        #[test]
        fn test_cache_put_and_match() {
            let cache = Cache::new("test-cache".to_string());
            let request = FetchRequest::new("https://example.com/data.json", RequestMethod::Get);
            let response = FetchResponse::new(200, b"test data".to_vec());

            cache.put(&request, response.clone()).unwrap();

            let matched = cache.match_request(&request);
            assert!(matched.is_some());
            assert_eq!(matched.unwrap().status, 200);
        }

        #[test]
        fn test_cache_match_url() {
            let cache = Cache::new("test-cache".to_string());
            let request = FetchRequest::new("https://example.com/data.json", RequestMethod::Get);
            let response = FetchResponse::new(200, b"test data".to_vec());

            cache.put(&request, response).unwrap();

            let matched = cache.match_url("https://example.com/data.json");
            assert!(matched.is_some());
        }

        #[test]
        fn test_cache_delete() {
            let cache = Cache::new("test-cache".to_string());
            let request = FetchRequest::new("https://example.com/data.json", RequestMethod::Get);
            let response = FetchResponse::new(200, b"test data".to_vec());

            cache.put(&request, response).unwrap();
            assert!(cache.delete(&request));
            assert!(cache.match_request(&request).is_none());
        }

        #[test]
        fn test_cache_keys() {
            let cache = Cache::new("test-cache".to_string());
            let request1 = FetchRequest::new("https://example.com/a.json", RequestMethod::Get);
            let request2 = FetchRequest::new("https://example.com/b.json", RequestMethod::Get);

            cache
                .put(&request1, FetchResponse::new(200, vec![]))
                .unwrap();
            cache
                .put(&request2, FetchResponse::new(200, vec![]))
                .unwrap();

            let keys = cache.keys();
            assert_eq!(keys.len(), 2);
        }

        #[test]
        fn test_cache_no_partial_response() {
            let cache = Cache::new("test-cache".to_string());
            let request = FetchRequest::new("https://example.com/data.json", RequestMethod::Get);
            let response = FetchResponse::new(206, vec![]);

            let result = cache.put(&request, response);
            assert!(matches!(result, Err(ServiceWorkerError::CacheError(_))));
        }
    }

    // Cache Storage Tests
    mod cache_storage_tests {
        use super::*;

        #[test]
        fn test_open_cache() {
            let storage = CacheStorage::new(test_origin());

            let cache = storage.open("v1");
            assert_eq!(cache.name(), "v1");
        }

        #[test]
        fn test_open_same_cache_twice() {
            let storage = CacheStorage::new(test_origin());

            let cache1 = storage.open("v1");
            let cache2 = storage.open("v1");

            // Should be the same cache
            assert!(Arc::ptr_eq(&cache1, &cache2));
        }

        #[test]
        fn test_has_cache() {
            let storage = CacheStorage::new(test_origin());

            assert!(!storage.has("v1"));
            storage.open("v1");
            assert!(storage.has("v1"));
        }

        #[test]
        fn test_delete_cache() {
            let storage = CacheStorage::new(test_origin());
            storage.open("v1");

            assert!(storage.delete("v1"));
            assert!(!storage.has("v1"));
        }

        #[test]
        fn test_cache_keys() {
            let storage = CacheStorage::new(test_origin());
            storage.open("v1");
            storage.open("v2");

            let keys = storage.keys();
            assert_eq!(keys.len(), 2);
        }

        #[test]
        fn test_match_across_caches() {
            let storage = CacheStorage::new(test_origin());
            let cache = storage.open("v1");

            let request = FetchRequest::new("https://example.com/data.json", RequestMethod::Get);
            let response = FetchResponse::new(200, b"test".to_vec());
            cache.put(&request, response).unwrap();

            let matched = storage.match_request(&request);
            assert!(matched.is_some());
        }
    }

    // Fetch Event Tests
    mod fetch_event_tests {
        use super::*;

        #[test]
        fn test_create_fetch_event() {
            let request = FetchRequest::navigate("https://example.com/page.html");
            let event = FetchEvent::new(request);

            assert!(!event.was_responded());
            assert!(event.get_response().is_none());
        }

        #[test]
        fn test_respond_with() {
            let request = FetchRequest::navigate("https://example.com/page.html");
            let event = FetchEvent::new(request);

            let response = FetchResponse::new(200, b"Hello".to_vec());
            event.respond_with(response).unwrap();

            assert!(event.was_responded());
            let resp = event.get_response().unwrap();
            assert_eq!(resp.status, 200);
        }

        #[test]
        fn test_respond_with_twice_fails() {
            let request = FetchRequest::navigate("https://example.com/page.html");
            let event = FetchEvent::new(request);

            event.respond_with(FetchResponse::new(200, vec![])).unwrap();
            let result = event.respond_with(FetchResponse::new(200, vec![]));

            assert!(result.is_err());
        }
    }

    // Response Tests
    mod response_tests {
        use super::*;

        #[test]
        fn test_response_ok() {
            let resp = FetchResponse::new(200, vec![]);
            assert!(resp.ok());

            let resp = FetchResponse::new(404, vec![]);
            assert!(!resp.ok());
        }

        #[test]
        fn test_error_response() {
            let resp = FetchResponse::error();
            assert_eq!(resp.status, 0);
            assert_eq!(resp.response_type, ResponseType::Error);
        }

        #[test]
        fn test_redirect_response() {
            let resp = FetchResponse::redirect("https://example.com/new", 302);
            assert_eq!(resp.status, 302);
            assert_eq!(
                resp.headers.get("Location").unwrap(),
                "https://example.com/new"
            );
        }
    }

    // Integration Tests
    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_lifecycle() {
            // Create container and storage
            let container = Arc::new(ServiceWorkerContainer::new(test_origin()));
            let storage = Arc::new(CacheStorage::new(test_origin()));

            // Register service worker
            let reg = container
                .register("https://example.com/sw.js", None)
                .unwrap();

            // Complete installation
            reg.complete_install().unwrap();

            // Activate
            reg.start_activate().unwrap();
            reg.complete_activate().unwrap();

            // Set controller
            container.set_controller(reg.active());

            // Cache a resource
            let cache = storage.open("v1");
            let request = FetchRequest::new("https://example.com/data.json", RequestMethod::Get);
            cache
                .put(&request, FetchResponse::new(200, b"cached data".to_vec()))
                .unwrap();

            // Create fetch handler
            let handler = FetchEventHandler::new(Arc::clone(&container), Arc::clone(&storage));

            // Handle fetch (should return cached response)
            let result = handler.handle(request);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().status, 200);
        }

        #[test]
        fn test_scope_matching() {
            let container = ServiceWorkerContainer::new(test_origin());

            // Register with specific scope
            container
                .register(
                    "https://example.com/app/sw.js",
                    Some(RegistrationOptions {
                        scope: Some("https://example.com/app/".to_string()),
                        ..Default::default()
                    }),
                )
                .unwrap();

            // Should match URLs within scope
            assert!(container
                .match_registration("https://example.com/app/index.html")
                .is_some());
            assert!(container
                .match_registration("https://example.com/app/sub/page.html")
                .is_some());

            // Should not match URLs outside scope
            assert!(container
                .match_registration("https://example.com/other/")
                .is_none());
            assert!(container
                .match_registration("https://example.com/")
                .is_none());
        }
    }
}
