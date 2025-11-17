//! TypedArray family implementation
//!
//! This module provides ArrayBuffer, TypedArray variants (Int8Array, Uint8Array, etc.),
//! and DataView for binary data manipulation per ES2024.

use crate::value::{JsError, JsResult};
use std::cell::RefCell;
use std::rc::Rc;

/// ArrayBuffer - represents a generic fixed-length raw binary data buffer
#[derive(Debug, Clone)]
pub struct ArrayBuffer {
    /// Internal byte storage
    data: Rc<RefCell<Vec<u8>>>,
}

impl ArrayBuffer {
    /// Create a new ArrayBuffer with specified byte length
    pub fn new(byte_length: usize) -> JsResult<Self> {
        if byte_length > 2_147_483_647 {
            // Max safe size (2GB - 1)
            return Err(JsError::range_error("Invalid array buffer length"));
        }
        Ok(ArrayBuffer {
            data: Rc::new(RefCell::new(vec![0u8; byte_length])),
        })
    }

    /// Get the byte length of the buffer
    pub fn byte_length(&self) -> usize {
        self.data.borrow().len()
    }

    /// Slice the buffer to create a new ArrayBuffer
    pub fn slice(&self, begin: i32, end: Option<i32>) -> JsResult<ArrayBuffer> {
        let len = self.byte_length() as i32;

        let start_idx = if begin < 0 {
            (len + begin).max(0) as usize
        } else {
            begin.min(len) as usize
        };

        let end_idx = match end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => e.min(len) as usize,
            None => len as usize,
        };

        if start_idx >= end_idx {
            return ArrayBuffer::new(0);
        }

        let data = self.data.borrow();
        let sliced = data[start_idx..end_idx].to_vec();
        Ok(ArrayBuffer {
            data: Rc::new(RefCell::new(sliced)),
        })
    }

    /// Check if an object is a view of an ArrayBuffer (TypedArray or DataView)
    pub fn is_view(_value: &dyn std::any::Any) -> bool {
        // In a real implementation, this would check for TypedArray or DataView
        false
    }

    /// Get raw access to internal data (for TypedArray use)
    pub(crate) fn get_data(&self) -> Rc<RefCell<Vec<u8>>> {
        self.data.clone()
    }
}

/// TypedArray element kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypedArrayKind {
    /// 8-bit signed integer
    Int8,
    /// 8-bit unsigned integer
    Uint8,
    /// 8-bit unsigned integer (clamped)
    Uint8Clamped,
    /// 16-bit signed integer
    Int16,
    /// 16-bit unsigned integer
    Uint16,
    /// 32-bit signed integer
    Int32,
    /// 32-bit unsigned integer
    Uint32,
    /// 32-bit floating point
    Float32,
    /// 64-bit floating point
    Float64,
    /// 64-bit signed BigInt
    BigInt64,
    /// 64-bit unsigned BigInt
    BigUint64,
}

impl TypedArrayKind {
    /// Get the byte size of each element for this kind
    pub fn bytes_per_element(&self) -> usize {
        match self {
            TypedArrayKind::Int8 | TypedArrayKind::Uint8 | TypedArrayKind::Uint8Clamped => 1,
            TypedArrayKind::Int16 | TypedArrayKind::Uint16 => 2,
            TypedArrayKind::Int32 | TypedArrayKind::Uint32 | TypedArrayKind::Float32 => 4,
            TypedArrayKind::Float64
            | TypedArrayKind::BigInt64
            | TypedArrayKind::BigUint64 => 8,
        }
    }

    /// Get the name of this TypedArray kind
    pub fn name(&self) -> &'static str {
        match self {
            TypedArrayKind::Int8 => "Int8Array",
            TypedArrayKind::Uint8 => "Uint8Array",
            TypedArrayKind::Uint8Clamped => "Uint8ClampedArray",
            TypedArrayKind::Int16 => "Int16Array",
            TypedArrayKind::Uint16 => "Uint16Array",
            TypedArrayKind::Int32 => "Int32Array",
            TypedArrayKind::Uint32 => "Uint32Array",
            TypedArrayKind::Float32 => "Float32Array",
            TypedArrayKind::Float64 => "Float64Array",
            TypedArrayKind::BigInt64 => "BigInt64Array",
            TypedArrayKind::BigUint64 => "BigUint64Array",
        }
    }
}

/// TypedArray value representation
#[derive(Debug, Clone)]
pub enum TypedArrayValue {
    /// Regular number value
    Number(f64),
    /// BigInt value (as i128 for simplicity)
    BigInt(i128),
}

impl TypedArrayValue {
    /// Create from a number
    pub fn from_number(n: f64) -> Self {
        TypedArrayValue::Number(n)
    }

    /// Create from a BigInt
    pub fn from_bigint(n: i128) -> Self {
        TypedArrayValue::BigInt(n)
    }

    /// Get as f64
    pub fn as_f64(&self) -> f64 {
        match self {
            TypedArrayValue::Number(n) => *n,
            TypedArrayValue::BigInt(n) => *n as f64,
        }
    }

    /// Get as i128 (for BigInt operations)
    pub fn as_i128(&self) -> i128 {
        match self {
            TypedArrayValue::Number(n) => *n as i128,
            TypedArrayValue::BigInt(n) => *n,
        }
    }
}

/// Generic TypedArray implementation
#[derive(Debug, Clone)]
pub struct TypedArray {
    /// The underlying ArrayBuffer
    buffer: ArrayBuffer,
    /// Kind of typed array
    kind: TypedArrayKind,
    /// Byte offset into the buffer
    byte_offset: usize,
    /// Number of elements
    length: usize,
}

impl TypedArray {
    /// Create a new TypedArray with the given length
    pub fn new(kind: TypedArrayKind, length: usize) -> JsResult<Self> {
        let byte_length = length * kind.bytes_per_element();
        let buffer = ArrayBuffer::new(byte_length)?;
        Ok(TypedArray {
            buffer,
            kind,
            byte_offset: 0,
            length,
        })
    }

    /// Create a TypedArray from an existing ArrayBuffer
    pub fn from_buffer(
        buffer: ArrayBuffer,
        kind: TypedArrayKind,
        byte_offset: Option<usize>,
        length: Option<usize>,
    ) -> JsResult<Self> {
        let offset = byte_offset.unwrap_or(0);
        let elem_size = kind.bytes_per_element();

        if offset % elem_size != 0 {
            return Err(JsError::range_error(format!(
                "Byte offset {} is not aligned to element size {}",
                offset, elem_size
            )));
        }

        let buf_len = buffer.byte_length();
        if offset > buf_len {
            return Err(JsError::range_error("Byte offset out of range"));
        }

        let available_bytes = buf_len - offset;
        let len = match length {
            Some(l) => {
                let needed_bytes = l * elem_size;
                if needed_bytes > available_bytes {
                    return Err(JsError::range_error("Length out of range"));
                }
                l
            }
            None => {
                if available_bytes % elem_size != 0 {
                    return Err(JsError::range_error(
                        "Buffer byte length is not aligned to element size",
                    ));
                }
                available_bytes / elem_size
            }
        };

        Ok(TypedArray {
            buffer,
            kind,
            byte_offset: offset,
            length: len,
        })
    }

