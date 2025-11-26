//! Corten Benchmark CLI
//!
//! Command-line interface for running JavaScript benchmarks.

use benchmarks::{micro, runner, sunspider};
use std::process;

fn print_usage() {
    println!("Corten JavaScript Runtime Benchmark Tool");
    println!();
    println!("Usage:");
    println!("  corten-bench micro              Run micro-benchmarks");
    println!("  corten-bench sunspider          Run SunSpider suite");
    println!("  corten-bench all                Run all benchmarks");
    println!("  corten-bench --json <suite>     Output results as JSON");
    println!();
    println!("Examples:");
    println!("  corten-bench micro              # Run micro-benchmarks");
    println!("  corten-bench --json micro       # Output as JSON");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut json_output = false;
    let mut suite_name = "";

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_output = true;
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            suite => {
                suite_name = suite;
            }
        }
        i += 1;
    }

    if suite_name.is_empty() {
        eprintln!("Error: No benchmark suite specified");
        print_usage();
        process::exit(1);
    }

    // Run benchmarks based on suite
    let results = match suite_name {
        "micro" => {
            if !json_output {
                println!("Running micro-benchmarks...\n");
            }
            micro::run_all()
        }
        "sunspider" => {
            if !json_output {
                println!("Running SunSpider benchmark suite...\n");
            }
            let suite = sunspider::create_suite();
            let mut runtime = js_cli::Runtime::new(false);
            suite.run(&mut runtime)
        }
        "all" => {
            if !json_output {
                println!("Running all benchmarks...\n");
            }
            let mut all_results = Vec::new();

            if !json_output {
                println!("=== Micro-benchmarks ===\n");
            }
            all_results.extend(micro::run_all());

            if !json_output {
                println!("\n=== SunSpider ===\n");
            }
            let suite = sunspider::create_suite();
            let mut runtime = js_cli::Runtime::new(false);
            all_results.extend(suite.run(&mut runtime));

            all_results
        }
        _ => {
            eprintln!("Error: Unknown benchmark suite '{}'", suite_name);
            eprintln!("Valid suites: micro, sunspider, all");
            process::exit(1);
        }
    };

    // Output results
    if json_output {
        match runner::format_results_json(&results) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error formatting JSON: {}", e);
                process::exit(1);
            }
        }
    } else {
        println!("{}", runner::format_results(&results));

        // Summary statistics
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;
        let total_time: f64 = results.iter().map(|r| r.duration_ms).sum();

        println!("\nSummary:");
        println!("  Total benchmarks: {}", results.len());
        println!("  Successful: {}", successful);
        println!("  Failed: {}", failed);
        println!("  Total time: {:.2} ms ({:.2} s)", total_time, total_time / 1000.0);

        if failed > 0 {
            process::exit(1);
        }
    }
}
