// Math Fibonacci - simplified SunSpider benchmark
// Tests recursion and basic arithmetic

function fibonacci(n) {
    if (n < 2) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

let result = 0;
for (let i = 0; i < 10; i++) {
    result = fibonacci(20);
}
result
