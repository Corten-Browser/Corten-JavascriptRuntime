//! Contract tests for MathObject

use builtins::MathObject;

#[test]
fn test_math_abs() {
    assert_eq!(MathObject::abs(-5.0), 5.0);
    assert_eq!(MathObject::abs(5.0), 5.0);
    assert_eq!(MathObject::abs(0.0), 0.0);
}

#[test]
fn test_math_ceil() {
    assert_eq!(MathObject::ceil(1.1), 2.0);
    assert_eq!(MathObject::ceil(-1.1), -1.0);
    assert_eq!(MathObject::ceil(2.0), 2.0);
}

#[test]
fn test_math_floor() {
    assert_eq!(MathObject::floor(1.9), 1.0);
    assert_eq!(MathObject::floor(-1.1), -2.0);
    assert_eq!(MathObject::floor(2.0), 2.0);
}

#[test]
fn test_math_round() {
    assert_eq!(MathObject::round(1.4), 1.0);
    assert_eq!(MathObject::round(1.5), 2.0);
    assert_eq!(MathObject::round(-1.5), -1.0);
}

#[test]
fn test_math_sqrt() {
    assert_eq!(MathObject::sqrt(4.0), 2.0);
    assert_eq!(MathObject::sqrt(9.0), 3.0);
    assert!(MathObject::sqrt(-1.0).is_nan());
}

#[test]
fn test_math_pow() {
    assert_eq!(MathObject::pow(2.0, 3.0), 8.0);
    assert_eq!(MathObject::pow(3.0, 2.0), 9.0);
    assert_eq!(MathObject::pow(10.0, 0.0), 1.0);
}

#[test]
fn test_math_sin() {
    let result = MathObject::sin(0.0);
    assert!((result - 0.0).abs() < 1e-10);
}

#[test]
fn test_math_cos() {
    let result = MathObject::cos(0.0);
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn test_math_tan() {
    let result = MathObject::tan(0.0);
    assert!((result - 0.0).abs() < 1e-10);
}

#[test]
fn test_math_random() {
    let result = MathObject::random();
    assert!(result >= 0.0 && result < 1.0);
}

#[test]
fn test_math_max() {
    assert_eq!(MathObject::max(&[1.0, 2.0, 3.0]), 3.0);
    assert_eq!(MathObject::max(&[-1.0, -2.0]), -1.0);
}

#[test]
fn test_math_min() {
    assert_eq!(MathObject::min(&[1.0, 2.0, 3.0]), 1.0);
    assert_eq!(MathObject::min(&[-1.0, -2.0]), -2.0);
}

#[test]
fn test_math_log() {
    let result = MathObject::log(std::f64::consts::E);
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn test_math_exp() {
    let result = MathObject::exp(1.0);
    assert!((result - std::f64::consts::E).abs() < 1e-10);
}
