//! BigInt implementation for ES2024 arbitrary precision integers
//!
//! Provides BigInt constructor, prototype methods, and operations as specified
//! in the ECMAScript 2024 specification.

use crate::value::{BigIntValue, JsError, JsResult};
use num_bigint::{BigInt as NumBigInt, Sign};
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};

/// BigInt constructor functions
pub struct BigIntConstructor;

impl BigIntConstructor {
    /// Create BigInt from i64 integer
    pub fn from_integer(value: i64) -> JsResult<BigIntValue> {
        Ok(BigIntValue::new(NumBigInt::from(value)))
    }

    /// Create BigInt from i128 integer
    pub fn from_i128(value: i128) -> JsResult<BigIntValue> {
        Ok(BigIntValue::new(NumBigInt::from(value)))
    }

    /// Create BigInt from string representation
    ///
    /// Supports decimal, binary (0b), octal (0o), and hexadecimal (0x) formats
    pub fn from_string(s: &str) -> JsResult<BigIntValue> {
        let s = s.trim();

        // Check for binary, octal, or hex prefix
        let (radix, number_str) = if s.starts_with("0b") || s.starts_with("0B") {
            (2, &s[2..])
        } else if s.starts_with("0o") || s.starts_with("0O") {
            (8, &s[2..])
        } else if s.starts_with("0x") || s.starts_with("0X") {
            (16, &s[2..])
        } else {
            (10, s)
        };

        // Check for decimal point (not allowed in BigInt)
        if number_str.contains('.') {
            return Err(JsError::syntax_error(
                "Cannot convert non-integer to BigInt",
            ));
        }

        // Parse the number
        let result = if radix == 10 {
            number_str
                .parse::<NumBigInt>()
                .map_err(|_| JsError::syntax_error("Cannot convert to BigInt"))
        } else {
            NumBigInt::parse_bytes(number_str.as_bytes(), radix)
                .ok_or_else(|| JsError::syntax_error("Cannot convert to BigInt"))
        };

        match result {
            Ok(n) => Ok(BigIntValue::new(n)),
            Err(e) => Err(e),
        }
    }

    /// Create BigInt from f64 number
    ///
    /// Only succeeds if the number is a safe integer (no fractional part)
    pub fn from_number(n: f64) -> JsResult<BigIntValue> {
        if n.is_nan() {
            return Err(JsError::range_error("Cannot convert NaN to BigInt"));
        }

        if n.is_infinite() {
            return Err(JsError::range_error("Cannot convert Infinity to BigInt"));
        }

        // Check if the number has a fractional part
        if n != n.trunc() {
            return Err(JsError::range_error(
                "Cannot convert non-integer to BigInt",
            ));
        }

        // Convert to i64 if possible, otherwise use string representation
        if n.abs() < (i64::MAX as f64) {
            Ok(BigIntValue::new(NumBigInt::from(n as i64)))
        } else {
            // For very large numbers, use string conversion
            let s = format!("{:.0}", n);
            Self::from_string(&s)
        }
    }

    /// BigInt.asIntN(bits, bigint) - Clamps a BigInt to a signed integer with the specified number of bits
    pub fn as_int_n(bits: u32, bigint: &BigIntValue) -> BigIntValue {
        if bits == 0 {
            return BigIntValue::new(NumBigInt::zero());
        }

        let two = NumBigInt::from(2);
        let modulus = two.pow(bits);
        let half = &modulus / 2;

        // Get the value modulo 2^bits
        let mut result = bigint.inner().mod_floor(&modulus);

        // If the result is >= 2^(bits-1), wrap to negative
        if result >= half {
            result = result - &modulus;
        }

        BigIntValue::new(result)
    }

    /// BigInt.asUintN(bits, bigint) - Clamps a BigInt to an unsigned integer with the specified number of bits
    pub fn as_uint_n(bits: u32, bigint: &BigIntValue) -> BigIntValue {
        if bits == 0 {
            return BigIntValue::new(NumBigInt::zero());
        }

        let two = NumBigInt::from(2);
        let modulus = two.pow(bits);

        let result = bigint.inner().mod_floor(&modulus);
        BigIntValue::new(result)
    }
}

/// BigInt prototype methods
pub struct BigIntPrototype;

impl BigIntPrototype {
    // ========================================
    // Arithmetic Operations
    // ========================================

    /// Addition (+)
    pub fn add(a: &BigIntValue, b: &BigIntValue) -> JsResult<BigIntValue> {
        Ok(BigIntValue::new(a.inner() + b.inner()))
    }

    /// Subtraction (-)
    pub fn sub(a: &BigIntValue, b: &BigIntValue) -> JsResult<BigIntValue> {
        Ok(BigIntValue::new(a.inner() - b.inner()))
    }

    /// Multiplication (*)
    pub fn mul(a: &BigIntValue, b: &BigIntValue) -> JsResult<BigIntValue> {
        Ok(BigIntValue::new(a.inner() * b.inner()))
    }

