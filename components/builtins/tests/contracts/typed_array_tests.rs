//! Contract tests for TypedArray family
//!
//! These tests verify the public API for ArrayBuffer, TypedArray variants, and DataView.

use builtins::{
    ArrayBuffer, DataView, TypedArray, TypedArrayKind, TypedArrayValue,
};

// =============================================================================
// ArrayBuffer Contract Tests
// =============================================================================

#[test]
fn contract_array_buffer_constructor() {
    let buf = ArrayBuffer::new(16).unwrap();
    assert_eq!(buf.byte_length(), 16);
}

#[test]
fn contract_array_buffer_byte_length_getter() {
    let buf = ArrayBuffer::new(100).unwrap();
    assert_eq!(buf.byte_length(), 100);
}

#[test]
fn contract_array_buffer_slice() {
    let buf = ArrayBuffer::new(20).unwrap();
    let sliced = buf.slice(5, Some(15)).unwrap();
    assert_eq!(sliced.byte_length(), 10);
}

#[test]
fn contract_array_buffer_slice_with_negative_indices() {
    let buf = ArrayBuffer::new(10).unwrap();
    let sliced = buf.slice(-5, Some(-2)).unwrap();
    assert_eq!(sliced.byte_length(), 3);
}

#[test]
fn contract_array_buffer_is_view() {
    // Static method that checks if an object is a view
    let not_view = 42;
    assert!(!ArrayBuffer::is_view(&not_view));
}

// =============================================================================
// TypedArray Base Contract Tests
// =============================================================================

#[test]
fn contract_typed_array_length_getter() {
    let arr = TypedArray::new(TypedArrayKind::Int32, 10).unwrap();
    assert_eq!(arr.length(), 10);
}

#[test]
fn contract_typed_array_byte_length_getter() {
    let arr = TypedArray::new(TypedArrayKind::Int32, 10).unwrap();
    assert_eq!(arr.byte_length(), 40); // 10 * 4 bytes
}

#[test]
fn contract_typed_array_byte_offset_getter() {
    let buffer = ArrayBuffer::new(16).unwrap();
    let arr = TypedArray::from_buffer(buffer, TypedArrayKind::Int32, Some(4), None).unwrap();
    assert_eq!(arr.byte_offset(), 4);
}

#[test]
fn contract_typed_array_buffer_getter() {
    let arr = TypedArray::new(TypedArrayKind::Float64, 5).unwrap();
    let buf = arr.buffer();
    assert_eq!(buf.byte_length(), 40); // 5 * 8 bytes
}

#[test]
fn contract_typed_array_get_set() {
    let mut arr = TypedArray::new(TypedArrayKind::Int8, 3).unwrap();
    arr.set(0, TypedArrayValue::Number(42.0)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), 42.0);
}

#[test]
fn contract_typed_array_slice() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Uint8,
        vec![
            TypedArrayValue::Number(1.0),
            TypedArrayValue::Number(2.0),
            TypedArrayValue::Number(3.0),
            TypedArrayValue::Number(4.0),
        ],
    )
    .unwrap();
    let sliced = arr.slice(1, Some(3)).unwrap();
    assert_eq!(sliced.length(), 2);
    assert_eq!(sliced.get(0).unwrap().as_f64(), 2.0);
}

#[test]
fn contract_typed_array_subarray() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Int16,
        vec![
            TypedArrayValue::Number(10.0),
            TypedArrayValue::Number(20.0),
            TypedArrayValue::Number(30.0),
        ],
    )
    .unwrap();
    let sub = arr.subarray(1, None).unwrap();
    assert_eq!(sub.length(), 2);
    assert_eq!(sub.byte_offset(), 2); // 1 * 2 bytes
}

