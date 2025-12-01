//! REPL (Read-Eval-Print Loop) implementation

use crate::error::{CliError, CliResult};
use crate::runtime::Runtime;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Run the interactive REPL
///
/// # Arguments
/// * `runtime` - The Runtime instance to use for execution
///
/// # Returns
/// `Ok(())` when REPL exits normally
pub fn run_repl(runtime: &mut Runtime) -> CliResult<()> {
    let mut editor = DefaultEditor::new()
        .map_err(|e| CliError::ReplError(format!("Failed to initialize editor: {}", e)))?;

    println!("Corten JavaScript Runtime v0.1.0");
    println!("Type JavaScript code or 'exit' to quit.");
    println!();

    let mut line_buffer = String::new();
    let mut in_multiline = false;

    loop {
        let prompt = if in_multiline { "... " } else { "> " };

        match editor.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();

                // Check for exit commands
                if !in_multiline && (trimmed == "exit" || trimmed == ".exit" || trimmed == "quit") {
                    println!("Goodbye!");
                    break;
                }

                // Handle special REPL commands
                if !in_multiline && trimmed.starts_with('.') {
                    handle_repl_command(trimmed, runtime);
                    continue;
                }

                // Accumulate input
                if in_multiline {
                    line_buffer.push('\n');
                }
                line_buffer.push_str(&line);

                // Check if input is complete (simple heuristic)
                if is_input_complete(&line_buffer) {
                    in_multiline = false;

                    // Add to history
                    let _ = editor.add_history_entry(&line_buffer);

                    // Execute and print result
                    match runtime.execute_string(&line_buffer) {
                        Ok(value) => {
                            println!("{}", format_value(&value));
                        }
                        Err(CliError::ParseError(e)) => {
                            // Check if it's an incomplete input error
                            if e.contains("Unexpected end of input") {
                                in_multiline = true;
                                continue;
                            }
                            eprintln!("Error: {}", e);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }

                    line_buffer.clear();
                } else {
                    in_multiline = true;
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                if in_multiline {
                    println!("^C");
                    line_buffer.clear();
                    in_multiline = false;
                } else {
                    println!("Press Ctrl-D or type 'exit' to quit");
                }
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                println!("\nGoodbye!");
                break;
            }
            Err(err) => {
                return Err(CliError::ReplError(format!("Readline error: {}", err)));
            }
        }
    }

    Ok(())
}

/// Handle special REPL commands
fn handle_repl_command(command: &str, runtime: &Runtime) {
    match command {
        ".help" => {
            println!("REPL Commands:");
            println!("  .help     - Show this help message");
            println!("  .clear    - Clear the screen");
            println!("  .jit      - Show JIT status");
            println!("  .exit     - Exit the REPL");
            println!("  exit      - Exit the REPL");
            println!("  quit      - Exit the REPL");
        }
        ".clear" => {
            print!("\x1B[2J\x1B[1;1H");
        }
        ".jit" => {
            println!(
                "JIT compilation: {}",
                if runtime.is_jit_enabled() {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Type .help for available commands");
        }
    }
}

/// Check if the input appears to be complete
///
/// This is a simple heuristic that checks for balanced braces/brackets/parens
fn is_input_complete(input: &str) -> bool {
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut paren_count = 0;
    let mut in_string = false;
    let mut string_char = ' ';
    let mut escape_next = false;

    for c in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            escape_next = true;
            continue;
        }

        if !in_string {
            match c {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = c;
                }
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                _ => {}
            }
        } else if c == string_char {
            in_string = false;
        }
    }

    brace_count == 0 && bracket_count == 0 && paren_count == 0 && !in_string
}

/// Format a JavaScript value for display
fn format_value(value: &core_types::Value) -> String {
    match value {
        core_types::Value::Undefined => "undefined".to_string(),
        core_types::Value::Null => "null".to_string(),
        core_types::Value::Boolean(b) => b.to_string(),
        core_types::Value::Smi(n) => n.to_string(),
        core_types::Value::Double(f) => {
            if f.is_nan() {
                "NaN".to_string()
            } else if f.is_infinite() {
                if *f > 0.0 {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else {
                f.to_string()
            }
        }
        core_types::Value::HeapObject(_) => "[Object]".to_string(),
        core_types::Value::String(s) => format!("'{}'", s),
        core_types::Value::NativeObject(_) => "[native object]".to_string(),
        core_types::Value::NativeFunction(name) => format!("[Function: {}]", name),
        core_types::Value::BigInt(n) => format!("{}n", n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_input_complete_simple() {
        assert!(is_input_complete("let x = 42;"));
        assert!(is_input_complete("console.log('hello');"));
    }

    #[test]
    fn test_is_input_complete_incomplete_brace() {
        assert!(!is_input_complete("function test() {"));
        assert!(!is_input_complete("if (true) {"));
    }

    #[test]
    fn test_is_input_complete_with_blocks() {
        assert!(is_input_complete("function test() { return 42; }"));
        assert!(is_input_complete("if (true) { console.log('yes'); }"));
    }

    #[test]
    fn test_is_input_complete_with_strings() {
        assert!(is_input_complete(r#"let s = "hello {"; "#));
        assert!(!is_input_complete(r#"let s = "unclosed"#));
    }

    #[test]
    fn test_format_value_undefined() {
        let value = core_types::Value::Undefined;
        assert_eq!(format_value(&value), "undefined");
    }

    #[test]
    fn test_format_value_null() {
        let value = core_types::Value::Null;
        assert_eq!(format_value(&value), "null");
    }

    #[test]
    fn test_format_value_boolean() {
        assert_eq!(format_value(&core_types::Value::Boolean(true)), "true");
        assert_eq!(format_value(&core_types::Value::Boolean(false)), "false");
    }

    #[test]
    fn test_format_value_number() {
        assert_eq!(format_value(&core_types::Value::Smi(42)), "42");
        assert_eq!(format_value(&core_types::Value::Double(3.14)), "3.14");
    }

    #[test]
    fn test_format_value_special_floats() {
        assert_eq!(format_value(&core_types::Value::Double(f64::NAN)), "NaN");
        assert_eq!(
            format_value(&core_types::Value::Double(f64::INFINITY)),
            "Infinity"
        );
        assert_eq!(
            format_value(&core_types::Value::Double(f64::NEG_INFINITY)),
            "-Infinity"
        );
    }
}
