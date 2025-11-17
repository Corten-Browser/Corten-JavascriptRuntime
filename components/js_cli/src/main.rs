//! Corten JavaScript Runtime CLI
//!
//! Entry point for the JavaScript runtime. Parses CLI arguments and
//! delegates to the Runtime for execution.

use clap::Parser as ClapParser;
use js_cli::{Cli, CliError, Runtime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut runtime = Runtime::new(cli.jit)
        .with_print_bytecode(cli.print_bytecode)
        .with_print_ast(cli.print_ast);

    // Execute based on CLI arguments
    if let Some(file) = cli.file {
        match runtime.execute_file(&file) {
            Ok(result) => {
                // Print result if not undefined
                if !matches!(result, core_types::Value::Undefined) {
                    println!("{:?}", result);
                }
            }
            Err(CliError::IoError(e)) => {
                eprintln!("Error: Could not read file '{}': {}", file, e);
                std::process::exit(1);
            }
            Err(CliError::ParseError(e)) => {
                eprintln!("Syntax Error: {}", e);
                std::process::exit(1);
            }
            Err(CliError::JsError(e)) => {
                eprintln!("JavaScript Error: {:?}", e);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else if let Some(code) = cli.eval {
        match runtime.execute_string(&code) {
            Ok(result) => {
                // Print result if not undefined
                if !matches!(result, core_types::Value::Undefined) {
                    println!("{:?}", result);
                }
            }
            Err(CliError::ParseError(e)) => {
                eprintln!("Syntax Error: {}", e);
                std::process::exit(1);
            }
            Err(CliError::JsError(e)) => {
                eprintln!("JavaScript Error: {:?}", e);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else if cli.repl {
        runtime.repl()?;
    } else {
        // Default: show usage
        println!("Corten JavaScript Runtime v0.1.0");
        println!();
        println!("Usage:");
        println!("  corten-js --file <FILE>     Execute a JavaScript file");
        println!("  corten-js --eval <CODE>     Evaluate inline JavaScript code");
        println!("  corten-js --repl            Start interactive REPL");
        println!();
        println!("Run 'corten-js --help' for more options.");
    }

    Ok(())
}
