//! JavaScript value representation
//!
//! Placeholder for core_types::Value dependency.
//! Will be replaced when core_types is integrated.

/// JavaScript runtime value
///
/// This is a placeholder that will be replaced by core_types::Value
/// when the dependency is available.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// JavaScript undefined
    Undefined,
    /// JavaScript null
    Null,
    /// JavaScript boolean
    Boolean(bool),
    /// JavaScript number (IEEE 754 double)
    Number(f64),
    /// JavaScript string
    String(String),
}

impl Value {
    /// Check if value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Try to get the number value
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Encode value to bytes for serialization
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Value::Undefined => bytes.push(0),
            Value::Null => bytes.push(1),
            Value::Boolean(b) => {
                bytes.push(2);
                bytes.push(if *b { 1 } else { 0 });
            }
            Value::Number(n) => {
                bytes.push(3);
                bytes.extend_from_slice(&n.to_le_bytes());
            }
            Value::String(s) => {
                bytes.push(4);
                let s_bytes = s.as_bytes();
                bytes.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s_bytes);
            }
        }
        bytes
    }

    /// Decode value from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), String> {
        if bytes.is_empty() {
            return Err("Empty bytes".to_string());
        }

        let tag = bytes[0];
        match tag {
            0 => Ok((Value::Undefined, 1)),
            1 => Ok((Value::Null, 1)),
            2 => {
                if bytes.len() < 2 {
                    return Err("Not enough bytes for boolean".to_string());
                }
                Ok((Value::Boolean(bytes[1] != 0), 2))
            }
            3 => {
                if bytes.len() < 9 {
                    return Err("Not enough bytes for number".to_string());
                }
                let n = f64::from_le_bytes(bytes[1..9].try_into().unwrap());
                Ok((Value::Number(n), 9))
            }
            4 => {
                if bytes.len() < 5 {
                    return Err("Not enough bytes for string length".to_string());
                }
                let len = u32::from_le_bytes(bytes[1..5].try_into().unwrap()) as usize;
                if bytes.len() < 5 + len {
                    return Err("Not enough bytes for string content".to_string());
                }
                let s = String::from_utf8(bytes[5..5 + len].to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                Ok((Value::String(s), 5 + len))
            }
            _ => Err(format!("Unknown value tag: {}", tag)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_is_number() {
        assert!(Value::Number(1.0).is_number());
        assert!(!Value::Null.is_number());
    }

    #[test]
    fn test_value_as_number() {
        assert_eq!(Value::Number(42.0).as_number(), Some(42.0));
        assert_eq!(Value::Null.as_number(), None);
    }

    #[test]
    fn test_value_serialize_undefined() {
        let val = Value::Undefined;
        let bytes = val.to_bytes();
        let (restored, _) = Value::from_bytes(&bytes).unwrap();
        assert_eq!(val, restored);
    }

    #[test]
    fn test_value_serialize_null() {
        let val = Value::Null;
        let bytes = val.to_bytes();
        let (restored, _) = Value::from_bytes(&bytes).unwrap();
        assert_eq!(val, restored);
    }

    #[test]
    fn test_value_serialize_boolean() {
        let val = Value::Boolean(true);
        let bytes = val.to_bytes();
        let (restored, _) = Value::from_bytes(&bytes).unwrap();
        assert_eq!(val, restored);
    }

    #[test]
    fn test_value_serialize_number() {
        let val = Value::Number(3.14159);
        let bytes = val.to_bytes();
        let (restored, _) = Value::from_bytes(&bytes).unwrap();
        assert_eq!(val, restored);
    }

    #[test]
    fn test_value_serialize_string() {
        let val = Value::String("hello world".to_string());
        let bytes = val.to_bytes();
        let (restored, _) = Value::from_bytes(&bytes).unwrap();
        assert_eq!(val, restored);
    }
}
