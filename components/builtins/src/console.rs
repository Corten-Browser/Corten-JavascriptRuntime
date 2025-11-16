//! Console object methods

use crate::value::JsValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

/// Console output writer trait
pub trait ConsoleWriter {
    /// Write a message to the console output
    fn write(&self, message: &str);
}

/// Default console writer that prints to stdout
struct StdoutWriter;

impl ConsoleWriter for StdoutWriter {
    fn write(&self, message: &str) {
        println!("{}", message);
    }
}

/// Console object
pub struct ConsoleObject {
    output: Rc<RefCell<Vec<String>>>,
    timers: RefCell<HashMap<String, Instant>>,
    writer: Box<dyn ConsoleWriter>,
}

impl ConsoleObject {
    /// Create a new console with default stdout output
    pub fn new() -> Self {
        ConsoleObject {
            output: Rc::new(RefCell::new(Vec::new())),
            timers: RefCell::new(HashMap::new()),
            writer: Box::new(StdoutWriter),
        }
    }

    /// Create a console with custom output capture
    pub fn new_with_output(output: Rc<RefCell<Vec<String>>>) -> Self {
        let captured_output = output.clone();
        ConsoleObject {
            output,
            timers: RefCell::new(HashMap::new()),
            writer: Box::new(CaptureWriter { output: captured_output }),
        }
    }

    /// Format values for output
    fn format_values(values: &[JsValue]) -> String {
        values
            .iter()
            .map(|v| v.to_js_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// console.log(...values)
    pub fn log(&self, values: &[JsValue]) {
        let message = Self::format_values(values);
        self.output.borrow_mut().push(message.clone());
        self.writer.write(&message);
    }

    /// console.error(...values)
    pub fn error(&self, values: &[JsValue]) {
        let message = format!("Error: {}", Self::format_values(values));
        self.output.borrow_mut().push(message.clone());
        self.writer.write(&message);
    }

    /// console.warn(...values)
    pub fn warn(&self, values: &[JsValue]) {
        let message = format!("Warning: {}", Self::format_values(values));
        self.output.borrow_mut().push(message.clone());
        self.writer.write(&message);
    }

    /// console.info(...values)
    pub fn info(&self, values: &[JsValue]) {
        let message = format!("Info: {}", Self::format_values(values));
        self.output.borrow_mut().push(message.clone());
        self.writer.write(&message);
    }

    /// console.debug(...values)
    pub fn debug(&self, values: &[JsValue]) {
        let message = format!("Debug: {}", Self::format_values(values));
        self.output.borrow_mut().push(message.clone());
        self.writer.write(&message);
    }

    /// console.assert(condition, message)
    pub fn assert(&self, condition: bool, message: &str) {
        if !condition {
            let msg = format!("Assertion failed: {}", message);
            self.output.borrow_mut().push(msg.clone());
            self.writer.write(&msg);
        }
    }

    /// console.time(label)
    pub fn time(&self, label: &str) {
        self.timers.borrow_mut().insert(label.to_string(), Instant::now());
    }

    /// console.timeEnd(label)
    pub fn time_end(&self, label: &str) {
        if let Some(start) = self.timers.borrow_mut().remove(label) {
            let elapsed = start.elapsed();
            let message = format!("{}: {}ms", label, elapsed.as_millis());
            self.output.borrow_mut().push(message.clone());
            self.writer.write(&message);
        }
    }
}

impl Default for ConsoleObject {
    fn default() -> Self {
        Self::new()
    }
}

/// Writer that captures output to a vector
struct CaptureWriter {
    #[allow(dead_code)]
    output: Rc<RefCell<Vec<String>>>,
}

impl ConsoleWriter for CaptureWriter {
    fn write(&self, _message: &str) {
        // Output is already captured in ConsoleObject methods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.log(&[JsValue::string("hello")]);

        assert_eq!(output.borrow().len(), 1);
        assert_eq!(output.borrow()[0], "hello");
    }

    #[test]
    fn test_log_multiple_values() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.log(&[JsValue::string("a"), JsValue::number(1.0), JsValue::boolean(true)]);

        assert_eq!(output.borrow()[0], "a 1 true");
    }

    #[test]
    fn test_error() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.error(&[JsValue::string("error message")]);

        assert!(output.borrow()[0].contains("Error:"));
        assert!(output.borrow()[0].contains("error message"));
    }

    #[test]
    fn test_warn() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.warn(&[JsValue::string("warning")]);

        assert!(output.borrow()[0].contains("Warning:"));
    }

    #[test]
    fn test_info() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.info(&[JsValue::string("info")]);

        assert!(output.borrow()[0].contains("Info:"));
    }

    #[test]
    fn test_debug() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.debug(&[JsValue::string("debug")]);

        assert!(output.borrow()[0].contains("Debug:"));
    }

    #[test]
    fn test_assert_passing() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.assert(true, "should not print");

        assert_eq!(output.borrow().len(), 0);
    }

    #[test]
    fn test_assert_failing() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.assert(false, "assertion failed");

        assert!(output.borrow().len() > 0);
        assert!(output.borrow()[0].contains("assertion failed"));
    }

    #[test]
    fn test_time_and_time_end() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.time("test-timer");
        std::thread::sleep(std::time::Duration::from_millis(10));
        console.time_end("test-timer");

        assert!(output.borrow().len() > 0);
        let last = &output.borrow()[output.borrow().len() - 1];
        assert!(last.contains("test-timer"));
        assert!(last.contains("ms"));
    }
}
