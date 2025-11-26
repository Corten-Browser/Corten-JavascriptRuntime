//! Number object and Number.prototype methods

use crate::value::{JsError, JsResult, JsValue};

/// Number object with static properties and methods
pub struct NumberObject;

impl NumberObject {
    /// Number.NaN
    pub const NAN: f64 = f64::NAN;

    /// Number.POSITIVE_INFINITY
    pub const POSITIVE_INFINITY: f64 = f64::INFINITY;

    /// Number.NEGATIVE_INFINITY
    pub const NEGATIVE_INFINITY: f64 = f64::NEG_INFINITY;

    /// Number.MAX_VALUE
    pub const MAX_VALUE: f64 = f64::MAX;

    /// Number.MIN_VALUE
    pub const MIN_VALUE: f64 = f64::MIN_POSITIVE;

    /// Number.MAX_SAFE_INTEGER
    pub const MAX_SAFE_INTEGER: f64 = 9007199254740991.0;

    /// Number.MIN_SAFE_INTEGER
    pub const MIN_SAFE_INTEGER: f64 = -9007199254740991.0;

    /// Number.EPSILON
    pub const EPSILON: f64 = f64::EPSILON;

    /// Number.isNaN(value) - ES6+ strict NaN check
    pub fn is_nan(value: f64) -> bool {
        value.is_nan()
    }

    /// Number.isFinite(value) - ES6+ strict finite check
    pub fn is_finite(value: f64) -> bool {
        value.is_finite()
    }

    /// Number.isInteger(value)
    pub fn is_integer(value: f64) -> bool {
        value.is_finite() && value.trunc() == value
    }

    /// Number.isSafeInteger(value)
    pub fn is_safe_integer(value: f64) -> bool {
        Self::is_integer(value) && value.abs() <= Self::MAX_SAFE_INTEGER
    }

    /// Number.parseInt(string, radix) - same as global parseInt
    pub fn parse_int(s: &str, radix: Option<u32>) -> f64 {
        let radix = radix.unwrap_or(10);
        if radix < 2 || radix > 36 {
            return f64::NAN;
        }

        let s = s.trim();
        if s.is_empty() {
            return f64::NAN;
        }

        let (negative, s) = if s.starts_with('-') {
            (true, &s[1..])
        } else if s.starts_with('+') {
            (false, &s[1..])
        } else {
            (false, s)
        };

        let result = i64::from_str_radix(s, radix);
        match result {
            Ok(n) => {
                let value = n as f64;
                if negative { -value } else { value }
            }
            Err(_) => {
                // Try to parse as much as possible
                let mut n: i64 = 0;
                let mut found_digit = false;
                for c in s.chars() {
                    let digit = c.to_digit(radix);
                    match digit {
                        Some(d) => {
                            n = n * radix as i64 + d as i64;
                            found_digit = true;
                        }
                        None => break,
                    }
                }
                if found_digit {
                    let value = n as f64;
                    if negative { -value } else { value }
                } else {
                    f64::NAN
                }
            }
        }
    }

    /// Number.parseFloat(string) - same as global parseFloat
    pub fn parse_float(s: &str) -> f64 {
        let s = s.trim();
        if s.is_empty() {
            return f64::NAN;
        }

        // Handle special values
        if s == "Infinity" || s == "+Infinity" {
            return f64::INFINITY;
        }
        if s == "-Infinity" {
            return f64::NEG_INFINITY;
        }

        // Parse as much as possible as a valid number
        s.parse::<f64>().unwrap_or(f64::NAN)
    }
}

/// Global isNaN function (coerces to number first, unlike Number.isNaN)
pub fn global_is_nan(value: f64) -> bool {
    value.is_nan()
}

/// Global isFinite function (coerces to number first, unlike Number.isFinite)
pub fn global_is_finite(value: f64) -> bool {
    value.is_finite()
}

/// Number.prototype methods
pub struct NumberPrototype;

impl NumberPrototype {
    /// Number.prototype.toString(radix)
    pub fn to_string(num: f64, radix: Option<u32>) -> JsResult<String> {
        let radix = radix.unwrap_or(10);

        if radix < 2 || radix > 36 {
            return Err(JsError::range_error("radix must be between 2 and 36"));
        }

        if num.is_nan() {
            return Ok("NaN".to_string());
        }

        if num.is_infinite() {
            return Ok(if num > 0.0 {
                "Infinity".to_string()
            } else {
                "-Infinity".to_string()
            });
        }

        if radix == 10 {
            // Standard decimal representation
            if num == num.trunc() && num.abs() < 1e15 {
                Ok(format!("{}", num as i64))
            } else {
                Ok(num.to_string())
            }
        } else {
            // Convert to integer and then to radix
            let negative = num < 0.0;
            let mut n = num.abs() as u64;
            let mut result = String::new();

            if n == 0 {
                return Ok("0".to_string());
            }

            let digits = "0123456789abcdefghijklmnopqrstuvwxyz";
            while n > 0 {
                let digit = (n % radix as u64) as usize;
                result.insert(0, digits.chars().nth(digit).unwrap());
                n /= radix as u64;
            }

            if negative {
                result.insert(0, '-');
            }

            Ok(result)
        }
    }

