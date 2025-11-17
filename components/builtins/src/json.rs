//! JSON object methods

use crate::value::{JsError, JsResult, JsValue};
use serde_json;

/// JSON object with static methods
pub struct JSONObject;

impl JSONObject {
    /// JSON.parse(text)
    pub fn parse(text: &str) -> JsResult<JsValue> {
        let json_value: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| JsError::syntax_error(format!("JSON parse error: {}", e)))?;

        Self::json_to_js_value(&json_value)
    }

    /// JSON.stringify(value)
    pub fn stringify(value: &JsValue) -> JsResult<String> {
        // Handle undefined specially - it returns the string "undefined", not a JSON string
        if matches!(value, JsValue::Undefined) {
            return Ok("undefined".to_string());
        }

        let json_value = Self::js_value_to_json(value)?;
        let result = serde_json::to_string(&json_value)
            .unwrap_or_else(|_| "null".to_string());

        // Post-process to remove unnecessary decimal points for whole numbers
        Ok(Self::format_json_numbers(&result))
    }

    /// Format JSON string to remove unnecessary .0 from whole numbers
    fn format_json_numbers(json: &str) -> String {
        let mut result = String::with_capacity(json.len());
        let chars: Vec<char> = json.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // Check if we're at a number that might need formatting
            if c.is_ascii_digit() || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
                let start = i;

                // Skip minus sign
                if c == '-' {
                    result.push(c);
                    i += 1;
                }

                // Collect digits before decimal
                while i < chars.len() && chars[i].is_ascii_digit() {
                    result.push(chars[i]);
                    i += 1;
                }

                // Check for decimal point
                if i < chars.len() && chars[i] == '.' {
                    let decimal_start = i;
                    i += 1;

                    // Collect digits after decimal
                    let mut decimal_digits = String::new();
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        decimal_digits.push(chars[i]);
                        i += 1;
                    }

                    // Check for exponent
                    if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                        // Has exponent, keep decimal part
                        result.push('.');
                        result.push_str(&decimal_digits);
                        result.push(chars[i]);
                        i += 1;
                        // Copy rest of exponent
                        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '+' || chars[i] == '-') {
                            result.push(chars[i]);
                            i += 1;
                        }
                    } else if decimal_digits.chars().all(|d| d == '0') {
                        // All zeros after decimal, skip the decimal part
                        // (don't add anything)
                    } else {
                        // Has non-zero decimal digits, keep them
                        result.push('.');
                        result.push_str(&decimal_digits);
                    }
                }
            } else {
                result.push(c);
                i += 1;
            }
        }

        result
    }

    /// Convert serde_json::Value to JsValue
    fn json_to_js_value(json: &serde_json::Value) -> JsResult<JsValue> {
        match json {
            serde_json::Value::Null => Ok(JsValue::null()),
            serde_json::Value::Bool(b) => Ok(JsValue::boolean(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(JsValue::number(i as f64))
                } else if let Some(f) = n.as_f64() {
                    Ok(JsValue::number(f))
                } else {
                    Ok(JsValue::number(0.0))
                }
            }
            serde_json::Value::String(s) => Ok(JsValue::string(s.clone())),
            serde_json::Value::Array(arr) => {
                let elements: Result<Vec<JsValue>, _> = arr
                    .iter()
                    .map(Self::json_to_js_value)
                    .collect();
                Ok(JsValue::array_from(elements?))
            }
            serde_json::Value::Object(obj) => {
                let js_obj = JsValue::object();
                for (key, val) in obj {
                    js_obj.set(key, Self::json_to_js_value(val)?);
                }
                Ok(js_obj)
            }
        }
    }

    /// Convert JsValue to serde_json::Value
    fn js_value_to_json(value: &JsValue) -> JsResult<serde_json::Value> {
        match value {
            JsValue::Undefined => Ok(serde_json::Value::Null), // undefined handled specially in stringify()
            JsValue::Null => Ok(serde_json::Value::Null),
            JsValue::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
            JsValue::Number(n) => {
                if n.is_nan() || n.is_infinite() {
                    Ok(serde_json::Value::Null)
                } else {
                    Ok(serde_json::json!(*n))
                }
            }
            JsValue::String(s) => Ok(serde_json::Value::String(s.clone())),
            JsValue::Array(arr) => {
                let elements: Result<Vec<serde_json::Value>, _> = arr
                    .borrow()
                    .elements
                    .iter()
                    .map(Self::js_value_to_json)
                    .collect();
                Ok(serde_json::Value::Array(elements?))
            }
            JsValue::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (key, val) in &obj.borrow().properties {
                    map.insert(key.clone(), Self::js_value_to_json(val)?);
                }
                Ok(serde_json::Value::Object(map))
            }
            JsValue::Error(err) => {
                // Errors serialize to an object with name, message, and stack
                let error = err.borrow();
                let mut map = serde_json::Map::new();
                map.insert("name".to_string(), serde_json::Value::String(error.name().to_string()));
                map.insert("message".to_string(), serde_json::Value::String(error.message().to_string()));
                map.insert("stack".to_string(), serde_json::Value::String(error.stack()));
                Ok(serde_json::Value::Object(map))
            }
            // Symbols are not serializable in JSON (would be undefined in real JS)
            JsValue::Symbol(_) => Ok(serde_json::Value::Null),
            // Map and Set serialize to empty objects (not JSON-serializable by default)
            JsValue::Map(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            JsValue::Set(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            // RegExp serializes to empty object (not JSON-serializable)
            JsValue::RegExp(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            // Functions are not serializable in JSON (would be undefined in real JS)
            JsValue::Function(_) => Ok(serde_json::Value::Null),
            JsValue::Constructor(_) => Ok(serde_json::Value::Null),
            // Proxy serializes to empty object (behavior depends on handler)
            JsValue::Proxy(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            // WeakMap and WeakSet are not serializable (would be empty object in real JS)
            JsValue::WeakMap(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            JsValue::WeakSet(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            // Generator serializes to empty object
            JsValue::Generator(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            // AsyncGenerator serializes to empty object
            JsValue::AsyncGenerator(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            // BigInt throws in JSON.stringify in real JS, we serialize to string
            JsValue::BigInt(n) => Ok(serde_json::Value::String(format!("{}", n))),
            // WeakRef serializes to empty object
            JsValue::WeakRef(_) => Ok(serde_json::Value::Object(serde_json::Map::new())),
            // FinalizationRegistry serializes to empty object
            JsValue::FinalizationRegistry(_) => {
                Ok(serde_json::Value::Object(serde_json::Map::new()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let result = JSONObject::parse("42").unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_parse_string() {
        let result = JSONObject::parse(r#""hello""#).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");
    }

    #[test]
    fn test_parse_boolean() {
        let result = JSONObject::parse("true").unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = JSONObject::parse("false").unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_parse_null() {
        let result = JSONObject::parse("null").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_parse_array() {
        let result = JSONObject::parse("[1, 2, 3]").unwrap();
        assert!(result.is_array());
        assert_eq!(result.array_length(), 3);
    }

    #[test]
    fn test_parse_object() {
        let result = JSONObject::parse(r#"{"key": "value"}"#).unwrap();
        assert!(result.is_object());
        assert_eq!(result.get("key").unwrap().as_string().unwrap(), "value");
    }

    #[test]
    fn test_parse_invalid() {
        let result = JSONObject::parse("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_stringify_number() {
        let val = JsValue::number(42.0);
        let result = JSONObject::stringify(&val).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_stringify_string() {
        let val = JsValue::string("hello");
        let result = JSONObject::stringify(&val).unwrap();
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_stringify_boolean() {
        let val = JsValue::boolean(true);
        let result = JSONObject::stringify(&val).unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_stringify_null() {
        let val = JsValue::null();
        let result = JSONObject::stringify(&val).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn test_stringify_array() {
        let val = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        let result = JSONObject::stringify(&val).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_stringify_undefined() {
        let val = JsValue::undefined();
        let result = JSONObject::stringify(&val).unwrap();
        assert_eq!(result, "undefined");
    }
}
