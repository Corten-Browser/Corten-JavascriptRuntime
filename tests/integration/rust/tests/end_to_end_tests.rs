//! Comprehensive End-to-End JavaScript Execution Tests
//!
//! Tests the complete JavaScript runtime stack: Parser -> AST -> BytecodeGenerator -> VM -> Result
//! Covers all major language features including:
//! - Basic arithmetic and expressions
//! - Console and Math builtins
//! - Functions and closures
//! - Exception handling
//! - Object creation and property access
//! - Control flow (if/else, while, for)
//! - Complex programs (fibonacci, factorial)
//! - Async/Promise basics

use core_types::Value;
use interpreter::VM;
use parser::{BytecodeGenerator, Parser};

/// Helper function to execute JavaScript source code through the full pipeline
fn execute_js(source: &str) -> Result<Value, String> {
    let mut parser = Parser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut gen = BytecodeGenerator::new();
    let bytecode = gen
        .generate(&ast)
        .map_err(|e| format!("Bytecode generation error: {:?}", e))?;

    let mut vm = VM::new();
    vm.execute(&bytecode)
        .map_err(|e| format!("Execution error: {:?}", e))
}

/// Helper function to check if result is a number with expected value
fn assert_number(result: Value, expected: i32, message: &str) {
    match result {
        Value::Smi(n) => assert_eq!(n, expected, "{}", message),
        Value::Double(n) => {
            assert_eq!(n as i32, expected, "{}", message);
        }
        _ => panic!("{}: Expected number {}, got {:?}", message, expected, result),
    }
}

/// Helper function to check if result is a float with expected value
fn assert_float(result: Value, expected: f64, tolerance: f64, message: &str) {
    match result {
        Value::Double(n) => {
            assert!(
                (n - expected).abs() < tolerance,
                "{}: Expected ~{}, got {}",
                message,
                expected,
                n
            );
        }
        Value::Smi(n) => {
            assert!(
                (n as f64 - expected).abs() < tolerance,
                "{}: Expected ~{}, got {}",
                message,
                expected,
                n
            );
        }
        _ => panic!("{}: Expected float ~{}, got {:?}", message, expected, result),
    }
}

/// Helper function to check if result is a string (HeapObject or String)
/// Note: Strings can be stored as HeapObjects or Value::String
fn assert_is_heap_object(result: Value, message: &str) {
    match result {
        Value::HeapObject(_) => {} // Strings are heap objects
        Value::String(_) => {}     // Or direct String values
        _ => panic!("{}: Expected HeapObject (string), got {:?}", message, result),
    }
}

// =============================================================================
// 1. Basic JavaScript Execution Tests
// =============================================================================

#[test]
fn test_arithmetic_expression() {
    let result = execute_js("1 + 2 * 3;").expect("Execution failed");
    assert_number(result, 7, "1 + 2 * 3 should equal 7");
}

#[test]
fn test_arithmetic_with_parentheses() {
    let result = execute_js("(1 + 2) * 3;").expect("Execution failed");
    assert_number(result, 9, "(1 + 2) * 3 should equal 9");
}

#[test]
fn test_variable_declaration_and_usage() {
    let result = execute_js("let x = 5; x + 10;").expect("Execution failed");
    assert_number(result, 15, "x + 10 should equal 15");
}

#[test]
fn test_multiple_variable_operations() {
    let result = execute_js("let a = 10; let b = 20; let c = a + b; c * 2;")
        .expect("Execution failed");
    assert_number(result, 60, "(a + b) * 2 should equal 60");
}

#[test]
fn test_function_definition_and_call() {
    let source = r#"
        function add(a, b) {
            return a + b;
        }
        add(3, 4);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 7, "add(3, 4) should return 7");
}

#[test]
fn test_function_with_multiple_statements() {
    let source = r#"
        function multiply_and_add(a, b, c) {
            let product = a * b;
            return product + c;
        }
        multiply_and_add(3, 4, 5);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 17, "multiply_and_add(3, 4, 5) should return 17");
}

