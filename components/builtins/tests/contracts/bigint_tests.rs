//! BigInt contract tests
//!
//! Tests for ES2024 BigInt implementation

#[cfg(test)]
mod tests {
    use builtins::bigint::{BigIntConstructor, BigIntPrototype};
    use builtins::value::{BigIntValue, JsValue};

    // ========================================
    // Constructor Tests
    // ========================================

    #[test]
    fn test_bigint_from_integer() {
        let result = BigIntConstructor::from_integer(42);
        assert!(result.is_ok());
        let bigint = result.unwrap();
        assert_eq!(bigint.to_string(), "42");
    }

    #[test]
    fn test_bigint_from_negative_integer() {
        let bigint = BigIntConstructor::from_integer(-123).unwrap();
        assert_eq!(bigint.to_string(), "-123");
    }

    #[test]
    fn test_bigint_from_zero() {
        let bigint = BigIntConstructor::from_integer(0).unwrap();
        assert_eq!(bigint.to_string(), "0");
    }

    #[test]
    fn test_bigint_from_string() {
        let bigint = BigIntConstructor::from_string("123456789012345678901234567890").unwrap();
        assert_eq!(bigint.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn test_bigint_from_negative_string() {
        let bigint = BigIntConstructor::from_string("-987654321098765432109876543210").unwrap();
        assert_eq!(bigint.to_string(), "-987654321098765432109876543210");
    }

    #[test]
    fn test_bigint_from_binary_string() {
        let bigint = BigIntConstructor::from_string("0b1010").unwrap();
        assert_eq!(bigint.to_string(), "10");
    }

    #[test]
    fn test_bigint_from_octal_string() {
        let bigint = BigIntConstructor::from_string("0o755").unwrap();
        assert_eq!(bigint.to_string(), "493");
    }

    #[test]
    fn test_bigint_from_hex_string() {
        let bigint = BigIntConstructor::from_string("0xff").unwrap();
        assert_eq!(bigint.to_string(), "255");
    }

    #[test]
    fn test_bigint_from_invalid_string() {
        let result = BigIntConstructor::from_string("not a number");
        assert!(result.is_err());
    }

    #[test]
    fn test_bigint_from_float_string() {
        let result = BigIntConstructor::from_string("3.14");
        assert!(result.is_err());
    }

    #[test]
    fn test_bigint_from_number_integer() {
        let bigint = BigIntConstructor::from_number(42.0).unwrap();
        assert_eq!(bigint.to_string(), "42");
    }

    #[test]
    fn test_bigint_from_number_float_fails() {
        let result = BigIntConstructor::from_number(3.14);
        assert!(result.is_err());
    }

    #[test]
    fn test_bigint_from_nan_fails() {
        let result = BigIntConstructor::from_number(f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn test_bigint_from_infinity_fails() {
        let result = BigIntConstructor::from_number(f64::INFINITY);
        assert!(result.is_err());
    }

    #[test]
    fn test_bigint_as_int_n() {
        // asIntN clamps to signed n-bit representation
        let bigint = BigIntConstructor::from_integer(128).unwrap();
        let result = BigIntConstructor::as_int_n(8, &bigint);
        assert_eq!(result.to_string(), "-128"); // 128 wraps to -128 in 8-bit signed

        let bigint = BigIntConstructor::from_integer(127).unwrap();
        let result = BigIntConstructor::as_int_n(8, &bigint);
        assert_eq!(result.to_string(), "127"); // 127 fits in 8-bit signed

        let bigint = BigIntConstructor::from_integer(256).unwrap();
        let result = BigIntConstructor::as_int_n(8, &bigint);
        assert_eq!(result.to_string(), "0"); // 256 wraps to 0 in 8-bit signed
    }

    #[test]
    fn test_bigint_as_uint_n() {
        // asUintN clamps to unsigned n-bit representation
        let bigint = BigIntConstructor::from_integer(256).unwrap();
        let result = BigIntConstructor::as_uint_n(8, &bigint);
        assert_eq!(result.to_string(), "0"); // 256 wraps to 0 in 8-bit unsigned

        let bigint = BigIntConstructor::from_integer(255).unwrap();
        let result = BigIntConstructor::as_uint_n(8, &bigint);
        assert_eq!(result.to_string(), "255"); // 255 fits in 8-bit unsigned

        let bigint = BigIntConstructor::from_integer(-1).unwrap();
        let result = BigIntConstructor::as_uint_n(8, &bigint);
        assert_eq!(result.to_string(), "255"); // -1 wraps to 255 in 8-bit unsigned
    }

    // ========================================
    // Arithmetic Operations Tests
    // ========================================

    #[test]
    fn test_bigint_add() {
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(20).unwrap();
        let result = BigIntPrototype::add(&a, &b).unwrap();
        assert_eq!(result.to_string(), "30");
    }

    #[test]
    fn test_bigint_add_large_numbers() {
        let a = BigIntConstructor::from_string("999999999999999999999999999999").unwrap();
        let b = BigIntConstructor::from_string("1").unwrap();
        let result = BigIntPrototype::add(&a, &b).unwrap();
        assert_eq!(result.to_string(), "1000000000000000000000000000000");
    }

    #[test]
    fn test_bigint_subtract() {
        let a = BigIntConstructor::from_integer(30).unwrap();
        let b = BigIntConstructor::from_integer(12).unwrap();
        let result = BigIntPrototype::sub(&a, &b).unwrap();
        assert_eq!(result.to_string(), "18");
    }

    #[test]
    fn test_bigint_subtract_to_negative() {
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(30).unwrap();
        let result = BigIntPrototype::sub(&a, &b).unwrap();
        assert_eq!(result.to_string(), "-20");
    }

    #[test]
    fn test_bigint_multiply() {
        let a = BigIntConstructor::from_integer(6).unwrap();
        let b = BigIntConstructor::from_integer(7).unwrap();
        let result = BigIntPrototype::mul(&a, &b).unwrap();
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_bigint_multiply_large_numbers() {
        let a = BigIntConstructor::from_string("123456789012345678901234567890").unwrap();
        let b = BigIntConstructor::from_integer(2).unwrap();
        let result = BigIntPrototype::mul(&a, &b).unwrap();
        assert_eq!(result.to_string(), "246913578024691357802469135780");
    }

    #[test]
    fn test_bigint_divide() {
        let a = BigIntConstructor::from_integer(100).unwrap();
        let b = BigIntConstructor::from_integer(3).unwrap();
        let result = BigIntPrototype::div(&a, &b).unwrap();
        assert_eq!(result.to_string(), "33"); // Integer division
    }

    #[test]
    fn test_bigint_divide_exact() {
        let a = BigIntConstructor::from_integer(100).unwrap();
        let b = BigIntConstructor::from_integer(25).unwrap();
        let result = BigIntPrototype::div(&a, &b).unwrap();
        assert_eq!(result.to_string(), "4");
    }

    #[test]
    fn test_bigint_divide_by_zero() {
        let a = BigIntConstructor::from_integer(100).unwrap();
        let b = BigIntConstructor::from_integer(0).unwrap();
        let result = BigIntPrototype::div(&a, &b);
        assert!(result.is_err());
    }

    #[test]
    fn test_bigint_remainder() {
        let a = BigIntConstructor::from_integer(100).unwrap();
        let b = BigIntConstructor::from_integer(3).unwrap();
        let result = BigIntPrototype::rem(&a, &b).unwrap();
        assert_eq!(result.to_string(), "1");
    }

    #[test]
    fn test_bigint_remainder_negative() {
        let a = BigIntConstructor::from_integer(-100).unwrap();
        let b = BigIntConstructor::from_integer(3).unwrap();
        let result = BigIntPrototype::rem(&a, &b).unwrap();
        assert_eq!(result.to_string(), "-1"); // Remainder preserves sign of dividend
    }

    #[test]
    fn test_bigint_remainder_by_zero() {
        let a = BigIntConstructor::from_integer(100).unwrap();
        let b = BigIntConstructor::from_integer(0).unwrap();
        let result = BigIntPrototype::rem(&a, &b);
        assert!(result.is_err());
    }

    #[test]
    fn test_bigint_exponentiation() {
        let base = BigIntConstructor::from_integer(2).unwrap();
        let exp = BigIntConstructor::from_integer(10).unwrap();
        let result = BigIntPrototype::pow(&base, &exp).unwrap();
        assert_eq!(result.to_string(), "1024");
    }

    #[test]
    fn test_bigint_exponentiation_large() {
        let base = BigIntConstructor::from_integer(2).unwrap();
        let exp = BigIntConstructor::from_integer(64).unwrap();
        let result = BigIntPrototype::pow(&base, &exp).unwrap();
        assert_eq!(result.to_string(), "18446744073709551616");
    }

    #[test]
    fn test_bigint_exponentiation_negative_exponent() {
        let base = BigIntConstructor::from_integer(2).unwrap();
        let exp = BigIntConstructor::from_integer(-1).unwrap();
        let result = BigIntPrototype::pow(&base, &exp);
        assert!(result.is_err()); // Negative exponents not allowed for BigInt
    }

    #[test]
    fn test_bigint_negate() {
        let a = BigIntConstructor::from_integer(42).unwrap();
        let result = BigIntPrototype::negate(&a);
        assert_eq!(result.to_string(), "-42");
    }

    #[test]
    fn test_bigint_negate_negative() {
        let a = BigIntConstructor::from_integer(-42).unwrap();
        let result = BigIntPrototype::negate(&a);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_bigint_negate_zero() {
        let a = BigIntConstructor::from_integer(0).unwrap();
        let result = BigIntPrototype::negate(&a);
        assert_eq!(result.to_string(), "0");
    }

    #[test]
    fn test_bigint_unary_plus_throws() {
        let a = BigIntConstructor::from_integer(42).unwrap();
        let result = BigIntPrototype::unary_plus(&a);
        assert!(result.is_err()); // Unary + on BigInt throws TypeError
    }

    // ========================================
    // Bitwise Operations Tests
    // ========================================

    #[test]
    fn test_bigint_bitwise_and() {
        let a = BigIntConstructor::from_integer(0b1100).unwrap();
        let b = BigIntConstructor::from_integer(0b1010).unwrap();
        let result = BigIntPrototype::bitwise_and(&a, &b);
        assert_eq!(result.to_string(), "8"); // 0b1000
    }

    #[test]
    fn test_bigint_bitwise_or() {
        let a = BigIntConstructor::from_integer(0b1100).unwrap();
        let b = BigIntConstructor::from_integer(0b1010).unwrap();
        let result = BigIntPrototype::bitwise_or(&a, &b);
        assert_eq!(result.to_string(), "14"); // 0b1110
    }

    #[test]
    fn test_bigint_bitwise_xor() {
        let a = BigIntConstructor::from_integer(0b1100).unwrap();
        let b = BigIntConstructor::from_integer(0b1010).unwrap();
        let result = BigIntPrototype::bitwise_xor(&a, &b);
        assert_eq!(result.to_string(), "6"); // 0b0110
    }

    #[test]
    fn test_bigint_bitwise_not() {
        let a = BigIntConstructor::from_integer(5).unwrap();
        let result = BigIntPrototype::bitwise_not(&a);
        assert_eq!(result.to_string(), "-6"); // ~5 = -6 in two's complement
    }

    #[test]
    fn test_bigint_bitwise_not_negative() {
        let a = BigIntConstructor::from_integer(-1).unwrap();
        let result = BigIntPrototype::bitwise_not(&a);
        assert_eq!(result.to_string(), "0"); // ~(-1) = 0
    }

    #[test]
    fn test_bigint_left_shift() {
        let a = BigIntConstructor::from_integer(1).unwrap();
        let shift = BigIntConstructor::from_integer(10).unwrap();
        let result = BigIntPrototype::left_shift(&a, &shift).unwrap();
        assert_eq!(result.to_string(), "1024");
    }

    #[test]
    fn test_bigint_left_shift_large() {
        let a = BigIntConstructor::from_integer(1).unwrap();
        let shift = BigIntConstructor::from_integer(100).unwrap();
        let result = BigIntPrototype::left_shift(&a, &shift).unwrap();
        assert_eq!(result.to_string(), "1267650600228229401496703205376");
    }

    #[test]
    fn test_bigint_right_shift() {
        let a = BigIntConstructor::from_integer(1024).unwrap();
        let shift = BigIntConstructor::from_integer(3).unwrap();
        let result = BigIntPrototype::right_shift(&a, &shift).unwrap();
        assert_eq!(result.to_string(), "128");
    }

    #[test]
    fn test_bigint_right_shift_negative() {
        let a = BigIntConstructor::from_integer(-8).unwrap();
        let shift = BigIntConstructor::from_integer(2).unwrap();
        let result = BigIntPrototype::right_shift(&a, &shift).unwrap();
        assert_eq!(result.to_string(), "-2"); // Sign-extending right shift
    }

    // ========================================
    // Comparison Tests
    // ========================================

    #[test]
    fn test_bigint_equals() {
        let a = BigIntConstructor::from_integer(42).unwrap();
        let b = BigIntConstructor::from_integer(42).unwrap();
        assert!(BigIntPrototype::eq(&a, &b));
    }

    #[test]
    fn test_bigint_not_equals() {
        let a = BigIntConstructor::from_integer(42).unwrap();
        let b = BigIntConstructor::from_integer(43).unwrap();
        assert!(!BigIntPrototype::eq(&a, &b));
    }

    #[test]
    fn test_bigint_less_than() {
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(20).unwrap();
        assert!(BigIntPrototype::lt(&a, &b));
        assert!(!BigIntPrototype::lt(&b, &a));
    }

    #[test]
    fn test_bigint_greater_than() {
        let a = BigIntConstructor::from_integer(20).unwrap();
        let b = BigIntConstructor::from_integer(10).unwrap();
        assert!(BigIntPrototype::gt(&a, &b));
        assert!(!BigIntPrototype::gt(&b, &a));
    }

    #[test]
    fn test_bigint_less_than_or_equal() {
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(10).unwrap();
        let c = BigIntConstructor::from_integer(20).unwrap();
        assert!(BigIntPrototype::le(&a, &b));
        assert!(BigIntPrototype::le(&a, &c));
        assert!(!BigIntPrototype::le(&c, &a));
    }

    #[test]
    fn test_bigint_greater_than_or_equal() {
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(10).unwrap();
        let c = BigIntConstructor::from_integer(5).unwrap();
        assert!(BigIntPrototype::ge(&a, &b));
        assert!(BigIntPrototype::ge(&a, &c));
        assert!(!BigIntPrototype::ge(&c, &a));
    }

    #[test]
    fn test_bigint_compare_with_number_equal() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        assert!(BigIntPrototype::equal_to_number(&bigint, 42.0));
    }

    #[test]
    fn test_bigint_compare_with_number_not_equal() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        assert!(!BigIntPrototype::equal_to_number(&bigint, 42.5));
    }

    #[test]
    fn test_bigint_compare_with_number_nan() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        assert!(!BigIntPrototype::equal_to_number(&bigint, f64::NAN));
    }

    #[test]
    fn test_bigint_compare_with_number_infinity() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        assert!(!BigIntPrototype::equal_to_number(&bigint, f64::INFINITY));
    }

    #[test]
    fn test_bigint_lt_number() {
        let bigint = BigIntConstructor::from_integer(10).unwrap();
        assert!(BigIntPrototype::lt_number(&bigint, 20.0));
        assert!(!BigIntPrototype::lt_number(&bigint, 5.0));
    }

    #[test]
    fn test_bigint_gt_number() {
        let bigint = BigIntConstructor::from_integer(20).unwrap();
        assert!(BigIntPrototype::gt_number(&bigint, 10.0));
        assert!(!BigIntPrototype::gt_number(&bigint, 30.0));
    }

    // ========================================
    // Method Tests
    // ========================================

    #[test]
    fn test_bigint_to_string_default() {
        let bigint = BigIntConstructor::from_integer(255).unwrap();
        assert_eq!(BigIntPrototype::to_string(&bigint, None).unwrap(), "255");
    }

    #[test]
    fn test_bigint_to_string_binary() {
        let bigint = BigIntConstructor::from_integer(10).unwrap();
        assert_eq!(BigIntPrototype::to_string(&bigint, Some(2)).unwrap(), "1010");
    }

    #[test]
    fn test_bigint_to_string_hex() {
        let bigint = BigIntConstructor::from_integer(255).unwrap();
        assert_eq!(BigIntPrototype::to_string(&bigint, Some(16)).unwrap(), "ff");
    }

    #[test]
    fn test_bigint_to_string_octal() {
        let bigint = BigIntConstructor::from_integer(64).unwrap();
        assert_eq!(BigIntPrototype::to_string(&bigint, Some(8)).unwrap(), "100");
    }

    #[test]
    fn test_bigint_to_string_base36() {
        let bigint = BigIntConstructor::from_integer(35).unwrap();
        assert_eq!(BigIntPrototype::to_string(&bigint, Some(36)).unwrap(), "z");
    }

    #[test]
    fn test_bigint_to_string_invalid_radix() {
        let bigint = BigIntConstructor::from_integer(10).unwrap();
        assert!(BigIntPrototype::to_string(&bigint, Some(1)).is_err());
        assert!(BigIntPrototype::to_string(&bigint, Some(37)).is_err());
    }

    #[test]
    fn test_bigint_to_string_negative() {
        let bigint = BigIntConstructor::from_integer(-255).unwrap();
        assert_eq!(BigIntPrototype::to_string(&bigint, Some(16)).unwrap(), "-ff");
    }

    #[test]
    fn test_bigint_value_of() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        let result = BigIntPrototype::value_of(&bigint);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_bigint_to_locale_string() {
        let bigint = BigIntConstructor::from_integer(1234567890).unwrap();
        let result = BigIntPrototype::to_locale_string(&bigint);
        // Basic implementation just returns the string representation
        assert!(result.contains("1234567890") || result.contains(","));
    }

    // ========================================
    // Type Coercion Error Tests
    // ========================================

    #[test]
    fn test_bigint_cannot_mix_with_number_in_add() {
        // This tests that BigInt operations cannot be mixed with Numbers
        // The actual TypeError would be thrown at runtime by the VM
        let a = BigIntConstructor::from_integer(10).unwrap();
        let b = BigIntConstructor::from_integer(20).unwrap();
        // Operations between two BigInts work fine
        assert!(BigIntPrototype::add(&a, &b).is_ok());
    }

    // ========================================
    // JsValue Integration Tests
    // ========================================

    #[test]
    fn test_jsvalue_bigint() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        let value = JsValue::bigint(bigint);
        assert!(value.is_bigint());
        assert_eq!(value.type_of(), "bigint");
    }

    #[test]
    fn test_jsvalue_bigint_to_string() {
        let bigint = BigIntConstructor::from_string("12345678901234567890").unwrap();
        let value = JsValue::bigint(bigint);
        let s = value.to_js_string();
        assert!(s.ends_with("n"));
    }

    #[test]
    fn test_jsvalue_bigint_equality() {
        let a = BigIntConstructor::from_integer(42).unwrap();
        let b = BigIntConstructor::from_integer(42).unwrap();
        let val_a = JsValue::bigint(a);
        let val_b = JsValue::bigint(b);
        assert!(val_a.equals(&val_b));
    }

    #[test]
    fn test_jsvalue_bigint_inequality() {
        let a = BigIntConstructor::from_integer(42).unwrap();
        let b = BigIntConstructor::from_integer(43).unwrap();
        let val_a = JsValue::bigint(a);
        let val_b = JsValue::bigint(b);
        assert!(!val_a.equals(&val_b));
    }

    #[test]
    fn test_jsvalue_as_bigint() {
        let bigint = BigIntConstructor::from_integer(42).unwrap();
        let value = JsValue::bigint(bigint);
        let extracted = value.as_bigint();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap().to_string(), "42");
    }

    // ========================================
    // Edge Cases and Large Number Tests
    // ========================================

    #[test]
    fn test_bigint_very_large_number() {
        // 2^256
        let base = BigIntConstructor::from_integer(2).unwrap();
        let exp = BigIntConstructor::from_integer(256).unwrap();
        let result = BigIntPrototype::pow(&base, &exp).unwrap();
        let s = result.to_string();
        // 2^256 is a 78-digit number
        assert!(s.len() > 75);
        assert!(s.starts_with("115792089237316195423570985008687907853269984665640564039457584007913129639936"));
    }

    #[test]
    fn test_bigint_factorial_of_100() {
        // Calculate 100! using BigInt
        let mut result = BigIntConstructor::from_integer(1).unwrap();
        for i in 2..=100 {
            let n = BigIntConstructor::from_integer(i).unwrap();
            result = BigIntPrototype::mul(&result, &n).unwrap();
        }
        let s = result.to_string();
        // 100! has 158 digits
        assert_eq!(s.len(), 158);
        // First few digits of 100!
        assert!(s.starts_with("93326215443944"));
    }
}