    /// Division (/)
    ///
    /// Performs integer division (truncates towards zero)
    pub fn div(a: &BigIntValue, b: &BigIntValue) -> JsResult<BigIntValue> {
        if b.inner().is_zero() {
            return Err(JsError::range_error("Division by zero"));
        }

        Ok(BigIntValue::new(a.inner() / b.inner()))
    }

    /// Remainder (%)
    pub fn rem(a: &BigIntValue, b: &BigIntValue) -> JsResult<BigIntValue> {
        if b.inner().is_zero() {
            return Err(JsError::range_error("Division by zero"));
        }

        Ok(BigIntValue::new(a.inner() % b.inner()))
    }

    /// Exponentiation (**)
    ///
    /// Returns error if exponent is negative
    pub fn pow(base: &BigIntValue, exp: &BigIntValue) -> JsResult<BigIntValue> {
        if exp.inner().is_negative() {
            return Err(JsError::range_error(
                "Exponent must be non-negative for BigInt",
            ));
        }

        // Convert exponent to u32 for the pow operation
        let exp_u32 = match exp.inner().to_u32() {
            Some(n) => n,
            None => {
                return Err(JsError::range_error("Exponent too large"));
            }
        };

        let result = base.inner().pow(exp_u32);
        Ok(BigIntValue::new(result))
    }

    /// Unary negation (-)
    pub fn negate(a: &BigIntValue) -> BigIntValue {
        BigIntValue::new(-a.inner().clone())
    }

    /// Unary plus (+) - Throws TypeError per spec
    pub fn unary_plus(_a: &BigIntValue) -> JsResult<BigIntValue> {
        Err(JsError::type_error(
            "Cannot convert a BigInt value to a number",
        ))
    }

    // ========================================
    // Bitwise Operations
    // ========================================

    /// Bitwise AND (&)
    pub fn bitwise_and(a: &BigIntValue, b: &BigIntValue) -> BigIntValue {
        BigIntValue::new(a.inner() & b.inner())
    }

    /// Bitwise OR (|)
    pub fn bitwise_or(a: &BigIntValue, b: &BigIntValue) -> BigIntValue {
        BigIntValue::new(a.inner() | b.inner())
    }

    /// Bitwise XOR (^)
    pub fn bitwise_xor(a: &BigIntValue, b: &BigIntValue) -> BigIntValue {
        BigIntValue::new(a.inner() ^ b.inner())
    }

    /// Bitwise NOT (~)
    ///
    /// Returns -(n + 1) for BigInt (equivalent to ~n in two's complement)
    pub fn bitwise_not(a: &BigIntValue) -> BigIntValue {
        BigIntValue::new(-(a.inner() + NumBigInt::one()))
    }

    /// Left shift (<<)
    pub fn left_shift(a: &BigIntValue, shift: &BigIntValue) -> JsResult<BigIntValue> {
        let shift_amount = match shift.inner().to_u64() {
            Some(n) => n,
            None => {
                if shift.inner().is_negative() {
                    // Negative shift is equivalent to right shift
                    let abs_shift = (-shift.inner()).to_u64().unwrap_or(u64::MAX);
                    return Ok(BigIntValue::new(a.inner() >> abs_shift));
                }
                return Err(JsError::range_error("Shift amount too large"));
            }
        };

        Ok(BigIntValue::new(a.inner() << shift_amount))
    }

    /// Right shift (>>) - sign-extending
    pub fn right_shift(a: &BigIntValue, shift: &BigIntValue) -> JsResult<BigIntValue> {
        let shift_amount = match shift.inner().to_u64() {
            Some(n) => n,
            None => {
                if shift.inner().is_negative() {
                    // Negative shift is equivalent to left shift
                    let abs_shift = (-shift.inner()).to_u64().unwrap_or(u64::MAX);
                    return Ok(BigIntValue::new(a.inner() << abs_shift));
                }
                return Err(JsError::range_error("Shift amount too large"));
            }
        };

        Ok(BigIntValue::new(a.inner() >> shift_amount))
    }

    // ========================================
    // Comparison Operations
    // ========================================

    /// Equality comparison
    pub fn eq(a: &BigIntValue, b: &BigIntValue) -> bool {
        a.inner() == b.inner()
    }

    /// Less than
    pub fn lt(a: &BigIntValue, b: &BigIntValue) -> bool {
        a.inner() < b.inner()
    }

    /// Greater than
    pub fn gt(a: &BigIntValue, b: &BigIntValue) -> bool {
        a.inner() > b.inner()
    }

    /// Less than or equal
    pub fn le(a: &BigIntValue, b: &BigIntValue) -> bool {
        a.inner() <= b.inner()
    }

    /// Greater than or equal
    pub fn ge(a: &BigIntValue, b: &BigIntValue) -> bool {
        a.inner() >= b.inner()
    }