// =============================================================================
// 2. Console and Math Builtins Tests
// =============================================================================

#[test]
fn test_console_log_returns_undefined() {
    let result = execute_js("console.log('test');").expect("Execution failed");
    assert!(
        matches!(result, Value::Undefined),
        "console.log should return undefined"
    );
}

#[test]
fn test_console_log_multiple_values() {
    let result = execute_js("console.log('a', 'b', 'c');").expect("Execution failed");
    assert!(
        matches!(result, Value::Undefined),
        "console.log with multiple args should return undefined"
    );
}

#[test]
fn test_console_error_returns_undefined() {
    let result = execute_js("console.error('error message');").expect("Execution failed");
    assert!(
        matches!(result, Value::Undefined),
        "console.error should return undefined"
    );
}

#[test]
fn test_math_abs() {
    let result = execute_js("Math.abs(-5);").expect("Execution failed");
    assert_number(result, 5, "Math.abs(-5) should return 5");
}

#[test]
fn test_math_sqrt() {
    let result = execute_js("Math.sqrt(16);").expect("Execution failed");
    assert_float(result, 4.0, 0.001, "Math.sqrt(16) should return 4");
}

#[test]
fn test_math_operations_combined() {
    let result = execute_js("Math.abs(-5) + Math.sqrt(16);").expect("Execution failed");
    assert_float(result, 9.0, 0.001, "Math.abs(-5) + Math.sqrt(16) should return 9");
}

#[test]
fn test_math_floor() {
    let result = execute_js("Math.floor(3.7);").expect("Execution failed");
    assert_number(result, 3, "Math.floor(3.7) should return 3");
}

#[test]
fn test_math_ceil() {
    let result = execute_js("Math.ceil(3.2);").expect("Execution failed");
    assert_number(result, 4, "Math.ceil(3.2) should return 4");
}

#[test]
fn test_math_pow() {
    let result = execute_js("Math.pow(2, 8);").expect("Execution failed");
    assert_float(result, 256.0, 0.001, "Math.pow(2, 8) should return 256");
}

#[test]
fn test_math_pi() {
    let result = execute_js("Math.PI;").expect("Execution failed");
    assert_float(result, 3.14159265, 0.0001, "Math.PI should be ~3.14159");
}

#[test]
fn test_math_max() {
    let result = execute_js("Math.max(1, 5, 3, 2);").expect("Execution failed");
    assert_float(result, 5.0, 0.001, "Math.max(1, 5, 3, 2) should return 5");
}

#[test]
fn test_math_min() {
    let result = execute_js("Math.min(4, 2, 6, 1);").expect("Execution failed");
    assert_float(result, 1.0, 0.001, "Math.min(4, 2, 6, 1) should return 1");
}

// =============================================================================
// 3. Closure and Scope Tests
// =============================================================================

#[test]
fn test_closure_captures_outer_variable() {
    let source = r#"
        function outer() {
            let x = 10;
            return function inner() {
                return x;
            };
        }
        let f = outer();
        f();
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 10, "Closure should capture outer variable x = 10");
}

#[test]
fn test_nested_closures() {
    let source = r#"
        function a() {
            let x = 1;
            return function b() {
                let y = 2;
                return function c() {
                    return x + y;
                };
            };
        }
        a()()();
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 3, "Nested closures should capture x=1 and y=2, return 3");
}

#[test]
fn test_closure_with_modification() {
    let source = r#"
        function counter() {
            let count = 0;
            return function increment() {
                count = count + 1;
                return count;
            };
        }
        let inc = counter();
        inc();
        inc();
        inc();
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 3, "Counter closure should increment to 3");
}

#[test]
fn test_closure_preserves_environment() {
    let source = r#"
        function makeAdder(x) {
            return function(y) {
                return x + y;
            };
        }
        let add5 = makeAdder(5);
        let add10 = makeAdder(10);
        add5(3) + add10(3);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 21, "add5(3) + add10(3) should equal 8 + 13 = 21");
}