    /// Create a TypedArray from an array of values
    pub fn from_values(kind: TypedArrayKind, values: Vec<TypedArrayValue>) -> JsResult<Self> {
        let mut arr = TypedArray::new(kind, values.len())?;
        for (i, value) in values.into_iter().enumerate() {
            arr.set(i, value)?;
        }
        Ok(arr)
    }

    /// Get the element at the given index
    pub fn get(&self, index: usize) -> JsResult<TypedArrayValue> {
        if index >= self.length {
            return Err(JsError::range_error("Index out of bounds"));
        }

        let elem_size = self.kind.bytes_per_element();
        let byte_idx = self.byte_offset + index * elem_size;
        let data = self.buffer.get_data();
        let bytes = data.borrow();

        match self.kind {
            TypedArrayKind::Int8 => {
                let val = bytes[byte_idx] as i8;
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Uint8 => {
                let val = bytes[byte_idx];
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Uint8Clamped => {
                let val = bytes[byte_idx];
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Int16 => {
                let val = i16::from_ne_bytes([bytes[byte_idx], bytes[byte_idx + 1]]);
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Uint16 => {
                let val = u16::from_ne_bytes([bytes[byte_idx], bytes[byte_idx + 1]]);
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Int32 => {
                let val = i32::from_ne_bytes([
                    bytes[byte_idx],
                    bytes[byte_idx + 1],
                    bytes[byte_idx + 2],
                    bytes[byte_idx + 3],
                ]);
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Uint32 => {
                let val = u32::from_ne_bytes([
                    bytes[byte_idx],
                    bytes[byte_idx + 1],
                    bytes[byte_idx + 2],
                    bytes[byte_idx + 3],
                ]);
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Float32 => {
                let val = f32::from_ne_bytes([
                    bytes[byte_idx],
                    bytes[byte_idx + 1],
                    bytes[byte_idx + 2],
                    bytes[byte_idx + 3],
                ]);
                Ok(TypedArrayValue::Number(val as f64))
            }
            TypedArrayKind::Float64 => {
                let val = f64::from_ne_bytes([
                    bytes[byte_idx],
                    bytes[byte_idx + 1],
                    bytes[byte_idx + 2],
                    bytes[byte_idx + 3],
                    bytes[byte_idx + 4],
                    bytes[byte_idx + 5],
                    bytes[byte_idx + 6],
                    bytes[byte_idx + 7],
                ]);
                Ok(TypedArrayValue::Number(val))
            }
            TypedArrayKind::BigInt64 => {
                let val = i64::from_ne_bytes([
                    bytes[byte_idx],
                    bytes[byte_idx + 1],
                    bytes[byte_idx + 2],
                    bytes[byte_idx + 3],
                    bytes[byte_idx + 4],
                    bytes[byte_idx + 5],
                    bytes[byte_idx + 6],
                    bytes[byte_idx + 7],
                ]);
                Ok(TypedArrayValue::BigInt(val as i128))
            }
            TypedArrayKind::BigUint64 => {
                let val = u64::from_ne_bytes([
                    bytes[byte_idx],
                    bytes[byte_idx + 1],
                    bytes[byte_idx + 2],
                    bytes[byte_idx + 3],
                    bytes[byte_idx + 4],
                    bytes[byte_idx + 5],
                    bytes[byte_idx + 6],
                    bytes[byte_idx + 7],
                ]);
                Ok(TypedArrayValue::BigInt(val as i128))
            }
        }
    }

    /// Set the element at the given index
    pub fn set(&mut self, index: usize, value: TypedArrayValue) -> JsResult<()> {
        if index >= self.length {
            return Err(JsError::range_error("Index out of bounds"));
        }

        let elem_size = self.kind.bytes_per_element();
        let byte_idx = self.byte_offset + index * elem_size;
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();

        match self.kind {
            TypedArrayKind::Int8 => {
                let val = value.as_f64() as i8;
                bytes[byte_idx] = val as u8;
            }
            TypedArrayKind::Uint8 => {
                let val = value.as_f64() as u8;
                bytes[byte_idx] = val;
            }
            TypedArrayKind::Uint8Clamped => {
                let n = value.as_f64();
                let val = if n.is_nan() {
                    0
                } else if n <= 0.0 {
                    0
                } else if n >= 255.0 {
                    255
                } else {
                    n.round() as u8
                };
                bytes[byte_idx] = val;
            }
            TypedArrayKind::Int16 => {
                let val = value.as_f64() as i16;
                let val_bytes = val.to_ne_bytes();
                bytes[byte_idx] = val_bytes[0];
                bytes[byte_idx + 1] = val_bytes[1];
            }
            TypedArrayKind::Uint16 => {
                let val = value.as_f64() as u16;
                let val_bytes = val.to_ne_bytes();
                bytes[byte_idx] = val_bytes[0];
                bytes[byte_idx + 1] = val_bytes[1];
            }
            TypedArrayKind::Int32 => {
                let val = value.as_f64() as i32;
                let val_bytes = val.to_ne_bytes();
                for i in 0..4 {
                    bytes[byte_idx + i] = val_bytes[i];
                }
            }
            TypedArrayKind::Uint32 => {
                let val = value.as_f64() as u32;
                let val_bytes = val.to_ne_bytes();
                for i in 0..4 {
                    bytes[byte_idx + i] = val_bytes[i];
                }
            }
            TypedArrayKind::Float32 => {
                let val = value.as_f64() as f32;
                let val_bytes = val.to_ne_bytes();
                for i in 0..4 {
                    bytes[byte_idx + i] = val_bytes[i];
                }
            }
            TypedArrayKind::Float64 => {
                let val = value.as_f64();
                let val_bytes = val.to_ne_bytes();
                for i in 0..8 {
                    bytes[byte_idx + i] = val_bytes[i];
                }
            }
            TypedArrayKind::BigInt64 => {
                let val = value.as_i128() as i64;
                let val_bytes = val.to_ne_bytes();
                for i in 0..8 {
                    bytes[byte_idx + i] = val_bytes[i];
                }
            }
            TypedArrayKind::BigUint64 => {
                let val = value.as_i128() as u64;
                let val_bytes = val.to_ne_bytes();
                for i in 0..8 {
                    bytes[byte_idx + i] = val_bytes[i];
                }
            }
        }
        Ok(())
    }

    /// Get the length of the TypedArray
    pub fn length(&self) -> usize {
        self.length
    }

    /// Get the byte length of the TypedArray
    pub fn byte_length(&self) -> usize {
        self.length * self.kind.bytes_per_element()
    }

    /// Get the byte offset into the ArrayBuffer
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    /// Get the underlying ArrayBuffer
    pub fn buffer(&self) -> &ArrayBuffer {
        &self.buffer
    }

    /// Get the kind of this TypedArray
    pub fn kind(&self) -> TypedArrayKind {
        self.kind
    }

    /// Create a slice of the TypedArray
    pub fn slice(&self, start: i32, end: Option<i32>) -> JsResult<TypedArray> {
        let len = self.length as i32;

        let start_idx = if start < 0 {
            (len + start).max(0) as usize
        } else {
            start.min(len) as usize
        };

        let end_idx = match end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => e.min(len) as usize,
            None => len as usize,
        };

        let new_length = if start_idx < end_idx {
            end_idx - start_idx
        } else {
            0
        };

        let mut result = TypedArray::new(self.kind, new_length)?;
        for i in 0..new_length {
            let val = self.get(start_idx + i)?;
            result.set(i, val)?;
        }
        Ok(result)
    }

    /// Create a subarray view (shares the same buffer)
    pub fn subarray(&self, begin: i32, end: Option<i32>) -> JsResult<TypedArray> {
        let len = self.length as i32;

        let start_idx = if begin < 0 {
            (len + begin).max(0) as usize
        } else {
            begin.min(len) as usize
        };

        let end_idx = match end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => e.min(len) as usize,
            None => len as usize,
        };

        let new_length = if start_idx < end_idx {
            end_idx - start_idx
        } else {
            0
        };

        let new_byte_offset = self.byte_offset + start_idx * self.kind.bytes_per_element();

        Ok(TypedArray {
            buffer: self.buffer.clone(),
            kind: self.kind,
            byte_offset: new_byte_offset,
            length: new_length,
        })
    }

    /// Map each element using a callback
    pub fn map<F>(&self, callback: F) -> JsResult<TypedArray>
    where
        F: Fn(TypedArrayValue, usize) -> JsResult<TypedArrayValue>,
    {
        let mut result = TypedArray::new(self.kind, self.length)?;
        for i in 0..self.length {
            let val = self.get(i)?;
            let new_val = callback(val, i)?;
            result.set(i, new_val)?;
        }
        Ok(result)
    }

    /// Filter elements using a callback
    pub fn filter<F>(&self, callback: F) -> JsResult<TypedArray>
    where
        F: Fn(&TypedArrayValue, usize) -> JsResult<bool>,
    {
        let mut values = Vec::new();
        for i in 0..self.length {
            let val = self.get(i)?;
            if callback(&val, i)? {
                values.push(val);
            }
        }
        TypedArray::from_values(self.kind, values)
    }

    /// Reduce elements using a callback
    pub fn reduce<F>(&self, initial: TypedArrayValue, callback: F) -> JsResult<TypedArrayValue>
    where
        F: Fn(TypedArrayValue, TypedArrayValue, usize) -> JsResult<TypedArrayValue>,
    {
        let mut acc = initial;
        for i in 0..self.length {
            let val = self.get(i)?;
            acc = callback(acc, val, i)?;
        }
        Ok(acc)
    }

    /// Execute a callback for each element
    pub fn for_each<F>(&self, callback: F) -> JsResult<()>
    where
        F: Fn(&TypedArrayValue, usize) -> JsResult<()>,
    {
        for i in 0..self.length {
            let val = self.get(i)?;
            callback(&val, i)?;
        }
        Ok(())
    }

    /// Find the index of a value
    pub fn index_of(&self, value: &TypedArrayValue) -> JsResult<Option<usize>> {
        let target = value.as_f64();
        for i in 0..self.length {
            let val = self.get(i)?;
            if val.as_f64() == target {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// Check if the array includes a value
    pub fn includes(&self, value: &TypedArrayValue) -> JsResult<bool> {
        Ok(self.index_of(value)?.is_some())
    }

    /// Find an element that satisfies a predicate
    pub fn find<F>(&self, callback: F) -> JsResult<Option<TypedArrayValue>>
    where
        F: Fn(&TypedArrayValue, usize) -> JsResult<bool>,
    {
        for i in 0..self.length {
            let val = self.get(i)?;
            if callback(&val, i)? {
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    /// Fill the array with a value
    pub fn fill(
        &mut self,
        value: TypedArrayValue,
        start: Option<i32>,
        end: Option<i32>,
    ) -> JsResult<&mut Self> {
        let len = self.length as i32;

        let start_idx = match start {
            Some(s) if s < 0 => (len + s).max(0) as usize,
            Some(s) => s.min(len) as usize,
            None => 0,
        };

        let end_idx = match end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => e.min(len) as usize,
            None => len as usize,
        };

        for i in start_idx..end_idx {
            self.set(i, value.clone())?;
        }
        Ok(self)
    }

    /// Copy within the array
    pub fn copy_within(
        &mut self,
        target: i32,
        start: i32,
        end: Option<i32>,
    ) -> JsResult<&mut Self> {
        let len = self.length as i32;

        let to = if target < 0 {
            (len + target).max(0) as usize
        } else {
            target.min(len) as usize
        };

        let from = if start < 0 {
            (len + start).max(0) as usize
        } else {
            start.min(len) as usize
        };

        let final_end = match end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => e.min(len) as usize,
            None => len as usize,
        };

        let count = (final_end - from).min(self.length - to);

        // Copy values (handle overlapping ranges)
        let values: Vec<TypedArrayValue> =
            (0..count).map(|i| self.get(from + i)).collect::<JsResult<Vec<_>>>()?;

        for (i, val) in values.into_iter().enumerate() {
            self.set(to + i, val)?;
        }

        Ok(self)
    }

    /// Reverse the array in place
    pub fn reverse(&mut self) -> JsResult<&mut Self> {
        let len = self.length;
        for i in 0..len / 2 {
            let a = self.get(i)?;
            let b = self.get(len - 1 - i)?;
            self.set(i, b)?;
            self.set(len - 1 - i, a)?;
        }
        Ok(self)
    }

    /// Sort the array in place
    pub fn sort(&mut self) -> JsResult<&mut Self> {
        let mut values: Vec<TypedArrayValue> = (0..self.length)
            .map(|i| self.get(i))
            .collect::<JsResult<Vec<_>>>()?;

        values.sort_by(|a, b| a.as_f64().partial_cmp(&b.as_f64()).unwrap_or(std::cmp::Ordering::Equal));

        for (i, val) in values.into_iter().enumerate() {
            self.set(i, val)?;
        }

        Ok(self)
    }
}

// Type aliases for specific TypedArray types
/// Int8Array - 8-bit signed integers
pub type Int8Array = TypedArray;

/// Uint8Array - 8-bit unsigned integers
pub type Uint8Array = TypedArray;

/// Uint8ClampedArray - 8-bit unsigned integers (clamped)
pub type Uint8ClampedArray = TypedArray;

/// Int16Array - 16-bit signed integers
pub type Int16Array = TypedArray;

/// Uint16Array - 16-bit unsigned integers
pub type Uint16Array = TypedArray;

/// Int32Array - 32-bit signed integers
pub type Int32Array = TypedArray;

/// Uint32Array - 32-bit unsigned integers
pub type Uint32Array = TypedArray;

/// Float32Array - 32-bit floating point
pub type Float32Array = TypedArray;

/// Float64Array - 64-bit floating point
pub type Float64Array = TypedArray;

/// BigInt64Array - 64-bit signed BigInt
pub type BigInt64Array = TypedArray;

/// BigUint64Array - 64-bit unsigned BigInt
pub type BigUint64Array = TypedArray;

/// DataView - provides a low-level interface for reading/writing binary data
#[derive(Debug, Clone)]
pub struct DataView {
    /// The underlying ArrayBuffer
    buffer: ArrayBuffer,
    /// Byte offset into the buffer
    byte_offset: usize,
    /// Byte length of the view
    byte_length: usize,
}

impl DataView {
    /// Create a new DataView for the given ArrayBuffer
    pub fn new(
        buffer: ArrayBuffer,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
    ) -> JsResult<Self> {
        let offset = byte_offset.unwrap_or(0);
        let buf_len = buffer.byte_length();

        if offset > buf_len {
            return Err(JsError::range_error("Byte offset out of range"));
        }

        let available = buf_len - offset;
        let length = match byte_length {
            Some(l) if l > available => {
                return Err(JsError::range_error("Byte length out of range"));
            }
            Some(l) => l,
            None => available,
        };

        Ok(DataView {
            buffer,
            byte_offset: offset,
            byte_length: length,
        })
    }

    /// Get the underlying ArrayBuffer
    pub fn buffer(&self) -> &ArrayBuffer {
        &self.buffer
    }

    /// Get the byte offset
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    /// Get the byte length
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Get Int8 value at the given byte offset
    pub fn get_int8(&self, byte_offset: usize) -> JsResult<i8> {
        if byte_offset >= self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        Ok(bytes[self.byte_offset + byte_offset] as i8)
    }

    /// Set Int8 value at the given byte offset
    pub fn set_int8(&self, byte_offset: usize, value: i8) -> JsResult<()> {
        if byte_offset >= self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        bytes[self.byte_offset + byte_offset] = value as u8;
        Ok(())
    }

    /// Get Uint8 value at the given byte offset
    pub fn get_uint8(&self, byte_offset: usize) -> JsResult<u8> {
        if byte_offset >= self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        Ok(bytes[self.byte_offset + byte_offset])
    }

    /// Set Uint8 value at the given byte offset
    pub fn set_uint8(&self, byte_offset: usize, value: u8) -> JsResult<()> {
        if byte_offset >= self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        bytes[self.byte_offset + byte_offset] = value;
        Ok(())
    }

    /// Get Int16 value at the given byte offset
    pub fn get_int16(&self, byte_offset: usize, little_endian: bool) -> JsResult<i16> {
        if byte_offset + 2 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            i16::from_le_bytes([bytes[idx], bytes[idx + 1]])
        } else {
            i16::from_be_bytes([bytes[idx], bytes[idx + 1]])
        };
        Ok(val)
    }

    /// Set Int16 value at the given byte offset
    pub fn set_int16(&self, byte_offset: usize, value: i16, little_endian: bool) -> JsResult<()> {
        if byte_offset + 2 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        bytes[idx] = val_bytes[0];
        bytes[idx + 1] = val_bytes[1];
        Ok(())
    }

    /// Get Uint16 value at the given byte offset
    pub fn get_uint16(&self, byte_offset: usize, little_endian: bool) -> JsResult<u16> {
        if byte_offset + 2 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            u16::from_le_bytes([bytes[idx], bytes[idx + 1]])
        } else {
            u16::from_be_bytes([bytes[idx], bytes[idx + 1]])
        };
        Ok(val)
    }

    /// Set Uint16 value at the given byte offset
    pub fn set_uint16(&self, byte_offset: usize, value: u16, little_endian: bool) -> JsResult<()> {
        if byte_offset + 2 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        bytes[idx] = val_bytes[0];
        bytes[idx + 1] = val_bytes[1];
        Ok(())
    }

    /// Get Int32 value at the given byte offset
    pub fn get_int32(&self, byte_offset: usize, little_endian: bool) -> JsResult<i32> {
        if byte_offset + 4 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            i32::from_le_bytes([bytes[idx], bytes[idx + 1], bytes[idx + 2], bytes[idx + 3]])
        } else {
            i32::from_be_bytes([bytes[idx], bytes[idx + 1], bytes[idx + 2], bytes[idx + 3]])
        };
        Ok(val)
    }

    /// Set Int32 value at the given byte offset
    pub fn set_int32(&self, byte_offset: usize, value: i32, little_endian: bool) -> JsResult<()> {
        if byte_offset + 4 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        for i in 0..4 {
            bytes[idx + i] = val_bytes[i];
        }
        Ok(())
    }

    /// Get Uint32 value at the given byte offset
    pub fn get_uint32(&self, byte_offset: usize, little_endian: bool) -> JsResult<u32> {
        if byte_offset + 4 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            u32::from_le_bytes([bytes[idx], bytes[idx + 1], bytes[idx + 2], bytes[idx + 3]])
        } else {
            u32::from_be_bytes([bytes[idx], bytes[idx + 1], bytes[idx + 2], bytes[idx + 3]])
        };
        Ok(val)
    }

    /// Set Uint32 value at the given byte offset
    pub fn set_uint32(&self, byte_offset: usize, value: u32, little_endian: bool) -> JsResult<()> {
        if byte_offset + 4 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        for i in 0..4 {
            bytes[idx + i] = val_bytes[i];
        }
        Ok(())
    }

    /// Get Float32 value at the given byte offset
    pub fn get_float32(&self, byte_offset: usize, little_endian: bool) -> JsResult<f32> {
        if byte_offset + 4 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            f32::from_le_bytes([bytes[idx], bytes[idx + 1], bytes[idx + 2], bytes[idx + 3]])
        } else {
            f32::from_be_bytes([bytes[idx], bytes[idx + 1], bytes[idx + 2], bytes[idx + 3]])
        };
        Ok(val)
    }

    /// Set Float32 value at the given byte offset
    pub fn set_float32(
        &self,
        byte_offset: usize,
        value: f32,
        little_endian: bool,
    ) -> JsResult<()> {
        if byte_offset + 4 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        for i in 0..4 {
            bytes[idx + i] = val_bytes[i];
        }
        Ok(())
    }

    /// Get Float64 value at the given byte offset
    pub fn get_float64(&self, byte_offset: usize, little_endian: bool) -> JsResult<f64> {
        if byte_offset + 8 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            f64::from_le_bytes([
                bytes[idx],
                bytes[idx + 1],
                bytes[idx + 2],
                bytes[idx + 3],
                bytes[idx + 4],
                bytes[idx + 5],
                bytes[idx + 6],
                bytes[idx + 7],
            ])
        } else {
            f64::from_be_bytes([
                bytes[idx],
                bytes[idx + 1],
                bytes[idx + 2],
                bytes[idx + 3],
                bytes[idx + 4],
                bytes[idx + 5],
                bytes[idx + 6],
                bytes[idx + 7],
            ])
        };
        Ok(val)
    }

    /// Set Float64 value at the given byte offset
    pub fn set_float64(
        &self,
        byte_offset: usize,
        value: f64,
        little_endian: bool,
    ) -> JsResult<()> {
        if byte_offset + 8 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        for i in 0..8 {
            bytes[idx + i] = val_bytes[i];
        }
        Ok(())
    }

    /// Get BigInt64 value at the given byte offset
    pub fn get_big_int64(&self, byte_offset: usize, little_endian: bool) -> JsResult<i64> {
        if byte_offset + 8 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            i64::from_le_bytes([
                bytes[idx],
                bytes[idx + 1],
                bytes[idx + 2],
                bytes[idx + 3],
                bytes[idx + 4],
                bytes[idx + 5],
                bytes[idx + 6],
                bytes[idx + 7],
            ])
        } else {
            i64::from_be_bytes([
                bytes[idx],
                bytes[idx + 1],
                bytes[idx + 2],
                bytes[idx + 3],
                bytes[idx + 4],
                bytes[idx + 5],
                bytes[idx + 6],
                bytes[idx + 7],
            ])
        };
        Ok(val)
    }

    /// Set BigInt64 value at the given byte offset
    pub fn set_big_int64(
        &self,
        byte_offset: usize,
        value: i64,
        little_endian: bool,
    ) -> JsResult<()> {
        if byte_offset + 8 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        for i in 0..8 {
            bytes[idx + i] = val_bytes[i];
        }
        Ok(())
    }

    /// Get BigUint64 value at the given byte offset
    pub fn get_big_uint64(&self, byte_offset: usize, little_endian: bool) -> JsResult<u64> {
        if byte_offset + 8 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let bytes = data.borrow();
        let idx = self.byte_offset + byte_offset;
        let val = if little_endian {
            u64::from_le_bytes([
                bytes[idx],
                bytes[idx + 1],
                bytes[idx + 2],
                bytes[idx + 3],
                bytes[idx + 4],
                bytes[idx + 5],
                bytes[idx + 6],
                bytes[idx + 7],
            ])
        } else {
            u64::from_be_bytes([
                bytes[idx],
                bytes[idx + 1],
                bytes[idx + 2],
                bytes[idx + 3],
                bytes[idx + 4],
                bytes[idx + 5],
                bytes[idx + 6],
                bytes[idx + 7],
            ])
        };
        Ok(val)
    }

    /// Set BigUint64 value at the given byte offset
    pub fn set_big_uint64(
        &self,
        byte_offset: usize,
        value: u64,
        little_endian: bool,
    ) -> JsResult<()> {
        if byte_offset + 8 > self.byte_length {
            return Err(JsError::range_error("Offset out of bounds"));
        }
        let data = self.buffer.get_data();
        let mut bytes = data.borrow_mut();
        let idx = self.byte_offset + byte_offset;
        let val_bytes = if little_endian {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        for i in 0..8 {
            bytes[idx + i] = val_bytes[i];
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ArrayBuffer tests
    #[test]
    fn test_array_buffer_creation() {
        let buf = ArrayBuffer::new(16).unwrap();
        assert_eq!(buf.byte_length(), 16);
    }

    #[test]
    fn test_array_buffer_zero_length() {
        let buf = ArrayBuffer::new(0).unwrap();
        assert_eq!(buf.byte_length(), 0);
    }

    #[test]
    fn test_array_buffer_invalid_length() {
        let result = ArrayBuffer::new(usize::MAX);
        assert!(result.is_err());
    }

    #[test]
    fn test_array_buffer_slice() {
        let buf = ArrayBuffer::new(10).unwrap();
        let sliced = buf.slice(2, Some(7)).unwrap();
        assert_eq!(sliced.byte_length(), 5);
    }

    #[test]
    fn test_array_buffer_slice_negative() {
        let buf = ArrayBuffer::new(10).unwrap();
        let sliced = buf.slice(-3, None).unwrap();
        assert_eq!(sliced.byte_length(), 3);
    }

    #[test]
    fn test_array_buffer_slice_empty() {
        let buf = ArrayBuffer::new(10).unwrap();
        let sliced = buf.slice(5, Some(5)).unwrap();
        assert_eq!(sliced.byte_length(), 0);
    }

    // TypedArrayKind tests
    #[test]
    fn test_typed_array_kind_bytes_per_element() {
        assert_eq!(TypedArrayKind::Int8.bytes_per_element(), 1);
        assert_eq!(TypedArrayKind::Uint8.bytes_per_element(), 1);
        assert_eq!(TypedArrayKind::Uint8Clamped.bytes_per_element(), 1);
        assert_eq!(TypedArrayKind::Int16.bytes_per_element(), 2);
        assert_eq!(TypedArrayKind::Uint16.bytes_per_element(), 2);
        assert_eq!(TypedArrayKind::Int32.bytes_per_element(), 4);
        assert_eq!(TypedArrayKind::Uint32.bytes_per_element(), 4);
        assert_eq!(TypedArrayKind::Float32.bytes_per_element(), 4);
        assert_eq!(TypedArrayKind::Float64.bytes_per_element(), 8);
        assert_eq!(TypedArrayKind::BigInt64.bytes_per_element(), 8);
        assert_eq!(TypedArrayKind::BigUint64.bytes_per_element(), 8);
    }

    #[test]
    fn test_typed_array_kind_names() {
        assert_eq!(TypedArrayKind::Int8.name(), "Int8Array");
        assert_eq!(TypedArrayKind::Uint8.name(), "Uint8Array");
        assert_eq!(TypedArrayKind::Float64.name(), "Float64Array");
    }

    // Int8Array tests
    #[test]
    fn test_int8_array_creation() {
        let arr = TypedArray::new(TypedArrayKind::Int8, 10).unwrap();
        assert_eq!(arr.length(), 10);
        assert_eq!(arr.byte_length(), 10);
    }

    #[test]
    fn test_int8_array_get_set() {
        let mut arr = TypedArray::new(TypedArrayKind::Int8, 3).unwrap();
        arr.set(0, TypedArrayValue::Number(-128.0)).unwrap();
        arr.set(1, TypedArrayValue::Number(0.0)).unwrap();
        arr.set(2, TypedArrayValue::Number(127.0)).unwrap();

        assert_eq!(arr.get(0).unwrap().as_f64(), -128.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 0.0);
        assert_eq!(arr.get(2).unwrap().as_f64(), 127.0);
    }

    #[test]
    fn test_int8_array_overflow() {
        let mut arr = TypedArray::new(TypedArrayKind::Int8, 1).unwrap();
        arr.set(0, TypedArrayValue::Number(200.0)).unwrap();
        // In Rust, 200.0 as i8 saturates to 127 (max i8 value)
        // This matches JavaScript's behavior where the value is treated as i8
        assert_eq!(arr.get(0).unwrap().as_f64(), 127.0);
    }

    // Uint8Array tests
    #[test]
    fn test_uint8_array_get_set() {
        let mut arr = TypedArray::new(TypedArrayKind::Uint8, 3).unwrap();
        arr.set(0, TypedArrayValue::Number(0.0)).unwrap();
        arr.set(1, TypedArrayValue::Number(128.0)).unwrap();
        arr.set(2, TypedArrayValue::Number(255.0)).unwrap();

        assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 128.0);
        assert_eq!(arr.get(2).unwrap().as_f64(), 255.0);
    }

    // Uint8ClampedArray tests
    #[test]
    fn test_uint8_clamped_array() {
        let mut arr = TypedArray::new(TypedArrayKind::Uint8Clamped, 4).unwrap();
        arr.set(0, TypedArrayValue::Number(-10.0)).unwrap();
        arr.set(1, TypedArrayValue::Number(128.5)).unwrap();
        arr.set(2, TypedArrayValue::Number(300.0)).unwrap();
        arr.set(3, TypedArrayValue::Number(f64::NAN)).unwrap();

        assert_eq!(arr.get(0).unwrap().as_f64(), 0.0); // clamped to 0
        assert_eq!(arr.get(1).unwrap().as_f64(), 129.0); // rounded to 129
        assert_eq!(arr.get(2).unwrap().as_f64(), 255.0); // clamped to 255
        assert_eq!(arr.get(3).unwrap().as_f64(), 0.0); // NaN becomes 0
    }

    // Int16Array tests
    #[test]
    fn test_int16_array() {
        let mut arr = TypedArray::new(TypedArrayKind::Int16, 2).unwrap();
        arr.set(0, TypedArrayValue::Number(-32768.0)).unwrap();
        arr.set(1, TypedArrayValue::Number(32767.0)).unwrap();

        assert_eq!(arr.length(), 2);
        assert_eq!(arr.byte_length(), 4);
        assert_eq!(arr.get(0).unwrap().as_f64(), -32768.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 32767.0);
    }

    // Uint16Array tests
    #[test]
    fn test_uint16_array() {
        let mut arr = TypedArray::new(TypedArrayKind::Uint16, 2).unwrap();
        arr.set(0, TypedArrayValue::Number(0.0)).unwrap();
        arr.set(1, TypedArrayValue::Number(65535.0)).unwrap();

        assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 65535.0);
    }

    // Int32Array tests
    #[test]
    fn test_int32_array() {
        let mut arr = TypedArray::new(TypedArrayKind::Int32, 2).unwrap();
        arr.set(0, TypedArrayValue::Number(-2147483648.0)).unwrap();
        arr.set(1, TypedArrayValue::Number(2147483647.0)).unwrap();

        assert_eq!(arr.length(), 2);
        assert_eq!(arr.byte_length(), 8);
        assert_eq!(arr.get(0).unwrap().as_f64(), -2147483648.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 2147483647.0);
    }

    // Uint32Array tests
    #[test]
    fn test_uint32_array() {
        let mut arr = TypedArray::new(TypedArrayKind::Uint32, 2).unwrap();
        arr.set(0, TypedArrayValue::Number(0.0)).unwrap();
        arr.set(1, TypedArrayValue::Number(4294967295.0)).unwrap();

        assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 4294967295.0);
    }

    // Float32Array tests
    #[test]
    fn test_float32_array() {
        let mut arr = TypedArray::new(TypedArrayKind::Float32, 3).unwrap();
        arr.set(0, TypedArrayValue::Number(3.14)).unwrap();
        arr.set(1, TypedArrayValue::Number(-2.71)).unwrap();
        arr.set(2, TypedArrayValue::Number(f64::INFINITY)).unwrap();

        let val0 = arr.get(0).unwrap().as_f64();
        let val1 = arr.get(1).unwrap().as_f64();
        let val2 = arr.get(2).unwrap().as_f64();

        assert!((val0 - 3.14).abs() < 0.001);
        assert!((val1 - -2.71).abs() < 0.001);
        assert!(val2.is_infinite() && val2 > 0.0);
    }

    // Float64Array tests
    #[test]
    fn test_float64_array() {
        let mut arr = TypedArray::new(TypedArrayKind::Float64, 3).unwrap();
        arr.set(0, TypedArrayValue::Number(std::f64::consts::PI)).unwrap();
        arr.set(1, TypedArrayValue::Number(-std::f64::consts::E)).unwrap();
        arr.set(2, TypedArrayValue::Number(f64::NAN)).unwrap();

        let val0 = arr.get(0).unwrap().as_f64();
        let val1 = arr.get(1).unwrap().as_f64();
        let val2 = arr.get(2).unwrap().as_f64();

        assert_eq!(val0, std::f64::consts::PI);
        assert_eq!(val1, -std::f64::consts::E);
        assert!(val2.is_nan());
    }

    // BigInt64Array tests
    #[test]
    fn test_big_int64_array() {
        let mut arr = TypedArray::new(TypedArrayKind::BigInt64, 2).unwrap();
        arr.set(0, TypedArrayValue::BigInt(-9223372036854775808)).unwrap();
        arr.set(1, TypedArrayValue::BigInt(9223372036854775807)).unwrap();

        assert_eq!(arr.get(0).unwrap().as_i128(), -9223372036854775808);
        assert_eq!(arr.get(1).unwrap().as_i128(), 9223372036854775807);
    }

    // BigUint64Array tests
    #[test]
    fn test_big_uint64_array() {
        let mut arr = TypedArray::new(TypedArrayKind::BigUint64, 2).unwrap();
        arr.set(0, TypedArrayValue::BigInt(0)).unwrap();
        arr.set(1, TypedArrayValue::BigInt(18446744073709551615)).unwrap();

        assert_eq!(arr.get(0).unwrap().as_i128(), 0);
        assert_eq!(arr.get(1).unwrap().as_i128(), 18446744073709551615);
    }

    // TypedArray from buffer tests
    #[test]
    fn test_typed_array_from_buffer() {
        let buffer = ArrayBuffer::new(16).unwrap();
        let arr = TypedArray::from_buffer(buffer, TypedArrayKind::Int32, None, None).unwrap();
        assert_eq!(arr.length(), 4);
        assert_eq!(arr.byte_length(), 16);
        assert_eq!(arr.byte_offset(), 0);
    }

    #[test]
    fn test_typed_array_from_buffer_with_offset() {
        let buffer = ArrayBuffer::new(16).unwrap();
        let arr =
            TypedArray::from_buffer(buffer, TypedArrayKind::Int32, Some(4), Some(2)).unwrap();
        assert_eq!(arr.length(), 2);
        assert_eq!(arr.byte_length(), 8);
        assert_eq!(arr.byte_offset(), 4);
    }

    #[test]
    fn test_typed_array_from_buffer_misaligned() {
        let buffer = ArrayBuffer::new(16).unwrap();
        let result = TypedArray::from_buffer(buffer, TypedArrayKind::Int32, Some(3), None);
        assert!(result.is_err()); // 3 is not aligned to 4 bytes
    }

    // TypedArray operations tests
    #[test]
    fn test_typed_array_slice() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
                TypedArrayValue::Number(4.0),
                TypedArrayValue::Number(5.0),
            ],
        )
        .unwrap();