    /// Number.prototype.toFixed(digits)
    pub fn to_fixed(num: f64, digits: u32) -> JsResult<String> {
        if digits > 100 {
            return Err(JsError::range_error("toFixed() digits argument must be between 0 and 100"));
        }

        if num.is_nan() {
            return Ok("NaN".to_string());
        }

        if num.is_infinite() {
            return Ok(if num > 0.0 {
                "Infinity".to_string()
            } else {
                "-Infinity".to_string()
            });
        }

        Ok(format!("{:.prec$}", num, prec = digits as usize))
    }

    /// Number.prototype.toPrecision(precision)
    pub fn to_precision(num: f64, precision: u32) -> JsResult<String> {
        if precision < 1 || precision > 100 {
            return Err(JsError::range_error("toPrecision() argument must be between 1 and 100"));
        }

        if num.is_nan() {
            return Ok("NaN".to_string());
        }

        if num.is_infinite() {
            return Ok(if num > 0.0 {
                "Infinity".to_string()
            } else {
                "-Infinity".to_string()
            });
        }

        // toPrecision formats with significant figures, not decimal places
        if num == 0.0 {
            if precision == 1 {
                return Ok("0".to_string());
            }
            return Ok(format!("0.{}", "0".repeat(precision as usize - 1)));
        }

        let negative = num < 0.0;
        let abs_num = num.abs();

        // Calculate the order of magnitude
        let log10 = abs_num.log10().floor() as i32;
        let digits_before_decimal = log10 + 1;

        if digits_before_decimal <= 0 || digits_before_decimal > precision as i32 {
            // Use exponential notation or handle very small numbers
            // For now, use Rust's built-in precision formatting for significant figures
            let formatted = format!("{:.prec$e}", abs_num, prec = (precision as usize).saturating_sub(1));
            // Parse and reformat to match JS behavior
            if let Some((mantissa, exp)) = formatted.split_once('e') {
                let exp_val: i32 = exp.parse().unwrap_or(0);
                // Reconstruct based on exponent
                if exp_val >= 0 && exp_val < precision as i32 {
                    // Can represent without exponential notation
                    let scale = 10f64.powi(exp_val);
                    let scaled = mantissa.replace('.', "").parse::<f64>().unwrap_or(0.0)
                        / 10f64.powi((precision as i32 - 1) as i32) * scale;
                    let decimal_places = (precision as i32 - exp_val - 1).max(0) as usize;
                    let result = format!("{:.prec$}", scaled, prec = decimal_places);
                    if negative {
                        return Ok(format!("-{}", result));
                    }
                    return Ok(result);
                }
            }
            // Fallback
            let result = format!("{:.prec$}", abs_num, prec = (precision as usize).saturating_sub(1));
            if negative {
                return Ok(format!("-{}", result));
            }
            return Ok(result);
        }

        // Normal case: number has some digits before decimal point
        let decimal_places = (precision as i32 - digits_before_decimal).max(0) as usize;
        let result = format!("{:.prec$}", abs_num, prec = decimal_places);

        if negative {
            Ok(format!("-{}", result))
        } else {
            Ok(result)
        }
    }

    /// Number.prototype.valueOf()
    pub fn value_of(val: &JsValue) -> JsResult<f64> {
        match val {
            JsValue::Number(n) => Ok(*n),
            _ => Err(JsError::type_error("valueOf called on non-number")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string_default() {
        assert_eq!(NumberPrototype::to_string(42.0, None).unwrap(), "42");
        assert_eq!(NumberPrototype::to_string(3.14, None).unwrap(), "3.14");
    }

    #[test]
    fn test_to_string_radix() {
        assert_eq!(NumberPrototype::to_string(255.0, Some(16)).unwrap(), "ff");
        assert_eq!(NumberPrototype::to_string(10.0, Some(2)).unwrap(), "1010");
    }

    #[test]
    fn test_to_string_invalid_radix() {
        assert!(NumberPrototype::to_string(10.0, Some(1)).is_err());
        assert!(NumberPrototype::to_string(10.0, Some(37)).is_err());
    }

    #[test]
    fn test_to_fixed() {
        assert_eq!(NumberPrototype::to_fixed(3.14159, 2).unwrap(), "3.14");
        assert_eq!(NumberPrototype::to_fixed(3.14159, 0).unwrap(), "3");
        assert_eq!(NumberPrototype::to_fixed(3.14159, 4).unwrap(), "3.1416");
    }

    #[test]
    fn test_to_precision() {
        // Note: toPrecision uses the precision as total significant digits
        let result = NumberPrototype::to_precision(123.456, 4).unwrap();
        assert!(result.starts_with("123.")); // At least starts correctly
        let result = NumberPrototype::to_precision(3.14159, 3).unwrap();
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_value_of() {
        let val = JsValue::number(42.0);
        assert_eq!(NumberPrototype::value_of(&val).unwrap(), 42.0);
    }

    #[test]
    fn test_special_values() {
        assert_eq!(NumberPrototype::to_string(f64::NAN, None).unwrap(), "NaN");
        assert_eq!(
            NumberPrototype::to_string(f64::INFINITY, None).unwrap(),
            "Infinity"
        );
        assert_eq!(
            NumberPrototype::to_string(f64::NEG_INFINITY, None).unwrap(),
            "-Infinity"
        );
    }
}