// =============================================================================
// 4. Exception Handling Tests
// =============================================================================

#[test]
fn test_try_catch_catches_throw() {
    let source = r#"
        let result = 0;
        try {
            throw "error";
        } catch (e) {
            result = 42;
        }
        result;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 42, "Should catch thrown error and set result to 42");
}

#[test]
fn test_finally_always_runs() {
    let source = r#"
        let x = 0;
        try {
            x = 1;
            throw "error";
        } catch (e) {
            x = 2;
        } finally {
            x = x + 10;
        }
        x;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 12, "Finally should run, x should be 2 + 10 = 12");
}

#[test]
fn test_try_without_throw() {
    let source = r#"
        let result = 0;
        try {
            result = 10;
        } catch (e) {
            result = 99;
        }
        result;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 10, "Without throw, catch should not execute");
}

#[test]
fn test_finally_without_exception() {
    let source = r#"
        let x = 1;
        try {
            x = x + 1;
        } finally {
            x = x * 10;
        }
        x;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 20, "Finally runs even without exception, x should be (1+1)*10 = 20");
}

#[test]
fn test_nested_try_catch() {
    let source = r#"
        let result = 0;
        try {
            try {
                throw "inner";
            } catch (e) {
                result = 1;
            }
            result = result + 10;
        } catch (e) {
            result = 100;
        }
        result;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 11, "Inner catch handles exception, outer continues, result = 11");
}

// =============================================================================
// 5. Object Creation and Property Access Tests
// =============================================================================

#[test]
fn test_object_literal_creation() {
    let source = r#"
        let obj = {};
        obj.x = 5;
        obj.x;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 5, "Object property should be accessible");
}

#[test]
fn test_nested_object_properties() {
    let source = r#"
        let obj = {};
        obj.a = {};
        obj.a.b = 42;
        obj.a.b;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 42, "Nested object property should be accessible");
}

#[test]
fn test_object_with_multiple_properties() {
    let source = r#"
        let obj = {};
        obj.x = 10;
        obj.y = 20;
        obj.z = 30;
        obj.x + obj.y + obj.z;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 60, "Sum of object properties should be 60");
}

#[test]
fn test_object_property_reassignment() {
    let source = r#"
        let obj = {};
        obj.value = 5;
        obj.value = obj.value * 3;
        obj.value;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 15, "Object property should be reassigned to 15");
}

// =============================================================================
// 6. Control Flow Tests
// =============================================================================

#[test]
fn test_if_else_execution_true_branch() {
    let source = r#"
        let x = 10;
        if (x > 5) {
            x = x * 2;
        } else {
            x = x / 2;
        }
        x;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 20, "True branch should execute, x = 10 * 2 = 20");
}

#[test]
fn test_if_else_execution_false_branch() {
    let source = r#"
        let x = 3;
        if (x > 5) {
            x = x * 2;
        } else {
            x = x + 10;
        }
        x;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 13, "False branch should execute, x = 3 + 10 = 13");
}

#[test]
fn test_nested_if_else() {
    let source = r#"
        let x = 15;
        let result = 0;
        if (x > 10) {
            if (x > 20) {
                result = 3;
            } else {
                result = 2;
            }
        } else {
            result = 1;
        }
        result;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 2, "Nested if should evaluate to 2 (medium)");
}

#[test]
fn test_while_loop_execution() {
    let source = r#"
        let sum = 0;
        let i = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 10, "While loop should sum 0+1+2+3+4 = 10");
}

#[test]
fn test_while_loop_with_condition() {
    let source = r#"
        let x = 100;
        while (x > 10) {
            x = x / 2;
        }
        x;
    "#;
    let result = execute_js(source).expect("Execution failed");
    // 100 -> 50 -> 25 -> 12.5 -> 6.25
    assert_float(result, 6.25, 0.001, "While loop should divide until x <= 10");
}