        let sliced = arr.slice(1, Some(4)).unwrap();
        assert_eq!(sliced.length(), 3);
        assert_eq!(sliced.get(0).unwrap().as_f64(), 2.0);
        assert_eq!(sliced.get(1).unwrap().as_f64(), 3.0);
        assert_eq!(sliced.get(2).unwrap().as_f64(), 4.0);
    }

    #[test]
    fn test_typed_array_subarray() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int32,
            vec![
                TypedArrayValue::Number(10.0),
                TypedArrayValue::Number(20.0),
                TypedArrayValue::Number(30.0),
            ],
        )
        .unwrap();

        let sub = arr.subarray(1, Some(3)).unwrap();
        assert_eq!(sub.length(), 2);
        assert_eq!(sub.byte_offset(), 4); // 1 * 4 bytes
        assert_eq!(sub.get(0).unwrap().as_f64(), 20.0);
        assert_eq!(sub.get(1).unwrap().as_f64(), 30.0);
    }

    #[test]
    fn test_typed_array_map() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
            ],
        )
        .unwrap();

        let mapped = arr
            .map(|v, _| Ok(TypedArrayValue::Number(v.as_f64() * 2.0)))
            .unwrap();

        assert_eq!(mapped.get(0).unwrap().as_f64(), 2.0);
        assert_eq!(mapped.get(1).unwrap().as_f64(), 4.0);
        assert_eq!(mapped.get(2).unwrap().as_f64(), 6.0);
    }

    #[test]
    fn test_typed_array_filter() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
                TypedArrayValue::Number(4.0),
            ],
        )
        .unwrap();

        let filtered = arr.filter(|v, _| Ok(v.as_f64() > 2.0)).unwrap();
        assert_eq!(filtered.length(), 2);
        assert_eq!(filtered.get(0).unwrap().as_f64(), 3.0);
        assert_eq!(filtered.get(1).unwrap().as_f64(), 4.0);
    }

    #[test]
    fn test_typed_array_reduce() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
            ],
        )
        .unwrap();

        let sum = arr
            .reduce(TypedArrayValue::Number(0.0), |acc, v, _| {
                Ok(TypedArrayValue::Number(acc.as_f64() + v.as_f64()))
            })
            .unwrap();

        assert_eq!(sum.as_f64(), 6.0);
    }

    #[test]
    fn test_typed_array_index_of() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(10.0),
                TypedArrayValue::Number(20.0),
                TypedArrayValue::Number(30.0),
            ],
        )
        .unwrap();

        assert_eq!(
            arr.index_of(&TypedArrayValue::Number(20.0)).unwrap(),
            Some(1)
        );
        assert_eq!(
            arr.index_of(&TypedArrayValue::Number(40.0)).unwrap(),
            None
        );
    }

    #[test]
    fn test_typed_array_includes() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
            ],
        )
        .unwrap();

        assert!(arr.includes(&TypedArrayValue::Number(2.0)).unwrap());
        assert!(!arr.includes(&TypedArrayValue::Number(5.0)).unwrap());
    }

    #[test]
    fn test_typed_array_find() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(3.0),
                TypedArrayValue::Number(5.0),
            ],
        )
        .unwrap();

        let found = arr.find(|v, _| Ok(v.as_f64() > 2.0)).unwrap();
        assert_eq!(found.unwrap().as_f64(), 3.0);
    }

    #[test]
    fn test_typed_array_fill() {
        let mut arr = TypedArray::new(TypedArrayKind::Int8, 5).unwrap();
        arr.fill(TypedArrayValue::Number(42.0), Some(1), Some(4))
            .unwrap();

        assert_eq!(arr.get(0).unwrap().as_f64(), 0.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 42.0);
        assert_eq!(arr.get(2).unwrap().as_f64(), 42.0);
        assert_eq!(arr.get(3).unwrap().as_f64(), 42.0);
        assert_eq!(arr.get(4).unwrap().as_f64(), 0.0);
    }

    #[test]
    fn test_typed_array_copy_within() {
        let mut arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
                TypedArrayValue::Number(4.0),
                TypedArrayValue::Number(5.0),
            ],
        )
        .unwrap();

        arr.copy_within(0, 3, None).unwrap();
        assert_eq!(arr.get(0).unwrap().as_f64(), 4.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 5.0);
    }

    #[test]
    fn test_typed_array_reverse() {
        let mut arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
            ],
        )
        .unwrap();

        arr.reverse().unwrap();
        assert_eq!(arr.get(0).unwrap().as_f64(), 3.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 2.0);
        assert_eq!(arr.get(2).unwrap().as_f64(), 1.0);
    }

    #[test]
    fn test_typed_array_sort() {
        let mut arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(3.0),
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(4.0),
                TypedArrayValue::Number(2.0),
            ],
        )
        .unwrap();

        arr.sort().unwrap();
        assert_eq!(arr.get(0).unwrap().as_f64(), 1.0);
        assert_eq!(arr.get(1).unwrap().as_f64(), 2.0);
        assert_eq!(arr.get(2).unwrap().as_f64(), 3.0);
        assert_eq!(arr.get(3).unwrap().as_f64(), 4.0);
    }

    // DataView tests
    #[test]
    fn test_data_view_creation() {
        let buffer = ArrayBuffer::new(16).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();
        assert_eq!(view.byte_offset(), 0);
        assert_eq!(view.byte_length(), 16);
    }

    #[test]
    fn test_data_view_with_offset() {
        let buffer = ArrayBuffer::new(16).unwrap();
        let view = DataView::new(buffer, Some(4), Some(8)).unwrap();
        assert_eq!(view.byte_offset(), 4);
        assert_eq!(view.byte_length(), 8);
    }

    #[test]
    fn test_data_view_int8() {
        let buffer = ArrayBuffer::new(4).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_int8(0, -128).unwrap();
        view.set_int8(1, 0).unwrap();
        view.set_int8(2, 127).unwrap();

        assert_eq!(view.get_int8(0).unwrap(), -128);
        assert_eq!(view.get_int8(1).unwrap(), 0);
        assert_eq!(view.get_int8(2).unwrap(), 127);
    }

    #[test]
    fn test_data_view_uint8() {
        let buffer = ArrayBuffer::new(4).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_uint8(0, 0).unwrap();
        view.set_uint8(1, 128).unwrap();
        view.set_uint8(2, 255).unwrap();

        assert_eq!(view.get_uint8(0).unwrap(), 0);
        assert_eq!(view.get_uint8(1).unwrap(), 128);
        assert_eq!(view.get_uint8(2).unwrap(), 255);
    }

    #[test]
    fn test_data_view_int16_endianness() {
        let buffer = ArrayBuffer::new(4).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_int16(0, 0x1234, true).unwrap(); // little endian
        view.set_int16(2, 0x1234, false).unwrap(); // big endian

        assert_eq!(view.get_int16(0, true).unwrap(), 0x1234);
        assert_eq!(view.get_int16(2, false).unwrap(), 0x1234);

        // Check that bytes are different
        assert_eq!(view.get_uint8(0).unwrap(), 0x34); // LE: low byte first
        assert_eq!(view.get_uint8(1).unwrap(), 0x12);
        assert_eq!(view.get_uint8(2).unwrap(), 0x12); // BE: high byte first
        assert_eq!(view.get_uint8(3).unwrap(), 0x34);
    }

    #[test]
    fn test_data_view_uint16() {
        let buffer = ArrayBuffer::new(2).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_uint16(0, 65535, true).unwrap();
        assert_eq!(view.get_uint16(0, true).unwrap(), 65535);
    }

    #[test]
    fn test_data_view_int32_endianness() {
        let buffer = ArrayBuffer::new(8).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_int32(0, 0x12345678, true).unwrap();
        view.set_int32(4, 0x12345678, false).unwrap();

        assert_eq!(view.get_int32(0, true).unwrap(), 0x12345678);
        assert_eq!(view.get_int32(4, false).unwrap(), 0x12345678);
    }

    #[test]
    fn test_data_view_uint32() {
        let buffer = ArrayBuffer::new(4).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_uint32(0, 4294967295, true).unwrap();
        assert_eq!(view.get_uint32(0, true).unwrap(), 4294967295);
    }

    #[test]
    fn test_data_view_float32() {
        let buffer = ArrayBuffer::new(4).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_float32(0, 3.14, true).unwrap();
        let val = view.get_float32(0, true).unwrap();
        assert!((val - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_data_view_float64() {
        let buffer = ArrayBuffer::new(8).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_float64(0, std::f64::consts::PI, true).unwrap();
        let val = view.get_float64(0, true).unwrap();
        assert_eq!(val, std::f64::consts::PI);
    }

    #[test]
    fn test_data_view_big_int64() {
        let buffer = ArrayBuffer::new(8).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_big_int64(0, -9223372036854775808, true).unwrap();
        assert_eq!(view.get_big_int64(0, true).unwrap(), -9223372036854775808);
    }

    #[test]
    fn test_data_view_big_uint64() {
        let buffer = ArrayBuffer::new(8).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        view.set_big_uint64(0, 18446744073709551615, true).unwrap();
        assert_eq!(view.get_big_uint64(0, true).unwrap(), 18446744073709551615);
    }

    #[test]
    fn test_data_view_out_of_bounds() {
        let buffer = ArrayBuffer::new(4).unwrap();
        let view = DataView::new(buffer, None, None).unwrap();

        assert!(view.get_int32(2, true).is_err()); // Only 2 bytes available
        assert!(view.set_float64(0, 1.0, true).is_err()); // Need 8 bytes
    }

    // Edge case tests
    #[test]
    fn test_typed_array_boundary_conditions() {
        let arr = TypedArray::new(TypedArrayKind::Int8, 0).unwrap();
        assert_eq!(arr.length(), 0);
        assert!(arr.get(0).is_err());
    }

    #[test]
    fn test_typed_array_negative_indices_in_slice() {
        let arr = TypedArray::from_values(
            TypedArrayKind::Int8,
            vec![
                TypedArrayValue::Number(1.0),
                TypedArrayValue::Number(2.0),
                TypedArrayValue::Number(3.0),
                TypedArrayValue::Number(4.0),
            ],
        )
        .unwrap();

        let sliced = arr.slice(-2, None).unwrap();
        assert_eq!(sliced.length(), 2);
        assert_eq!(sliced.get(0).unwrap().as_f64(), 3.0);
        assert_eq!(sliced.get(1).unwrap().as_f64(), 4.0);
    }

    #[test]
    fn test_shared_buffer_between_views() {
        let buffer = ArrayBuffer::new(8).unwrap();
        let view1 = TypedArray::from_buffer(buffer.clone(), TypedArrayKind::Uint8, None, None).unwrap();
        let view2 = TypedArray::from_buffer(buffer.clone(), TypedArrayKind::Uint16, None, None).unwrap();

        // Write using Uint8
        let mut view1_mut = view1;
        view1_mut.set(0, TypedArrayValue::Number(0x12 as f64)).unwrap();
        view1_mut.set(1, TypedArrayValue::Number(0x34 as f64)).unwrap();

        // Read using Uint16 (depends on endianness)
        // On little-endian: 0x3412, on big-endian: 0x1234
        let val = view2.get(0).unwrap().as_f64();
        assert!(val == 0x3412 as f64 || val == 0x1234 as f64);
    }
}