#[test]
fn contract_typed_array_map() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Uint8,
        vec![
            TypedArrayValue::Number(1.0),
            TypedArrayValue::Number(2.0),
        ],
    )
    .unwrap();
    let mapped = arr.map(|v, _| Ok(TypedArrayValue::Number(v.as_f64() * 10.0))).unwrap();
    assert_eq!(mapped.get(0).unwrap().as_f64(), 10.0);
    assert_eq!(mapped.get(1).unwrap().as_f64(), 20.0);
}

#[test]
fn contract_typed_array_filter() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Int8,
        vec![
            TypedArrayValue::Number(1.0),
            TypedArrayValue::Number(5.0),
            TypedArrayValue::Number(3.0),
        ],
    )
    .unwrap();
    let filtered = arr.filter(|v, _| Ok(v.as_f64() > 2.0)).unwrap();
    assert_eq!(filtered.length(), 2);
}

#[test]
fn contract_typed_array_reduce() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Int32,
        vec![
            TypedArrayValue::Number(1.0),
            TypedArrayValue::Number(2.0),
            TypedArrayValue::Number(3.0),
        ],
    )
    .unwrap();
    let sum = arr.reduce(TypedArrayValue::Number(0.0), |acc, v, _| {
        Ok(TypedArrayValue::Number(acc.as_f64() + v.as_f64()))
    }).unwrap();
    assert_eq!(sum.as_f64(), 6.0);
}

#[test]
fn contract_typed_array_for_each() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Uint8,
        vec![
            TypedArrayValue::Number(1.0),
            TypedArrayValue::Number(2.0),
        ],
    )
    .unwrap();
    let count = std::cell::RefCell::new(0);
    arr.for_each(|_, _| {
        *count.borrow_mut() += 1;
        Ok(())
    }).unwrap();
    assert_eq!(*count.borrow(), 2);
}

#[test]
fn contract_typed_array_index_of() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Int8,
        vec![
            TypedArrayValue::Number(5.0),
            TypedArrayValue::Number(10.0),
            TypedArrayValue::Number(15.0),
        ],
    )
    .unwrap();
    assert_eq!(arr.index_of(&TypedArrayValue::Number(10.0)).unwrap(), Some(1));
    assert_eq!(arr.index_of(&TypedArrayValue::Number(20.0)).unwrap(), None);
}

#[test]
fn contract_typed_array_includes() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Uint16,
        vec![
            TypedArrayValue::Number(100.0),
            TypedArrayValue::Number(200.0),
        ],
    )
    .unwrap();
    assert!(arr.includes(&TypedArrayValue::Number(200.0)).unwrap());
    assert!(!arr.includes(&TypedArrayValue::Number(300.0)).unwrap());
}

#[test]
fn contract_typed_array_find() {
    let arr = TypedArray::from_values(
        TypedArrayKind::Float32,
        vec![
            TypedArrayValue::Number(1.5),
            TypedArrayValue::Number(2.5),
            TypedArrayValue::Number(3.5),
        ],
    )
    .unwrap();
    let found = arr.find(|v, _| Ok(v.as_f64() > 2.0)).unwrap();
    assert!(found.is_some());
    assert!((found.unwrap().as_f64() - 2.5).abs() < 0.01);
}

#[test]
fn contract_typed_array_fill() {
    let mut arr = TypedArray::new(TypedArrayKind::Int8, 5).unwrap();
    arr.fill(TypedArrayValue::Number(7.0), None, None).unwrap();
    for i in 0..5 {
        assert_eq!(arr.get(i).unwrap().as_f64(), 7.0);
    }
}

#[test]
fn contract_typed_array_copy_within() {
    let mut arr = TypedArray::from_values(
        TypedArrayKind::Uint8,
        vec![
            TypedArrayValue::Number(1.0),
            TypedArrayValue::Number(2.0),
            TypedArrayValue::Number(3.0),
            TypedArrayValue::Number(4.0),
        ],
    )
    .unwrap();
    arr.copy_within(0, 2, None).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), 3.0);
    assert_eq!(arr.get(1).unwrap().as_f64(), 4.0);
}

