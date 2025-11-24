//! Same-Origin Policy (SOP) enforcement for JavaScript runtime
//!
//! Implements origin comparison per HTML specification section 7.5
//! https://html.spec.whatwg.org/multipage/origin.html

use std::fmt;

/// Represents an origin tuple (scheme, host, port)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin {
    /// URL scheme (e.g., "https", "http", "file")
    pub scheme: String,
    /// Host (e.g., "example.com", "localhost")
    pub host: String,
    /// Port number (None means default port for scheme)
    pub port: Option<u16>,
}

impl Origin {
    /// Create a new origin
    pub fn new(scheme: impl Into<String>, host: impl Into<String>, port: Option<u16>) -> Self {
        Self {
            scheme: scheme.into().to_lowercase(),
            host: host.into().to_lowercase(),
            port,
        }
    }

    /// Parse an origin from a URL string
    pub fn parse(url: &str) -> Result<Self, OriginError> {
        // Handle opaque origins
        if url == "null" {
            return Err(OriginError::OpaqueOrigin);
        }

        // Simple URL parsing
        let url = url.trim();

        // Extract scheme
        let (scheme, rest) = url
            .split_once("://")
            .ok_or_else(|| OriginError::InvalidUrl("Missing scheme".to_string()))?;

        // Extract host and optional port
        let authority = rest.split('/').next().unwrap_or(rest);
        let authority = authority.split('?').next().unwrap_or(authority);
        let authority = authority.split('#').next().unwrap_or(authority);

        // Handle userinfo (user:pass@host)
        let host_port = if let Some((_userinfo, hp)) = authority.split_once('@') {
            hp
        } else {
            authority
        };

        // Handle IPv6 addresses
        let (host, port) = if host_port.starts_with('[') {
            // IPv6 address
            if let Some((ipv6, port_str)) = host_port.rsplit_once("]:") {
                let host = format!("{}]", ipv6);
                let port = port_str
                    .parse::<u16>()
                    .map_err(|_| OriginError::InvalidUrl("Invalid port".to_string()))?;
                (host, Some(port))
            } else if host_port.ends_with(']') {
                (host_port.to_string(), None)
            } else {
                return Err(OriginError::InvalidUrl("Malformed IPv6 address".to_string()));
            }
        } else if let Some((h, p)) = host_port.rsplit_once(':') {
            // IPv4 or hostname with port
            let port = p
                .parse::<u16>()
                .map_err(|_| OriginError::InvalidUrl("Invalid port".to_string()))?;
            (h.to_string(), Some(port))
        } else {
            // No port specified
            (host_port.to_string(), None)
        };

        if host.is_empty() {
            return Err(OriginError::InvalidUrl("Empty host".to_string()));
        }

        Ok(Origin::new(scheme, host, port))
    }

    /// Get the effective port (resolving default ports)
    pub fn effective_port(&self) -> u16 {
        self.port.unwrap_or_else(|| default_port(&self.scheme))
    }

    /// Check if this origin is same-origin with another
    ///
    /// Two origins are same-origin if:
    /// 1. Their schemes are identical (case-insensitive)
    /// 2. Their hosts are identical (case-insensitive)
    /// 3. Their ports are identical (with defaults applied)
    pub fn is_same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme
            && self.host == other.host
            && self.effective_port() == other.effective_port()
    }

    /// Check if this origin is same-origin-domain with another
    /// (includes document.domain relaxation)
    pub fn is_same_origin_domain(&self, other: &Origin, domain: Option<&str>) -> bool {
        // If both have the same domain set, compare domains
        if let Some(d) = domain {
            let d = d.to_lowercase();
            return self.scheme == other.scheme
                && self.host.ends_with(&d)
                && other.host.ends_with(&d);
        }
        // Otherwise, fall back to same-origin
        self.is_same_origin(other)
    }

    /// Serialize origin to string
    pub fn serialize(&self) -> String {
        let default = default_port(&self.scheme);
        if self.port == Some(default) || self.port.is_none() {
            format!("{}://{}", self.scheme, self.host)
        } else {
            format!("{}://{}:{}", self.scheme, self.host, self.port.unwrap())
        }
    }

    /// Create an opaque origin (used for sandboxed iframes, data: URLs, etc.)
    pub fn opaque() -> OpaqueOrigin {
        OpaqueOrigin::new()
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

/// Default ports for common schemes
fn default_port(scheme: &str) -> u16 {
    match scheme {
        "http" => 80,
        "https" => 443,
        "ws" => 80,
        "wss" => 443,
        "ftp" => 21,
        _ => 0,
    }
}

/// Represents an opaque origin (unique, cannot be same-origin with anything)
#[derive(Debug, Clone)]
pub struct OpaqueOrigin {
    /// Unique identifier for this opaque origin
    id: u64,
}

impl OpaqueOrigin {
    /// Create a new opaque origin with a unique ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            id: COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }

    /// Opaque origins are only same-origin with themselves
    pub fn is_same_origin(&self, other: &OpaqueOrigin) -> bool {
        self.id == other.id
    }
}

impl Default for OpaqueOrigin {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during origin operations
#[derive(Debug, Clone, PartialEq)]
pub enum OriginError {
    /// The URL is invalid
    InvalidUrl(String),
    /// The origin is opaque (null)
    OpaqueOrigin,
    /// Cross-origin access denied
    CrossOriginDenied {
        source: String,
        target: String,
    },
}

impl fmt::Display for OriginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OriginError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            OriginError::OpaqueOrigin => write!(f, "Opaque origin"),
            OriginError::CrossOriginDenied { source, target } => {
                write!(
                    f,
                    "Blocked cross-origin request from {} to {}",
                    source, target
                )
            }
        }
    }
}