#[test]
fn test_for_loop_execution() {
    let source = r#"
        let product = 1;
        for (let i = 1; i <= 4; i = i + 1) {
            product = product * i;
        }
        product;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 24, "For loop should compute 1*2*3*4 = 24");
}

#[test]
fn test_for_loop_with_step() {
    let source = r#"
        let sum = 0;
        for (let i = 0; i < 10; i = i + 2) {
            sum = sum + i;
        }
        sum;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 20, "For loop with step 2 should sum 0+2+4+6+8 = 20");
}

#[test]
fn test_for_loop_countdown() {
    let source = r#"
        let result = 0;
        for (let i = 5; i > 0; i = i - 1) {
            result = result * 10 + i;
        }
        result;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 54321, "For loop countdown should build 54321");
}

// =============================================================================
// 7. Full Programs Tests
// =============================================================================

#[test]
fn test_fibonacci_function() {
    let source = r#"
        function fib(n) {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 55, "fib(10) should return 55");
}

#[test]
fn test_fibonacci_small_values() {
    let source = r#"
        function fib(n) {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(0) + fib(1) + fib(2) + fib(3) + fib(4);
    "#;
    let result = execute_js(source).expect("Execution failed");
    // fib(0)=0, fib(1)=1, fib(2)=1, fib(3)=2, fib(4)=3 => 0+1+1+2+3=7
    assert_number(result, 7, "Sum of fib(0-4) should be 7");
}

#[test]
fn test_factorial_function() {
    let source = r#"
        function factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 120, "factorial(5) should return 120");
}

#[test]
fn test_factorial_edge_cases() {
    let source = r#"
        function factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(0) + factorial(1) + factorial(2);
    "#;
    let result = execute_js(source).expect("Execution failed");
    // factorial(0)=1, factorial(1)=1, factorial(2)=2 => 1+1+2=4
    assert_number(result, 4, "Sum of factorial(0-2) should be 4");
}

#[test]
fn test_gcd_function() {
    let source = r#"
        function gcd(a, b) {
            if (b === 0) {
                return a;
            }
            return gcd(b, a % b);
        }
        gcd(48, 18);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 6, "gcd(48, 18) should return 6");
}

#[test]
fn test_sum_of_squares() {
    let source = r#"
        function sumOfSquares(n) {
            let sum = 0;
            for (let i = 1; i <= n; i = i + 1) {
                sum = sum + i * i;
            }
            return sum;
        }
        sumOfSquares(5);
    "#;
    let result = execute_js(source).expect("Execution failed");
    // 1 + 4 + 9 + 16 + 25 = 55
    assert_number(result, 55, "Sum of squares 1-5 should be 55");
}

#[test]
fn test_is_prime_function() {
    let source = r#"
        function isPrime(n) {
            if (n < 2) {
                return false;
            }
            let i = 2;
            while (i * i <= n) {
                if (n % i === 0) {
                    return false;
                }
                i = i + 1;
            }
            return true;
        }
        let count = 0;
        for (let n = 2; n <= 20; n = n + 1) {
            if (isPrime(n)) {
                count = count + 1;
            }
        }
        count;
    "#;
    let result = execute_js(source).expect("Execution failed");
    // Primes <= 20: 2, 3, 5, 7, 11, 13, 17, 19 = 8 primes
    assert_number(result, 8, "There should be 8 primes from 2 to 20");
}

// =============================================================================
// 8. Async/Promise Tests (Basic)
// =============================================================================

#[test]
fn test_promise_resolve_creates_promise() {
    let source = r#"
        let p = Promise.resolve(42);
        p;
    "#;
    let result = execute_js(source).expect("Execution failed");
    // Promise.resolve should return a Promise object (HeapObject or NativeObject)
    assert!(
        matches!(result, Value::HeapObject(_) | Value::NativeObject(_)),
        "Promise.resolve should create a promise object, got {:?}",
        result
    );
}

