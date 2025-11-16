//! CLI argument parsing tests
//!
//! Tests for verifying clap argument parsing works correctly

use clap::Parser as ClapParser;
use js_cli::Cli;

/// Test parsing no arguments (default behavior)
#[test]
fn cli_parse_no_args() {
    let args: Vec<&str> = vec!["corten-js"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, None);
    assert!(!cli.repl);
    assert!(cli.jit); // Default is true
    assert!(!cli.print_bytecode);
    assert!(!cli.print_ast);
}

/// Test parsing --file option
#[test]
fn cli_parse_file_long() {
    let args = vec!["corten-js", "--file", "script.js"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("script.js".to_string()));
}

/// Test parsing -f option (short form)
#[test]
fn cli_parse_file_short() {
    let args = vec!["corten-js", "-f", "script.js"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("script.js".to_string()));
}

/// Test parsing --repl option
#[test]
fn cli_parse_repl_long() {
    let args = vec!["corten-js", "--repl"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.repl);
}

/// Test parsing -r option (short form)
#[test]
fn cli_parse_repl_short() {
    let args = vec!["corten-js", "-r"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.repl);
}

/// Test parsing --jit option
#[test]
fn cli_parse_jit() {
    let args = vec!["corten-js", "--jit"];
    let cli = Cli::try_parse_from(args).unwrap();

    // JIT should still be true (default or explicit)
    assert!(cli.jit);
}

/// Test parsing --print-bytecode option
#[test]
fn cli_parse_print_bytecode() {
    let args = vec!["corten-js", "--print-bytecode"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.print_bytecode);
}

/// Test parsing --print-ast option
#[test]
fn cli_parse_print_ast() {
    let args = vec!["corten-js", "--print-ast"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.print_ast);
}

/// Test parsing multiple options together
#[test]
fn cli_parse_multiple_options() {
    let args = vec![
        "corten-js",
        "--file",
        "test.js",
        "--print-bytecode",
        "--print-ast",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("test.js".to_string()));
    assert!(cli.print_bytecode);
    assert!(cli.print_ast);
    assert!(cli.jit);
}

/// Test parsing file with path containing spaces
#[test]
fn cli_parse_file_with_spaces() {
    let args = vec!["corten-js", "-f", "path/to/my script.js"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("path/to/my script.js".to_string()));
}

/// Test parsing absolute file path
#[test]
fn cli_parse_absolute_path() {
    let args = vec!["corten-js", "--file", "/home/user/scripts/app.js"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("/home/user/scripts/app.js".to_string()));
}

/// Test that options are independent
#[test]
fn cli_options_independent() {
    // File and REPL can be combined (though typically mutually exclusive in practice)
    let args = vec!["corten-js", "-f", "test.js", "-r"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("test.js".to_string()));
    assert!(cli.repl);
}

/// Test parsing with all debug options enabled
#[test]
fn cli_parse_all_debug_options() {
    let args = vec![
        "corten-js",
        "-f",
        "debug.js",
        "--print-bytecode",
        "--print-ast",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.print_bytecode);
    assert!(cli.print_ast);
}

/// Test parsing preserves original file path format
#[test]
fn cli_preserves_file_path() {
    let test_paths = vec![
        "./local.js",
        "../parent/script.js",
        "~/home/script.js",
        "C:\\Windows\\script.js",
        "relative/path/to/file.js",
    ];

    for path in test_paths {
        let args = vec!["corten-js", "-f", path];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.file, Some(path.to_string()));
    }
}

/// Test parsing unknown option fails
#[test]
fn cli_parse_unknown_option_fails() {
    let args = vec!["corten-js", "--unknown-option"];
    let result = Cli::try_parse_from(args);

    assert!(result.is_err());
}

/// Test parsing missing file argument fails
#[test]
fn cli_parse_missing_file_arg_fails() {
    let args = vec!["corten-js", "--file"];
    let result = Cli::try_parse_from(args);

    assert!(result.is_err());
}

/// Test parsing duplicate options causes error (clap default behavior)
#[test]
fn cli_parse_duplicate_file_fails() {
    let args = vec!["corten-js", "-f", "first.js", "-f", "second.js"];
    let result = Cli::try_parse_from(args);

    // Duplicate args are not allowed by default in clap
    assert!(result.is_err());
}

/// Test that duplicate boolean flags cause error
#[test]
fn cli_parse_duplicate_boolean_flags_fails() {
    let args = vec!["corten-js", "--print-bytecode", "--print-bytecode"];
    let result = Cli::try_parse_from(args);

    // Duplicate flags are not allowed by default
    assert!(result.is_err());
}

/// Test parsing file with .mjs extension
#[test]
fn cli_parse_mjs_file() {
    let args = vec!["corten-js", "-f", "module.mjs"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("module.mjs".to_string()));
}

/// Test parsing file with .cjs extension
#[test]
fn cli_parse_cjs_file() {
    let args = vec!["corten-js", "-f", "module.cjs"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.file, Some("module.cjs".to_string()));
}

/// Test options order doesn't matter
#[test]
fn cli_options_order_independent() {
    let args1 = vec!["corten-js", "-f", "test.js", "--print-ast"];
    let args2 = vec!["corten-js", "--print-ast", "-f", "test.js"];

    let cli1 = Cli::try_parse_from(args1).unwrap();
    let cli2 = Cli::try_parse_from(args2).unwrap();

    assert_eq!(cli1.file, cli2.file);
    assert_eq!(cli1.print_ast, cli2.print_ast);
}
