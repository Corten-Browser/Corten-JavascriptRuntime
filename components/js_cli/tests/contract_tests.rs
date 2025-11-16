//! Contract tests for js_cli component
//!
//! These tests verify that the component meets its contract specification:
//! - Runtime struct with new, execute_file, execute_string, repl methods
//! - CLI argument parsing with all required options
//! - Proper error handling

use js_cli::{Cli, CliError, CliResult, Runtime};
use std::fs;

/// Test Runtime::new with JIT enabled
#[test]
fn contract_runtime_new_with_jit_enabled() {
    let runtime = Runtime::new(true);
    assert!(runtime.is_jit_enabled());
}

/// Test Runtime::new with JIT disabled
#[test]
fn contract_runtime_new_with_jit_disabled() {
    let runtime = Runtime::new(false);
    assert!(!runtime.is_jit_enabled());
}

/// Test Runtime builder pattern
#[test]
fn contract_runtime_builder_pattern() {
    let runtime = Runtime::new(true)
        .with_print_bytecode(true)
        .with_print_ast(true);

    assert!(runtime.is_jit_enabled());
    assert!(runtime.is_print_bytecode_enabled());
    assert!(runtime.is_print_ast_enabled());
}

/// Test Runtime::execute_string with simple expression
#[test]
fn contract_runtime_execute_string_simple() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("let x = 42;");
    assert!(result.is_ok());
}

/// Test Runtime::execute_file with valid file
#[test]
fn contract_runtime_execute_file_valid() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.js");

    fs::write(&file_path, "let x = 42;").unwrap();

    let mut runtime = Runtime::new(false);
    let result = runtime.execute_file(file_path.to_str().unwrap());

    assert!(result.is_ok());
}

/// Test Runtime::execute_file with non-existent file
#[test]
fn contract_runtime_execute_file_not_found() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_file("/nonexistent/path/to/file.js");

    assert!(result.is_err());
    match result {
        Err(CliError::IoError(_)) => {}
        _ => panic!("Expected IoError for non-existent file"),
    }
}

/// Test Runtime::execute_string with syntax error
#[test]
fn contract_runtime_execute_string_parse_error() {
    let mut runtime = Runtime::new(false);
    // Invalid JavaScript syntax
    let result = runtime.execute_string("let x = {");

    assert!(result.is_err());
}

/// Test CLI struct creation
#[test]
fn contract_cli_default_creation() {
    let cli = Cli::new();

    assert_eq!(cli.file, None);
    assert!(!cli.repl);
    assert!(cli.jit);
    assert!(!cli.print_bytecode);
    assert!(!cli.print_ast);
}

/// Test CLI with file option
#[test]
fn contract_cli_with_file() {
    let cli = Cli::with_file("test.js".to_string());

    assert_eq!(cli.file, Some("test.js".to_string()));
    assert!(!cli.repl);
    assert!(cli.jit);
}

/// Test CLI with REPL mode
#[test]
fn contract_cli_with_repl() {
    let cli = Cli::with_repl();

    assert_eq!(cli.file, None);
    assert!(cli.repl);
    assert!(cli.jit);
}

/// Test CLI Default trait
#[test]
fn contract_cli_default_trait() {
    let cli: Cli = Default::default();

    assert_eq!(cli.file, None);
    assert!(!cli.repl);
    assert!(cli.jit);
}

/// Test that Runtime returns Value type
#[test]
fn contract_runtime_returns_value() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("let x = 42;");

    match result {
        Ok(value) => {
            // Value should be from core_types
            let _ = value; // This compiles if Value is the correct type
        }
        Err(_) => {}
    }
}

/// Test error types
#[test]
fn contract_error_types() {
    // Test that all error types are properly defined
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let cli_error: CliError = io_error.into();
    match cli_error {
        CliError::IoError(_) => {}
        _ => panic!("Expected IoError"),
    }

    let parse_error = CliError::ParseError("test".to_string());
    match parse_error {
        CliError::ParseError(msg) => assert_eq!(msg, "test"),
        _ => panic!("Expected ParseError"),
    }

    let repl_error = CliError::ReplError("test".to_string());
    match repl_error {
        CliError::ReplError(msg) => assert_eq!(msg, "test"),
        _ => panic!("Expected ReplError"),
    }
}

/// Test CliResult type alias
#[test]
fn contract_cli_result_type() {
    let success: CliResult<i32> = Ok(42);
    assert_eq!(success.unwrap(), 42);

    let failure: CliResult<i32> = Err(CliError::ParseError("test".to_string()));
    assert!(failure.is_err());
}

/// Test Runtime with bytecode printing disabled by default
#[test]
fn contract_runtime_default_options() {
    let runtime = Runtime::new(true);

    assert!(runtime.is_jit_enabled());
    assert!(!runtime.is_print_bytecode_enabled());
    assert!(!runtime.is_print_ast_enabled());
}

/// Test multiple script executions
#[test]
fn contract_runtime_multiple_executions() {
    let mut runtime = Runtime::new(false);

    // Execute multiple scripts
    let _ = runtime.execute_string("let a = 1;");
    let _ = runtime.execute_string("let b = 2;");
    let _ = runtime.execute_string("let c = 3;");

    // Runtime should handle multiple executions
}

/// Test empty source code
#[test]
fn contract_runtime_empty_source() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("");

    // Empty source should either succeed (returning undefined) or fail gracefully
    // depending on parser implementation
    let _ = result;
}

/// Test whitespace-only source
#[test]
fn contract_runtime_whitespace_source() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("   \n\t  ");

    // Whitespace-only should be handled gracefully
    let _ = result;
}

/// Test CLI clone implementation
#[test]
fn contract_cli_clone() {
    let cli1 = Cli::with_file("test.js".to_string());
    let cli2 = cli1.clone();

    assert_eq!(cli1.file, cli2.file);
    assert_eq!(cli1.repl, cli2.repl);
    assert_eq!(cli1.jit, cli2.jit);
}

/// Test CLI Debug implementation
#[test]
fn contract_cli_debug() {
    let cli = Cli::new();
    let debug_str = format!("{:?}", cli);

    assert!(debug_str.contains("Cli"));
}

/// Test error Display implementation
#[test]
fn contract_error_display() {
    let io_error = CliError::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found",
    ));
    let display_str = format!("{}", io_error);
    assert!(display_str.contains("File error"));

    let parse_error = CliError::ParseError("syntax error".to_string());
    let display_str = format!("{}", parse_error);
    assert!(display_str.contains("Parse error"));

    let repl_error = CliError::ReplError("readline error".to_string());
    let display_str = format!("{}", repl_error);
    assert!(display_str.contains("REPL error"));
}

/// Test that execute_file reads actual file content
#[test]
fn contract_execute_file_reads_content() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("script.js");

    let js_code = r#"
        let message = "Hello, World!";
        let count = 10;
    "#;

    fs::write(&file_path, js_code).unwrap();

    let mut runtime = Runtime::new(false);
    let result = runtime.execute_file(file_path.to_str().unwrap());

    // Should successfully parse and execute the multi-line script
    assert!(result.is_ok());
}

/// Test Runtime state preservation
#[test]
fn contract_runtime_state_independent() {
    let runtime1 = Runtime::new(true);
    let runtime2 = Runtime::new(false);

    // Each runtime should have independent state
    assert!(runtime1.is_jit_enabled());
    assert!(!runtime2.is_jit_enabled());
}