#[test]
fn contract_typed_array_reverse() {
    let mut arr = TypedArray::from_values(
        TypedArrayKind::Int16,
        vec![
            TypedArrayValue::Number(1.0),
            TypedArrayValue::Number(2.0),
            TypedArrayValue::Number(3.0),
        ],
    )
    .unwrap();
    arr.reverse().unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), 3.0);
    assert_eq!(arr.get(2).unwrap().as_f64(), 1.0);
}

#[test]
fn contract_typed_array_sort() {
    let mut arr = TypedArray::from_values(
        TypedArrayKind::Float64,
        vec![
            TypedArrayValue::Number(3.14),
            TypedArrayValue::Number(1.41),
            TypedArrayValue::Number(2.71),
        ],
    )
    .unwrap();
    arr.sort().unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), 1.41);
    assert_eq!(arr.get(1).unwrap().as_f64(), 2.71);
    assert_eq!(arr.get(2).unwrap().as_f64(), 3.14);
}

// =============================================================================
// Int8Array Specific Tests
// =============================================================================

#[test]
fn contract_int8_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::Int8, 2).unwrap();
    arr.set(0, TypedArrayValue::Number(-128.0)).unwrap();
    arr.set(1, TypedArrayValue::Number(127.0)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), -128.0);
    assert_eq!(arr.get(1).unwrap().as_f64(), 127.0);
}

#[test]
fn contract_int8_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Int8.bytes_per_element(), 1);
}

// =============================================================================
// Uint8Array Specific Tests
// =============================================================================

#[test]
fn contract_uint8_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::Uint8, 2).unwrap();
    arr.set(0, TypedArrayValue::Number(0.0)).unwrap();
    arr.set(1, TypedArrayValue::Number(255.0)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);
    assert_eq!(arr.get(1).unwrap().as_f64(), 255.0);
}

#[test]
fn contract_uint8_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Uint8.bytes_per_element(), 1);
}

// =============================================================================
// Uint8ClampedArray Specific Tests
// =============================================================================

#[test]
fn contract_uint8_clamped_array_clamping() {
    let mut arr = TypedArray::new(TypedArrayKind::Uint8Clamped, 3).unwrap();
    arr.set(0, TypedArrayValue::Number(-50.0)).unwrap();
    arr.set(1, TypedArrayValue::Number(150.5)).unwrap();
    arr.set(2, TypedArrayValue::Number(500.0)).unwrap();

    assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);   // Clamped to 0
    assert_eq!(arr.get(1).unwrap().as_f64(), 151.0); // Rounded to 151
    assert_eq!(arr.get(2).unwrap().as_f64(), 255.0); // Clamped to 255
}

#[test]
fn contract_uint8_clamped_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Uint8Clamped.bytes_per_element(), 1);
}

// =============================================================================
// Int16Array Specific Tests
// =============================================================================

#[test]
fn contract_int16_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::Int16, 2).unwrap();
    arr.set(0, TypedArrayValue::Number(-32768.0)).unwrap();
    arr.set(1, TypedArrayValue::Number(32767.0)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), -32768.0);
    assert_eq!(arr.get(1).unwrap().as_f64(), 32767.0);
}

#[test]
fn contract_int16_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Int16.bytes_per_element(), 2);
}

// =============================================================================
// Uint16Array Specific Tests
// =============================================================================

#[test]
fn contract_uint16_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::Uint16, 2).unwrap();
    arr.set(0, TypedArrayValue::Number(0.0)).unwrap();
    arr.set(1, TypedArrayValue::Number(65535.0)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);
    assert_eq!(arr.get(1).unwrap().as_f64(), 65535.0);
}

#[test]
fn contract_uint16_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Uint16.bytes_per_element(), 2);
}

// =============================================================================
// Int32Array Specific Tests
// =============================================================================

