//! Chrome DevTools Protocol implementation
//!
//! Provides debugging capabilities through the Chrome DevTools Protocol,
//! enabling remote debugging, profiling, and inspection of JavaScript code.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, json};

/// Debug protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub id: Option<u64>,
    pub method: Option<String>,
    pub params: Option<JsonValue>,
    pub result: Option<JsonValue>,
    pub error: Option<ProtocolError>,
}

/// DevTools protocol error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: i32,
    pub message: String,
}

/// Breakpoint information
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: String,
    pub script_id: String,
    pub line_number: u32,
    pub column_number: Option<u32>,
    pub condition: Option<String>,
    pub enabled: bool,
}

/// Call frame during debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    pub call_frame_id: String,
    pub function_name: String,
    pub location: Location,
    pub scope_chain: Vec<Scope>,
}

/// Source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub script_id: String,
    pub line_number: u32,
    pub column_number: u32,
}

/// Variable scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    #[serde(rename = "type")]
    pub scope_type: String,  // "global", "local", "closure"
    pub object: RemoteObject,
}

/// Remote object representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteObject {
    #[serde(rename = "type")]
    pub object_type: String,  // "object", "function", "undefined", etc.
    pub value: Option<JsonValue>,
    pub description: Option<String>,
    pub object_id: Option<String>,
}

/// DevTools Protocol Server
pub struct DevToolsServer {
    breakpoints: HashMap<String, Breakpoint>,
    paused: bool,
    call_stack: Vec<CallFrame>,
    scripts: HashMap<String, String>,  // script_id -> source
    next_script_id: u64,
    next_breakpoint_id: u64,
    next_object_id: u64,
}

impl DevToolsServer {
    /// Create a new DevTools server
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            paused: false,
            call_stack: Vec::new(),
            scripts: HashMap::new(),
            next_script_id: 1,
            next_breakpoint_id: 1,
            next_object_id: 1,
        }
    }

    /// Handle incoming protocol message
    pub fn handle_message(&mut self, message: &ProtocolMessage) -> ProtocolMessage {
        let method = message.method.as_deref().unwrap_or("");

        match method {
            "Debugger.enable" => self.debugger_enable(message),
            "Debugger.setBreakpoint" => self.debugger_set_breakpoint(message),
            "Debugger.removeBreakpoint" => self.debugger_remove_breakpoint(message),
            "Debugger.resume" => self.debugger_resume(message),
            "Debugger.stepOver" => self.debugger_step_over(message),
            "Debugger.stepInto" => self.debugger_step_into(message),
            "Debugger.stepOut" => self.debugger_step_out(message),
            "Debugger.pause" => self.debugger_pause(message),
            "Runtime.evaluate" => self.runtime_evaluate(message),
            "Runtime.getProperties" => self.runtime_get_properties(message),
            _ => self.method_not_found(message),
        }
    }

    fn debugger_enable(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({})),
            error: None,
        }
    }

    fn debugger_set_breakpoint(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        let params = msg.params.as_ref().unwrap();
        let location = &params["location"];

        let bp_id = format!("bp_{}", self.next_breakpoint_id);
        self.next_breakpoint_id += 1;

        let breakpoint = Breakpoint {
            id: bp_id.clone(),
            script_id: location["scriptId"].as_str().unwrap_or("").to_string(),
            line_number: location["lineNumber"].as_u64().unwrap_or(0) as u32,
            column_number: location["columnNumber"].as_u64().map(|n| n as u32),
            condition: params.get("condition").and_then(|c| c.as_str()).map(String::from),
            enabled: true,
        };

        self.breakpoints.insert(bp_id.clone(), breakpoint);

        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({
                "breakpointId": bp_id,
                "actualLocation": location
            })),
            error: None,
        }
    }

    fn debugger_remove_breakpoint(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        if let Some(params) = &msg.params {
            if let Some(bp_id) = params["breakpointId"].as_str() {
                self.breakpoints.remove(bp_id);
            }
        }

        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({})),
            error: None,
        }
    }

    fn debugger_resume(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        self.paused = false;
        self.call_stack.clear();

        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({})),
            error: None,
        }
    }

    fn debugger_step_over(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        // Step to next statement
        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({})),
            error: None,
        }
    }

    fn debugger_step_into(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        // Step into function call
        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({})),
            error: None,
        }
    }

    fn debugger_step_out(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        // Step out of current function
        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({})),
            error: None,
        }
    }

    fn debugger_pause(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        self.paused = true;

        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({})),
            error: None,
        }
    }

    fn runtime_evaluate(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        let expression = msg.params.as_ref()
            .and_then(|p| p["expression"].as_str())
            .unwrap_or("");

        // Simplified: evaluate as number or string
        let result = if let Ok(num) = expression.parse::<f64>() {
            RemoteObject {
                object_type: "number".to_string(),
                value: Some(json!(num)),
                description: Some(num.to_string()),
                object_id: None,
            }
        } else {
            RemoteObject {
                object_type: "string".to_string(),
                value: Some(json!(expression)),
                description: Some(format!("\"{}\"", expression)),
                object_id: None,
            }
        };

        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({ "result": result })),
            error: None,
        }
    }

    fn runtime_get_properties(&mut self, msg: &ProtocolMessage) -> ProtocolMessage {
        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: Some(json!({ "result": [] })),
            error: None,
        }
    }

    fn method_not_found(&self, msg: &ProtocolMessage) -> ProtocolMessage {
        ProtocolMessage {
            id: msg.id,
            method: None,
            params: None,
            result: None,
            error: Some(ProtocolError {
                code: -32601,
                message: "Method not found".to_string(),
            }),
        }
    }

    // Public accessors

    /// Check if execution is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get all breakpoints
    pub fn breakpoints(&self) -> &HashMap<String, Breakpoint> {
        &self.breakpoints
    }

    /// Get current call stack
    pub fn call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    /// Add script and return its ID
    pub fn add_script(&mut self, source: String) -> String {
        let id = format!("script_{}", self.next_script_id);
        self.next_script_id += 1;
        self.scripts.insert(id.clone(), source);
        id
    }

    /// Get script source by ID
    pub fn get_script(&self, script_id: &str) -> Option<&String> {
        self.scripts.get(script_id)
    }

    /// Check if we should pause at location
    pub fn should_pause_at(&self, script_id: &str, line: u32) -> bool {
        self.breakpoints.values().any(|bp| {
            bp.enabled && bp.script_id == script_id && bp.line_number == line
        })
    }

    /// Push a call frame onto the stack
    pub fn push_call_frame(&mut self, frame: CallFrame) {
        self.call_stack.push(frame);
    }

    /// Pop a call frame from the stack
    pub fn pop_call_frame(&mut self) -> Option<CallFrame> {
        self.call_stack.pop()
    }

    /// Set paused state
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Get the next object ID for remote objects
    pub fn next_object_id(&mut self) -> String {
        let id = format!("obj_{}", self.next_object_id);
        self.next_object_id += 1;
        id
    }
}

impl Default for DevToolsServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for convenience
pub type DebugProtocol = DevToolsServer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = DevToolsServer::new();
        assert!(!server.is_paused());
        assert!(server.breakpoints().is_empty());
        assert!(server.call_stack().is_empty());
    }

    #[test]
    fn test_default_creation() {
        let server = DevToolsServer::default();
        assert!(!server.is_paused());
    }
}