impl std::error::Error for OriginError {}

/// Same-Origin Policy enforcement context
pub struct SameOriginPolicy {
    /// The current document's origin
    current_origin: Origin,
    /// Whether cross-origin checks are enabled
    enabled: bool,
}

impl SameOriginPolicy {
    /// Create a new SOP enforcement context
    pub fn new(origin: Origin) -> Self {
        Self {
            current_origin: origin,
            enabled: true,
        }
    }

    /// Disable SOP checks (for testing or special contexts)
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable SOP checks
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Check if access to target origin is allowed
    pub fn check_access(&self, target: &Origin) -> Result<(), OriginError> {
        if !self.enabled {
            return Ok(());
        }

        if self.current_origin.is_same_origin(target) {
            Ok(())
        } else {
            Err(OriginError::CrossOriginDenied {
                source: self.current_origin.serialize(),
                target: target.serialize(),
            })
        }
    }

    /// Check if a URL is same-origin
    pub fn is_same_origin_url(&self, url: &str) -> bool {
        Origin::parse(url)
            .map(|o| self.current_origin.is_same_origin(&o))
            .unwrap_or(false)
    }

    /// Get the current origin
    pub fn origin(&self) -> &Origin {
        &self.current_origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_parse() {
        let origin = Origin::parse("https://example.com:443/path").unwrap();
        assert_eq!(origin.scheme, "https");
        assert_eq!(origin.host, "example.com");
        assert_eq!(origin.port, Some(443));
    }

    #[test]
    fn test_origin_parse_no_port() {
        let origin = Origin::parse("https://example.com").unwrap();
        assert_eq!(origin.scheme, "https");
        assert_eq!(origin.host, "example.com");
        assert_eq!(origin.port, None);
        assert_eq!(origin.effective_port(), 443);
    }

    #[test]
    fn test_same_origin() {
        let a = Origin::parse("https://example.com:443").unwrap();
        let b = Origin::parse("https://example.com").unwrap();
        assert!(a.is_same_origin(&b));
    }

    #[test]
    fn test_different_scheme() {
        let a = Origin::parse("https://example.com").unwrap();
        let b = Origin::parse("http://example.com").unwrap();
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_different_host() {
        let a = Origin::parse("https://example.com").unwrap();
        let b = Origin::parse("https://other.com").unwrap();
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_different_port() {
        let a = Origin::parse("https://example.com:443").unwrap();
        let b = Origin::parse("https://example.com:8443").unwrap();
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_case_insensitive() {
        let a = Origin::parse("HTTPS://EXAMPLE.COM").unwrap();
        let b = Origin::parse("https://example.com").unwrap();
        assert!(a.is_same_origin(&b));
    }

    #[test]
    fn test_opaque_origin() {
        let a = OpaqueOrigin::new();
        let b = OpaqueOrigin::new();
        assert!(a.is_same_origin(&a));
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_sop_check() {
        let origin = Origin::parse("https://example.com").unwrap();
        let sop = SameOriginPolicy::new(origin);

        let same = Origin::parse("https://example.com/other").unwrap();
        assert!(sop.check_access(&same).is_ok());

        let different = Origin::parse("https://other.com").unwrap();
        assert!(sop.check_access(&different).is_err());
    }

    #[test]
    fn test_serialize() {
        let origin = Origin::parse("https://example.com:443").unwrap();
        // Default port should be omitted
        assert_eq!(origin.serialize(), "https://example.com");

        let origin = Origin::parse("https://example.com:8443").unwrap();
        // Non-default port should be included
        assert_eq!(origin.serialize(), "https://example.com:8443");
    }

    #[test]
    fn test_ipv6_origin() {
        let origin = Origin::parse("https://[::1]:8080").unwrap();
        assert_eq!(origin.host, "[::1]");
        assert_eq!(origin.port, Some(8080));
    }
}