#[test]
fn contract_int32_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::Int32, 2).unwrap();
    arr.set(0, TypedArrayValue::Number(-2147483648.0)).unwrap();
    arr.set(1, TypedArrayValue::Number(2147483647.0)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), -2147483648.0);
    assert_eq!(arr.get(1).unwrap().as_f64(), 2147483647.0);
}

#[test]
fn contract_int32_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Int32.bytes_per_element(), 4);
}

// =============================================================================
// Uint32Array Specific Tests
// =============================================================================

#[test]
fn contract_uint32_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::Uint32, 2).unwrap();
    arr.set(0, TypedArrayValue::Number(0.0)).unwrap();
    arr.set(1, TypedArrayValue::Number(4294967295.0)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);
    assert_eq!(arr.get(1).unwrap().as_f64(), 4294967295.0);
}

#[test]
fn contract_uint32_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Uint32.bytes_per_element(), 4);
}

// =============================================================================
// Float32Array Specific Tests
// =============================================================================

#[test]
fn contract_float32_array_precision() {
    let mut arr = TypedArray::new(TypedArrayKind::Float32, 1).unwrap();
    arr.set(0, TypedArrayValue::Number(3.141592653589793)).unwrap();
    let val = arr.get(0).unwrap().as_f64();
    // Float32 has less precision than Float64
    assert!((val - 3.14159).abs() < 0.00001);
}

#[test]
fn contract_float32_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Float32.bytes_per_element(), 4);
}

#[test]
fn contract_float32_array_special_values() {
    let mut arr = TypedArray::new(TypedArrayKind::Float32, 3).unwrap();
    arr.set(0, TypedArrayValue::Number(f64::INFINITY)).unwrap();
    arr.set(1, TypedArrayValue::Number(f64::NEG_INFINITY)).unwrap();
    arr.set(2, TypedArrayValue::Number(f64::NAN)).unwrap();

    assert!(arr.get(0).unwrap().as_f64().is_infinite());
    assert!(arr.get(0).unwrap().as_f64() > 0.0);
    assert!(arr.get(1).unwrap().as_f64().is_infinite());
    assert!(arr.get(1).unwrap().as_f64() < 0.0);
    assert!(arr.get(2).unwrap().as_f64().is_nan());
}

// =============================================================================
// Float64Array Specific Tests
// =============================================================================

#[test]
fn contract_float64_array_precision() {
    let mut arr = TypedArray::new(TypedArrayKind::Float64, 1).unwrap();
    let pi = std::f64::consts::PI;
    arr.set(0, TypedArrayValue::Number(pi)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_f64(), pi);
}

#[test]
fn contract_float64_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::Float64.bytes_per_element(), 8);
}

// =============================================================================
// BigInt64Array Specific Tests
// =============================================================================

#[test]
fn contract_bigint64_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::BigInt64, 2).unwrap();
    arr.set(0, TypedArrayValue::BigInt(i64::MIN as i128)).unwrap();
    arr.set(1, TypedArrayValue::BigInt(i64::MAX as i128)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_i128(), i64::MIN as i128);
    assert_eq!(arr.get(1).unwrap().as_i128(), i64::MAX as i128);
}

#[test]
fn contract_bigint64_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::BigInt64.bytes_per_element(), 8);
}

// =============================================================================
// BigUint64Array Specific Tests
// =============================================================================

#[test]
fn contract_biguint64_array_range() {
    let mut arr = TypedArray::new(TypedArrayKind::BigUint64, 2).unwrap();
    arr.set(0, TypedArrayValue::BigInt(0)).unwrap();
    arr.set(1, TypedArrayValue::BigInt(u64::MAX as i128)).unwrap();
    assert_eq!(arr.get(0).unwrap().as_i128(), 0);
    assert_eq!(arr.get(1).unwrap().as_i128(), u64::MAX as i128);
}

#[test]
fn contract_biguint64_array_bytes_per_element() {
    assert_eq!(TypedArrayKind::BigUint64.bytes_per_element(), 8);
}

