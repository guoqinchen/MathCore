//! Arrow Array types and utilities

use arrow2::array::*;
use arrow2::datatypes::*;

/// Arrow array type constants
pub const TYPE_ID_INT64: i32 = 0;
pub const TYPE_ID_FLOAT64: i32 = 1;
pub const TYPE_ID_UTF8: i32 = 2;
pub const TYPE_ID_BINARY: i32 = 3;

/// Create an Int64Array from a Vec<i64>
pub fn new_int64_array(values: Vec<i64>) -> Int64Array {
    PrimitiveArray::from_vec(values)
}

/// Create a Float64Array from a Vec<f64>
pub fn new_float64_array(values: Vec<f64>) -> Float64Array {
    PrimitiveArray::from_vec(values)
}

/// Create a Utf8Array (StringArray) from Vec<&str>
pub fn new_utf8_array(values: Vec<&str>) -> Utf8Array<i32> {
    let mut builder = MutableUtf8Array::<i32>::new();
    builder.extend(values.iter().map(|v| Some(*v)));
    builder.into()
}

/// Create a nullable Utf8Array
pub fn new_nullable_utf8_array(values: Vec<Option<&str>>) -> Utf8Array<i32> {
    let mut builder = MutableUtf8Array::<i32>::new();
    builder.extend(values);
    builder.into()
}

/// Convert ArrayRef to specific array types
pub trait AsAnyArray: Send + Sync {
    fn as_int64(&self) -> Option<&Int64Array>;
    fn as_float64(&self) -> Option<&Float64Array>;
    fn as_utf8(&self) -> Option<&Utf8Array<i32>>;
}

impl AsAnyArray for std::sync::Arc<dyn Array> {
    fn as_int64(&self) -> Option<&Int64Array> {
        self.as_any().downcast_ref()
    }

    fn as_float64(&self) -> Option<&Float64Array> {
        self.as_any().downcast_ref()
    }

    fn as_utf8(&self) -> Option<&Utf8Array<i32>> {
        self.as_any().downcast_ref()
    }
}

/// Get data type name
pub fn type_name(dt: &DataType) -> &'static str {
    match dt {
        DataType::Int8 => "Int8",
        DataType::Int16 => "Int16",
        DataType::Int32 => "Int32",
        DataType::Int64 => "Int64",
        DataType::UInt8 => "UInt8",
        DataType::UInt16 => "UInt16",
        DataType::UInt32 => "UInt32",
        DataType::UInt64 => "UInt64",
        DataType::Float32 => "Float32",
        DataType::Float64 => "Float64",
        DataType::Utf8 => "Utf8",
        DataType::Binary => "Binary",
        DataType::List(_) => "List",
        DataType::Struct(_) => "Struct",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int64_array() {
        let arr = new_int64_array(vec![1, 2, 3]);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.value(0), 1);
        assert_eq!(arr.value(2), 3);
    }

    #[test]
    fn test_float64_array() {
        let arr = new_float64_array(vec![1.0, 2.0, 3.0]);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.value(1), 2.0);
    }

    #[test]
    fn test_utf8_array() {
        let arr = new_utf8_array(vec!["a", "b", "c"]);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.value(0), "a");
    }

    #[test]
    fn test_nullable_utf8_array() {
        let arr = new_nullable_utf8_array(vec![Some("a"), None, Some("c")]);
        assert_eq!(arr.len(), 3);
        assert!(arr.is_null(1));
        assert_eq!(arr.value(2), "c");
    }
}
