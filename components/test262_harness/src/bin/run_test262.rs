//! Test262 Compliance Test Runner
//!
//! This binary provides a command-line interface to run Test262 conformance tests
//! against the Corten JavaScript Runtime using the test262_harness infrastructure.

use std::env;
use std::path::Path;
use std::time::Instant;
use test262_harness::{Test262Harness, TestReport};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command-line arguments
    let mut test_dir = "test262/test/language/expressions";
    let mut execute_mode = false;
    let mut limit: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--execute" | "-e" => {
                execute_mode = true;
                i += 1;
            }
            "--limit" | "-l" => {
                i += 1;
                if i < args.len() {
                    limit = args[i].parse().ok();
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: Unknown option: {}", arg);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
            _ => {
                test_dir = &args[i];
                i += 1;
            }
        }
    }

    // Validate test directory exists
    if !Path::new(test_dir).exists() {
        eprintln!("Error: Test directory not found: {}", test_dir);
        eprintln!("\nPlease ensure Test262 is cloned:");
        eprintln!("  git clone --depth 1 https://github.com/tc39/test262.git test262");
        std::process::exit(1);
    }

    // Print header
    println!("====================================");
    println!("Test262 Compliance Test Runner");
    println!("====================================");
    println!("Directory: {}", test_dir);
    println!("Mode: {}", if execute_mode { "Parse + Execute" } else { "Parse only" });
    if let Some(l) = limit {
        println!("Limit: {} tests", l);
    }
    println!();

    // Create harness and configure it
    let mut harness = Test262Harness::new();
    harness.set_execute(execute_mode);

    // Count total tests to run
    let total_available = count_tests(test_dir);
    let tests_to_run = limit.unwrap_or(total_available);
    println!("Found {} test files, running {}...\n", total_available, tests_to_run);

    // Run tests
    let start = Instant::now();
    let report = if let Some(l) = limit {
        run_with_limit(&mut harness, test_dir, l)
    } else {
        harness.run_directory(test_dir)
    };
    let duration = start.elapsed();

    // Print results
    print_report(&report, duration);

    // Exit with appropriate code
    if report.failed > 0 {
        std::process::exit(1);
    }
}

/// Print usage information
fn print_usage() {
    println!("Test262 Compliance Test Runner");
    println!();
    println!("USAGE:");
    println!("    run_test262 [OPTIONS] [DIRECTORY]");
    println!();
    println!("OPTIONS:");
    println!("    -e, --execute       Run in execution mode (parse + execute)");
    println!("    -l, --limit NUM     Limit number of tests to run");
    println!("    -h, --help          Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Run parse-only tests for expressions");
    println!("    run_test262 test262/test/language/expressions");
    println!();
    println!("    # Run parse + execute tests with limit");
    println!("    run_test262 --execute --limit 100 test262/test/language/statements");
    println!();
    println!("    # Test a specific feature");
    println!("    run_test262 test262/test/language/expressions/addition");
}

/// Count total number of test files in directory
fn count_tests(dir: &str) -> usize {
    use walkdir::WalkDir;

    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "js")
                .unwrap_or(false)
        })
        .count()
}

/// Run tests with a limit
fn run_with_limit(harness: &mut Test262Harness, dir: &str, limit: usize) -> TestReport {
    use test262_harness::TestFile;
    use walkdir::WalkDir;

    let mut report = TestReport::new();
    let mut count = 0;

    let walker = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "js")
                .unwrap_or(false)
        });

    for entry in walker {
        if count >= limit {
            break;
        }
        count += 1;

        let path = entry.path().to_string_lossy().to_string();

        // Show progress every 100 tests
        if count % 100 == 0 {
            print!("\rProcessed {} / {} tests...", count, limit);
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }

        match TestFile::load(&path) {
            Ok(test) => {
                let result = harness.run_test(&test);
                report.add_result(&path, result);
            }
            Err(e) => {
                use test262_harness::TestResult;
                let result = TestResult::Skip(format!("Could not load test: {}", e));
                report.add_result(&path, result);
            }
        }
    }

    if count > 0 {
        println!(); // Clear progress line
    }

    report
}

/// Print test report
fn print_report(report: &TestReport, duration: std::time::Duration) {
    println!("\n====================================");
    println!("RESULTS");
    println!("====================================");
    println!("Total:    {}", report.total);
    println!("Passed:   {} ({:.1}%)", report.passed, report.pass_rate());
    println!("Failed:   {} ({:.1}%)", report.failed, report.failure_rate());
    println!("Skipped:  {} ({:.1}%)", report.skipped, report.skip_rate());
    println!("Timeout:  {}", report.timeout);
    println!("Duration: {:.2}s", duration.as_secs_f64());
    println!();

    // Show sample of failures if any
    if !report.failures.is_empty() {
        println!("Sample failures (showing first 10):");
        for (i, (path, reason)) in report.failures.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, path);
            let truncated = if reason.len() > 80 {
                format!("{}...", &reason[..77])
            } else {
                reason.clone()
            };
            println!("     → {}", truncated);
        }

        if report.failures.len() > 10 {
            println!("  ... and {} more failures", report.failures.len() - 10);
        }
        println!();
    }

    // Overall assessment
    if report.passed == report.total {
        println!("✓ All tests passed!");
    } else if report.pass_rate() >= 90.0 {
        println!("✓ Excellent pass rate!");
    } else if report.pass_rate() >= 75.0 {
        println!("○ Good progress, some work remaining");
    } else if report.pass_rate() >= 50.0 {
        println!("○ Moderate compliance, significant work needed");
    } else {
        println!("✗ Low pass rate, major implementation work required");
    }
}
