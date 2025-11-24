//! Test262 Baseline Runner
//!
//! Runs Test262 tests using the Corten JavaScript parser and interpreter.

use interpreter::VM;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

/// Test result statistics
#[derive(Default)]
struct Stats {
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    /// Failed tests grouped by error type
    errors: HashMap<String, Vec<String>>,
}

/// Test metadata from YAML frontmatter
struct TestMetadata {
    is_negative_parse: bool,
    is_module: bool,
    features: Vec<String>,
    expected_error: Option<String>,
}

impl TestMetadata {
    fn parse(source: &str) -> Self {
        let mut is_negative_parse = false;
        let mut is_module = false;
        let mut features = Vec::new();
        let mut expected_error = None;

        // Extract YAML frontmatter
        if let Some(start) = source.find("/*---") {
            if let Some(end) = source.find("---*/") {
                let yaml = &source[start + 5..end];

                // Check for negative parse test
                if yaml.contains("phase: parse") && yaml.contains("type:") {
                    is_negative_parse = true;
                    // Extract expected error type
                    for line in yaml.lines() {
                        if line.trim().starts_with("type:") {
                            expected_error = Some(line.trim()["type:".len()..].trim().to_string());
                        }
                    }
                }

                // Check for module
                if yaml.contains("module") {
                    is_module = true;
                }

                // Extract features
                let mut in_features = false;
                for line in yaml.lines() {
                    if line.trim().starts_with("features:") {
                        in_features = true;
                        continue;
                    }
                    if in_features {
                        if line.trim().starts_with("- ") {
                            features.push(line.trim()[2..].to_string());
                        } else if !line.starts_with(' ') && !line.starts_with('\t') {
                            in_features = false;
                        }
                    }
                }
            }
        }

        TestMetadata {
            is_negative_parse,
            is_module,
            features,
            expected_error,
        }
    }
}

/// Features we don't support yet
const UNSUPPORTED_FEATURES: &[&str] = &[
    "regexp-unicode-property-escapes",
    "regexp-match-indices",
    "regexp-named-groups",
    "regexp-lookbehind",
    "regexp-dotall",
    "class-fields-private",
    "class-static-fields-private",
    "class-methods-private",
    "class-static-methods-private",
    "decorators",
    "import-assertions",
    "import-attributes",
    "json-modules",
    "top-level-await",
    "ShadowRealm",
    "Temporal",
    "resizable-arraybuffer",
    "array-find-from-last",
    "change-array-by-copy",
    "symbols-as-weakmap-keys",
    "iterator-helpers",
    "explicit-resource-management",
    "Float16Array",
    "set-methods",
    "uint8array-base64",
    "promise-try",
    "RegExp.escape",
];

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut test_dir = "test262/test/language";
    let mut limit: Option<usize> = None;
    let mut execute_mode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--execute" | "-e" => execute_mode = true,
            "--limit" | "-l" => {
                i += 1;
                if i < args.len() {
                    limit = args[i].parse().ok();
                }
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                std::process::exit(1);
            }
            s => test_dir = s,
        }
        i += 1;
    }

    println!("Test262 {} Test", if execute_mode { "Runtime Execution" } else { "Parse-Only" });
    println!("============================");
    println!("Test directory: {}", test_dir);
    if let Some(l) = limit {
        println!("Limit: {} tests", l);
    }
    println!("Mode: {}", if execute_mode { "Parse + Execute" } else { "Parse only" });
    println!();

    let start = Instant::now();
    let stats = run_tests(test_dir, limit, execute_mode);
    let duration = start.elapsed();

    // Print results
    println!("\n============================");
    println!("RESULTS");
    println!("============================");
    println!("Total:   {}", stats.total);
    println!("Passed:  {} ({:.1}%)", stats.passed, 100.0 * stats.passed as f64 / stats.total as f64);
    println!("Failed:  {} ({:.1}%)", stats.failed, 100.0 * stats.failed as f64 / stats.total as f64);
    println!("Skipped: {} ({:.1}%)", stats.skipped, 100.0 * stats.skipped as f64 / stats.total as f64);
    println!("Time:    {:.2}s", duration.as_secs_f64());

    // Print error breakdown
    if !stats.errors.is_empty() {
        println!("\nError Breakdown:");
        let mut errors: Vec<_> = stats.errors.iter().collect();
        errors.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        for (error, files) in errors.iter().take(10) {
            println!("  {} ({} tests)", error, files.len());
            for file in files.iter().take(3) {
                println!("    - {}", file);
            }
            if files.len() > 3 {
                println!("    ... and {} more", files.len() - 3);
            }
        }
    }
}

