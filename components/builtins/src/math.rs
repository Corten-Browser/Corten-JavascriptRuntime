//! Math object methods

use std::f64::consts;

/// Math object with static methods
pub struct MathObject;

impl MathObject {
    /// Math.abs(x)
    pub fn abs(x: f64) -> f64 {
        x.abs()
    }

    /// Math.ceil(x)
    pub fn ceil(x: f64) -> f64 {
        x.ceil()
    }

    /// Math.floor(x)
    pub fn floor(x: f64) -> f64 {
        x.floor()
    }

    /// Math.round(x)
    pub fn round(x: f64) -> f64 {
        // JavaScript's round behaves differently for negative numbers
        // -1.5 rounds to -1 in JS (towards positive infinity for .5 cases)
        if x.fract() == 0.5 {
            x.ceil()
        } else if x.fract() == -0.5 {
            x.ceil()
        } else {
            x.round()
        }
    }

    /// Math.sqrt(x)
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }

    /// Math.pow(base, exponent)
    pub fn pow(base: f64, exponent: f64) -> f64 {
        base.powf(exponent)
    }

    /// Math.sin(x)
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }

    /// Math.cos(x)
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }

    /// Math.tan(x)
    pub fn tan(x: f64) -> f64 {
        x.tan()
    }

    /// Math.random()
    pub fn random() -> f64 {
        // Simple random using system time (for production, use proper RNG)
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Linear congruential generator
        let x = (seed.wrapping_mul(1103515245).wrapping_add(12345)) as f64;
        (x / u128::MAX as f64).abs() % 1.0
    }

    /// Math.max(...values)
    pub fn max(values: &[f64]) -> f64 {
        if values.is_empty() {
            return f64::NEG_INFINITY;
        }

        let mut result = f64::NEG_INFINITY;
        for &v in values {
            if v.is_nan() {
                return f64::NAN;
            }
            if v > result {
                result = v;
            }
        }
        result
    }

    /// Math.min(...values)
    pub fn min(values: &[f64]) -> f64 {
        if values.is_empty() {
            return f64::INFINITY;
        }

        let mut result = f64::INFINITY;
        for &v in values {
            if v.is_nan() {
                return f64::NAN;
            }
            if v < result {
                result = v;
            }
        }
        result
    }

    /// Math.log(x) - natural logarithm
    pub fn log(x: f64) -> f64 {
        x.ln()
    }

    /// Math.exp(x)
    pub fn exp(x: f64) -> f64 {
        x.exp()
    }

    /// Math.PI
    pub const PI: f64 = consts::PI;

    /// Math.E
    pub const E: f64 = consts::E;

    /// Math.LN2
    pub const LN2: f64 = consts::LN_2;

    /// Math.LN10
    pub const LN10: f64 = consts::LN_10;

    /// Math.LOG2E
    pub const LOG2E: f64 = consts::LOG2_E;

    /// Math.LOG10E
    pub const LOG10E: f64 = consts::LOG10_E;

    /// Math.SQRT2
    pub const SQRT2: f64 = consts::SQRT_2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        assert_eq!(MathObject::abs(-5.0), 5.0);
        assert_eq!(MathObject::abs(5.0), 5.0);
        assert_eq!(MathObject::abs(0.0), 0.0);
    }

    #[test]
    fn test_ceil() {
        assert_eq!(MathObject::ceil(1.1), 2.0);
        assert_eq!(MathObject::ceil(-1.1), -1.0);
        assert_eq!(MathObject::ceil(2.0), 2.0);
    }

    #[test]
    fn test_floor() {
        assert_eq!(MathObject::floor(1.9), 1.0);
        assert_eq!(MathObject::floor(-1.1), -2.0);
        assert_eq!(MathObject::floor(2.0), 2.0);
    }

    #[test]
    fn test_round() {
        assert_eq!(MathObject::round(1.4), 1.0);
        assert_eq!(MathObject::round(1.5), 2.0);
        assert_eq!(MathObject::round(-1.5), -1.0);
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(MathObject::sqrt(4.0), 2.0);
        assert_eq!(MathObject::sqrt(9.0), 3.0);
        assert!(MathObject::sqrt(-1.0).is_nan());
    }

    #[test]
    fn test_pow() {
        assert_eq!(MathObject::pow(2.0, 3.0), 8.0);
        assert_eq!(MathObject::pow(3.0, 2.0), 9.0);
        assert_eq!(MathObject::pow(10.0, 0.0), 1.0);
    }

    #[test]
    fn test_trigonometry() {
        assert!((MathObject::sin(0.0) - 0.0).abs() < 1e-10);
        assert!((MathObject::cos(0.0) - 1.0).abs() < 1e-10);
        assert!((MathObject::tan(0.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_random() {
        let r = MathObject::random();
        assert!(r >= 0.0 && r < 1.0);
    }

    #[test]
    fn test_max() {
        assert_eq!(MathObject::max(&[1.0, 2.0, 3.0]), 3.0);
        assert_eq!(MathObject::max(&[-1.0, -2.0]), -1.0);
        assert_eq!(MathObject::max(&[]), f64::NEG_INFINITY);
    }

    #[test]
    fn test_min() {
        assert_eq!(MathObject::min(&[1.0, 2.0, 3.0]), 1.0);
        assert_eq!(MathObject::min(&[-1.0, -2.0]), -2.0);
        assert_eq!(MathObject::min(&[]), f64::INFINITY);
    }

    #[test]
    fn test_log_and_exp() {
        let result = MathObject::log(consts::E);
        assert!((result - 1.0).abs() < 1e-10);

        let result = MathObject::exp(1.0);
        assert!((result - consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_constants() {
        assert!((MathObject::PI - std::f64::consts::PI).abs() < 1e-10);
        assert!((MathObject::E - std::f64::consts::E).abs() < 1e-10);
    }
}
