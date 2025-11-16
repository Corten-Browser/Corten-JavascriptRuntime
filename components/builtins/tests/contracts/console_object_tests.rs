//! Contract tests for ConsoleObject

use builtins::{ConsoleObject, JsValue};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_console_log() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.log(&[JsValue::string("hello")]);

    assert_eq!(output.borrow().len(), 1);
    assert!(output.borrow()[0].contains("hello"));
}

#[test]
fn test_console_error() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.error(&[JsValue::string("error message")]);

    assert!(output.borrow().len() > 0);
}

#[test]
fn test_console_warn() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.warn(&[JsValue::string("warning")]);

    assert!(output.borrow().len() > 0);
}

#[test]
fn test_console_info() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.info(&[JsValue::string("info")]);

    assert!(output.borrow().len() > 0);
}

#[test]
fn test_console_debug() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.debug(&[JsValue::string("debug")]);

    assert!(output.borrow().len() > 0);
}

#[test]
fn test_console_assert_passing() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.assert(true, "should not print");

    // Passing assertion should not output anything
    assert_eq!(output.borrow().len(), 0);
}

#[test]
fn test_console_assert_failing() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.assert(false, "assertion failed");

    // Failing assertion should output
    assert!(output.borrow().len() > 0);
    assert!(output.borrow()[0].contains("assertion failed"));
}

#[test]
fn test_console_time_and_time_end() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let console = ConsoleObject::new_with_output(output.clone());

    console.time("test-timer");
    // Some operation
    std::thread::sleep(std::time::Duration::from_millis(10));
    console.time_end("test-timer");

    assert!(output.borrow().len() > 0);
    let last = &output.borrow()[output.borrow().len() - 1];
    assert!(last.contains("test-timer"));
}