fn run_tests(test_dir: &str, limit: Option<usize>, execute: bool) -> Stats {
    let mut stats = Stats::default();
    let mut count = 0;

    let walker = WalkDir::new(test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "js")
                .unwrap_or(false)
        });

    for entry in walker {
        if let Some(l) = limit {
            if count >= l {
                break;
            }
        }
        count += 1;

        let path = entry.path();
        let result = run_single_test(path, execute);

        match result {
            TestResult::Pass => stats.passed += 1,
            TestResult::Fail(msg) => {
                stats.failed += 1;
                let error_type = extract_error_type(&msg);
                stats.errors
                    .entry(error_type)
                    .or_default()
                    .push(path.display().to_string());
            }
            TestResult::Skip(_) => stats.skipped += 1,
        }
        stats.total += 1;

        // Progress indicator every 1000 tests
        if stats.total % 1000 == 0 {
            println!(
                "Progress: {} tests ({} passed, {} failed, {} skipped)",
                stats.total, stats.passed, stats.failed, stats.skipped
            );
        }
    }

    stats
}

enum TestResult {
    Pass,
    Fail(String),
    Skip(String),
}

fn run_single_test(path: &Path, execute: bool) -> TestResult {
    // Read test file
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return TestResult::Skip(format!("Could not read file: {}", e)),
    };

    // Parse metadata
    let metadata = TestMetadata::parse(&source);

    // Skip unsupported features
    for feature in &metadata.features {
        if UNSUPPORTED_FEATURES.contains(&feature.as_str()) {
            return TestResult::Skip(format!("Unsupported feature: {}", feature));
        }
    }

    // Skip module tests for now (require different parsing mode)
    if metadata.is_module {
        return TestResult::Skip("Module test".to_string());
    }

    // Try to parse
    let parse_result = parser::Parser::new(&source).parse();

    if metadata.is_negative_parse {
        // Negative test: should fail to parse
        match parse_result {
            Err(_) => TestResult::Pass,
            Ok(_) => TestResult::Fail(format!(
                "Expected parse error ({:?}) but parsed successfully",
                metadata.expected_error
            )),
        }
    } else {
        // Positive test: should parse successfully
        match parse_result {
            Ok(ast) => {
                // If not in execute mode, pass on successful parse
                if !execute {
                    return TestResult::Pass;
                }

                // Generate bytecode
                let mut generator = parser::BytecodeGenerator::new();
                let bytecode = match generator.generate(&ast) {
                    Ok(bc) => bc,
                    Err(e) => return TestResult::Fail(format!("Bytecode generation error: {:?}", e)),
                };

                // Execute
                let mut vm = VM::new();
                let nested = generator.take_nested_functions();
                for func in nested {
                    vm.register_function(func);
                }

                match vm.execute(&bytecode) {
                    Ok(_) => TestResult::Pass,
                    Err(e) => TestResult::Fail(format!("Runtime error: {:?}", e)),
                }
            }
            Err(e) => TestResult::Fail(format!("Parse error: {:?}", e)),
        }
    }
}

fn extract_error_type(msg: &str) -> String {
    // Extract error type from parse error message
    if msg.contains("Unexpected") {
        "Unexpected token".to_string()
    } else if msg.contains("Expected") {
        if msg.contains("Expected parse error") {
            "Should have failed".to_string()
        } else {
            "Missing token".to_string()
        }
    } else if msg.contains("Invalid") {
        "Invalid syntax".to_string()
    } else if msg.contains("Duplicate") {
        "Duplicate".to_string()
    } else if msg.contains("Unterminated") {
        "Unterminated".to_string()
    } else {
        "Other".to_string()
    }
}
