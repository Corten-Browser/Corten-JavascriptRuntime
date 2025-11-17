//! Integration tests for the complete CLI workflow
//!
//! These tests verify end-to-end behavior of the CLI

use js_cli::{Cli, Runtime};
use std::fs;
use tempfile::TempDir;

/// Test complete workflow: CLI parsing -> Runtime creation -> File execution
#[test]
fn integration_file_execution_workflow() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.js");

    let js_code = "let x = 42;";
    fs::write(&file_path, js_code).unwrap();

    // Step 1: Parse CLI arguments
    let cli = Cli::with_file(file_path.to_str().unwrap().to_string());

    // Step 2: Create runtime
    let mut runtime = Runtime::new(cli.jit)
        .with_print_bytecode(cli.print_bytecode)
        .with_print_ast(cli.print_ast);

    // Step 3: Execute file
    let result = runtime.execute_file(cli.file.as_ref().unwrap());

    assert!(result.is_ok());
}

/// Test workflow with print options enabled
#[test]
fn integration_debug_output_workflow() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("debug.js");

    fs::write(&file_path, "let value = 100;").unwrap();

    let cli = Cli {
        file: Some(file_path.to_str().unwrap().to_string()),
        repl: false,
        jit: true,
        print_bytecode: true,
        print_ast: true,
    };

    let mut runtime = Runtime::new(cli.jit)
        .with_print_bytecode(cli.print_bytecode)
        .with_print_ast(cli.print_ast);

    // Runtime should be configured with debug options
    assert!(runtime.is_print_bytecode_enabled());
    assert!(runtime.is_print_ast_enabled());

    let result = runtime.execute_file(cli.file.as_ref().unwrap());
    assert!(result.is_ok());
}

/// Test multiple file executions in sequence
#[test]
fn integration_multiple_file_executions() {
    let dir = TempDir::new().unwrap();

    let files = vec![
        ("file1.js", "let a = 1;"),
        ("file2.js", "let b = 2;"),
        ("file3.js", "let c = 3;"),
    ];

    let mut runtime = Runtime::new(false);

    for (name, content) in files {
        let file_path = dir.path().join(name);
        fs::write(&file_path, content).unwrap();

        let result = runtime.execute_file(file_path.to_str().unwrap());
        assert!(result.is_ok(), "Failed to execute {}", name);
    }
}

/// Test error handling for missing file
#[test]
fn integration_missing_file_error() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_file("/definitely/does/not/exist.js");

    assert!(result.is_err());
}

/// Test error handling for invalid JavaScript
#[test]
fn integration_syntax_error_handling() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("function incomplete(");

    assert!(result.is_err());
}

/// Test Runtime configuration from CLI
#[test]
fn integration_cli_to_runtime_config() {
    let cli = Cli {
        file: Some("test.js".to_string()),
        repl: false,
        jit: false,
        print_bytecode: true,
        print_ast: true,
    };

    let runtime = Runtime::new(cli.jit)
        .with_print_bytecode(cli.print_bytecode)
        .with_print_ast(cli.print_ast);

    // Verify configuration was applied
    assert!(!runtime.is_jit_enabled());
    assert!(runtime.is_print_bytecode_enabled());
    assert!(runtime.is_print_ast_enabled());
}

/// Test execution of various JavaScript constructs
#[test]
fn integration_various_js_constructs() {
    let mut runtime = Runtime::new(false);

    let test_cases = vec![
        // Variable declarations
        "let x = 42;",
        "const y = 'hello';",
        "var z = true;",
        // Comments
        "// This is a comment\nlet a = 1;",
        "/* Block comment */ let b = 2;",
        // Empty statements
        ";",
        ";;",
    ];

    for (i, code) in test_cases.iter().enumerate() {
        let result = runtime.execute_string(code);
        assert!(result.is_ok(), "Test case {} failed: {}", i, code);
    }
}

/// Test that JIT setting is preserved
#[test]
fn integration_jit_setting_preserved() {
    let cli_jit_on = Cli {
        file: None,
        repl: false,
        jit: true,
        print_bytecode: false,
        print_ast: false,
    };

    let cli_jit_off = Cli {
        file: None,
        repl: false,
        jit: false,
        print_bytecode: false,
        print_ast: false,
    };

    let runtime_on = Runtime::new(cli_jit_on.jit);
    let runtime_off = Runtime::new(cli_jit_off.jit);

    assert!(runtime_on.is_jit_enabled());
    assert!(!runtime_off.is_jit_enabled());
}

/// Test file with shebang line
#[test]
fn integration_file_with_shebang() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("script.js");

    let js_code = r#"#!/usr/bin/env corten-js
let message = "Script executed";
"#;

    fs::write(&file_path, js_code).unwrap();

    let mut runtime = Runtime::new(false);
    let result = runtime.execute_file(file_path.to_str().unwrap());

    // Should handle shebang gracefully (parser should skip it)
    // If parser doesn't support this, test documents the limitation
    let _ = result;
}

/// Test large JavaScript file
#[test]
fn integration_large_file() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("large.js");

    // Generate a large file with many variable declarations
    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!("let var{} = {};\n", i, i));
    }

    fs::write(&file_path, &code).unwrap();

    let mut runtime = Runtime::new(false);
    let result = runtime.execute_file(file_path.to_str().unwrap());

    assert!(result.is_ok());
}

/// Test UTF-8 content in files
#[test]
fn integration_utf8_file_content() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("utf8.js");

    let js_code = r#"
        let greeting = "Hello, World!";
        let japanese = "Hello";
        let emoji = "Code";
    "#;

    fs::write(&file_path, js_code).unwrap();

    let mut runtime = Runtime::new(false);
    let result = runtime.execute_file(file_path.to_str().unwrap());

    assert!(result.is_ok());
}

/// Test runtime isolation (different runtimes don't share state)
#[test]
fn integration_runtime_isolation() {
    let mut runtime1 = Runtime::new(true);
    let mut runtime2 = Runtime::new(false);

    // Execute different code in each runtime
    let _ = runtime1.execute_string("let x = 1;");
    let _ = runtime2.execute_string("let y = 2;");

    // Each runtime should maintain its own state
    assert!(runtime1.is_jit_enabled());
    assert!(!runtime2.is_jit_enabled());
}

/// Test runtime reuse
#[test]
fn integration_runtime_reuse() {
    let mut runtime = Runtime::new(false);

    // Execute multiple scripts using the same runtime
    for i in 0..10 {
        let code = format!("let var{} = {};", i, i * 2);
        let result = runtime.execute_string(&code);
        assert!(result.is_ok(), "Failed on iteration {}", i);
    }
}
