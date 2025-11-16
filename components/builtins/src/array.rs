//! Array.prototype methods

use crate::value::{JsError, JsResult, JsValue};

/// Array.prototype methods
pub struct ArrayPrototype;

impl ArrayPrototype {
    /// Array.prototype.push(element)
    pub fn push(arr: &JsValue, element: JsValue) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let mut data = array_data.borrow_mut();
            data.elements.push(element);
            Ok(JsValue::number(data.elements.len() as f64))
        } else {
            Err(JsError::type_error("push called on non-array"))
        }
    }

    /// Array.prototype.pop()
    pub fn pop(arr: &JsValue) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let mut data = array_data.borrow_mut();
            Ok(data.elements.pop().unwrap_or(JsValue::undefined()))
        } else {
            Err(JsError::type_error("pop called on non-array"))
        }
    }

    /// Array.prototype.shift()
    pub fn shift(arr: &JsValue) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let mut data = array_data.borrow_mut();
            if data.elements.is_empty() {
                Ok(JsValue::undefined())
            } else {
                Ok(data.elements.remove(0))
            }
        } else {
            Err(JsError::type_error("shift called on non-array"))
        }
    }

    /// Array.prototype.unshift(element)
    pub fn unshift(arr: &JsValue, element: JsValue) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let mut data = array_data.borrow_mut();
            data.elements.insert(0, element);
            Ok(JsValue::number(data.elements.len() as f64))
        } else {
            Err(JsError::type_error("unshift called on non-array"))
        }
    }

    /// Array.prototype.slice(start, end)
    pub fn slice(arr: &JsValue, start: i32, end: Option<i32>) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            let len = data.elements.len() as i32;

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

            let sliced: Vec<JsValue> = if start_idx < end_idx {
                data.elements[start_idx..end_idx].to_vec()
            } else {
                Vec::new()
            };

            Ok(JsValue::array_from(sliced))
        } else {
            Err(JsError::type_error("slice called on non-array"))
        }
    }

    /// Array.prototype.splice(start, deleteCount, ...items)
    pub fn splice(
        arr: &JsValue,
        start: i32,
        delete_count: usize,
        items: Vec<JsValue>,
    ) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let mut data = array_data.borrow_mut();
            let len = data.elements.len() as i32;

            let start_idx = if start < 0 {
                (len + start).max(0) as usize
            } else {
                start.min(len) as usize
            };

            let actual_delete = delete_count.min(data.elements.len() - start_idx);

            // Remove elements
            let removed: Vec<JsValue> = data.elements.drain(start_idx..start_idx + actual_delete).collect();

            // Insert new elements
            for (i, item) in items.into_iter().enumerate() {
                data.elements.insert(start_idx + i, item);
            }

            Ok(JsValue::array_from(removed))
        } else {
            Err(JsError::type_error("splice called on non-array"))
        }
    }

    /// Array.prototype.map(callback)
    pub fn map<F>(arr: &JsValue, callback: F) -> JsResult<JsValue>
    where
        F: Fn(JsValue) -> JsResult<JsValue>,
    {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            let mut result = Vec::new();
            for element in &data.elements {
                result.push(callback(element.clone())?);
            }
            Ok(JsValue::array_from(result))
        } else {
            Err(JsError::type_error("map called on non-array"))
        }
    }

    /// Array.prototype.filter(callback)
    pub fn filter<F>(arr: &JsValue, callback: F) -> JsResult<JsValue>
    where
        F: Fn(&JsValue) -> JsResult<bool>,
    {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            let mut result = Vec::new();
            for element in &data.elements {
                if callback(element)? {
                    result.push(element.clone());
                }
            }
            Ok(JsValue::array_from(result))
        } else {
            Err(JsError::type_error("filter called on non-array"))
        }
    }

    /// Array.prototype.reduce(callback, initialValue)
    pub fn reduce<F>(arr: &JsValue, initial: JsValue, callback: F) -> JsResult<JsValue>
    where
        F: Fn(JsValue, JsValue) -> JsResult<JsValue>,
    {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            let mut accumulator = initial;
            for element in &data.elements {
                accumulator = callback(accumulator, element.clone())?;
            }
            Ok(accumulator)
        } else {
            Err(JsError::type_error("reduce called on non-array"))
        }
    }

    /// Array.prototype.forEach(callback)
    pub fn for_each<F>(arr: &JsValue, callback: F) -> JsResult<()>
    where
        F: Fn(&JsValue) -> JsResult<()>,
    {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            for element in &data.elements {
                callback(element)?;
            }
            Ok(())
        } else {
            Err(JsError::type_error("forEach called on non-array"))
        }
    }

    /// Array.prototype.find(callback)
    pub fn find<F>(arr: &JsValue, callback: F) -> JsResult<JsValue>
    where
        F: Fn(&JsValue) -> JsResult<bool>,
    {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            for element in &data.elements {
                if callback(element)? {
                    return Ok(element.clone());
                }
            }
            Ok(JsValue::undefined())
        } else {
            Err(JsError::type_error("find called on non-array"))
        }
    }

    /// Array.prototype.findIndex(callback)
    pub fn find_index<F>(arr: &JsValue, callback: F) -> JsResult<i32>
    where
        F: Fn(&JsValue) -> JsResult<bool>,
    {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            for (i, element) in data.elements.iter().enumerate() {
                if callback(element)? {
                    return Ok(i as i32);
                }
            }
            Ok(-1)
        } else {
            Err(JsError::type_error("findIndex called on non-array"))
        }
    }

    /// Array.prototype.includes(value)
    pub fn includes(arr: &JsValue, value: &JsValue) -> bool {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            data.elements.iter().any(|e| e.equals(value))
        } else {
            false
        }
    }

    /// Array.prototype.indexOf(value)
    pub fn index_of(arr: &JsValue, value: &JsValue) -> Option<usize> {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            data.elements.iter().position(|e| e.equals(value))
        } else {
            None
        }
    }

    /// Array.prototype.join(separator)
    pub fn join(arr: &JsValue, separator: &str) -> JsResult<String> {
        if let JsValue::Array(array_data) = arr {
            let data = array_data.borrow();
            let strings: Vec<String> = data.elements.iter().map(|e| e.to_js_string()).collect();
            Ok(strings.join(separator))
        } else {
            Err(JsError::type_error("join called on non-array"))
        }
    }

    /// Array.prototype.sort()
    pub fn sort(arr: &JsValue) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let mut data = array_data.borrow_mut();
            // Sort by string conversion (default JS behavior)
            data.elements.sort_by(|a, b| {
                a.to_js_string().cmp(&b.to_js_string())
            });
            Ok(arr.clone())
        } else {
            Err(JsError::type_error("sort called on non-array"))
        }
    }

    /// Array.prototype.reverse()
    pub fn reverse(arr: &JsValue) -> JsResult<JsValue> {
        if let JsValue::Array(array_data) = arr {
            let mut data = array_data.borrow_mut();
            data.elements.reverse();
            Ok(arr.clone())
        } else {
            Err(JsError::type_error("reverse called on non-array"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let arr = JsValue::array();
        ArrayPrototype::push(&arr, JsValue::number(1.0)).unwrap();
        assert_eq!(arr.array_length(), 1);
    }

    #[test]
    fn test_pop() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        let result = ArrayPrototype::pop(&arr).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);
        assert_eq!(arr.array_length(), 1);
    }

    #[test]
    fn test_shift() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        let result = ArrayPrototype::shift(&arr).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
        assert_eq!(arr.array_length(), 1);
    }

    #[test]
    fn test_unshift() {
        let arr = JsValue::array_from(vec![JsValue::number(2.0)]);
        ArrayPrototype::unshift(&arr, JsValue::number(1.0)).unwrap();
        assert_eq!(arr.array_length(), 2);
    }

    #[test]
    fn test_slice() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);
        let sliced = ArrayPrototype::slice(&arr, 1, Some(3)).unwrap();
        assert_eq!(sliced.array_length(), 2);
    }

    #[test]
    fn test_map() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        let result = ArrayPrototype::map(&arr, |v| {
            Ok(JsValue::number(v.as_number().unwrap() * 2.0))
        })
        .unwrap();
        assert_eq!(result.array_length(), 2);
    }

    #[test]
    fn test_filter() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);
        let result = ArrayPrototype::filter(&arr, |v| Ok(v.as_number().unwrap() > 1.0)).unwrap();
        assert_eq!(result.array_length(), 2);
    }

    #[test]
    fn test_reduce() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);
        let result = ArrayPrototype::reduce(&arr, JsValue::number(0.0), |acc, v| {
            Ok(JsValue::number(
                acc.as_number().unwrap() + v.as_number().unwrap(),
            ))
        })
        .unwrap();
        assert_eq!(result.as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_includes() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        assert!(ArrayPrototype::includes(&arr, &JsValue::number(2.0)));
        assert!(!ArrayPrototype::includes(&arr, &JsValue::number(3.0)));
    }

    #[test]
    fn test_index_of() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        assert_eq!(
            ArrayPrototype::index_of(&arr, &JsValue::number(2.0)),
            Some(1)
        );
        assert_eq!(ArrayPrototype::index_of(&arr, &JsValue::number(3.0)), None);
    }

    #[test]
    fn test_join() {
        let arr = JsValue::array_from(vec![
            JsValue::string("a"),
            JsValue::string("b"),
            JsValue::string("c"),
        ]);
        let result = ArrayPrototype::join(&arr, ",").unwrap();
        assert_eq!(result, "a,b,c");
    }
}