// =============================================================================
// DataView Contract Tests
// =============================================================================

#[test]
fn contract_data_view_constructor() {
    let buffer = ArrayBuffer::new(16).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    assert_eq!(view.byte_length(), 16);
}

#[test]
fn contract_data_view_with_offset_and_length() {
    let buffer = ArrayBuffer::new(20).unwrap();
    let view = DataView::new(buffer, Some(5), Some(10)).unwrap();
    assert_eq!(view.byte_offset(), 5);
    assert_eq!(view.byte_length(), 10);
}

#[test]
fn contract_data_view_buffer_getter() {
    let buffer = ArrayBuffer::new(8).unwrap();
    let view = DataView::new(buffer.clone(), None, None).unwrap();
    assert_eq!(view.buffer().byte_length(), 8);
}

#[test]
fn contract_data_view_get_set_int8() {
    let buffer = ArrayBuffer::new(4).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_int8(0, -100).unwrap();
    assert_eq!(view.get_int8(0).unwrap(), -100);
}

#[test]
fn contract_data_view_get_set_uint8() {
    let buffer = ArrayBuffer::new(4).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_uint8(0, 200).unwrap();
    assert_eq!(view.get_uint8(0).unwrap(), 200);
}

#[test]
fn contract_data_view_get_set_int16_little_endian() {
    let buffer = ArrayBuffer::new(4).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_int16(0, -1000, true).unwrap();
    assert_eq!(view.get_int16(0, true).unwrap(), -1000);
}

#[test]
fn contract_data_view_get_set_int16_big_endian() {
    let buffer = ArrayBuffer::new(4).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_int16(0, -1000, false).unwrap();
    assert_eq!(view.get_int16(0, false).unwrap(), -1000);
}

#[test]
fn contract_data_view_get_set_uint16() {
    let buffer = ArrayBuffer::new(4).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_uint16(0, 50000, true).unwrap();
    assert_eq!(view.get_uint16(0, true).unwrap(), 50000);
}

#[test]
fn contract_data_view_get_set_int32() {
    let buffer = ArrayBuffer::new(8).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_int32(0, -100000, false).unwrap();
    assert_eq!(view.get_int32(0, false).unwrap(), -100000);
}

#[test]
fn contract_data_view_get_set_uint32() {
    let buffer = ArrayBuffer::new(8).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_uint32(0, 3000000000, true).unwrap();
    assert_eq!(view.get_uint32(0, true).unwrap(), 3000000000);
}

#[test]
fn contract_data_view_get_set_float32() {
    let buffer = ArrayBuffer::new(8).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_float32(0, 123.456, true).unwrap();
    let val = view.get_float32(0, true).unwrap();
    assert!((val - 123.456).abs() < 0.001);
}

#[test]
fn contract_data_view_get_set_float64() {
    let buffer = ArrayBuffer::new(16).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_float64(0, std::f64::consts::E, false).unwrap();
    assert_eq!(view.get_float64(0, false).unwrap(), std::f64::consts::E);
}

#[test]
fn contract_data_view_get_set_big_int64() {
    let buffer = ArrayBuffer::new(16).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_big_int64(0, -9000000000000000000, true).unwrap();
    assert_eq!(view.get_big_int64(0, true).unwrap(), -9000000000000000000);
}

#[test]
fn contract_data_view_get_set_big_uint64() {
    let buffer = ArrayBuffer::new(16).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    view.set_big_uint64(0, 18000000000000000000, false).unwrap();
    assert_eq!(view.get_big_uint64(0, false).unwrap(), 18000000000000000000);
}

#[test]
fn contract_data_view_endianness_matters() {
    let buffer = ArrayBuffer::new(4).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();

    // Write in little endian
    view.set_int32(0, 0x12345678, true).unwrap();

    // Read back - little endian should give same value
    assert_eq!(view.get_int32(0, true).unwrap(), 0x12345678);

    // Big endian should give different value (bytes reversed)
    assert_ne!(view.get_int32(0, false).unwrap(), 0x12345678);
}