    /// Compare BigInt with Number for equality (loose equality)
    pub fn equal_to_number(bigint: &BigIntValue, num: f64) -> bool {
        if num.is_nan() || num.is_infinite() {
            return false;
        }

        // If num has a fractional part, they can't be equal
        if num != num.trunc() {
            return false;
        }

        // Convert bigint to f64 for comparison (may lose precision for very large values)
        match bigint.inner().to_f64() {
            Some(bigint_f64) => bigint_f64 == num,
            None => {
                // BigInt is too large to convert to f64
                // Try comparing through string or other means
                let num_bigint = match BigIntConstructor::from_number(num) {
                    Ok(n) => n,
                    Err(_) => return false,
                };
                bigint.inner() == num_bigint.inner()
            }
        }
    }

    /// Compare BigInt < Number
    pub fn lt_number(bigint: &BigIntValue, num: f64) -> bool {
        if num.is_nan() {
            return false;
        }

        if num.is_infinite() {
            return num > 0.0; // BigInt < +Infinity, BigInt > -Infinity
        }

        match bigint.inner().to_f64() {
            Some(bigint_f64) => bigint_f64 < num,
            None => {
                // BigInt is too large for f64
                // Check sign
                if bigint.inner().sign() == Sign::Minus {
                    num > 0.0
                } else {
                    false // Positive BigInt too large is greater than any f64
                }
            }
        }
    }

    /// Compare BigInt > Number
    pub fn gt_number(bigint: &BigIntValue, num: f64) -> bool {
        if num.is_nan() {
            return false;
        }

        if num.is_infinite() {
            return num < 0.0; // BigInt > -Infinity, BigInt < +Infinity
        }

        match bigint.inner().to_f64() {
            Some(bigint_f64) => bigint_f64 > num,
            None => {
                // BigInt is too large for f64
                // Check sign
                if bigint.inner().sign() == Sign::Minus {
                    false
                } else {
                    num < 0.0 || true // Positive BigInt too large is greater than any f64
                }
            }
        }
    }

    // ========================================
    // Prototype Methods
    // ========================================

    /// bigint.toString(radix?)
    pub fn to_string(bigint: &BigIntValue, radix: Option<u32>) -> JsResult<String> {
        let radix = radix.unwrap_or(10);

        if radix < 2 || radix > 36 {
            return Err(JsError::range_error("radix must be between 2 and 36"));
        }

        if radix == 10 {
            Ok(bigint.inner().to_string())
        } else {
            let negative = bigint.inner().sign() == Sign::Minus;
            let abs_value = bigint.inner().abs();

            if abs_value.is_zero() {
                return Ok("0".to_string());
            }

            let mut digits = Vec::new();
            let radix_bigint = NumBigInt::from(radix);
            let mut current = abs_value;

            while !current.is_zero() {
                let (quotient, remainder) = current.div_rem(&radix_bigint);
                let digit = remainder.to_u32().unwrap() as usize;
                digits.push(digit);
                current = quotient;
            }

            digits.reverse();

            let digit_chars = "0123456789abcdefghijklmnopqrstuvwxyz";
            let result: String = digits
                .iter()
                .map(|&d| digit_chars.chars().nth(d).unwrap())
                .collect();

            if negative {
                Ok(format!("-{}", result))
            } else {
                Ok(result)
            }
        }
    }

    /// bigint.valueOf()
    pub fn value_of(bigint: &BigIntValue) -> BigIntValue {
        bigint.clone()
    }

    /// bigint.toLocaleString()
    ///
    /// Basic implementation - returns string representation
    /// Full implementation would use locale-specific formatting
    pub fn to_locale_string(bigint: &BigIntValue) -> String {
        // Basic implementation: add thousand separators
        let s = bigint.inner().to_string();
        let negative = s.starts_with('-');
        let digits = if negative { &s[1..] } else { &s };

        let mut result = String::new();
        let len = digits.len();

        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }

        if negative {
            format!("-{}", result)
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigint_creation() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        assert_eq!(bigint.to_string(), "42");
    }

    #[test]
    fn test_bigint_from_string() {
        let bigint = BigIntConstructor::from_string("123").unwrap();
        assert_eq!(bigint.to_string(), "123");
    }

    #[test]
    fn test_bigint_arithmetic() {
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(20).unwrap();
        let result = BigIntPrototype::add(&a, &b).unwrap();
        assert_eq!(result.to_string(), "30");
    }

    #[test]
    fn test_bigint_bitwise() {
        let a = BigIntConstructor::from_integer(5).unwrap();
        let result = BigIntPrototype::bitwise_not(&a);
        assert_eq!(result.to_string(), "-6");
    }

    #[test]
    fn test_bigint_comparison() {
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(20).unwrap();
        assert!(BigIntPrototype::lt(&a, &b));
        assert!(!BigIntPrototype::gt(&a, &b));
    }
}