#[test]
fn test_promise_reject_creates_promise() {
    let source = r#"
        let p = Promise.reject("error");
        p;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert!(
        matches!(result, Value::HeapObject(_) | Value::NativeObject(_)),
        "Promise.reject should create a promise object, got {:?}",
        result
    );
}

// =============================================================================
// 9. Additional Edge Cases and Complex Scenarios
// =============================================================================

#[test]
fn test_logical_operators() {
    let source = r#"
        let a = true;
        let b = false;
        let result = 0;
        if (a && !b) {
            result = 1;
        }
        result;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 1, "Logical operators should work correctly");
}

#[test]
fn test_string_concatenation() {
    let source = r#"
        let a = "Hello";
        let b = " ";
        let c = "World";
        a + b + c;
    "#;
    let result = execute_js(source).expect("Execution failed");
    // String concatenation result is a HeapObject in current implementation
    assert_is_heap_object(result, "String concatenation should produce a heap object");
}

#[test]
fn test_comparison_operators() {
    let source = r#"
        let count = 0;
        if (5 < 10) count = count + 1;
        if (10 > 5) count = count + 1;
        if (5 <= 5) count = count + 1;
        if (10 >= 10) count = count + 1;
        if (5 === 5) count = count + 1;
        if (5 !== 6) count = count + 1;
        count;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 6, "All comparison operators should work, count = 6");
}

#[test]
fn test_assignment_operators() {
    let source = r#"
        let x = 10;
        x = x + 5;
        x = x - 3;
        x = x * 2;
        x = x / 4;
        x;
    "#;
    let result = execute_js(source).expect("Execution failed");
    // x = 10 -> 15 -> 12 -> 24 -> 6
    assert_number(result, 6, "Assignment operations should result in 6");
}

#[test]
fn test_nested_function_calls() {
    let source = r#"
        function double(x) {
            return x * 2;
        }
        function addOne(x) {
            return x + 1;
        }
        double(addOne(double(addOne(5))));
    "#;
    let result = execute_js(source).expect("Execution failed");
    // 5 -> 6 -> 12 -> 13 -> 26
    assert_number(result, 26, "Nested function calls should result in 26");
}

#[test]
fn test_early_return() {
    let source = r#"
        function findFirst(a, b, c) {
            if (a > 0) {
                return a;
            }
            if (b > 0) {
                return b;
            }
            return c;
        }
        findFirst(-1, 5, 10);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 5, "Early return should return first positive number");
}

#[test]
fn test_boolean_coercion() {
    let source = r#"
        let count = 0;
        if (true) count = count + 1;
        if (!false) count = count + 1;
        if (1) count = count + 1;
        if (!0) count = count + 1;
        count;
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 4, "Boolean coercion should work correctly");
}

#[test]
fn test_variable_shadowing() {
    let source = r#"
        let x = 10;
        function inner() {
            let x = 20;
            return x;
        }
        x + inner();
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert_number(result, 30, "Variable shadowing should work, 10 + 20 = 30");
}

#[test]
fn test_complex_expression() {
    let source = r#"
        let a = 5;
        let b = 3;
        let c = 2;
        ((a + b) * c - a) / b + c * c;
    "#;
    let result = execute_js(source).expect("Execution failed");
    // ((5 + 3) * 2 - 5) / 3 + 2 * 2 = (16 - 5) / 3 + 4 = 11 / 3 + 4 = 3.666... + 4 = 7.666...
    assert_float(result, 7.666666, 0.01, "Complex expression should evaluate correctly");
}

#[test]
fn test_empty_function() {
    let source = r#"
        function doNothing() {
        }
        doNothing();
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert!(
        matches!(result, Value::Undefined),
        "Empty function should return undefined"
    );
}

#[test]
fn test_return_without_value() {
    let source = r#"
        function earlyExit(x) {
            if (x < 0) {
                return;
            }
            return x * 2;
        }
        earlyExit(-5);
    "#;
    let result = execute_js(source).expect("Execution failed");
    assert!(
        matches!(result, Value::Undefined),
        "Return without value should return undefined"
    );
}