// =============================================================================
// Error Handling Contract Tests
// =============================================================================

#[test]
fn contract_typed_array_out_of_bounds_get() {
    let arr = TypedArray::new(TypedArrayKind::Int8, 5).unwrap();
    assert!(arr.get(10).is_err());
}

#[test]
fn contract_typed_array_out_of_bounds_set() {
    let mut arr = TypedArray::new(TypedArrayKind::Int8, 5).unwrap();
    assert!(arr.set(10, TypedArrayValue::Number(1.0)).is_err());
}

#[test]
fn contract_data_view_out_of_bounds() {
    let buffer = ArrayBuffer::new(4).unwrap();
    let view = DataView::new(buffer, None, None).unwrap();
    assert!(view.get_int32(2, true).is_err()); // Not enough bytes
}

#[test]
fn contract_array_buffer_invalid_size() {
    let result = ArrayBuffer::new(usize::MAX);
    assert!(result.is_err());
}

#[test]
fn contract_typed_array_misaligned_offset() {
    let buffer = ArrayBuffer::new(16).unwrap();
    // Int32 requires 4-byte alignment, offset 3 is invalid
    let result = TypedArray::from_buffer(buffer, TypedArrayKind::Int32, Some(3), None);
    assert!(result.is_err());
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn contract_shared_buffer_between_typed_arrays() {
    let buffer = ArrayBuffer::new(8).unwrap();

    // Create two different views of the same buffer
    let mut u8_view = TypedArray::from_buffer(
        buffer.clone(),
        TypedArrayKind::Uint8,
        None,
        None,
    ).unwrap();

    let u32_view = TypedArray::from_buffer(
        buffer.clone(),
        TypedArrayKind::Uint32,
        None,
        None,
    ).unwrap();

    // Write via Uint8Array
    u8_view.set(0, TypedArrayValue::Number(0xFF as f64)).unwrap();
    u8_view.set(1, TypedArrayValue::Number(0x00 as f64)).unwrap();
    u8_view.set(2, TypedArrayValue::Number(0x00 as f64)).unwrap();
    u8_view.set(3, TypedArrayValue::Number(0x00 as f64)).unwrap();

    // Read via Uint32Array - depends on endianness
    let val = u32_view.get(0).unwrap().as_f64() as u32;
    // Either 0x000000FF (big endian) or 0xFF000000 (little endian is wrong here due to byte order)
    // On little-endian systems: bytes [0xFF, 0x00, 0x00, 0x00] as u32 = 255
    assert!(val == 255 || val == 0xFF000000);
}

#[test]
fn contract_data_view_and_typed_array_share_buffer() {
    let buffer = ArrayBuffer::new(8).unwrap();

    let view = DataView::new(buffer.clone(), None, None).unwrap();
    let arr = TypedArray::from_buffer(buffer, TypedArrayKind::Float64, None, None).unwrap();

    // Write via DataView
    view.set_float64(0, std::f64::consts::PI, true).unwrap();

    // Read via Float64Array
    let val = arr.get(0).unwrap().as_f64();
    assert_eq!(val, std::f64::consts::PI);
}

#[test]
fn contract_all_typed_array_kinds_have_names() {
    let kinds = vec![
        TypedArrayKind::Int8,
        TypedArrayKind::Uint8,
        TypedArrayKind::Uint8Clamped,
        TypedArrayKind::Int16,
        TypedArrayKind::Uint16,
        TypedArrayKind::Int32,
        TypedArrayKind::Uint32,
        TypedArrayKind::Float32,
        TypedArrayKind::Float64,
        TypedArrayKind::BigInt64,
        TypedArrayKind::BigUint64,
    ];

    for kind in kinds {
        let name = kind.name();
        assert!(!name.is_empty());
        assert!(name.ends_with("Array"));
    }
}
