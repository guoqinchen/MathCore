//! Schema definitions for Arrow data plane

use arrow2::datatypes::*;

/// Common field names in math computations
pub mod field_names {
    pub const ID: &str = "id";
    pub const VALUE: &str = "value";
    pub const LABEL: &str = "label";
    pub const EXPRESSION: &str = "expression";
    pub const RESULT: &str = "result";
    pub const COMPUTE_TIME_US: &str = "compute_time_us";
    pub const TIMESTAMP: &str = "timestamp";
    pub const X: &str = "x";
    pub const Y: &str = "y";
    pub const Z: &str = "z";
}

/// Standard data types
pub mod data_types {
    use arrow2::datatypes::*;

    pub fn int64() -> DataType {
        DataType::Int64
    }
    pub fn float64() -> DataType {
        DataType::Float64
    }
    pub fn utf8() -> DataType {
        DataType::Utf8
    }
    pub fn bool_() -> DataType {
        DataType::Boolean
    }
}

/// Create a standard compute result schema
pub fn compute_result_schema() -> Schema {
    Schema::from(vec![
        Field::new(field_names::ID, DataType::Int64, false),
        Field::new(field_names::VALUE, DataType::Float64, false),
        Field::new(field_names::LABEL, DataType::Utf8, true),
        Field::new(field_names::RESULT, DataType::Utf8, false),
        Field::new(field_names::COMPUTE_TIME_US, DataType::UInt64, true),
    ])
}

/// Create a time series schema
pub fn time_series_schema() -> Schema {
    Schema::from(vec![
        Field::new(field_names::TIMESTAMP, DataType::Int64, false),
        Field::new(field_names::VALUE, DataType::Float64, false),
        Field::new(field_names::LABEL, DataType::Utf8, true),
    ])
}

/// Create a 2D point schema
pub fn point2d_schema() -> Schema {
    Schema::from(vec![
        Field::new(field_names::X, DataType::Float64, false),
        Field::new(field_names::Y, DataType::Float64, false),
    ])
}

/// Create a 3D point schema
pub fn point3d_schema() -> Schema {
    Schema::from(vec![
        Field::new(field_names::X, DataType::Float64, false),
        Field::new(field_names::Y, DataType::Float64, false),
        Field::new("z", DataType::Float64, false),
    ])
}

/// Create a batch metadata schema
pub fn batch_metadata_schema() -> Schema {
    Schema::from(vec![
        Field::new("batch_id", DataType::Utf8, false),
        Field::new("num_rows", DataType::UInt32, false),
        Field::new("created_at", DataType::UInt64, false),
    ])
}

/// Schema builder for custom schemas
pub struct SchemaBuilder {
    fields: Vec<Field>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add_field(mut self, name: impl Into<String>, dtype: DataType, nullable: bool) -> Self {
        self.fields.push(Field::new(name, dtype, nullable));
        self
    }

    pub fn add_int64_field(mut self, name: impl Into<String>, nullable: bool) -> Self {
        self.fields
            .push(Field::new(name, DataType::Int64, nullable));
        self
    }

    pub fn add_float64_field(mut self, name: impl Into<String>, nullable: bool) -> Self {
        self.fields
            .push(Field::new(name, DataType::Float64, nullable));
        self
    }

    pub fn add_utf8_field(mut self, name: impl Into<String>, nullable: bool) -> Self {
        self.fields.push(Field::new(name, DataType::Utf8, nullable));
        self
    }

    pub fn build(self) -> Schema {
        Schema::from(self.fields)
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert schema to JSON for debugging
pub fn schema_to_json(schema: &Schema) -> String {
    let fields: Vec<String> = schema
        .fields
        .iter()
        .map(|f| {
            format!(
                r#"{{"name":"{}","dtype":"{}","nullable":{}}}"#,
                f.name,
                format!("{:?}", f.data_type),
                f.is_nullable
            )
        })
        .collect();
    format!(r#"{{"fields":[{}]"#, fields.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builder() {
        let schema = SchemaBuilder::new()
            .add_int64_field("id", false)
            .add_float64_field("value", false)
            .add_utf8_field("name", true)
            .build();

        assert_eq!(schema.fields.len(), 3);
    }

    #[test]
    fn test_compute_result_schema() {
        let schema = compute_result_schema();
        assert_eq!(schema.fields.len(), 5);
    }

    #[test]
    fn test_time_series_schema() {
        let schema = time_series_schema();
        assert_eq!(schema.fields.len(), 3);
    }
}
